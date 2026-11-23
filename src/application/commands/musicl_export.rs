//! The `musicL` export of a playlist (ADR 0115).
//!
//! The export writes one RSS 2.0 file. The channel has the playlist name, the
//! playlist description, the stored playlist GUID and `podcast:medium`
//! `musicL`. Each song of the playlist gives one `podcast:remoteItem`, in the
//! playlist sequence (ADR 0115 Decisions 2 and 4). A track with no feed GUID,
//! or from a feed with a stored medium other than `music`, is left out and
//! reported (ADR 0115 Decision 5).

#![warn(clippy::pedantic)]

use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use quick_xml::events::{BytesDecl, BytesText, Event};
use quick_xml::Writer;
use rusqlite::Connection;

use crate::db::playlist_guid::{self, ExportPlaylist, ExportTrack};
use crate::metadata::web_url_value;

/// The Podcasting 2.0 namespace.
const PODCAST_NAMESPACE: &str = "https://podcastindex.org/namespace/1.0";

/// The one medium that the station accepts (musicindex ADR 0092 R-STATION-3
/// item 1).
const MUSIC_MEDIUM: &str = "music";

/// One `podcast:remoteItem` of the export.
#[derive(Clone, Debug, Eq, PartialEq)]
struct RemoteItem {
    feed_guid: String,
    item_guid: String,
    feed_url: Option<String>,
    medium: Option<String>,
}

/// The cause that the export left out a track.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum LeaveOutCause {
    /// The track feed has no feed GUID.
    NoFeedGuid,
    /// The track feed has this stored medium, which is not `music`.
    NotMusic(String),
}

/// One track that the export left out.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LeftOut {
    /// The local track id.
    pub(crate) track_id: i64,
    /// The track title, when it has a value.
    pub(crate) title: Option<String>,
    /// The cause.
    pub(crate) cause: LeaveOutCause,
}

/// The result of one export.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExportReport {
    /// The written file.
    pub(crate) path: PathBuf,
    /// The `podcast:guid` of the file.
    pub(crate) guid: String,
    /// The number of written `podcast:remoteItem` elements.
    pub(crate) written: usize,
    /// Each track that the export left out.
    pub(crate) left_out: Vec<LeftOut>,
}

/// Export the playlist `playlist_id` to the file `path` (ADR 0115).
///
/// `new_guid` makes the playlist GUID when the playlist has none.
///
/// # Errors
///
/// Returns an error when no such playlist exists, when no track remains, or
/// when the database read or the file write fails. When no track remains,
/// the function writes no file and stores no GUID.
pub(crate) fn export(
    conn: &Connection,
    playlist_id: i64,
    path: &Path,
    new_guid: impl FnOnce() -> String,
) -> Result<ExportReport> {
    let playlist = playlist_guid::export_playlist(conn, playlist_id)?
        .ok_or_else(|| anyhow!("no playlist with id {playlist_id}"))?;
    let tracks = playlist_guid::export_tracks(conn, playlist_id)?;
    let (items, left_out) = select_songs(tracks);
    if items.is_empty() {
        let mut text =
            format!("Playlist {playlist_id} has no song to export. HitIt wrote no file.\n");
        text.push_str(&left_out_text(&left_out));
        return Err(anyhow!(text.trim_end().to_owned()));
    }
    let guid = playlist_guid::musicl_guid(conn, playlist_id, new_guid)?;
    let xml = render(&playlist, &guid, &items).context("make the musicL text")?;
    write_file(path, &xml)?;
    Ok(ExportReport {
        path: path.to_path_buf(),
        guid,
        written: items.len(),
        left_out,
    })
}

/// The text that the command prints for `report` (ADR 0115 Decision 6).
pub(crate) fn report_text(report: &ExportReport) -> String {
    let mut text = String::new();
    let _ = writeln!(text, "Wrote {}", report.path.display());
    let _ = writeln!(text, "Playlist GUID: {}", report.guid);
    let _ = writeln!(text, "Songs written: {}", report.written);
    text.push_str(&left_out_text(&report.left_out));
    text
}

