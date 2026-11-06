//! Saved search names and seen results (ADR 0089 Decision 4, packet 007).
//!
//! Migration 22 adds the nullable `name` column to `saved_searches`, and
//! the table `saved_search_seen`. Each row of the table is one result key
//! that the operator saw in one saved search. The count of new results of
//! a saved search is its results minus these keys (ADR 0089 Decision 4).
//!
//! `src/db/saved_searches.rs` owns the writes of both tables. This module
//! owns only migration 22: its DDL and its read contract.

#![warn(clippy::pedantic)]

use anyhow::{Context, Result};
use rusqlite::Connection;

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
