#![warn(clippy::pedantic)]

//! Broadcast event registry service for ADR 0113.
//!
//! This module owns the GPUI-free service boundary around relay event creation,
//! liveness checks, local rows, and token files. It calls the relay through
//! `api::Client`; it never sends metadata and never returns broadcaster token
//! text to callers.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use rusqlite::Connection;
use serde::Serialize;

use crate::broadcast::publisher_targets::PublisherTarget;
use crate::broadcast::tokens;
use crate::{api, config, db};

type Clock = fn() -> Result<i64>;

/// Local registry for relay broadcast events.
pub struct BroadcastRegistry<'a> {
    conn: &'a Connection,
    endpoint: String,
    client: api::Client,
    token_directory: PathBuf,
    clock: Clock,
}

/// A created broadcast event without broadcaster token text.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CreatedBroadcastEvent {
    pub event: db::BroadcastEventRow,
    pub token_path: String,
}

/// The result of a definite broadcast event liveness check.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CheckedBroadcastEvent {
    pub event: db::BroadcastEventRow,
    pub status: db::BroadcastEventStatus,
}

/// A relay answer arrived, but storing or reading back that answer failed (ADR 0113).
#[derive(Debug)]
pub(crate) struct EventCheckSaveError {
    pub(crate) observed_status: db::BroadcastEventStatus,
    source: anyhow::Error,
}

impl std::fmt::Display for EventCheckSaveError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "save relay answer: {:#}", self.source)
    }
}

impl std::error::Error for EventCheckSaveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self.source.as_ref())
    }
}

/// The target list of the configured host, as `Forget` needs it (ADR 0110 Decision 6).
#[derive(Clone, Copy, Debug)]
pub enum TargetListRead<'a> {
    /// `HitIt` read the target list.
    Read(&'a [PublisherTarget]),
    /// The read failed. The text names the failure.
    Failed(&'a str),
}

/// Why `Forget` kept a token file (ADR 0110 Decision 6).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenFileKeptReason {
    /// The row is Reserved. `HitIt` did not write the file.
    ReservedRow,
    /// A target of the configured host names the file.
    UsedByTarget,
    /// `HitIt` could not read the target list.
    TargetListUnread,
}

impl TokenFileKeptReason {
    /// Text for the operator.
    #[must_use]
    pub const fn text(self) -> &'static str {
        match self {
            Self::ReservedRow => "the row is a reserved event and HitIt did not write the file",
            Self::UsedByTarget => "a publisher target uses the file",
            Self::TargetListUnread => "HitIt could not read the publisher target list",
        }
    }
}

/// The result of `Forget`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ForgottenBroadcastEvent {
    pub event: db::BroadcastEventRow,
    pub token_file_kept: bool,
    pub token_file_kept_reason: Option<TokenFileKeptReason>,
}

/// The result of adding a reserved event (ADR 0110 Decision 2).
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AddedReservedEvent {
    pub event: db::BroadcastEventRow,
    /// The relay check failed, and the row has the status Unknown.
    pub check_failure: Option<String>,
}

/// Forget a local registration without requiring a remote endpoint.
///
/// The row is always removed. The token file is removed only for a Created
/// row when `HitIt` read the target list and no target names the file
/// (ADR 0110 Decision 6).
///
/// # Errors
/// Reports database or token-file removal errors.
pub fn forget_local_event(
    conn: &Connection,
    event_id: &str,
    targets: TargetListRead<'_>,
) -> Result<Option<ForgottenBroadcastEvent>> {
    let Some(event) = db::broadcast_event_by_event_id(conn, event_id)? else {
        return Ok(None);
    };

    let kept_reason = if event.kind == db::BroadcastEventKind::Reserved {
        Some(TokenFileKeptReason::ReservedRow)
    } else {
        match targets {
            TargetListRead::Failed(_) => Some(TokenFileKeptReason::TargetListUnread),
            TargetListRead::Read(list) => list
                .iter()
                .any(|target| target.token_file == Path::new(&event.token_path))
                .then_some(TokenFileKeptReason::UsedByTarget),
        }
    };
    if kept_reason.is_none() {
        remove_token_file(Path::new(&event.token_path))?;
    }
    anyhow::ensure!(
        db::delete_broadcast_event(conn, event.id)?,
        "broadcast event disappeared before delete: {}",
        event.event_id
    );
    Ok(Some(ForgottenBroadcastEvent {
        event,
        token_file_kept: kept_reason.is_some(),
        token_file_kept_reason: kept_reason,
    }))
}

