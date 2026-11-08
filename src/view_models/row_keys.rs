//! The key actions of the focused row of a list (ADR 0092).
//!
//! The row menu opens from the keyboard (Shift+F10 and the Menu key), and a
//! playlist row moves with Alt+Up and Alt+Down. This module gives the
//! availability of each key action, with its accessibility label. The
//! renderer only binds the keys and never decides whether one applies (the
//! Durable Set: typed action state).

#![warn(clippy::pedantic)]

use crate::view_models::library::{
    ContentListRowDisplay, DestinationListRowTarget, ReleasesListItemVm, ReleasesRowId,
};
use crate::view_models::release_page::{TrackRowMenuAction, TrackRowMenuVm};

/// One key action of a focused row: its availability and its accessibility
/// label (ADR 0092 Decision 4).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RowKeyAction {
    /// `true` when the key does something. A key with `false` does nothing.
    pub(crate) available: bool,
    /// The accessibility label of the action. It is empty when the action is
    /// not available.
    pub(crate) a11y_label: String,
}

impl RowKeyAction {
    /// An action that does nothing.
    #[must_use]
    pub(crate) const fn unavailable() -> Self {
        Self {
            available: false,
            a11y_label: String::new(),
        }
    }

    fn available(a11y_label: &str) -> Self {
        Self {
            available: true,
            a11y_label: a11y_label.to_owned(),
        }
    }
}

/// The key actions of the focused row of a list (ADR 0092 Decisions 2 and 3).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RowKeysVm {
    /// The id of the row menu that "open menu" opens. `None` when the row has
    /// no menu.
    pub(crate) menu_id: Option<String>,
    /// Shift+F10 and the Menu key.
    pub(crate) open_menu: RowKeyAction,
    /// Alt+Up.
    pub(crate) move_up: RowKeyAction,
    /// Alt+Down.
    pub(crate) move_down: RowKeyAction,
}

impl RowKeysVm {
    /// A row with no key action: no focused row, or a row with no menu.
    #[must_use]
    pub(crate) const fn none() -> Self {
        Self {
            menu_id: None,
            open_menu: RowKeyAction::unavailable(),
            move_up: RowKeyAction::unavailable(),
            move_down: RowKeyAction::unavailable(),
        }
    }

    /// The key actions that follow from the row menu `menu`. The menu gives
    /// "Move up" and "Move down" only when the row can move, so one source
    /// decides the keys and the menu items.
    #[must_use]
    pub(crate) fn from_menu(menu: Option<&TrackRowMenuVm>) -> Self {
        let Some(menu) = menu.filter(|menu| !menu.items.is_empty()) else {
            return Self::none();
        };
        let item_action = |action: TrackRowMenuAction| {
            menu.items
                .iter()
                .find(|item| item.action == action)
                .map_or_else(RowKeyAction::unavailable, |item| {
                    RowKeyAction::available(&item.a11y_label)
                })
        };
        Self {
            menu_id: Some(menu.id.clone()),
            open_menu: RowKeyAction::available(&menu.trigger_a11y_label),
            move_up: item_action(TrackRowMenuAction::MoveUp),
            move_down: item_action(TrackRowMenuAction::MoveDown),
        }
    }
}

/// The key actions of the focused row of the Releases list. A group row, a
/// status row and a row in selection mode have no menu, so they have no key
/// action.
#[must_use]
pub(crate) fn releases_row_keys(
    items: &[ReleasesListItemVm],
    focused: Option<&ReleasesRowId>,
    selecting: bool,
) -> RowKeysVm {
    let Some(ReleasesRowId::Release(focused_key)) = focused else {
        return RowKeysVm::none();
    };
    if selecting {
        return RowKeysVm::none();
    }
    items
        .iter()
        .find_map(|item| match item {
            ReleasesListItemVm::Release(row) | ReleasesListItemVm::Single(row) => match &row.open {
                DestinationListRowTarget::Release(key) if key == focused_key => Some(
                    RowKeysVm::from_menu(row.menu.as_ref().map(|menu| &menu.menu)),
                ),
                _ => None,
            },
            _ => None,
        })
        .unwrap_or_else(RowKeysVm::none)
}

/// The key actions of the focused row of the Inbox list. `rows` carry the
/// focus only while the list shows it.
#[must_use]
pub(crate) fn content_list_row_keys(rows: &[ContentListRowDisplay]) -> RowKeysVm {
    rows.iter()
        .find(|row| row.selection.focused)
        .map_or_else(RowKeysVm::none, |row| {
            RowKeysVm::from_menu(row.menu.as_ref())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_models::library::playlist_row_menu_with_moves;

    /// ADR 0092 Decision 2: a row with a menu opens it from the keyboard, and
    /// a row with no menu does not.
    #[test]
    fn adr_0092_open_menu_needs_a_menu_with_items() {
        let menu = playlist_row_menu_with_moves(7, "Focus", false, false);
        let keys = RowKeysVm::from_menu(Some(&menu));
        assert!(keys.open_menu.available);
        assert_eq!(keys.menu_id.as_deref(), Some("playlist-row-7-menu"));
        assert_eq!(keys.open_menu.a11y_label, "More actions for Focus");
        assert!(!keys.move_up.available && !keys.move_down.available);

        assert_eq!(RowKeysVm::from_menu(None), RowKeysVm::none());
        let mut empty = menu;
        empty.items.clear();
        assert_eq!(RowKeysVm::from_menu(Some(&empty)), RowKeysVm::none());
    }

    /// ADR 0092 Decision 3: the move keys follow the items "Move up" and
    /// "Move down" of the menu.
    #[test]
    fn adr_0092_move_keys_follow_the_menu_items() {
        let both = playlist_row_menu_with_moves(1, "A", true, true);
        let keys = RowKeysVm::from_menu(Some(&both));
        assert!(keys.move_up.available && keys.move_down.available);
        assert!(keys.move_up.a11y_label.contains("up"));
        assert!(keys.move_down.a11y_label.contains("down"));
        let up_only =
            RowKeysVm::from_menu(Some(&playlist_row_menu_with_moves(1, "A", true, false)));
        assert!(up_only.move_up.available && !up_only.move_down.available);
    }

    /// ADR 0092 Decision 2: no focused Releases row, a group row, or
    /// selection mode gives no key action.
    #[test]
    fn adr_0092_releases_without_a_focused_menu_row_has_no_keys() {
        assert_eq!(releases_row_keys(&[], None, false), RowKeysVm::none());
        assert_eq!(
            releases_row_keys(&[], Some(&ReleasesRowId::Window("w".to_owned())), false),
            RowKeysVm::none()
        );
        assert_eq!(content_list_row_keys(&[]), RowKeysVm::none());
    }
}
