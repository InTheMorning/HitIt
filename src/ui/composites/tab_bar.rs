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
                // The tab bar follows the list and the status bar (ADR 0092
                // task 004, ADR 0101 task 008).
                .tab_index(2)
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
            .trigger_tab_index(2)
            .placement(PopoverPlacement::Above)
            .items(menu_items),
    )
    .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::composites::status_bar::{status_bar, StatusBarSlots};
    use crate::view_models::sidebar::{SidebarCounts, SidebarVm};
    use crate::view_models::status_bar::{
        StatusBarDisplay, StatusItemDetail, StatusItemDisplay, StatusItemIcon, StatusItemKind,
    };

    struct TabOrderRoot {
        page_control: gpui::FocusHandle,
        list: gpui::FocusHandle,
    }

    impl gpui::Render for TabOrderRoot {
        fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
            let sidebar =
                SidebarVm::new(SidebarCounts::default(), Some(SidebarDestination::Releases));
            let vm = TabBarVm::new(&sidebar);
            let status_item = StatusItemDisplay {
                kind: StatusItemKind::Files,
                id: "status-item-files",
                icon: StatusItemIcon::Files,
                word: "Update 1 file".into(),
                a11y_label: "Update 1 file".into(),
                detail: StatusItemDetail::TagUpdatePopup,
                enabled: true,
            };
            div()
                .size_full()
                .flex()
                .flex_col()
                .child(
                    div()
                        .flex_1()
                        .track_focus(&self.list.clone().tab_index(1).tab_stop(true))
                        .child(
                            Button::plain("page-control")
                                .label("In Library")
                                .track_focus(&self.page_control)
                                .on_click(|_, _, _| {}),
                        )
                        .child(div().id("list-row").child("One track")),
                )
                .child(status_bar(
                    StatusBarDisplay {
                        id: "status-bar",
                        event: None,
                        items: vec![status_item],
                    },
                    StatusBarSlots::new().on_open_item(|_, _, _| {}),
                ))
                .child(render_tab_bar(&vm, Vec::new(), |_, _, _| {}))
        }
    }

    #[gpui::test]
    fn adr_0092_task_004_the_list_comes_before_the_tab_bar(cx: &mut gpui::TestAppContext) {
        cx.update(gpui_component::init);
        let (root, cx) = cx.add_window_view(|_, cx| TabOrderRoot {
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
        assert!(cx.update(|window, _| list.is_focused(window)));
        cx.update(gpui::Window::focus_next);
        assert_eq!(
            cx.update(|window, cx| window.focused(cx).map(|focus| focus.tab_index)),
            Some(1),
            "Tab reaches a status bar item after the list"
        );
        cx.update(gpui::Window::focus_next);
        assert_eq!(
            cx.update(|window, cx| window.focused(cx).map(|focus| focus.tab_index)),
            Some(2),
            "Tab reaches the first tab bar control after the status bar"
        );
    }
}
