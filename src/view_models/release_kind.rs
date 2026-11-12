//! The kind of a release (ADR 0084 Decision 5, ADR 0091 Decision 1).
//!
//! The kind rule is the crate `musicindex-search`, module `kind` (musicindex
//! ADR 0020 R-WORD-1). This module re-exports it, so the uses in the app do not
//! change. The action labels stay in their own owners.

pub(crate) use musicindex_search::kind::ReleaseKind;

#[cfg(test)]
use musicindex_search::query::KindValue;

#[cfg(test)]
mod tests {
    use super::*;

    /// R7-01: a stated value wins over the track count, after trim and
    /// read without case. An unstated value falls back to the count rule,
    /// and a track count never gives an EP.
    #[test]
    fn adr_0084_release_kind_r7_01_resolve_follows_decision_5() {
        assert_eq!(ReleaseKind::resolve(Some("Album"), 1), ReleaseKind::Album);
        assert_eq!(ReleaseKind::resolve(Some("ep"), 1), ReleaseKind::Ep);
        assert_eq!(
            ReleaseKind::resolve(Some(" Single "), 5),
            ReleaseKind::Single
        );

        assert_eq!(
            ReleaseKind::resolve(Some("unknown"), 1),
            ReleaseKind::Single
        );
        assert_eq!(ReleaseKind::resolve(Some("unknown"), 5), ReleaseKind::Album);

        assert_eq!(ReleaseKind::resolve(None, 0), ReleaseKind::Album);
        assert_eq!(ReleaseKind::resolve(None, 12), ReleaseKind::Album);
    }

    /// R87-2-06: each `kind:` value gives its release kind.
    #[test]
    fn adr_0087_library_fields_r87_2_06_query_kind_gives_the_release_kind() {
        assert_eq!(
            ReleaseKind::from_query_kind(KindValue::Album),
            ReleaseKind::Album
        );
        assert_eq!(ReleaseKind::from_query_kind(KindValue::Ep), ReleaseKind::Ep);
        assert_eq!(
            ReleaseKind::from_query_kind(KindValue::Single),
            ReleaseKind::Single
        );
    }

    /// R7-01: the visible label and the action noun of each kind.
    #[test]
    fn adr_0084_release_kind_label_and_noun() {
        assert_eq!(ReleaseKind::Album.label(), "Album");
        assert_eq!(ReleaseKind::Ep.label(), "EP");
        assert_eq!(ReleaseKind::Single.label(), "Single");

        assert_eq!(ReleaseKind::Album.noun(), "album");
        assert_eq!(ReleaseKind::Ep.noun(), "EP");
        assert_eq!(ReleaseKind::Single.noun(), "single");
    }
}
