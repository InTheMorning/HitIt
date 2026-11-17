//! The panel of an open value routes cell of the Library track detail compare
//! grid (musicindex ADR 0082 R-ROUTES-8 item 1). The grid shows it in the row
//! under the compare row, because a compare cell is too narrow for it.
//!
//! The view model `view_models::value_routes` gives each value. This file only
//! renders it with the tokens of R-ROUTES-7.

#![warn(clippy::pedantic)]

use std::collections::BTreeSet;

use gpui::{
    canvas, div, point, prelude::*, px, relative, AnyElement, ClipboardItem, Context, SharedString,
    Styled,
};

use crate::library::LibraryApp;
use crate::ui::composites::{DisclosureIndicator, DisclosureIndicatorDisplay};
use crate::ui::primitives::{Button, ButtonSize, Divider, MultilineText, Tooltip};
use crate::ui::style::{color, spacing};
use crate::ui::tokens::{log_font_family, FontSize, Radius, ValueRoutesShape};
use crate::view_models::track_metadata_grid::{
    TrackMetadataGridVm, TrackMetadataValueRouteItemDisplay,
};
use crate::view_models::value_routes::{
    segment_widths, value_routes_view, ShareSegment, ShareSegmentKind, ValueRouteRow,
    ValueRoutesView, PART_SEPARATOR,
};

/// musicindex ADR 0082: the panel of an open value routes cell. The summary bar and its
/// names line (R-ROUTES-1 and R-ROUTES-2), then the panel with the groups
/// "Recipients" and "Fees" (R-ROUTES-3). The compare color of the grid stays on
/// the names (R-ROUTES-8 item 2).
pub(super) fn value_routes_panel(
    raw_value: &str,
    column: &str,
    row_id: &str,
    color: gpui::Rgba,
    expanded_cells: &BTreeSet<String>,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let Some(view) = value_routes_view(raw_value) else {
        return MultilineText::new(TrackMetadataGridVm::text_value_display(raw_value))
            .max_lines(20)
            .color_raw(color)
            .into_any_element();
    };
    if let Some(empty) = view.empty_text {
        return div()
            .text_color(color::text_secondary())
            .child(empty)
            .into_any_element();
    }
    let callout = FontSize::Caption.scaled(cx);
    let mut root = div()
        .w_full()
        .min_w_0()
        .pt(spacing::XS)
        .flex()
        .flex_col()
        .gap(spacing::SM)
        .text_size(FontSize::Body.scaled(cx));
    if !view.segments.is_empty() {
        root = root.child(share_preview(&view, column, row_id, cx));
    }
    let separator_inset = spacing::LG;
    let mut panel = div()
        .w_full()
        .min_w_0()
        .overflow_hidden()
        .flex()
        .flex_col()
        .py(spacing::XS)
        .bg(color::bg_group())
        .rounded(ValueRoutesShape::scaled(
            ValueRoutesShape::PANEL_RADIUS_PX,
            cx,
        ));
    for group in &view.groups {
        // R-ROUTES-3 item 3: the heading of a group.
        panel = panel.child(
            div()
                .px(spacing::LG)
                .pt(spacing::SM)
                .pb(spacing::XXS)
                .text_size(FontSize::Micro.scaled(cx))
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(color::text_secondary())
                .child(group.heading),
        );
        if group.rows.is_empty() {
            panel = panel.child(
                div()
                    .px(spacing::LG)
                    .py(spacing::XS)
                    .text_color(color::text_secondary())
                    .child(group.empty_text),
            );
        }
        for (position, row) in group.rows.iter().enumerate() {
            if position > 0 {
                // R-ROUTES-3 item 7: 1 px, 16 px from the left edge.
                panel = panel.child(div().ml(separator_inset).child(Divider::horizontal()));
            }
            panel = panel.child(value_route_row(
                row,
                column,
                row_id,
                color,
                expanded_cells,
                cx,
            ));
        }
    }
    if let Some(fee_line) = &view.fee_line {
        // R-ROUTES-3 item 6: the fee line is the last row of the panel.
        panel = panel.child(
            div()
                .px(spacing::LG)
                .pt(spacing::SM)
                .pb(spacing::XS)
                .text_size(callout)
                .text_color(color::text_secondary())
                .child(SharedString::from(fee_line.clone())),
        );
    }
    root.child(panel).into_any_element()
}

/// R-ROUTES-1 and R-ROUTES-2: the summary bar and the names line under it.
/// The names line is muted, in the Callout style, at the center. Each part
/// moves as one unit.
fn share_preview(
    view: &ValueRoutesView,
    column: &str,
    row_id: &str,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(spacing::XS)
        .child(share_stack(&view.segments, column, row_id, cx))
        .child(
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .justify_center()
                .text_size(FontSize::Caption.scaled(cx))
                .text_color(color::text_secondary())
                .children(view.names.iter().enumerate().map(|(index, part)| {
                    let text = if index == 0 {
                        part.clone()
                    } else {
                        format!("{PART_SEPARATOR}{part}")
                    };
                    div().whitespace_nowrap().child(SharedString::from(text))
                })),
        )
        .into_any_element()
}

