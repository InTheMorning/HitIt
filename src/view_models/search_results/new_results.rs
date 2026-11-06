//! Result keys and the count of new results of a saved search (ADR 0089
//! Decision 4, packet 007 Part C).
//!
//! A result key is the identity of one result row: a release key, a track
//! key or an artist key. Its text is stored in `saved_search_seen`. The
//! count of new results of a saved search is its result keys minus the
//! seen keys. The module has no GPUI type and no I/O.

#![warn(clippy::pedantic)]

use std::collections::BTreeSet;

use crate::view_models::workspace::{ReleaseKey, TrackKey};

use super::SearchResultsInspectorPageVm;

/// The identity of one result row. Its text is the stored form.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ResultKey {
    /// A release row of the Releases tab.
    Release(ReleaseKey),
    /// A track row of the Tracks tab.
    Track(TrackKey),
    /// An artist row of the Artists tab, by the artist identifier of its
    /// source.
    Artist(String),
}

impl ResultKey {
    /// The stored text of the key. Each kind has its own prefix, and the prefixes
    /// use no colon, so no query field token is written here. The track
    /// text gives the length of its release text, so two keys never share
    /// one text.
    #[must_use]
    pub(crate) fn text(&self) -> String {
        match self {
            Self::Release(key) => release_text(key),
            Self::Track(key) => {
                let release = key.release.as_ref().map(release_text).unwrap_or_default();
                format!("track/{}/{}/{}", release.len(), release, key.item_guid)
            }
            Self::Artist(id) => format!("artist/{id}"),
        }
    }
}

fn release_text(key: &ReleaseKey) -> String {
    match key {
        ReleaseKey::FeedGuid(guid) => format!("release/feed/{guid}"),
        ReleaseKey::LibraryFeed(id) => format!("release/library/{id}"),
    }
}

/// The count of new results of a saved search (ADR 0089 Decision 4).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NewResultCount {
    /// The count of the new results. Zero is a count, and it shows no text.
    Count(usize),
    /// The last run failed, or it did not run. No count shows, and no zero.
    Unavailable,
}

/// The count of new results of one saved search: `result_keys` minus
/// `seen`. A saved search with no seen set gives zero. Its first run stores
/// its keys, so an upgrade shows no false count.
#[must_use]
pub(crate) fn new_result_count(
    result_keys: &BTreeSet<String>,
    seen: Option<&BTreeSet<String>>,
) -> usize {
    match seen {
        None => 0,
        Some(seen) => result_keys
            .iter()
            .filter(|key| !seen.contains(*key))
            .count(),
    }
}

/// The text of a count of new results. `count_text` is the short badge and
/// `sentence` is the accessibility sentence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NewResultsText {
    /// For example "3 new".
    pub(crate) count_text: String,
    /// For example "3 new results".
    pub(crate) sentence: String,
}

/// The text of `count`. A zero count and an unavailable count give no
/// text (ADR 0089 Decision 4).
#[must_use]
pub(crate) fn new_results_text(count: NewResultCount) -> Option<NewResultsText> {
    match count {
        NewResultCount::Count(number) if number > 0 => Some(NewResultsText {
            count_text: format!("{number} new"),
            sentence: format!(
                "{number} new {}",
                if number == 1 { "result" } else { "results" }
            ),
        }),
        NewResultCount::Count(_) | NewResultCount::Unavailable => None,
    }
}

impl SearchResultsInspectorPageVm {
    /// The stored text of each result key of the page: the release rows,
    /// the artist rows and the track rows that the page holds now.
    #[must_use]
    pub(crate) fn result_keys(&self) -> BTreeSet<String> {
        let releases = self
            .releases
            .rows()
            .into_iter()
            .filter_map(|row| row.key)
            .map(|key| ResultKey::Release(key).text());
        let artists = self
            .artists
            .rows()
            .into_iter()
            .map(|row| ResultKey::Artist(row.id).text());
        let tracks = self
            .tracks
            .rows()
            .into_iter()
            .filter_map(|row| row.key)
            .map(|key| ResultKey::Track(key).text());
        releases.chain(artists).chain(tracks).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed(guid: &str) -> ReleaseKey {
        ReleaseKey::FeedGuid(guid.to_owned())
    }

    fn track(release: Option<ReleaseKey>, item: &str) -> TrackKey {
        TrackKey {
            release,
            item_guid: item.to_owned(),
        }
    }

    fn set(keys: &[&str]) -> BTreeSet<String> {
        keys.iter().map(|key| (*key).to_owned()).collect()
    }

    #[test]
    fn release_keys_have_one_text_for_each_origin() {
        assert_eq!(
            ResultKey::Release(feed("guid-1")).text(),
            "release/feed/guid-1"
        );
        assert_eq!(
            ResultKey::Release(ReleaseKey::LibraryFeed(7)).text(),
            "release/library/7"
        );
    }

    #[test]
    fn track_key_text_keeps_the_release_and_the_item() {
        assert_eq!(
            ResultKey::Track(track(Some(feed("guid-1")), "item-1")).text(),
            "track/19/release/feed/guid-1/item-1"
        );
        assert_eq!(
            ResultKey::Track(track(None, "item-1")).text(),
            "track/0//item-1"
        );
    }

    #[test]
    fn track_keys_with_colons_in_their_parts_stay_apart() {
        let first = ResultKey::Track(track(Some(feed("a")), "b:item")).text();
        let second = ResultKey::Track(track(Some(feed("a:b")), "item")).text();
        assert_ne!(first, second);
    }

    #[test]
    fn artist_key_text_uses_the_artist_identifier() {
        assert_eq!(
            ResultKey::Artist("artist-9".to_owned()).text(),
            "artist/artist-9"
        );
    }

    #[test]
    fn new_result_count_is_results_minus_seen() {
        let results = set(&["release/feed/a", "release/feed/b", "release/feed/c"]);
        let seen = set(&["release/feed/a", "initialised"]);
        assert_eq!(new_result_count(&results, Some(&seen)), 2);
    }

    #[test]
    fn new_result_count_is_zero_for_a_first_run_with_no_seen_set() {
        let results = set(&["release/feed/a", "release/feed/b"]);
        assert_eq!(new_result_count(&results, None), 0);
    }

    #[test]
    fn new_result_count_is_zero_after_every_key_is_seen() {
        let results = set(&["release/feed/a", "artist/x"]);
        let seen = set(&["release/feed/a", "artist/x", "initialised"]);
        assert_eq!(new_result_count(&results, Some(&seen)), 0);
    }

    #[test]
    fn new_results_text_is_absent_for_zero_and_for_unavailable() {
        assert_eq!(new_results_text(NewResultCount::Count(0)), None);
        assert_eq!(new_results_text(NewResultCount::Unavailable), None);
    }

    #[test]
    fn new_results_text_names_the_count_and_the_sentence() {
        let three = new_results_text(NewResultCount::Count(3)).unwrap();
        assert_eq!(three.count_text, "3 new");
        assert_eq!(three.sentence, "3 new results");
        let one = new_results_text(NewResultCount::Count(1)).unwrap();
        assert_eq!(one.sentence, "1 new result");
    }
}
