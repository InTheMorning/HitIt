//! The group headings of the Artists list (musicindex ADR 0057 R-ARTIST-1).
//!
//! The view model gives the words of each heading and its count. This shell
//! owns only layout, with named tokens.

#![warn(clippy::pedantic)]

use gpui::{div, prelude::*, AnyElement, Context, FontWeight, SharedString, Styled};

use crate::library::LibraryApp;
use crate::ui::primitives::Label;
use crate::ui::style::spacing;
use crate::ui::tokens::{FontSize, SemanticColor};
use crate::view_models::artist_groups::ArtistGroupHeadingVm;

/// Renders the heading of one group: its words and count, and the line that
/// says what the group is. In the Tiles view it takes a full line of the
/// grid.
pub(crate) fn render_artist_group_heading(
    heading: &ArtistGroupHeadingVm,
    _cx: &mut Context<LibraryApp>,
) -> AnyElement {
    div()
        .id(SharedString::from(heading.element_id.clone()))
        .w_full()
        .flex()
        .flex_col()
        .gap(spacing::XS)
        .pt(spacing::LG)
        .pb(spacing::XS)
        .child(
            Label::new(heading.title.clone())
                .size(FontSize::Caption)
                .weight(FontWeight::MEDIUM)
                .color(SemanticColor::SecondaryLabel),
        )
        .child(
            Label::new(heading.intro)
                .size(FontSize::Micro)
                .color(SemanticColor::TertiaryLabel),
        )
        .into_any_element()
}

/// Gives `element` a full line of the Tiles grid, so that a status row does
/// not sit between two tiles.
pub(crate) fn render_full_line(element: AnyElement) -> AnyElement {
    div().w_full().child(element).into_any_element()
}
