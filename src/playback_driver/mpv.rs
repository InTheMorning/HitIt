//! mpv JSON IPC playback driver.

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use std::{error::Error, fmt};

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};

use super::{DriverStatus, PlaybackDriver};

const SOCKET_READY_TIMEOUT: Duration = Duration::from_secs(3);
const IPC_TIMEOUT: Duration = Duration::from_secs(2);
/// The longest time that one command waits for its reply. A read timeout of
/// `IPC_TIMEOUT` is not an error before this time ends (ADR 0068 task 001).
const REPLY_DEADLINE: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub(crate) struct MpvNotInstalled {
    pub(crate) path: PathBuf,
}

impl fmt::Display for MpvNotInstalled {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}{}",
            super::MPV_NOT_INSTALLED_PREFIX,
            self.path.display()
        )
    }
}

impl Error for MpvNotInstalled {}

#[derive(Debug)]
pub struct MpvDriver {
    mpv_path: PathBuf,
    runtime_dir: PathBuf,
    inner: Mutex<MpvState>,
}

#[derive(Debug)]
struct MpvState {
    child: Option<Child>,
    socket_path: Option<PathBuf>,
    stream: Option<BufReader<UnixStream>>,
    next_request_id: u64,
    status: DriverStatus,
}

impl MpvDriver {
    pub fn new(mpv_path: impl Into<PathBuf>) -> Result<Self> {
        Self::with_runtime_dir(mpv_path, default_runtime_dir()?)
    }

    /// Makes a driver whose runtime folder is `name` below the default runtime folder.
    pub(crate) fn new_isolated(mpv_path: impl Into<PathBuf>, name: &str) -> Result<Self> {
        Self::with_runtime_dir(mpv_path, default_runtime_dir()?.join(name))
    }

