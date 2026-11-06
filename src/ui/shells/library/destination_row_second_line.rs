//! The second line of a destination row (ADR 0089 Decision 6, musicindex ADR
//! 0038 R-DOT-4). The type dot and the type word lead the line, and the
//! secondary text follows. The view model gives the type and the text. The
//! renderer maps each type kind to its entity color here.

#![warn(clippy::pedantic)]

use gpui::{div, prelude::*, AnyElement, ParentElement, SharedString, Styled};

use crate::ui::composites::{EntityKind, TagBadge, TagBadgeDisplay};
use crate::ui::primitives::Label;
use crate::ui::style::spacing;
use crate::ui::tokens::{FontSize, SemanticColor};
use crate::view_models::library::{RowTypeKind, RowTypeTag};

/// The second line of a destination row (ADR 0089 Decision 6): the type dot
/// and the type word lead, before the secondary text. `None` when the row has
/// neither.
pub(crate) fn destination_row_second_line(
    type_tag: Option<RowTypeTag>,
    secondary_text: String,
) -> Option<AnyElement> {
    if type_tag.is_none() && secondary_text.is_empty() {
        return None;
    }
    Some(
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(spacing::XXS)
            .min_w_0()
            .when_some(type_tag, |el, tag| {
                el.child(TagBadge::new(TagBadgeDisplay {
                    kind: row_type_entity_kind(tag.kind),
                    label: Some(SharedString::from(tag.word)),
                }))
            })
            .when(!secondary_text.is_empty(), |el| {
                el.child(
                    Label::new(secondary_text)
                        .size(FontSize::Micro)
                        .color(SemanticColor::TertiaryLabel)
                        .truncated(),
                )
            })
            .into_any_element(),
    )
}

/// The entity color of a row type (ADR 0089 Decision 6). The renderer maps
/// the view model's kind to its dot color, so the kind stays out of the view
/// model.
pub(crate) const fn row_type_entity_kind(kind: RowTypeKind) -> EntityKind {
    match kind {
        RowTypeKind::Release => EntityKind::Release,
        RowTypeKind::Artist => EntityKind::Artist,
        RowTypeKind::Playlist => EntityKind::Playlist,
    }
}