impl<'a> BroadcastRegistry<'a> {
    /// Build a registry using the default token directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the endpoint or token directory cannot be resolved.
    pub fn new(conn: &'a Connection, endpoint: &str) -> Result<Self> {
        Self::with_token_directory(conn, endpoint, tokens::default_token_directory()?)
    }

    /// Build a registry using an explicit token directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the endpoint is not a valid `MusicIndex` endpoint.
    pub fn with_token_directory(
        conn: &'a Connection,
        endpoint: &str,
        token_directory: PathBuf,
    ) -> Result<Self> {
        Self::with_clock(conn, endpoint, token_directory, unix_timestamp)
    }

    /// Create a relay event and store its token path.
    ///
    /// The relay returns the broadcaster token one time only. This method writes
    /// that token to disk before it inserts the registry row, then returns only
    /// the row and token path.
    ///
    /// # Errors
    ///
    /// Returns an error if relay creation, token storage, or row insertion
    /// fails. If row insertion fails, the newly written token file is removed.
    pub fn create_event(&self, label: Option<&str>) -> Result<CreatedBroadcastEvent> {
        let response = self.client.create_live_item().context("create live item")?;
        if db::broadcast_event_by_event_id(self.conn, &response.event_id)?.is_some() {
            return Err(anyhow!(
                "broadcast event already exists: {}",
                response.event_id
            ));
        }

        let token_path =
            tokens::token_path_in_directory(&self.token_directory, &response.event_id)?;
        tokens::write_token_file(&token_path, &response.broadcaster_token)
            .context("write broadcast token file")?;

        let input = db::BroadcastEventInput {
            event_id: response.event_id,
            label: normalized_label(label),
            endpoint: self.endpoint.clone(),
            token_path: path_to_string(&token_path),
            created_at: (self.clock)()?,
            last_checked_at: None,
            last_status: Some(db::BroadcastEventStatus::Unknown),
            kind: db::BroadcastEventKind::Created,
        };
        let row_id = match db::insert_broadcast_event(self.conn, &input) {
            Ok(row_id) => row_id,
            Err(error) => {
                remove_token_after_failed_insert(&token_path)?;
                return Err(error).context("insert broadcast event");
            }
        };
        let event = db::broadcast_event_by_id(self.conn, row_id)?
            .context("created broadcast event disappeared")?;

        Ok(CreatedBroadcastEvent {
            event,
            token_path: path_to_string(&token_path),
        })
    }

    /// List stored broadcast events.
    ///
    /// # Errors
    ///
    /// Returns an error if the database query fails.
    pub fn list_events(&self) -> Result<Vec<db::BroadcastEventRow>> {
        db::broadcast_events(self.conn)
    }

    /// Forget one stored broadcast event.
    ///
    /// This removes the local row. It keeps the token file in the cases of
    /// [`forget_local_event`]. It sends no request to the relay, because the
    /// relay has no delete route.
    ///
    /// # Errors
    ///
    /// Returns an error if the event identifier is invalid, the token file
    /// cannot be removed, or the database delete fails.
    pub fn forget_event(
        &self,
        event_id: &str,
        targets: TargetListRead<'_>,
    ) -> Result<Option<ForgottenBroadcastEvent>> {
        forget_local_event(self.conn, event_id, targets)
    }

