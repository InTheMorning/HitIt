//! Search-results inspector shell.
//!
//! ADR 0084 Decision 2 routes global search results into the Music list
//! column. This shell consumes the GPUI-free search-results view model and
//! renders only the body content: the query, the tab control, the rows and
//! the empty states. Frame chrome and the toolbar remain owned by
//! `frame_shell`.

#![warn(clippy::pedantic)]
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{
    div, AnyElement, App, FontWeight, Image, InteractiveElement, IntoElement, ParentElement,
    ScrollHandle, SharedString, Styled, Window,
};

use crate::ui::composites::list_page_head::{
    list_page_head, ListHeadButtonHandler, ListPageHeadHandlers, ListScopeHandler, ListSortHandler,
};
use crate::ui::composites::music_column_scroll::{
    music_column_scrollbar, music_column_tracked_scrollbar,
};
use crate::ui::composites::page_scroll_content::page_scroll_content;
use crate::ui::composites::{
    CollectionArtImage, EntityKind, Segment, SegmentDisplay, SegmentedControl, TagBadge,
    TagBadgeDisplay, Thumbnail, ThumbnailSize,
};
use crate::ui::control_styles::ControlStyle;
use crate::ui::layouts::OVERLAY_SCROLLBAR_WIDTH;
use crate::ui::primitives::{Button as UiButton, Label, LabelVariant, MultilineText};
use crate::ui::shells::search_result_rows::{
    origin_label, render_pending_result_row, render_result_row, render_window_group_result_row,
    tab_id, SearchResultRowHandlers, SearchResultSelectHandler, SearchResultToggleHandler,
    SearchRowLeading,
};
use crate::ui::tokens::{FontSize, SemanticColor, Size, Spacing};
use crate::view_models::detail_selection::DetailRowKey;
use crate::view_models::list_page::{
    ListPageHeadVm, ListPageKey, ListPageState, ListScope, ListSort,
};
use crate::view_models::search_results::{
    EmptyStateDisplay, IndexDetailDisplay, IndexDetailKind, SearchFailureAction,
    SearchFailureAvailability, SearchPageActionDisplay, SearchResultItemVm, SearchResultOrigin,
    SearchResultsInspectorPageVm, SearchResultsTab, SearchTabCount,
};
use crate::view_models::workspace::FrameNavigationEntry;

type TabSelectHandler = Rc<dyn Fn(SearchResultsTab, &mut Window, &mut App) + 'static>;
type FailureHandler = Rc<dyn Fn(SearchFailureAction, &mut Window, &mut App) + 'static>;
type HeaderActionHandler = ListHeadButtonHandler;

/// Callback slots and the visible rows, supplied by the screen or frame
/// owner. The app layer makes the rows with the Library facts of this
/// render (packet 010).
#[derive(Default)]
#[must_use]
pub(crate) struct SearchResultsInspectorSlots {
    tab_select: Option<TabSelectHandler>,
    scope_select: Option<ListScopeHandler>,
    sort_select: Option<ListSortHandler>,
    result_select: Option<SearchResultSelectHandler>,
    row_toggle: Option<SearchResultToggleHandler>,
    failure_action: Option<FailureHandler>,
    header_actions: Vec<(SearchPageActionDisplay, HeaderActionHandler)>,
    thumbnails: BTreeMap<String, Option<Arc<Image>>>,
    items: Vec<SearchResultItemVm>,
    arts: BTreeMap<String, CollectionArtImage>,
    scroll: ScrollHandle,
    /// The page that Back left open over the search results list (R-NAV-2
    /// item 2). `None` for a page that has no list root.
    marked_entry: Option<FrameNavigationEntry>,
    /// The title row text, when the page gives one in place of its query
    /// (ADR 0089 Decision 4). `None` shows the query.
    header_title: Option<String>,
    /// The status text of the header, when the page gives one in place of
    /// the status of the results. `None` shows the status of the results.
    header_status: Option<String>,
}

