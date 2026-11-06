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
    AnyElement, App, AppContext, Context, Entity, IntoElement, ParentElement, Render, Window,
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
            InspectSourcesSheetBody {
                source: source.clone(),
                body,
            }
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
struct InspectSourcesSheetBody<T: 'static> {
    source: Entity<T>,
    body: fn(&mut T, &mut Context<T>) -> AnyElement,
}

impl<T: 'static> Render for InspectSourcesSheetBody<T> {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body = self.body;
        self.source.update(cx, body)
    }
}
