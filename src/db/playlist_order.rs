//! The stored curator order of the playlists (ADR 0089 amendment of
//! 2026-11-08, musicindex ADR 0053 R-LIST-9).
//!
//! Migration 24 adds `position` to `playlists`. The migration fills it from
//! the name sequence of before the change (`name` without case, then `id`),
//! so no playlist moves. A position is 0-based, as `playlist_tracks.position`
//! is. `position` is `NOT NULL` with the default 0, so an insert that names
//! no position stays valid. A row of the default 0 sorts by name, then `id`,
//! after the row of its position.
//!
//! The playlist insert (`db::playlist_create`) gives a new playlist the next
//! free position. `move_playlist` and `set_playlist_order` write the new
//! sequence in one transaction. `db::playlist_delete` closes the gap.

#![warn(clippy::pedantic)]

use anyhow::{Context, Result};
use rusqlite::{params, Connection};

/// The read contract of `playlists` after migration 24: the version 11
/// columns, then the one column that migration 24 appends.
pub(super) const COLUMNS: &(&str, &[&str]) = &(
    "playlists",
    &[
        "id",
        "name",
        "description",
        "created_at",
        "updated_at",
        "position",
    ],
);

const DDL: &str = r"
ALTER TABLE playlists ADD COLUMN position INTEGER NOT NULL DEFAULT 0;
";

/// Migration 24 (ADR 0016 registry, ADR 0089 amendment of 2026-11-08).
pub(super) fn apply(conn: &Connection) -> Result<()> {
    conn.execute_batch(DDL)
        .context("Add the playlist position column")?;
    let ids = ids_in_sequence(conn, "ORDER BY name COLLATE NOCASE, id")
        .context("Read the playlists in name order")?;
    write_sequence(conn, &ids).context("Fill the playlist positions")
}

