//! The heading row above the entries of a list that has no list bar
//! (musicindex ADR 0053 R-LIST-11 item 3).
//!
//! The count of the entries is at the left. The sort menu or the Reverse
//! toggle is at the right (musicindex ADR 0054 R-LIST-13 and R-LIST-14).
//! The row has no view icons (musicindex ADR 0053 R-LIST-10 item 2). The
//! view model gives each text, each id and each state. This composite only
//! lays them out, with the renderers of the list page head.

#![warn(clippy::pedantic)]

use gpui::{div, prelude::*, AnyElement, App, IntoElement, ParentElement, SharedString, Styled};

use crate::ui::composites::list_page_head::{
    reverse_toggle, sort_menu, ListReverseHandler, ListSortHandler,
};
use crate::ui::primitives::Label;
use crate::ui::tokens::{FontSize, SemanticColor, Spacing};
use crate::view_models::list_page::{ListBarControlsVm, SectionHeadingVm};

/// Renders the heading row of `heading`. A handler of `None` leaves its
/// control without an action.
pub(crate) fn section_heading(
    heading: SectionHeadingVm,
    on_sort: Option<&ListSortHandler>,
    on_reverse: Option<&ListReverseHandler>,
    cx: &App,
) -> AnyElement {
    let SectionHeadingVm {
        count_text,
        controls:
            ListBarControlsVm {
                sort,
                reverse_toggle: toggle,
                ..
            },
    } = heading;
    let mut right = div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .ml_auto()
        .gap(Spacing::XS.scaled(cx));
    if let Some(sort) = sort {
        right = right.child(sort_menu(sort, on_sort, false));
    }
    if let Some(toggle) = toggle {
        right = right.child(reverse_toggle(&toggle, on_reverse, false));
    }
    div()
        .id(SharedString::from("music-section-heading"))
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(Spacing::SM.scaled(cx))
        .child(
            Label::new(count_text)
                .size(FontSize::Caption)
                .color(SemanticColor::TertiaryLabel),
        )
        .child(right)
        .into_any_element()
}
