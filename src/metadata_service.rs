use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use rusqlite::Connection;

use crate::api::{PaymentRoute, Track};
use crate::audio_tags::{AudioTags, Id3v24Edit};
use crate::db;
use crate::metadata::{
    auto_populated_pending_id3_edits, expand_woar_metadata_rows, feed_funding, feed_nostr,
    pending_id3_edits_for_apply, resolved_license, summarize_value_routes, track_funding,
    track_metadata_rows, track_nostr, MetadataColumn, PendingId3Edit, TrackContext,
    LICENSE_NAME_FRAME, LICENSE_URL_FRAME, MUSICINDEX_VALUE_ROUTES_FRAME,
};
use crate::musicbrainz::LookupMetadata;
use crate::rss::{validate_nostr_identity, IdentityValidation, NostrIdentity};

/// The frame that holds the one resolved Nostr key (ADR 0080 Decision 1).
const RSS_NOSTR_HANDLE_FRAME: &str = "TXXX:RSS Nostr Handle";

/// The frame that holds the one resolved funding link (ADR 0080 Decision 10).
const FUNDING_FRAME: &str = "WPAY";

pub fn id3_edits_for_track_context(track_context: &TrackContext) -> Vec<Id3v24Edit> {
    let rows = expand_woar_metadata_rows(track_metadata_rows(track_context, None, false));
    let mut pending =
        auto_populated_pending_id3_edits(&rows, &BTreeMap::new(), &BTreeSet::new(), None);
    apply_resolved_nostr_key(&mut pending, track_context);
    apply_resolved_funding_link(&mut pending, track_context);
    apply_resolved_license(&mut pending, track_context);
    pending_id3_edits_for_apply(&pending)
}

/// ADR 0080 Decisions 1 and 7: a file holds one Nostr key. The item's own
/// key wins when it passes NIP-19 validation as an `npub`. The channel's key
/// wins when the item has none, or when its key fails validation. Neither
/// key valid leaves the file with no Nostr frame.
///
/// The compare grid keeps its own "Nostr handle" and "RSS feed nostr handle"
/// rows apart (ADR 0075 Decision B). This function only resolves the value
/// that a write puts in the file.
fn apply_resolved_nostr_key(
    pending: &mut BTreeMap<String, PendingId3Edit>,
    track_context: &TrackContext,
) {
    pending.retain(|_, edit| edit.frame != RSS_NOSTR_HANDLE_FRAME);
    let Some(value) = resolved_nostr_key(track_context) else {
        return;
    };
    pending.insert(
        "adr-0080:resolved-nostr-handle".to_owned(),
        PendingId3Edit {
            field: "Nostr handle".to_owned(),
            frame: RSS_NOSTR_HANDLE_FRAME.to_owned(),
            value,
            source: MetadataColumn::Rss,
        },
    );
}

/// ADR 0080 Decision 10: a file holds one funding link in `WPAY`. The first
/// valid `podcast:funding` of the item wins. The first valid one of the
/// channel is the fallback. A value with no `http` or `https` scheme is not
/// valid. With no valid link, the file gets no `WPAY` edit.
///
/// The compare grid keeps its own "Funding" and "RSS feed funding" rows
/// apart. This function resolves the value that a write puts in the file,
/// as `apply_resolved_nostr_key` does for the Nostr key.
fn apply_resolved_funding_link(
    pending: &mut BTreeMap<String, PendingId3Edit>,
    track_context: &TrackContext,
) {
    pending.retain(|_, edit| edit.frame != FUNDING_FRAME);
    let Some(value) = track_funding(&track_context.track)
        .or_else(|| track_context.feed.as_ref().and_then(feed_funding))
    else {
        return;
    };
    pending.insert(
        "adr-0080:resolved-funding-link".to_owned(),
        PendingId3Edit {
            field: "Funding".to_owned(),
            frame: FUNDING_FRAME.to_owned(),
            value,
            source: MetadataColumn::Rss,
        },
    );
}

/// ADR 0080 Decision 11: the RSS license is the `podcast:license` of the
/// item, otherwise the one of the channel. Its `http` or `https` URL goes
/// to `WCOP`, and its text goes to `TXXX:MusicIndex License`. A license with
/// no valid URL gives no `WCOP` edit. A license with no text gives no text
/// edit. The writer decides if the URL stays in `WCOP` or joins
/// `TXXX:LICENSE` (`write_id3v24_edits`).
fn apply_resolved_license(
    pending: &mut BTreeMap<String, PendingId3Edit>,
    track_context: &TrackContext,
) {
    pending.retain(|_, edit| edit.frame != LICENSE_URL_FRAME && edit.frame != LICENSE_NAME_FRAME);
    let (url, name) = resolved_license(&track_context.track, track_context.feed.as_ref());
    for (key, field, frame, value) in [
        (
            "adr-0080:resolved-license-url",
            "License URL",
            LICENSE_URL_FRAME,
            url,
        ),
        (
            "adr-0080:resolved-license-name",
            "License name",
            LICENSE_NAME_FRAME,
            name,
        ),
    ] {
        let Some(value) = value else { continue };
        pending.insert(
            key.to_owned(),
            PendingId3Edit {
                field: field.to_owned(),
                frame: frame.to_owned(),
                value,
                source: MetadataColumn::Rss,
            },
        );
    }
}

