//! The order, the tiles and the drag of My playlists (ADR 0089 amendment of
//! 2026-11-08, musicindex ADR 0053 R-LIST-9 row "Playlists that a person
//! curates").
//!
//! My playlists has the sorts Curator Order, A to Z and Newest. Curator
//! Order follows the stored `position` of each playlist, and the curator
//! sets it. A to Z reads the name in the language of the reader
//! (`compare_names`). Newest reads `created_at`, so an edit does not make a
//! playlist new. A reversed entry of the sort menu reverses the shown sequence of each sort.
//!
//! Only the sort Curator Order, not reversed, in the Rows view, gives a
//! playlist row a drag and the menu items Move Up and Move Down. This module
//! is pure: the renderer only composes the parts.

#![warn(clippy::pedantic)]

use crate::db;
use crate::view_models::collection_art::CollectionArtVm;
use crate::view_models::library::{PlaylistSidebarRowVm, PlaylistSidebarVm, RowTypeTag};
use crate::view_models::list_page::{ListSort, ListView};
use crate::view_models::name_order::compare_names;
use crate::view_models::row_keys::RowKeysVm;
use crate::view_models::workspace::FrameNavigationEntry;

/// The word between the first and the second part of a tile line.
const SECOND_LINE_SEPARATOR: &str = " \u{b7} ";

/// The playlists in the shown order of the sort `sort`, reversed when
/// `reversed`. A tie falls to the name, then to the id, so the order is
/// stable. A sort that My playlists does not offer reads as Curator Order.
#[must_use]
pub(crate) fn ordered_playlists(
    playlists: &[db::Playlist],
    sort: ListSort,
    reversed: bool,
) -> Vec<&db::Playlist> {
    let by_name = |a: &db::Playlist, b: &db::Playlist| {
        compare_names(&a.name, &b.name).then_with(|| a.id.cmp(&b.id))
    };
    let mut ordered: Vec<&db::Playlist> = playlists.iter().collect();
    ordered.sort_by(|a, b| match sort {
        ListSort::AToZ => by_name(a, b),
        ListSort::Newest => b.created_at.cmp(&a.created_at).then_with(|| by_name(a, b)),
        _ => a.position.cmp(&b.position).then_with(|| by_name(a, b)),
    });
    if reversed {
        ordered.reverse();
    }
    ordered
}

/// Whether a playlist row can be dragged or moved: only with the sort
/// Curator Order, not reversed, in the Rows view (ADR 0089 amendment of
/// 2026-11-08).
#[must_use]
pub(crate) fn can_reorder(sort: ListSort, reversed: bool, view: ListView) -> bool {
    sort == ListSort::CuratorOrder && !reversed && view == ListView::Rows
}

/// One move of a playlist by a key (ADR 0092 Decision 3): the playlist and
/// the place in the shown list that it moves to. The playlist keeps the
/// keyboard cursor, so the cursor follows the playlist and not the place.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlaylistKeyMove {
    /// The playlist that moves, and that keeps the cursor.
    pub(crate) playlist_id: i64,
    /// The place that the playlist moves to, from 0.
    pub(crate) to_index: usize,
}

impl PlaylistSidebarVm {
    /// Puts the keyboard cursor on the row `focused`, when `shows` is true
    /// (ADR 0092 Decision 1). A list that does not have the keyboard focus
    /// shows no cursor.
    #[must_use]
    pub(crate) fn with_focus(mut self, focused: Option<i64>, shows: bool) -> Self {
        for row in &mut self.rows {
            row.focused = shows && focused == Some(row.id);
        }
        self
    }

    /// The key actions of the row `focused`: the availability of the menu key
    /// and of the move keys (ADR 0092 Decisions 2 and 3). The row menu gives
    /// "Move up" and "Move down" under the same rule as the drag, so the keys
    /// and the menu share one source.
    #[must_use]
    pub(crate) fn row_keys(&self, focused: Option<i64>) -> RowKeysVm {
        let row = focused.and_then(|id| self.rows.iter().find(|row| row.id == id));
        RowKeysVm::from_menu(row.map(|row| &row.menu))
    }

    /// The move that Alt+Up (`up` true) or Alt+Down makes on the row
    /// `focused`. `None` when the key does nothing.
    #[must_use]
    pub(crate) fn key_move(&self, focused: Option<i64>, up: bool) -> Option<PlaylistKeyMove> {
        let keys = self.row_keys(focused);
        let action = if up { &keys.move_up } else { &keys.move_down };
        if !action.available {
            return None;
        }
        let row = self.rows.iter().find(|row| Some(row.id) == focused)?;
        let to_index = if up {
            row.shown_index.checked_sub(1)?
        } else {
            row.shown_index.checked_add(1)?
        };
        Some(PlaylistKeyMove {
            playlist_id: row.id,
            to_index,
        })
    }
}

