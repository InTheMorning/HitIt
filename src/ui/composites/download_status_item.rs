//! Download progress item composite (ADR 0084 Decision 10, packets 022
//! and 027).
//!
//! The item shows the download batch on each section, for example
//! "Downloading 3 of 9", and a bar with its count text, for example
//! "3 of 9 done", while the batch runs (ADR 0101 Decision 12). It renders the display facts of
//! [`DownloadProgressDisplay`] and exposes two callbacks: "Stop queued" of
//! a running batch, and "Dismiss" of a finished batch. It makes no text,
//! no count and no time. The text wraps, so a narrow window does not clip
//! it (ADR 0063).

#![warn(clippy::pedantic)]

use std::rc::Rc;

use gpui::{
    div, prelude::*, App, ClickEvent, FontWeight, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window,
};

use crate::ui::control_styles::ControlStyle;
use crate::ui::icons::{Icon, IconName, IconSize};
use crate::ui::primitives::{Button as UiButton, ProgressBar, Tooltip};
use crate::ui::tokens::{color, FontSize, SemanticColor, Spacing};
use crate::view_models::download_progress::{
    DownloadProgressDismissAction, DownloadProgressDisplay, DownloadProgressIconRole,
    DownloadProgressStopQueuedAction,
};

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// Download progress item element.
#[derive(IntoElement)]
#[must_use]
pub(crate) struct DownloadStatusItem {
    display: DownloadProgressDisplay,
    on_stop_queued: ClickHandler,
    on_dismiss: ClickHandler,
}

/// Creates a download progress item. `on_stop_queued` runs when the
/// operator clicks "Stop queued" of a running batch. `on_dismiss` runs
/// when the operator clicks "Dismiss" of a finished batch.
pub(crate) fn download_status_item(
    display: DownloadProgressDisplay,
    on_stop_queued: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    on_dismiss: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> DownloadStatusItem {
    DownloadStatusItem {
        display,
        on_stop_queued: Rc::new(on_stop_queued),
        on_dismiss: Rc::new(on_dismiss),
    }
}

impl RenderOnce for DownloadStatusItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let DownloadProgressDisplay {
            id,
            icon_role,
            title,
            item_name,
            failure_text,
            failure_lines,
            a11y_label,
            progress,
            stop_queued,
            dismiss,
            ..
        } = self.display;
        let icon_color = color(cx, icon_color(icon_role));
        let tooltip = Tooltip::new(a11y_label);

        div()
            .id(id)
            .flex()
            .flex_row()
            .items_start()
            .gap(Spacing::MD.scaled(cx))
            .flex_shrink_0()
            .min_w_0()
            .px(Spacing::XL.scaled(cx))
            .py(Spacing::XS.scaled(cx))
            .border_b_1()
            .border_color(color(cx, SemanticColor::Separator))
            .bg(color(cx, SemanticColor::GroupBackground))
            .child(
                div()
                    .flex_shrink_0()
                    .text_color(icon_color)
                    .child(Icon::new(icon(icon_role)).size(IconSize::Action)),
            )
            .child(
                div()
                    .id("download-progress-summary")
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .gap(Spacing::XXS.scaled(cx))
                    .tooltip(move |window, cx| tooltip.build(window, cx))
                    .child(
                        div()
                            .text_size(FontSize::Body.scaled(cx))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(color(cx, SemanticColor::Label))
                            .child(SharedString::from(title)),
                    )
                    .when(item_name.is_some() || failure_text.is_some(), |el| {
                        el.child(render_detail_line(item_name, failure_text, cx))
                    })
                    .when_some(progress, |el, progress| {
                        el.child(ProgressBar::new(progress.fraction(), progress.count_text()))
                    })
                    .children(failure_lines.into_iter().map(|line| {
                        div()
                            .text_size(FontSize::Caption.scaled(cx))
                            .text_color(color(cx, SemanticColor::DangerLabel))
                            .child(SharedString::from(line))
                    })),
            )
            .when_some(stop_queued, |el, stop_queued| {
                el.child(render_stop_queued_button(stop_queued, &self.on_stop_queued))
            })
            .when_some(dismiss, |el, dismiss| {
                el.child(render_dismiss_button(dismiss, &self.on_dismiss))
            })
    }
}

