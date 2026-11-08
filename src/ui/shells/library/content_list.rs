//! Library content-list surface.
//!
//! ADR 0062 makes the Music default content region a paged list of recent
//! music. This shell owns only GPUI layout and interaction wiring; row labels,
//! state labels, load-more labels, and empty/error copy come from the view
//! model.
//!
//! ADR 0083 task 007: a Tracks row and an Inbox row show "⋯" on a
//! hover and on the keyboard focus, and a right-click opens the same menu.
//! A Inbox row keeps its one main fix as a visible button.

#![warn(clippy::pedantic)]

use std::collections::BTreeMap;
use std::sync::Arc;

use gpui::{
    div, prelude::FluentBuilder, AnyElement, App, ClickEvent, Context, FontWeight, Image,
    InteractiveElement, IntoElement, ParentElement, ScrollHandle, SharedString, Styled,
};

use crate::library::LibraryApp;
use crate::ui::composites::page_scroll_content::page_scroll_content;
use crate::ui::composites::{
    EntityKind, ListRow, ListRowA11yLabel, SkeletonTrackRow, TagBadge, TagBadgeDisplay, Thumbnail,
    ThumbnailSize,
};
use crate::ui::control_styles::ControlStyle;
use crate::ui::primitives::{Button as UiButton, Checkbox, CheckboxState, Label};
pub(crate) use crate::ui::shells::library::content_list_row_actions::ContentListMenuContext;
use crate::ui::shells::library::content_list_row_actions::{
    content_list_row_menu, render_content_list_row_action,
};
use crate::ui::shells::library::content_list_scroll::attach_content_list_scroll;
use crate::ui::shells::library::content_list_state::render_content_list_state;
use crate::ui::tokens::{FontSize, SemanticColor, Spacing};
use crate::view_models::library::{
    ContentListEntityKind, ContentListLoadMoreDisplay, ContentListLoadingStateDisplay,
    ContentListPageStateDisplay, ContentListPageVm, ContentListRowDisplay,
    ContentListSelectionTarget,
};

/// The images of a content list: the cover of each row by row id.
pub(crate) type ContentListImages<'a> = &'a BTreeMap<String, Option<Arc<Image>>>;

struct ContentListRenderState {
    items: Vec<ContentListRowDisplay>,
    menu: Option<ContentListMenuContext>,
    loading_display: Option<ContentListLoadingStateDisplay>,
    state_display: Option<ContentListPageStateDisplay>,
    load_more_display: Option<ContentListLoadMoreDisplay>,
    routing: (ContentListSelectionTarget, Option<AnyElement>),
}

/// Renders the Music content-list page. `items` is the page's visible
/// rows and group tiles, with the Decision 10 checkbox facts of the
/// selection that `routing` names. The focused item carries the scroll
/// probe of `routing`. `thumbnails` gives the cover of each row and the
/// art of each group. `menu` gives each row a menu cell. `None` gives no
/// menu cell.
pub(crate) fn render_library_content_list(
    page: &ContentListPageVm,
    items: Vec<ContentListRowDisplay>,
    thumbnails: ContentListImages<'_>,
    scroll_handle: &ScrollHandle,
    routing: (ContentListSelectionTarget, Option<AnyElement>),
    menu: Option<ContentListMenuContext>,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let render_state = ContentListRenderState {
        items,
        menu,
        routing,
        loading_display: page.loading_display(),
        state_display: page.page_state_display(),
        load_more_display: page.load_more_display(),
    };
    let body = render_content_list_rows(page, render_state, thumbnails, scroll_handle, cx);

    div()
        .id(ContentListPageVm::page_id())
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .min_w_0()
        .overflow_hidden()
        .child(body)
        .into_any_element()
}

fn render_content_list_rows(
    page: &ContentListPageVm,
    render_state: ContentListRenderState,
    thumbnails: ContentListImages<'_>,
    scroll_handle: &ScrollHandle,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let pending_rows = render_state
        .loading_display
        .as_ref()
        .map_or(0, |display| display.skeleton_count);
    let (target, mut probe) = render_state.routing;

    // ADR 0084 Decision 10: `track_scroll` (inside `attach_content_list_scroll`)
    // reads the bounds of its direct children to find a row by index, so
    // this div's rows are its direct children. An outer wrapper here
    // would give the tracked handle one child — the wrapper — and no
    // way to scroll to one row.
    let mut content = page_scroll_content(cx)
        .id(ContentListPageVm::rows_id())
        .gap(Spacing::XXS.scaled(cx))
        .px(Spacing::MD.scaled(cx))
        .pb(Spacing::MD.scaled(cx))
        .children(render_state.items.into_iter().map(|row| {
            let thumbnail = thumbnails.get(&row.id).cloned().flatten();
            let probe = probe.take_if(|_| row.selection.focused);
            render_content_list_row(
                &row,
                thumbnail,
                (target, probe),
                render_state.menu.as_ref(),
                cx,
            )
        }))
        .children((0..pending_rows).map(|index| {
            SkeletonTrackRow::new(ContentListPageVm::skeleton_row_id(index))
                .show_duration(false)
                .into_any_element()
        }));
    if let Some(display) = render_state.state_display {
        content = content.child(render_content_list_state(display, cx));
    }
    if let Some(display) = render_state.load_more_display {
        content = content.child(render_content_list_load_more(&display, cx));
    }

    attach_content_list_scroll(
        content,
        page.has_more(),
        page.is_loading(),
        scroll_handle,
        cx,
    )
}

