//! The Live list of the Index (ADR 0089 Decision 1, packet 011). It takes
//! the empty state of musicindex ADR 0026 R-EMPTY-1 and the type dot of
//! musicindex ADR 0033 R-DOT.
//!
//! The list shows what is live now, from `GET /v1/live-items` with
//! `view=now`. Each row names its feed. A click opens the release page of
//! that feed. A row with a `content_link` also has a link out. The module
//! has no renderer type, and it reads no data beyond the page it is given.
//! `MusicIndex` responses are untrusted input.

#![warn(clippy::pedantic)]

use crate::api::LiveItem;
use crate::view_models::collection_art::{CollectionArtVm, CollectionKind};
use crate::view_models::index_name_list::{IndexListStatusRowVm, IndexNameList, IndexNameListPage};
use crate::view_models::library::{DestinationListRowTarget, DestinationListRowVm};
use crate::view_models::list_selection::RowSelectionVm;
use crate::view_models::workspace::{FrameNavigationEntry, ReleaseKey};

/// The type word of a live row. The type dot of the row has the live color.
pub(crate) const LIVE_TYPE_WORD: &str = "Live";
/// The empty state of the Live list (musicindex ADR 0026 R-EMPTY-1).
pub(crate) const LIVE_EMPTY_TITLE: &str = "Nothing is live now.";
/// The label of the link out of a live row.
pub(crate) const LIVE_LINK_LABEL: &str = "Open link";
/// The label of a live item with no title.
const UNTITLED_LIVE_ITEM: &str = "Untitled live item";

/// One item of the Live list (ADR 0089 packet 011). A row opens the release
/// page of its feed. A status row is the loading row, the error row or
/// "Load more".
#[derive(Clone, Debug, PartialEq, Eq)]
#[expect(
    clippy::large_enum_variant,
    reason = "the list holds at most one status item; a box for each row adds one allocation for each row"
)]
pub(crate) enum LiveListItemVm {
    /// A live row.
    Row(LiveRowVm),
    /// The loading row, the error row or "Load more".
    Status(IndexListStatusRowVm),
}

/// One live row (ADR 0089 packet 011). `row.label` is the title,
/// `row.secondary_text` is the feed title and `row.open` is the release
/// page of the feed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LiveRowVm {
    /// The row shared with the destination lists.
    pub(crate) row: DestinationListRowVm,
    /// The type word beside the type dot.
    pub(crate) type_word: &'static str,
    /// The scheduled time as text, or an empty text when the item gives none.
    pub(crate) when: String,
    /// The link out, when the item gives a safe web link.
    pub(crate) link: Option<LiveLinkVm>,
}

/// The link out of a live row (ADR 0089 packet 011).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LiveLinkVm {
    /// The web link. It is `http` or `https` only.
    pub(crate) url: String,
    /// The visible label of the button.
    pub(crate) label: &'static str,
    /// The accessibility label of the button.
    pub(crate) a11y_label: String,
}

impl LiveListItemVm {
    /// The items of `rows`, with `status` before or after them (ADR 0084
    /// packet 016 decision D8).
    #[must_use]
    pub(crate) fn of(rows: Vec<LiveRowVm>, status: Option<IndexListStatusRowVm>) -> Vec<Self> {
        let (leading, trailing) = match status {
            Some(row) if row.at_list_start() => (Some(row), None),
            other => (None, other),
        };
        leading
            .map(Self::Status)
            .into_iter()
            .chain(rows.into_iter().map(Self::Row))
            .chain(trailing.map(Self::Status))
            .collect()
    }

    /// The element id of the item.
    #[must_use]
    pub(crate) fn element_id(&self) -> &str {
        match self {
            Self::Row(row) => &row.row.element_id,
            Self::Status(row) => row.element_id(),
        }
    }
}

