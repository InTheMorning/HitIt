//! Shell presenter of the confirmation of "Use Defaults…" in Settings.
//!
//! "Use Defaults…" resets and saves the General and Library settings with no
//! undo, so it is destructive (musicindex ADR 0064 R-ACT-12). The view model
//! `view_models::settings` supplies each text. Only the confirm handler
//! resets the settings.

#![warn(clippy::pedantic)]

use gpui::{App, Context, SharedString, Window};

use crate::ui::composites::{
    open_confirmation_dialog, ConfirmationDialogDisplay, ConfirmationDialogHandlers,
};
use crate::view_models::settings::UseDefaultsConfirmation;

/// Opens the confirmation. `on_confirm` runs only after a click on the
/// confirm button.
pub(crate) fn open_use_defaults_dialog<T>(
    window: &mut Window,
    cx: &mut Context<T>,
    display: UseDefaultsConfirmation,
    on_confirm: impl Fn(&mut T, &mut Window, &mut Context<T>) + 'static,
) where
    T: 'static,
{
    let entity = cx.weak_entity();
    let handlers = ConfirmationDialogHandlers::new(
        |_, _: &mut App| {},
        move |window, cx: &mut App| {
            let _ = entity.update(cx, |this, cx| on_confirm(this, window, cx));
        },
    );
    open_confirmation_dialog(window, cx, confirmation_display(display), handlers);
}

fn confirmation_display(display: UseDefaultsConfirmation) -> ConfirmationDialogDisplay {
    ConfirmationDialogDisplay {
        title: SharedString::from(display.title),
        message: SharedString::from(display.message),
        cancel_button_id: SharedString::from(display.cancel_button_id),
        cancel_label: SharedString::from(display.cancel_label),
        cancel_a11y_label: SharedString::from(display.cancel_a11y_label),
        confirm_button_id: SharedString::from(display.confirm_button_id),
        confirm_label: SharedString::from(display.confirm_label),
        confirm_a11y_label: SharedString::from(display.confirm_a11y_label),
        destructive: true,
        items: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::confirmation_display;
    use crate::view_models::settings::use_defaults_confirmation;

    /// musicindex ADR 0064 R-ACT-12: the
    /// confirmation of "Use Defaults…" is destructive. So the shared dialog
    /// puts "Use Defaults" on the left and gives "Cancel" the first focus
    /// (musicindex ADR 0064 R-ACT-8 and R-ACT-9).
    #[test]
    fn adr_0064_use_defaults_confirmation_is_destructive() {
        let display = confirmation_display(use_defaults_confirmation());
        assert!(
            display.destructive,
            "musicindex ADR 0064 R-ACT-12: the Use Defaults confirmation is destructive. Set `destructive: true`."
        );
        assert_eq!(display.confirm_label.as_ref(), "Use Defaults");
        assert_eq!(display.cancel_label.as_ref(), "Cancel");
    }
}
