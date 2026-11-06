//! The `MusicIndex` name list pages of the Releases and Artists lists
//! (ADR 0084 Decision 8, packet 016; ADR 0089 Decision 5).
//!
//! The scope "All" adds the `MusicIndex` items of the name lists of
//! Stophammer ADR 0070, in one name order. The scope control is in
//! `list_page`. This module gives the sort key, the page state of one name
//! list, the watermark and the status row. It has no renderer type.

#![warn(clippy::pedantic)]

use crate::view_models::recent_feeds::{RecentFeedsPageState, RecentFeedsPageVm};
use crate::view_models::releases_everything::Revision;

/// The sort key of a name (Stophammer ADR 0070 §1, ADR 0084 Decision 7).
/// It removes the white space at the two ends, makes each internal run of
/// white space one space, and makes each character Unicode lowercase. It
/// keeps an article such as "The" and each punctuation mark. Rust `String`
/// order is code point order, as the Stophammer order is.
#[must_use]
pub(crate) fn name_sort_key(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// The watermark of packet 016 decision D7. While `MusicIndex` has more
/// pages, a row shows only when its order position is at or before the
/// last loaded `MusicIndex` row. Thus a row never moves after it shows.
/// When `MusicIndex` has no more pages, each row shows.
#[must_use]
pub(crate) fn watermark_shows<K: Ord>(key: &K, last_loaded: Option<&K>, has_more: bool) -> bool {
    !has_more || last_loaded.is_some_and(|last| key <= last)
}

/// One page of `MusicIndex` name list rows and its cursor.
#[derive(Clone, Debug)]
pub(crate) struct IndexNameListBatch<Row> {
    /// The rows of this page, in sort key order.
    pub(crate) rows: Vec<Row>,
    /// The cursor of the next page, when there is one.
    pub(crate) cursor: Option<String>,
    /// Whether `MusicIndex` has more pages.
    pub(crate) has_more: bool,
}

/// The request that [`IndexNameListPage::begin_load`] asks for.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IndexNameListLoadIntent {
    /// The cursor to send. `None` asks for page 1.
    pub(crate) cursor: Option<String>,
    /// The request generation. A result of an older generation changes
    /// nothing (packet 016 decision D10).
    pub(crate) generation: u64,
    /// The `from` text of the request, or `None` for the start of the list
    /// (ADR 0089 Decision 6, the letter bar).
    pub(crate) from: Option<String>,
}

/// The load state of one name list.
#[derive(Clone, Debug, Eq, PartialEq)]
enum IndexNameListLoadState {
    /// No request was sent.
    NotStarted,
    /// A request is in flight. `append` is false for page 1.
    Loading { append: bool },
    /// The last request gave its page.
    Loaded,
    /// The last request failed. `append` is false for page 1. `request`
    /// names the failed request when it is not the name list request, for
    /// example a feed request of the page (ADR 0084 packet 026).
    Failed {
        append: bool,
        detail: String,
        request: Option<String>,
    },
}

/// The loaded pages of one `MusicIndex` name list (ADR 0084 packet 016).
/// The pages stay until the app stops. No page expires. "Try again" after
/// an error on page 1 is the only new load of page 1 (decision D9).
#[derive(Clone, Debug)]
pub(crate) struct IndexNameListPage<Row> {
    rows: Vec<Row>,
    cursor: Option<String>,
    has_more: bool,
    state: IndexNameListLoadState,
    generation: u64,
    /// The `from` text of the list (ADR 0089 Decision 6). `None` starts the
    /// list at its first row.
    from: Option<String>,
    /// The rows of a page whose name list request had an answer while
    /// another request of the page failed (ADR 0084 packet 026 decision
    /// D10). They show only after "Try again" gets each answer.
    pending: Option<IndexNameListBatch<Row>>,
    /// Each change of the page gives a new revision (ADR 0084 task 037).
    revision: Revision,
}

impl<Row> Default for IndexNameListPage<Row> {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            cursor: None,
            has_more: false,
            state: IndexNameListLoadState::NotStarted,
            generation: 0,
            from: None,
            pending: None,
            revision: Revision::default(),
        }
    }
}

