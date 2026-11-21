//! Stored tag-update skips for ADR 0108.

use anyhow::{Context, Result};
use rusqlite::{params, Connection};

pub(crate) const COLUMNS: &[(&str, &[&str])] = &[(
    "tag_update_skips",
    &["track_id", "frame_label", "stored_value", "recorded_at"],
)];

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct TagUpdateSkip {
    pub(crate) track_id: i64,
    pub(crate) frame_label: String,
    pub(crate) stored_value: String,
}

pub(crate) fn apply(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE tag_update_skips (
            track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
            frame_label TEXT NOT NULL,
            stored_value TEXT NOT NULL,
            recorded_at INTEGER NOT NULL,
            PRIMARY KEY (track_id, frame_label, stored_value)
        )",
    )
    .context("create tag update skips")?;
    Ok(())
}

pub(crate) fn record_tag_update_skips(conn: &Connection, skips: &[TagUpdateSkip]) -> Result<()> {
    let transaction = conn.unchecked_transaction()?;
    let mut statement = transaction.prepare(
        "INSERT OR IGNORE INTO tag_update_skips
         (track_id, frame_label, stored_value, recorded_at) VALUES (?1, ?2, ?3, unixepoch())",
    )?;
    for skip in skips {
        statement
            .execute(params![skip.track_id, skip.frame_label, skip.stored_value])
            .context("record tag update skip")?;
    }
    drop(statement);
    transaction.commit()?;
    Ok(())
}

pub(crate) fn tag_update_skips(conn: &Connection) -> Result<Vec<TagUpdateSkip>> {
    let mut statement = conn.prepare(
        "SELECT track_id, frame_label, stored_value FROM tag_update_skips
         ORDER BY track_id, frame_label, stored_value",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(TagUpdateSkip {
            track_id: row.get(0)?,
            frame_label: row.get(1)?,
            stored_value: row.get(2)?,
        })
    })?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .context("read tag update skips")
}

pub(crate) fn delete_tag_update_skips(conn: &Connection, skips: &[TagUpdateSkip]) -> Result<()> {
    let transaction = conn.unchecked_transaction()?;
    let mut statement = transaction.prepare(
        "DELETE FROM tag_update_skips
         WHERE track_id = ?1 AND frame_label = ?2 AND stored_value = ?3",
    )?;
    for skip in skips {
        statement
            .execute(params![skip.track_id, skip.frame_label, skip.stored_value])
            .context("delete tag update skip")?;
    }
    drop(statement);
    transaction.commit()?;
    Ok(())
}
