//! The letter bar of a Music list page (ADR 0089 Decision 6, musicindex
//! ADR 0027 R-LIST-1 item 2).
//!
//! The bar shows only with the sort A to Z. It has the letters A to Z and
//! the key "#" for the names that start with no letter. In the scope "All"
//! a letter sets the `from` text of the name request. In the scope "In
//! Library" a letter moves the list to its first row. This module gives
//! the letters and the rules of the two actions. It has no renderer type.
//! `src/ui/composites/letter_bar.rs` draws the bar.

#![warn(clippy::pedantic)]

use crate::view_models::index_name_list::{from_keeps, name_sort_key};
use crate::view_models::list_page::ListPageKey;

/// Whether the name `name` shows at the start `from` of a name list (ADR
/// 0089 Decision 6). `from` is a sort key, as the request sends it. `None`
/// shows every name. `reversed` follows the order of the node list
/// ([`from_keeps`]).
#[must_use]
pub(crate) fn name_at_or_after(name: &str, from: Option<&str>, reversed: bool) -> bool {
    from_keeps(&name_sort_key(name), from, reversed)
}

/// One key of the letter bar. `Other` is the key "#", for a name that starts
/// with no ASCII letter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LetterKey {
    /// A letter, in upper case.
    Letter(char),
    /// The key "#".
    Other,
}

impl LetterKey {
    /// The key of the name `name`. The name is read in its sort key form.
    #[must_use]
    pub(crate) fn of_name(name: &str) -> Self {
        match name_sort_key(name).chars().next() {
            Some(first) if first.is_ascii_alphabetic() => Self::Letter(first.to_ascii_uppercase()),
            _ => Self::Other,
        }
    }

    /// The `from` text of the name request for this key (Stophammer
    /// `from`). The key "#" gives `None`, so the list starts at its first
    /// row, which holds the names that start with no letter.
    #[must_use]
    pub(crate) fn to_from_text(self) -> Option<String> {
        match self {
            Self::Letter(letter) => Some(letter.to_ascii_lowercase().to_string()),
            Self::Other => None,
        }
    }

    /// The visible label of the key.
    #[must_use]
    pub(crate) fn label(self) -> String {
        match self {
            Self::Letter(letter) => letter.to_string(),
            Self::Other => "#".to_owned(),
        }
    }

    /// The part of the accessibility label that names the key.
    fn a11y_part(self) -> String {
        match self {
            Self::Letter(letter) => letter.to_string(),
            Self::Other => "a number or symbol".to_owned(),
        }
    }
}

/// One letter of the bar.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LetterVm {
    /// Stable element id.
    pub(crate) id: String,
    /// The key that the letter selects.
    pub(crate) key: LetterKey,
    /// The visible label, for example "A" or "#".
    pub(crate) label: String,
    /// The accessibility label. A selected letter says so, so the mark is
    /// not the only sign of the selection.
    pub(crate) a11y_label: String,
    /// Whether this letter is the selected letter.
    pub(crate) selected: bool,
    /// Whether the keyboard cursor of the bar is on this letter and shows.
    pub(crate) cursor: bool,
}

/// One move of the keyboard cursor in the letter bar. The bar is one tab stop,
/// and the arrow keys move inside it (WAI-ARIA toolbar pattern).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LetterStep {
    /// Left: the letter before, or the last letter from the first.
    Previous,
    /// Right: the next letter, or the first letter from the last.
    Next,
    /// Home: the first letter.
    First,
    /// End: the last letter, "#".
    Last,
}

/// The letter bar of a list page: A to Z, then "#" (R-LIST-1 item 2).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LetterBarVm {
    /// Stable element id of the bar.
    pub(crate) control_id: String,
    /// The letters, in order.
    pub(crate) letters: Vec<LetterVm>,
}