impl<Row> IndexNameListPage<Row> {
    /// The revision of the page (ADR 0084 task 037, Design step 2). Each
    /// method that changes the page gives it a new revision.
    #[must_use]
    pub(crate) const fn revision(&self) -> Revision {
        self.revision
    }

    /// The loaded rows, in sort key order.
    #[must_use]
    pub(crate) fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// Whether `MusicIndex` has more pages.
    #[must_use]
    pub(crate) const fn has_more(&self) -> bool {
        self.has_more
    }

    /// The `from` text of the list, or `None` for the start of the list.
    #[must_use]
    pub(crate) fn from(&self) -> Option<&str> {
        self.from.as_deref()
    }

    /// Starts the list again at `from` (ADR 0089 Decision 6, the letter
    /// bar). The loaded rows go, and page 1 loads from `from`. `None`
    /// starts the list at its first row. The result is the request of page 1.
    #[must_use]
    pub(crate) fn begin_from(&mut self, from: Option<String>) -> Option<IndexNameListLoadIntent> {
        self.from = from;
        self.begin_load(false)
    }

    /// Whether no request was sent.
    #[must_use]
    pub(crate) const fn is_not_started(&self) -> bool {
        matches!(self.state, IndexNameListLoadState::NotStarted)
    }

    /// Whether the request of page 1 is in flight. The list then shows no
    /// item row (decision D8).
    #[must_use]
    #[cfg(test)]
    pub(crate) const fn first_page_in_flight(&self) -> bool {
        matches!(
            self.state,
            IndexNameListLoadState::Loading { append: false }
        )
    }

    /// Begins a load of page 1 (`append` false) or of the next page
    /// (`append` true). A load of page 1 removes the loaded rows and adds 1
    /// to the generation. An append gives `None` while a request is in
    /// flight, or when `MusicIndex` has no more pages.
    #[must_use]
    pub(crate) fn begin_load(&mut self, append: bool) -> Option<IndexNameListLoadIntent> {
        if append {
            if matches!(self.state, IndexNameListLoadState::Loading { .. }) || !self.has_more {
                return None;
            }
            self.pending = None;
            self.state = IndexNameListLoadState::Loading { append: true };
            self.revision.bump();
            return Some(IndexNameListLoadIntent {
                cursor: self.cursor.clone(),
                generation: self.generation,
                from: self.from.clone(),
            });
        }
        self.generation += 1;
        self.pending = None;
        self.rows.clear();
        self.cursor = None;
        self.has_more = false;
        self.state = IndexNameListLoadState::Loading { append: false };
        self.revision.bump();
        Some(IndexNameListLoadIntent {
            cursor: None,
            generation: self.generation,
            from: self.from.clone(),
        })
    }

    /// Begins the "Try again" of a page whose rows wait (ADR 0084 packet 026
    /// decision D10). Gives the waiting rows and the generation, and starts the load state again. The caller sends only the failed
    /// requests of the page. Gives `None` when no rows wait.
    #[must_use]
    pub(crate) fn begin_pending_retry(&mut self) -> Option<(IndexNameListBatch<Row>, u64)> {
        let IndexNameListLoadState::Failed { append, .. } = self.state else {
            return None;
        };
        let pending = self.pending.take()?;
        self.state = IndexNameListLoadState::Loading { append };
        self.revision.bump();
        Some((pending, self.generation))
    }

    /// Begins the load that "Try again" asks for: page 1 after an error on
    /// page 1, or the same next page after an error on a later page. Gives
    /// `None` when the last request did not fail.
    #[must_use]
    pub(crate) fn begin_retry(&mut self) -> Option<IndexNameListLoadIntent> {
        let IndexNameListLoadState::Failed { append, .. } = self.state else {
            return None;
        };
        self.begin_load(append)
    }

    /// True when `generation` is the generation of the request in flight.
    #[must_use]
    pub(crate) const fn is_current(&self, generation: u64) -> bool {
        generation == self.generation
    }

    /// Adds the rows of `batch`. Gives `false` and changes nothing when
    /// `generation` is old.
    pub(crate) fn finish_load(&mut self, batch: IndexNameListBatch<Row>, generation: u64) -> bool {
        if generation != self.generation {
            return false;
        }
        self.rows.extend(batch.rows);
        self.cursor = batch.cursor;
        self.has_more = batch.has_more;
        self.state = IndexNameListLoadState::Loaded;
        self.revision.bump();
        true
    }

