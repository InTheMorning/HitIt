//! Playlist local query family.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::application::application_query_service::ApplicationQueryService;
use crate::application::command_bus::{ApplicationCommand, CommandOutcome, CommandResult};
use crate::application::command_context::CommandContext;
use crate::application::errors::command::CommandError;
use crate::{db, playlist_service};

/// Reads the playlist list and the cover of each playlist in the background
/// (ADR 0084 Decision 12; task 039, step 4). The sidebar keeps its last list
/// until the result arrives.
#[derive(Clone, Debug)]
pub(crate) struct LoadPlaylistSidebar {
    conn: Arc<Mutex<Connection>>,
}

impl LoadPlaylistSidebar {
    /// Creates the playlist sidebar load for the Library connection.
    #[must_use]
    pub(crate) const fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

/// The output of [`LoadPlaylistSidebar`].
#[derive(Clone, Debug)]
pub(crate) struct PlaylistSidebarLoad {
    pub(crate) playlists: Vec<db::Playlist>,
    pub(crate) covers: BTreeMap<i64, Vec<String>>,
}

impl ApplicationCommand for LoadPlaylistSidebar {
    type Output = PlaylistSidebarLoad;

    fn execute(self, context: &CommandContext) -> CommandResult<Self::Output> {
        if context.cancellation().is_cancelled() {
            return Err(CommandError::Cancelled);
        }
        let conn = self
            .conn
            .lock()
            .map_err(|_| CommandError::Other("database lock poisoned".to_string()))?;
        let query = ApplicationQueryService::new();
        let playlists = query.playlists(&conn)?;
        // A failed cover read shows each row in the entity color (ADR 0084
        // Decision 12), so it gives no error.
        let covers = query.playlist_cover_images(&conn).unwrap_or_default();
        Ok(CommandOutcome::without_events(PlaylistSidebarLoad {
            playlists,
            covers,
        }))
    }
}

impl ApplicationQueryService {
    /// Lists local playlists.
    ///
    /// # Errors
    ///
    /// Returns an error when local playlist storage cannot be read.
    pub fn playlists(&self, conn: &Connection) -> Result<Vec<db::Playlist>, CommandError> {
        playlist_service::list(conn).map_err(|error| query_error(&error))
    }

    /// Reads the first four images of each local playlist (ADR 0084
    /// Decision 12).
    ///
    /// # Errors
    ///
    /// Returns an error when local playlist storage cannot be read.
    pub fn playlist_cover_images(
        &self,
        conn: &Connection,
    ) -> Result<std::collections::BTreeMap<i64, Vec<String>>, CommandError> {
        playlist_service::cover_images(conn).map_err(|error| query_error(&error))
    }

    /// Lists tracks in a local playlist.
    ///
    /// # Errors
    ///
    /// Returns an error when local playlist storage cannot be read.
    pub fn playlist_tracks(
        &self,
        conn: &Connection,
        playlist_id: i64,
    ) -> Result<Vec<db::TrackRow>, CommandError> {
        playlist_service::tracks(conn, playlist_id).map_err(|error| query_error(&error))
    }
}

fn query_error(error: &anyhow::Error) -> CommandError {
    CommandError::Query(format!("{error:#}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn setup_test_db() -> anyhow::Result<Connection> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        db::init_schema(&conn)?;
        db::migrate_schema(&conn)?;
        Ok(conn)
    }

    #[test]
    fn playlist_queries_return_local_snapshots() -> anyhow::Result<()> {
        let conn = setup_test_db()?;
        let service = ApplicationQueryService::new();
        let playlist_id = playlist_service::create(&conn, "Focus")?;
        let feed_id = create_feed(&conn)?;
        let track_id = create_track(&conn, feed_id, "first")?;
        playlist_service::append_track(&conn, playlist_id, track_id)?;

        let playlists = service.playlists(&conn)?;
        let tracks = service.playlist_tracks(&conn, playlist_id)?;

        assert_eq!(playlists.len(), 1);
        assert_eq!(playlists[0].id, playlist_id);
        assert_eq!(playlists[0].name, "Focus");
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].id, track_id);

        Ok(())
    }

    /// R11-05 (ADR 0084 Decision 12): one read gives the first four images
    /// of each playlist, in playlist order. A track image comes before the
    /// album image of its feed. An empty image and a repeated image are
    /// dropped before the count.
    #[test]
    fn adr_0084_logo_frame_r11_05_playlist_cover_images_read() -> anyhow::Result<()> {
        let conn = setup_test_db()?;
        let service = ApplicationQueryService::new();
        let feed_id = create_feed(&conn)?;
        conn.execute(
            "UPDATE feeds SET album_image_href = 'album.jpg' WHERE id = ?1",
            [feed_id],
        )?;
        let images = [
            "", "a.jpg", "a.jpg", "b.jpg", " ", "c.jpg", "d.jpg", "e.jpg",
        ];
        let mix = playlist_service::create(&conn, "Mix")?;
        for (index, image) in images.iter().enumerate() {
            let track_id = create_track(&conn, feed_id, &format!("mix-{index}"))?;
            conn.execute(
                "UPDATE tracks SET track_image_href = ?1 WHERE id = ?2",
                rusqlite::params![image, track_id],
            )?;
            playlist_service::append_track(&conn, mix, track_id)?;
        }
        let album_only = playlist_service::create(&conn, "Album")?;
        let track_id = create_track(&conn, feed_id, "album-only")?;
        playlist_service::append_track(&conn, album_only, track_id)?;
        let empty = playlist_service::create(&conn, "Empty")?;

        let covers = service.playlist_cover_images(&conn)?;

        assert_eq!(
            covers.get(&mix).cloned(),
            Some(
                ["album.jpg", "a.jpg", "b.jpg", "c.jpg"]
                    .map(str::to_owned)
                    .to_vec()
            )
        );
        assert_eq!(
            covers.get(&album_only).cloned(),
            Some(vec!["album.jpg".to_owned()])
        );
        assert_eq!(covers.get(&empty), None);
        Ok(())
    }

    fn create_feed(conn: &Connection) -> anyhow::Result<i64> {
        conn.execute(
            "INSERT INTO feeds (feed_url, feed_guid, title)
             VALUES (?1, ?2, ?3)",
            rusqlite::params!["https://example.test/feed.xml", "feed-guid", "Feed Title"],
        )?;
        Ok(conn.last_insert_rowid())
    }

    fn create_track(conn: &Connection, feed_id: i64, item_guid: &str) -> anyhow::Result<i64> {
        conn.execute(
            "INSERT INTO tracks (feed_id, item_guid, track_title, is_in_library)
             VALUES (?1, ?2, ?3, 1)",
            rusqlite::params![feed_id, item_guid, format!("Track {item_guid}")],
        )?;
        Ok(conn.last_insert_rowid())
    }
}
