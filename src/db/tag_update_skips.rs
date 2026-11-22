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

/// Migration 33: a stored skip keeps its meaning after the frame rename of
/// musicindex ADR 0086 R-TAG-2. Each old label changes to its new label. When
/// a row with the new label exists already, the old row is removed.
pub(crate) fn apply_podcast_frame_labels(conn: &Connection) -> Result<()> {
    for (old, new) in crate::audio_tags::RENAMED_FRAMES {
        conn.execute(
            "UPDATE OR IGNORE tag_update_skips SET frame_label = ?2 WHERE frame_label = ?1",
            params![old, new],
        )
        .context("rename tag update skip label")?;
        conn.execute(
            "DELETE FROM tag_update_skips WHERE frame_label = ?1",
            params![old],
        )
        .context("remove duplicate tag update skip label")?;
    }
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

#[cfg(test)]
mod tests {
    use rusqlite::{params, Connection};

    use crate::db::upgrades;

    fn labels(conn: &Connection) -> Vec<(i64, String, String)> {
        let mut statement = conn
            .prepare(
                "SELECT track_id, frame_label, stored_value FROM tag_update_skips
                 ORDER BY track_id, frame_label",
            )
            .unwrap();
        statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap()
    }

    /// Migration 33 (musicindex ADR 0086 R-TAG-2): each old label changes to
    /// its new label, a label that is not old stays, and a row that would
    /// copy an existing new row is removed.
    #[test]
    fn migration_33_renames_the_old_frame_labels() {
        let conn = Connection::open_in_memory().unwrap();
        upgrades::create_fixture(&conn, 32).unwrap();
        conn.pragma_update(None, "foreign_keys", false).unwrap();
        for (track_id, label) in [
            (1, "TXXX:MusicIndex Track Guid"),
            (1, "TIT2"),
            (2, "COMM:MusicIndex Album Description"),
            (2, "COMM:Podcast Feed Description"),
        ] {
            conn.execute(
                "INSERT INTO tag_update_skips (track_id, frame_label, stored_value, recorded_at)
                 VALUES (?1, ?2, 'value', 0)",
                params![track_id, label],
            )
            .unwrap();
        }

        crate::db::migrate_schema_to(&conn, 33).unwrap();

        let value = || "value".to_owned();
        assert_eq!(
            labels(&conn),
            [
                (1, "TIT2".to_owned(), value()),
                (1, "TXXX:Podcast Item Guid".to_owned(), value()),
                (2, "COMM:Podcast Feed Description".to_owned(), value()),
            ]
        );
    }
}
