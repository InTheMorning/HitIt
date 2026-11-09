//! The text that counts the releases of an artist row (musicindex ADR 0023
//! R-WORD-9). The view model picks the words. The UI composes no text.

use std::collections::BTreeMap;

use crate::view_models::artist_identity::ARTIST_ROW_SEPARATOR;

/// The words of each kind, singular and plural, in the fixed order of the
/// text: albums, EPs, singles, playlists. The track counts of a map give an
/// album or a single only.
const KIND_WORDS: [(&str, &str); 4] = [
    ("album", "albums"),
    ("EP", "EPs"),
    ("single", "singles"),
    ("playlist", "playlists"),
];
const ALBUM: usize = 0;
const SINGLE: usize = 2;

/// The count text of a row. `track_counts` maps a track count, as text, to
/// the number of releases. The text gives the total when the map is absent,
/// when a key or a value is not a whole number, or when the values do not
/// sum to `total`.
#[must_use]
pub(crate) fn release_count_text(
    track_counts: Option<&BTreeMap<String, i64>>,
    total: i64,
) -> String {
    track_counts
        .and_then(|map| kind_counts(map, total))
        .map_or_else(|| total_text(total), kinds_text)
}

/// The secondary text of a row with `parts` first and the count text last.
/// A row with no total adds no count part.
#[must_use]
pub(crate) fn with_release_count(
    parts: &[&str],
    track_counts: Option<&BTreeMap<String, i64>>,
    total: Option<i64>,
) -> String {
    let count = total.map(|total| release_count_text(track_counts, total));
    parts
        .iter()
        .copied()
        .chain(count.as_deref())
        .collect::<Vec<_>>()
        .join(ARTIST_ROW_SEPARATOR)
}

fn total_text(total: i64) -> String {
    format!("{total} release{}", if total == 1 { "" } else { "s" })
}

/// The release count of each kind, or `None` when the map is not valid.
fn kind_counts(map: &BTreeMap<String, i64>, total: i64) -> Option<[i64; 4]> {
    let mut kinds = [0_i64; 4];
    let mut sum = 0_i64;
    for (tracks, &releases) in map {
        let tracks: i64 = tracks.parse().ok()?;
        if releases < 0 {
            return None;
        }
        kinds[if tracks == 1 { SINGLE } else { ALBUM }] += releases;
        sum = sum.checked_add(releases)?;
    }
    (sum == total && sum > 0).then_some(kinds)
}

/// The text of the kind counts. At most two kinds show. The releases of the
/// other kinds follow as "n more".
fn kinds_text(kinds: [i64; 4]) -> String {
    let present: Vec<(usize, i64)> = kinds
        .into_iter()
        .enumerate()
        .filter(|&(_, count)| count > 0)
        .collect();
    let mut parts: Vec<String> = present
        .iter()
        .take(2)
        .map(|&(kind, count)| {
            let (one, many) = KIND_WORDS[kind];
            format!("{count} {}", if count == 1 { one } else { many })
        })
        .collect();
    let more: i64 = present.iter().skip(2).map(|&(_, count)| count).sum();
    if more > 0 {
        parts.push(format!("{more} more"));
    }
    parts.join(ARTIST_ROW_SEPARATOR)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RULE: &str = "musicindex ADR 0023 R-WORD-9";

    fn counts(pairs: &[(&str, i64)]) -> BTreeMap<String, i64> {
        pairs.iter().map(|&(k, v)| (k.to_owned(), v)).collect()
    }

    fn text(pairs: &[(&str, i64)], total: i64) -> String {
        release_count_text(Some(&counts(pairs)), total)
    }

    #[test]
    fn r_word_9_one_track_gives_a_single() {
        assert_eq!(text(&[("1", 17)], 17), "17 singles", "{RULE}: jSmovie");
    }

    #[test]
    fn r_word_9_kinds_come_in_a_fixed_order() {
        assert_eq!(
            text(&[("1", 38), ("9", 14)], 52),
            "14 albums \u{b7} 38 singles",
            "{RULE}: albums before singles"
        );
        assert_eq!(
            text(&[("1", 2), ("7", 1)], 3),
            "1 album \u{b7} 2 singles",
            "{RULE}: $2 Holla"
        );
    }

    #[test]
    fn r_word_9_a_count_of_one_is_singular() {
        assert_eq!(
            text(&[("1", 1), ("4", 1)], 2),
            "1 album \u{b7} 1 single",
            "{RULE}: singular"
        );
    }

    #[test]
    fn r_word_9_zero_tracks_counts_as_an_album() {
        assert_eq!(
            text(&[("0", 2), ("5", 1)], 3),
            "3 albums",
            "{RULE}: each count but one gives an album"
        );
    }

    #[test]
    fn r_word_9_no_map_gives_the_total() {
        assert_eq!(release_count_text(None, 17), "17 releases", "{RULE}");
        assert_eq!(release_count_text(None, 1), "1 release", "{RULE}");
        assert_eq!(text(&[("1", 1)], 1), "1 single", "{RULE}");
    }

    #[test]
    fn r_word_9_a_map_that_does_not_sum_gives_the_total() {
        assert_eq!(
            text(&[("1", 2), ("7", 1)], 5),
            "5 releases",
            "{RULE}: the values must sum to the total"
        );
    }

    #[test]
    fn r_word_9_a_key_that_is_not_a_number_gives_the_total() {
        assert_eq!(
            text(&[("1", 2), ("many", 1)], 3),
            "3 releases",
            "{RULE}: the map is rejected"
        );
    }

    #[test]
    fn r_word_9_an_empty_map_gives_the_total() {
        assert_eq!(text(&[], 0), "0 releases", "{RULE}: a publisher with 0");
    }

    #[test]
    fn r_word_9_three_kinds_show_two_and_n_more() {
        assert_eq!(
            kinds_text([2, 1, 5, 3]),
            "2 albums \u{b7} 1 EP \u{b7} 8 more",
            "{RULE}: two kinds at most"
        );
    }

    #[test]
    fn r_word_9_count_is_the_last_part_of_the_row_text() {
        let map = counts(&[("1", 18), ("3", 1)]);
        assert_eq!(
            with_release_count(&["Not in Library"], Some(&map), Some(19)),
            "Not in Library \u{b7} 1 album \u{b7} 18 singles",
            "{RULE}"
        );
        assert_eq!(
            with_release_count(&["Not in Library"], Some(&map), None),
            "Not in Library",
            "{RULE}: no total, no count part"
        );
    }
}