fn render_detail_line(
    item_name: Option<String>,
    failure_text: Option<String>,
    cx: &App,
) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .gap(Spacing::SM.scaled(cx))
        .text_size(FontSize::Caption.scaled(cx))
        .when_some(item_name, |el, name| {
            el.child(
                div()
                    .min_w_0()
                    .text_color(color(cx, SemanticColor::SecondaryLabel))
                    .child(SharedString::from(name)),
            )
        })
        .when_some(failure_text, |el, text| {
            el.child(
                div()
                    .flex_shrink_0()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(color(cx, SemanticColor::WarningLabel))
                    .child(SharedString::from(text)),
            )
        })
}

fn render_dismiss_button(
    dismiss: DownloadProgressDismissAction,
    handler: &ClickHandler,
) -> UiButton {
    let handler = Rc::clone(handler);
    UiButton::styled(dismiss.id, ControlStyle::Neutral)
        .leading_icon(IconName::Close)
        .label(dismiss.label)
        .a11y_label(dismiss.a11y_label)
        .tooltip(dismiss.a11y_label)
        .on_click(move |event, window, cx| handler(event, window, cx))
}

fn render_stop_queued_button(
    stop_queued: DownloadProgressStopQueuedAction,
    handler: &ClickHandler,
) -> UiButton {
    let handler = Rc::clone(handler);
    UiButton::styled(stop_queued.id, ControlStyle::Secondary)
        .leading_icon(IconName::Stop)
        .label(stop_queued.label)
        .a11y_label(stop_queued.a11y_label.clone())
        .tooltip(stop_queued.a11y_label)
        .on_click(move |event, window, cx| handler(event, window, cx))
}

const fn icon(role: DownloadProgressIconRole) -> IconName {
    match role {
        DownloadProgressIconRole::InProgress => IconName::Info,
        DownloadProgressIconRole::Success => IconName::Check,
        DownloadProgressIconRole::Warning => IconName::Warning,
    }
}

const fn icon_color(role: DownloadProgressIconRole) -> SemanticColor {
    match role {
        DownloadProgressIconRole::InProgress => SemanticColor::InfoLabel,
        DownloadProgressIconRole::Success => SemanticColor::SuccessLabel,
        DownloadProgressIconRole::Warning => SemanticColor::WarningLabel,
    }
}

#[cfg(test)]
mod tests {
    use gpui::{px, size, Context, Render, TestAppContext};

    use super::*;
    use crate::ui::primitives::progress_bar::{TRACK_HEIGHT, TRACK_MIN_WIDTH};
    use crate::view_models::download_progress::DownloadBatch;

    struct Item(DownloadProgressDisplay);

    impl Render for Item {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            download_status_item(self.0.clone(), |_, _, _| {}, |_, _, _| {})
        }
    }

    /// ADR 0101 Decision 12: the bar of a running batch shows. Its track is at
    /// least `TRACK_MIN_WIDTH` wide and `TRACK_HEIGHT` high, and the fill is
    /// the fraction of the track. Walk of 2026-11-16: the track had the color
    /// of the item and did not show.
    #[gpui::test]
    fn the_progress_bar_of_a_running_batch_has_a_track_and_a_fill(cx: &mut TestAppContext) {
        cx.update(gpui_component::init);
        let batch = DownloadBatch {
            total: 4,
            done: 1,
            failures: Vec::new(),
            stopped: 0,
            queued: 0,
            current_label: None,
            finished_at: None,
            waiting: None,
        };
        let display = DownloadProgressDisplay::project(&batch).expect("a display");
        let (_root, cx) = cx.add_window_view(|_, _| Item(display));
        cx.simulate_resize(size(px(390.0), px(300.0)));
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let track = cx
            .debug_bounds("progress-bar-track")
            .expect("the track has bounds");
        let fill = cx
            .debug_bounds("progress-bar-fill")
            .expect("the fill has bounds");
        assert!(f32::from(track.size.width) >= TRACK_MIN_WIDTH, "{track:?}");
        assert!(
            (f32::from(track.size.height) - TRACK_HEIGHT).abs() < 0.5,
            "{track:?}"
        );
        let expected = f32::from(track.size.width) * 0.25;
        assert!(
            (f32::from(fill.size.width) - expected).abs() < 0.5,
            "{fill:?}"
        );
    }
}
