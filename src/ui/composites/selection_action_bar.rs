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

use gpui::{div, prelude::*, AnyElement, App, IntoElement, RenderOnce, SharedString, Window};

use crate::ui::composites::playlist_popover::{
    AddToPlaylistDisplay, AddToPlaylistPopover, PlaylistOption,
};
use crate::ui::control_styles::ControlStyle;
use crate::ui::primitives::{Button as UiButton, Label};
use crate::ui::tokens::{color, FontSize, SemanticColor, Spacing};
use crate::view_models::detail_selection::ListBarVm;
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

/// Renders the list bar above the rows of a group page (ADR 0090). The
/// mode action is at the left, then the count and the selection actions.
/// The bar has the selection background only while selecting. Each control
/// renders from the typed action of `bar`. Each click sends its command to
/// `on_command`.
pub(crate) fn render_list_bar(
    bar: ListBarVm,
    playlists: &SelectionActionBarPlaylists,
    on_command: &ListBarHandler,
    cx: &App,
) -> AnyElement {
    let ListBarVm {
        mode_action,
        selection,
    } = bar;
    let selecting = selection.is_some();

    let mode_command = on_command.clone();
    let mut children: Vec<AnyElement> =
        vec![
            UiButton::styled(SharedString::from(mode_action.id), ControlStyle::Neutral)
                .label(mode_action.label)
                .a11y_label(mode_action.a11y_label)
                .on_click(move |_, window, cx| mode_command(ListBarCommand::Toggle, window, cx))
                .into_any_element(),
        ];

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
                        id: SharedString::from(action.element_id),
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
            children.push(
                UiButton::styled(SharedString::from(action.element_id), style)
                    .label(action.label)
                    .a11y_label(action.a11y_label)
                    .on_click(move |_, window, cx| {
                        command_handler(command.clone(), window, cx);
                    })
                    .into_any_element(),
            );
        }
    }

    let mut bar = div()
        .id("music-list-bar")
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(Spacing::SM.scaled(cx))
        .py(Spacing::SM.scaled(cx))
        .children(children);
    if selecting {
        bar = bar.bg(color(cx, SemanticColor::SecondarySystemBackground));
    }
    bar.into_any_element()
}
