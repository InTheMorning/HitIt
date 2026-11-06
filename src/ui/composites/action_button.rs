//! Metadata-action button composite.
//!
//! This is the first consumer of the ADR 0025 control-style boundary. Call
//! sites still receive a chainable native button, while the visual role maps
//! through [`crate::ui::control_styles::ControlStyle`].
//!
//! Accessibility note (ADR 0038 task 005): [`ActionButtonDisplay`] carries an
//! `a11y_label` sourced from a view-model. GPUI 0.2.x does not yet expose an
//! accessibility-label sink on its button widget, so the field is contract-only
//! today. When the framework grows the surface, the wiring is one-line — the
//! label is already present at the call site.

#![warn(clippy::pedantic)]

use gpui::{App, SharedString};

use crate::ui::control_styles::ControlStyle;
use crate::ui::primitives::Button;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionButtonDisplay {
    /// The stable identity of the button. It keeps the focus when the
    /// label changes, for example "Compare ID3" to "Reading ID3...".
    pub key: &'static str,
    pub label: SharedString,
    pub a11y_label: SharedString,
}

/// The element id of a metadata-action button. It names the key, never the
/// label, so the button keeps its identity when its label changes (ADR 0089
/// Decision 8).
#[must_use]
fn action_button_id(key: &str) -> SharedString {
    SharedString::from(format!("metadata-action:{key}"))
}

/// Build a metadata-action button. It has the neutral fill. A command that writes
/// data replaces the role with `ControlStyle::CompactAction` (musicindex ADR 0042).
pub fn action_button(display: ActionButtonDisplay, _cx: &App) -> Button {
    let ActionButtonDisplay {
        key,
        label,
        a11y_label,
    } = display;
    Button::styled(action_button_id(key), ControlStyle::MetadataAction)
        .label(label)
        .a11y_label(a11y_label)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_contract_carries_button_label() {
        let display = ActionButtonDisplay {
            key: "apply-id3-edits",
            label: SharedString::from("Apply"),
            a11y_label: SharedString::from("Apply metadata changes"),
        };

        assert_eq!(display.label, SharedString::from("Apply"));
        assert_eq!(
            display.a11y_label,
            SharedString::from("Apply metadata changes")
        );
    }

    #[test]
    fn display_contract_carries_a11y_label_distinct_from_visible_label() {
        let display = ActionButtonDisplay {
            key: "remove-feed",
            label: SharedString::from("Remove"),
            a11y_label: SharedString::from("Remove feed from library"),
        };

        assert_ne!(display.label, display.a11y_label);
    }

    /// ADR 0089 Decision 8: the id of a button names its key, not its label.
    /// A label change keeps the focus on the same button.
    #[test]
    fn button_id_ignores_the_label() {
        let before = ActionButtonDisplay {
            key: "compare-id3",
            label: SharedString::from("Compare ID3"),
            a11y_label: SharedString::from("Compare ID3 tags"),
        };
        let after = ActionButtonDisplay {
            label: SharedString::from("Reading ID3..."),
            ..before.clone()
        };

        assert_eq!(action_button_id(before.key), action_button_id(after.key));
        assert_ne!(
            action_button_id("compare-id3"),
            action_button_id("apply-id3-edits")
        );
    }
}
