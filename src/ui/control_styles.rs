//! Semantic control-style roles mapped onto native button primitives.
//!
//! Screens choose product intent (`MetadataAction`, `RowAction`, ...). The
//! design system owns the concrete native button variant, size, typography,
//! border, and token choices. The shape of a button is not a role choice:
//! the primitive owns it (musicindex ADR 0020 R-ACT-2).

#![warn(clippy::pedantic)]

use crate::ui::primitives::{ButtonSize, ButtonVariant};
use crate::ui::tokens::{FontSize, SemanticColor};

/// Reusable button/action roles admitted by ADR 0025.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlStyle {
    Primary,
    /// The role Action of musicindex ADR 0042 R-ACT-4: the command adds or
    /// changes something.
    Secondary,
    /// The command changes no stored data (R-ACT-4 item 1).
    Neutral,
    /// A text label with no fill: a letter of the letter bar, an item of the
    /// narrow tab bar, or a link in text (R-ACT-6 item 2).
    Unfilled,
    /// `Secondary` at the size of a row.
    CompactAction,
    /// Back and Forward: a circle with the neutral fill (musicindex ADR 0044
    /// R-NAV-4).
    NeutralIcon,
    Destructive,
    ToolbarIcon,
    RowAction,
    DestructiveRowAction,
    MetadataAction,
    /// A selected tab, segment or letter. It has a tint and an outline, and
    /// no fill (musicindex ADR 0020 R-ACT-1 item 3).
    SelectedTab,
}

/// Pure style mapping consumed by the native [`crate::ui::primitives::Button`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlStyleSpec {
    pub variant: ButtonVariant,
    pub size: ButtonSize,
    pub font_size: FontSize,
    pub foreground: Option<SemanticColor>,
    pub border: Option<SemanticColor>,
}

impl ControlStyle {
    /// Return the pure role-to-token mapping for this control style.
    #[must_use]
    pub const fn spec(self) -> ControlStyleSpec {
        match self {
            Self::Primary => ControlStyleSpec {
                variant: ButtonVariant::Filled,
                size: ButtonSize::Md,
                font_size: FontSize::Body,
                foreground: None,
                border: None,
            },
            Self::Secondary => ControlStyleSpec {
                variant: ButtonVariant::Tinted,
                size: ButtonSize::Md,
                font_size: FontSize::Body,
                foreground: None,
                border: None,
            },
            // The primitive gives `Plain` with a text label the neutral fill.
            Self::Neutral | Self::Unfilled => ControlStyleSpec {
                variant: ButtonVariant::Plain,
                size: ButtonSize::Md,
                font_size: FontSize::Body,
                foreground: None,
                border: None,
            },
            Self::NeutralIcon => ControlStyleSpec {
                variant: ButtonVariant::Neutral,
                size: ButtonSize::Sm,
                font_size: FontSize::Caption,
                foreground: None,
                border: None,
            },
            Self::CompactAction => ControlStyleSpec {
                variant: ButtonVariant::Tinted,
                size: ButtonSize::Sm,
                font_size: FontSize::Caption,
                foreground: None,
                border: None,
            },
            Self::Destructive => ControlStyleSpec {
                variant: ButtonVariant::Destructive,
                size: ButtonSize::Md,
                font_size: FontSize::Body,
                foreground: None,
                border: None,
            },
            Self::ToolbarIcon | Self::RowAction => ControlStyleSpec {
                variant: ButtonVariant::Plain,
                size: ButtonSize::Sm,
                font_size: FontSize::Caption,
                foreground: None,
                border: None,
            },
            Self::DestructiveRowAction => ControlStyleSpec {
                variant: ButtonVariant::Plain,
                size: ButtonSize::Sm,
                font_size: FontSize::Caption,
                foreground: Some(SemanticColor::DangerLabel),
                border: None,
            },
            Self::MetadataAction => ControlStyleSpec {
                variant: ButtonVariant::Plain,
                size: ButtonSize::Sm,
                font_size: FontSize::Micro,
                foreground: None,
                border: None,
            },
            Self::SelectedTab => ControlStyleSpec {
                variant: ButtonVariant::Tinted,
                size: ButtonSize::Md,
                font_size: FontSize::Body,
                foreground: None,
                border: Some(SemanticColor::AccentOutline),
            },
        }
    }

    /// The role of a command by its result (musicindex ADR 0042 R-ACT-4 item 3).
    #[must_use]
    pub const fn by_result(changes_stored_data: bool) -> Self {
        if changes_stored_data {
            Self::Secondary
        } else {
            Self::Neutral
        }
    }

    /// Whether this compact action role should expose hover help by default.
    #[must_use]
    pub const fn prefers_tooltip(self) -> bool {
        matches!(
            self,
            Self::ToolbarIcon | Self::NeutralIcon | Self::RowAction | Self::DestructiveRowAction
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_action_is_compact_plain_button() {
        let spec = ControlStyle::MetadataAction.spec();
        assert_eq!(spec.variant, ButtonVariant::Plain);
        assert_eq!(spec.size, ButtonSize::Sm);
        assert_eq!(spec.font_size, FontSize::Micro);
        assert!(spec.foreground.is_none());
        assert!(spec.border.is_none());
    }

    #[test]
    fn destructive_uses_native_destructive_variant() {
        let spec = ControlStyle::Destructive.spec();
        assert_eq!(spec.variant, ButtonVariant::Destructive);
        assert!(spec.border.is_none());
        assert!(spec.foreground.is_none());
    }

    #[test]
    fn toolbar_and_row_actions_share_compact_plain_mapping() {
        let toolbar = ControlStyle::ToolbarIcon.spec();
        let row = ControlStyle::RowAction.spec();
        assert_eq!(toolbar, row);
        assert_eq!(toolbar.variant, ButtonVariant::Plain);
        assert_eq!(toolbar.size, ButtonSize::Sm);
    }

    #[test]
    fn destructive_row_action_is_compact_plain_danger_text() {
        let spec = ControlStyle::DestructiveRowAction.spec();
        assert_eq!(spec.variant, ButtonVariant::Plain);
        assert_eq!(spec.size, ButtonSize::Sm);
        assert_eq!(spec.font_size, FontSize::Caption);
        assert_eq!(spec.foreground, Some(SemanticColor::DangerLabel));
        assert!(spec.border.is_none());
    }

    #[test]
    fn compact_action_roles_prefer_tooltips() {
        assert!(ControlStyle::ToolbarIcon.prefers_tooltip());
        assert!(ControlStyle::RowAction.prefers_tooltip());
        assert!(ControlStyle::DestructiveRowAction.prefers_tooltip());
        assert!(!ControlStyle::Primary.prefers_tooltip());
        assert!(!ControlStyle::Neutral.prefers_tooltip());
    }

    /// musicindex ADR 0044 R-NAV-4: Back and Forward have the neutral fill.
    #[test]
    fn neutral_icon_has_the_neutral_fill() {
        let spec = ControlStyle::NeutralIcon.spec();
        assert_eq!(spec.variant, ButtonVariant::Neutral);
        assert_eq!(spec.size, ButtonSize::Sm);
        assert!(spec.foreground.is_none());
    }

    /// musicindex ADR 0020 R-ACT-1 item 3: a selected tab has a tint and an
    /// outline, and no fill.
    #[test]
    fn selected_tab_has_a_tint_and_an_outline() {
        let spec = ControlStyle::SelectedTab.spec();

        assert_eq!(spec.variant, ButtonVariant::Tinted);
        assert_eq!(spec.border, Some(SemanticColor::AccentOutline));
    }
}
