//! Frame chrome display contracts for workspace shells.

#![warn(clippy::pedantic)]

use serde::{Deserialize, Serialize};

use super::frame::{WorkspaceFrameId, WorkspaceFrameState};
use crate::view_models::index_name_list::MusicScopeControlDisplay;

/// Presentation mode for content rows.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ContentViewMode {
    /// Tiled artwork-first browser.
    #[default]
    Tiles,
    /// Compact row list.
    List,
}

impl ContentViewMode {
    /// Returns the visible segment label.
    #[must_use]
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Tiles => "Tiles",
            Self::List => "List",
        }
    }

    /// Returns the stable segment id suffix.
    #[must_use]
    pub(crate) const fn id_suffix(self) -> &'static str {
        match self {
            Self::Tiles => "tiles",
            Self::List => "list",
        }
    }

    const fn content_list_a11y_label(self) -> &'static str {
        match self {
            Self::Tiles => "Show Music content as tiles",
            Self::List => "Show Music content as a list",
        }
    }
}

/// Display data for one content view-mode segment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ContentViewModeOptionDisplay {
    /// Presentation mode selected by the segment.
    pub(crate) mode: ContentViewMode,
    /// Stable segment identifier.
    pub(crate) id: &'static str,
    /// Visible segment label.
    pub(crate) label: &'static str,
    /// Accessibility label for assistive technologies and tooltips.
    pub(crate) a11y_label: &'static str,
}

/// Display contract for selecting a content view mode.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ContentViewModeControlDisplay {
    /// Stable control identifier.
    pub(crate) id: &'static str,
    /// Currently selected mode.
    pub(crate) selected: ContentViewMode,
    /// Ordered mode options.
    pub(crate) options: [ContentViewModeOptionDisplay; 2],
}

impl ContentViewModeControlDisplay {
    const CONTENT_LIST_ID: &'static str = "workspace-content-list-view-mode";
    const CONTENT_LIST_TILES_ID: &'static str = "workspace-content-list-view-mode-tiles";
    const CONTENT_LIST_LIST_ID: &'static str = "workspace-content-list-view-mode-list";

    /// Creates the default content-list view-mode display.
    #[must_use]
    pub(crate) const fn default_for_content_list(selected: ContentViewMode) -> Self {
        Self {
            id: Self::CONTENT_LIST_ID,
            selected,
            options: [
                Self::content_list_option(ContentViewMode::Tiles),
                Self::content_list_option(ContentViewMode::List),
            ],
        }
    }

    const fn content_list_option(mode: ContentViewMode) -> ContentViewModeOptionDisplay {
        ContentViewModeOptionDisplay {
            mode,
            id: match mode {
                ContentViewMode::Tiles => Self::CONTENT_LIST_TILES_ID,
                ContentViewMode::List => Self::CONTENT_LIST_LIST_ID,
            },
            label: mode.label(),
            a11y_label: mode.content_list_a11y_label(),
        }
    }
}

/// Display contract for one frame-chrome button.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FrameChromeButtonDisplay {
    /// Stable element or command identifier for the button.
    pub(crate) id: String,
    /// Accessibility label for assistive technologies and tooltips.
    pub(crate) a11y_label: &'static str,
    /// Whether the command should render unavailable.
    pub(crate) disabled: bool,
}

impl FrameChromeButtonDisplay {
    /// Creates a display contract for a frame-chrome button.
    #[must_use]
    pub(crate) fn new(id: impl Into<String>, a11y_label: &'static str, disabled: bool) -> Self {
        Self {
            id: id.into(),
            a11y_label,
            disabled,
        }
    }
}

/// Display contract for the frame shell's trailing header action (ADR
/// 0084 Decision 10 Amendment, 2026-10-19). The Music list column uses
/// it for "Select" and "Done". The shell renders the given label and
/// accessibility label; it decides neither.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FrameHeaderActionDisplay {
    /// Stable element identifier for the action.
    pub(crate) id: String,
    /// Visible button label, for example "Select" or "Done".
    pub(crate) label: &'static str,
    /// Accessibility label for assistive technologies and tooltips.
    pub(crate) a11y_label: String,
}

