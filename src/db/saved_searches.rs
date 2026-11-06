//! Saved searches (ADR 0084 Decision 3, packet 012; ADR 0089 Decision 4,
//! packet 007).
//!
//! Migration 21 creates `saved_searches`. Each row is one toolbar search
//! that the operator saved: the trimmed query text, the tab that was open
//! at the save, and the time of the save. Migration 22 adds the optional
//! name column and the table `saved_search_seen`. The rename and the
//! duplicate write the name. `src/db/saved_search_seen.rs` writes the seen
//! table.
//!
//! This module is the only writer of `saved_searches`. The caller gives each
//! time, so no function here reads the clock.

#![warn(clippy::pedantic)]

use anyhow::{Context, Result};
use rusqlite::{params, Connection, ErrorCode, OptionalExtension};

/// The read contract of `saved_searches` after migration 21.
pub(super) const COLUMNS: &[(&str, &[&str])] = &[(
    "saved_searches",
    &["id", "query", "start_tab", "created_at"],
)];

const DDL: &str = r"
CREATE TABLE saved_searches (
    id INTEGER PRIMARY KEY,
    query TEXT NOT NULL,
    start_tab TEXT NOT NULL CHECK (start_tab IN ('releases', 'artists', 'tracks')),
    created_at INTEGER NOT NULL
);
CREATE UNIQUE INDEX saved_searches_query ON saved_searches (query COLLATE NOCASE);
";

/// Migration 21 (ADR 0016 registry, ADR 0084 Decision 3).
pub(super) fn apply(conn: &Connection) -> Result<()> {
    conn.execute_batch(DDL)
        .context("Create the saved search table")
}

/// The tab that was open when the operator saved a search. It is the
/// first tab that the saved search shows.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StoredStartTab {
    Releases,
    Artists,
    Tracks,
}

impl StoredStartTab {
    /// The stored text of this tab.
    #[must_use]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Releases => "releases",
            Self::Artists => "artists",
            Self::Tracks => "tracks",
        }
    }

    fn parse(text: &str) -> Option<Self> {
        match text {
            "releases" => Some(Self::Releases),
            "artists" => Some(Self::Artists),
            "tracks" => Some(Self::Tracks),
            _ => None,
        }
    }
}

/// One stored saved search.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StoredSavedSearch {
    pub(crate) id: i64,
    pub(crate) query: String,
    pub(crate) start_tab: StoredStartTab,
    pub(crate) created_at: i64,
    /// The name that the operator gave, or `None` for the query text.
    pub(crate) name: Option<String>,
}

/// The result of [`insert_saved_search`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SavedSearchInsert {
    /// The search is saved with this id.
    Inserted(i64),
    /// A saved search with the same query, without case, is already
    /// stored. The table does not change.
    AlreadySaved,
}

/// Saves the query `query` with its start tab. `created_at` is the time of
/// the save in Unix seconds, from the caller. A query that is already
/// saved, also with another case, gives [`SavedSearchInsert::AlreadySaved`].
pub(crate) fn insert_saved_search(
    conn: &Connection,
    query: &str,
    start_tab: StoredStartTab,
    created_at: i64,
) -> Result<SavedSearchInsert> {
    insert_row(conn, query, start_tab, None, created_at)
}

/// Inserts one row with an optional name. The query is trimmed and must not
/// be empty. A query that another row holds gives `AlreadySaved`.
fn insert_row(
    conn: &Connection,
    query: &str,
    start_tab: StoredStartTab,
    name: Option<&str>,
    created_at: i64,
) -> Result<SavedSearchInsert> {
    let query = query.trim();
    anyhow::ensure!(!query.is_empty(), "A saved search needs a query");
    // A row with no name uses the migration-21 columns, so the insert also
    // runs on a database at version 21.
    let result = match name {
        None => conn.execute(
            "INSERT INTO saved_searches (query, start_tab, created_at) VALUES (?1, ?2, ?3)",
            params![query, start_tab.as_str(), created_at],
        ),
        Some(name) => conn.execute(
            "INSERT INTO saved_searches (query, start_tab, created_at, name) VALUES (?1, ?2, ?3, ?4)",
            params![query, start_tab.as_str(), created_at, name],
        ),
    };
    match result {
        Ok(_) => Ok(SavedSearchInsert::Inserted(conn.last_insert_rowid())),
        Err(rusqlite::Error::SqliteFailure(error, _))
            if error.code == ErrorCode::ConstraintViolation =>
        {
            Ok(SavedSearchInsert::AlreadySaved)
        }
        Err(error) => Err(error).context("Save the search"),
    }
}