/// The drag fact of a playlist row. A row with this fact starts a drag, and
/// the drop on another row moves the playlist to the place of that row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlaylistRowDragVm {
    /// The playlist that the drag moves.
    pub(crate) playlist_id: i64,
    /// The text of the drag preview: the name of the playlist.
    pub(crate) label: String,
    /// The accessibility label of the drag.
    pub(crate) a11y_label: String,
}

impl PlaylistRowDragVm {
    /// The drag of the playlist `playlist_id` named `name`.
    #[must_use]
    pub(crate) fn new(playlist_id: i64, name: &str) -> Self {
        Self {
            playlist_id,
            label: name.to_owned(),
            a11y_label: format!("Drag the playlist \"{name}\" to a new place in the list"),
        }
    }
}

/// One tile of My playlists (musicindex ADR 0053 R-LIST-9). The tile has the
/// art of the row, its name as the first line, and its type word and track
/// count as the second line.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlaylistTileVm {
    /// Stable element id. It is the element id of the row.
    pub(crate) element_id: String,
    /// The first line: the name of the playlist.
    pub(crate) label: String,
    /// The second line: the type word and the track count.
    pub(crate) second_line: String,
    /// The accessibility label of the tile. It is the label of the row.
    pub(crate) a11y_label: String,
    /// The art of the row (ADR 0084 Decision 12).
    pub(crate) art: CollectionArtVm,
    /// The entry that a click opens.
    pub(crate) open: FrameNavigationEntry,
    /// `true` when the tile opens the page that Back left open (R-NAV-2
    /// item 2).
    pub(crate) marked: bool,
    /// `true` when the tile opens the page that is selected now.
    pub(crate) selected: bool,
}

impl PlaylistTileVm {
    /// The tile of one playlist `row`.
    #[must_use]
    pub(crate) fn of(row: &PlaylistSidebarRowVm) -> Self {
        let RowTypeTag { word, .. } = &row.type_tag;
        Self {
            element_id: row.element_id.clone(),
            label: row.name.clone(),
            second_line: [word.as_str(), row.track_count_label.as_str()]
                .join(SECOND_LINE_SEPARATOR),
            a11y_label: format!("Playlist: {}", row.name),
            art: row.art.clone(),
            open: FrameNavigationEntry::PlaylistDetail(row.id),
            marked: row.marked,
            selected: row.selected,
        }
    }
}