    /// Add a reserved event from its event ID and token file (ADR 0110 Decision 2).
    ///
    /// The method reads the token file only to check it. It does not keep the
    /// token text and does not copy, move or change the file.
    ///
    /// # Errors
    ///
    /// Returns an error that names the failed check. `HitIt` stores no row when
    /// a check of steps 1 to 5 fails or when the relay answers
    /// `event_not_found`. When the relay does not answer, the row is stored
    /// with the status Unknown and the result carries the failure.
    pub fn add_reserved_event(
        &self,
        event_id: &str,
        token_path: &Path,
        label: Option<&str>,
    ) -> Result<AddedReservedEvent> {
        let event_id = event_id.trim();
        anyhow::ensure!(
            !event_id.is_empty(),
            "add reserved event check 1 (event ID): the event ID is empty"
        );
        anyhow::ensure!(
            db::broadcast_event_by_event_id(self.conn, event_id)?.is_none(),
            "add reserved event check 1 (event ID): the registry already has event {event_id}"
        );
        check_reserved_token_file(token_path)?;

        let (status, check_failure) = match self.client.fetch_live_metadata_classified(event_id) {
            Ok(api::LiveMetadataRead::Snapshot(_) | api::LiveMetadataRead::MetadataNotFound) => {
                (db::BroadcastEventStatus::Live, None)
            }
            Ok(api::LiveMetadataRead::EventNotFound) => {
                return Err(anyhow!(
                    "add reserved event check 6 (relay): the relay says event {event_id} does not exist"
                ));
            }
            Ok(api::LiveMetadataRead::OtherNotFound(code)) => (
                db::BroadcastEventStatus::Unknown,
                Some(other_not_found_text(code.as_deref())),
            ),
            Err(error) => (
                db::BroadcastEventStatus::Unknown,
                Some(format!("add reserved event check 6 (relay): {error:#}")),
            ),
        };
        let checked_at = (self.clock)()?;
        let input = db::BroadcastEventInput {
            event_id: event_id.to_owned(),
            label: normalized_label(label),
            endpoint: self.endpoint.clone(),
            token_path: path_to_string(token_path),
            created_at: checked_at,
            last_checked_at: check_failure.is_none().then_some(checked_at),
            last_status: Some(status),
            kind: db::BroadcastEventKind::Reserved,
        };
        let row_id =
            db::insert_broadcast_event(self.conn, &input).context("insert broadcast event")?;
        let event = db::broadcast_event_by_id(self.conn, row_id)?
            .context("reserved broadcast event disappeared")?;
        Ok(AddedReservedEvent {
            event,
            check_failure,
        })
    }

    /// Check one stored broadcast event for relay liveness.
    ///
    /// A successful metadata read marks the event live. A `404` with the code
    /// `metadata_not_found` also marks it live. A `404` with the code
    /// `event_not_found` marks it dead (ADR 0110 Decision 4). Other `404`
    /// answers and transport errors leave the stored status unchanged.
    ///
    /// # Errors
    ///
    /// Returns an error if the event is not stored, if the relay read has a
    /// transport failure or an unknown `404`, or if the status update fails.
    pub fn check_event(&self, event_id: &str) -> Result<CheckedBroadcastEvent> {
        let event = self.require_event(event_id)?;
        let client = api::Client::new_with_base_url(event.endpoint.clone());
        let status = match client
            .fetch_live_metadata_classified(&event.event_id)
            .with_context(|| format!("check broadcast event {}", event.event_id))?
        {
            api::LiveMetadataRead::Snapshot(_) | api::LiveMetadataRead::MetadataNotFound => {
                db::BroadcastEventStatus::Live
            }
            api::LiveMetadataRead::EventNotFound => db::BroadcastEventStatus::Dead,
            api::LiveMetadataRead::OtherNotFound(code) => {
                return Err(anyhow!(
                    "check broadcast event {}: {}",
                    event.event_id,
                    other_not_found_text(code.as_deref())
                ));
            }
        };
        self.store_checked_event(&event, status).map_err(|source| {
            EventCheckSaveError {
                observed_status: status,
                source,
            }
            .into()
        })
    }

    fn store_checked_event(
        &self,
        event: &db::BroadcastEventRow,
        status: db::BroadcastEventStatus,
    ) -> Result<CheckedBroadcastEvent> {
        let checked_at = (self.clock)()?;
        anyhow::ensure!(
            db::update_broadcast_event_status(self.conn, event.id, status, Some(checked_at))?,
            "broadcast event disappeared before status update: {}",
            event.event_id
        );
        let event = db::broadcast_event_by_id(self.conn, event.id)?
            .context("checked broadcast event disappeared")?;
        Ok(CheckedBroadcastEvent { event, status })
    }

    fn require_event(&self, event_id: &str) -> Result<db::BroadcastEventRow> {
        db::broadcast_event_by_event_id(self.conn, event_id)?
            .with_context(|| format!("broadcast event not found: {event_id}"))
    }

    fn with_clock(
        conn: &'a Connection,
        endpoint: &str,
        token_directory: PathBuf,
        clock: Clock,
    ) -> Result<Self> {
        let endpoint = config::normalize_musicindex_endpoint(endpoint)?;
        Ok(Self {
            conn,
            endpoint: endpoint.clone(),
            client: api::Client::new_with_base_url(endpoint),
            token_directory,
            clock,
        })
    }
}

fn other_not_found_text(code: Option<&str>) -> String {
    match code {
        Some(code) => format!("the relay answered 404 with the unknown code {code:?}"),
        None => "the relay answered 404 with no readable code".to_owned(),
    }
}