/// The playlist ids in the order that `order_by` gives.
fn ids_in_sequence(conn: &Connection, order_by: &str) -> Result<Vec<i64>> {
    let mut statement = conn.prepare(&format!("SELECT id FROM playlists {order_by}"))?;
    let ids = statement
        .query_map([], |row| row.get::<_, i64>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(ids)
}

/// Give the playlist at index `n` of `ids` the position `n`.
fn write_sequence(conn: &Connection, ids: &[i64]) -> Result<()> {
    let mut update = conn.prepare("UPDATE playlists SET position = ?1 WHERE id = ?2")?;
    for (index, id) in ids.iter().enumerate() {
        update.execute(params![i64::try_from(index)?, id])?;
    }
    Ok(())
}

/// The position that a new playlist takes: one after the last.
pub(super) fn next_position(conn: &Connection) -> Result<i64> {
    conn.query_row(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM playlists",
        [],
        |row| row.get(0),
    )
    .context("read the next playlist position")
}

/// Close the gap that a deleted playlist at `position` leaves.
pub(super) fn close_gap(conn: &Connection, position: i64) -> Result<()> {
    conn.execute(
        "UPDATE playlists SET position = position - 1 WHERE position > ?1",
        params![position],
    )
    .context("close the playlist position gap")?;
    Ok(())
}

/// The position of a playlist, or `None` when it does not exist.
pub(super) fn position_of(conn: &Connection, playlist_id: i64) -> Result<Option<i64>> {
    match conn.query_row(
        "SELECT position FROM playlists WHERE id = ?1",
        params![playlist_id],
        |row| row.get(0),
    ) {
        Ok(position) => Ok(Some(position)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(error).context("read the playlist position"),
    }
}

/// Writes the sequence `ids` as the curator order, in one transaction.
///
/// `ids` must name each playlist once. The positions become 0, 1, 2 and so
/// on, with no gap and no duplicate. A call that misses a playlist, repeats
/// one or names an unknown one writes nothing.
///
/// # Errors
///
/// Returns an error when `ids` is not the set of the playlists, or when the
/// database write fails.
pub fn set_playlist_order(conn: &Connection, ids: &[i64]) -> Result<()> {
    let transaction = conn
        .unchecked_transaction()
        .context("begin the playlist order write")?;
    let mut current = ids_in_sequence(&transaction, "ORDER BY id")?;
    let mut wanted = ids.to_vec();
    current.sort_unstable();
    wanted.sort_unstable();
    anyhow::ensure!(
        current == wanted,
        "The playlist order must name each playlist once"
    );
    write_sequence(&transaction, ids)?;
    transaction
        .commit()
        .context("commit the playlist order write")
}

/// Moves one playlist to `to_index` of the curator order, in one
/// transaction. The other playlists keep their relative sequence. An index
/// past the last one moves the playlist last.
///
/// # Errors
///
/// Returns an error when the playlist does not exist or the write fails.
pub fn move_playlist(conn: &Connection, playlist_id: i64, to_index: usize) -> Result<()> {
    let transaction = conn
        .unchecked_transaction()
        .context("begin the playlist move")?;
    let mut ids = ids_in_sequence(&transaction, "ORDER BY position, name COLLATE NOCASE, id")?;
    let from = ids
        .iter()
        .position(|id| *id == playlist_id)
        .context("The playlist to move does not exist")?;
    ids.remove(from);
    ids.insert(to_index.min(ids.len()), playlist_id);
    write_sequence(&transaction, &ids)?;
    transaction.commit().context("commit the playlist move")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{
        inspect_schema, playlist_create, playlist_delete, playlists_list, upgrades,
        SchemaCompatibility, CURRENT_VERSION,
    };

    fn names(conn: &Connection) -> Vec<String> {
        playlists_list(conn)
            .unwrap()
            .into_iter()
            .map(|playlist| playlist.name)
            .collect()
    }

    fn positions(conn: &Connection) -> Vec<i64> {
        let mut statement = conn
            .prepare("SELECT position FROM playlists ORDER BY position")
            .unwrap();
        statement
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<std::result::Result<Vec<_>, _>>()
            .unwrap()
    }

    /// ADR 0089 amendment of 2026-11-08: migration 24 keeps the name sequence
    /// of a version 23 database. The names differ in case and in id order.
    #[test]
    fn adr_0089_curator_order_migration_24_keeps_the_name_sequence() {
        let conn = Connection::open_in_memory().unwrap();
        upgrades::create_fixture(&conn, 23).unwrap();
        for (id, name) in [(1, "zebra"), (2, "Apple"), (3, "mango")] {
            conn.execute(
                "INSERT INTO playlists(id, name) VALUES (?1, ?2)",
                params![id, name],
            )
            .unwrap();
        }

        crate::db::migrate_schema_to(&conn, 24).unwrap();

        assert_eq!(CURRENT_VERSION, 24);
        assert_eq!(inspect_schema(&conn).unwrap(), SchemaCompatibility::Current);
        upgrades::verify_target(&conn, 24).unwrap();
        assert_eq!(names(&conn), ["Apple", "mango", "zebra"]);
        assert_eq!(positions(&conn), [0, 1, 2]);
    }

    /// ADR 0089 amendment of 2026-11-08: a new playlist goes last.
    #[test]
    fn adr_0089_curator_order_create_puts_the_new_playlist_last() {
        let conn = Connection::open_in_memory().unwrap();
        upgrades::create_fixture(&conn, CURRENT_VERSION).unwrap();
        playlist_create(&conn, "Zulu").unwrap();
        playlist_create(&conn, "Alpha").unwrap();
        playlist_create(&conn, "Mike").unwrap();

        assert_eq!(names(&conn), ["Zulu", "Alpha", "Mike"]);
        assert_eq!(positions(&conn), [0, 1, 2]);
    }

    /// ADR 0089 amendment of 2026-11-08: a move writes a sequence with no gap
    /// and no duplicate, and the others keep their relative order.
    #[test]
    fn adr_0089_curator_order_move_writes_a_correct_sequence() {
        let conn = Connection::open_in_memory().unwrap();
        upgrades::create_fixture(&conn, CURRENT_VERSION).unwrap();
        let a = playlist_create(&conn, "A").unwrap();
        let b = playlist_create(&conn, "B").unwrap();
        let c = playlist_create(&conn, "C").unwrap();
        let d = playlist_create(&conn, "D").unwrap();

        move_playlist(&conn, d, 0).unwrap();
        assert_eq!(names(&conn), ["D", "A", "B", "C"]);
        move_playlist(&conn, d, 2).unwrap();
        assert_eq!(names(&conn), ["A", "B", "D", "C"]);
        move_playlist(&conn, a, 99).unwrap();
        assert_eq!(names(&conn), ["B", "D", "C", "A"]);
        assert_eq!(positions(&conn), [0, 1, 2, 3]);
        assert!(move_playlist(&conn, 999, 0).is_err());

        set_playlist_order(&conn, &[c, b, a, d]).unwrap();
        assert_eq!(names(&conn), ["C", "B", "A", "D"]);
        assert!(set_playlist_order(&conn, &[c, b, a]).is_err());
        assert!(set_playlist_order(&conn, &[c, b, a, a]).is_err());
        assert_eq!(names(&conn), ["C", "B", "A", "D"]);
        assert_eq!(positions(&conn), [0, 1, 2, 3]);
    }

    /// ADR 0089 amendment of 2026-11-08: a delete closes the gap and keeps
    /// the relative sequence.
    #[test]
    fn adr_0089_curator_order_delete_closes_the_gap() {
        let conn = Connection::open_in_memory().unwrap();
        upgrades::create_fixture(&conn, CURRENT_VERSION).unwrap();
        playlist_create(&conn, "A").unwrap();
        let b = playlist_create(&conn, "B").unwrap();
        playlist_create(&conn, "C").unwrap();

        playlist_delete(&conn, b).unwrap();

        assert_eq!(names(&conn), ["A", "C"]);
        assert_eq!(positions(&conn), [0, 1]);
        playlist_create(&conn, "D").unwrap();
        assert_eq!(names(&conn), ["A", "C", "D"]);
    }
}
