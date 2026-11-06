//! The title row of a Music list page (ADR 0089 Decision 6, musicindex ADR
//! 0027 R-LIST-1 item 4).
//!
//! The title is at the left. The sort button, the view icons and the
//! Select action are at the right. The scope control sits below the title,
//! at the left. The view model gives each text, each id and each
//! availability fact. This composite only lays them out.
//!
//! ADR 0089 Decision 6 names this row as the one owner of the list page
//! head. A list page renders it above its rows.

#![warn(clippy::pedantic)]

use std::rc::Rc;

use gpui::{
    div, prelude::*, AnyElement, App, FontWeight, IntoElement, ParentElement, SharedString, Styled,
    Window,
};

use crate::ui::composites::{Segment, SegmentDisplay, SegmentedControl};
use crate::ui::control_styles::ControlStyle;
use crate::ui::primitives::Button;
use crate::ui::tokens::{FontSize, Spacing};
use crate::view_models::list_page::{ListPageHeadVm, ListScope, ListScopeControlVm};

/// The callback of a scope selection (ADR 0089 Decision 5).
pub(crate) type ListScopeHandler = Rc<dyn Fn(&ListScope, &mut Window, &mut App) + 'static>;

/// The callback of a title row button, such as "Select".
pub(crate) type ListHeadButtonHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

/// Renders the title row of a list page. `on_scope` is the scope selection,
/// and `on_select` is the click of the Select action. A `None` callback
/// leaves its control without a click.
#[must_use]
pub(crate) fn list_page_head(
    display: ListPageHeadVm,
    on_scope: Option<ListScopeHandler>,
    on_select: Option<ListHeadButtonHandler>,
    cx: &App,
) -> AnyElement {
    let mut trailing = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::XS.scaled(cx));

    if let Some(sort) = display.sort {
        trailing = trailing.child(
            Button::styled(SharedString::from(sort.id), ControlStyle::Ghost)
                .label(sort.label)
                .a11y_label(sort.a11y_label.clone())
                .tooltip(sort.a11y_label),
        );
    }

    if let Some(select) = display.select {
        let a11y_label = select.a11y_label.clone();
        let mut button = Button::styled(SharedString::from(select.id), ControlStyle::Ghost)
            .label(select.label)
            .a11y_label(a11y_label.clone())
            .tooltip(a11y_label);
        if let Some(handler) = on_select {
            button = button.on_click(move |_, window, cx| {
                handler(window, cx);
            });
        }
        trailing = trailing.child(button);
    }

    let title_row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::SM.scaled(cx))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_size(FontSize::Title.scaled(cx))
                .font_weight(FontWeight::SEMIBOLD)
                .truncate()
                .child(SharedString::from(display.title)),
        )
        .child(trailing);

    let mut head = div()
        .flex()
        .flex_col()
        .gap(Spacing::XS.scaled(cx))
        .px(Spacing::MD.scaled(cx))
        .pt(Spacing::XS.scaled(cx))
        .pb(Spacing::XS.scaled(cx))
        .child(title_row);

    if let Some(scope) = display.scope {
        head = head.child(scope_control(scope, on_scope));
    }

    head.into_any_element()
}

/// The scope control of a list page: a `SegmentedControl` with the
/// `Filter` style, "All | In Library".
fn scope_control(display: ListScopeControlVm, on_scope: Option<ListScopeHandler>) -> AnyElement {
    let mut control = SegmentedControl::new(display.selected)
        .filter_style()
        .segments(display.options.map(|option| {
            Segment::new(SegmentDisplay {
                id: SharedString::from(option.id).into(),
                key: option.scope,
                label: SharedString::from(option.label),
                a11y_label: SharedString::from(option.a11y_label),
            })
        }));
    if let Some(handler) = on_scope {
        control = control.on_select(move |scope, window, cx| {
            handler(scope, window, cx);
        });
    }
    div()
        .id(SharedString::from(display.control_id))
        .child(control)
        .into_any_element()
}