impl FrameHeaderActionDisplay {
    /// Creates a display contract for a frame-shell header action.
    #[must_use]
    pub(crate) fn new(
        id: impl Into<String>,
        label: &'static str,
        a11y_label: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label,
            a11y_label: a11y_label.into(),
        }
    }
}

/// Display contract for one frame-chrome menu item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FrameChromeMenuItemDisplay {
    /// Stable element or command identifier for the menu item.
    pub(crate) id: String,
    /// Visible menu label.
    pub(crate) label: &'static str,
    /// Accessibility label for the menu item.
    pub(crate) a11y_label: &'static str,
    /// Whether the command should render unavailable.
    pub(crate) disabled: bool,
}

/// Display contract consumed by the shared workspace frame shell.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FrameShellDisplay {
    /// Frame identifier represented by this shell.
    pub(crate) frame_id: WorkspaceFrameId,
    /// Primary frame title.
    pub(crate) title: String,
    /// Optional secondary frame context.
    pub(crate) subtitle: Option<String>,
    /// Optional trailing status text.
    pub(crate) status: Option<String>,
    /// Optional close command display.
    pub(crate) close: Option<FrameChromeButtonDisplay>,
    /// Additional frame action menu items.
    pub(crate) action_menu_items: Vec<FrameChromeMenuItemDisplay>,
    /// Optional trailing header action (ADR 0084 Decision 10 Amendment).
    /// The Music list column's "Select" and "Done" button is the first
    /// caller.
    pub(crate) header_action: Option<FrameHeaderActionDisplay>,
    /// Optional scope control of the Music Releases or Artists list (ADR
    /// 0084 Decision 8, packet 016 decision D1). It sits at the leading
    /// edge of the filter row.
    pub(crate) scope_control: Option<MusicScopeControlDisplay>,
    /// Stable content slot identifier for mounting frame body content.
    pub(crate) content_slot_id: String,
}

impl FrameShellDisplay {
    const CLOSE_A11Y_LABEL: &'static str = "Close frame";

    /// Projects frame state into shell chrome display data. Back, Forward
    /// and the page title are not frame chrome (ADR 0089 Decision 9).
    #[must_use]
    pub(crate) fn from_frame(frame: &WorkspaceFrameState, allow_close: bool) -> Self {
        let frame_id = frame.id();
        Self {
            frame_id,
            title: frame.title().to_string(),
            subtitle: frame.subtitle().map(ToOwned::to_owned),
            status: frame.status().map(ToOwned::to_owned),
            close: allow_close.then(|| {
                FrameChromeButtonDisplay::new(
                    format!("workspace-frame-{}-close", frame_id.value()),
                    Self::CLOSE_A11Y_LABEL,
                    false,
                )
            }),
            action_menu_items: Vec::new(),
            header_action: None,
            scope_control: None,
            content_slot_id: format!("workspace-frame-{}-content", frame_id.value()),
        }
    }

    /// Returns this shell display with a trailing header action attached
    /// (ADR 0084 Decision 10 Amendment).
    #[must_use]
    pub(crate) fn with_header_action(mut self, display: FrameHeaderActionDisplay) -> Self {
        self.header_action = Some(display);
        self
    }

    /// Returns this shell display with the scope control of a Music list
    /// attached (ADR 0084 packet 016).
    #[must_use]
    pub(crate) const fn with_scope_control(mut self, display: MusicScopeControlDisplay) -> Self {
        self.scope_control = Some(display);
        self
    }

    /// Returns whether the frame header row carries visible information.
    #[must_use]
    pub(crate) fn header_visible(&self) -> bool {
        !self.title.is_empty()
            || self.subtitle.is_some()
            || self.status.is_some()
            || self.close.is_some()
            || !self.action_menu_items.is_empty()
            || self.header_action.is_some()
    }
}