/// Checks 2 to 5 of ADR 0110 Decision 2. The token text does not leave this function.
fn check_reserved_token_file(path: &Path) -> Result<()> {
    anyhow::ensure!(
        path.is_absolute(),
        "add reserved event check 2 (absolute path): the token path is not absolute: {}",
        path.display()
    );
    let metadata = fs::symlink_metadata(path).with_context(|| {
        format!(
            "add reserved event check 3 (regular file): cannot read {}",
            path.display()
        )
    })?;
    anyhow::ensure!(
        !metadata.file_type().is_symlink(),
        "add reserved event check 3 (regular file): the token path is a symlink: {}",
        path.display()
    );
    anyhow::ensure!(
        metadata.file_type().is_file(),
        "add reserved event check 3 (regular file): the token path is not a regular file: {}",
        path.display()
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;

        let mode = metadata.permissions().mode() & 0o777;
        anyhow::ensure!(
            mode == 0o600,
            "add reserved event check 4 (mode): the token file mode is {mode:o}, not 600: {}",
            path.display()
        );
    }
    let content = fs::read_to_string(path).with_context(|| {
        format!(
            "add reserved event check 5 (one line): cannot read {}",
            path.display()
        )
    })?;
    let line = content.strip_suffix('\n').unwrap_or(&content);
    anyhow::ensure!(
        !line.trim().is_empty() && !line.contains('\n'),
        "add reserved event check 5 (one line): the token file must hold one line that is not empty: {}",
        path.display()
    );
    Ok(())
}

fn normalized_label(label: Option<&str>) -> Option<String> {
    label
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(str::to_owned)
}

fn path_to_string(path: &Path) -> String {
    path.display().to_string()
}

fn remove_token_after_failed_insert(path: &Path) -> Result<()> {
    remove_token_file(path).with_context(|| {
        format!(
            "remove broadcast token file after failed row insert: {}",
            path.display()
        )
    })
}

fn remove_token_file(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => {
            Err(error).with_context(|| format!("remove broadcast token file {}", path.display()))
        }
    }
}

fn unix_timestamp() -> Result<i64> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("read system time for broadcast event")?
        .as_secs();
    i64::try_from(seconds).context("convert broadcast event timestamp")
}

#[cfg(test)]
mod tests {
    use std::io::{Read as _, Write as _};

    use anyhow::Result;

    use super::*;

    const FIXED_TIME: i64 = 1_778_284_900;