    pub fn with_runtime_dir(mpv_path: impl Into<PathBuf>, runtime_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&runtime_dir)
            .with_context(|| format!("create mpv runtime dir {}", runtime_dir.display()))?;
        fs::set_permissions(&runtime_dir, fs::Permissions::from_mode(0o700))
            .with_context(|| format!("secure mpv runtime dir {}", runtime_dir.display()))?;
        Ok(Self {
            mpv_path: mpv_path.into(),
            runtime_dir,
            inner: Mutex::new(MpvState {
                child: None,
                socket_path: None,
                stream: None,
                next_request_id: 1,
                status: DriverStatus::default(),
            }),
        })
    }

    pub fn ping(&self) -> Result<()> {
        let mut state = self.inner.lock().expect("mpv driver mutex");
        self.ensure_started(&mut state)?;
        self.command(&mut state, json!(["get_property", "mpv-version"]))
            .map(|_| ())
    }

    fn ensure_started(&self, state: &mut MpvState) -> Result<()> {
        if state.stream.is_some() {
            return Ok(());
        }

        let socket_path = self.socket_path();
        remove_stale_socket(&socket_path)?;
        let mut child = Command::new(&self.mpv_path)
            .arg("--idle")
            .arg("--no-video")
            .arg(format!("--input-ipc-server={}", socket_path.display()))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    MpvNotInstalled {
                        path: self.mpv_path.clone(),
                    }
                    .into()
                } else {
                    anyhow!(error).context(format!("spawn mpv at {}", self.mpv_path.display()))
                }
            })?;

        let stream = match wait_for_socket(&socket_path) {
            Ok(stream) => stream,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error);
            }
        };
        stream
            .set_read_timeout(Some(IPC_TIMEOUT))
            .context("set mpv IPC read timeout")?;
        stream
            .set_write_timeout(Some(IPC_TIMEOUT))
            .context("set mpv IPC write timeout")?;

        state.child = Some(child);
        state.socket_path = Some(socket_path);
        state.stream = Some(BufReader::new(stream));
        self.observe_property(state, 1, "time-pos")?;
        self.observe_property(state, 2, "pause")?;
        state.status = DriverStatus::default();
        Ok(())
    }

    fn observe_property(&self, state: &mut MpvState, observer_id: u64, name: &str) -> Result<()> {
        self.command(state, json!(["observe_property", observer_id, name]))
            .map(|_| ())
    }

    fn command(&self, state: &mut MpvState, command: Value) -> Result<Option<Value>> {
        self.command_until(state, command, Instant::now() + REPLY_DEADLINE)
    }

    fn command_until(
        &self,
        state: &mut MpvState,
        command: Value,
        deadline: Instant,
    ) -> Result<Option<Value>> {
        let request_id = state.next_request_id;
        state.next_request_id = state
            .next_request_id
            .checked_add(1)
            .context("mpv request id overflow")?;
        let request = json!({
            "command": command,
            "request_id": request_id,
        });
        let line = serde_json::to_vec(&request).context("serialize mpv command")?;
        let stream = state
            .stream
            .as_mut()
            .context("mpv IPC stream is not open")?;
        stream
            .get_mut()
            .write_all(&line)
            .context("write mpv command")?;
        stream
            .get_mut()
            .write_all(b"\n")
            .context("write mpv command newline")?;
        stream.get_mut().flush().context("flush mpv command")?;

        loop {
            let message = read_message(stream, deadline)?;
            handle_event(&mut state.status, &message);
            if message
                .get("request_id")
                .and_then(Value::as_u64)
                .is_some_and(|id| id == request_id)
            {
                let error = message
                    .get("error")
                    .and_then(Value::as_str)
                    .unwrap_or("missing error field");
                if error != "success" {
                    return Err(anyhow!("mpv command failed: {error}"));
                }
                return Ok(message.get("data").cloned());
            }
        }
    }

    fn socket_path(&self) -> PathBuf {
        self.runtime_dir
            .join(format!("mpv-{}.sock", std::process::id()))
    }

    pub fn shutdown(&self) {
        let _ = self.shutdown_for_maintenance();
    }

    /// Stop and reap only this driver's child, retaining failed ownership for retry (ADR 0066).
    pub(crate) fn shutdown_for_maintenance(&self) -> Result<()> {
        let mut state = self
            .inner
            .lock()
            .map_err(|_| anyhow!("mpv driver mutex poisoned"))?;
        let _ = send_quit(&mut state);
        if let Some(child) = state.child.as_mut() {
            let grace = Instant::now() + Duration::from_millis(500);
            let deadline = grace + Duration::from_millis(500);
            let mut killed = false;
            loop {
                if child
                    .try_wait()
                    .context("wait for app-owned mpv child")?
                    .is_some()
                {
                    break;
                }
                if Instant::now() >= deadline {
                    return Err(anyhow!(
                        "app-owned mpv child has not exited; retry session drain"
                    ));
                }
                if !killed && Instant::now() >= grace {
                    child.kill().context("stop app-owned mpv child")?;
                    killed = true;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        }
        state.child = None;
        state.stream = None;
        if let Some(path) = &state.socket_path {
            match fs::remove_file(path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error).context("remove app-owned mpv IPC socket"),
            }
        }
        state.socket_path = None;
        Ok(())
    }
}

impl PlaybackDriver for MpvDriver {
    fn load(&self, path: &Path, start_ms: u64) -> Result<()> {
        let mut state = self.inner.lock().expect("mpv driver mutex");
        self.ensure_started(&mut state)?;
        state.status.eof = false;
        // mpv keeps its pause property from the file before. A load starts playback.
        self.command(&mut state, json!(["set_property", "pause", false]))?;
        self.command(&mut state, json!(["loadfile", path.display().to_string()]))?;
        if start_ms > 0 {
            let seconds = start_ms as f64 / 1000.0;
            self.command(&mut state, json!(["seek", seconds, "absolute"]))?;
        }
        state.status.position_ms = start_ms;
        state.status.paused = false;
        Ok(())
    }

    fn seek(&self, position_ms: u64) -> Result<()> {
        let mut state = self.inner.lock().expect("mpv driver mutex");
        self.ensure_started(&mut state)?;
        let seconds = position_ms as f64 / 1000.0;
        self.command(&mut state, json!(["seek", seconds, "absolute"]))?;
        state.status.position_ms = position_ms;
        state.status.eof = false;
        Ok(())
    }

    fn pause(&self, paused: bool) -> Result<()> {
        let mut state = self.inner.lock().expect("mpv driver mutex");
        self.ensure_started(&mut state)?;
        self.command(&mut state, json!(["set_property", "pause", paused]))?;
        state.status.paused = paused;
        Ok(())
    }

    fn stop(&self) -> Result<()> {
        let mut state = self.inner.lock().expect("mpv driver mutex");
        if state.stream.is_none() {
            return Ok(());
        }
        self.command(&mut state, json!(["stop"]))?;
        state.status = DriverStatus::default();
        Ok(())
    }