impl PlaylistSidebarVm {
    /// The tile of each row, in the shown order.
    #[must_use]
    pub(crate) fn tiles(&self) -> Vec<PlaylistTileVm> {
        self.rows.iter().map(PlaylistTileVm::of).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_models::collection_art::CollectionKind;

    fn playlist(id: i64, name: &str, position: i64, created_at: i64) -> db::Playlist {
        db::Playlist {
            id,
            name: name.to_owned(),
            description: None,
            track_count: 0,
            created_at,
            updated_at: 0,
            position,
        }
    }

    fn names(rows: &[&db::Playlist]) -> Vec<String> {
        rows.iter().map(|row| row.name.clone()).collect()
    }

    fn sample() -> Vec<db::Playlist> {
        vec![
            playlist(1, "mango", 2, 10),
            playlist(2, "Apple", 0, 30),
            playlist(3, "zebra", 1, 20),
            playlist(4, "banana", 3, 30),
        ]
    }

    /// ADR 0089 amendment of 2026-11-08: Curator Order follows `position`.
    #[test]
    fn adr_0053_r9_curated_curator_order_follows_position() {
        let playlists = sample();
        let shown = ordered_playlists(&playlists, ListSort::CuratorOrder, false);
        assert_eq!(names(&shown), ["Apple", "zebra", "mango", "banana"]);
        let shown = ordered_playlists(&playlists, ListSort::CuratorOrder, true);
        assert_eq!(names(&shown), ["banana", "mango", "zebra", "Apple"]);
    }

    /// A to Z uses `compare_names`, so case does not decide, and Z to A is
    /// the reverse of it.
    #[test]
    fn adr_0053_r9_curated_a_to_z_uses_compare_names() {
        let playlists = sample();
        let shown = ordered_playlists(&playlists, ListSort::AToZ, false);
        assert_eq!(names(&shown), ["Apple", "banana", "mango", "zebra"]);
        let shown = ordered_playlists(&playlists, ListSort::AToZ, true);
        assert_eq!(names(&shown), ["zebra", "mango", "banana", "Apple"]);
    }

    /// Newest reads `created_at`, newest first, and a tie falls to the name.
    /// Oldest is the reverse of it.
    #[test]
    fn adr_0053_r9_curated_newest_reads_created_at() {
        let playlists = sample();
        let shown = ordered_playlists(&playlists, ListSort::Newest, false);
        assert_eq!(names(&shown), ["Apple", "banana", "zebra", "mango"]);
        let shown = ordered_playlists(&playlists, ListSort::Newest, true);
        assert_eq!(names(&shown), ["mango", "zebra", "banana", "Apple"]);
    }

    /// Only Curator Order, not reversed, in Rows reorders.
    #[test]
    fn adr_0053_r9_curated_drag_exists_only_in_curator_order_not_reversed_rows() {
        for sort in [ListSort::CuratorOrder, ListSort::AToZ, ListSort::Newest] {
            for reversed in [false, true] {
                for view in [ListView::Rows, ListView::Tiles] {
                    assert_eq!(
                        can_reorder(sort, reversed, view),
                        sort == ListSort::CuratorOrder && !reversed && view == ListView::Rows,
                        "{sort:?} reversed {reversed} {view:?}"
                    );
                }
            }
        }
        let drag = PlaylistRowDragVm::new(7, "Focus");
        assert_eq!(drag.playlist_id, 7);
        assert_eq!(drag.label, "Focus");
        assert!(drag.a11y_label.contains("Focus"));
    }

    /// The sidebar gives the drag and the Move Up and Move Down items only in
    /// Curator Order, not reversed, in the Rows view. The first row has no
    /// Move Up, and the last row has no Move Down.
    #[test]
    fn adr_0053_r9_curated_sidebar_drag_only_in_curator_order_not_reversed_rows() {
        use crate::view_models::library::LibraryViewModel;
        use crate::view_models::list_page::ListPageKey;
        use crate::view_models::release_page::TrackRowMenuAction;
        let key = ListPageKey::MyPlaylists;
        let mut vm = LibraryViewModel::new();
        vm.replace_playlists(sample());

        let sidebar = vm.playlist_sidebar();
        let shown: Vec<&str> = sidebar.rows.iter().map(|row| row.name.as_str()).collect();
        assert_eq!(shown, ["Apple", "zebra", "mango", "banana"]);
        assert!(sidebar.rows.iter().all(|row| row.drag.is_some()));
        let places: Vec<usize> = sidebar.rows.iter().map(|row| row.shown_index).collect();
        assert_eq!(places, [0, 1, 2, 3]);
        let moves = |row: &PlaylistSidebarRowVm| -> Vec<TrackRowMenuAction> {
            row.menu.items.iter().map(|item| item.action).collect()
        };
        assert!(!moves(&sidebar.rows[0]).contains(&TrackRowMenuAction::MoveUp));
        assert!(moves(&sidebar.rows[0]).contains(&TrackRowMenuAction::MoveDown));
        assert!(moves(&sidebar.rows[1]).contains(&TrackRowMenuAction::MoveUp));
        assert!(!moves(&sidebar.rows[3]).contains(&TrackRowMenuAction::MoveDown));
        let drag = sidebar.rows[1].drag.as_ref().expect("drag");
        assert_eq!(drag.playlist_id, 3);
        assert_eq!(drag.label, "zebra");

        let has_no_move = |sidebar: &PlaylistSidebarVm| {
            sidebar.rows.iter().all(|row| {
                row.drag.is_none()
                    && !moves(row).contains(&TrackRowMenuAction::MoveUp)
                    && !moves(row).contains(&TrackRowMenuAction::MoveDown)
            })
        };
        vm.set_list_reversed(key, true);
        assert!(has_no_move(&vm.playlist_sidebar()), "reversed");
        vm.set_list_reversed(key, false);
        for sort in [ListSort::AToZ, ListSort::Newest] {
            vm.set_list_sort(key, sort);
            assert!(has_no_move(&vm.playlist_sidebar()), "{sort:?}");
        }
        vm.set_list_sort(key, ListSort::CuratorOrder);
        vm.set_list_view(key, ListView::Tiles);
        assert!(has_no_move(&vm.playlist_sidebar()), "tiles");
        assert_eq!(vm.playlist_sidebar().tiles().len(), 4);
    }

    /// ADR 0092 Decisions 2 and 3: the row keys of each condition. The first
    /// row has no Move up, the last row has no Move down, and A to Z, Newest,
    /// reversed and Tiles give no move key. The menu key stays for each
    /// condition that shows a row with a menu.
    #[test]
    fn adr_0092_playlist_row_keys_follow_the_condition() {
        use crate::view_models::library::LibraryViewModel;
        use crate::view_models::list_page::ListPageKey;
        let key = ListPageKey::MyPlaylists;
        let mut vm = LibraryViewModel::new();
        vm.replace_playlists(sample());
        let sidebar = vm.playlist_sidebar();
        let ids: Vec<i64> = sidebar.rows.iter().map(|row| row.id).collect();
        assert_eq!(ids, [2, 3, 1, 4]);

        let first = sidebar.row_keys(Some(2));
        assert!(first.open_menu.available);
        assert!(!first.move_up.available && first.move_down.available);
        let middle = sidebar.row_keys(Some(3));
        assert!(middle.move_up.available && middle.move_down.available);
        let last = sidebar.row_keys(Some(4));
        assert!(last.move_up.available && !last.move_down.available);
        assert_eq!(sidebar.row_keys(None), RowKeysVm::none());
        assert_eq!(sidebar.row_keys(Some(99)), RowKeysVm::none());

        let has_no_move_key = |sidebar: &PlaylistSidebarVm| {
            ids.iter().all(|id| {
                let keys = sidebar.row_keys(Some(*id));
                keys.open_menu.available
                    && !keys.move_up.available
                    && !keys.move_down.available
                    && sidebar.key_move(Some(*id), true).is_none()
                    && sidebar.key_move(Some(*id), false).is_none()
            })
        };
        vm.set_list_reversed(key, true);
        assert!(has_no_move_key(&vm.playlist_sidebar()), "reversed");
        vm.set_list_reversed(key, false);
        for sort in [ListSort::AToZ, ListSort::Newest] {
            vm.set_list_sort(key, sort);
            assert!(has_no_move_key(&vm.playlist_sidebar()), "{sort:?}");
        }
        vm.set_list_sort(key, ListSort::CuratorOrder);
        vm.set_list_view(key, ListView::Tiles);
        assert!(has_no_move_key(&vm.playlist_sidebar()), "tiles");
    }

    /// ADR 0092 Decision 3: a key move names the moved playlist, so the
    /// cursor stays on it, and it gives the place of the neighbor row. The
    /// first row does not move up, and the last row does not move down.
    #[test]
    fn adr_0092_key_move_targets_the_moved_playlist() {
        use crate::view_models::library::LibraryViewModel;
        let mut vm = LibraryViewModel::new();
        vm.replace_playlists(sample());
        let sidebar = vm.playlist_sidebar();
        assert_eq!(sidebar.key_move(Some(2), true), None);
        assert_eq!(
            sidebar.key_move(Some(2), false),
            Some(PlaylistKeyMove {
                playlist_id: 2,
                to_index: 1
            })
        );
        assert_eq!(
            sidebar.key_move(Some(1), true),
            Some(PlaylistKeyMove {
                playlist_id: 1,
                to_index: 1
            })
        );
        assert_eq!(sidebar.key_move(Some(4), false), None);
        assert_eq!(sidebar.key_move(None, true), None);
    }

    /// ADR 0092 Decision 1: the cursor shows on the focused row, and only
    /// while the list shows it.
    #[test]
    fn adr_0092_cursor_shows_only_with_the_list_focus() {
        use crate::view_models::library::LibraryViewModel;
        let mut vm = LibraryViewModel::new();
        vm.replace_playlists(sample());
        let shown = vm.playlist_sidebar().with_focus(Some(3), true);
        let focused: Vec<i64> = shown
            .rows
            .iter()
            .filter(|row| row.focused)
            .map(|row| row.id)
            .collect();
        assert_eq!(focused, [3]);
        let hidden = vm.playlist_sidebar().with_focus(Some(3), false);
        assert!(hidden.rows.iter().all(|row| !row.focused));
    }

    /// A tile carries the art of its row and opens the playlist page.
    #[test]
    fn adr_0053_r9_curated_tile_carries_the_row_art_and_open_target() {
        let row = PlaylistSidebarRowVm {
            id: 7,
            element_id: "playlist-7".to_owned(),
            name: "Focus".to_owned(),
            track_count_label: "3 tracks".to_owned(),
            type_tag: RowTypeTag {
                kind: crate::view_models::library::RowTypeKind::Playlist,
                word: "Playlist".to_owned(),
            },
            selected: false,
            art: CollectionArtVm::resolve(
                None,
                std::iter::empty::<&str>(),
                CollectionKind::Playlist,
            ),
            marked: true,
            focused: false,
            menu: crate::view_models::library::playlist_row_menu_with_moves(
                7, "Focus", false, false,
            ),
            delete_confirmation: crate::view_models::library::playlist_delete_confirmation(
                "Focus", 3,
            ),
            drag: None,
            shown_index: 0,
        };
        let tile = PlaylistTileVm::of(&row);
        assert_eq!(tile.art, row.art);
        assert_eq!(tile.open, FrameNavigationEntry::PlaylistDetail(7));
        assert_eq!(tile.label, "Focus");
        assert_eq!(tile.second_line, "Playlist \u{b7} 3 tracks");
        assert_eq!(tile.a11y_label, "Playlist: Focus");
        assert!(tile.marked);
        assert_eq!(tile.element_id, row.element_id);
    }
}
