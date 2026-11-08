//! The action bar of a Releases or Tracks list selection (ADR 0084
//! Decision 10, amended 2026-10-19 for the selection mode).
//!
//! The bar shows only in selection mode: the screen calls
//! [`SelectionActionBar::new`] only when [`SelectionActionBarVm::new`]
//! gave `Some`. With no selected row, the bar carries only its count
//! label text, "Select items", and no action button. Each button
//! renders from the typed action the view model already decided; this
//! composite invents no availability of its own.

#![warn(clippy::pedantic)]

use std::rc::Rc;

use gpui::{
    div, prelude::*, AnyElement, App, Entity, IntoElement, RenderOnce, ScrollHandle, SharedString,
    Window,
};

use crate::ui::composites::list_page_head::{
    reverse_toggle, sort_menu, view_icons, ListReverseHandler, ListSortHandler, ListViewHandler,
};
use crate::ui::composites::music_column_scroll::ListBarPin;
use crate::ui::composites::playlist_popover::{
    AddToPlaylistDisplay, AddToPlaylistPopover, PlaylistOption,
};
use crate::ui::control_styles::ControlStyle;
use crate::ui::layouts::OVERLAY_SCROLLBAR_WIDTH;
use crate::ui::primitives::{Button as UiButton, Label};
use crate::ui::style::spacing;
use crate::ui::tokens::{color, FontSize, SemanticColor, Spacing};
use crate::view_models::detail_selection::ListBarVm;
use crate::view_models::list_page::ListBarControlsVm;
use crate::view_models::selection_action_bar::{
    ListBarCommand, SelectionActionBarVm, SelectionActionKind,
};

type RunHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;
type SelectHandler = Rc<dyn Fn(&i64, &mut Window, &mut App) + 'static>;
type CreateHandler = Rc<dyn Fn(&String, &mut Window, &mut App) + 'static>;

/// Display-ready input for [`SelectionActionBar`]'s "Add to playlist"
/// popover.
pub(crate) struct SelectionActionBarPlaylists {
    pub(crate) playlists: Vec<PlaylistOption>,
    pub(crate) new_playlist_a11y_label: SharedString,
    pub(crate) back_a11y_label: SharedString,
    pub(crate) create_a11y_label: SharedString,
}

/// The action bar at the bottom of a Releases or Tracks list with a
/// selection (ADR 0084 Decision 10).
#[derive(IntoElement)]
#[must_use]
pub(crate) struct SelectionActionBar {
    bar: SelectionActionBarVm,
    playlists: SelectionActionBarPlaylists,
    on_download: Option<RunHandler>,
    on_remove: Option<RunHandler>,
    on_clear: Option<RunHandler>,
    on_select_all: Option<RunHandler>,
    on_add_to_playlist: Option<SelectHandler>,
    on_create_playlist: Option<CreateHandler>,
    on_remove_from_playlist: Option<RunHandler>,
    on_move_to_top: Option<RunHandler>,
    on_move_to_bottom: Option<RunHandler>,
}

impl SelectionActionBar {
    pub(crate) fn new(bar: SelectionActionBarVm, playlists: SelectionActionBarPlaylists) -> Self {
        Self {
            bar,
            playlists,
            on_download: None,
            on_remove: None,
            on_clear: None,
            on_select_all: None,
            on_add_to_playlist: None,
            on_create_playlist: None,
            on_remove_from_playlist: None,
            on_move_to_top: None,
            on_move_to_bottom: None,
        }
    }

    /// "Remove n from playlist" of the playlist page (packet 025).
    pub(crate) fn on_remove_from_playlist(
        mut self,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_remove_from_playlist = Some(Rc::new(handler));
        self
    }

    /// "Move to top" of the playlist page (packet 025).
    pub(crate) fn on_move_to_top(
        mut self,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_move_to_top = Some(Rc::new(handler));
        self
    }

    /// "Move to bottom" of the playlist page (packet 025).
    pub(crate) fn on_move_to_bottom(
        mut self,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_move_to_bottom = Some(Rc::new(handler));
        self
    }

    /// "Select all" of each bar (ADR 0084 Decision 10).
    pub(crate) fn on_select_all(
        mut self,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select_all = Some(Rc::new(handler));
        self
    }

    pub(crate) fn on_download(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_download = Some(Rc::new(handler));
        self
    }

    pub(crate) fn on_remove(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_remove = Some(Rc::new(handler));
        self
    }

    pub(crate) fn on_clear(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_clear = Some(Rc::new(handler));
        self
    }