fn left_out_text(left_out: &[LeftOut]) -> String {
    let mut text = String::new();
    let _ = writeln!(text, "Tracks left out: {}", left_out.len());
    for entry in left_out {
        let title = entry.title.as_deref().unwrap_or("(no title)");
        let cause = match &entry.cause {
            LeaveOutCause::NoFeedGuid => "the feed has no feed GUID".to_owned(),
            LeaveOutCause::NotMusic(medium) => {
                format!("the feed medium is \"{medium}\", not \"{MUSIC_MEDIUM}\"")
            }
        };
        let _ = writeln!(text, "- track {} \"{title}\": {cause}", entry.track_id);
    }
    text
}

/// A trimmed value, or `None` when the value is empty.
fn non_empty(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

/// Split the tracks into the songs to write and the tracks to leave out
/// (ADR 0115 Decisions 4 and 5).
fn select_songs(tracks: Vec<ExportTrack>) -> (Vec<RemoteItem>, Vec<LeftOut>) {
    let mut items = Vec::new();
    let mut left_out = Vec::new();
    for track in tracks {
        let medium = non_empty(track.feed_medium.as_deref());
        let cause = match (&medium, non_empty(track.feed_guid.as_deref())) {
            (_, None) => Some(LeaveOutCause::NoFeedGuid),
            (Some(medium), Some(_)) if !medium.eq_ignore_ascii_case(MUSIC_MEDIUM) => {
                Some(LeaveOutCause::NotMusic(medium.clone()))
            }
            (_, Some(feed_guid)) => {
                items.push(RemoteItem {
                    feed_guid,
                    item_guid: track.item_guid.trim().to_owned(),
                    feed_url: web_url_value(&track.feed_url),
                    medium,
                });
                None
            }
        };
        if let Some(cause) = cause {
            left_out.push(LeftOut {
                track_id: track.track_id,
                title: track.title,
                cause,
            });
        }
    }
    (items, left_out)
}

/// The XML text of the export.
fn render(playlist: &ExportPlaylist, guid: &str, items: &[RemoteItem]) -> io::Result<String> {
    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
    writer
        .create_element("rss")
        .with_attributes([("version", "2.0"), ("xmlns:podcast", PODCAST_NAMESPACE)])
        .write_inner_content(|writer| {
            writer
                .create_element("channel")
                .write_inner_content(|writer| write_channel(writer, playlist, guid, items))?;
            Ok(())
        })?;
    let mut text = String::from_utf8(writer.into_inner()).map_err(io::Error::other)?;
    text.push('\n');
    Ok(text)
}

fn write_channel(
    writer: &mut Writer<Vec<u8>>,
    playlist: &ExportPlaylist,
    guid: &str,
    items: &[RemoteItem],
) -> io::Result<()> {
    writer
        .create_element("title")
        .write_text_content(BytesText::new(&playlist.name))?;
    if let Some(description) = non_empty(playlist.description.as_deref()) {
        writer
            .create_element("description")
            .write_text_content(BytesText::new(&description))?;
    }
    writer
        .create_element("podcast:guid")
        .write_text_content(BytesText::new(guid))?;
    writer
        .create_element("podcast:medium")
        .write_text_content(BytesText::new("musicL"))?;
    for item in items {
        let mut attributes = vec![
            ("feedGuid", item.feed_guid.as_str()),
            ("itemGuid", item.item_guid.as_str()),
        ];
        if let Some(feed_url) = &item.feed_url {
            attributes.push(("feedUrl", feed_url.as_str()));
        }
        if let Some(medium) = &item.medium {
            attributes.push(("medium", medium.as_str()));
        }
        writer
            .create_element("podcast:remoteItem")
            .with_attributes(attributes)
            .write_empty()?;
    }
    Ok(())
}

/// Write `text` to a temporary file next to `path`, then move it to `path`.
/// A failed write does not leave a part of the file at `path`.
fn write_file(path: &Path, text: &str) -> Result<()> {
    let name = path
        .file_name()
        .ok_or_else(|| anyhow!("the path {} has no file name", path.display()))?;
    let mut temporary_name = std::ffi::OsString::from(".");
    temporary_name.push(name);
    temporary_name.push(".partial");
    let temporary = path.with_file_name(temporary_name);
    fs::write(&temporary, text).with_context(|| format!("write {}", temporary.display()))?;
    fs::rename(&temporary, path).with_context(|| format!("move the file to {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use rusqlite::params;

    fn setup() -> Result<Connection> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        db::init_schema(&conn)?;
        db::migrate_schema(&conn)?;
        Ok(conn)
    }

    fn feed(conn: &Connection, url: &str, guid: Option<&str>, medium: Option<&str>) -> Result<i64> {
        conn.execute(
            "INSERT INTO feeds (feed_url, feed_guid, title, podcast_medium)
             VALUES (?1, ?2, 'Feed', ?3)",
            params![url, guid, medium],
        )?;
        Ok(conn.last_insert_rowid())
    }

    fn track(conn: &Connection, feed_id: i64, item_guid: &str) -> Result<i64> {
        conn.execute(
            "INSERT INTO tracks (feed_id, item_guid, track_title, is_in_library)
             VALUES (?1, ?2, ?3, 1)",
            params![feed_id, item_guid, format!("Song {item_guid}")],
        )?;
        Ok(conn.last_insert_rowid())
    }

    fn playlist(conn: &Connection, name: &str, tracks: &[i64]) -> Result<i64> {
        let id = db::playlist_create(conn, name)?;
        for track_id in tracks {
            db::playlist_append(conn, id, *track_id)?;
        }
        Ok(id)
    }

    fn export_to(
        conn: &Connection,
        playlist_id: i64,
        guid: &str,
    ) -> Result<(ExportReport, String, tempfile::TempDir)> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("list.xml");
        let report = export(conn, playlist_id, &path, || guid.to_owned())?;
        let text = fs::read_to_string(&path)?;
        Ok((report, text, dir))
    }

    /// The remote items of `xml`, as attribute maps, in the file sequence.
    fn remote_items(xml: &str) -> Vec<std::collections::BTreeMap<String, String>> {
        let mut reader = quick_xml::Reader::from_str(xml);
        let mut items = Vec::new();
        loop {
            match reader.read_event().expect("read the export") {
                Event::Empty(element) if element.name().as_ref() == b"podcast:remoteItem" => {
                    items.push(
                        element
                            .attributes()
                            .map(|attribute| {
                                let attribute = attribute.expect("attribute");
                                (
                                    String::from_utf8(attribute.key.as_ref().to_vec())
                                        .expect("key"),
                                    attribute
                                        .decode_and_unescape_value(reader.decoder())
                                        .expect("value")
                                        .into_owned(),
                                )
                            })
                            .collect(),
                    );
                }
                Event::Eof => return items,
                _ => {}
            }
        }
    }

    /// ADR 0115 invariant 1 and Decision 4: each remote item has both GUIDs,
    /// in the playlist sequence.
    #[test]
    fn each_remote_item_has_both_guids() -> Result<()> {
        let conn = setup()?;
        let feed_id = feed(
            &conn,
            "https://a.test/feed.xml",
            Some("feed-a"),
            Some("music"),
        )?;
        let first = track(&conn, feed_id, "item-1")?;
        let second = track(&conn, feed_id, "item-2")?;
        let id = playlist(&conn, "Station", &[second, first])?;

        let (report, xml, _dir) = export_to(&conn, id, "guid-1")?;

        assert_eq!(report.written, 2);
        let items = remote_items(&xml);
        let pairs: Vec<_> = items
            .iter()
            .map(|item| (item["feedGuid"].as_str(), item["itemGuid"].as_str()))
            .collect();
        assert_eq!(pairs, [("feed-a", "item-2"), ("feed-a", "item-1")]);
        assert!(xml.contains("<podcast:medium>musicL</podcast:medium>"));
        assert!(xml.contains("<title>Station</title>"));
        Ok(())
    }

    /// ADR 0115 invariant 2 and Decision 3: two exports give the same GUID,
    /// also after a rename.
    #[test]
    fn two_exports_and_a_rename_keep_the_guid() -> Result<()> {
        let conn = setup()?;
        let feed_id = feed(
            &conn,
            "https://a.test/feed.xml",
            Some("feed-a"),
            Some("music"),
        )?;
        let song = track(&conn, feed_id, "item-1")?;
        let id = playlist(&conn, "Station", &[song])?;

        let (first, first_xml, _a) = export_to(&conn, id, "guid-first")?;
        db::playlist_rename(&conn, id, "Renamed")?;
        let (second, second_xml, _b) = export_to(&conn, id, "guid-second")?;

        assert_eq!(first.guid, "guid-first");
        assert_eq!(second.guid, "guid-first");
        assert!(first_xml.contains("<podcast:guid>guid-first</podcast:guid>"));
        assert!(second_xml.contains("<podcast:guid>guid-first</podcast:guid>"));
        assert!(second_xml.contains("<title>Renamed</title>"));
        Ok(())
    }

    /// ADR 0115 Decision 5: a track with no feed GUID and a track that is not
    /// a song are left out and reported.
    #[test]
    fn left_out_tracks_are_reported_with_the_cause() -> Result<()> {
        let conn = setup()?;
        let music = feed(
            &conn,
            "https://a.test/feed.xml",
            Some("feed-a"),
            Some("music"),
        )?;
        let no_guid = feed(&conn, "https://b.test/feed.xml", None, Some("music"))?;
        let podcast = feed(
            &conn,
            "https://c.test/feed.xml",
            Some("feed-c"),
            Some("podcast"),
        )?;
        let song = track(&conn, music, "item-1")?;
        let orphan = track(&conn, no_guid, "item-2")?;
        let episode = track(&conn, podcast, "item-3")?;
        let id = playlist(&conn, "Station", &[orphan, song, episode])?;

        let (report, xml, _dir) = export_to(&conn, id, "guid-1")?;

        assert_eq!(report.written, 1);
        assert_eq!(
            report.left_out,
            [
                LeftOut {
                    track_id: orphan,
                    title: Some("Song item-2".into()),
                    cause: LeaveOutCause::NoFeedGuid,
                },
                LeftOut {
                    track_id: episode,
                    title: Some("Song item-3".into()),
                    cause: LeaveOutCause::NotMusic("podcast".into()),
                },
            ]
        );
        assert_eq!(remote_items(&xml).len(), 1);
        let text = report_text(&report);
        assert!(text.contains(&format!(
            "- track {orphan} \"Song item-2\": the feed has no feed GUID"
        )));
        assert!(text.contains(&format!(
            "- track {episode} \"Song item-3\": the feed medium is \"podcast\", not \"music\""
        )));
        assert!(text.contains("Playlist GUID: guid-1"));
        assert!(text.contains("Songs written: 1"));
        Ok(())
    }

    /// ADR 0115 Decision 4: `feedUrl` only for an http or https URL.
    #[test]
    fn feed_url_only_for_a_web_url() -> Result<()> {
        let conn = setup()?;
        let web = feed(
            &conn,
            "https://a.test/feed.xml",
            Some("feed-a"),
            Some("music"),
        )?;
        let local = feed(
            &conn,
            "file:///music/feed.xml",
            Some("feed-b"),
            Some("music"),
        )?;
        let first = track(&conn, web, "item-1")?;
        let second = track(&conn, local, "item-2")?;
        let id = playlist(&conn, "Station", &[first, second])?;

        let (_report, xml, _dir) = export_to(&conn, id, "guid-1")?;

        let items = remote_items(&xml);
        assert_eq!(
            items[0].get("feedUrl").map(String::as_str),
            Some("https://a.test/feed.xml")
        );
        assert_eq!(items[1].get("feedUrl"), None);
        Ok(())
    }

    /// ADR 0115 Decision 4: no `medium` when the feed has no stored medium.
    #[test]
    fn no_medium_when_the_feed_has_none() -> Result<()> {
        let conn = setup()?;
        let known = feed(
            &conn,
            "https://a.test/feed.xml",
            Some("feed-a"),
            Some("music"),
        )?;
        let unknown = feed(&conn, "https://b.test/feed.xml", Some("feed-b"), None)?;
        let first = track(&conn, known, "item-1")?;
        let second = track(&conn, unknown, "item-2")?;
        let id = playlist(&conn, "Station", &[first, second])?;

        let (report, xml, _dir) = export_to(&conn, id, "guid-1")?;

        assert_eq!(report.written, 2);
        let items = remote_items(&xml);
        assert_eq!(items[0].get("medium").map(String::as_str), Some("music"));
        assert_eq!(items[1].get("medium"), None);
        Ok(())
    }

    /// ADR 0115 Decision 6: no file and no stored GUID when no track remains.
    #[test]
    fn no_file_when_no_track_remains() -> Result<()> {
        let conn = setup()?;
        let no_guid = feed(&conn, "https://b.test/feed.xml", None, Some("music"))?;
        let orphan = track(&conn, no_guid, "item-1")?;
        let id = playlist(&conn, "Station", &[orphan])?;
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("list.xml");

        let error = export(&conn, id, &path, || "guid-1".to_owned()).expect_err("no song remains");

        assert!(!path.exists());
        assert_eq!(fs::read_dir(dir.path())?.count(), 0);
        let message = error.to_string();
        assert!(message.contains("has no song to export"));
        assert!(message.contains("the feed has no feed GUID"));
        let stored: Option<String> = conn.query_row(
            "SELECT musicl_guid FROM playlists WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        assert_eq!(stored, None);
        Ok(())
    }

    /// The station input: the stophammer parser reads the export with no
    /// error and returns each remote item.
    #[test]
    fn stophammer_parser_reads_the_export() -> Result<()> {
        let conn = setup()?;
        let web = feed(
            &conn,
            "https://a.test/feed.xml",
            Some("feed-a"),
            Some("music"),
        )?;
        let unknown = feed(&conn, "https://b.test/feed.xml", Some("feed-b"), None)?;
        let first = track(&conn, web, "item-1")?;
        let second = track(&conn, unknown, "item-2")?;
        let id = playlist(&conn, "Station & Friends", &[first, second])?;

        let (_report, xml, _dir) = export_to(&conn, id, "guid-1")?;

        let parsed = stophammer_parser::profile::stophammer()
            .parse(&xml)
            .map_err(|error| anyhow!("{error:?}"))?;
        assert_eq!(parsed.feed_guid, "guid-1");
        assert_eq!(parsed.title, "Station & Friends");
        assert_eq!(parsed.raw_medium.as_deref(), Some("musicL"));
        let refs: Vec<_> = parsed
            .remote_items
            .iter()
            .map(|item| {
                (
                    item.remote_feed_guid.as_str(),
                    item.item_guid.as_deref(),
                    item.remote_feed_url.as_deref(),
                )
            })
            .collect();
        assert_eq!(
            refs,
            [
                ("feed-a", Some("item-1"), Some("https://a.test/feed.xml")),
                ("feed-b", Some("item-2"), Some("https://b.test/feed.xml")),
            ]
        );
        Ok(())
    }
}