impl SearchResultsInspectorSlots {
    /// Creates empty search-results inspector slots.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Gives the title row text in place of the query. The view model gives
    /// the text (ADR 0089 Decision 4).
    pub(crate) fn with_header_title(mut self, title: String) -> Self {
        self.header_title = Some(title);
        self
    }

    /// Gives the status text of the header in place of the status of the
    /// results. The view model gives the text (ADR 0089 Decision 4).
    pub(crate) fn with_header_status(mut self, status: Option<String>) -> Self {
        self.header_status = status;
        self
    }

    /// Supplies disclosure and clipboard wiring for the VM's failure actions.
    pub(crate) fn on_failure_action(
        mut self,
        handler: impl Fn(SearchFailureAction, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.failure_action = Some(Rc::new(handler));
        self
    }

    /// Supplies the tab-selection callback.
    pub(crate) fn on_tab_select(
        mut self,
        handler: impl Fn(SearchResultsTab, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.tab_select = Some(Rc::new(handler));
        self
    }

    /// Supplies the callback of the "All | In Library" control (ADR 0089
    /// Decision 4). The app writes the scope into the query text.
    pub(crate) fn on_scope_select(
        mut self,
        handler: impl Fn(ListScope, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.scope_select = Some(Rc::new(move |scope: &ListScope, window, cx| {
            handler(*scope, window, cx);
        }));
        self
    }

    /// Supplies the callback of the sort menu (ADR 0089 Decision 4). The app
    /// sets the sort of the page. The query does not change.
    pub(crate) fn on_sort_select(
        mut self,
        handler: impl Fn(ListSort, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.sort_select = Some(Rc::new(move |sort: &ListSort, window, cx| {
            handler(*sort, window, cx);
        }));
        self
    }

    /// Supplies the result-row activation callback.
    pub(crate) fn on_result_select(
        mut self,
        handler: impl Fn(
                SearchResultsTab,
                String,
                Option<DetailRowKey>,
                gpui::Modifiers,
                &mut Window,
                &mut App,
            ) + 'static,
    ) -> Self {
        self.result_select = Some(Rc::new(handler));
        self
    }

    /// Supplies the checkbox callback of a row (ADR 0084 Decision 10).
    pub(crate) fn on_row_toggle(
        mut self,
        handler: impl Fn(DetailRowKey, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.row_toggle = Some(Rc::new(handler));
        self
    }

    /// Adds a typed action to the header, for example "Save search" or
    /// "Delete…" (packet 012), with its click handler. The actions show in
    /// the order they are added. The handler runs only when the view model
    /// makes the action available.
    pub(crate) fn header_action(
        mut self,
        display: SearchPageActionDisplay,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.header_actions.push((display, Rc::new(handler)));
        self
    }

    /// Supplies resolved row thumbnails keyed by row thumbnail href.
    pub(crate) fn with_thumbnails(
        mut self,
        thumbnails: BTreeMap<String, Option<Arc<Image>>>,
    ) -> Self {
        self.thumbnails = thumbnails;
        self
    }

    /// Supplies the visible items of the active tab: the rows, and the
    /// group rows of a saved search (ADR 0086 packet 005).
    pub(crate) fn with_items(mut self, items: Vec<SearchResultItemVm>) -> Self {
        self.items = items;
        self
    }

    /// Supplies the loaded logo frame art of each row with art, by row id
    /// (packet 028 D3), and of each group row, by its element id. An item
    /// with art and no entry shows the entity color.
    pub(crate) fn with_arts(mut self, arts: BTreeMap<String, CollectionArtImage>) -> Self {
        self.arts = arts;
        self
    }

    /// Supplies the scroll of the rows (packet 028 D2). The owner keeps the
    /// handle, and scrolls the focused row into view after a key.
    pub(crate) fn with_scroll(mut self, scroll: &ScrollHandle) -> Self {
        self.scroll = scroll.clone();
        self
    }

    /// Supplies the page that Back left open over the results list, for the
    /// marked row (R-NAV-2 item 2).
    pub(crate) fn with_marked_entry(mut self, entry: Option<FrameNavigationEntry>) -> Self {
        self.marked_entry = entry;
        self
    }

    fn thumbnail_for_href(&self, href: &str) -> Option<Arc<Image>> {
        self.thumbnails.get(href).cloned().flatten()
    }
}

/// Renders a search-results inspector shell.
pub(crate) fn render_search_results_inspector(
    vm: &SearchResultsInspectorPageVm,
    slots: &SearchResultsInspectorSlots,
    cx: &App,
) -> AnyElement {
    let tab = vm.tab();
    let header = render_inspector_header(
        InspectorHeader {
            title: slots.header_title.as_deref().unwrap_or_else(|| vm.query()),
            scope: vm.library_scope(),
            list_key: vm.list_key(),
            list_state: vm.list_state(),
            sort_select: slots.sort_select.as_ref(),
            status_text: slots.header_status.as_deref().or_else(|| vm.status_text()),
            selected: tab,
            counts: vm.tab_counts(),
            tab_select: slots.tab_select.as_ref(),
            scope_select: slots.scope_select.as_ref(),
            actions: &slots.header_actions,
        },
        cx,
    );
    let body = if let Some(empty) = vm.empty_state() {
        render_empty_state(empty, slots.failure_action.as_ref(), cx)
    } else {
        render_active_result_list(
            tab,
            vm.index_pending_row_count(),
            vm.index_notice(),
            slots,
            cx,
        )
    };

    div()
        .id(SharedString::from(format!(
            "search-results-inspector-{}",
            tab_id(tab)
        )))
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .min_w_0()
        .overflow_hidden()
        .child(header)
        .child(body)
        .into_any_element()
}

/// Renders a remote Index detail page reached from search-result drill-down.
pub(crate) fn render_index_detail_display(display: &IndexDetailDisplay, cx: &App) -> AnyElement {
    let kind = entity_kind_for_index_detail(display.kind);
    let mut heading = div()
        .flex()
        .flex_col()
        .min_w_0()
        .gap(Spacing::XS.scaled(cx))
        .child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(Spacing::XS.scaled(cx))
                .child(TagBadge::new(TagBadgeDisplay {
                    kind,
                    label: Some(SharedString::from(display.kind.label())),
                }))
                .child(origin_label(SearchResultOrigin::Index)),
        )
        .child(
            Label::new(display.title.clone())
                .variant(LabelVariant::Title)
                .truncated(),
        );
    if !display.secondary_text.is_empty() {
        heading = heading.child(
            Label::new(display.secondary_text.clone())
                .variant(LabelVariant::Caption)
                .truncated(),
        );
    }

    let suffix = detail_id_suffix(display.kind);
    let container = div()
        .id(SharedString::from(format!("index-detail-{suffix}")))
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .min_w_0()
        .p(Spacing::MD.scaled(cx))
        .child(
            page_scroll_content(cx)
                .gap(Spacing::LG.scaled(cx))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_start()
                        .gap(Spacing::MD.scaled(cx))
                        .child(Thumbnail::new(kind, ThumbnailSize::Lg))
                        .child(heading),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(Spacing::XS.scaled(cx))
                        .child(detail_metadata_row("Source", "Index", cx))
                        .child(detail_metadata_row("ID", &display.id, cx)),
                ),
        );
    music_column_scrollbar(
        container,
        SharedString::from(format!("index-detail-{suffix}-scrollbar")),
    )
}

/// The parts of the header of a search page.
#[derive(Clone, Copy)]
struct InspectorHeader<'a> {
    /// The title row text: the query, or the title that the page gives
    /// (ADR 0089 Decision 4).
    title: &'a str,
    /// The "All | In Library" scope of the query.
    scope: ListScope,
    /// The key and the list state of the page: its sort (ADR 0089 Decision
    /// 5). The view icons stay absent on the search pages.
    list_key: ListPageKey,
    list_state: ListPageState,
    sort_select: Option<&'a ListSortHandler>,
    status_text: Option<&'a str>,
    selected: SearchResultsTab,
    /// The count of each tab, in the order of `SearchResultsTab::ALL`.
    counts: [SearchTabCount; 5],
    tab_select: Option<&'a TabSelectHandler>,
    scope_select: Option<&'a ListScopeHandler>,
    actions: &'a [(SearchPageActionDisplay, HeaderActionHandler)],
}

/// The header at the width of the list column (ADR 0089 Decision 4). The
/// title row and the scope row come from the shared `list_page_head`, with
/// the typed action of the page in the title row. The type tabs sit in a row
/// below the scope, then the status text. Column text does not call
/// `truncate()` (ADR 0063).
fn render_inspector_header(header: InspectorHeader<'_>, cx: &App) -> AnyElement {
    let display = ListPageHeadVm::for_search(
        header.list_key,
        header.list_state,
        header.title.to_owned(),
        header.scope,
        header
            .actions
            .iter()
            .map(|(display, _)| display.clone())
            .collect(),
    );
    let handlers = ListPageHeadHandlers {
        scope: header.scope_select.cloned(),
        sort: header.sort_select.cloned(),
        actions: header
            .actions
            .iter()
            .map(|(_, handler)| Rc::clone(handler))
            .collect(),
        ..ListPageHeadHandlers::default()
    };
    let mut column = div()
        .flex()
        .flex_col()
        .gap(Spacing::XS.scaled(cx))
        .child(list_page_head(display, handlers, cx))
        .child(
            div()
                .w_full()
                .min_w_0()
                .px(Spacing::MD.scaled(cx))
                .pb(Spacing::XS.scaled(cx))
                .child(render_tab_strip(
                    header.selected,
                    header.counts,
                    header.tab_select,
                )),
        );
    if let Some(status) = header.status_text {
        column = column.child(
            div()
                .px(Spacing::MD.scaled(cx))
                .pb(Spacing::MD.scaled(cx))
                .child(
                    MultilineText::new(status.to_owned())
                        .max_lines(usize::MAX)
                        .wrap_lines()
                        .size(FontSize::Caption)
                        .color(SemanticColor::SecondaryLabel),
                ),
        );
    }
    column.into_any_element()
}

/// Renders one empty state with no failure action, for example the page
/// of a deleted saved search (packet 012).
pub(crate) fn render_search_empty_state(empty: &EmptyStateDisplay, cx: &App) -> AnyElement {
    render_empty_state(empty, None, cx)
}

/// The type tabs, each with its count (ADR 0089 Decision 4). The counts
/// come from the view model, and no tab changes the query.
fn render_tab_strip(
    selected: SearchResultsTab,
    counts: [SearchTabCount; 5],
    on_tab_select: Option<&TabSelectHandler>,
) -> SegmentedControl<SearchResultsTab> {
    let mut control = SegmentedControl::new(selected).segments(
        counts
            .into_iter()
            .map(|count| Segment::new(tab_segment_display(&count))),
    );

    if let Some(handler) = on_tab_select.cloned() {
        control = control.on_select(move |tab, window, cx| {
            handler(*tab, window, cx);
        });
    }

    control
}

fn tab_segment_display(count: &SearchTabCount) -> SegmentDisplay<SearchResultsTab> {
    let tab = count.tab;
    SegmentDisplay {
        id: SharedString::from(format!("search-results-tab-{}", tab_id(tab))).into(),
        key: tab,
        label: SharedString::from(count.segment_label()),
        a11y_label: SharedString::from(format!(
            "Show {} search results, {} found",
            tab.label(),
            count.text()
        )),
    }
}

/// The heading of one group of the tab All (ADR 0089 Decision 4).
fn render_group_heading(group: SearchResultsTab, cx: &App) -> AnyElement {
    div()
        .id(SharedString::from(format!(
            "search-results-group-{}",
            tab_id(group)
        )))
        .w_full()
        .pt(Spacing::SM.scaled(cx))
        .child(
            Label::new(group.label())
                .size(FontSize::Caption)
                .weight(FontWeight::MEDIUM)
                .color(SemanticColor::SecondaryLabel),
        )
        .into_any_element()
}

fn detail_metadata_row(label: &str, value: &str, cx: &App) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .items_start()
        .gap(Spacing::SM.scaled(cx))
        .child(
            Label::new(label.to_string())
                .size(FontSize::Micro)
                .color(SemanticColor::TertiaryLabel),
        )
        .child(
            div().flex_1().min_w_0().child(
                Label::new(value.to_string())
                    .size(FontSize::Micro)
                    .truncated(),
            ),
        )
        .into_any_element()
}

/// The rows of the active tab, then `pending_rows` loading rows while the
/// `MusicIndex` results load (ADR 0087 packet 007 decision D2). The view
/// model gives the count. `index_notice` is the note of the Index part
/// after the rows, when the view model gives one (walk of 2026-10-21).
fn render_active_result_list(
    tab: SearchResultsTab,
    pending_rows: usize,
    index_notice: Option<&EmptyStateDisplay>,
    slots: &SearchResultsInspectorSlots,
    cx: &App,
) -> AnyElement {
    let tab_slug = tab_id(tab);
    let handlers = SearchResultRowHandlers {
        select: slots.result_select.as_ref(),
        toggle: slots.row_toggle.as_ref(),
    };
    let children: Vec<AnyElement> = slots
        .items
        .iter()
        .map(|item| {
            let row = match item {
                SearchResultItemVm::Row(row) => row,
                SearchResultItemVm::Heading(group) => return render_group_heading(*group, cx),
                SearchResultItemVm::WindowGroup(tile) => {
                    let art = slots
                        .arts
                        .get(&tile.element_id)
                        .cloned()
                        .unwrap_or(CollectionArtImage::EntityColor);
                    return render_window_group_result_row(
                        SearchResultsTab::Releases,
                        tile,
                        art,
                        handlers,
                        cx,
                    );
                }
            };
            let leading = if let Some(art) = &row.art {
                SearchRowLeading::Collection {
                    image: slots
                        .arts
                        .get(&row.id)
                        .cloned()
                        .unwrap_or(CollectionArtImage::EntityColor),
                    kind: art.kind,
                }
            } else {
                let href = row.thumbnail_href.as_deref();
                SearchRowLeading::Thumbnail(href.and_then(|href| slots.thumbnail_for_href(href)))
            };
            let row = row
                .clone()
                .with_marked(row.tab, slots.marked_entry.as_ref());
            render_result_row(row, leading, handlers, cx)
        })
        .chain((0..pending_rows).map(|index| render_pending_result_row(tab, index)))
        .collect();
    let notice = index_notice.map(|notice| {
        // The note is a child of the scroll container with the rows, so it
        // must not shrink under them (packet 028 review).
        div()
            .flex_shrink_0()
            .pt(Spacing::MD.scaled(cx))
            .child(empty_state_content(
                notice,
                LabelVariant::Caption,
                slots.failure_action.as_ref(),
                cx,
            ))
    });
    // Packet 028 D2: each row is a direct child of the scroll container, in
    // the order of the view model rows. Thus `scroll_to_item` finds the
    // focused row by its index. The loading rows and the note are after
    // the rows. The right
    // padding keeps the clearance of `page_scroll_content` and adds the
    // column inset.
    let container = page_scroll_content(cx)
        .id(SharedString::from(format!(
            "search-results-{tab_slug}-rows"
        )))
        .gap(Spacing::XXS.scaled(cx))
        .pl(Spacing::MD.scaled(cx))
        .pr(OVERLAY_SCROLLBAR_WIDTH + Spacing::SM.scaled(cx) + Spacing::MD.scaled(cx))
        .pb(Spacing::MD.scaled(cx))
        .children(children)
        .children(notice);
    music_column_tracked_scrollbar(container, &slots.scroll, None)
}

fn render_empty_state(
    empty: &EmptyStateDisplay,
    on_failure: Option<&FailureHandler>,
    cx: &App,
) -> AnyElement {
    let content = empty_state_content(empty, LabelVariant::Headline, on_failure, cx);
    let container = div()
        .id("search-results-notice")
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .min_w_0()
        .items_center()
        .p(Spacing::MD.scaled(cx))
        .child(
            page_scroll_content(cx)
                .w_full()
                .items_center()
                .child(content),
        );
    music_column_scrollbar(container, "search-results-notice-scrollbar")
}

/// The title, the text, the failure actions and the open report of one
/// notice. The empty state and the note of the Index part share it.
fn empty_state_content(
    empty: &EmptyStateDisplay,
    title_variant: LabelVariant,
    on_failure: Option<&FailureHandler>,
    cx: &App,
) -> gpui::Div {
    let mut content = div()
        .w_full()
        .max_w(Size::NoticeWidth.scaled(cx))
        .min_w_0()
        .flex()
        .flex_col()
        .flex_shrink_0()
        .gap(Spacing::MD.scaled(cx))
        .child(Label::new(empty.title.clone()).variant(title_variant))
        .child(
            MultilineText::new(empty.secondary.clone())
                .max_lines(usize::MAX)
                .wrap_lines()
                .size(FontSize::Body)
                .color(SemanticColor::SecondaryLabel),
        );

    if let Some(failure) = &empty.failure {
        if let Some(handler) = on_failure {
            let mut actions = div().flex().flex_wrap().gap(Spacing::SM.scaled(cx));
            for intent in [
                SearchFailureAction::ToggleDetails,
                SearchFailureAction::CopyReport,
            ] {
                let display = failure.action(intent);
                let handler = handler.clone();
                actions = actions.child(
                    UiButton::styled(
                        SharedString::from(format!("search-failure-{intent:?}")),
                        ControlStyle::Neutral,
                    )
                    .label(display.label)
                    .a11y_label(display.a11y_label)
                    .disabled(display.availability != SearchFailureAvailability::Available)
                    .on_activate(move |window, cx| handler(display.action, window, cx)),
                );
            }
            content = content.child(actions);
        }
        if let Some(report) = failure.visible_report() {
            content = content.child(
                MultilineText::new(report.to_owned())
                    .max_lines(usize::MAX)
                    .wrap_lines()
                    .size(FontSize::Caption)
                    .color(SemanticColor::SecondaryLabel),
            );
        }
    }
    content
}

const fn entity_kind_for_index_detail(kind: IndexDetailKind) -> EntityKind {
    match kind {
        IndexDetailKind::Release => EntityKind::Release,
        IndexDetailKind::Track => EntityKind::Track,
    }
}

/// The element id suffix of a detail page. The id of a release page keeps
/// the suffix "feed", so that each element id stays the same.
const fn detail_id_suffix(kind: IndexDetailKind) -> &'static str {
    match kind {
        IndexDetailKind::Release => "feed",
        IndexDetailKind::Track => "track",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// F6 of the walk of 2026-10-20 (ADR 0084 Decision 5): the loading and
    /// failed header of an Index release page shows the tag "release".
    #[test]
    fn adr_0084_walk_fix_index_release_header_kind_label_is_release() {
        assert_eq!(IndexDetailKind::Release.label(), "release");
        assert_eq!(
            entity_kind_for_index_detail(IndexDetailKind::Release).label(),
            "Release"
        );
    }

    #[test]
    fn slots_accept_callbacks() {
        let slots = SearchResultsInspectorSlots::new()
            .on_tab_select(|_, _, _| {})
            .on_result_select(|_, _, _, _, _, _| {})
            .on_row_toggle(|_, _, _| {});

        assert!(slots.tab_select.is_some());
        assert!(slots.result_select.is_some());
        assert!(slots.row_toggle.is_some());
    }
}
