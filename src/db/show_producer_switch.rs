//! The stored fact of a producer switch (ADR 0111 Decision 7).
//!
//! One row holds "the switch of this show stopped the private producer", the
//! show instance, the private instance and the wall clock start of the
//! recording. A restart of `HitIt` reads the row, so "End show" starts the
//! private producer again.

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};

pub(crate) const COLUMNS: &[(&str, &[&str])] = &[(
    "show_producer_switch",
    &[
        "singleton",
        "show_instance",
        "private_instance",
        "private_stopped",
        "recording_started_at",
    ],
)];

/// The stored fact. `private_stopped` is the fact itself.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StoredSwitch {
    /// Show instance of the switch.
    pub(crate) show_instance: String,
    /// Private instance of the switch.
    pub(crate) private_instance: String,
    /// The switch stopped the private producer.
    pub(crate) private_stopped: bool,
    /// Wall clock start of the recording, as an RFC 3339 text.
    pub(crate) recording_started_at: Option<String>,
}

pub(crate) fn apply(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE show_producer_switch (
            singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
            show_instance TEXT NOT NULL,
            private_instance TEXT NOT NULL,
            private_stopped INTEGER NOT NULL CHECK (private_stopped IN (0, 1)),
            recording_started_at TEXT
        )",
    )
    .context("create show producer switch")?;
    Ok(())
}

/// Store the fact. `HitIt` calls this before the switch stops the private producer.
pub(crate) fn store_private_stopped(
    conn: &Connection,
    show_instance: &str,
    private_instance: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO show_producer_switch
            (singleton, show_instance, private_instance, private_stopped, recording_started_at)
         VALUES (1, ?1, ?2, 1, NULL)
         ON CONFLICT(singleton) DO UPDATE SET
            show_instance = excluded.show_instance,
            private_instance = excluded.private_instance,
            private_stopped = 1,
            recording_started_at = NULL",
        params![show_instance, private_instance],
    )
    .context("store the producer switch fact")?;
    Ok(())
}

/// Clear the fact. The row stays, so the start time of the recording stays.
pub(crate) fn clear(conn: &Connection) -> Result<()> {
    conn.execute(
        "UPDATE show_producer_switch SET private_stopped = 0 WHERE singleton = 1",
        [],
    )
    .context("clear the producer switch fact")?;
    Ok(())
}

/// Store the recording start of a show whose switch did not stop the private producer.
pub(crate) fn store_recording_started(
    conn: &Connection,
    show_instance: &str,
    private_instance: &str,
    started_at: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO show_producer_switch
            (singleton, show_instance, private_instance, private_stopped, recording_started_at)
         VALUES (1, ?1, ?2, 0, ?3)
         ON CONFLICT(singleton) DO UPDATE SET recording_started_at = excluded.recording_started_at",
        params![show_instance, private_instance, started_at],
    )
    .context("store the recording start")?;
    Ok(())
}

/// Read the stored row, if one exists.
pub(crate) fn read(conn: &Connection) -> Result<Option<StoredSwitch>> {
    conn.query_row(
        "SELECT show_instance, private_instance, private_stopped, recording_started_at
         FROM show_producer_switch WHERE singleton = 1",
        [],
        |row| {
            Ok(StoredSwitch {
                show_instance: row.get(0)?,
                private_instance: row.get(1)?,
                private_stopped: row.get::<_, i64>(2)? != 0,
                recording_started_at: row.get(3)?,
            })
        },
    )
    .optional()
    .context("read the producer switch fact")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        apply(&conn).unwrap();
        conn
    }

    #[test]
    fn the_fact_is_stored_read_and_cleared() {
        let conn = conn();
        assert_eq!(read(&conn).unwrap(), None);
        store_private_stopped(&conn, "show-1", "mixxx").unwrap();
        let stored = read(&conn).unwrap().unwrap();
        assert!(stored.private_stopped);
        assert_eq!(stored.show_instance, "show-1");
        assert_eq!(stored.private_instance, "mixxx");
        store_recording_started(&conn, "show-1", "mixxx", "2026-11-21T20:00:00+00:00").unwrap();
        let stored = read(&conn).unwrap().unwrap();
        assert!(stored.private_stopped, "the start time keeps the fact");
        assert_eq!(
            stored.recording_started_at.as_deref(),
            Some("2026-11-21T20:00:00+00:00")
        );
        clear(&conn).unwrap();
        let stored = read(&conn).unwrap().unwrap();
        assert!(!stored.private_stopped, "the fact is clear");
        assert!(
            stored.recording_started_at.is_some(),
            "the start time stays"
        );
    }
}
