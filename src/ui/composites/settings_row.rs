//! One setting of Settings: the name and the hint at the start, the control
//! at the end (ADR 0093 Decision 6).
//!
//! The view model gives the name, the hint and the control type. This
//! composite only lays them out. The name block has a minimum width of the
//! `ColumnShort` token. When the row is too narrow for the name block and the
//! control, the control wraps below the name. A text field is always below
//! its name.

#![warn(clippy::pedantic)]

use gpui::{div, prelude::*, AnyElement, App, SharedString};

use crate::ui::tokens::{color, FontSize, SemanticColor, Size, Spacing};

/// The display fields of one row.
pub(crate) struct SettingsRowDisplay {
    pub(crate) name: SharedString,
    pub(crate) hint: SharedString,
    /// True for a control that is always below the name.
    pub(crate) control_below: bool,
}

/// Renders one row of a Settings section.
#[must_use]
pub(crate) fn settings_row(
    display: SettingsRowDisplay,
    control: AnyElement,
    cx: &App,
) -> AnyElement {
    let text = div()
        .flex()
        .flex_col()
        .gap(Spacing::XXS.scaled(cx))
        .min_w_0()
        .child(
            div()
                .text_size(FontSize::Body.scaled(cx))
                .font_weight(gpui::FontWeight::MEDIUM)
                .child(display.name),
        )
        .child(
            div()
                .text_size(FontSize::Caption.scaled(cx))
                .text_color(color(cx, SemanticColor::SecondaryLabel))
                .child(display.hint),
        );
    if display.control_below {
        return div()
            .w_full()
            .min_w_0()
            .flex()
            .flex_col()
            .gap(Spacing::XS.scaled(cx))
            .child(text)
            .child(control)
            .into_any_element();
    }
    div()
        .w_full()
        .min_w_0()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .justify_between()
        .gap(Spacing::XS.scaled(cx))
        .child(text.flex_1().min_w(Size::ColumnShort.scaled(cx)))
        .child(div().min_w_0().max_w_full().child(control))
        .into_any_element()
}