/// The page of the Live list (ADR 0089 packet 011): its items, with the
/// status rows, and the empty text. `items` holds the rows, the loading row,
/// the error row and "Load more" that `page` gives. `empty` is set only after
/// an answer with no live item (musicindex ADR 0026 R-EMPTY-1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LivePageVm {
    /// The items in order, with the status rows at their places.
    pub(crate) items: Vec<LiveListItemVm>,
    /// The empty text, or `None` before the first answer and while a
    /// request runs.
    pub(crate) empty: Option<&'static str>,
}

impl LivePageVm {
    /// The page of `page`, marking the row that `marked` opens (R-NAV-2 item
    /// 2). A live item with no feed GUID has no row, because its release page
    /// cannot open.
    #[must_use]
    pub(crate) fn of(
        page: &IndexNameListPage<LiveItem>,
        marked: Option<&FrameNavigationEntry>,
    ) -> Self {
        let rows = page
            .rows()
            .iter()
            .filter_map(live_row)
            .map(|mut row| {
                row.row = row.row.with_marked(marked);
                row
            })
            .collect();
        let status = page.status_row(IndexNameList::Live);
        let empty = (!page.is_not_started() && page.rows().is_empty() && status.is_none())
            .then_some(LIVE_EMPTY_TITLE);
        Self {
            items: LiveListItemVm::of(rows, status),
            empty,
        }
    }
}

/// One live row of `item`, or `None` when the item names no feed.
fn live_row(item: &LiveItem) -> Option<LiveRowVm> {
    let feed_guid = non_empty(item.feed_guid.as_deref())?;
    let label = non_empty(item.title.as_deref()).unwrap_or(UNTITLED_LIVE_ITEM);
    let feed_title = non_empty(item.feed_title.as_deref()).unwrap_or_default();
    let item_guid = non_empty(item.live_item_guid.as_deref()).unwrap_or_default();
    let a11y_label = if feed_title.is_empty() {
        format!("{LIVE_TYPE_WORD}, {label}")
    } else {
        format!("{LIVE_TYPE_WORD}, {label}, from {feed_title}")
    };
    let art = CollectionArtVm::resolve(
        non_empty(item.feed_image_url.as_deref()),
        Vec::<&str>::new(),
        CollectionKind::Release,
    );
    let row = DestinationListRowVm {
        element_id: format!("music-live-{feed_guid}-{item_guid}"),
        label: label.to_owned(),
        secondary_text: feed_title.to_owned(),
        a11y_label,
        thumbnail_href: None,
        art: Some(art),
        open: DestinationListRowTarget::Release(ReleaseKey::from_index_feed_guid(feed_guid)),
        selection: RowSelectionVm::none(),
        release_kind: None,
        duration_label: None,
        feed_roles: Vec::new(),
        menu: None,
        download_state: None,
        marked: false,
    };
    let link = live_link_url(item.content_link.as_deref()).map(|url| LiveLinkVm {
        url,
        label: LIVE_LINK_LABEL,
        a11y_label: format!("Open the live link of {label} in the browser"),
    });
    Some(LiveRowVm {
        row,
        type_word: LIVE_TYPE_WORD,
        when: live_schedule_text(item.scheduled_start, item.scheduled_end),
        link,
    })
}

/// The link out of a live item, or `None`. Only a web link with the `http`
/// or `https` scheme passes (ADR 0089 packet 011). The value is untrusted,
/// so a link with another scheme or no scheme gives no button.
#[must_use]
pub(crate) fn live_link_url(content_link: Option<&str>) -> Option<String> {
    let url = non_empty(content_link)?;
    is_web_link(url).then(|| url.to_owned())
}

/// Whether `url` starts with `http://` or `https://`, followed by a host
/// with no blank. The scheme is compared without case, as a URL scheme is.
fn is_web_link(url: &str) -> bool {
    let host = if url
        .get(..7)
        .is_some_and(|scheme| scheme.eq_ignore_ascii_case("http://"))
    {
        url.get(7..)
    } else if url
        .get(..8)
        .is_some_and(|scheme| scheme.eq_ignore_ascii_case("https://"))
    {
        url.get(8..)
    } else {
        None
    };
    host.is_some_and(|host| !host.is_empty() && !host.chars().any(char::is_whitespace))
}