/// The item's own Nostr key, when it is a valid `npub`. Otherwise the
/// channel's key, under the same rule. `None` when neither key validates.
fn resolved_nostr_key(track_context: &TrackContext) -> Option<String> {
    let item_key = track_nostr(&track_context.track);
    valid_npub(item_key.as_deref()).or_else(|| {
        let channel_key = track_context.feed.as_ref().and_then(feed_nostr);
        valid_npub(channel_key.as_deref())
    })
}

/// `Some` when `candidate` decodes as a valid NIP-19 `npub`. A `nprofile`
/// value and an invalid value are not accepted (ADR 0080 Decision 7).
fn valid_npub(candidate: Option<&str>) -> Option<String> {
    let candidate = candidate?;
    match validate_nostr_identity(candidate) {
        IdentityValidation::Valid(NostrIdentity::PublicKey { .. }) => Some(candidate.to_owned()),
        _ => None,
    }
}

/// How a tag write treats the payment route frame (ADR 0076 Decision 9).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RouteFrameWrite {
    /// The write always sets the frame to the stored route. A download, the
    /// route repair and a feed update write the frame this way.
    Always,
    /// The write changes the frame only when the edit list already has a
    /// route frame. An operator who applies selected tag edits writes the
    /// frame this way, so an edit that the operator did not select is not
    /// added.
    WhenSelected,
}

/// ADR 0076 Decision 9: each write of the payment route frame uses the route
/// stored in the database. The function removes each route frame edit from
/// `edits` and then adds one edit with the stored route.
///
/// When the database has no route for the track, the `MusicIndex` route of
/// the caller goes into the database first. The edit then uses the stored
/// value. An empty or missing stored route gives no route frame edit.
///
/// # Errors
///
/// Returns an error when the stored route cannot be read or written.
pub fn with_stored_route_frame(
    conn: &Connection,
    track_id: i64,
    musicindex_routes: Option<&[PaymentRoute]>,
    mut edits: Vec<Id3v24Edit>,
    write: RouteFrameWrite,
) -> Result<Vec<Id3v24Edit>> {
    let selected = edits
        .iter()
        .any(|edit| edit.frame_label == MUSICINDEX_VALUE_ROUTES_FRAME);
    if write == RouteFrameWrite::WhenSelected && !selected {
        return Ok(edits);
    }
    edits.retain(|edit| edit.frame_label != MUSICINDEX_VALUE_ROUTES_FRAME);
    let stored = db::payment_routes::route_for_write(conn, track_id, musicindex_routes)?;
    if let Some(value) = stored.as_deref().and_then(summarize_value_routes) {
        edits.push(Id3v24Edit {
            frame_label: MUSICINDEX_VALUE_ROUTES_FRAME.to_owned(),
            value,
        });
    }
    Ok(edits)
}