    pub(crate) fn on_add_to_playlist(
        mut self,
        handler: impl Fn(&i64, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_add_to_playlist = Some(Rc::new(handler));
        self
    }

    pub(crate) fn on_create_playlist(
        mut self,
        handler: impl Fn(&String, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_create_playlist = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for SelectionActionBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let Self {
            bar,
            playlists,
            on_download,
            on_remove,
            on_clear,
            on_add_to_playlist,
            on_create_playlist,
            on_remove_from_playlist,
            on_move_to_top,
            on_move_to_bottom,
            on_select_all,
        } = self;

        let mut children: Vec<AnyElement> = vec![Label::new(bar.count_label)
            .size(FontSize::Caption)
            .color(SemanticColor::SecondaryLabel)
            .into_any_element()];

        for action in bar.actions {
            let (style, handler) = match action.kind {
                SelectionActionKind::AddToPlaylist => {
                    let mut popover = AddToPlaylistPopover::new(AddToPlaylistDisplay {
                        id: SharedString::from(action.element_id),
                        playlists: playlists.playlists.clone(),
                        trigger_label: SharedString::from(action.label),
                        trigger_a11y_label: SharedString::from(action.a11y_label),
                        new_playlist_a11y_label: playlists.new_playlist_a11y_label.clone(),
                        back_a11y_label: playlists.back_a11y_label.clone(),
                        create_a11y_label: playlists.create_a11y_label.clone(),
                    });
                    if let Some(handler) = on_add_to_playlist.clone() {
                        popover = popover.on_select(move |playlist_id, window, cx| {
                            handler(playlist_id, window, cx);
                        });
                    }
                    if let Some(handler) = on_create_playlist.clone() {
                        popover =
                            popover.on_create(move |name, window, cx| handler(name, window, cx));
                    }
                    children.push(popover.into_any_element());
                    continue;
                }
                SelectionActionKind::Download => (ControlStyle::Secondary, on_download.clone()),
                SelectionActionKind::Remove => (ControlStyle::Destructive, on_remove.clone()),
                SelectionActionKind::RemoveFromPlaylist => {
                    (ControlStyle::Destructive, on_remove_from_playlist.clone())
                }
                SelectionActionKind::MoveToTop => (ControlStyle::Secondary, on_move_to_top.clone()),
                SelectionActionKind::MoveToBottom => {
                    (ControlStyle::Secondary, on_move_to_bottom.clone())
                }
                SelectionActionKind::SelectAll => (ControlStyle::Neutral, on_select_all.clone()),
                SelectionActionKind::Clear => (ControlStyle::Neutral, on_clear.clone()),
            };
            children.push(action_button(
                action.element_id,
                action.label,
                action.a11y_label,
                style,
                handler,
            ));
        }

        div()
            .id("music-selection-action-bar")
            .flex()
            .flex_row()
            .flex_shrink_0()
            .flex_wrap()
            .items_center()
            .gap(Spacing::SM.scaled(cx))
            .px(Spacing::MD.scaled(cx))
            .py(Spacing::SM.scaled(cx))
            .border_t_1()
            .border_color(color(cx, SemanticColor::Separator))
            .bg(color(cx, SemanticColor::SecondarySystemBackground))
            .children(children)
    }
}

fn action_button(
    element_id: &'static str,
    label: String,
    a11y_label: String,
    style: ControlStyle,
    handler: Option<RunHandler>,
) -> AnyElement {
    UiButton::styled(element_id, style)
        .label(label)
        .a11y_label(a11y_label)
        .on_click(move |_, window, cx| {
            if let Some(handler) = &handler {
                handler(window, cx);
            }
        })
        .into_any_element()
}

/// The handler of the list bar of a group page. It receives each command
/// of the bar (ADR 0090).
pub(crate) type ListBarHandler = Rc<dyn Fn(ListBarCommand, &mut Window, &mut App) + 'static>;

/// The playlist choices and the labels of the "Add to playlist" popover of
/// a list bar.
pub(crate) fn list_bar_playlists(playlists: Vec<PlaylistOption>) -> SelectionActionBarPlaylists {
    SelectionActionBarPlaylists {
        playlists,
        new_playlist_a11y_label: SharedString::from("Create a new playlist"),
        back_a11y_label: SharedString::from("Back to playlist choices"),
        create_a11y_label: SharedString::from("Create playlist and add the selection"),
    }
}

/// Which copy of the list bar a renderer draws (ADR 0090 Decision 5).
///
/// The in-flow copy sits above the rows, and it is the tab stop of each
/// control. The pinned copy overlays the top of the rows while the in-flow
/// copy has scrolled away. It has its own element ids, and no control of it
/// is a tab stop, so only one "Select" or "Done" is a tab stop at a time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ListBarCopy {
    InFlow,
    Pinned,
}

impl ListBarCopy {
    /// The element id of a control or the bar, unique to this copy.
    fn element_id(self, id: &str) -> SharedString {
        match self {
            Self::InFlow => SharedString::from(id.to_owned()),
            Self::Pinned => SharedString::from(format!("pinned-{id}")),
        }
    }

