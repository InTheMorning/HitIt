//! The stored playlist GUID and the export rows of a playlist (ADR 0115).
//!
//! Migration 34 adds `playlists.musicl_guid`. The first `musicL` export of a
//! playlist makes a random UUID and stores it. Each subsequent export reads
//! the same value. A rename does not change it (ADR 0115 Decision 3).

#![warn(clippy::pedantic)]

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};

/// The read contract of `playlists` after migration 34: the columns of
/// migration 24, then the one column that migration 34 appends.
pub(super) const COLUMNS: &(&str, &[&str]) = &(
    "playlists",
    &[
        "id",
        "name",
        "description",
        "created_at",
        "updated_at",
        "position",
        "musicl_guid",
    ],
);

const DDL: &str = r"
ALTER TABLE playlists ADD COLUMN musicl_guid TEXT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_playlists_musicl_guid ON playlists(musicl_guid);
";

/// Migration 34 (ADR 0016 registry, ADR 0115 Decision 3).
pub(super) fn apply(conn: &Connection) -> Result<()> {
    conn.execute_batch(DDL)
        .context("add the playlist musicL GUID column")
}

/// The channel values of one playlist.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExportPlaylist {
    /// The playlist name.
    pub(crate) name: String,
    /// The playlist description, when it has a value.
    pub(crate) description: Option<String>,
}

/// One track of a playlist, with the values of its feed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExportTrack {
    /// The local track id.
    pub(crate) track_id: i64,
    /// The track title, when it has a value.
    pub(crate) title: Option<String>,
    /// The `podcast:guid` of the track feed, when the feed has one.
    pub(crate) feed_guid: Option<String>,
    /// The GUID of the track.
    pub(crate) item_guid: String,
    /// The stored feed URL.
    pub(crate) feed_url: String,
    /// The stored `podcast:medium` value of the track feed.
    pub(crate) feed_medium: Option<String>,
}

/// The channel values of the playlist `playlist_id`, or `None` when no such
/// playlist exists.
pub(crate) fn export_playlist(
    conn: &Connection,
    playlist_id: i64,
) -> Result<Option<ExportPlaylist>> {
    conn.query_row(
        "SELECT name, description FROM playlists WHERE id = ?1",
        params![playlist_id],
        |row| {
            Ok(ExportPlaylist {
                name: row.get(0)?,
                description: row.get(1)?,
            })
        },
    )
    .optional()
    .context("read the playlist for the musicL export")
}

/// The tracks of the playlist `playlist_id`, in the playlist sequence.
pub(crate) fn export_tracks(conn: &Connection, playlist_id: i64) -> Result<Vec<ExportTrack>> {
    let mut statement = conn.prepare(
        "SELECT t.id, t.track_title, f.feed_guid, t.item_guid, f.feed_url, f.podcast_medium
         FROM playlist_tracks pt
         JOIN tracks t ON t.id = pt.track_id
         JOIN feeds f ON f.id = t.feed_id
         WHERE pt.playlist_id = ?1
         ORDER BY pt.position",
    )?;
    let rows = statement
        .query_map(params![playlist_id], |row| {
            Ok(ExportTrack {
                track_id: row.get(0)?,
                title: row.get(1)?,
                feed_guid: row.get(2)?,
                item_guid: row.get(3)?,
                feed_url: row.get(4)?,
                feed_medium: row.get(5)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()
        .context("read the playlist tracks for the musicL export")?;
    Ok(rows)
}

/// The stored GUID of the playlist `playlist_id`. When the playlist has no
/// GUID, the function stores `new_guid()` and gives it.
pub(crate) fn musicl_guid(
    conn: &Connection,
    playlist_id: i64,
    new_guid: impl FnOnce() -> String,
) -> Result<String> {
    let stored: Option<String> = conn
        .query_row(
            "SELECT musicl_guid FROM playlists WHERE id = ?1",
            params![playlist_id],
            |row| row.get(0),
        )
        .context("read the playlist musicL GUID")?;
    if let Some(guid) = stored {
        return Ok(guid);
    }
    let guid = new_guid();
    conn.execute(
        "UPDATE playlists SET musicl_guid = ?2 WHERE id = ?1 AND musicl_guid IS NULL",
        params![playlist_id, guid],
    )
    .context("store the playlist musicL GUID")?;
    Ok(guid)
}