    /// Records a failed request. The loaded rows stay. Gives `false` and
    /// changes nothing when `generation` is old.
    pub(crate) fn fail_load(&mut self, detail: impl Into<String>, generation: u64) -> bool {
        if generation != self.generation {
            return false;
        }
        let append = matches!(self.state, IndexNameListLoadState::Loading { append: true });
        self.state = IndexNameListLoadState::Failed {
            append,
            detail: detail.into(),
            request: None,
        };
        self.revision.bump();
        true
    }

    /// Records a page whose name list request had an answer while another
    /// request of the page failed (ADR 0084 packet 026 decision D10). The
    /// rows of `batch` wait, and the error row names `request`. Gives
    /// `false` and changes nothing when `generation` is old.
    pub(crate) fn fail_pending(
        &mut self,
        batch: IndexNameListBatch<Row>,
        request: String,
        detail: impl Into<String>,
        generation: u64,
    ) -> bool {
        if generation != self.generation {
            return false;
        }
        let append = matches!(self.state, IndexNameListLoadState::Loading { append: true });
        self.state = IndexNameListLoadState::Failed {
            append,
            detail: detail.into(),
            request: Some(request),
        };
        self.pending = Some(batch);
        self.revision.bump();
        true
    }

    /// The status row at the end of the list, when it has one (decision
    /// D8): the loading row of page 1, the error row with "Try again", or
    /// "Load more".
    #[must_use]
    pub(crate) fn status_row(&self, list: IndexNameList) -> Option<IndexListStatusRowVm> {
        match &self.state {
            IndexNameListLoadState::NotStarted => None,
            IndexNameListLoadState::Loading { append: false } => {
                Some(IndexListStatusRowVm::Loading {
                    element_id: list.element_id("loading"),
                    label: list.loading_label(),
                })
            }
            IndexNameListLoadState::Loading { append: true } => Some(list.load_more_row(false)),
            IndexNameListLoadState::Loaded => self.has_more.then(|| list.load_more_row(true)),
            IndexNameListLoadState::Failed {
                append,
                detail,
                request,
            } => Some(IndexListStatusRowVm::Failed {
                element_id: list.element_id("error"),
                first_page: !*append,
                message: list.failure_message(*append),
                detail: format!("{}: {detail}", request.as_deref().unwrap_or(list.request())),
                retry: IndexListActionDisplay {
                    id: list.element_id("retry"),
                    label: "Try again",
                    a11y_label: list.retry_a11y_label(),
                    available: true,
                },
            }),
        }
    }
}

/// The `MusicIndex` name list of a destination list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IndexNameList {
    /// `GET /v1/feeds/by-name`, for the Releases list.
    Releases,
    /// `GET /v1/artist-names`, for the Artists list.
    Artists,
    /// `GET /v1/feeds/recent`, for the Releases list in the sort Newest
    /// (ADR 0089 Decision 5).
    RecentReleases,
}

impl IndexNameList {
    const fn noun(self) -> &'static str {
        match self {
            Self::Releases | Self::RecentReleases => "releases",
            Self::Artists => "artists",
        }
    }

    const fn singular(self) -> &'static str {
        match self {
            Self::Releases | Self::RecentReleases => "release",
            Self::Artists => "artist",
        }
    }

    /// The request that the error report names.
    const fn request(self) -> &'static str {
        match self {
            Self::Releases => "GET /v1/feeds/by-name",
            Self::RecentReleases => "GET /v1/feeds/recent",
            Self::Artists => "GET /v1/artist-names",
        }
    }

    fn element_id(self, part: &str) -> String {
        match self {
            Self::RecentReleases => format!("music-index-recent-{part}"),
            Self::Releases | Self::Artists => format!("music-index-{}-{part}", self.noun()),
        }
    }

    const fn loading_label(self) -> &'static str {
        match self {
            Self::Releases => "Loading MusicIndex releases",
            Self::RecentReleases => "Loading recent MusicIndex releases",
            Self::Artists => "Loading MusicIndex artists",
        }
    }

    fn failure_message(self, append: bool) -> String {
        let (noun, singular) = (self.noun(), self.singular());
        if append {
            format!(
                "MusicIndex did not answer the request for more {noun}. The list keeps the loaded {noun}."
            )
        } else {
            format!(
                "MusicIndex did not answer the {singular} list request. The list shows Library {noun} only."
            )
        }
    }

    fn retry_a11y_label(self) -> String {
        format!("Send the MusicIndex {} list request again", self.singular())
    }

    fn load_more_row(self, available: bool) -> IndexListStatusRowVm {
        IndexListStatusRowVm::LoadMore(IndexListActionDisplay {
            id: self.element_id("load-more"),
            label: "Load more",
            a11y_label: format!("Load more MusicIndex {}", self.noun()),
            available,
        })
    }
}