/// R-ROUTES-1: the summary bar. Each segment shows its values on hover.
fn share_stack(
    segments: &[ShareSegment],
    column: &str,
    row_id: &str,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let width = ValueRoutesShape::scaled(ValueRoutesShape::STACK_WIDTH_PX, cx);
    let gap = ValueRoutesShape::scaled(ValueRoutesShape::STACK_GAP_PX, cx);
    let minimum = ValueRoutesShape::scaled(ValueRoutesShape::STACK_MINIMUM_SEGMENT_PX, cx);
    let stripe = ValueRoutesShape::scaled(ValueRoutesShape::STACK_FEE_STRIPE_PX, cx);
    let end_radius = ValueRoutesShape::scaled(ValueRoutesShape::STACK_END_RADIUS_PX, cx);
    let count = segments.len();
    let percents: Vec<f64> = segments.iter().map(|segment| segment.percent).collect();
    let widths = segment_widths(
        &percents,
        f32::from(width),
        f32::from(gap),
        f32::from(minimum),
    );
    div()
        .flex()
        .flex_row()
        .gap(gap)
        .w(width)
        .h(ValueRoutesShape::scaled(
            ValueRoutesShape::STACK_HEIGHT_PX,
            cx,
        ))
        .rounded(end_radius)
        .overflow_hidden()
        .children(segments.iter().zip(widths).enumerate().map(
            |(index, (segment, segment_width))| {
                let tooltip = segment.tooltip.clone();
                let mut part = div()
                    .id(SharedString::from(format!(
                        "value-share:{column}:{row_id}:{index}"
                    )))
                    .relative()
                    .flex_shrink_0()
                    .w(px(segment_width))
                    .h_full()
                    .overflow_hidden()
                    .tooltip(move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx));
                // R-ROUTES-1: the bar has round ends. `GPUI` clips a child only
                // to a rectangle, so the first and the last segment get the radius.
                if index == 0 {
                    part = part.rounded_l(end_radius);
                }
                if index + 1 == count {
                    part = part.rounded_r(end_radius);
                }
                part = match segment.kind {
                    ShareSegmentKind::Split => part.bg(color::share()),
                    ShareSegmentKind::Other => part.bg(color::text_secondary()),
                    ShareSegmentKind::Fee => {
                        part.bg(color::share_track()).child(fee_stripes(stripe))
                    }
                };
                part
            },
        ))
        .into_any_element()
}

/// R-ROUTES-1 item 5: diagonal stripes of the share color over the share track
/// color, `stripe` wide each.
fn fee_stripes(stripe: gpui::Pixels) -> impl IntoElement {
    canvas(
        |_bounds, _window, _cx| {},
        move |bounds, (), window, _cx| {
            let height = bounds.size.height;
            // A stripe at 45 degrees is wider on the x axis by the square root of 2.
            let along = stripe * std::f32::consts::SQRT_2;
            let mut x = bounds.origin.x - height;
            let end = bounds.origin.x + bounds.size.width;
            let top = bounds.origin.y;
            let bottom = top + height;
            while x < end {
                let mut path = gpui::Path::new(point(x, bottom));
                path.line_to(point(x + along, bottom));
                path.line_to(point(x + along + height, top));
                path.line_to(point(x + height, top));
                path.line_to(point(x, bottom));
                window.paint_path(path, color::share());
                x += along * 2.0;
            }
        },
    )
    .absolute()
    .size_full()
}

/// R-ROUTES-4 item 3: the split bar of a row.
fn share_meter(fraction: f32, cx: &mut Context<LibraryApp>) -> AnyElement {
    // In a narrow row the meter is the first column to become smaller.
    div()
        .min_w_0()
        .w(ValueRoutesShape::scaled(
            ValueRoutesShape::METER_WIDTH_PX,
            cx,
        ))
        .h(ValueRoutesShape::scaled(
            ValueRoutesShape::METER_HEIGHT_PX,
            cx,
        ))
        .rounded(ValueRoutesShape::scaled(
            ValueRoutesShape::METER_RADIUS_PX,
            cx,
        ))
        .overflow_hidden()
        .bg(color::share_track())
        .child(div().h_full().w(relative(fraction)).bg(color::share()))
        .into_any_element()
}

/// The minimum width of the name in a narrow row. This is a `HitIt`
/// value, not a shared token.
const NAME_MINIMUM_REM: f32 = 3.0;
/// The minimum width of the percentage column in a narrow row. It holds
/// "100.0 %". This is a `HitIt` value, not a shared token.
const SHARE_MINIMUM_REM: f32 = 3.25;

