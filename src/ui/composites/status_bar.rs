//! Status bar composite.
//!
//! ADR 0101 Decision 7 owns the one status bar at the bottom of the window.
//! The left side shows the latest app event and truncates. The right side
//! shows small items that keep their place. The composite renders display
//! facts and exposes one callback: open the detail of an item.

#![warn(clippy::pedantic)]

use std::rc::Rc;

use gpui::{
    div, prelude::*, App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window,
};

use crate::ui::control_styles::ControlStyle;
use crate::ui::icons::IconName;
use crate::ui::primitives::{Button as UiButton, Tooltip};
use crate::ui::tokens::{color, FontSize, SemanticColor, Size, Spacing};
use crate::view_models::status_bar::{
    StatusBarDisplay, StatusEventDisplay, StatusItemDetail, StatusItemDisplay, StatusItemIcon,
};

type StatusItemHandler = Rc<dyn Fn(StatusItemDetail, &mut Window, &mut App) + 'static>;

/// Callback slots supplied by the app shell.
#[derive(Default)]
#[must_use]
pub(crate) struct StatusBarSlots {
    open_item: Option<StatusItemHandler>,
}

impl StatusBarSlots {
    /// Creates empty status bar slots.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Supplies the callback that opens the detail of an item.
    pub(crate) fn on_open_item(
        mut self,
        handler: impl Fn(StatusItemDetail, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.open_item = Some(Rc::new(handler));
        self
    }
}

/// Status bar element.
#[derive(IntoElement)]
#[must_use]
pub(crate) struct StatusBar {
    display: StatusBarDisplay,
    slots: StatusBarSlots,
}

/// Creates a status bar.
pub(crate) fn status_bar(display: StatusBarDisplay, slots: StatusBarSlots) -> StatusBar {
    StatusBar { display, slots }
}

impl RenderOnce for StatusBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let StatusBarDisplay { id, event, items } = self.display;
        let handler = self.slots.open_item;
        div()
            .id(id)
            .flex()
            .flex_row()
            // A narrow bar wraps: the items go below the event, and wrap
            // among themselves when they do not fit one row.
            .flex_wrap()
            .items_center()
            .justify_between()
            .gap(Spacing::XS.scaled(cx))
            .flex_shrink_0()
            .px(Spacing::XL.scaled(cx))
            .py(Spacing::XS.scaled(cx))
            // The bar keeps one row of height when it has no event and no
            // item, so the content above does not move when an event comes.
            .min_h(Size::MinHitTarget.scaled(cx))
            .border_t_1()
            .border_color(color(cx, SemanticColor::Separator))
            .bg(color(cx, SemanticColor::GroupBackground))
            .child(render_event(event.as_ref(), cx))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_center()
                    .gap(Spacing::SM.scaled(cx))
                    .max_w_full()
                    .children(
                        items
                            .into_iter()
                            .map(|item| render_item(item, handler.clone())),
                    ),
            )
    }
}

fn render_event(event: Option<&StatusEventDisplay>, cx: &App) -> impl IntoElement {
    let tooltip = event.map(|event| Tooltip::new(event.a11y_label.clone()));
    div()
        .id("status-bar-event")
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::SM.scaled(cx))
        .flex_1()
        .min_w(Size::MenuCompact.scaled(cx))
        .text_size(FontSize::Caption.scaled(cx))
        .text_color(color(cx, SemanticColor::SecondaryLabel))
        .when_some(tooltip, |el, tooltip| {
            el.tooltip(move |window, cx| tooltip.build(window, cx))
        })
        .when_some(event, |el, event| {
            el.child(
                div()
                    .flex_shrink_0()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(color(cx, SemanticColor::TertiaryLabel))
                    .child(SharedString::from(event.time_label.clone())),
            )
            .child(
                div()
                    .min_w_0()
                    .truncate()
                    .child(SharedString::from(event.words.clone())),
            )
        })
}

fn render_item(item: StatusItemDisplay, handler: Option<StatusItemHandler>) -> UiButton {
    let detail = item.detail;
    let mut button = UiButton::styled(item.id, ControlStyle::Neutral)
        .leading_icon(item_icon(item.icon))
        .label(item.word)
        .a11y_label(item.a11y_label.clone())
        .tooltip(item.a11y_label)
        // Tab reaches the bar after the page and its list. A detail list has
        // Tab index 1 (ADR 0092 task 002), so an item with index 0 came
        // before the rows (operator walk of 2026-11-17).
        .tab_index(1);
    match handler {
        Some(handler) if item.enabled => {
            // A pointer click, Enter and Space open the detail (ADR 0069).
            button = button.on_activate(move |window, cx| {
                handler(detail, window, cx);
            });
        }
        _ => button = button.disabled(true),
    }
    button
}

const fn item_icon(icon: StatusItemIcon) -> IconName {
    match icon {
        StatusItemIcon::Feed => IconName::Rss,
        StatusItemIcon::Files => IconName::Track,
        StatusItemIcon::Reports => IconName::Warning,
        StatusItemIcon::Show => IconName::Show,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_models::status_bar::StatusItemKind;

    struct BarTabRoot {
        page_control: gpui::FocusHandle,
        list: gpui::FocusHandle,
    }

    impl gpui::Render for BarTabRoot {
        fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
            let item = StatusItemDisplay {
                kind: StatusItemKind::Files,
                id: "status-item-files",
                icon: StatusItemIcon::Files,
                word: "Update 28 files".into(),
                a11y_label: "Update 28 files".into(),
                detail: StatusItemDetail::TagUpdatePopup,
                enabled: true,
            };
            div()
                .size_full()
                .flex()
                .flex_col()
                .child(
                    div()
                        .track_focus(&self.list.clone().tab_index(1).tab_stop(true))
                        .child(
                            UiButton::plain("page-control")
                                .label("Reverse")
                                .track_focus(&self.page_control)
                                .on_click(|_, _, _| {}),
                        ),
                )
                .child(status_bar(
                    StatusBarDisplay {
                        id: "status-bar",
                        event: None,
                        items: vec![item],
                    },
                    StatusBarSlots::new().on_open_item(|_, _, _| {}),
                ))
        }
    }

    /// Operator walk of 2026-11-17: Tab went from "Reverse" to the status bar
    /// item "Update 28 files" before the rows of the release page.
    #[gpui::test]
    fn adr_0101_task_008_tab_reaches_the_list_before_the_status_bar(cx: &mut gpui::TestAppContext) {
        cx.update(gpui_component::init);
        let (root, cx) = cx.add_window_view(|_, cx| BarTabRoot {
            page_control: cx.focus_handle(),
            list: cx.focus_handle(),
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.run_until_parked();
        let (page_control, list) = cx.update(|_, cx| {
            let root = root.read(cx);
            (root.page_control.clone(), root.list.clone())
        });
        cx.update(|window, cx| window.focus(&page_control, cx));
        cx.update(gpui::Window::focus_next);
        assert!(
            cx.update(|window, _| list.is_focused(window)),
            "ADR 0092 task 002: Tab goes from the last page control to the list, \
             not to a status bar item. Give each item `Button::tab_index(1)`."
        );
        cx.update(gpui::Window::focus_next);
        assert!(
            cx.update(|window, _| !list.is_focused(window) && !page_control.is_focused(window)),
            "Tab goes from the list to the status bar item"
        );
    }
}