pub fn musicbrainz_lookup_metadata(track: &Track, tags: &AudioTags) -> LookupMetadata {
    LookupMetadata {
        title: tags.title.clone().or_else(|| track.title.clone()),
        artist: tags.artist.clone().or_else(|| track.track_artist.clone()),
        album: tags.album.clone().or_else(|| track.feed_title.clone()),
        track_number: tags
            .track_number
            .clone()
            .or_else(|| track.track_number.map(|number| number.to_string())),
        total_tracks: None,
        duration_secs: track.duration_secs.map(i64::from),
        isrc: tags
            .custom
            .get("ISRC")
            .cloned()
            .or_else(|| tags.custom.get("isrc").cloned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{Feed, SourceEntityId, SourceEntityLink};

    /// A real, validly encoded `npub` for the track (ADR 0080 Decision 7).
    const NPUB_TRACK: &str = "npub180cvv07tjdrrgpa0j7j7tmnyl2yr6yr7l8j4s3evf6u64th6gkwsyjh6w6";
    /// A real, validly encoded `npub` for the feed, distinct from the
    /// track's, so a test can tell which one a resolved edit carried.
    const NPUB_FEED: &str = "npub12um9zqae9uaydfszralpn0e0r90d559gd4qsrzar0j2yvut7t2zqwff5ck";

    /// A track with no website, no Nostr key and no description of its own.
    /// Its feed has all three.
    fn track_context_without_own_identity() -> TrackContext {
        TrackContext {
            track: Track {
                track_guid: Some("track-guid".into()),
                feed_guid: Some("feed-guid".into()),
                title: Some("Song".into()),
                ..Default::default()
            },
            feed: Some(Feed {
                feed_guid: Some("feed-guid".into()),
                title: Some("Feed Title".into()),
                description: Some("Feed description".into()),
                source_links: Some(vec![SourceEntityLink {
                    link_type: Some("website".into()),
                    url: Some("https://example.test/feed".into()),
                    ..Default::default()
                }]),
                source_ids: Some(vec![SourceEntityId {
                    scheme: Some("nostr_npub".into()),
                    value: Some(NPUB_FEED.into()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            rss_observation: None,
            observation_receipts: Vec::new(),
            provider_state: Default::default(),
            confirmed_labels: Vec::new(),
        }
    }

    /// A track with its own website, Nostr key and description, apart from
    /// its feed's own distinct values.
    fn track_context_with_own_identity() -> TrackContext {
        TrackContext {
            track: Track {
                track_guid: Some("track-guid".into()),
                feed_guid: Some("feed-guid".into()),
                title: Some("Song".into()),
                description: Some("Track description".into()),
                source_links: Some(vec![SourceEntityLink {
                    link_type: Some("website".into()),
                    url: Some("https://example.test/track".into()),
                    ..Default::default()
                }]),
                source_ids: Some(vec![SourceEntityId {
                    scheme: Some("nostr_npub".into()),
                    value: Some(NPUB_TRACK.into()),
                    ..Default::default()
                }]),
                ..Default::default()
            },
            feed: Some(Feed {
                feed_guid: Some("feed-guid".into()),
                title: Some("Feed Title".into()),
                description: Some("Feed description".into()),
                source_links: Some(vec![SourceEntityLink {
                    link_type: Some("website".into()),
                    url: Some("https://example.test/feed".into()),
                    ..Default::default()
                }]),
                source_ids: Some(vec![SourceEntityId {
                    scheme: Some("nostr_npub".into()),
                    value: Some(NPUB_FEED.into()),
                    ..Default::default()
                }]),
                ..Default::default()
            }),
            rss_observation: None,
            observation_receipts: Vec::new(),
            provider_state: Default::default(),
            confirmed_labels: Vec::new(),
        }
    }

    fn edit(frame_label: &str, value: &str) -> Id3v24Edit {
        Id3v24Edit {
            frame_label: frame_label.to_owned(),
            value: value.to_owned(),
        }
    }

    /// R22-06 / R80-11 (ADR 0075 packet 022, updated by ADR 0080 packet
    /// 001): a track with no own website, Nostr key or description tags the
    /// channel's values in their own frames. The channel website goes to
    /// `WOAR` (Decision 2), the channel description goes to
    /// `COMM:MusicIndex Album Description` (Decision 5) because the item
    /// has none, and the channel's Nostr key resolves into
    /// `TXXX:RSS Nostr Handle` (Decision 1) because the item has none. This
    /// test holds the expected edits as fixed values. It also proves ADR
    /// 0011: the feed and track GUIDs map to their own `TXXX` frames.
    #[test]
    fn adr_0075_track_header_r22_06_tag_edits_stay_equal_without_own_identity() {
        let context = track_context_without_own_identity();
        let edits = id3_edits_for_track_context(&context);

        assert_eq!(
            edits,
            vec![
                edit("COMM:MusicIndex Album Description", "Feed description"),
                edit("TALB", "Feed Title"),
                edit("TIT2", "Song"),
                edit("TXXX:MusicIndex Feed Guid", "feed-guid"),
                edit("TXXX:MusicIndex Track Guid", "track-guid"),
                edit("TXXX:RSS Nostr Handle", NPUB_FEED),
                edit("WOAR", "https://example.test/feed"),
            ],
            "ADR 0011 feed/track GUID frames and ADR 0075/0080 track header tags must match exactly"
        );
    }

    /// R22-06 / R80-11: a track with its own website, Nostr key and
    /// description tags its own values, apart from the feed's, and each
    /// frame now follows ADR 0080: the item page goes to `WOAF` and the
    /// channel website to `WOAR` (Decision 2), the item's own Nostr key
    /// wins over the channel's (Decision 1), and the item's own description
    /// stays in `COMM:MusicIndex Description` with no album row (Decision
    /// 5). It also proves ADR 0011: the feed and track GUIDs map to their
    /// own `TXXX` frames.
    #[test]
    fn adr_0075_track_header_r22_06_tag_edits_stay_equal_with_own_identity() {
        let context = track_context_with_own_identity();
        let edits = id3_edits_for_track_context(&context);

        assert_eq!(
            edits,
            vec![
                edit("COMM:MusicIndex Description", "Track description"),
                edit("TALB", "Feed Title"),
                edit("TIT2", "Song"),
                edit("TXXX:MusicIndex Feed Guid", "feed-guid"),
                edit("TXXX:MusicIndex Track Guid", "track-guid"),
                edit("TXXX:RSS Nostr Handle", NPUB_TRACK),
                edit("WOAF", "https://example.test/track"),
                edit("WOAR", "https://example.test/feed"),
            ],
            "ADR 0011 feed/track GUID frames and ADR 0075/0080 track header tags must match exactly"
        );
    }

    /// R80-01: a track with its own valid Nostr key holds that key. Without
    /// one, the edits hold the valid channel key.
    #[test]
    fn adr_0080_nostr_edit_prefers_the_item_key_then_the_channel_key() {
        let with_own_key = id3_edits_for_track_context(&track_context_with_own_identity());
        assert!(with_own_key.contains(&edit("TXXX:RSS Nostr Handle", NPUB_TRACK)));

        let without_own_key = id3_edits_for_track_context(&track_context_without_own_identity());
        assert!(without_own_key.contains(&edit("TXXX:RSS Nostr Handle", NPUB_FEED)));
    }

    /// R80-02: an invalid item key gives the valid channel key. Two invalid
    /// keys give no Nostr edit.
    #[test]
    fn adr_0080_nostr_edit_falls_back_past_an_invalid_key() {
        let mut context = track_context_with_own_identity();
        context.track.source_ids = Some(vec![SourceEntityId {
            scheme: Some("nostr_npub".into()),
            value: Some("npub1notavalidkey".into()),
            ..Default::default()
        }]);
        let edits = id3_edits_for_track_context(&context);
        assert!(
            edits.contains(&edit("TXXX:RSS Nostr Handle", NPUB_FEED)),
            "an invalid item key must give the valid channel key"
        );

        context.feed.as_mut().expect("feed").source_ids = Some(vec![SourceEntityId {
            scheme: Some("nostr_npub".into()),
            value: Some("npub1bbbbbb".into()),
            ..Default::default()
        }]);
        let edits = id3_edits_for_track_context(&context);
        assert!(
            !edits
                .iter()
                .any(|edit| edit.frame_label == "TXXX:RSS Nostr Handle"),
            "two invalid keys must give no Nostr edit"
        );
    }

    /// R80-03: the item page goes to `WOAF` and the channel website goes to
    /// `WOAR`. Neither value lands in the other frame.
    #[test]
    fn adr_0080_website_edits_stay_apart_by_owner() {
        let edits = id3_edits_for_track_context(&track_context_with_own_identity());
        assert!(edits.contains(&edit("WOAF", "https://example.test/track")));
        assert!(edits.contains(&edit("WOAR", "https://example.test/feed")));
        assert!(!edits.contains(&edit("WOAR", "https://example.test/track")));
        assert!(!edits.contains(&edit("WOAF", "https://example.test/feed")));
    }

    /// R80-04: a URL edit holds a plain URL, with no label text. A value
    /// that does not parse as a URL writes no edit at all.
    #[test]
    fn adr_0080_website_edit_is_a_plain_url_or_absent() {
        let edits = id3_edits_for_track_context(&track_context_with_own_identity());
        let woar = edits
            .iter()
            .find(|edit| edit.frame_label == "WOAR")
            .expect("WOAR edit");
        assert_eq!(woar.value, "https://example.test/feed");

        let mut context = track_context_with_own_identity();
        context.track.source_links = Some(vec![SourceEntityLink {
            link_type: Some("website".into()),
            url: Some("not a url".into()),
            ..Default::default()
        }]);
        let edits = id3_edits_for_track_context(&context);
        assert!(
            !edits.iter().any(|edit| edit.frame_label == "WOAF"),
            "a value that does not parse as a URL must write no WOAF edit"
        );
    }

    /// R80-05: a track without its own description gives
    /// `COMM:MusicIndex Album Description` with the channel description, and
    /// no `COMM:MusicIndex Description`.
    #[test]
    fn adr_0080_description_falls_back_to_the_album_row_only() {
        let edits = id3_edits_for_track_context(&track_context_without_own_identity());
        assert!(edits.contains(&edit(
            "COMM:MusicIndex Album Description",
            "Feed description"
        )));
        assert!(!edits
            .iter()
            .any(|edit| edit.frame_label == "COMM:MusicIndex Description"));
    }

    /// R80-51: the tag edits for a track with its own artwork URL hold that
    /// URL in `TXXX:MusicIndex Image`.
    #[test]
    fn adr_0080_image_edit_holds_the_track_artwork_url() {
        let mut context = track_context_without_own_identity();
        context.track.image_url = Some("https://example.test/track.png".into());
        let edits = id3_edits_for_track_context(&context);

        assert!(edits.contains(&edit(
            "TXXX:MusicIndex Image",
            "https://example.test/track.png"
        )));
    }

    /// R80-52: a track with no artwork URL of its own gives the feed
    /// image URL in `TXXX:MusicIndex Image`.
    #[test]
    fn adr_0080_image_edit_falls_back_to_the_feed_artwork_url() {
        let mut context = track_context_without_own_identity();
        context.feed.as_mut().expect("feed").image_url =
            Some("https://example.test/feed.png".into());
        let edits = id3_edits_for_track_context(&context);

        assert!(edits.contains(&edit(
            "TXXX:MusicIndex Image",
            "https://example.test/feed.png"
        )));
    }

    /// R80-53: a `data:` URL, a relative URL, an `ftp` URL, and an `https`
    /// URL of 2,049 characters each give no `TXXX:MusicIndex Image` edit.
    #[test]
    fn adr_0080_image_edit_rejects_an_unsupported_or_oversized_url() {
        let prefix_len = "https://example.test/".len();
        let long_url = format!("https://example.test/{}", "a".repeat(2049 - prefix_len));
        assert_eq!(
            long_url.len(),
            2049,
            "the fixture URL must be 2,049 characters"
        );

        for invalid in [
            "data:image/png;base64,AAAA",
            "/relative/path.png",
            "ftp://example.test/image.png",
            long_url.as_str(),
        ] {
            let mut context = track_context_without_own_identity();
            context.track.image_url = Some(invalid.to_owned());
            let edits = id3_edits_for_track_context(&context);

            assert!(
                !edits
                    .iter()
                    .any(|edit| edit.frame_label == "TXXX:MusicIndex Image"),
                "{invalid} must give no TXXX:MusicIndex Image edit"
            );
        }
    }

    /// R80-56: two writes of the same edits give one
    /// `TXXX:MusicIndex Image` value, with no duplicate frame.
    ///
    /// This writes only the one frame, not the full edits of a track
    /// context: the "Artwork" row shares the same artwork URL and maps to
    /// `APIC`, whose write downloads the image bytes, an unrelated network
    /// call this test must not depend on.
    #[test]
    fn adr_0080_image_edit_write_is_idempotent() {
        let edits = vec![Id3v24Edit {
            frame_label: "TXXX:MusicIndex Image".into(),
            value: "https://example.test/track.png".into(),
        }];

        let temp = tempfile::NamedTempFile::new().expect("temp file");
        std::fs::write(temp.path(), b"not really an mp3").expect("write file");
        crate::audio_tags::write_id3v24_edits(temp.path(), &edits).expect("first write");
        crate::audio_tags::write_id3v24_edits(temp.path(), &edits).expect("second write");

        let tags = crate::audio_tags::read_audio_tags(temp.path()).expect("read tags back");
        let values = tags
            .fields
            .iter()
            .filter(|field| field.frame_id == "TXXX:MusicIndex Image")
            .map(|field| field.value.as_str())
            .collect::<Vec<_>>();
        assert_eq!(values, vec!["https://example.test/track.png"]);
    }

    /// R80-09: a round trip. Fresh edits, written to a file and read back,
    /// carry the same value under the same frame label. A DB-backed scan
    /// (`application::queries::tag_update::changed_frames`) compares a file
    /// the same way, so it reports no difference for a file this app wrote.
    #[test]
    fn adr_0080_round_trip_edits_read_back_unchanged() {
        let temp = tempfile::NamedTempFile::new().expect("temp file");
        std::fs::write(temp.path(), b"not really an mp3").expect("write file");
        let edits = id3_edits_for_track_context(&track_context_with_own_identity());

        crate::audio_tags::write_id3v24_edits(temp.path(), &edits).expect("write edits");
        let tags = crate::audio_tags::read_audio_tags(temp.path()).expect("read tags back");

        for expected in &edits {
            let found = tags.fields.iter().any(|field| {
                field.frame_id == expected.frame_label && field.value == expected.value
            });
            assert!(
                found,
                "frame {} must read back with the value the writer produced",
                expected.frame_label
            );
        }
    }

    /// R49-04 (ADR 0075 packet 049): a track with its own date gives a
    /// `TDRC` edit with that date.
    #[test]
    fn adr_0075_release_date_r49_04_own_date_writes_tdrc() {
        let mut context = track_context_without_own_identity();
        context.track.pub_date = Some(1_704_067_200);
        let edits = id3_edits_for_track_context(&context);
        assert!(edits.contains(&edit("TDRC", "2024-01-01")));
    }

    /// R49-05 (ADR 0075 packet 049): a track without its own date gets no
    /// `TDRC` edit from a feed release date or an oldest item date.
    #[test]
    fn adr_0075_release_date_r49_05_no_own_date_writes_no_tdrc() {
        let mut context = track_context_without_own_identity();
        context.feed.as_mut().expect("feed").release_date = Some(1_672_531_200);
        context.feed.as_mut().expect("feed").oldest_item_at = Some(1_640_995_200);
        let edits = id3_edits_for_track_context(&context);
        assert!(
            !edits.iter().any(|edit| edit.frame_label == "TDRC"),
            "a track with no own date must write no TDRC edit"
        );
    }

    fn funding_link(url: Option<&str>, label: &str) -> SourceEntityLink {
        SourceEntityLink {
            link_type: Some("funding".into()),
            url: url.map(str::to_owned),
            label: Some(label.to_owned()),
            ..Default::default()
        }
    }

    fn wpay_values(edits: &[Id3v24Edit]) -> Vec<&str> {
        edits
            .iter()
            .filter(|edit| edit.frame_label == "WPAY")
            .map(|edit| edit.value.as_str())
            .collect()
    }

    /// ADR 0080 Decision 10: the item funding link wins over the channel
    /// links.
    #[test]
    fn adr_0080_funding_edit_prefers_the_item_link() {
        let mut context = track_context_with_own_identity();
        context
            .track
            .source_links
            .as_mut()
            .expect("links")
            .push(funding_link(Some("https://item.test/tip"), "Tip jar"));
        context
            .feed
            .as_mut()
            .expect("feed")
            .source_links
            .as_mut()
            .expect("links")
            .push(funding_link(Some("https://channel.test/pay"), "Support"));

        let edits = id3_edits_for_track_context(&context);

        assert_eq!(wpay_values(&edits), vec!["https://item.test/tip"]);
    }

    /// ADR 0080 Decision 10: without an item link, the edit holds the
    /// first channel link. It holds no label text.
    #[test]
    fn adr_0080_funding_edit_falls_back_to_the_first_channel_link() {
        let mut context = track_context_without_own_identity();
        let links = context
            .feed
            .as_mut()
            .expect("feed")
            .source_links
            .as_mut()
            .expect("links");
        links.push(funding_link(
            Some("https://channel.test/one"),
            "First label",
        ));
        links.push(funding_link(
            Some("https://channel.test/two"),
            "Second label",
        ));

        let edits = id3_edits_for_track_context(&context);

        assert_eq!(wpay_values(&edits), vec!["https://channel.test/one"]);
    }

    /// ADR 0080 Decision 10: a value with no `http` or `https` scheme gives
    /// no frame, and the next channel value is used.
    #[test]
    fn adr_0080_funding_edit_skips_a_value_with_no_http_scheme() {
        let mut context = track_context_without_own_identity();
        let links = context
            .feed
            .as_mut()
            .expect("feed")
            .source_links
            .as_mut()
            .expect("links");
        links.push(funding_link(None, "Null url"));
        links.push(funding_link(Some("javascript:alert(1)"), "Script"));
        links.push(funding_link(Some("ftp://channel.test/file"), "Ftp"));
        links.push(funding_link(Some("channel.test/no-scheme"), "No scheme"));
        links.push(funding_link(Some("https://channel.test/valid"), "Valid"));

        let edits = id3_edits_for_track_context(&context);
        assert_eq!(wpay_values(&edits), vec!["https://channel.test/valid"]);

        context
            .feed
            .as_mut()
            .expect("feed")
            .source_links
            .as_mut()
            .expect("links")
            .pop();
        let edits = id3_edits_for_track_context(&context);
        assert!(
            wpay_values(&edits).is_empty(),
            "no valid funding link must give no WPAY edit"
        );
    }

    /// ADR 0080 Decision 10: an item link that is not valid does not block
    /// the channel link.
    #[test]
    fn adr_0080_funding_edit_uses_the_channel_link_past_an_invalid_item_link() {
        let mut context = track_context_with_own_identity();
        context
            .track
            .source_links
            .as_mut()
            .expect("links")
            .push(funding_link(Some("javascript:alert(1)"), "Script"));
        context
            .feed
            .as_mut()
            .expect("feed")
            .source_links
            .as_mut()
            .expect("links")
            .push(funding_link(Some("https://channel.test/pay"), "Support"));

        let edits = id3_edits_for_track_context(&context);

        assert_eq!(wpay_values(&edits), vec!["https://channel.test/pay"]);
    }

    /// ADR 0080 Decision 10: a funding row gives no `WOAR`, `WOAF` or
    /// transcript value, and the compare grid shows it in its own rows.
    #[test]
    fn adr_0080_funding_row_gives_no_website_or_transcript_edit() {
        let mut context = track_context_without_own_identity();
        context
            .track
            .source_links
            .get_or_insert_with(Vec::new)
            .push(funding_link(Some("https://item.test/tip"), "Tip jar"));
        let feed = context.feed.as_mut().expect("feed");
        feed.source_links = Some(vec![funding_link(
            Some("https://channel.test/pay"),
            "Support",
        )]);

        let edits = id3_edits_for_track_context(&context);

        for frame in [
            "WOAF",
            "WOAR",
            "SYLT:MusicIndex Transcript",
            "USLT:MusicIndex Transcript",
        ] {
            assert!(
                !edits.iter().any(|edit| edit.frame_label == frame),
                "a funding row must give no {frame} edit: {edits:?}"
            );
        }
        let rows = crate::metadata::track_metadata_rows(&context, None, false);
        let fields = rows
            .iter()
            .filter_map(|row| match row {
                crate::metadata::MetadataGridRow::Data(row) => Some(row.field.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(fields.contains(&"Funding"), "{fields:?}");
        assert!(fields.contains(&"RSS feed funding"), "{fields:?}");
    }

    /// ADR 0080 Decision 10: a write of the freshly built edits reads back
    /// unchanged, and a second write gives equal frames with one `WPAY`.
    #[test]
    fn adr_0080_funding_edit_round_trips_with_one_frame() {
        let mut context = track_context_with_own_identity();
        context
            .track
            .source_links
            .as_mut()
            .expect("links")
            .push(funding_link(Some("https://item.test/tip"), "Tip jar"));
        let edits = vec![edit("WPAY", "https://item.test/tip")];
        assert_eq!(
            id3_edits_for_track_context(&context)
                .into_iter()
                .filter(|edit| edit.frame_label == "WPAY")
                .collect::<Vec<_>>(),
            edits
        );

        let temp = tempfile::NamedTempFile::new().expect("temp file");
        std::fs::write(temp.path(), b"not really an mp3").expect("write file");
        crate::audio_tags::write_id3v24_edits(temp.path(), &edits).expect("first write");
        let first = crate::audio_tags::read_audio_tags(temp.path())
            .expect("read")
            .fields;
        crate::audio_tags::write_id3v24_edits(temp.path(), &edits).expect("second write");
        let second = crate::audio_tags::read_audio_tags(temp.path())
            .expect("read")
            .fields;

        assert_eq!(first, second);
        let values = second
            .iter()
            .filter(|field| field.frame_id == "WPAY")
            .map(|field| field.value.as_str())
            .collect::<Vec<_>>();
        assert_eq!(values, vec!["https://item.test/tip"]);
    }
    fn license_edits(edits: &[Id3v24Edit]) -> Vec<(&str, &str)> {
        let mut found = edits
            .iter()
            .filter(|edit| {
                edit.frame_label == "WCOP" || edit.frame_label == "TXXX:MusicIndex License"
            })
            .map(|edit| (edit.frame_label.as_str(), edit.value.as_str()))
            .collect::<Vec<_>>();
        found.sort_by_key(|(label, _)| *label != "WCOP");
        found
    }

    /// ADR 0080 Decision 11: the edits of a track with its own license hold
    /// its URL and its text. Without it, they hold the channel values.
    #[test]
    fn adr_0080_license_edit_prefers_the_item_license() {
        let mut context = track_context_with_own_identity();
        context.feed.as_mut().expect("feed").license_url = Some("https://channel.test/by".into());
        context.feed.as_mut().expect("feed").license_text = Some("channel-license".into());

        let channel_only = id3_edits_for_track_context(&context);
        assert_eq!(
            license_edits(&channel_only),
            vec![
                ("WCOP", "https://channel.test/by"),
                ("TXXX:MusicIndex License", "channel-license")
            ]
        );

        context.track.license_url = Some("https://item.test/by".into());
        context.track.license_text = Some("item-license".into());
        let own = id3_edits_for_track_context(&context);
        assert_eq!(
            license_edits(&own),
            vec![
                ("WCOP", "https://item.test/by"),
                ("TXXX:MusicIndex License", "item-license")
            ]
        );
    }

    /// ADR 0080 Decision 11: text with no `url` gives only the text frame. A
    /// `url` that is not `http` or `https` gives no URL frame. An item
    /// license with no URL does not take the URL of the channel.
    #[test]
    fn adr_0080_license_edit_text_alone_and_invalid_url() {
        let mut context = track_context_without_own_identity();
        context.track.license_text = Some("All rights reserved".into());
        context.feed.as_mut().expect("feed").license_url = Some("https://channel.test/by".into());

        let edits = id3_edits_for_track_context(&context);
        assert_eq!(
            license_edits(&edits),
            vec![("TXXX:MusicIndex License", "All rights reserved")]
        );

        context.track.license_url = Some("my-podcast-license-v1".into());
        context.track.license_text = Some("my-podcast-license-v1".into());
        let edits = id3_edits_for_track_context(&context);
        assert_eq!(
            license_edits(&edits),
            vec![("TXXX:MusicIndex License", "my-podcast-license-v1")]
        );

        context.track.license_url = Some("ftp://item.test/by".into());
        let edits = id3_edits_for_track_context(&context);
        assert!(edits.iter().all(|edit| edit.frame_label != "WCOP"));
    }

    /// ADR 0080 Decision 11: the grid shows the RSS license on its own rows,
    /// apart from the "License" row of MusicBrainz.
    #[test]
    fn adr_0080_license_rows_stay_apart_from_the_musicbrainz_row() {
        let mut context = track_context_without_own_identity();
        context.track.license_url = Some("https://item.test/by".into());
        context.track.license_text = Some("cc-by-4.0".into());
        let candidate = crate::musicbrainz::MusicBrainzCandidate {
            recording_id: "recording-id".into(),
            url_relations: vec![crate::musicbrainz::MusicBrainzUrlRelation {
                relation_type: "license".into(),
                url: "https://mb.example/license".into(),
                owner: crate::musicbrainz::MusicBrainzRelationOwner::Release,
            }],
            ..Default::default()
        };

        let rows = crate::metadata::track_metadata_rows(&context, Some(&candidate), true);
        let data = |field: &str| {
            rows.iter()
                .filter_map(|row| match row {
                    crate::metadata::MetadataGridRow::Data(row) if row.field == field => Some(row),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };

        let url = data("License URL");
        let name = data("License name");
        let musicbrainz = data("License");
        assert_eq!(url.len(), 1);
        assert_eq!(url[0].rss_value.as_deref(), Some("https://item.test/by"));
        assert_eq!(url[0].id3_frame.as_deref(), Some("WCOP"));
        assert_eq!(name.len(), 1);
        assert_eq!(name[0].rss_value.as_deref(), Some("cc-by-4.0"));
        assert_eq!(
            name[0].id3_frame.as_deref(),
            Some("TXXX:MusicIndex License")
        );
        assert_eq!(musicbrainz.len(), 1);
        assert_eq!(musicbrainz[0].rss_value, None);
        assert_eq!(
            musicbrainz[0].musicbrainz_value.as_deref(),
            Some("https://mb.example/license")
        );
        assert_ne!(url[0].row_id, musicbrainz[0].row_id);
    }

    /// ADR 0080 Decision 12: the confirmed labels of the context give one
    /// `TPUB` edit. The `itunes:owner` name gives `TXXX:V4V_PUBLISHER` only,
    /// and the RSS value of the "Label" row is empty for it.
    #[test]
    fn adr_0080_label_owner_name_gives_no_tpub_and_a_confirmed_label_does() {
        let mut context = track_context_without_own_identity();
        context.track.publisher_text = Some("Owner Name".into());
        let edits = id3_edits_for_track_context(&context);
        assert!(edits.iter().all(|edit| edit.frame_label != "TPUB"));
        assert!(edits
            .iter()
            .any(|edit| edit.frame_label == "TXXX:V4V_PUBLISHER" && edit.value == "Owner Name"));
        let rows = crate::metadata::track_metadata_rows(&context, None, false);
        let label = rows.iter().find_map(|row| match row {
            crate::metadata::MetadataGridRow::Data(row) if row.field == "Label" => Some(row),
            _ => None,
        });
        assert_eq!(label.expect("Label row").rss_value, None);

        context.confirmed_labels = vec!["First Label".into(), "Second Label".into()];
        let edits = id3_edits_for_track_context(&context);
        let tpub = edits
            .iter()
            .filter(|edit| edit.frame_label == "TPUB")
            .map(|edit| edit.value.as_str())
            .collect::<Vec<_>>();
        assert_eq!(tpub, vec!["First Label \u{b7} Second Label"]);
    }

    /// ADR 0080 Decision 12: the MusicBrainz label shares the "Label" row and
    /// stays apart from the RSS value.
    #[test]
    fn adr_0080_label_musicbrainz_label_shares_the_row_and_stays_apart() {
        let mut context = track_context_without_own_identity();
        context.confirmed_labels = vec!["Rss Label".into()];
        let candidate = crate::musicbrainz::MusicBrainzCandidate {
            recording_id: "recording-id".into(),
            labels: vec!["Mb Label".into()],
            ..Default::default()
        };
        let rows = crate::metadata::track_metadata_rows(&context, Some(&candidate), true);
        let labels = rows
            .iter()
            .filter_map(|row| match row {
                crate::metadata::MetadataGridRow::Data(row) if row.field == "Label" => Some(row),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(labels.len(), 1, "{labels:?}");
        assert_eq!(labels[0].rss_value.as_deref(), Some("Rss Label"));
        assert_eq!(labels[0].musicbrainz_value.as_deref(), Some("Mb Label"));
        assert_eq!(labels[0].id3_frame.as_deref(), Some("TPUB"));
    }
}
