//! Saved search names and seen results (ADR 0089 Decision 4, packet 007).
//!
//! Migration 22 adds the nullable `name` column to `saved_searches`, and
//! the table `saved_search_seen`. Each row of the table is one result key
//! that the operator saw in one saved search. The count of new results of
//! a saved search is its results minus these keys (ADR 0089 Decision 4).
//!
//! `src/db/saved_searches.rs` owns the writes of `saved_searches`. This
//! module owns migration 22, its read contract, and the writers of
//! `saved_search_seen`: `read_seen` and `mark_seen`.

#![warn(clippy::pedantic)]

use std::collections::BTreeSet;

use anyhow::{Context, Result};
use rusqlite::{params, Connection};

/// The key that each seen set holds once it is initialised. A saved search
/// whose first run had no result still gets this row, so its later results
/// count as new. Result keys never start with this text.
pub(crate) const SEEN_INITIALISED_MARKER: &str = "initialised";

/// The seen keys of the saved search `id`. `None` when no row exists, which
/// means the saved search has not been initialised yet (ADR 0089 Decision
/// 4).
pub(crate) fn read_seen(conn: &Connection, id: i64) -> Result<Option<BTreeSet<String>>> {
    let mut statement = conn
        .prepare("SELECT result_key FROM saved_search_seen WHERE saved_search_id = ?1")
        .context("Read the seen results of the saved search")?;
    let keys = statement
        .query_map(params![id], |row| row.get::<_, String>(0))
        .context("Read the seen results of the saved search")?
        .collect::<rusqlite::Result<BTreeSet<String>>>()
        .context("Read a seen result of the saved search")?;
    Ok((!keys.is_empty()).then_some(keys))
}

/// Stores each key of `keys` as seen for the saved search `id`, and the
/// initialised marker. A key that is already stored changes nothing. The
/// caller gives the keys, so no function here reads the Library or the
/// Index.
pub(crate) fn mark_seen(conn: &Connection, id: i64, keys: &BTreeSet<String>) -> Result<()> {
    let transaction = conn
        .unchecked_transaction()
        .context("Start the seen results write")?;
    {
        let mut statement = transaction
            .prepare(
                "INSERT OR IGNORE INTO saved_search_seen (saved_search_id, result_key) VALUES (?1, ?2)",
            )
            .context("Prepare the seen results write")?;
        statement
            .execute(params![id, SEEN_INITIALISED_MARKER])
            .context("Store the seen marker")?;
        for key in keys {
            statement
                .execute(params![id, key])
                .context("Store a seen result")?;
        }
    }
    transaction
        .commit()
        .context("Commit the seen results write")
}

/// The read contract of `saved_searches` after migration 22: the
/// migration-21 columns, then the `name` column that migration 22 appends.
pub(super) const EXTENDED_COLUMNS: &[(&str, &[&str])] = &[(
    "saved_searches",
    &["id", "query", "start_tab", "created_at", "name"],
)];

/// The read contract of the table that migration 22 creates.
pub(super) const COLUMNS: &[(&str, &[&str])] =
    &[("saved_search_seen", &["saved_search_id", "result_key"])];

const DDL: &str = r"
ALTER TABLE saved_searches ADD COLUMN name TEXT NULL;
CREATE TABLE saved_search_seen (
    saved_search_id INTEGER NOT NULL REFERENCES saved_searches(id) ON DELETE CASCADE,
    result_key TEXT NOT NULL,
    PRIMARY KEY (saved_search_id, result_key)
);
";

/// Migration 22 (ADR 0016 registry, ADR 0089 Decision 4).
pub(super) fn apply(conn: &Connection) -> Result<()> {
    conn.execute_batch(DDL)
        .context("Add the saved search name and the seen result table")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use rusqlite::Connection;

    use super::{mark_seen, read_seen, SEEN_INITIALISED_MARKER};
    use crate::db::saved_searches::{
        delete_saved_search, insert_saved_search, SavedSearchInsert, StoredStartTab,
    };
    use crate::db::{upgrades, CURRENT_VERSION};

    /// A database with one saved search. Gives the connection and its id.
    fn setup_db() -> (Connection, i64) {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        upgrades::create_fixture(&conn, CURRENT_VERSION).unwrap();
        let SavedSearchInsert::Inserted(id) =
            insert_saved_search(&conn, "jazz", StoredStartTab::Releases, 1).unwrap()
        else {
            panic!("the first saved search is inserted");
        };
        (conn, id)
    }

    fn keys(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    /// ADR 0089 Decision 4, packet 007 Part C: a saved search with no seen
    /// row is not initialised, and `mark_seen` stores its keys and the marker.
    #[test]
    fn mark_seen_initialises_a_saved_search_and_read_returns_its_keys() {
        let (conn, id) = setup_db();
        assert_eq!(read_seen(&conn, id).unwrap(), None);

        mark_seen(&conn, id, &keys(&["artist:a", "release:feed:b"])).unwrap();

        let stored = read_seen(&conn, id).unwrap().expect("initialised");
        assert!(stored.contains("artist:a"));
        assert!(stored.contains("release:feed:b"));
        assert!(stored.contains(SEEN_INITIALISED_MARKER));
    }

    /// A key that is stored already changes nothing, so a second mark is safe.
    #[test]
    fn mark_seen_twice_keeps_each_key_once() {
        let (conn, id) = setup_db();
        mark_seen(&conn, id, &keys(&["artist:a"])).unwrap();
        mark_seen(&conn, id, &keys(&["artist:a", "artist:b"])).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM saved_search_seen WHERE saved_search_id = ?1",
                [id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 3, "the marker, a and b");
    }

    /// An empty first run still initialises the saved search, so later
    /// results count as new.
    #[test]
    fn mark_seen_with_no_keys_still_initialises() {
        let (conn, id) = setup_db();
        mark_seen(&conn, id, &BTreeSet::new()).unwrap();
        assert!(read_seen(&conn, id).unwrap().is_some());
    }

    /// Deleting a saved search removes its seen rows (ON DELETE CASCADE).
    #[test]
    fn deleting_a_saved_search_removes_its_seen_rows() {
        let (conn, id) = setup_db();
        mark_seen(&conn, id, &keys(&["artist:a"])).unwrap();
        assert!(delete_saved_search(&conn, id).unwrap());
        let count: i64 = conn
            .query_row("SELECT count(*) FROM saved_search_seen", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 0);
    }
}
