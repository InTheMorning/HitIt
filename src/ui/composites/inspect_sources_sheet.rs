//! The Inspect Sources sheet (ADR 0089 Decision 8, ADR 0083 Decision 8).
//!
//! The sheet shows the source values of one track in a `Root` sheet, over
//! the track page. The body is a view that observes its source entity, so a
//! change of the source redraws the open sheet in place. The sheet closes
//! with Close, Escape or a click on the overlay, and `Root` returns the
//! focus to the control that opened it. The host passes the body and the
//! close function. This composite holds no domain state.

#![warn(clippy::pedantic)]

use gpui::{
    div, prelude::*, AnyElement, App, AppContext, Context, Entity, FocusHandle, IntoElement,
    Render, Subscription, Window,
};
use gpui_component::WindowExt;

use crate::ui::layouts::INSPECT_SOURCES_SHEET_WIDTH;

/// The title of the sheet. It is the item menu label (ADR 0089 Decision 8).
const INSPECT_SOURCES_SHEET_TITLE: &str = "Inspect Sources";

/// The host of one sheet. `body` renders the content from the source at
/// each render. `on_close` runs when the sheet closes.
pub(crate) struct InspectSourcesSheet<T: 'static> {
    pub(crate) source: Entity<T>,
    pub(crate) body: fn(&mut T, &mut Context<T>) -> AnyElement,
    pub(crate) on_close: fn(&mut T, &mut Context<T>),
}

impl<T: 'static> InspectSourcesSheet<T> {
    /// Opens the sheet over the window. A sheet that is open is replaced.
    pub(crate) fn open(self, window: &mut Window, cx: &mut App) {
        let Self {
            source,
            body,
            on_close,
        } = self;
        let view = cx.new(|cx| {
            cx.observe(&source, |_, _, cx| cx.notify()).detach();
            InspectSourcesSheetBody::new(source.clone(), body, window, cx)
        });
        let close_source = source;
        window.open_sheet(cx, move |sheet, _window, _cx| {
            let close_source = close_source.clone();
            sheet
                .title(INSPECT_SOURCES_SHEET_TITLE)
                .size(INSPECT_SOURCES_SHEET_WIDTH)
                .on_close(move |_, _, cx| {
                    close_source.update(cx, on_close);
                })
                .child(view.clone())
        });
    }
}

/// The view of the sheet body. It reads the source at each render.
///
/// The body keeps a focus handle of its own. A control of the body can lose
/// its focus when its label or its state changes, for example when Compare
/// ID3 waits for its result and the button is disabled. Then GPUI moves the
/// focus to the nearest focusable ancestor, which is this body. Thus Escape
/// still reaches the sheet (ADR 0089 Decision 8, ADR 0069).
struct InspectSourcesSheetBody<T: 'static> {
    source: Entity<T>,
    body: fn(&mut T, &mut Context<T>) -> AnyElement,
    focus: FocusHandle,
    _focus_lost: Subscription,
}

impl<T: 'static> InspectSourcesSheetBody<T> {
    fn new(
        source: Entity<T>,
        body: fn(&mut T, &mut Context<T>) -> AnyElement,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus = cx.focus_handle();
        // GPUI gives the restore target: the nearest focusable ancestor of
        // the control that lost the focus (`focus_lost_restore_target`).
        let focus_lost = cx.on_focus_lost(window, |_, window, cx| {
            if let Some(target) = window.focus_lost_restore_target(cx) {
                window.focus(&target, cx);
            }
        });
        Self {
            source,
            body,
            focus,
            _focus_lost: focus_lost,
        }
    }
}

impl<T: 'static> Render for InspectSourcesSheetBody<T> {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body = self.body;
        let content = self.source.update(cx, body);
        div().track_focus(&self.focus).child(content)
    }
}

#[cfg(test)]
mod tests {
    use gpui::prelude::*;
    use gpui::{div, AnyElement, Context, Entity, FocusHandle, IntoElement, Render, Window};

    use crate::ui::composites::action_button::{action_button, ActionButtonDisplay};

    use super::InspectSourcesSheetBody;

    /// The source of the body. While `busy`, Compare ID3 is disabled, as it
    /// is while its result loads.
    struct Source {
        busy: bool,
    }

    fn compare_body(source: &mut Source, cx: &mut Context<Source>) -> AnyElement {
        let entity = cx.entity();
        let busy = source.busy;
        let label = if busy {
            "Reading ID3..."
        } else {
            "Compare ID3"
        };
        let mut button = action_button(
            ActionButtonDisplay {
                key: "compare-id3",
                label: label.into(),
                a11y_label: "Compare ID3 tags".into(),
            },
            cx,
        )
        .disabled(busy);
        if !busy {
            button = button.on_click(move |_, _, cx| {
                entity.update(cx, |source, cx| {
                    source.busy = true;
                    cx.notify();
                });
            });
        }
        button.into_any_element()
    }

    /// A window with one sheet body, as the sheet shows it.
    struct Host {
        body: Entity<InspectSourcesSheetBody<Source>>,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().child(self.body.clone())
        }
    }

    /// ADR 0089 Decision 8 and ADR 0069: the Compare button has the focus. It
    /// becomes disabled while its result loads, and its focus must not be
    /// lost there. The body keeps the focus, so Escape still closes the sheet.
    #[gpui::test]
    fn sheet_keeps_the_focus_when_compare_is_disabled_while_it_loads(
        cx: &mut gpui::TestAppContext,
    ) {
        cx.update(gpui_component::init);
        let source = cx.update(|cx| cx.new(|_| Source { busy: false }));
        let (host, cx) = cx.add_window_view(|window, cx| {
            let body =
                cx.new(|cx| InspectSourcesSheetBody::new(source.clone(), compare_body, window, cx));
            Host { body }
        });
        let body_focus: FocusHandle = cx.update(|_, cx| host.read(cx).body.read(cx).focus.clone());
        cx.update(|window, cx| {
            window.activate_window();
            window.focus(&body_focus, cx);
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        // Tab moves the focus from the body to the Compare button.
        cx.update(|window, cx| {
            window.focus_next(cx);
            window.draw(cx).clear(cx);
        });
        assert!(
            cx.update(|window, _| !body_focus.is_focused(window)),
            "Tab moves the focus from the body to the Compare button"
        );

        cx.update(|_, cx| {
            source.update(cx, |source, cx| {
                source.busy = true;
                cx.notify();
            });
        });
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
            assert!(
                body_focus.is_focused(window),
                "the focus moves to the body when the Compare button is disabled"
            );
        });
    }
}