    fn poll(&self) -> Result<DriverStatus> {
        let mut state = self.inner.lock().expect("mpv driver mutex");
        if state.stream.is_none() {
            return Ok(state.status.clone());
        }
        if let Some(value) = self.command(&mut state, json!(["get_property", "time-pos"]))? {
            if let Some(seconds) = value.as_f64() {
                state.status.position_ms = seconds_to_ms(seconds);
            }
        }
        if let Some(value) = self.command(&mut state, json!(["get_property", "pause"]))? {
            if let Some(paused) = value.as_bool() {
                state.status.paused = paused;
            }
        }
        Ok(state.status.clone())
    }
}

impl Drop for MpvDriver {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn read_message(stream: &mut BufReader<UnixStream>, deadline: Instant) -> Result<Value> {
    // `read_until` keeps each byte that it read before a timeout, so a reply
    // that arrives in parts, or a character split by a timeout, stays whole.
    let mut line = Vec::new();
    loop {
        match stream.read_until(b'\n', &mut line) {
            Ok(0) => anyhow::bail!("mpv IPC socket closed"),
            Ok(_) if line.ends_with(b"\n") => break,
            Ok(_) => anyhow::bail!("mpv IPC socket closed"),
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                anyhow::ensure!(
                    Instant::now() < deadline,
                    "mpv gave no IPC reply in {} s",
                    REPLY_DEADLINE.as_secs()
                );
            }
            Err(error) => return Err(error).context("read mpv IPC message"),
        }
    }
    serde_json::from_slice(line.trim_ascii_end()).context("parse mpv IPC message")
}

fn send_quit(state: &mut MpvState) -> Result<()> {
    let request_id = state.next_request_id;
    state.next_request_id = state
        .next_request_id
        .checked_add(1)
        .context("mpv request id overflow")?;
    let request = json!({
        "command": ["quit"],
        "request_id": request_id,
    });
    let line = serde_json::to_vec(&request).context("serialize mpv quit command")?;
    let Some(stream) = state.stream.as_mut() else {
        return Ok(());
    };
    stream
        .get_mut()
        .write_all(&line)
        .context("write mpv quit command")?;
    stream
        .get_mut()
        .write_all(b"\n")
        .context("write mpv quit command newline")?;
    stream.get_mut().flush().context("flush mpv quit command")
}

fn handle_event(status: &mut DriverStatus, message: &Value) {
    match message.get("event").and_then(Value::as_str) {
        Some("property-change") => handle_property_change(status, message),
        Some("end-file") => {
            // mpv also sends this event when a command stops or replaces a file.
            // Only an end that no command caused is the end of the track.
            let reason = message.get("reason").and_then(Value::as_str);
            if !matches!(reason, Some("stop" | "quit" | "redirect")) {
                status.eof = true;
            }
        }
        _ => {}
    }
}

fn handle_property_change(status: &mut DriverStatus, message: &Value) {
    match message.get("name").and_then(Value::as_str) {
        Some("time-pos") => {
            if let Some(seconds) = message.get("data").and_then(Value::as_f64) {
                status.position_ms = seconds_to_ms(seconds);
            }
        }
        Some("pause") => {
            if let Some(paused) = message.get("data").and_then(Value::as_bool) {
                status.paused = paused;
            }
        }
        _ => {}
    }
}

fn seconds_to_ms(seconds: f64) -> u64 {
    if seconds.is_sign_negative() {
        return 0;
    }
    (seconds * 1000.0).round() as u64
}