impl LetterBarVm {
    /// The letter bar of `key`, with `selected` marked.
    #[must_use]
    pub(crate) fn new(key: ListPageKey, selected: Option<LetterKey>) -> Self {
        let noun = key.noun();
        let keys = ('A'..='Z')
            .map(LetterKey::Letter)
            .chain(std::iter::once(LetterKey::Other));
        let letters = keys
            .map(|letter| {
                let is_selected = selected == Some(letter);
                let current = if is_selected { ", selected" } else { "" };
                LetterVm {
                    id: format!("music-letter-{}-{}", noun, letter.label()),
                    key: letter,
                    label: letter.label(),
                    a11y_label: format!(
                        "Show {noun} starting with {}{current}",
                        letter.a11y_part()
                    ),
                    selected: is_selected,
                    cursor: false,
                }
            })
            .collect();
        Self {
            control_id: format!("music-letters-{noun}"),
            letters,
        }
    }

    /// The letter of the keyboard cursor: `cursor`, else the selected
    /// letter, else the first letter. The bar always has letters.
    #[must_use]
    pub(crate) fn cursor_key(&self, cursor: Option<LetterKey>) -> LetterKey {
        cursor
            .or_else(|| {
                self.letters
                    .iter()
                    .find(|letter| letter.selected)
                    .map(|letter| letter.key)
            })
            .unwrap_or(self.letters[0].key)
    }

    /// The letter after one move from `cursor`.
    #[must_use]
    pub(crate) fn moved(&self, cursor: Option<LetterKey>, step: LetterStep) -> LetterKey {
        let count = self.letters.len();
        let current = self.cursor_key(cursor);
        let index = self
            .letters
            .iter()
            .position(|letter| letter.key == current)
            .unwrap_or(0);
        let next = match step {
            LetterStep::Previous => (index + count - 1) % count,
            LetterStep::Next => (index + 1) % count,
            LetterStep::First => 0,
            LetterStep::Last => count - 1,
        };
        self.letters[next].key
    }

    /// The same bar with the keyboard cursor on the letter of `cursor`.
    #[must_use]
    pub(crate) fn with_cursor_shown(mut self, cursor: Option<LetterKey>) -> Self {
        let key = self.cursor_key(cursor);
        for letter in &mut self.letters {
            letter.cursor = letter.key == key;
        }
        self
    }
}