fn render_content_list_row(
    row: &ContentListRowDisplay,
    thumbnail: Option<Arc<Image>>,
    (target, probe): (ContentListSelectionTarget, Option<AnyElement>),
    menu_context: Option<&ContentListMenuContext>,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let row_id = row.id.clone();
    let entity_kind = entity_kind_for_content(row.entity_kind());
    let mut list_row = ListRow::new(SharedString::from(row.element_id()))
        .a11y_label(ListRowA11yLabel {
            label: SharedString::from(row.a11y_label().to_string()),
        })
        .selected(row.selection.is_marked())
        .focused(row.selection.focused)
        .marked(row.marked)
        .scroll_probe(probe);
    if let Some(checkbox) = render_content_row_checkbox(row, target, cx) {
        list_row = list_row.checkbox(checkbox);
    }

    list_row = list_row
        .child(Thumbnail::new(entity_kind, ThumbnailSize::Sm).image(thumbnail))
        .child(render_content_list_row_text(row, entity_kind, cx));

    // ADR 0083 task 007, decision D10: the one main fix of a Needs
    // attention row shows at all times, before the menu cell.
    if let Some(fix) = &row.fix {
        list_row = list_row.child(render_content_list_row_action(fix, cx));
    }

    // ADR 0083 task 007, decisions D9 and D10: the menu cell and the
    // right-click menu exist only when the view model gives a menu.
    if let Some(context) = menu_context {
        list_row = list_row.menu_cell(content_list_row_menu(row, context, cx));
    }

    // ADR 0083 task 008, decision D4: a Tracks list row of a Library
    // track in `Normal` starts a drag of its track.
    list_row
        .drag(row.drag.clone())
        .when(row.accepts_row_click(), |el| {
            el.on_click(cx.listener(move |this, event: &ClickEvent, _window, cx| {
                this.click_content_row(target, &row_id, event.modifiers(), cx);
            }))
        })
        .into_any_element()
}

/// The checkbox of a row or a tile, or `None` outside selection mode (ADR
/// 0084 Decision 10 Amendment, 2026-10-19). A row with no selection model,
/// for example a Discover row with no feed GUID, gets `None` in each mode.
/// The click changes the selection that `target` names, and never opens
/// the page of the row.
fn render_content_row_checkbox(
    row: &ContentListRowDisplay,
    target: ContentListSelectionTarget,
    cx: &mut Context<LibraryApp>,
) -> Option<AnyElement> {
    if !row.selection.checkbox_visible {
        return None;
    }
    let row_id = row.id.clone();
    let checkbox = Checkbox::new(
        SharedString::from(format!("{}-checkbox", row.element_id())),
        CheckboxState::from_checked(row.selection.check_state.is_checked()),
    )
    .a11y_label(row.selection.checkbox_a11y_label.clone())
    .on_click(cx.listener(move |this, _: &ClickEvent, _window, cx| {
        // Stop the click from also reaching the row's own handler.
        cx.stop_propagation();
        this.toggle_content_row(target, &row_id, cx);
    }));
    Some(checkbox.into_any_element())
}

/// The text of a content-list row: the title, the secondary text, and a
/// status line. ADR 0084 Decision 1: the list column is narrow, so the
/// Library state, the "New" mark and the entity badge sit on the status
/// line under the text, not at the right of the row. The title then has the
/// full text width.
fn render_content_list_row_text(
    row: &ContentListRowDisplay,
    entity_kind: EntityKind,
    cx: &App,
) -> AnyElement {
    let mut text = div().flex_1().min_w_0().overflow_hidden().child(
        Label::new(row.title().to_string())
            .size(FontSize::Micro)
            .weight(FontWeight::MEDIUM),
    );

    if !row.secondary_text().is_empty() {
        text = text.child(
            Label::new(row.secondary_text().to_string())
                .size(FontSize::Micro)
                .color(SemanticColor::TertiaryLabel),
        );
    }

    let mut status = div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(Spacing::XS.scaled(cx))
        .when_some(row.library_badge.label, |status, label| {
            status.child(
                Label::new(label)
                    .size(FontSize::Micro)
                    .color(SemanticColor::TertiaryLabel),
            )
        });
    if let Some(state_label) = row.state_label {
        status = status.child(
            Label::new(state_label)
                .size(FontSize::Micro)
                .color(SemanticColor::Accent),
        );
    }
    status = status.child(TagBadge::new(TagBadgeDisplay {
        kind: entity_kind,
        label: Some(SharedString::from(row.entity_badge.label)),
    }));

    text.child(status).into_any_element()
}

/// The "Load more" button. A failed older page shows its notice above
/// the button, in the style of the failed state (ADR 0086 Decision 5).
fn render_content_list_load_more(
    display: &ContentListLoadMoreDisplay,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    div()
        .pt(Spacing::SM.scaled(cx))
        .child(
            UiButton::styled(display.button_id, ControlStyle::Neutral)
                .label(display.label)
                .a11y_label(display.a11y_label)
                .disabled(display.disabled)
                .on_click(cx.listener(|this, _: &ClickEvent, _window, cx| {
                    this.start_recent_music_load(true, cx);
                })),
        )
        .into_any_element()
}

const fn entity_kind_for_content(kind: ContentListEntityKind) -> EntityKind {
    match kind {
        ContentListEntityKind::Release => EntityKind::Release,
        ContentListEntityKind::Track => EntityKind::Track,
    }
}