fn wait_for_socket(socket_path: &Path) -> Result<UnixStream> {
    let deadline = Instant::now() + SOCKET_READY_TIMEOUT;
    loop {
        match UnixStream::connect(socket_path) {
            Ok(stream) => return Ok(stream),
            Err(error) if Instant::now() < deadline => {
                if error.kind() != std::io::ErrorKind::NotFound
                    && error.kind() != std::io::ErrorKind::ConnectionRefused
                {
                    return Err(error).context("connect mpv IPC socket");
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(error) => return Err(error).context("connect mpv IPC socket before timeout"),
        }
    }
}

fn remove_stale_socket(socket_path: &Path) -> Result<()> {
    if !socket_path.exists() {
        return Ok(());
    }
    match UnixStream::connect(socket_path) {
        Ok(_) => Err(anyhow!(
            "mpv IPC socket already active at {}",
            socket_path.display()
        )),
        Err(_) => fs::remove_file(socket_path)
            .with_context(|| format!("remove stale mpv socket {}", socket_path.display())),
    }
}

fn default_runtime_dir() -> Result<PathBuf> {
    let base = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("hitit"));
    Ok(base.join("mpv"))
}

#[cfg(test)]
mod tests {
    use std::thread;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn missing_mpv_error_names_the_program_and_attempted_path() -> Result<()> {
        let runtime = tempdir()?;
        let mpv_path = runtime.path().join("missing-mpv");
        let driver = MpvDriver::with_runtime_dir(&mpv_path, runtime.path().join("socket"))?;

        let error = driver.ping().expect_err("missing mpv must fail to start");

        assert!(error.to_string().contains("mpv is not installed"));
        assert!(error.to_string().contains(&mpv_path.display().to_string()));
        Ok(())
    }

    #[test]
    fn adr_0066_shutdown_reaps_only_the_app_owned_child_without_mpv() {
        let driver = test_driver();
        let owned = Command::new("sleep").arg("30").spawn().unwrap();
        let mut independent = Command::new("sleep").arg("30").spawn().unwrap();
        driver.inner.lock().unwrap().child = Some(owned);
        let result = driver.shutdown_for_maintenance();
        let independent_running = independent.try_wait().unwrap().is_none();
        independent.kill().unwrap();
        independent.wait().unwrap();
        result.unwrap();
        assert!(driver.inner.lock().unwrap().child.is_none());
        assert!(independent_running, "unowned processes must remain running");
    }

    fn test_state(stream: UnixStream) -> MpvState {
        MpvState {
            child: None,
            socket_path: None,
            stream: Some(BufReader::new(stream)),
            next_request_id: 1,
            status: DriverStatus::default(),
        }
    }

    fn test_driver() -> MpvDriver {
        MpvDriver {
            mpv_path: PathBuf::from("mpv"),
            runtime_dir: std::env::temp_dir(),
            inner: Mutex::new(MpvState {
                child: None,
                socket_path: None,
                stream: None,
                next_request_id: 1,
                status: DriverStatus::default(),
            }),
        }
    }

    fn write_message(stream: &mut BufReader<UnixStream>, message: Value) -> Result<()> {
        let line = serde_json::to_vec(&message).context("serialize fake mpv message")?;
        stream
            .get_mut()
            .write_all(&line)
            .context("write fake mpv message")?;
        stream
            .get_mut()
            .write_all(b"\n")
            .context("write fake mpv newline")?;
        stream.get_mut().flush().context("flush fake mpv message")
    }

    fn read_request(stream: &mut BufReader<UnixStream>) -> Result<Value> {
        let mut line = String::new();
        stream
            .read_line(&mut line)
            .context("read fake mpv request")?;
        serde_json::from_str(line.trim_end()).context("parse fake mpv request")
    }

    fn run_fake_peer(
        peer: UnixStream,
        scenario: impl FnOnce(&mut BufReader<UnixStream>) -> Result<()> + Send + 'static,
    ) -> thread::JoinHandle<Result<()>> {
        thread::spawn(move || {
            let mut stream = BufReader::new(peer);
            scenario(&mut stream)
        })
    }

    fn join_fake_peer(handle: thread::JoinHandle<Result<()>>) -> Result<()> {
        handle.join().map_err(|payload| {
            if let Some(message) = payload.downcast_ref::<&str>() {
                anyhow!("fake mpv peer panicked: {message}")
            } else if let Some(message) = payload.downcast_ref::<String>() {
                anyhow!("fake mpv peer panicked: {message}")
            } else {
                anyhow!("fake mpv peer panicked")
            }
        })?
    }

    #[test]
    fn load_clears_the_pause_of_the_file_before() -> Result<()> {
        let (driver_stream, peer_stream) = UnixStream::pair().context("create fake IPC pair")?;
        let driver = test_driver();
        *driver.inner.lock().expect("mpv driver mutex") = test_state(driver_stream);
        let peer = run_fake_peer(peer_stream, |stream| {
            let first = read_request(stream)?;
            assert_eq!(first["command"], json!(["set_property", "pause", false]));
            write_message(stream, json!({"request_id": 1, "error": "success"}))?;

            let second = read_request(stream)?;
            assert_eq!(second["command"], json!(["loadfile", "/music/a.flac"]));
            // mpv reports the end of the replaced file after the load.
            write_message(stream, json!({"event": "end-file", "reason": "stop"}))?;
            write_message(stream, json!({"request_id": 2, "error": "success"}))
        });

        driver.load(Path::new("/music/a.flac"), 0)?;
        join_fake_peer(peer)?;

        let status = driver
            .inner
            .lock()
            .expect("mpv driver mutex")
            .status
            .clone();
        assert!(!status.paused);
        assert!(!status.eof);
        Ok(())
    }

    #[test]
    fn command_writes_monotonically_increasing_request_ids() -> Result<()> {
        let (driver_stream, peer_stream) = UnixStream::pair().context("create fake IPC pair")?;
        let driver = test_driver();
        let mut state = test_state(driver_stream);
        let peer = run_fake_peer(peer_stream, |stream| {
            let first = read_request(stream)?;
            assert_eq!(first["command"], json!(["get_property", "time-pos"]));
            assert_eq!(first["request_id"], 1);
            write_message(stream, json!({"request_id": 1, "error": "success"}))?;

            let second = read_request(stream)?;
            assert_eq!(second["command"], json!(["get_property", "pause"]));
            assert_eq!(second["request_id"], 2);
            write_message(stream, json!({"request_id": 2, "error": "success"}))
        });

        driver.command(&mut state, json!(["get_property", "time-pos"]))?;
        driver.command(&mut state, json!(["get_property", "pause"]))?;
        join_fake_peer(peer)?;

        assert_eq!(state.next_request_id, 3);
        Ok(())
    }

    #[test]
    fn command_ignores_interleaved_events_and_returns_matching_data() -> Result<()> {
        let (driver_stream, peer_stream) = UnixStream::pair().context("create fake IPC pair")?;
        let driver = test_driver();
        let mut state = test_state(driver_stream);
        let peer = run_fake_peer(peer_stream, |stream| {
            let request = read_request(stream)?;
            assert_eq!(request["request_id"], 1);
            write_message(
                stream,
                json!({
                    "event": "property-change",
                    "name": "time-pos",
                    "data": 4.2
                }),
            )?;
            write_message(
                stream,
                json!({
                    "event": "property-change",
                    "name": "pause",
                    "data": true
                }),
            )?;
            write_message(stream, json!({"request_id": 99, "error": "success"}))?;
            write_message(
                stream,
                json!({"request_id": 1, "error": "success", "data": "ok"}),
            )
        });

        let data = driver.command(&mut state, json!(["get_property", "path"]))?;
        join_fake_peer(peer)?;

        assert_eq!(data, Some(json!("ok")));
        assert_eq!(state.status.position_ms, 4_200);
        assert!(state.status.paused);
        Ok(())
    }

    #[test]
    fn command_retries_after_read_timeout() -> Result<()> {
        let (driver_stream, peer_stream) = UnixStream::pair().context("create fake IPC pair")?;
        driver_stream
            .set_read_timeout(Some(Duration::from_millis(20)))
            .context("set fake IPC read timeout")?;
        let driver = test_driver();
        let mut state = test_state(driver_stream);
        let peer = run_fake_peer(peer_stream, |stream| {
            let request = read_request(stream)?;
            thread::sleep(Duration::from_millis(50));
            write_message(
                stream,
                json!({"request_id": request["request_id"], "error": "success"}),
            )
        });

        driver.command(&mut state, json!(["get_property", "time-pos"]))?;
        join_fake_peer(peer)?;
        Ok(())
    }

    #[test]
    fn command_keeps_partial_message_after_read_timeout() -> Result<()> {
        let (driver_stream, peer_stream) = UnixStream::pair().context("create fake IPC pair")?;
        driver_stream
            .set_read_timeout(Some(Duration::from_millis(20)))
            .context("set fake IPC read timeout")?;
        let driver = test_driver();
        let mut state = test_state(driver_stream);
        let peer = run_fake_peer(peer_stream, |stream| {
            let request = read_request(stream)?;
            let response = format!(
                "{{\"request_id\":{},\"error\":\"success\"}}",
                request["request_id"]
            );
            stream
                .get_mut()
                .write_all(&response.as_bytes()[..8])
                .context("write fake mpv response prefix")?;
            stream.get_mut().flush().context("flush fake mpv prefix")?;
            thread::sleep(Duration::from_millis(50));
            stream
                .get_mut()
                .write_all(&response.as_bytes()[8..])
                .context("write fake mpv response suffix")?;
            stream
                .get_mut()
                .write_all(b"\n")
                .context("write fake mpv response newline")?;
            stream.get_mut().flush().context("flush fake mpv response")
        });

        driver.command(&mut state, json!(["get_property", "time-pos"]))?;
        join_fake_peer(peer)?;
        Ok(())
    }

    #[test]
    fn command_reports_closed_socket_after_read_timeout() -> Result<()> {
        let (driver_stream, peer_stream) = UnixStream::pair().context("create fake IPC pair")?;
        driver_stream
            .set_read_timeout(Some(Duration::from_millis(20)))
            .context("set fake IPC read timeout")?;
        let driver = test_driver();
        let mut state = test_state(driver_stream);
        let peer = run_fake_peer(peer_stream, |stream| {
            let request = read_request(stream)?;
            assert_eq!(request["request_id"], 1);
            Ok(())
        });

        let error = driver
            .command(&mut state, json!(["get_property", "time-pos"]))
            .expect_err("closed mpv socket must fail the command");
        join_fake_peer(peer)?;

        assert!(error.to_string().contains("mpv IPC socket closed"));
        Ok(())
    }

    #[test]
    fn command_reports_missing_reply_after_deadline() -> Result<()> {
        let (driver_stream, peer_stream) = UnixStream::pair().context("create fake IPC pair")?;
        driver_stream
            .set_read_timeout(Some(Duration::from_millis(20)))
            .context("set fake IPC read timeout")?;
        let driver = test_driver();
        let mut state = test_state(driver_stream);
        let peer = run_fake_peer(peer_stream, |stream| {
            read_request(stream)?;
            thread::sleep(Duration::from_millis(400));
            Ok(())
        });

        let started = Instant::now();
        let error = driver
            .command_until(
                &mut state,
                json!(["get_property", "time-pos"]),
                Instant::now() + Duration::from_millis(100),
            )
            .expect_err("a silent mpv must fail the command at the deadline");
        assert!(started.elapsed() < Duration::from_millis(400));
        join_fake_peer(peer)?;

        assert!(error.to_string().contains("mpv gave no IPC reply"));
        Ok(())
    }

    #[test]
    fn command_reports_mpv_error_response() -> Result<()> {
        let (driver_stream, peer_stream) = UnixStream::pair().context("create fake IPC pair")?;
        let driver = test_driver();
        let mut state = test_state(driver_stream);
        let peer = run_fake_peer(peer_stream, |stream| {
            let request = read_request(stream)?;
            assert_eq!(request["request_id"], 1);
            write_message(
                stream,
                json!({"request_id": 1, "error": "property unavailable"}),
            )
        });

        let error = driver
            .command(&mut state, json!(["get_property", "missing"]))
            .expect_err("mpv error response should fail command");
        join_fake_peer(peer)?;

        assert_eq!(
            error.to_string(),
            "mpv command failed: property unavailable"
        );
        Ok(())
    }

    #[test]
    fn property_events_update_cached_status() {
        let mut status = DriverStatus::default();

        handle_event(
            &mut status,
            &json!({
                "event": "property-change",
                "name": "time-pos",
                "data": 12.345
            }),
        );
        handle_event(
            &mut status,
            &json!({
                "event": "property-change",
                "name": "pause",
                "data": true
            }),
        );

        assert_eq!(status.position_ms, 12_345);
        assert!(status.paused);
    }

    #[test]
    fn end_file_event_sets_eof() {
        let mut status = DriverStatus::default();

        handle_event(&mut status, &json!({"event": "end-file"}));

        assert!(status.eof);
    }

    #[test]
    fn end_file_event_of_a_stopped_or_replaced_file_is_not_eof() {
        for reason in ["stop", "quit", "redirect"] {
            let mut status = DriverStatus::default();

            handle_event(&mut status, &json!({"event": "end-file", "reason": reason}));

            assert!(!status.eof, "reason {reason}");
        }
        let mut status = DriverStatus::default();
        handle_event(&mut status, &json!({"event": "end-file", "reason": "eof"}));
        assert!(status.eof);
    }

    #[test]
    fn negative_seconds_clamp_to_zero_ms() {
        assert_eq!(seconds_to_ms(-1.0), 0);
    }
}