    #[expect(
        clippy::unnecessary_wraps,
        reason = "fixed_time must match the production Clock type, fn() -> Result<i64>"
    )]
    fn fixed_time() -> Result<i64> {
        Ok(FIXED_TIME)
    }

    fn test_db(temp: &tempfile::TempDir) -> Result<Connection> {
        db::open_db(&temp.path().join("hitit.sqlite")).map(|prepared| prepared.connection)
    }

    fn registry<'a>(
        conn: &'a Connection,
        endpoint: &str,
        temp: &tempfile::TempDir,
    ) -> Result<BroadcastRegistry<'a>> {
        BroadcastRegistry::with_clock(conn, endpoint, temp.path().join("tokens"), fixed_time)
    }

    fn serve_once(status: &str, body: impl Into<String>) -> String {
        let listener = crate::test_ports::fresh_local_listener().expect("bind listener");
        let addr = listener.local_addr().expect("listener addr");
        let status = status.to_owned();
        let body = body.into();
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept request");
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request);
            let headers = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(headers.as_bytes()).expect("write headers");
            stream.write_all(body.as_bytes()).expect("write body");
        });
        format!("http://{addr}")
    }

    fn serve_disconnect_once() -> String {
        let listener = crate::test_ports::fresh_local_listener().expect("bind listener");
        let addr = listener.local_addr().expect("listener addr");
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept request");
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request);
        });
        format!("http://{addr}")
    }

    fn create_response(event_id: &str, token: &str) -> String {
        serde_json::json!({
            "event_id": event_id,
            "broadcaster_token": token,
            "metadata_url": format!("https://relay.example.test/v1/liveitems/{event_id}/metadata"),
            "events_url": "https://relay.example.test/events",
        })
        .to_string()
    }

    fn insert_event(
        conn: &Connection,
        event_id: &str,
        endpoint: &str,
        temp: &tempfile::TempDir,
        status: db::BroadcastEventStatus,
        checked_at: Option<i64>,
    ) -> Result<i64> {
        let token_path = temp.path().join(format!("{event_id}.token"));
        tokens::write_token_file(&token_path, "stored-token")?;
        db::insert_broadcast_event(
            conn,
            &db::BroadcastEventInput {
                event_id: event_id.to_owned(),
                label: Some("Sunday set".to_owned()),
                endpoint: endpoint.to_owned(),
                token_path: path_to_string(&token_path),
                created_at: FIXED_TIME - 1,
                last_checked_at: checked_at,
                last_status: Some(status),
                kind: db::BroadcastEventKind::Created,
            },
        )
    }

    #[cfg(unix)]
    fn file_mode(path: &Path) -> Result<u32> {
        use std::os::unix::fs::PermissionsExt as _;

        Ok(fs::metadata(path)
            .with_context(|| format!("read mode for {}", path.display()))?
            .permissions()
            .mode()
            & 0o777)
    }

    #[test]
    fn create_event_writes_token_file_and_registry_row_without_returning_token_text() -> Result<()>
    {
        let temp = tempfile::tempdir()?;
        let conn = test_db(&temp)?;
        let body = create_response("event-one", "secret-token");
        let endpoint = serve_once("200 OK", body);
        let registry = registry(&conn, &endpoint, &temp)?;

        let created = registry.create_event(Some(" Sunday set "))?;

        assert_eq!(created.event.event_id, "event-one");
        assert_eq!(created.event.label.as_deref(), Some("Sunday set"));
        assert_eq!(created.event.endpoint, endpoint);
        assert_eq!(created.event.created_at, FIXED_TIME);
        assert_eq!(
            created.event.last_status,
            Some(db::BroadcastEventStatus::Unknown)
        );
        assert_eq!(
            tokens::read_token_file(Path::new(&created.token_path))?,
            "secret-token"
        );
        let json = serde_json::to_string(&created)?;
        assert!(
            !json.contains("secret-token"),
            "created event JSON must not expose token text"
        );
        #[cfg(unix)]
        assert_eq!(
            file_mode(Path::new(&created.token_path))?,
            0o600,
            "created token file should be owner-readable only"
        );
        assert_eq!(
            registry.list_events()?,
            vec![created.event],
            "created event should be listable"
        );
        Ok(())
    }

    #[test]
    fn create_event_cleans_up_token_file_when_insert_fails() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let conn = test_db(&temp)?;
        conn.execute_batch(
            "CREATE TRIGGER fail_broadcast_event_insert
             BEFORE INSERT ON broadcast_events
             BEGIN
                 SELECT RAISE(FAIL, 'forced insert failure');
             END;",
        )?;
        let body = create_response("event-insert-fails", "secret-token");
        let endpoint = serve_once("200 OK", body);
        let registry = registry(&conn, &endpoint, &temp)?;

        let error = registry
            .create_event(None)
            .expect_err("forced database insert failure should bubble up");

        assert!(
            error.to_string().contains("insert broadcast event"),
            "error should name the failed row insert: {error}"
        );
        let token_path =
            tokens::token_path_in_directory(&temp.path().join("tokens"), "event-insert-fails")?;
        assert!(
            !token_path.exists(),
            "failed insert should remove the newly written token file"
        );
        Ok(())
    }

    #[test]
    fn create_event_does_not_replace_existing_token_file_for_duplicate_event_id() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let conn = test_db(&temp)?;
        let token_path = tokens::token_path_in_directory(&temp.path().join("tokens"), "event-one")?;
        tokens::write_token_file(&token_path, "original-token")?;
        db::insert_broadcast_event(
            &conn,
            &db::BroadcastEventInput {
                event_id: "event-one".to_owned(),
                label: None,
                endpoint: "http://127.0.0.1:9".to_owned(),
                token_path: path_to_string(&token_path),
                created_at: FIXED_TIME,
                last_checked_at: None,
                last_status: Some(db::BroadcastEventStatus::Dead),
                kind: db::BroadcastEventKind::Created,
            },
        )?;
        let body = create_response("event-one", "replacement-token");
        let endpoint = serve_once("200 OK", body);
        let registry = registry(&conn, &endpoint, &temp)?;

        let error = registry
            .create_event(None)
            .expect_err("duplicate event id should not replace a stored token");

        assert!(
            error.to_string().contains("already exists"),
            "duplicate error should explain the local event conflict: {error}"
        );
        assert_eq!(
            tokens::read_token_file(&token_path)?,
            "original-token",
            "existing token file must not be overwritten for a duplicate event"
        );
        Ok(())
    }

    #[test]
    fn check_event_maps_live_metadata_response_to_live_status() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let conn = test_db(&temp)?;
        let endpoint = serve_once(
            "200 OK",
            r#"{"event_id":"event-one","seq":1,"updated_at":"2026-09-07T00:00:00Z","metadata":{}}"#,
        );
        let id = insert_event(
            &conn,
            "event-one",
            &endpoint,
            &temp,
            db::BroadcastEventStatus::Unknown,
            None,
        )?;
        let registry = registry(&conn, &endpoint, &temp)?;

        let checked = registry.check_event("event-one")?;

        assert_eq!(checked.status, db::BroadcastEventStatus::Live);
        assert_eq!(
            checked.event.last_status,
            Some(db::BroadcastEventStatus::Live)
        );
        assert_eq!(checked.event.last_checked_at, Some(FIXED_TIME));
        assert_eq!(
            db::broadcast_event_by_id(&conn, id)?
                .context("event should still exist")?
                .last_status,
            Some(db::BroadcastEventStatus::Live)
        );
        Ok(())
    }

    /// ADR 0110 Decision 4: each row of the relay answer table.
    #[test]
    fn adr_0110_check_event_tells_the_404_codes_apart() -> Result<()> {
        let cases = [
            (
                r#"{"error":"metadata_not_found"}"#,
                Some(db::BroadcastEventStatus::Live),
            ),
            (
                r#"{"error":"event_not_found"}"#,
                Some(db::BroadcastEventStatus::Dead),
            ),
            (r#"{"error":"liveitem_not_found"}"#, None),
            (r#"{"message":"gone"}"#, None),
            ("not json", None),
        ];
        for (body, expected) in cases {
            let temp = tempfile::tempdir()?;
            let conn = test_db(&temp)?;
            let endpoint = serve_once("404 Not Found", body);
            let id = insert_event(
                &conn,
                "event-one",
                &endpoint,
                &temp,
                db::BroadcastEventStatus::Unknown,
                Some(1),
            )?;
            let registry = registry(&conn, &endpoint, &temp)?;

            let result = registry.check_event("event-one");

            let row = db::broadcast_event_by_id(&conn, id)?.context("stored event")?;
            if let Some(status) = expected {
                assert_eq!(result?.status, status, "body {body}");
                assert_eq!(row.last_status, Some(status));
                assert_eq!(row.last_checked_at, Some(FIXED_TIME));
            } else {
                assert!(result.is_err(), "body {body} must give a failure");
                assert_eq!(row.last_status, Some(db::BroadcastEventStatus::Unknown));
                assert_eq!(row.last_checked_at, Some(1));
            }
        }
        Ok(())
    }

    #[test]
    fn check_event_leaves_status_unchanged_on_transport_failure() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let conn = test_db(&temp)?;
        let endpoint = serve_disconnect_once();
        let id = insert_event(
            &conn,
            "event-one",
            &endpoint,
            &temp,
            db::BroadcastEventStatus::Live,
            Some(123),
        )?;
        let registry = registry(&conn, &endpoint, &temp)?;

        let error = registry
            .check_event("event-one")
            .expect_err("transport failure should return an error");

        let response = error
            .downcast_ref::<crate::api::LiveMetadataReadError>()
            .expect("ADR 0113: retain response facts through registry context");
        assert_eq!(response.response_status, None);

        assert!(
            error
                .to_string()
                .contains("check broadcast event event-one"),
            "error should name the liveness check without token text: {error}"
        );
        let row = db::broadcast_event_by_id(&conn, id)?.context("event should still exist")?;
        assert_eq!(row.last_status, Some(db::BroadcastEventStatus::Live));
        assert_eq!(row.last_checked_at, Some(123));
        Ok(())
    }

    /// Situational ADR 0113: rejected and unreadable HTTP responses remain distinct from no response.
    #[test]
    fn check_event_retains_http_status_without_changing_saved_facts() -> Result<()> {
        for (status, expected) in [("503 Service Unavailable", 503), ("200 OK", 200)] {
            let temp = tempfile::tempdir()?;
            let conn = test_db(&temp)?;
            let endpoint = serve_once(status, "not metadata JSON");
            let id = insert_event(
                &conn,
                "event-one",
                &endpoint,
                &temp,
                db::BroadcastEventStatus::Live,
                Some(123),
            )?;
            let registry = registry(&conn, &endpoint, &temp)?;
            let error = registry
                .check_event("event-one")
                .expect_err("metadata request cannot answer");
            let response = error
                .downcast_ref::<crate::api::LiveMetadataReadError>()
                .expect("retain HTTP status through registry context");
            assert_eq!(response.response_status, Some(expected));
            let row = db::broadcast_event_by_id(&conn, id)?.context("stored event")?;
            assert_eq!(row.last_status, Some(db::BroadcastEventStatus::Live));
            assert_eq!(row.last_checked_at, Some(123));
        }
        Ok(())
    }

    fn insert_kind_event(
        conn: &Connection,
        event_id: &str,
        kind: db::BroadcastEventKind,
        temp: &tempfile::TempDir,
    ) -> Result<PathBuf> {
        let token_path = temp.path().join(format!("{event_id}.token"));
        tokens::write_token_file(&token_path, "stored-token")?;
        db::insert_broadcast_event(
            conn,
            &db::BroadcastEventInput {
                event_id: event_id.to_owned(),
                label: None,
                endpoint: "http://127.0.0.1:9".to_owned(),
                token_path: path_to_string(&token_path),
                created_at: FIXED_TIME,
                last_checked_at: None,
                last_status: Some(db::BroadcastEventStatus::Unknown),
                kind,
            },
        )?;
        Ok(token_path)
    }

    fn target_using(path: &Path) -> PublisherTarget {
        PublisherTarget {
            name: "default".to_owned(),
            event_id: "event-one".to_owned(),
            token_file: path.to_path_buf(),
            stream_delay_secs: 0.0,
        }
    }

    /// ADR 0110 Decision 6: `Forget` removes the file only in the one permitted case.
    #[test]
    fn adr_0110_forget_keeps_a_token_file_that_hitit_must_not_remove() -> Result<()> {
        let unused = PublisherTarget {
            token_file: PathBuf::from("/fixture/other.token"),
            ..target_using(Path::new("/fixture/other.token"))
        };
        let own_path = |temp: &tempfile::TempDir| temp.path().join("event-one.token");
        // (kind, target list uses file, target list unread) -> (file kept, reason)
        let cases = [
            (
                db::BroadcastEventKind::Reserved,
                false,
                false,
                Some(TokenFileKeptReason::ReservedRow),
            ),
            (
                db::BroadcastEventKind::Created,
                true,
                false,
                Some(TokenFileKeptReason::UsedByTarget),
            ),
            (
                db::BroadcastEventKind::Created,
                false,
                true,
                Some(TokenFileKeptReason::TargetListUnread),
            ),
            (db::BroadcastEventKind::Created, false, false, None),
        ];
        for (kind, used, unread, expected) in cases {
            let temp = tempfile::tempdir()?;
            let conn = test_db(&temp)?;
            let token_path = insert_kind_event(&conn, "event-one", kind, &temp)?;
            assert_eq!(token_path, own_path(&temp));
            let list = if used {
                vec![target_using(&token_path)]
            } else {
                vec![unused.clone()]
            };
            let targets = if unread {
                TargetListRead::Failed("host not reachable")
            } else {
                TargetListRead::Read(&list)
            };

            let forgotten =
                forget_local_event(&conn, "event-one", targets)?.context("forgotten event")?;

            assert_eq!(forgotten.token_file_kept_reason, expected);
            assert_eq!(forgotten.token_file_kept, expected.is_some());
            assert_eq!(token_path.exists(), expected.is_some());
            assert!(db::broadcast_event_by_event_id(&conn, "event-one")?.is_none());
        }
        let temp = tempfile::tempdir()?;
        let conn = test_db(&temp)?;
        assert!(forget_local_event(&conn, "missing", TargetListRead::Read(&[]))?.is_none());
        Ok(())
    }

    #[cfg(unix)]
    fn write_reserved_token(path: &Path, content: &str, mode: u32) -> Result<()> {
        use std::os::unix::fs::PermissionsExt as _;

        fs::write(path, content)?;
        fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
        Ok(())
    }

    /// ADR 0110 Decision 2: each failed check names itself and stores no row.
    #[cfg(unix)]
    #[test]
    fn adr_0110_add_reserved_event_names_the_failed_check_and_stores_no_row() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let conn = test_db(&temp)?;
        let registry = registry(&conn, "http://127.0.0.1:9", &temp)?;
        let dir = temp.path();
        let good = dir.join("good.token");
        write_reserved_token(&good, "fixture-token\n", 0o600)?;
        insert_event(
            &conn,
            "taken",
            "http://127.0.0.1:9",
            &temp,
            db::BroadcastEventStatus::Live,
            None,
        )?;
        let wide = dir.join("wide.token");
        write_reserved_token(&wide, "fixture-token\n", 0o644)?;
        let empty = dir.join("empty.token");
        write_reserved_token(&empty, "", 0o600)?;
        let blank = dir.join("blank.token");
        write_reserved_token(&blank, "\n", 0o600)?;
        let two = dir.join("two.token");
        write_reserved_token(&two, "one\ntwo\n", 0o600)?;
        let link = dir.join("link.token");
        std::os::unix::fs::symlink(&good, &link)?;
        let folder = dir.join("folder.token");
        fs::create_dir(&folder)?;

        let cases: [(&str, &Path, &str); 9] = [
            ("", &good, "check 1"),
            ("taken", &good, "check 1"),
            ("fixture-event", Path::new("relative.token"), "check 2"),
            ("fixture-event", &link, "check 3"),
            ("fixture-event", &folder, "check 3"),
            ("fixture-event", &wide, "check 4"),
            ("fixture-event", &empty, "check 5"),
            ("fixture-event", &blank, "check 5"),
            ("fixture-event", &two, "check 5"),
        ];
        for (event_id, path, check) in cases {
            let error = registry
                .add_reserved_event(event_id, path, None)
                .expect_err("a failed check must refuse the row");
            assert!(
                error.to_string().contains(check),
                "{event_id} {} should name {check}: {error}",
                path.display()
            );
            assert!(
                db::broadcast_event_by_event_id(&conn, "fixture-event")?.is_none(),
                "no row after a failed check"
            );
        }
        assert_eq!(db::broadcast_events(&conn)?.len(), 1);
        Ok(())
    }

    /// ADR 0110 Decision 2: a valid file gives a Reserved row and the file stays as it was.
    #[cfg(unix)]
    #[test]
    fn adr_0110_add_reserved_event_stores_a_reserved_row_and_keeps_the_file() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let conn = test_db(&temp)?;
        let token = temp.path().join("show.token");
        write_reserved_token(&token, "fixture-token\n", 0o600)?;
        let before = fs::read(&token)?;
        let endpoint = serve_once("404 Not Found", r#"{"error":"metadata_not_found"}"#);
        let registry = registry(&conn, &endpoint, &temp)?;

        let added = registry.add_reserved_event(" fixture-event ", &token, Some(" Friday "))?;

        assert_eq!(added.check_failure, None);
        assert_eq!(added.event.event_id, "fixture-event");
        assert_eq!(added.event.kind, db::BroadcastEventKind::Reserved);
        assert_eq!(added.event.label.as_deref(), Some("Friday"));
        assert_eq!(added.event.endpoint, endpoint);
        assert_eq!(added.event.token_path, path_to_string(&token));
        assert_eq!(
            added.event.last_status,
            Some(db::BroadcastEventStatus::Live)
        );
        assert_eq!(fs::read(&token)?, before);
        assert_eq!(file_mode(&token)?, 0o600);
        assert!(!serde_json::to_string(&added)?.contains("fixture-token"));
        Ok(())
    }

    /// ADR 0110 Decision 2: `event_not_found` stores no row. No answer stores an Unknown row.
    #[cfg(unix)]
    #[test]
    fn adr_0110_add_reserved_event_handles_the_relay_answers() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let conn = test_db(&temp)?;
        let token = temp.path().join("show.token");
        write_reserved_token(&token, "fixture-token\n", 0o600)?;

        let endpoint = serve_once("404 Not Found", r#"{"error":"event_not_found"}"#);
        let error = registry(&conn, &endpoint, &temp)?
            .add_reserved_event("fixture-event", &token, None)
            .expect_err("a missing event must refuse the row");
        assert!(error.to_string().contains("check 6"), "{error}");
        assert!(db::broadcast_events(&conn)?.is_empty());

        let endpoint = serve_disconnect_once();
        let added =
            registry(&conn, &endpoint, &temp)?.add_reserved_event("fixture-event", &token, None)?;
        assert!(added.check_failure.is_some());
        assert_eq!(added.event.kind, db::BroadcastEventKind::Reserved);
        assert_eq!(
            added.event.last_status,
            Some(db::BroadcastEventStatus::Unknown)
        );
        assert_eq!(added.event.last_checked_at, None);
        Ok(())
    }
}
