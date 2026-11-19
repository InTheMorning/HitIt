//! Plays one tone through a driver instance and stops it (ADR 0104 Decision 4).
//!
//! The function takes a driver and a file. It has no access to the Show session,
//! the Show cue or the drop file.

use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{bail, Result};

use super::PlaybackDriver;

/// The longest wait for the end of the test sound.
pub(crate) const TEST_SOUND_TIMEOUT: Duration = Duration::from_secs(5);

/// The time between two polls of the driver.
pub(crate) const TEST_SOUND_POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Loads the file, waits for EOF or for `timeout`, then stops the driver.
///
/// # Errors
///
/// Returns an error when the driver cannot load the file or reports an error.
/// The driver gets a stop call in each case.
pub(crate) fn play<D: PlaybackDriver>(
    driver: &D,
    tone: &Path,
    timeout: Duration,
    poll_interval: Duration,
) -> Result<()> {
    let result = wait_for_end(driver, tone, timeout, poll_interval);
    let stopped = driver.stop();
    result?;
    stopped
}

fn wait_for_end<D: PlaybackDriver>(
    driver: &D,
    tone: &Path,
    timeout: Duration,
    poll_interval: Duration,
) -> Result<()> {
    driver.load(tone, 0)?;
    let deadline = Instant::now() + timeout;
    loop {
        let status = driver.poll()?;
        if let Some(error) = status.error {
            bail!("The player reported an error: {error}");
        }
        if status.eof || Instant::now() >= deadline {
            return Ok(());
        }
        std::thread::sleep(poll_interval);
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Mutex;

    use super::*;
    use crate::playback_driver::DriverStatus;

    #[derive(Default)]
    struct FakeDriver {
        calls: Mutex<Vec<String>>,
        eof_after_polls: usize,
        fail_load: bool,
    }

    impl FakeDriver {
        fn calls(&self) -> Vec<String> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl PlaybackDriver for FakeDriver {
        fn load(&self, path: &Path, _: u64) -> Result<()> {
            self.calls
                .lock()
                .unwrap()
                .push(format!("load {}", path.display()));
            if self.fail_load {
                bail!("cannot load");
            }
            Ok(())
        }
        fn seek(&self, _: u64) -> Result<()> {
            self.calls.lock().unwrap().push("seek".into());
            Ok(())
        }
        fn pause(&self, _: bool) -> Result<()> {
            self.calls.lock().unwrap().push("pause".into());
            Ok(())
        }
        fn stop(&self) -> Result<()> {
            self.calls.lock().unwrap().push("stop".into());
            Ok(())
        }
        fn poll(&self) -> Result<DriverStatus> {
            let mut calls = self.calls.lock().unwrap();
            calls.push("poll".into());
            let polls = calls.iter().filter(|call| *call == "poll").count();
            Ok(DriverStatus {
                eof: polls > self.eof_after_polls,
                ..DriverStatus::default()
            })
        }
    }

    const FAST: Duration = Duration::from_millis(1);

    #[test]
    fn test_sound_loads_one_file_and_stops_its_instance() {
        let driver = FakeDriver {
            eof_after_polls: 2,
            ..FakeDriver::default()
        };
        let tone = PathBuf::from("/cache/test-sound.wav");
        play(&driver, &tone, Duration::from_secs(5), FAST).unwrap();
        let calls = driver.calls();
        assert_eq!(
            calls.iter().filter(|call| call.starts_with("load")).count(),
            1
        );
        assert_eq!(calls[0], "load /cache/test-sound.wav");
        assert_eq!(calls.last().unwrap(), "stop");
        assert!(!calls.iter().any(|call| call == "seek" || call == "pause"));
    }

    #[test]
    fn test_sound_stops_at_the_timeout_without_eof() {
        let driver = FakeDriver {
            eof_after_polls: usize::MAX,
            ..FakeDriver::default()
        };
        play(&driver, Path::new("t.wav"), Duration::from_millis(20), FAST).unwrap();
        assert_eq!(driver.calls().last().unwrap(), "stop");
    }

    #[test]
    fn test_sound_stops_the_instance_after_a_load_error() {
        let driver = FakeDriver {
            fail_load: true,
            ..FakeDriver::default()
        };
        assert!(play(&driver, Path::new("t.wav"), Duration::from_secs(5), FAST).is_err());
        assert_eq!(driver.calls().last().unwrap(), "stop");
    }
}