/// The scheduled time of a live item as text, in UTC (ADR 0089 packet 011).
/// A time out of range gives no words. The text is empty when neither time
/// is known.
#[must_use]
pub(crate) fn live_schedule_text(start: Option<i64>, end: Option<i64>) -> String {
    let start = start.and_then(utc_text);
    let end = end.and_then(utc_text);
    match (start, end) {
        (Some(start), Some(end)) => format!("Starts {start}, ends {end}"),
        (Some(start), None) => format!("Starts {start}"),
        (None, Some(end)) => format!("Ends {end}"),
        (None, None) => String::new(),
    }
}

fn utc_text(timestamp: i64) -> Option<String> {
    chrono::DateTime::from_timestamp(timestamp, 0)
        .map(|time| time.format("%b %-d, %H:%M UTC").to_string())
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|text| !text.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_models::index_name_list::IndexNameListBatch;

    fn item(feed: Option<&str>, title: Option<&str>, link: Option<&str>) -> LiveItem {
        LiveItem {
            feed_guid: feed.map(str::to_owned),
            live_item_guid: Some("item-1".to_owned()),
            title: title.map(str::to_owned),
            feed_title: Some("Late Night Feed".to_owned()),
            feed_image_url: None,
            content_link: link.map(str::to_owned),
            scheduled_start: Some(1_710_291_600),
            scheduled_end: None,
        }
    }

    fn loaded(rows: Vec<LiveItem>, has_more: bool) -> IndexNameListPage<LiveItem> {
        let mut page = IndexNameListPage::default();
        let intent = page.begin_load(false).expect("page 1 starts");
        assert!(page.finish_load(
            IndexNameListBatch {
                rows,
                cursor: Some("next".to_owned()),
                has_more,
            },
            intent.generation,
        ));
        page
    }

    /// ADR 0089 packet 011: a live row shows its title, its feed title, the
    /// type word, the scheduled time and the release page of its feed.
    #[test]
    fn adr_0089_live_row_shows_title_feed_word_time_and_release_target() {
        let page = loaded(
            vec![item(
                Some("feed-guid"),
                Some("Tonight's Listening Party"),
                None,
            )],
            false,
        );
        let items = LivePageVm::of(&page, None).items;
        let [LiveListItemVm::Row(row)] = items.as_slice() else {
            panic!("one live row");
        };
        assert_eq!(row.row.label, "Tonight's Listening Party");
        assert_eq!(row.row.secondary_text, "Late Night Feed");
        assert_eq!(row.type_word, "Live");
        assert_eq!(row.when, "Starts Mar 13, 01:00 UTC");
        assert_eq!(
            row.row.open,
            DestinationListRowTarget::Release(ReleaseKey::from_index_feed_guid("feed-guid"))
        );
        assert!(row.link.is_none(), "a null content link gives no button");
    }

    /// ADR 0089 packet 011: a live item with no feed GUID has no row, and
    /// a row with no title takes the untitled label.
    #[test]
    fn adr_0089_live_row_needs_a_feed_guid_and_has_an_untitled_label() {
        let page = loaded(
            vec![
                item(None, Some("No feed"), None),
                item(Some("feed"), None, None),
            ],
            false,
        );
        let items = LivePageVm::of(&page, None).items;
        assert_eq!(items.len(), 1, "the row with no feed GUID is dropped");
        let LiveListItemVm::Row(row) = &items[0] else {
            panic!("a row");
        };
        assert_eq!(row.row.label, UNTITLED_LIVE_ITEM);
    }

    /// ADR 0089 packet 011: the link out has the `http` or `https` scheme
    /// only. Another scheme, a relative value and a blank value give none.
    #[test]
    fn adr_0089_live_link_accepts_only_http_and_https() {
        assert_eq!(
            live_link_url(Some("https://example.test/stream")).as_deref(),
            Some("https://example.test/stream")
        );
        assert_eq!(
            live_link_url(Some("http://example.test/stream")).as_deref(),
            Some("http://example.test/stream")
        );
        for value in [
            "javascript:alert(1)",
            "file:///etc/passwd",
            "/relative/path",
            "example.test/no-scheme",
            "https://",
            "https://exa mple.test",
            "   ",
        ] {
            assert_eq!(live_link_url(Some(value)), None, "{value}");
        }
        assert_eq!(live_link_url(None), None);
    }

    /// ADR 0089 packet 011: a row with a safe content link gets its button,
    /// with the label and the accessibility label.
    #[test]
    fn adr_0089_live_row_link_has_its_label() {
        let page = loaded(
            vec![item(
                Some("feed"),
                Some("Show"),
                Some("https://example.test/stream"),
            )],
            false,
        );
        let items = LivePageVm::of(&page, None).items;
        let LiveListItemVm::Row(row) = &items[0] else {
            panic!("a row");
        };
        let link = row.link.as_ref().expect("the link out shows");
        assert_eq!(link.url, "https://example.test/stream");
        assert_eq!(link.label, "Open link");
        assert_eq!(link.a11y_label, "Open the live link of Show in the browser");
    }

    /// ADR 0089 packet 011: the empty text shows only after an answer with
    /// no live item, and "Load more" comes from the pagination of the page.
    #[test]
    fn adr_0089_live_empty_state_and_load_more_come_from_the_page() {
        let untouched: IndexNameListPage<LiveItem> = IndexNameListPage::default();
        assert_eq!(LivePageVm::of(&untouched, None).empty, None);

        let empty = loaded(Vec::new(), false);
        assert_eq!(
            LivePageVm::of(&empty, None).empty,
            Some("Nothing is live now.")
        );
        assert!(LivePageVm::of(&empty, None).items.is_empty());

        let more = loaded(vec![item(Some("feed"), Some("A"), None)], true);
        assert_eq!(LivePageVm::of(&more, None).empty, None);
        let items = LivePageVm::of(&more, None).items;
        let LiveListItemVm::Status(IndexListStatusRowVm::LoadMore(action)) =
            items.last().expect("the load more row")
        else {
            panic!("the last item is Load more");
        };
        assert!(action.available);
        assert_eq!(action.label, "Load more");
    }

    /// ADR 0089 packet 011: a failed first page shows its error row first,
    /// and the row that the open page marks is the one of its feed.
    #[test]
    fn adr_0089_live_error_row_leads_and_the_open_release_row_is_marked() {
        let mut page: IndexNameListPage<LiveItem> = IndexNameListPage::default();
        let intent = page.begin_load(false).expect("page 1 starts");
        assert!(page.fail_load("timeout", intent.generation));
        let items = LivePageVm::of(&page, None).items;
        assert!(matches!(
            items.as_slice(),
            [LiveListItemVm::Status(IndexListStatusRowVm::Failed { .. })]
        ));
        assert_eq!(
            LivePageVm::of(&page, None).empty,
            None,
            "an error is not empty"
        );

        let page = loaded(vec![item(Some("feed"), Some("A"), None)], false);
        let open = FrameNavigationEntry::Release(ReleaseKey::from_index_feed_guid("feed"));
        let items = LivePageVm::of(&page, Some(&open)).items;
        let LiveListItemVm::Row(row) = &items[0] else {
            panic!("a row");
        };
        assert!(row.row.marked);
    }

    /// ADR 0089 packet 011: the schedule text names each known time.
    #[test]
    fn adr_0089_live_schedule_text_names_each_known_time() {
        assert_eq!(live_schedule_text(None, None), "");
        assert_eq!(live_schedule_text(Some(0), None), "Starts Jan 1, 00:00 UTC");
        assert_eq!(live_schedule_text(None, Some(0)), "Ends Jan 1, 00:00 UTC");
        assert_eq!(
            live_schedule_text(Some(0), Some(3600)),
            "Starts Jan 1, 00:00 UTC, ends Jan 1, 01:00 UTC"
        );
    }
}