/// The status row of the Releases list in the sort Newest (ADR 0089
/// Decision 5). It reads the recent pager: the loading row of page 1, the
/// error row of page 1, or "Load more" when the Index has more pages.
#[must_use]
pub(crate) fn recent_releases_status_row(page: &RecentFeedsPageVm) -> Option<IndexListStatusRowVm> {
    let list = IndexNameList::RecentReleases;
    match page.state() {
        RecentFeedsPageState::Loading => page.is_loading().then(|| IndexListStatusRowVm::Loading {
            element_id: list.element_id("loading"),
            label: list.loading_label(),
        }),
        RecentFeedsPageState::Error { message, detail } => Some(IndexListStatusRowVm::Failed {
            element_id: list.element_id("error"),
            first_page: true,
            message: list.failure_message(false),
            detail: format!("{message}. {}: {detail}", list.request()),
            retry: IndexListActionDisplay {
                id: list.element_id("retry"),
                label: "Try again",
                a11y_label: list.retry_a11y_label(),
                available: true,
            },
        }),
        RecentFeedsPageState::Loaded(_) => {
            if page.is_loading() {
                Some(list.load_more_row(false))
            } else if let Some(detail) = page.older_pages_failure() {
                // ADR 0086 Decision 5: a failed extra page keeps the loaded
                // rows and names the failure. "Try again" asks for it again.
                Some(IndexListStatusRowVm::Failed {
                    element_id: list.element_id("error"),
                    first_page: false,
                    message: list.failure_message(true),
                    detail: format!("{}: {detail}", list.request()),
                    retry: IndexListActionDisplay {
                        id: list.element_id("retry"),
                        label: "Try again",
                        a11y_label: list.retry_a11y_label(),
                        available: true,
                    },
                })
            } else {
                page.has_more().then(|| list.load_more_row(true))
            }
        }
    }
}

/// One action of a status row, with its typed availability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IndexListActionDisplay {
    /// Stable element id.
    pub(crate) id: String,
    /// The visible label.
    pub(crate) label: &'static str,
    /// The accessibility label.
    pub(crate) a11y_label: String,
    /// Whether the action can run now.
    pub(crate) available: bool,
}

/// The status row at the end of a list in the scope "Everything" (packet
/// 016 decision D8).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum IndexListStatusRowVm {
    /// The request of page 1 is in flight.
    Loading {
        element_id: String,
        label: &'static str,
    },
    /// A request failed. `retry` sends it again. `first_page` is true for
    /// a failure of page 1.
    Failed {
        element_id: String,
        first_page: bool,
        message: String,
        detail: String,
        retry: IndexListActionDisplay,
    },
    /// `MusicIndex` has more pages. While a request for the next page is in
    /// flight, the action is not available.
    LoadMore(IndexListActionDisplay),
}

impl IndexListStatusRowVm {
    /// Whether the row shows before the item rows. The loading row and the
    /// error row of page 1 show first, so a long Library list does not
    /// hide them. The other rows show after the last item row.
    #[must_use]
    pub(crate) const fn at_list_start(&self) -> bool {
        match self {
            Self::Loading { .. } => true,
            Self::Failed { first_page, .. } => *first_page,
            Self::LoadMore(_) => false,
        }
    }

