//! Saved searches (ADR 0084 Decision 3, packet 012; ADR 0089 Decision 4,
//! packet 007; ADR 0087 Decision 6).
//!
//! Migration 21 creates `saved_searches`. Each row is one toolbar search
//! that the operator saved: the trimmed query text, the tab that was open
//! at the save, and the time of the save. Migration 22 adds the optional
//! name column and the table `saved_search_seen`, which
//! `src/db/saved_search_seen.rs` owns. Migration 23 (this file) gives each
//! row a name, makes the name required and unique without case, and drops
//! the unique index of the query.
//!
//! A saved search is keyed by its name (operator decision, 2026-11-06). The
//! row id stays the primary key and the navigation key. Two rows may hold
//! one query. A name that another row holds gets " 2", " 3" and so on.
//!
//! This module is the only writer of `saved_searches`. The caller gives each
//! time, so no function here reads the clock.

#![warn(clippy::pedantic)]

use std::collections::HashSet;

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};

use super::saved_search_seen;

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

/// The table after migration 23. It replaces the table of migration 22 in
/// one rebuild, so `name` is required.
const NAMED_DDL: &str = r"
CREATE TABLE saved_searches_named (
    id INTEGER PRIMARY KEY,
    query TEXT NOT NULL,
    start_tab TEXT NOT NULL CHECK (start_tab IN ('releases', 'artists', 'tracks')),
    created_at INTEGER NOT NULL,
    name TEXT NOT NULL CHECK (trim(name) <> '')
);
";

/// The row of `saved_searches` that migration 23 names.
struct NameSource {
    id: i64,
    query: String,
    name: Option<String>,
}

/// The name that the operator gave, trimmed. `None` for a blank name.
fn given_name(source: &NameSource) -> Option<&str> {
    source
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
}

/// Migration 23 (ADR 0016 registry, ADR 0089 Decision 4, operator decision
/// of 2026-11-06). Each row gets a name, `name` becomes required and unique
/// without case, and the unique index of the query is dropped.
///
/// `SQLite` cannot add `NOT NULL` with `ALTER TABLE`, and it cannot drop an
/// index that a rebuild does not remake. The migration rebuilds the table:
///
/// 1. It sets the seen rows aside, and drops `saved_search_seen`. No seen
///    row refers to `saved_searches` after this step, so the next drop
///    cascades to nothing.
/// 2. It creates `saved_searches_named`, and copies each row with its id.
/// 3. It drops `saved_searches`, which also drops its query index. It renames
///    the new table, and creates the unique index of the name.
/// 4. It creates `saved_search_seen` again, and copies the seen rows back.
///
/// The migration runs inside the transaction of the registry, so a failure
/// rolls back every step.
pub(super) fn apply_names(conn: &Connection) -> Result<()> {
    let sources = name_sources(conn)?;
    let names = assign_names(&sources);
    saved_search_seen::set_aside_for_rebuild(conn)?;
    conn.execute_batch(NAMED_DDL)
        .context("Create the named saved search table")?;
    {
        let mut copy = conn
            .prepare(
                "INSERT INTO saved_searches_named (id, query, start_tab, created_at, name)
                 SELECT id, query, start_tab, created_at, ?1 FROM saved_searches WHERE id = ?2",
            )
            .context("Prepare the copy of the saved searches")?;
        for (id, name) in &names {
            copy.execute(params![name, id])
                .context("Copy a saved search with its name")?;
        }
    }
    conn.execute_batch(
        "DROP TABLE saved_searches;
         ALTER TABLE saved_searches_named RENAME TO saved_searches;
         CREATE UNIQUE INDEX saved_searches_name ON saved_searches (name COLLATE NOCASE);",
    )
    .context("Replace the saved search table")?;
    saved_search_seen::restore_after_rebuild(conn)
}

/// Each row of `saved_searches` in `id` order, for migration 23.
fn name_sources(conn: &Connection) -> Result<Vec<NameSource>> {
    let mut statement = conn
        .prepare("SELECT id, query, name FROM saved_searches ORDER BY id")
        .context("Read the saved searches to name")?;
    let rows = statement
        .query_map([], |row| {
            Ok(NameSource {
                id: row.get(0)?,
                query: row.get(1)?,
                name: row.get(2)?,
            })
        })
        .context("Read the saved searches to name")?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .context("Read a saved search to name")
}

