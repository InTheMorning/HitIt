//! The narrow shell tab bar (ADR 0089 Decision 1, musicindex ADR 0027
//! R-SHELL-1 item 7).
//!
//! The bar sits at the bottom of the window when the window is narrower
//! than `SHELL_TAB_BAR_BREAKPOINT`. It draws the tabs of `TabBarVm`, then the
//! More menu. It owns no state. A tab click calls the `on_select` callback,
//! and the top app routes it.

#![warn(clippy::pedantic)]

use std::rc::Rc;

use gpui::{div, prelude::*, AnyElement, App, Window};

use crate::ui::control_styles::ControlStyle;
use crate::ui::icons::IconName;
use crate::ui::layouts::TAB_BAR_HEIGHT;
use crate::ui::primitives::{
    Button, ContextMenu, ContextMenuItem, ContextMenuItemDisplay, ContextMenuScope,
    PopoverPlacement,
};
use crate::ui::style::{color, spacing};
use crate::view_models::sidebar::{SidebarDestination, SidebarIcon, TabBarVm};

/// The element id of the More menu trigger.
const MORE_MENU_ID: &str = "shell-tab-bar-more";

/// Maps a semantic sidebar icon to a named `IconName` (ADR 0089 Decision 1).
/// The renderer owns this mapping. The sidebar and the tab bar share it.
#[must_use]
pub(crate) const fn sidebar_icon_name(icon: SidebarIcon) -> IconName {
    match icon {
        SidebarIcon::Search => IconName::Search,
        SidebarIcon::Release => IconName::Release,
        SidebarIcon::Artist => IconName::Artist,
        SidebarIcon::Playlist => IconName::Playlist,
        SidebarIcon::PublicPlaylist => IconName::PublicPlaylist,
        SidebarIcon::Live => IconName::Live,
        SidebarIcon::Warning => IconName::Warning,
        SidebarIcon::Show => IconName::Show,
        SidebarIcon::Settings => IconName::Settings,
    }
}

/// Renders the tab bar (ADR 0089 Decision 1). `feed_items` are the feed
/// commands that the More menu ends with. `on_select` runs with the
/// destination of a tab or of a menu row.
pub(crate) fn render_tab_bar(
    vm: &TabBarVm,
    feed_items: Vec<ContextMenuItem>,
    on_select: impl Fn(SidebarDestination, &mut Window, &mut App) + 'static,
) -> AnyElement {
    let on_select = Rc::new(on_select);
    let mut bar = div()
        .id("shell-tab-bar")
        .flex()
        .flex_row()
        .items_center()
        .justify_around()
        .w_full()
        .h(TAB_BAR_HEIGHT)
        .flex_shrink_0()
        .px(spacing::XS)
        .bg(color::bg_sidebar())
        .border_t_1()
        .border_color(color::border_subtle());
    for item in vm.items() {
        let destination = item.destination;
        let on_select = Rc::clone(&on_select);
        let style = if item.selected {
            ControlStyle::SelectedTab
        } else {
            ControlStyle::Unfilled
        };
        bar = bar.child(
            Button::styled(item.element_id, style)
                .leading_icon(sidebar_icon_name(item.icon))
                .label(item.label.clone())
                .a11y_label(item.a11y_label.clone())
                .on_click(move |_, window, cx| (on_select)(destination, window, cx)),
        );
    }
    let menu_items = vm
        .more_rows()
        .iter()
        .map(|row| {
            let destination = row.destination;
            let on_select = Rc::clone(&on_select);
            ContextMenuItem::new(ContextMenuItemDisplay {
                id: row.element_id.into(),
                label: row.label.into(),
                a11y_label: row.a11y_label.clone().into(),
                destructive: false,
                disabled: false,
            })
            .checked(row.selected)
            .on_select(move |window, cx| (on_select)(destination, window, cx))
        })
        .chain(feed_items);
    let more_style = if vm.more_selected() {
        ControlStyle::SelectedTab
    } else {
        ControlStyle::Unfilled
    };
    bar.child(
        ContextMenu::new(MORE_MENU_ID, ContextMenuScope::WorkspaceFrame, "More")
            .trigger_label("More")
            .trigger_icon(IconName::More)
            .trigger_style(more_style)
            .placement(PopoverPlacement::Above)
            .items(menu_items),
    )
    .into_any_element()
}
