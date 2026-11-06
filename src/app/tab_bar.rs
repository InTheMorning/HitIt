//! The narrow shell tab bar of the top app (ADR 0089 Decision 1).
//!
//! Below `SHELL_TAB_BAR_BREAKPOINT` the window shows this bar in place of
//! the sidebar. The bar builds its view model from the sidebar rows, and each
//! tab or menu row routes through `TopApp::open_sidebar_row`, the same route
//! as a sidebar click. The sidebar view keeps its state while the bar shows.

#![warn(clippy::pedantic)]

use gpui::{AnyElement, Context};

use crate::ui::composites::tab_bar::render_tab_bar;
use crate::view_models::sidebar::TabBarVm;

use super::TopApp;

impl TopApp {
    /// Builds the narrow tab bar (ADR 0089 Decision 1). The More menu ends
    /// with the feed update command of the sidebar feed area.
    pub(super) fn render_narrow_tab_bar(&self, cx: &mut Context<Self>) -> AnyElement {
        let vm = TabBarVm::new(&self.library.read(cx).sidebar_vm());
        let feed_items = self
            .library
            .update(cx, |library, cx| library.feed_update_menu_items(cx));
        let top = cx.entity();
        render_tab_bar(&vm, feed_items, move |destination, window, cx| {
            top.update(cx, |app, cx| {
                app.open_sidebar_row(destination, window, cx);
            });
        })
    }
}
