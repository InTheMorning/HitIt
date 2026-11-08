//! The text order of names in the language of the reader (musicindex ADR
//! 0053 R-LIST-12 item 1).
//!
//! A list that the app sorts by name on its own side uses this order. The
//! order reads a number in a name as a number, so "2.mp3" comes before
//! "15.mp3". It ignores case at the first levels, so "abba" and "ABBA" sort
//! together, and an accent sorts near its base letter. The reader locale
//! comes from the system. A locale that cannot be read gives the root
//! locale.
//!
//! A list that must match the order of the node does not use this order.
//! The node owns that order (R-LIST-8 item 3). See `order_key` of the crate
//! `musicindex-search`.

#![warn(clippy::pedantic)]

use std::cmp::Ordering;
use std::sync::OnceLock;

use icu_collator::options::{CollatorOptions, Strength};
use icu_collator::preferences::CollationNumericOrdering;
use icu_collator::{CollatorBorrowed, CollatorPreferences};
use icu_locale_core::Locale;

/// The collator of the reader locale. It is built one time.
static READER_COLLATOR: OnceLock<CollatorBorrowed<'static>> = OnceLock::new();

/// Builds a collator for `locale` with numeric order on. The strength is
/// tertiary, so case and accents decide only when the base letters are
/// equal.
fn build_collator(locale: Option<&Locale>) -> CollatorBorrowed<'static> {
    let mut prefs = locale.map(CollatorPreferences::from).unwrap_or_default();
    prefs.numeric_ordering = Some(CollationNumericOrdering::True);
    let mut options = CollatorOptions::default();
    options.strength = Some(Strength::Tertiary);
    CollatorBorrowed::try_new(prefs, options)
        .or_else(|_| {
            CollatorBorrowed::try_new(CollatorPreferences::default(), CollatorOptions::default())
        })
        .expect("the compiled collation data holds the root locale")
}

/// The locale of the reader, or `None` when the system gives none or a
/// text that is not a locale.
fn reader_locale() -> Option<Locale> {
    sys_locale::get_locale()?.parse().ok()
}

/// The text of a name as a person reads it: no space at the two ends, and
/// one space for each run of white space.
fn read_text(name: &str) -> String {
    name.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Compares two names in the language of the reader (musicindex ADR 0053
/// R-LIST-12 item 1). A number in a name compares as a number.
#[must_use]
pub(crate) fn compare_names(a: &str, b: &str) -> Ordering {
    let collator = READER_COLLATOR.get_or_init(|| build_collator(reader_locale().as_ref()));
    collator.compare(&read_text(a), &read_text(b))
}

/// Sorts `items` by the name that `name` gives, in the language of the
/// reader. The sort is stable, so items with equal names keep their order.
pub(crate) fn sort_by_name<T>(items: &mut [T], name: impl Fn(&T) -> &str) {
    items.sort_by(|a, b| compare_names(name(a), name(b)));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn english() -> CollatorBorrowed<'static> {
        build_collator(Some(&"en-US".parse().expect("a locale")))
    }

    fn sorted_by(collator: &CollatorBorrowed<'static>, names: &[&str]) -> Vec<String> {
        let mut names: Vec<String> = names.iter().map(|name| (*name).to_owned()).collect();
        names.sort_by(|a, b| collator.compare(&read_text(a), &read_text(b)));
        names
    }

    /// Musicindex ADR 0053 R-LIST-12 item 1: a number is a number.
    #[test]
    fn adr_0053_r12_numbers_sort_as_numbers() {
        assert_eq!(compare_names("2.mp3", "15.mp3"), Ordering::Less);
        assert_eq!(compare_names("Track 9", "Track 10"), Ordering::Less);
        assert_eq!(compare_names("Track 10", "Track 9"), Ordering::Greater);
    }

    /// Musicindex ADR 0053 R-LIST-12 item 1: case does not split a name.
    #[test]
    fn adr_0053_r12_case_sorts_together() {
        let order = sorted_by(&english(), &["Abc", "ABBA", "zed", "abba"]);
        assert_eq!(
            order[..2]
                .iter()
                .filter(|n| n.eq_ignore_ascii_case("abba"))
                .count(),
            2
        );
        assert_eq!(order[2], "Abc");
    }

    /// Musicindex ADR 0053 R-LIST-12 item 1: an accent sorts near its base
    /// letter in an English locale.
    #[test]
    fn adr_0053_r12_accent_sorts_near_its_base_letter() {
        let order = sorted_by(&english(), &["Zulu", "Zëmar Red", "Zebra"]);
        assert_eq!(order, ["Zebra", "Zëmar Red", "Zulu"]);
    }

    /// The text is read as a person reads it: white space does not count.
    #[test]
    fn adr_0053_r12_white_space_is_collapsed() {
        assert_eq!(compare_names("  The   Ohm ", "The Ohm"), Ordering::Equal);
    }
}