/// The result of a rename of a saved search (ADR 0089 Decision 4).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SavedSearchWrite {
    /// The name changed. A blank name clears it.
    Written,
    /// No row has this id. The table does not change.
    Missing,
}

/// The result of a copy of a saved search (ADR 0089 Decision 4).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SavedSearchCopy {
    /// The copy is stored with this id.
    Copied(i64),
    /// The copy text is already saved by another row. The table does not
    /// change.
    AlreadySaved,
    /// No row has the id that was copied. The table does not change.
    Missing,
}

/// The result of a change of the query text of a saved search (ADR 0087
/// Decision 6, ADR 0089 Decision 4).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SavedSearchQueryWrite {
    /// The stored text is the trimmed new text.
    Written,
    /// Another row holds the new text, without case. The table does not
    /// change.
    AlreadySaved,
    /// No row has this id. The table does not change.
    Missing,
    /// The new text is blank. The table does not change.
    Blank,
}

/// Sets the query text of the saved search `id`. The text is trimmed. The
/// row keeps its name, its start tab and its time. A text that another row
/// holds gives [`SavedSearchQueryWrite::AlreadySaved`], so the unique query
/// index never panics.
pub(crate) fn update_saved_search_query(
    conn: &Connection,
    id: i64,
    query: &str,
) -> Result<SavedSearchQueryWrite> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(SavedSearchQueryWrite::Blank);
    }
    match conn.execute(
        "UPDATE saved_searches SET query = ?2 WHERE id = ?1",
        params![id, query],
    ) {
        Ok(0) => Ok(SavedSearchQueryWrite::Missing),
        Ok(_) => Ok(SavedSearchQueryWrite::Written),
        Err(rusqlite::Error::SqliteFailure(error, _))
            if error.code == ErrorCode::ConstraintViolation =>
        {
            Ok(SavedSearchQueryWrite::AlreadySaved)
        }
        Err(error) => Err(error).context("Change the query of the saved search"),
    }
}

/// Sets the name of the saved search `id`. A blank name clears it, so the
/// row shows its query text again.
pub(crate) fn rename_saved_search(
    conn: &Connection,
    id: i64,
    name: &str,
) -> Result<SavedSearchWrite> {
    let name = name.trim();
    let stored = (!name.is_empty()).then_some(name);
    let changed = conn
        .execute(
            "UPDATE saved_searches SET name = ?2 WHERE id = ?1",
            params![id, stored],
        )
        .context("Rename the saved search")?;
    Ok(if changed == 0 {
        SavedSearchWrite::Missing
    } else {
        SavedSearchWrite::Written
    })
}

/// Copies the saved search `id` as a new row. The copy text is the query of
/// the original followed by " copy", so the unique query index accepts it.
/// The copy name is the label of the original followed by " copy". The row
/// keeps the start tab and gets the time `created_at`.
pub(crate) fn duplicate_saved_search(
    conn: &Connection,
    id: i64,
    created_at: i64,
) -> Result<SavedSearchCopy> {
    let original = conn
        .query_row(
            "SELECT query, start_tab, name FROM saved_searches WHERE id = ?1",
            params![id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            },
        )
        .optional()
        .context("Read the saved search to copy")?;
    let Some((query, start_tab, name)) = original else {
        return Ok(SavedSearchCopy::Missing);
    };
    let start_tab = StoredStartTab::parse(&start_tab)
        .with_context(|| format!("Saved search {id} has an unknown tab"))?;
    let label = name.unwrap_or_else(|| query.clone());
    let copy_name = format!("{label} copy");
    match insert_row(
        conn,
        &format!("{query} copy"),
        start_tab,
        Some(&copy_name),
        created_at,
    )? {
        SavedSearchInsert::Inserted(copy_id) => Ok(SavedSearchCopy::Copied(copy_id)),
        SavedSearchInsert::AlreadySaved => Ok(SavedSearchCopy::AlreadySaved),
    }
}

/// Each saved search, in `id` order.
pub(crate) fn saved_searches(conn: &Connection) -> Result<Vec<StoredSavedSearch>> {
    let mut statement = conn
        .prepare("SELECT id, query, start_tab, created_at, name FROM saved_searches ORDER BY id")
        .context("Read the saved searches")?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        })
        .context("Read the saved searches")?;
    let mut searches = Vec::new();
    for row in rows {
        let (id, query, start_tab, created_at, name) = row.context("Read a saved search")?;
        let start_tab = StoredStartTab::parse(&start_tab)
            .with_context(|| format!("Saved search {id} has the unknown tab {start_tab}"))?;
        searches.push(StoredSavedSearch {
            id,
            query,
            start_tab,
            created_at,
            name,
        });
    }
    Ok(searches)
}