/// R-ROUTES-4 and R-ROUTES-5: one row with two lines, and its open content.
fn value_route_row(
    row: &ValueRouteRow,
    column: &str,
    row_id: &str,
    color: gpui::Rgba,
    expanded_cells: &BTreeSet<String>,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let item_key = TrackMetadataGridVm::value_route_item_key(column, row_id, row.index);
    let TrackMetadataValueRouteItemDisplay {
        item_key: header_key,
        item_id,
        header_id,
        expanded,
    } = TrackMetadataGridVm::library_value_route_item_display(
        column,
        row_id,
        row.index,
        expanded_cells.contains(&item_key),
    );
    let callout = FontSize::Caption.scaled(cx);
    let mono = log_font_family(cx);
    let chevron_width = spacing::LG;
    let meter = share_meter(row.meter_fraction, cx);
    // R-ROUTES-4 items 1 and 6: line 1 has fixed columns for the split bar and
    // the percentage, so the split bars of a panel line up. In a narrow sheet the
    // row can be narrower than the columns. Then the meter and the percentage column become
    // smaller by the same quantity in each row, so the columns stay aligned. The
    // name and the percentage keep a minimum width.
    let line_one = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(spacing::SM)
        .child(
            div()
                .w(chevron_width)
                .flex_shrink_0()
                .child(DisclosureIndicator::new(DisclosureIndicatorDisplay {
                    expanded,
                })),
        )
        .child(
            div()
                .flex_1()
                .min_w(ValueRoutesShape::scaled_rem(NAME_MINIMUM_REM, cx))
                .truncate()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(color)
                .child(SharedString::from(row.name.clone())),
        )
        .child(meter)
        .child(
            div()
                .w(ValueRoutesShape::scaled_rem(
                    ValueRoutesShape::SHARE_COLUMN_REM,
                    cx,
                ))
                .min_w(ValueRoutesShape::scaled_rem(SHARE_MINIMUM_REM, cx))
                .text_right()
                .whitespace_nowrap()
                .child(SharedString::from(row.share_text.clone())),
        );
    let mut header = div()
        .id(SharedString::from(
            header_id.expect("Library value-route rows have header ids"),
        ))
        .cursor_pointer()
        .px(spacing::LG)
        .py(spacing::XS)
        .flex()
        .flex_col()
        .on_click(cx.listener(move |this, _: &gpui::ClickEvent, _window, cx| {
            this.toggle_metadata_cell(header_key.clone(), cx);
        }))
        .child(line_one);
    if let Some(short) = &row.short_address {
        // R-ROUTES-4 item 5: line 2, under the name, cut with an ellipsis. The
        // hover text gives the full address (R-ROUTES-5 item 4).
        let full = row.full_address.clone().unwrap_or_default();
        header = header.child(
            div()
                .id(SharedString::from(format!("{item_id}:address")))
                .pl(chevron_width + spacing::SM)
                .min_w_0()
                .truncate()
                .font_family(mono.clone())
                .text_size(callout)
                .text_color(color::text_secondary())
                .tooltip(move |window, cx| Tooltip::new(full.clone()).build(window, cx))
                .child(SharedString::from(short.clone())),
        );
    }
    let mut item = div()
        .id(SharedString::from(item_id.clone()))
        .flex()
        .flex_col()
        .child(header);
    if expanded {
        item = item.child(value_route_open_content(row, &item_id, mono, cx));
    }
    item.into_any_element()
}

/// The width of the label column of the open content. It holds "Custom value".
/// This is a `HitIt` value, not a shared token.
const DETAIL_LABEL_REM: f32 = 6.0;

/// R-ROUTES-5 items 2 and 4: the open content of a read-only row. Line 1 is the
/// full address with Copy. The labeled lines follow, with the labels in one
/// column (operator choice of 2026-11-17). `HitIt` has no add control and no
/// edit control.
fn value_route_open_content(
    row: &ValueRouteRow,
    item_id: &str,
    mono: SharedString,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let inset = spacing::SM;
    let mut content = div()
        .mx(inset)
        .mb(inset)
        .p(spacing::SM)
        .flex()
        .flex_col()
        .gap(spacing::XS)
        .bg(color::bg_open_entry())
        .rounded(Radius::MD.scaled(cx))
        .text_size(FontSize::Caption.scaled(cx));
    if let (Some(full), Some(open)) = (&row.full_address, &row.open_address) {
        let copied = full.clone();
        // Copy is at the center of the address lines, so a one-line address is
        // on the line of Copy, as on the website.
        content = content.child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(spacing::SM)
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .font_family(mono)
                        .child(SharedString::from(open.clone())),
                )
                .child(
                    Button::tinted(SharedString::from(format!("{item_id}:copy")))
                        .size(ButtonSize::Sm)
                        .label("Copy")
                        .a11y_label(SharedString::from(row.copy_label.clone()))
                        .on_click(move |_, _, cx| {
                            cx.write_to_clipboard(ClipboardItem::new_string(copied.clone()));
                        }),
                ),
        );
    }
    let label_width = ValueRoutesShape::scaled_rem(DETAIL_LABEL_REM, cx);
    for detail in &row.details {
        content = content.child(
            div()
                .flex()
                .flex_row()
                .items_start()
                .gap(spacing::SM)
                .child(
                    div()
                        .w(label_width)
                        .flex_shrink_0()
                        .text_color(color::text_secondary())
                        .child(detail.label),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(SharedString::from(detail.value.clone())),
                ),
        );
    }
    content.into_any_element()
}