/// The name of each row after migration 23, as `(id, name)` in `id` order.
/// A name that the operator gave keeps its text, so those names are claimed
/// first, in `id` order. A row with no given name takes its query text.
fn assign_names(sources: &[NameSource]) -> Vec<(i64, String)> {
    let mut claimed = HashSet::new();
    let mut names = Vec::with_capacity(sources.len());
    for source in sources.iter().filter(|source| given_name(source).is_some()) {
        let base = given_name(source).unwrap_or_default();
        names.push((source.id, claim_free_name(base, &mut claimed)));
    }
    for source in sources.iter().filter(|source| given_name(source).is_none()) {
        names.push((
            source.id,
            claim_free_name(source.query.trim(), &mut claimed),
        ));
    }
    names.sort_by_key(|(id, _)| *id);
    names
}

/// Claims the first free form of `base` in `claimed`, without case. The
/// forms are `base`, then `base 2`, `base 3` and so on.
fn claim_free_name(base: &str, claimed: &mut HashSet<String>) -> String {
    let mut candidate = base.to_owned();
    let mut number = 2;
    while !claimed.insert(candidate.to_ascii_lowercase()) {
        candidate = format!("{base} {number}");
        number += 1;
    }
    candidate
}

/// The first free form of `base` in the table (ADR 0089 Decision 4). The
/// row `except` may keep its own name, so it does not block the form.
fn free_name(conn: &Connection, base: &str, except: Option<i64>) -> Result<String> {
    let mut candidate = base.to_owned();
    let mut number = 2;
    while name_held(conn, &candidate, except)? {
        candidate = format!("{base} {number}");
        number += 1;
    }
    Ok(candidate)
}

/// Tells if another row than `except` holds `name`, without case.
fn name_held(conn: &Connection, name: &str, except: Option<i64>) -> Result<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM saved_searches
             WHERE name = ?1 COLLATE NOCASE AND (?2 IS NULL OR id <> ?2))",
        params![name, except],
        |row| row.get(0),
    )
    .context("Check the name of the saved searches")
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
    /// The name of the saved search. It is required and unique without case.
    pub(crate) name: String,
}

/// Saves the query `query` with its start tab. `created_at` is the time of
/// the save in Unix seconds, from the caller. The name is the query text,
/// or its first free form (ADR 0089 Decision 4). The table keeps a second
/// row with a query that another row holds. The caller decides that
/// (ADR 0084 Decision 3, `is_query_saved`). Gives the id of the new row.
pub(crate) fn insert_saved_search(
    conn: &Connection,
    query: &str,
    start_tab: StoredStartTab,
    created_at: i64,
) -> Result<i64> {
    insert_row(conn, query, start_tab, query, created_at)
}

/// Inserts one row. The query is trimmed and must not be empty. The name is
/// the first free form of `base_name`.
fn insert_row(
    conn: &Connection,
    query: &str,
    start_tab: StoredStartTab,
    base_name: &str,
    created_at: i64,
) -> Result<i64> {
    let query = query.trim();
    anyhow::ensure!(!query.is_empty(), "A saved search needs a query");
    let name = free_name(conn, base_name.trim(), None)?;
    conn.execute(
        "INSERT INTO saved_searches (query, start_tab, created_at, name) VALUES (?1, ?2, ?3, ?4)",
        params![query, start_tab.as_str(), created_at, name],
    )
    .context("Save the search")?;
    Ok(conn.last_insert_rowid())
}

/// The result of a rename of a saved search (ADR 0089 Decision 4).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SavedSearchWrite {
    /// The name changed. A blank name becomes the query text, or its first
    /// free form.
    Written,
    /// No row has this id. The table does not change.
    Missing,
    /// Another row holds the name, without case. The table does not change.
    NameTaken,
}

/// The result of a copy of a saved search (ADR 0089 Decision 4).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SavedSearchCopy {
    /// The copy is stored with this id.
    Copied(i64),
    /// No row has the id that was copied. The table does not change.
    Missing,
}

/// The result of a change of the query text of a saved search (ADR 0087
/// Decision 6, ADR 0089 Decision 4).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SavedSearchQueryWrite {
    /// The stored text is the trimmed new text.
    Written,
    /// No row has this id. The table does not change.
    Missing,
    /// The new text is blank. The table does not change.
    Blank,
}