    /// The stable element id of the row. "Load more" uses the id of its
    /// button. A virtual list keys the item of the row by this id (ADR
    /// 0084 task 037, Design step 8).
    #[must_use]
    pub(crate) fn element_id(&self) -> &str {
        match self {
            Self::Loading { element_id, .. } | Self::Failed { element_id, .. } => element_id,
            Self::LoadMore(action) => &action.id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn batch(rows: &[&str], cursor: Option<&str>, has_more: bool) -> IndexNameListBatch<String> {
        IndexNameListBatch {
            rows: rows.iter().map(|row| (*row).to_owned()).collect(),
            cursor: cursor.map(str::to_owned),
            has_more,
        }
    }

    /// R16-06: the sort key of Stophammer ADR 0070 §1.
    #[test]
    fn adr_0084_scopes_r16_06_name_sort_key() {
        assert_eq!(name_sort_key("$2 Holla"), "$2 holla");
        assert_eq!(name_sort_key("  The   Ohm "), "the ohm");
        assert_eq!(name_sort_key("The Ohm!"), "the ohm!");
        assert_eq!(name_sort_key("ÉCLAIR\tNoir"), "éclair noir");
    }

    /// R16-07: an append keeps the rows, and `has_more` and the cursor
    /// follow the batch.
    #[test]
    fn adr_0084_scopes_r16_07_append_keeps_rows() {
        let mut page = IndexNameListPage::<String>::default();
        assert!(page.is_not_started());
        let first = page.begin_load(false).unwrap();
        assert_eq!(first.cursor, None);
        assert!(page.first_page_in_flight());
        assert!(page.finish_load(batch(&["a", "b"], Some("c1"), true), first.generation));
        assert!(page.has_more());

        let next = page.begin_load(true).unwrap();
        assert_eq!(next.cursor.as_deref(), Some("c1"));
        assert!(page.begin_load(true).is_none(), "one append at a time");
        assert!(page.finish_load(batch(&["c"], None, false), next.generation));
        assert_eq!(page.rows(), ["a", "b", "c"]);
        assert!(!page.has_more());
        assert!(page.begin_load(true).is_none(), "no more pages");
    }

    /// R16-07: a finish and a failure of an older generation change
    /// nothing.
    #[test]
    fn adr_0084_scopes_r16_07_old_generation_changes_nothing() {
        let mut page = IndexNameListPage::<String>::default();
        let old = page.begin_load(false).unwrap();
        let new = page.begin_load(false).unwrap();
        assert!(new.generation > old.generation);

        assert!(!page.finish_load(batch(&["stale"], Some("x"), true), old.generation));
        assert!(!page.fail_load("stale", old.generation));
        assert!(page.rows().is_empty());
        assert!(page.first_page_in_flight());

        assert!(page.finish_load(batch(&["fresh"], None, false), new.generation));
        assert_eq!(page.rows(), ["fresh"]);
    }

    /// R16-12 and decision D8: the status rows of each load state.
    #[test]
    fn adr_0084_scopes_r16_12_status_rows() {
        let list = IndexNameList::Artists;
        let mut page = IndexNameListPage::<String>::default();
        assert_eq!(page.status_row(list), None);

        let first = page.begin_load(false).unwrap();
        assert_eq!(
            page.status_row(list),
            Some(IndexListStatusRowVm::Loading {
                element_id: "music-index-artists-loading".to_owned(),
                label: "Loading MusicIndex artists",
            })
        );

        assert!(page.fail_load("connection refused", first.generation));
        let status = page.status_row(list);
        assert!(status
            .as_ref()
            .is_some_and(IndexListStatusRowVm::at_list_start));
        let Some(IndexListStatusRowVm::Failed {
            message,
            detail,
            retry,
            ..
        }) = status
        else {
            panic!("a failure on page 1 gives the error row");
        };
        assert_eq!(
            message,
            "MusicIndex did not answer the artist list request. The list shows Library artists only."
        );
        assert_eq!(detail, "GET /v1/artist-names: connection refused");
        assert_eq!(retry.label, "Try again");
        assert!(retry.available);

        let again = page.begin_retry().unwrap();
        assert_eq!(again.cursor, None, "the retry of page 1 asks for page 1");
        assert!(page.finish_load(batch(&["a"], Some("c1"), true), again.generation));
        assert_eq!(
            page.status_row(list),
            Some(IndexListStatusRowVm::LoadMore(IndexListActionDisplay {
                id: "music-index-artists-load-more".to_owned(),
                label: "Load more",
                a11y_label: "Load more MusicIndex artists".to_owned(),
                available: true,
            }))
        );

        let next = page.begin_load(true).unwrap();
        assert!(matches!(
            page.status_row(list),
            Some(IndexListStatusRowVm::LoadMore(IndexListActionDisplay {
                available: false,
                ..
            }))
        ));
        assert!(page.fail_load("timeout", next.generation));
        assert_eq!(page.rows(), ["a"], "the loaded rows stay");
        let status = page.status_row(list);
        assert!(!status
            .as_ref()
            .is_some_and(IndexListStatusRowVm::at_list_start));
        let Some(IndexListStatusRowVm::Failed { message, .. }) = status else {
            panic!("the error row replaces Load more");
        };
        assert!(message.contains("more artists"));
        let retry = page.begin_retry().unwrap();
        assert_eq!(
            retry.cursor.as_deref(),
            Some("c1"),
            "the retry asks for the same page"
        );
    }

    /// R16-09 and decision D7: the watermark.
    #[test]
    fn adr_0084_scopes_r16_09_watermark() {
        assert!(watermark_shows(&"b", Some(&"c"), true));
        assert!(watermark_shows(&"c", Some(&"c"), true));
        assert!(!watermark_shows(&"d", Some(&"c"), true));
        assert!(!watermark_shows(&"a", None, true));
        assert!(watermark_shows(&"z", Some(&"c"), false));
        assert!(watermark_shows(&"z", None, false));
    }

    /// R16-12 for the Releases list: the texts of the loading row, the
    /// error row and "Load more" name the release list.
    #[test]
    fn adr_0084_scopes_r16_12_release_status_rows() {
        let list = IndexNameList::Releases;
        let mut page = IndexNameListPage::<String>::default();
        let first = page.begin_load(false).unwrap();
        assert!(matches!(
            page.status_row(list),
            Some(IndexListStatusRowVm::Loading {
                label: "Loading MusicIndex releases",
                ..
            })
        ));
        assert!(page.fail_load("connection refused", first.generation));
        let Some(IndexListStatusRowVm::Failed {
            message, detail, ..
        }) = page.status_row(list)
        else {
            panic!("a failure on page 1 gives the error row");
        };
        assert_eq!(
            message,
            "MusicIndex did not answer the release list request. The list shows Library releases only."
        );
        assert_eq!(detail, "GET /v1/feeds/by-name: connection refused");
        let again = page.begin_retry().unwrap();
        assert!(page.finish_load(batch(&["a"], Some("c1"), true), again.generation));
        let Some(IndexListStatusRowVm::LoadMore(load_more)) = page.status_row(list) else {
            panic!("more pages give Load more");
        };
        assert_eq!(load_more.a11y_label, "Load more MusicIndex releases");
    }

    /// ADR 0089 Decision 6: a letter starts the list at its `from` text.
    /// The loaded rows go, page 1 carries the text, and a load of the same
    /// list keeps it.
    #[test]
    fn adr_0089_letter_from_restarts_the_list_at_the_letter() {
        let mut page: IndexNameListPage<u8> = IndexNameListPage::default();
        let intent = page
            .begin_from(Some("b".to_owned()))
            .expect("page 1 is requested");
        assert_eq!(intent.cursor, None);
        assert_eq!(intent.from.as_deref(), Some("b"));
        assert_eq!(page.from(), Some("b"));
        assert!(page.finish_load(
            IndexNameListBatch {
                rows: vec![1, 2],
                cursor: None,
                has_more: false,
            },
            intent.generation,
        ));

        let intent = page.begin_from(None).expect("page 1 is requested");
        assert_eq!(intent.from, None, "hash starts the list at its first row");
        assert!(page.rows().is_empty(), "the loaded rows go");

        let again = page.begin_load(false).expect("page 1 is requested");
        assert_eq!(again.from, None, "a plain load keeps the text of the list");
    }
}
