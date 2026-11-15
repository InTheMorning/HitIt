//! Migration for audio files outside the Library (ADR 0100).

use anyhow::Result;
use rusqlite::Connection;

pub(crate) const COLUMNS: &[(&str, &[&str])] = &[(
    "cached_files",
    &[
        "id",
        "path",
        "track_id",
        "added_at",
        "file_size_bytes",
        "audio_duration_sec",
        "checksum",
        "extra_json",
    ],
)];

pub(crate) fn apply(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE cached_files (
            id INTEGER PRIMARY KEY,
            path TEXT NOT NULL UNIQUE,
            track_id INTEGER NULL REFERENCES tracks(id) ON DELETE SET NULL,
            added_at TEXT NOT NULL DEFAULT (datetime('now')),
            file_size_bytes INTEGER NULL,
            audio_duration_sec INTEGER NULL,
            checksum TEXT NULL,
            extra_json TEXT NOT NULL DEFAULT '{}'
        );
        CREATE INDEX idx_cached_files_track_id ON cached_files(track_id);",
    )?;
    Ok(())
}
