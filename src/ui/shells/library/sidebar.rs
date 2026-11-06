//! Sidebar rows (ADR 0089 Decision 1).
//!
//! Renders the rows of `SidebarVm`. The sidebar is one list with no group
//! labels, and it shows on each mount. A row click emits
//! `LibraryAppEvent::SidebarRowSelected`, and the top app routes it, so
//! Show and Settings reach their mounts. Callbacks dispatch back through
//! `cx.listener(...)` calls.

#![warn(clippy::pedantic)]

use gpui::{
    div, prelude::*, AnyElement, App, ClickEvent, Context, DragMoveEvent, FocusHandle, FontWeight,
    SharedString, Window,
};

use crate::library::{LibraryApp, LibraryAppEvent};
use crate::ui::composites::tab_bar::sidebar_icon_name as icon_name;
use crate::ui::composites::track_drag::{PlaylistTrackDragPayload, TrackDragPayload};
use crate::ui::icons::{Icon, IconName, IconSize};
use crate::ui::primitives::Label;
use crate::ui::style::{color, spacing};
use crate::ui::tokens::{resolve_color, SemanticColor};
use crate::view_models::sidebar::SidebarRowVm;

/// Renders the separator line that follows the upper rows (ADR 0089
/// Decision 1).
pub(crate) fn render_sidebar_separator() -> AnyElement {
    div()
        .id("sidebar-separator")
        .w_full()
        .my(spacing::XS)
        .border_t_1()
        .border_color(color::border_subtle())
        .into_any_element()
}

/// Renders one sidebar row with its icon, label and count (ADR 0089
/// Decision 1). A row with no icon keeps the width of the icon column, so
/// its label aligns with the others.
pub(crate) fn render_sidebar_row(
    row: &SidebarRowVm,
    focus: &FocusHandle,
    window: &Window,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let destination = row.destination;
    let selected = row.selected;
    let focused = focus.is_focused(window);
    let icon = icon_name(row.icon);
    let tooltip_text = row.tooltip.clone();
    let count_label = row.count.map(|count| count.to_string());
    let shows_attention_mark = row.shows_attention_mark;
    let focus_color = resolve_color(cx, SemanticColor::Focus, None);
    let music_destination = destination.music_destination();
    let spring_loaded = row.spring_loaded;

    let mut el = div()
        .id(SharedString::from(row.element_id))
        .track_focus(focus)
        .flex()
        .items_center()
        .w_full()
        .gap(spacing::SM)
        .px(spacing::SM)
        .py(spacing::XS)
        .cursor_pointer()
        .rounded(spacing::XS)
        .when(selected, |el| el.bg(color::bg_selected()))
        .when(focused, |el| el.border_2().border_color(focus_color))
        .hover(|el| el.bg(color::bg_surface_hi()))
        .tooltip({
            let tooltip = crate::ui::primitives::Tooltip::new(tooltip_text);
            move |window, cx| tooltip.build(window, cx)
        })
        .on_click(cx.listener(move |_this, _: &ClickEvent, _window, cx| {
            cx.emit(LibraryAppEvent::SidebarRowSelected(destination));
        }));

    // ADR 0083 Decision 6, task 008 decision D1: a spring-loaded row opens
    // its destination while a dragged track stays over it. The row accepts
    // no drop.
    if let (true, Some(music)) = (spring_loaded, music_destination) {
        el = el
            .on_drag_move(cx.listener(
                move |this, event: &DragMoveEvent<TrackDragPayload>, _window, cx| {
                    let on_row = event.bounds.contains(&event.event.position);
                    this.spring_load_pointer(music, on_row, cx);
                },
            ))
            .on_drag_move(cx.listener(
                move |this, event: &DragMoveEvent<PlaylistTrackDragPayload>, _window, cx| {
                    let on_row = event.bounds.contains(&event.event.position);
                    this.spring_load_pointer(music, on_row, cx);
                },
            ));
    }

    el.child(render_row_icon(icon, selected, shows_attention_mark, cx))
        .child(
            Label::new(row.label)
                .color(if selected {
                    crate::ui::tokens::SemanticColor::Accent
                } else {
                    crate::ui::tokens::SemanticColor::Label
                })
                .weight(if selected {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::NORMAL
                })
                .truncated(),
        )
        .when_some(count_label, |el, count| {
            el.child(div().flex_1()).child(
                Label::new(count)
                    .size(crate::ui::tokens::FontSize::Micro)
                    .color(SemanticColor::TertiaryLabel),
            )
        })
        .into_any_element()
}

/// Renders a row's icon, with the attention mark when one is called for
/// (ADR 0089 Decision 1). The mark never relies on color alone: the row's
/// tooltip and accessibility label already give the count in text.
fn render_row_icon(
    icon: IconName,
    selected: bool,
    shows_attention_mark: bool,
    cx: &App,
) -> AnyElement {
    let color = if selected {
        color::accent()
    } else {
        color::text_primary()
    };
    let glyph = Icon::new(icon).size(IconSize::Action).color(color);
    if !shows_attention_mark {
        return glyph.into_any_element();
    }
    div()
        .relative()
        .child(glyph)
        .child(
            div()
                .absolute()
                .top_0()
                .right_0()
                .size(spacing::XS)
                .rounded_full()
                .bg(resolve_color(cx, SemanticColor::Danger, None)),
        )
        .into_any_element()
}
