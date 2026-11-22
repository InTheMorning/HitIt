//! Library track-detail metadata value renderers.
//!
//! Owns expandable values and compact text/tag cells used
//! by the Library metadata grid cells.

#![warn(clippy::pedantic)]

use std::sync::Arc;

use gpui::{div, prelude::*, AnyElement, Context, Image, SharedString, Styled};

use crate::library::LibraryApp;
use crate::ui::composites::{
    DisclosureIndicator, DisclosureIndicatorDisplay, EntityKind, Thumbnail, ThumbnailSize,
    TrackMetadataFrameDisplay, TrackMetadataTagCell, TrackMetadataTagDisplay,
    TrackMetadataTextDisplay, TrackMetadataTextValue,
};
use crate::ui::primitives::MultilineText;
use crate::ui::style::{color, spacing, typography};
use crate::view_models::track_metadata_grid::{
    TrackMetadataExpandableCellDisplay, TrackMetadataExpandedFieldKind, TrackMetadataGridVm,
    TrackMetadataId3FrameColorContext, TrackMetadataId3FrameColorRole, ValueRoutesSummaryFallback,
};

#[expect(
    clippy::too_many_arguments,
    reason = "UI cell renderer keeps field context explicit"
)]
pub(super) fn metadata_value_cell(
    field: &str,
    row_id: &str,
    raw_value: &str,
    base_display: &str,
    glyph: Option<&str>,
    expanded: bool,
    color: gpui::Rgba,
    column: &str,
    file_image: Option<&Arc<Image>>,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let logical_field = TrackMetadataGridVm::logical_field(field);
    let field_kind = TrackMetadataGridVm::expanded_field_kind(logical_field);
    let display_value = TrackMetadataGridVm::display_with_glyph(glyph, base_display);
    let expandable = TrackMetadataGridVm::field_is_expandable(logical_field, raw_value);
    if !expandable {
        return compare_cell(&display_value, Some(color));
    }
    let display = TrackMetadataGridVm::library_expandable_cell_display(column, row_id, expanded);
    let summary = TrackMetadataGridVm::expandable_cell_summary(
        logical_field,
        raw_value,
        base_display,
        glyph,
        ValueRoutesSummaryFallback::DisplayValue,
    );
    // musicindex ADR 0082 R-ROUTES-8 item 1: an open value routes cell keeps
    // its summary. The grid shows the panel in the row under the compare row.
    let content = if expanded && field_kind != TrackMetadataExpandedFieldKind::ValueRoutes {
        expanded_metadata_value(
            field_kind,
            logical_field,
            raw_value,
            &display_value,
            color,
            file_image,
        )
    } else {
        div()
            .text_color(color::accent())
            .truncate()
            .child(SharedString::from(summary))
            .into_any_element()
    };
    let TrackMetadataExpandableCellDisplay {
        cell_key,
        cell_id,
        expanded: cell_expanded,
        ..
    } = display;
    div()
        .id(SharedString::from(cell_id))
        .cursor_pointer()
        .text_size(typography::SIZE_MICRO)
        .line_height(typography::LINE_BODY)
        .flex()
        .flex_row()
        .items_start()
        .gap(spacing::XS)
        .on_click(cx.listener(move |this, _: &gpui::ClickEvent, _window, cx| {
            this.toggle_metadata_cell(cell_key.clone(), cx);
        }))
        .child(DisclosureIndicator::new(DisclosureIndicatorDisplay {
            expanded: cell_expanded,
        }))
        .child(div().flex_1().min_w_0().child(content))
        .into_any_element()
}

#[expect(
    clippy::too_many_arguments,
    reason = "UI cell renderer keeps field context explicit"
)]
pub(super) fn metadata_tag_cell(
    field: &str,
    row_id: &str,
    raw_value: &str,
    base_display: &str,
    glyph: Option<&str>,
    expanded: bool,
    color: gpui::Rgba,
    frame_id: Option<&str>,
    file_image: Option<&Arc<Image>>,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let frame_color = id3_frame_color(TrackMetadataGridVm::id3_frame_color_role(
        frame_id,
        TrackMetadataId3FrameColorContext::Library,
    ));
    let value = metadata_value_cell(
        field,
        row_id,
        raw_value,
        base_display,
        glyph,
        expanded,
        color,
        "id3",
        file_image,
        cx,
    );
    TrackMetadataTagCell::new(TrackMetadataTagDisplay {
        value,
        frame: frame_id.map(|frame_id| TrackMetadataFrameDisplay {
            label: SharedString::from(TrackMetadataGridVm::id3_frame_display_label(Some(frame_id))),
            color: Some(frame_color),
        }),
    })
    .frame_color(frame_color)
    .into_any_element()
}

pub(super) fn compare_tag_cell(
    value: &str,
    color: Option<gpui::Rgba>,
    frame_id: Option<&str>,
    frame_color: Option<gpui::Rgba>,
) -> AnyElement {
    let mut body = TrackMetadataTextValue::new(TrackMetadataTextDisplay {
        value: SharedString::from(TrackMetadataGridVm::text_value_display(value)),
    });
    if let Some(color) = color {
        body = body.color_raw(color);
    }
    let mut cell = TrackMetadataTagCell::new(TrackMetadataTagDisplay {
        value: body.into_any_element(),
        frame: frame_id.map(|frame_id| TrackMetadataFrameDisplay {
            label: SharedString::from(TrackMetadataGridVm::id3_frame_display_label(Some(frame_id))),
            color: frame_color,
        }),
    });
    if let Some(frame_color) = frame_color {
        cell = cell.frame_color(frame_color);
    }
    cell.into_any_element()
}

fn expanded_metadata_value(
    field_kind: TrackMetadataExpandedFieldKind,
    field: &str,
    raw_value: &str,
    display_value: &str,
    color: gpui::Rgba,
    file_image: Option<&Arc<Image>>,
) -> AnyElement {
    if field_kind == TrackMetadataExpandedFieldKind::Artwork {
        if let Some(image) = file_image {
            return div()
                .flex()
                .flex_col()
                .gap(spacing::XS)
                .child(SharedString::from(TrackMetadataGridVm::text_value_display(
                    display_value,
                )))
                .child(
                    Thumbnail::new(EntityKind::Track, ThumbnailSize::Lg).image(Some(image.clone())),
                )
                .into_any_element();
        }
    }
    let value = TrackMetadataGridVm::expanded_display_value(field, raw_value, display_value);
    MultilineText::new(value)
        .max_lines(20)
        .color_raw(color)
        .into_any_element()
}

fn compare_cell(value: &str, color: Option<gpui::Rgba>) -> AnyElement {
    let mut cell = TrackMetadataTextValue::new(TrackMetadataTextDisplay {
        value: SharedString::from(TrackMetadataGridVm::text_value_display(value)),
    });
    if let Some(color) = color {
        cell = cell.color_raw(color);
    }
    cell.into_any_element()
}

fn id3_frame_color(role: TrackMetadataId3FrameColorRole) -> gpui::Rgba {
    match role {
        TrackMetadataId3FrameColorRole::Muted => color::text_muted(),
        TrackMetadataId3FrameColorRole::Accent => color::accent(),
        TrackMetadataId3FrameColorRole::V22 => color::id3_frame_v22(),
        TrackMetadataId3FrameColorRole::V23Only => color::id3_frame_v23_only(),
        TrackMetadataId3FrameColorRole::V24Only => color::id3_frame_v24_only(),
        TrackMetadataId3FrameColorRole::Unknown => color::id3_frame_unknown(),
    }
}
