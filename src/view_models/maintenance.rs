//! Page navigation and layout contracts for repair and diagnostics (ADR 0074).

#![warn(clippy::pedantic)]

use super::startup::StartupAvailability;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum MaintenanceView {
    #[default]
    Instructions,
    Report,
}

impl MaintenanceView {
    pub(crate) fn toggle(self) -> PageChoice<Self> {
        let (value, label) = match self {
            Self::Instructions => (Self::Report, "Show report"),
            Self::Report => (Self::Instructions, "Show instructions"),
        };
        PageChoice {
            value,
            label,
            a11y_label: label.into(),
            selected: false,
            availability: StartupAvailability::Available,
        }
    }
}

#[derive(Clone)]
pub(crate) struct PageChoice<T> {
    pub(crate) value: T,
    pub(crate) label: &'static str,
    pub(crate) a11y_label: String,
    pub(crate) selected: bool,
    pub(crate) availability: StartupAvailability,
}

impl<T> PageChoice<T> {
    pub(crate) fn new(value: T, label: &'static str, selected: bool) -> Self {
        Self {
            value,
            label,
            a11y_label: format!("Open {label}{}", if selected { ", selected" } else { "" }),
            selected,
            availability: StartupAvailability::Available,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MaintenanceLayout {
    Columns,
    Stacked,
}

impl MaintenanceLayout {
    pub(crate) const fn for_width(width: f32) -> Self {
        if width < 760.0 {
            Self::Stacked
        } else {
            Self::Columns
        }
    }

    /// The display sequence of the action controls of a maintenance page.
    /// `destructive` gives one flag for each control, in the sequence of the
    /// page. The result gives the indices of the controls in display
    /// sequence.
    ///
    /// In the vertical column, the destructive controls are last, as in a
    /// menu (ADR 0020 R-ACT-3 item 2). In the horizontal band, they are first
    /// (musicindex ADR 0064 R-ACT-11 and R-ACT-8). The other controls keep their sequence.
    #[must_use]
    pub(crate) fn action_sequence(self, destructive: &[bool]) -> Vec<usize> {
        let (first, last) = match self {
            Self::Columns => (false, true),
            Self::Stacked => (true, false),
        };
        let pick = |wanted: bool| {
            destructive
                .iter()
                .enumerate()
                .filter(move |(_, flag)| **flag == wanted)
                .map(|(index, _)| index)
        };
        pick(first).chain(pick(last)).collect()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum RecoveryPage {
    #[default]
    Startup,
    Configuration,
    Database,
}

impl RecoveryPage {
    pub(crate) const MENU_LABEL: &'static str = "Choose recovery page";

    pub(crate) fn choices(self) -> Vec<PageChoice<Self>> {
        [
            (Self::Startup, "Startup"),
            (Self::Configuration, "Configuration"),
            (Self::Database, "Database"),
        ]
        .into_iter()
        .map(|(page, label)| PageChoice::new(page, label, self == page))
        .collect()
    }
}