/// The index of the first item of the key `key` (ADR 0089 Decision 6, the
/// scope "In Library"). `labels` gives the label of each item, in list
/// order, and `None` for an item with no name, such as a status row. The
/// list is in name order, so the first match is the first row of the key.
#[must_use]
pub(crate) fn first_letter_index<'a>(
    labels: impl IntoIterator<Item = Option<&'a str>>,
    key: LetterKey,
) -> Option<usize> {
    let keys: Vec<Option<LetterKey>> = labels
        .into_iter()
        .map(|label| label.map(LetterKey::of_name))
        .collect();
    if key != LetterKey::Other {
        return keys.iter().position(|item| *item == Some(key));
    }
    // The text order of the reader (musicindex ADR 0053 R-LIST-12) puts a
    // name with an accented first letter among the letters. Only a name
    // before the first letter or after the last letter starts the "#" part.
    let is_letter = |item: &Option<LetterKey>| matches!(item, Some(LetterKey::Letter(_)));
    let first_letter = keys.iter().position(is_letter);
    let last_letter = keys.iter().rposition(is_letter);
    keys.iter().enumerate().find_map(|(index, item)| {
        let outside = match (first_letter, last_letter) {
            (Some(first), Some(last)) => index < first || index > last,
            _ => true,
        };
        (outside && *item == Some(LetterKey::Other)).then_some(index)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Walk of 2026-11-06: the bar is one unit, and the arrow keys move in it.
    #[test]
    fn adr_0089_letter_cursor_starts_on_the_selected_letter_and_wraps() {
        let bar = LetterBarVm::new(ListPageKey::Artists, Some(LetterKey::Letter('M')));
        assert_eq!(bar.cursor_key(None), LetterKey::Letter('M'));
        assert_eq!(bar.moved(None, LetterStep::Next), LetterKey::Letter('N'));
        assert_eq!(bar.moved(None, LetterStep::First), LetterKey::Letter('A'));
        assert_eq!(bar.moved(None, LetterStep::Last), LetterKey::Other);
        assert_eq!(
            bar.moved(Some(LetterKey::Other), LetterStep::Next),
            LetterKey::Letter('A')
        );
        assert_eq!(
            bar.moved(Some(LetterKey::Letter('A')), LetterStep::Previous),
            LetterKey::Other
        );

        let free = LetterBarVm::new(ListPageKey::Artists, None);
        assert_eq!(free.cursor_key(None), LetterKey::Letter('A'));
        let shown = free.with_cursor_shown(Some(LetterKey::Letter('C')));
        let marked: Vec<_> = shown
            .letters
            .iter()
            .filter(|letter| letter.cursor)
            .collect();
        assert_eq!(marked.len(), 1);
        assert_eq!(marked[0].key, LetterKey::Letter('C'));
    }

    /// R-LIST-1 item 2: each letter has its key, its label and its
    /// accessibility label, and the selected letter says so.
    #[test]
    fn adr_0089_letter_bar_has_a_to_z_and_hash_with_labels() {
        let bar = LetterBarVm::new(ListPageKey::Releases, Some(LetterKey::Letter('C')));
        assert_eq!(bar.control_id, "music-letters-releases");
        assert_eq!(bar.letters.len(), 27);
        assert_eq!(bar.letters[0].label, "A");
        assert_eq!(bar.letters[26].label, "#");
        assert_eq!(bar.letters[26].key, LetterKey::Other);
        for letter in &bar.letters {
            assert!(!letter.a11y_label.is_empty(), "{}", letter.label);
            assert!(letter.a11y_label.starts_with("Show releases starting with"));
        }
        let selected: Vec<&str> = bar
            .letters
            .iter()
            .filter(|letter| letter.selected)
            .map(|letter| letter.label.as_str())
            .collect();
        assert_eq!(selected, ["C"]);
        assert!(bar.letters[2].a11y_label.ends_with(", selected"));
        assert!(!bar.letters[3].a11y_label.contains("selected"));
    }

    /// ADR 0089 Decision 6: a name gives its letter, and a name with no
    /// ASCII letter first gives "#". The case and the white space do not
    /// change the key.
    #[test]
    fn adr_0089_letter_key_reads_the_sort_key_of_a_name() {
        assert_eq!(LetterKey::of_name("  Beatles"), LetterKey::Letter('B'));
        assert_eq!(LetterKey::of_name("the Who"), LetterKey::Letter('T'));
        assert_eq!(LetterKey::of_name("2 Chainz"), LetterKey::Other);
        assert_eq!(LetterKey::of_name(""), LetterKey::Other);
    }

    /// ADR 0089 Decision 6: a letter sets `from` to its lower case letter,
    /// and "#" sets no `from`, so the list starts at its first row.
    #[test]
    fn adr_0089_letter_sets_from_for_a_letter_and_none_for_hash() {
        assert_eq!(LetterKey::Letter('B').to_from_text().as_deref(), Some("b"));
        assert_eq!(LetterKey::Other.to_from_text(), None);
    }

    /// ADR 0089 Decision 6: In Library, a letter moves the list to its
    /// first row. A status row has no name, so it is skipped.
    #[test]
    fn adr_0089_letter_scrolls_to_the_first_row_of_the_letter() {
        let labels = [
            None,
            Some("Abba"),
            Some("Adele"),
            Some("Beck"),
            Some("Bjork"),
            Some("Zappa"),
        ];
        assert_eq!(
            first_letter_index(labels, LetterKey::Letter('B')),
            Some(3),
            "the first B row"
        );
        assert_eq!(first_letter_index(labels, LetterKey::Letter('C')), None);
        assert_eq!(first_letter_index(labels, LetterKey::Other), None);
    }

    /// Musicindex ADR 0053 R-LIST-12: in the text order of the reader a name
    /// with an accented first letter sits among the letters, so "#" does not
    /// jump to it.
    #[test]
    fn adr_0053_r12_other_key_skips_an_accented_name_among_the_letters() {
        let labels = [
            Some("$2 Holla"),
            Some("Alpha"),
            Some("\u{c9}lan"),
            Some("Zed"),
        ];
        assert_eq!(first_letter_index(labels, LetterKey::Other), Some(0));
        let labels = [Some("Alpha"), Some("\u{c9}lan"), Some("Zed")];
        assert_eq!(first_letter_index(labels, LetterKey::Other), None);
        let labels = [Some("Alpha"), Some("Zed"), Some("\u{3042}")];
        assert_eq!(first_letter_index(labels, LetterKey::Other), Some(2));
    }
}
