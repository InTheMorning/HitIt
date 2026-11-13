//! The order of a button row with a destructive button (musicindex ADR 0064
//! R-ACT-8).
//!
//! Each view model that owns the order of a button row gives the order as
//! typed actions. A test maps each action to a [`ButtonRowRole`] and calls
//! [`adr_0064_violation`]. Only tests use this module.

#![warn(clippy::pedantic)]

/// The role of one button in a button row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ButtonRowRole {
    /// The button removes data or edits with no undo.
    Destructive,
    /// The safe or usual action of the row, for example "Cancel" or "Save".
    Safe,
    /// Each other button.
    Other,
}

/// Gives the failure text when `row` does not obey musicindex ADR 0064
/// R-ACT-8, or `None` when it does. `row` gives the roles from left to
/// right.
///
/// When a row has two or more destructive buttons and other buttons, the
/// destructive buttons are adjacent at the left end of the row. This app
/// applies R-ACT-8 item 3 to each destructive button. A row with only
/// destructive buttons fails the check: it shows one "Remove…" menu button
/// (R-ACT-10).
#[must_use]
pub(crate) fn adr_0064_violation(row: &[ButtonRowRole]) -> Option<String> {
    const FIX: &str =
        "Make it the first button of the row, and make the safe action the last button.";
    let destructive = row
        .iter()
        .filter(|role| **role == ButtonRowRole::Destructive)
        .count();
    if destructive == 0 {
        return None;
    }
    if row.last() == Some(&ButtonRowRole::Destructive) {
        return Some(format!(
            "musicindex ADR 0064 R-ACT-8: a destructive button is never the rightmost button. {FIX} Row: {row:?}"
        ));
    }
    let leading = row
        .iter()
        .take_while(|role| **role == ButtonRowRole::Destructive)
        .count();
    if leading != destructive {
        return Some(format!(
            "musicindex ADR 0064 R-ACT-8 item 3: the destructive button is the leftmost button of the row. {FIX} Row: {row:?}"
        ));
    }
    if row.contains(&ButtonRowRole::Safe) && row.last() != Some(&ButtonRowRole::Safe) {
        return Some(format!(
            "musicindex ADR 0064 R-ACT-8 item 2: the safe action is the rightmost button. {FIX} Row: {row:?}"
        ));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{adr_0064_violation, ButtonRowRole::*};

    /// The check finds each form of the defect of 2026-11-12, and accepts
    /// the order of R-ACT-8.
    #[test]
    fn adr_0064_check_finds_each_wrong_order() {
        assert_eq!(adr_0064_violation(&[Destructive, Safe]), None);
        assert_eq!(adr_0064_violation(&[Destructive, Other, Safe]), None);
        assert_eq!(adr_0064_violation(&[Destructive, Other, Other]), None);
        assert_eq!(adr_0064_violation(&[Other, Safe]), None);
        assert_eq!(adr_0064_violation(&[Destructive, Destructive, Safe]), None);
        assert!(adr_0064_violation(&[Destructive, Other, Destructive, Safe])
            .is_some_and(|text| text.contains("leftmost button")));
        assert!(adr_0064_violation(&[Destructive, Destructive])
            .is_some_and(|text| text.contains("never the rightmost button")));
        assert!(adr_0064_violation(&[Safe, Destructive])
            .is_some_and(|text| text.contains("never the rightmost button")));
        assert!(adr_0064_violation(&[Other, Destructive, Safe])
            .is_some_and(|text| text.contains("leftmost button")));
        assert!(adr_0064_violation(&[Destructive, Safe, Other])
            .is_some_and(|text| text.contains("the safe action is the rightmost button")));
    }
}