/// Deletes the saved search `id`. Gives `true` when a row was deleted.
pub(crate) fn delete_saved_search(conn: &Connection, id: i64) -> Result<bool> {
    let deleted = conn
        .execute("DELETE FROM saved_searches WHERE id = ?1", params![id])
        .context("Delete the saved search")?;
    Ok(deleted > 0)
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::{
        delete_saved_search, duplicate_saved_search, insert_saved_search, rename_saved_search,
        saved_searches, update_saved_search_query, SavedSearchCopy, SavedSearchInsert,
        SavedSearchQueryWrite, SavedSearchWrite, StoredStartTab,
    };
    use crate::db::{inspect_schema, upgrades, SchemaCompatibility, CURRENT_VERSION, MIGRATIONS};

    /// R12-01 (ADR 0084 Decision 3): migration 21 adds `saved_searches` to
    /// a schema 20 database. Each retained record stays, the new table is
    /// empty. Migration 22 is not applied at target 21.
    #[test]
    fn adr_0084_saved_search_r12_01_version_20_migrates_to_21() {
        let conn = Connection::open_in_memory().unwrap();
        upgrades::create_fixture(&conn, 20).unwrap();
        conn.execute(
            "INSERT INTO feeds(id, feed_url, feed_guid, title) VALUES (1, 'https://example.test/one.xml', 'f1', 'One')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO playlists(id, name, description) VALUES (3, 'Friday Show', NULL)",
            [],
        )
        .unwrap();
        assert_eq!(
            inspect_schema(&conn).unwrap(),
            SchemaCompatibility::UpgradeRequired {
                applied: 20,
                current: MIGRATIONS.len()
            }
        );

        crate::db::migrate_schema_to(&conn, 21).unwrap();

        // Migration 22 is later in the registry, so version 21 is a partial
        // upgrade: the table exists, and migration 22 is not yet applied.
        assert_eq!(
            inspect_schema(&conn).unwrap(),
            SchemaCompatibility::UpgradeRequired {
                applied: 21,
                current: MIGRATIONS.len()
            }
        );
        upgrades::verify_target(&conn, 21).unwrap();
        let name: String = conn
            .query_row(
                "SELECT name FROM schema_migrations WHERE version = 21",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(name, "saved_searches");
        let feed: (String, String) = conn
            .query_row(
                "SELECT feed_guid, title FROM feeds WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(feed, ("f1".to_owned(), "One".to_owned()));
        let playlist: String = conn
            .query_row("SELECT name FROM playlists WHERE id = 3", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(playlist, "Friday Show");
        let saved: i64 = conn
            .query_row("SELECT count(*) FROM saved_searches", [], |row| row.get(0))
            .unwrap();
        assert_eq!(saved, 0, "the new table is empty at version 21");
    }

    /// R12-02 (ADR 0084 Decision 3): the three functions keep the `id`
    /// order. A second insert of "Bluegrass" after "bluegrass" gives
    /// `AlreadySaved` and changes no row.
    #[test]
    fn adr_0084_saved_search_r12_02_insert_list_delete_keep_id_order() {
        let conn = Connection::open_in_memory().unwrap();
        upgrades::create_fixture(&conn, CURRENT_VERSION).unwrap();

        let first = insert_saved_search(&conn, " bluegrass ", StoredStartTab::Tracks, 100).unwrap();
        let second = insert_saved_search(&conn, "ambient", StoredStartTab::Releases, 200).unwrap();
        let third = insert_saved_search(&conn, "jazz", StoredStartTab::Artists, 300).unwrap();
        let (
            SavedSearchInsert::Inserted(first),
            SavedSearchInsert::Inserted(second),
            SavedSearchInsert::Inserted(third),
        ) = (first, second, third)
        else {
            panic!("each new query is inserted");
        };
        assert!(first < second && second < third);

        assert_eq!(
            insert_saved_search(&conn, "Bluegrass", StoredStartTab::Releases, 400).unwrap(),
            SavedSearchInsert::AlreadySaved
        );

        let stored = saved_searches(&conn).unwrap();
        assert_eq!(
            stored
                .iter()
                .map(|search| (search.id, search.query.as_str(), search.start_tab))
                .collect::<Vec<_>>(),
            vec![
                (first, "bluegrass", StoredStartTab::Tracks),
                (second, "ambient", StoredStartTab::Releases),
                (third, "jazz", StoredStartTab::Artists),
            ],
            "the query is trimmed, and the rows come in id order"
        );
        assert_eq!(stored[0].created_at, 100);

        assert!(delete_saved_search(&conn, second).unwrap());
        assert!(!delete_saved_search(&conn, second).unwrap());
        assert_eq!(
            saved_searches(&conn)
                .unwrap()
                .iter()
                .map(|search| search.id)
                .collect::<Vec<_>>(),
            vec![first, third]
        );
    }

    /// The table refuses an empty query and a tab outside the three tabs.
    #[test]
    fn adr_0084_saved_search_empty_query_and_unknown_tab_are_refused() {
        let conn = Connection::open_in_memory().unwrap();
        upgrades::create_fixture(&conn, CURRENT_VERSION).unwrap();
        assert!(insert_saved_search(&conn, "   ", StoredStartTab::Releases, 1).is_err());
        assert!(conn
            .execute(
                "INSERT INTO saved_searches (query, start_tab, created_at) VALUES ('x', 'feeds', 1)",
                [],
            )
            .is_err());
    }

    /// ADR 0089 Decision 4 (packet 007): migration 22 adds the name column
    /// and the seen table to a schema 21 database. The retained saved search
    /// keeps its query, its name is empty, and the seen table is empty.
    #[test]
    fn adr_0089_saved_search_seen_r22_01_version_21_migrates_to_22() {
        let conn = Connection::open_in_memory().unwrap();
        upgrades::create_fixture(&conn, 21).unwrap();
        insert_saved_search(&conn, "bluegrass", StoredStartTab::Tracks, 100).unwrap();
        assert_eq!(
            inspect_schema(&conn).unwrap(),
            SchemaCompatibility::UpgradeRequired {
                applied: 21,
                current: MIGRATIONS.len()
            }
        );

        crate::db::migrate_schema_to(&conn, 22).unwrap();

        assert_eq!(CURRENT_VERSION, 22);
        assert_eq!(inspect_schema(&conn).unwrap(), SchemaCompatibility::Current);
        upgrades::verify_target(&conn, 22).unwrap();
        let stored = saved_searches(&conn).unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].query, "bluegrass");
        assert_eq!(stored[0].name, None);
        let seen: i64 = conn
            .query_row("SELECT count(*) FROM saved_search_seen", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(seen, 0, "the seen table is empty after migration 22");
    }

    /// ADR 0089 Decision 4 (packet 007): a failed migration 22 rolls back. The
    /// name column and the seen table do not remain, the ledger does not
    /// record 22, and the saved search of version 21 keeps its query.
    #[test]
    fn adr_0089_saved_search_seen_r22_04_failed_migration_rolls_back() {
        for boundary in [
            crate::db::MigrationBoundary::BeforeApply,
            crate::db::MigrationBoundary::AfterApply,
            crate::db::MigrationBoundary::AfterRecord,
        ] {
            let conn = Connection::open_in_memory().unwrap();
            upgrades::create_fixture(&conn, 21).unwrap();
            insert_saved_search(&conn, "bluegrass", StoredStartTab::Tracks, 100).unwrap();

            let result = crate::db::migrate_schema_with(&conn, 22, |version, reached| {
                anyhow::ensure!(
                    version != 22 || reached != boundary,
                    "Injected migration failure"
                );
                Ok(())
            });

            assert!(result.is_err(), "the injected failure stops migration 22");
            assert!(conn.is_autocommit());
            upgrades::verify_target(&conn, 21).unwrap();
            let name_columns: i64 = conn
                .query_row(
                    "SELECT count(*) FROM pragma_table_info('saved_searches') WHERE name = 'name'",
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(name_columns, 0, "the name column is rolled back");
            assert_eq!(saved_searches_query_count(&conn), 1);
        }
    }

    fn saved_searches_query_count(conn: &Connection) -> i64 {
        conn.query_row("SELECT count(*) FROM saved_searches", [], |row| row.get(0))
            .unwrap()
    }

    /// ADR 0089 Decision 4 (packet 007): rename sets or clears the name, and
    /// reports a missing row. Duplicate copies the query with " copy" and
    /// the name with " copy", and a copy that is already saved gives the
    /// typed result.
    #[test]
    fn adr_0089_saved_search_seen_r22_02_rename_and_duplicate_return_typed_results() {
        let conn = Connection::open_in_memory().unwrap();
        upgrades::create_fixture(&conn, CURRENT_VERSION).unwrap();
        let SavedSearchInsert::Inserted(first) =
            insert_saved_search(&conn, "bluegrass", StoredStartTab::Tracks, 100).unwrap()
        else {
            panic!("the first query is inserted");
        };

        assert_eq!(
            rename_saved_search(&conn, first, " Old time ").unwrap(),
            SavedSearchWrite::Written
        );
        assert_eq!(
            saved_searches(&conn).unwrap()[0].name.as_deref(),
            Some("Old time")
        );
        assert_eq!(
            rename_saved_search(&conn, first, "   ").unwrap(),
            SavedSearchWrite::Written
        );
        assert_eq!(saved_searches(&conn).unwrap()[0].name, None);
        assert_eq!(
            rename_saved_search(&conn, 999, "x").unwrap(),
            SavedSearchWrite::Missing
        );

        let SavedSearchCopy::Copied(copy_id) = duplicate_saved_search(&conn, first, 300).unwrap()
        else {
            panic!("the copy is stored");
        };
        let stored = saved_searches(&conn).unwrap();
        let copied = stored.iter().find(|search| search.id == copy_id).unwrap();
        assert_eq!(copied.query, "bluegrass copy");
        assert_eq!(copied.name.as_deref(), Some("bluegrass copy"));
        assert_eq!(copied.start_tab, StoredStartTab::Tracks);

        rename_saved_search(&conn, first, "Old time").unwrap();
        let SavedSearchCopy::Copied(second_copy) =
            duplicate_saved_search(&conn, copy_id, 400).unwrap()
        else {
            panic!("a copy of the copy is stored");
        };
        let listed = saved_searches(&conn).unwrap();
        let named = listed
            .iter()
            .find(|search| search.id == second_copy)
            .unwrap();
        assert_eq!(named.query, "bluegrass copy copy");
        assert_eq!(named.name.as_deref(), Some("bluegrass copy copy"));
        assert_eq!(
            duplicate_saved_search(&conn, first, 500).unwrap(),
            SavedSearchCopy::AlreadySaved,
            "the copy query \"bluegrass copy\" is already saved"
        );
        assert_eq!(
            duplicate_saved_search(&conn, 999, 1).unwrap(),
            SavedSearchCopy::Missing
        );
    }

    /// ADR 0087 Decision 6 (packet 007): the query text of a saved search
    /// changes in place. The name, the tab and the time stay. A blank text,
    /// a missing row and a text that another row holds change nothing.
    #[test]
    fn adr_0089_saved_search_query_r22_03_update_query_returns_typed_results() {
        let conn = Connection::open_in_memory().unwrap();
        upgrades::create_fixture(&conn, CURRENT_VERSION).unwrap();
        let SavedSearchInsert::Inserted(first) =
            insert_saved_search(&conn, "bluegrass", StoredStartTab::Tracks, 100).unwrap()
        else {
            panic!("the first query is inserted");
        };
        let SavedSearchInsert::Inserted(second) =
            insert_saved_search(&conn, "ambient", StoredStartTab::Releases, 200).unwrap()
        else {
            panic!("the second query is inserted");
        };
        rename_saved_search(&conn, first, "Old time").unwrap();

        assert_eq!(
            update_saved_search_query(&conn, first, "  bluegrass banjo  ").unwrap(),
            SavedSearchQueryWrite::Written
        );
        let stored = saved_searches(&conn).unwrap();
        assert_eq!(stored[0].query, "bluegrass banjo", "the text is trimmed");
        assert_eq!(stored[0].name.as_deref(), Some("Old time"));
        assert_eq!(stored[0].start_tab, StoredStartTab::Tracks);
        assert_eq!(stored[0].created_at, 100);

        assert_eq!(
            update_saved_search_query(&conn, first, "AMBIENT").unwrap(),
            SavedSearchQueryWrite::AlreadySaved,
            "another row holds \"ambient\" without case"
        );
        assert_eq!(
            update_saved_search_query(&conn, first, "  ").unwrap(),
            SavedSearchQueryWrite::Blank
        );
        assert_eq!(
            update_saved_search_query(&conn, 999, "jazz").unwrap(),
            SavedSearchQueryWrite::Missing
        );
        let after = saved_searches(&conn).unwrap();
        assert_eq!(
            after[0].query, "bluegrass banjo",
            "a refused text changes nothing"
        );
        assert_eq!(after[1].id, second);
        assert_eq!(after[1].query, "ambient");
    }
}
