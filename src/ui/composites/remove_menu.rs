//! The "Remove…" button of a row that has two or more destructive actions
//! and no other button (musicindex ADR 0064 R-ACT-10).
//!
//! A row with only destructive buttons cannot obey R-ACT-8. So the row
//! shows one button "Remove…" that opens a menu, and the menu follows ADR
//! 0020 R-ACT-3 item 2. This is not the row menu of ADR 0101 Decision 6.

#![warn(clippy::pedantic)]

use std::rc::Rc;

use gpui::{App, SharedString, Window};

use crate::ui::control_styles::ControlStyle;
use crate::ui::icons::IconName;
use crate::ui::primitives::{
    ContextMenu, ContextMenuItem, ContextMenuItemDisplay, ContextMenuScope,
};

/// Runs one item of the menu.
pub(crate) type RemoveMenuHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

/// One destructive item of the "Remove…" menu. The screen supplies
/// display-ready text.
pub(crate) struct RemoveMenuItem {
    pub(crate) id: SharedString,
    pub(crate) label: SharedString,
    pub(crate) a11y_label: SharedString,
    pub(crate) disabled: bool,
    pub(crate) on_select: Option<RemoveMenuHandler>,
}

/// The "Remove…" button and its menu. The items show from top to bottom in
/// the sequence of `items`.
pub(crate) fn remove_menu(
    id: impl Into<SharedString>,
    label: impl Into<SharedString>,
    a11y_label: impl Into<SharedString>,
    items: impl IntoIterator<Item = RemoveMenuItem>,
) -> ContextMenu {
    ContextMenu::new(id, ContextMenuScope::TrackList, a11y_label)
        .trigger_label(label)
        .trigger_style(ControlStyle::DestructiveRowAction)
        .trigger_icon(IconName::ChevronDown)
        .items(items.into_iter().map(|item| {
            let entry = ContextMenuItem::new(ContextMenuItemDisplay {
                id: item.id,
                label: item.label,
                a11y_label: item.a11y_label,
                destructive: true,
                disabled: item.disabled,
            });
            match item.on_select {
                Some(handler) if !item.disabled => {
                    entry.on_select(move |window, cx| handler(window, cx))
                }
                _ => entry,
            }
        }))
}