/// Sets the query text of the saved search `id`. The text is trimmed. The
/// row keeps its start tab and its time. A name that equals the old query
/// text, without case, was never given by the operator. It follows the new
/// text, in its first free form. Any other name stays.
pub(crate) fn update_saved_search_query(
    conn: &Connection,
    id: i64,
    query: &str,
) -> Result<SavedSearchQueryWrite> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(SavedSearchQueryWrite::Blank);
    }
    let Some((old_query, name)) = conn
        .query_row(
            "SELECT query, name FROM saved_searches WHERE id = ?1",
            params![id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()
        .context("Read the saved search to change")?
    else {
        return Ok(SavedSearchQueryWrite::Missing);
    };
    let name = if name.eq_ignore_ascii_case(&old_query) {
        free_name(conn, query, Some(id))?
    } else {
        name
    };
    conn.execute(
        "UPDATE saved_searches SET query = ?2, name = ?3 WHERE id = ?1",
        params![id, query, name],
    )
    .context("Change the query of the saved search")?;
    Ok(SavedSearchQueryWrite::Written)
}

/// Sets the name of the saved search `id`. A name that another row holds,
/// without case, is refused, and the table does not change. A blank name
/// becomes the query text, or its first free form.
pub(crate) fn rename_saved_search(
    conn: &Connection,
    id: i64,
    name: &str,
) -> Result<SavedSearchWrite> {
    let Some(query) = query_of(conn, id)? else {
        return Ok(SavedSearchWrite::Missing);
    };
    let name = name.trim();
    let name = if name.is_empty() {
        free_name(conn, &query, Some(id))?
    } else if name_held(conn, name, Some(id))? {
        return Ok(SavedSearchWrite::NameTaken);
    } else {
        name.to_owned()
    };
    conn.execute(
        "UPDATE saved_searches SET name = ?2 WHERE id = ?1",
        params![id, name],
    )
    .context("Rename the saved search")?;
    Ok(SavedSearchWrite::Written)
}

/// The query text of the saved search `id`, or `None` for no row.
fn query_of(conn: &Connection, id: i64) -> Result<Option<String>> {
    conn.query_row(
        "SELECT query FROM saved_searches WHERE id = ?1",
        params![id],
        |row| row.get(0),
    )
    .optional()
    .context("Read the saved search")
}

/// Copies the saved search `id` as a new row. The copy keeps the query and
/// the start tab. Its name is the name of the original followed by " copy",
/// or its first free form (ADR 0089 Decision 4). The row gets the time
/// `created_at`.
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
                    row.get::<_, String>(2)?,
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
    let copy_id = insert_row(conn, &query, start_tab, &format!("{name} copy"), created_at)?;
    Ok(SavedSearchCopy::Copied(copy_id))
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
                row.get::<_, String>(4)?,
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
    use std::collections::BTreeSet;

    use rusqlite::{params, Connection};

    use crate::db::saved_search_seen;

    use super::{
        delete_saved_search, duplicate_saved_search, insert_saved_search, rename_saved_search,
        saved_searches, update_saved_search_query, SavedSearchCopy, SavedSearchQueryWrite,
        SavedSearchWrite, StoredStartTab,
    };
    use crate::db::{inspect_schema, upgrades, SchemaCompatibility, CURRENT_VERSION, MIGRATIONS};

    /// Inserts a row of a version 22 or older table, without the name column.
    fn insert_raw(conn: &Connection, query: &str, tab: &str, created_at: i64) {
        conn.execute(
            "INSERT INTO saved_searches (query, start_tab, created_at) VALUES (?1, ?2, ?3)",
            params![query, tab, created_at],
        )
        .unwrap();
    }

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

    /// R12-02 (ADR 0084 Decision 3, changed by the operator decision of
    /// 2026-11-06): the functions keep the `id` order. A second insert of
    /// "Bluegrass" after "bluegrass" is stored as a new row. Its name is the
    /// first free form, "Bluegrass 2". The action keeps a saved query from
    /// a second save, not the table.
    #[test]
    fn adr_0084_saved_search_r12_02_insert_list_delete_keep_id_order() {
        let conn = Connection::open_in_memory().unwrap();
        upgrades::create_fixture(&conn, CURRENT_VERSION).unwrap();

        let first = insert_saved_search(&conn, " bluegrass ", StoredStartTab::Tracks, 100).unwrap();
        let second = insert_saved_search(&conn, "ambient", StoredStartTab::Releases, 200).unwrap();
        let third = insert_saved_search(&conn, "jazz", StoredStartTab::Artists, 300).unwrap();
        assert!(first < second && second < third);
        let fourth =
            insert_saved_search(&conn, "Bluegrass", StoredStartTab::Releases, 400).unwrap();

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
                (fourth, "Bluegrass", StoredStartTab::Releases),
            ],
            "the query is trimmed, and the rows come in id order"
        );
        assert_eq!(stored[0].created_at, 100);
        assert_eq!(
            stored
                .iter()
                .map(|search| search.name.as_str())
                .collect::<Vec<_>>(),
            vec!["bluegrass", "ambient", "jazz", "Bluegrass 2"]
        );

        assert!(delete_saved_search(&conn, second).unwrap());
        assert!(!delete_saved_search(&conn, second).unwrap());
        assert_eq!(
            saved_searches(&conn)
                .unwrap()
                .iter()
                .map(|search| search.id)
                .collect::<Vec<_>>(),
            vec![first, third, fourth]
        );
    }

    /// The table refuses an empty query, a blank name and a tab outside the
    /// three tabs.
    #[test]
    fn adr_0084_saved_search_empty_query_and_unknown_tab_are_refused() {
        let conn = Connection::open_in_memory().unwrap();
        upgrades::create_fixture(&conn, CURRENT_VERSION).unwrap();
        assert!(insert_saved_search(&conn, "   ", StoredStartTab::Releases, 1).is_err());
        assert!(conn
            .execute(
                "INSERT INTO saved_searches (query, start_tab, created_at, name) VALUES ('x', 'feeds', 1, 'x')",
                [],
            )
            .is_err());
        assert!(conn
            .execute(
                "INSERT INTO saved_searches (query, start_tab, created_at, name) VALUES ('x', 'releases', 1, '  ')",
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
        insert_raw(&conn, "bluegrass", "tracks", 100);
        assert_eq!(
            inspect_schema(&conn).unwrap(),
            SchemaCompatibility::UpgradeRequired {
                applied: 21,
                current: MIGRATIONS.len()
            }
        );

        crate::db::migrate_schema_to(&conn, 22).unwrap();

        assert_eq!(
            inspect_schema(&conn).unwrap(),
            SchemaCompatibility::UpgradeRequired {
                applied: 22,
                current: MIGRATIONS.len()
            },
            "migration 23 is still pending at version 22"
        );
        upgrades::verify_target(&conn, 22).unwrap();
        let (query, name): (String, Option<String>) = conn
            .query_row("SELECT query, name FROM saved_searches", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .unwrap();
        assert_eq!(query, "bluegrass");
        assert_eq!(name, None, "migration 22 leaves the name empty");
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
            insert_raw(&conn, "bluegrass", "tracks", 100);

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
    /// reports a missing row. Duplicate keeps the query and names the copy
    /// with " copy". Each later copy takes the next free number.
    #[test]
    fn adr_0089_saved_search_seen_r22_02_rename_and_duplicate_return_typed_results() {
        let conn = Connection::open_in_memory().unwrap();
        upgrades::create_fixture(&conn, CURRENT_VERSION).unwrap();
        let first = insert_saved_search(&conn, "bluegrass", StoredStartTab::Tracks, 100).unwrap();

        assert_eq!(
            rename_saved_search(&conn, first, " Old time ").unwrap(),
            SavedSearchWrite::Written
        );
        assert_eq!(saved_searches(&conn).unwrap()[0].name, "Old time");
        assert_eq!(
            rename_saved_search(&conn, first, "   ").unwrap(),
            SavedSearchWrite::Written
        );
        assert_eq!(saved_searches(&conn).unwrap()[0].name, "bluegrass");
        assert_eq!(
            rename_saved_search(&conn, 999, "x").unwrap(),
            SavedSearchWrite::Missing
        );
        rename_saved_search(&conn, first, "Old time").unwrap();

        let SavedSearchCopy::Copied(copy_id) = duplicate_saved_search(&conn, first, 300).unwrap()
        else {
            panic!("the copy is stored");
        };
        let stored = saved_searches(&conn).unwrap();
        let copied = stored.iter().find(|search| search.id == copy_id).unwrap();
        assert_eq!(copied.query, "bluegrass", "the copy keeps the query");
        assert_eq!(copied.name, "Old time copy");
        assert_eq!(copied.start_tab, StoredStartTab::Tracks);

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
        assert_eq!(named.query, "bluegrass");
        assert_eq!(named.name, "Old time copy copy");

        let SavedSearchCopy::Copied(third_copy) =
            duplicate_saved_search(&conn, first, 500).unwrap()
        else {
            panic!("a second copy of the original is stored");
        };
        let third = saved_searches(&conn)
            .unwrap()
            .into_iter()
            .find(|search| search.id == third_copy)
            .unwrap();
        assert_eq!(third.query, "bluegrass");
        assert_eq!(
            third.name, "Old time copy 2",
            "the name \"Old time copy\" is taken, so the next free number is used"
        );
        assert_eq!(
            duplicate_saved_search(&conn, 999, 1).unwrap(),
            SavedSearchCopy::Missing
        );
    }

    /// ADR 0087 Decision 6 (packet 007): the query text of a saved search
    /// changes in place. The tab and the time stay. A name that the operator
    /// gave stays. A blank text and a missing row change nothing. A text
    /// that another row holds is stored, because the query is not unique.
    #[test]
    fn adr_0089_saved_search_query_r22_03_update_query_returns_typed_results() {
        let conn = Connection::open_in_memory().unwrap();
        upgrades::create_fixture(&conn, CURRENT_VERSION).unwrap();
        let first = insert_saved_search(&conn, "bluegrass", StoredStartTab::Tracks, 100).unwrap();
        insert_saved_search(&conn, "ambient", StoredStartTab::Releases, 200).unwrap();
        rename_saved_search(&conn, first, "Old time").unwrap();

        assert_eq!(
            update_saved_search_query(&conn, first, "  bluegrass banjo  ").unwrap(),
            SavedSearchQueryWrite::Written
        );
        let stored = saved_searches(&conn).unwrap();
        assert_eq!(stored[0].query, "bluegrass banjo", "the text is trimmed");
        assert_eq!(stored[0].name, "Old time", "the given name stays");
        assert_eq!(stored[0].start_tab, StoredStartTab::Tracks);
        assert_eq!(stored[0].created_at, 100);

        assert_eq!(
            update_saved_search_query(&conn, first, "AMBIENT").unwrap(),
            SavedSearchQueryWrite::Written,
            "another row holds \"ambient\", and the query may repeat"
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
        assert_eq!(after[0].query, "AMBIENT");
        assert_eq!(after[1].query, "ambient");
    }

    /// ADR 0089 Decision 4 (operator decision of 2026-11-06, migration 23):
    /// the upgrade names each row of a version 22 database. A NULL or blank
    /// name takes the query text. A name that collides, without case, gets
    /// " 2", " 3" and so on. The names that the operator gave come first.
    /// The rows keep their id, their query, their tab, their time and their
    /// seen results. The query index is dropped.
    #[test]
    fn adr_0089_saved_search_name_r23_01_version_22_names_each_row() {
        let conn = Connection::open_in_memory().unwrap();
        upgrades::create_fixture(&conn, 22).unwrap();
        // The queries differ without case, as version 22 requires.
        let rows = [
            ("Jazz", None, "releases"),
            ("ambient", Some("jazz"), "artists"),
            ("bluegrass", Some("   "), "tracks"),
            ("blues", Some("bluegrass"), "tracks"),
            ("old time", Some("Old Time"), "tracks"),
            ("x", Some("old time"), "artists"),
        ];
        for (index, (query, name, tab)) in rows.iter().enumerate() {
            conn.execute(
                "INSERT INTO saved_searches (query, start_tab, created_at, name) VALUES (?1, ?2, ?3, ?4)",
                params![query, tab, i64::try_from(index).unwrap() + 10, name],
            )
            .unwrap();
        }
        let keys = |items: &[&str]| -> BTreeSet<String> {
            items.iter().map(|item| (*item).to_owned()).collect()
        };
        saved_search_seen::mark_seen(&conn, 1, &keys(&["artist:a", "release:feed:b"])).unwrap();
        saved_search_seen::mark_seen(&conn, 4, &keys(&["artist:c"])).unwrap();

        crate::db::migrate_schema_to(&conn, 23).unwrap();

        assert_eq!(CURRENT_VERSION, 23);
        assert_eq!(inspect_schema(&conn).unwrap(), SchemaCompatibility::Current);
        upgrades::verify_target(&conn, 23).unwrap();
        let stored = saved_searches(&conn).unwrap();
        assert_eq!(
            stored
                .iter()
                .map(|search| (search.id, search.query.as_str(), search.name.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (1, "Jazz", "Jazz 2"),
                (2, "ambient", "jazz"),
                (3, "bluegrass", "bluegrass 2"),
                (4, "blues", "bluegrass"),
                (5, "old time", "Old Time"),
                (6, "x", "old time 2"),
            ]
        );
        assert_eq!(stored[0].start_tab, StoredStartTab::Releases);
        assert_eq!(stored[1].start_tab, StoredStartTab::Artists);
        assert_eq!(stored[0].created_at, 10);
        assert_eq!(stored[5].created_at, 15);

        let seen_rows: i64 = conn
            .query_row("SELECT count(*) FROM saved_search_seen", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(seen_rows, 5, "every seen row stays, with two markers");
        let kept: Vec<(i64, String)> = conn
            .prepare("SELECT saved_search_id, result_key FROM saved_search_seen ORDER BY 1, 2")
            .unwrap()
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        assert_eq!(
            kept,
            vec![
                (1, "artist:a".to_owned()),
                (1, saved_search_seen::SEEN_INITIALISED_MARKER.to_owned()),
                (1, "release:feed:b".to_owned()),
                (4, "artist:c".to_owned()),
                (4, saved_search_seen::SEEN_INITIALISED_MARKER.to_owned()),
            ]
        );
        let foreign_key_problems: i64 = conn
            .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(foreign_key_problems, 0);
        let query_index: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_schema WHERE name = 'saved_searches_query'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(query_index, 0, "the unique index of the query is dropped");
    }

    /// ADR 0089 Decision 4 (operator decision of 2026-11-06, migration 23):
    /// after the upgrade, `name` is required and unique without case, and a
    /// query may repeat.
    #[test]
    fn adr_0089_saved_search_name_r23_02_name_is_required_and_unique() {
        let conn = Connection::open_in_memory().unwrap();
        upgrades::create_fixture(&conn, 22).unwrap();
        insert_raw(&conn, "jazz", "releases", 1);
        crate::db::migrate_schema_to(&conn, 23).unwrap();

        assert!(conn
            .execute(
                "INSERT INTO saved_searches (query, start_tab, created_at, name) VALUES ('a', 'releases', 1, NULL)",
                [],
            )
            .is_err());
        assert!(conn
            .execute(
                "INSERT INTO saved_searches (query, start_tab, created_at, name) VALUES ('a', 'releases', 1, 'JAZZ')",
                [],
            )
            .is_err(), "the name \"jazz\" is held, without case");
        conn.execute(
            "INSERT INTO saved_searches (query, start_tab, created_at, name) VALUES ('jazz', 'tracks', 1, 'Jazz 3')",
            [],
        )
        .unwrap();
        assert_eq!(saved_searches(&conn).unwrap().len(), 2);
    }

    /// ADR 0089 Decision 4 (operator decision of 2026-11-06): a rename that
    /// another row holds is refused, and nothing changes. A blank rename
    /// takes the query text, or its first free form.
    #[test]
    fn adr_0089_saved_search_name_r23_03_rename_refuses_a_taken_name() {
        let conn = Connection::open_in_memory().unwrap();
        upgrades::create_fixture(&conn, CURRENT_VERSION).unwrap();
        let first = insert_saved_search(&conn, "bluegrass", StoredStartTab::Tracks, 100).unwrap();
        let second = insert_saved_search(&conn, "ambient", StoredStartTab::Tracks, 200).unwrap();

        assert_eq!(
            rename_saved_search(&conn, second, "BLUEGRASS").unwrap(),
            SavedSearchWrite::NameTaken,
            "the name is held without case"
        );
        let stored = saved_searches(&conn).unwrap();
        assert_eq!(
            stored[1].name, "ambient",
            "a refused rename changes nothing"
        );

        assert_eq!(
            rename_saved_search(&conn, second, "bluegrass").unwrap(),
            SavedSearchWrite::NameTaken
        );
        assert_eq!(
            rename_saved_search(&conn, first, "bluegrass").unwrap(),
            SavedSearchWrite::Written,
            "a row may keep its own name"
        );

        // A second row with the query "bluegrass" gets the name "bluegrass 2".
        // A blank rename of it keeps its own free form, because it is not
        // blocked by itself.
        let copy = insert_saved_search(&conn, "bluegrass", StoredStartTab::Tracks, 300).unwrap();
        assert_eq!(saved_searches(&conn).unwrap()[2].name, "bluegrass 2");
        assert_eq!(
            rename_saved_search(&conn, copy, "  ").unwrap(),
            SavedSearchWrite::Written
        );
        assert_eq!(saved_searches(&conn).unwrap()[2].name, "bluegrass 2");

        // A blank rename of the second row takes its query, which is free.
        assert_eq!(
            rename_saved_search(&conn, second, "  ").unwrap(),
            SavedSearchWrite::Written
        );
        assert_eq!(saved_searches(&conn).unwrap()[1].name, "ambient");
    }

    /// ADR 0089 Decision 4 (operator decision of 2026-11-06): a second row
    /// with a shared query is stored. A name that follows the query moves to
    /// the new query. A given name stays.
    #[test]
    fn adr_0089_saved_search_name_r23_04_edit_query_follows_only_a_query_name() {
        let conn = Connection::open_in_memory().unwrap();
        upgrades::create_fixture(&conn, CURRENT_VERSION).unwrap();
        let auto = insert_saved_search(&conn, "bluegrass", StoredStartTab::Tracks, 100).unwrap();
        let given = insert_saved_search(&conn, "ambient", StoredStartTab::Tracks, 200).unwrap();
        rename_saved_search(&conn, given, "Old time").unwrap();
        let zither = insert_saved_search(&conn, "zither", StoredStartTab::Tracks, 300).unwrap();

        assert_eq!(
            update_saved_search_query(&conn, auto, "bluegrass banjo").unwrap(),
            SavedSearchQueryWrite::Written
        );
        assert_eq!(
            update_saved_search_query(&conn, given, "ambient jazz").unwrap(),
            SavedSearchQueryWrite::Written
        );
        assert_eq!(
            update_saved_search_query(&conn, zither, "bluegrass banjo").unwrap(),
            SavedSearchQueryWrite::Written,
            "the query is shared, so no conflict is refused"
        );
        let stored = saved_searches(&conn).unwrap();
        assert_eq!(stored[0].query, "bluegrass banjo");
        assert_eq!(stored[0].name, "bluegrass banjo");
        assert_eq!(stored[1].query, "ambient jazz");
        assert_eq!(stored[1].name, "Old time", "the given name stays");
        assert_eq!(stored[2].query, "bluegrass banjo");
        assert_eq!(
            stored[2].name, "bluegrass banjo 2",
            "the name that follows takes the next free form"
        );
    }

    /// ADR 0089 Decision 4 (packet 007 Part C): each saved search keeps its
    /// own seen results, also when two saved searches hold one query.
    #[test]
    fn adr_0089_saved_search_name_r23_05_shared_query_seen_rows_stay_per_row() {
        let conn = Connection::open_in_memory().unwrap();
        upgrades::create_fixture(&conn, CURRENT_VERSION).unwrap();
        let first = insert_saved_search(&conn, "bluegrass", StoredStartTab::Tracks, 100).unwrap();
        let second = insert_saved_search(&conn, "bluegrass", StoredStartTab::Tracks, 200).unwrap();
        assert_eq!(saved_searches(&conn).unwrap()[1].name, "bluegrass 2");
        crate::db::saved_search_seen::mark_seen(
            &conn,
            first,
            &std::iter::once("artist:a".to_owned()).collect(),
        )
        .unwrap();
        assert_eq!(
            crate::db::saved_search_seen::read_seen(&conn, second).unwrap(),
            None,
            "the second row is not initialised by the first"
        );
    }
}