    /// Applies this copy's tab rule to a control.
    fn tab_stop_button(self, button: UiButton) -> UiButton {
        match self {
            Self::InFlow => button,
            Self::Pinned => button.out_of_tab_sequence(),
        }
    }
}

/// Renders the list bar above the rows of a group page (ADR 0090). The
/// mode action is at the left, then the count and the selection actions.
/// The bar has the selection background only while selecting. Each control
/// renders from the typed action of `bar`. Each click sends its command to
/// `on_command`. `copy` gives the ids and the tab rule of this copy.
pub(crate) fn render_list_bar(
    bar: ListBarVm,
    playlists: &SelectionActionBarPlaylists,
    on_command: &ListBarHandler,
    copy: ListBarCopy,
    cx: &App,
) -> AnyElement {
    let ListBarVm {
        mode_action,
        selection,
        controls,
    } = bar;
    let selecting = selection.is_some();

    let mode_command = on_command.clone();
    let mode_button = UiButton::styled(copy.element_id(&mode_action.id), ControlStyle::Neutral)
        .label(mode_action.label)
        .a11y_label(mode_action.a11y_label)
        .on_click(move |_, window, cx| mode_command(ListBarCommand::Toggle, window, cx));
    let mut children: Vec<AnyElement> = vec![copy.tab_stop_button(mode_button).into_any_element()];

    if let Some(selection) = selection {
        children.push(
            Label::new(selection.count_label)
                .size(FontSize::Caption)
                .color(SemanticColor::SecondaryLabel)
                .into_any_element(),
        );
        for action in selection.actions {
            let (style, command) = match action.kind {
                SelectionActionKind::AddToPlaylist => {
                    let mut popover = AddToPlaylistPopover::new(AddToPlaylistDisplay {
                        id: copy.element_id(action.element_id),
                        playlists: playlists.playlists.clone(),
                        trigger_label: SharedString::from(action.label),
                        trigger_a11y_label: SharedString::from(action.a11y_label),
                        new_playlist_a11y_label: playlists.new_playlist_a11y_label.clone(),
                        back_a11y_label: playlists.back_a11y_label.clone(),
                        create_a11y_label: playlists.create_a11y_label.clone(),
                    });
                    let select_command = on_command.clone();
                    popover = popover.on_select(move |playlist_id, window, cx| {
                        select_command(ListBarCommand::AddToPlaylist(*playlist_id), window, cx);
                    });
                    let create_command = on_command.clone();
                    popover = popover.on_create(move |name, window, cx| {
                        create_command(ListBarCommand::CreatePlaylist(name.clone()), window, cx);
                    });
                    if copy == ListBarCopy::Pinned {
                        popover = popover.trigger_out_of_tab_sequence();
                    }
                    children.push(popover.into_any_element());
                    continue;
                }
                SelectionActionKind::Download
                | SelectionActionKind::MoveToTop
                | SelectionActionKind::MoveToBottom => {
                    (ControlStyle::Secondary, ListBarCommand::Run(action.kind))
                }
                SelectionActionKind::Remove | SelectionActionKind::RemoveFromPlaylist => {
                    (ControlStyle::Destructive, ListBarCommand::Run(action.kind))
                }
                SelectionActionKind::SelectAll | SelectionActionKind::Clear => {
                    (ControlStyle::Neutral, ListBarCommand::Run(action.kind))
                }
            };
            let command_handler = on_command.clone();
            let action_button = UiButton::styled(copy.element_id(action.element_id), style)
                .label(action.label)
                .a11y_label(action.a11y_label)
                .on_click(move |_, window, cx| {
                    command_handler(command.clone(), window, cx);
                });
            children.push(copy.tab_stop_button(action_button).into_any_element());
        }
    }

    // ADR 0090 Decisions 2, 3 and 6: the left group holds the mode action
    // and the selection actions. The right group holds the sort menu, the
    // view icons and the Reverse toggle, and it stays in selection mode. In a
    // narrow window each group wraps, and the two ends stay on separate
    // lines.
    let left = div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .min_w_0()
        .gap(Spacing::SM.scaled(cx))
        .children(children);
    let mut bar = div()
        .id(copy.element_id("music-list-bar").to_string())
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(Spacing::SM.scaled(cx))
        .py(Spacing::SM.scaled(cx))
        .child(left);
    if let Some(right) = render_list_bar_controls(controls, on_command, copy, cx) {
        bar = bar.child(right);
    }
    if selecting {
        bar = bar.bg(color(cx, SemanticColor::SecondarySystemBackground));
    }
    bar.into_any_element()
}

/// The right end of the list bar: the sort menu and the view icons, or the
/// Reverse toggle of a list with one fixed order (musicindex ADR 0054
/// R-LIST-13 and R-LIST-14, ADR 0053 R-LIST-9). Each control comes from the typed state of `controls`. `None`
/// when the bar has no control. The renderers are those of the list page
/// head.
fn render_list_bar_controls(
    controls: Option<ListBarControlsVm>,
    on_command: &ListBarHandler,
    copy: ListBarCopy,
    cx: &App,
) -> Option<AnyElement> {
    let ListBarControlsVm {
        sort,
        view,
        reverse_toggle: toggle,
    } = controls?;
    let pinned = copy == ListBarCopy::Pinned;
    let mut right = div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .ml_auto()
        .gap(Spacing::XS.scaled(cx));
    if let Some(sort) = sort {
        let command = on_command.clone();
        let handler: ListSortHandler = Rc::new(move |sort, reversed, window, cx| {
            command(ListBarCommand::Sort(*sort, reversed), window, cx);
        });
        right = right.child(sort_menu(sort, Some(&handler), pinned));
    }
    if let Some(view) = view {
        let command = on_command.clone();
        let handler: ListViewHandler = Rc::new(move |view, window, cx| {
            command(ListBarCommand::View(*view), window, cx);
        });
        right = right.child(view_icons(view, Some(&handler), pinned, cx));
    }
    if let Some(toggle) = toggle {
        let command = on_command.clone();
        let handler: ListReverseHandler = Rc::new(move |reversed, window, cx| {
            command(ListBarCommand::Reverse(reversed), window, cx);
        });
        right = right.child(reverse_toggle(&toggle, Some(&handler), pinned));
    }
    Some(right.into_any_element())
}

/// The in-flow bar and the pinned copy of a list bar (ADR 0090 Decision 5).
/// The pinned copy is `Some` only once the in-flow bar has scrolled above
/// the viewport, and it overlays the rows. Each host passes its own
/// `on_command`, so each command reaches the one path of that host. `entity`
/// is the host that redraws when the pinned state changes.
pub(crate) fn render_list_bar_copies<T: 'static>(
    list_bar: Option<ListBarVm>,
    playlists: &SelectionActionBarPlaylists,
    on_command: &ListBarHandler,
    pin: &ListBarPin,
    scroll: &ScrollHandle,
    entity: &Entity<T>,
    cx: &App,
) -> (Option<AnyElement>, Option<AnyElement>) {
    let pinned = list_bar
        .as_ref()
        .filter(|_| pin.is_pinned(scroll))
        .map(|bar| render_pinned_list_bar(bar.clone(), playlists, on_command, cx));
    let in_flow = list_bar
        .map(|bar| render_in_flow_list_bar(bar, playlists, on_command, pin, scroll, entity, cx));
    (in_flow, pinned)
}

/// The list bar above the rows. It carries the probe that records its
/// position for the pinned copy (ADR 0090 Decision 5).
fn render_in_flow_list_bar<T: 'static>(
    bar: ListBarVm,
    playlists: &SelectionActionBarPlaylists,
    on_command: &ListBarHandler,
    pin: &ListBarPin,
    scroll: &ScrollHandle,
    entity: &Entity<T>,
    cx: &App,
) -> AnyElement {
    let bar_element = render_list_bar(bar, playlists, on_command, ListBarCopy::InFlow, cx);
    let probe = pin.probe(scroll, entity);
    div()
        .relative()
        .child(bar_element)
        .child(probe)
        .into_any_element()
}

/// The pinned copy of a list bar (ADR 0090 Decision 5). It overlays the top
/// of the rows, so it has a background of its own. Its insets match the
/// in-flow bar, so the bar sits in the same columns.
fn render_pinned_list_bar(
    bar: ListBarVm,
    playlists: &SelectionActionBarPlaylists,
    on_command: &ListBarHandler,
    cx: &App,
) -> AnyElement {
    let bar_element = render_list_bar(bar, playlists, on_command, ListBarCopy::Pinned, cx);
    // The rows pass below this copy. A click on its empty parts must not
    // reach a row that it hides (operator run of 2026-11-06).
    div()
        .occlude()
        .absolute()
        .top_0()
        .left_0()
        .w_full()
        .pt(spacing::LG)
        .pl(spacing::LG)
        .pr(spacing::LG + OVERLAY_SCROLLBAR_WIDTH + Spacing::SM.scaled(cx))
        .bg(color(cx, SemanticColor::SystemBackground))
        .child(bar_element)
        .into_any_element()
}
