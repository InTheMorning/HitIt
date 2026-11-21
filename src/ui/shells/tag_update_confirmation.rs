//! Shell presenter of the "Update n file(s)" button, its popup and its
//! report (ADR 0076 Decision 8).
//!
//! The view model `view_models::tag_update` supplies each text, each mark
//! and the button availability. This presenter maps them to the shared
//! button primitive and the shared confirmation composite.

#![warn(clippy::pedantic)]

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{div, prelude::*, AnyElement, App, ClickEvent, Context, SharedString, Window};

use crate::ui::composites::{
    open_confirmation_dialog_from, ConfirmationDialogCheck, ConfirmationDialogCheckedLine,
    ConfirmationDialogDisplay, ConfirmationDialogHandlers, ConfirmationDialogItem,
    ConfirmationDialogSection,
};
use crate::ui::control_styles::ControlStyle;
use crate::ui::primitives::{Button, CheckboxState, Label};
use crate::ui::tokens::{FontSize, SemanticColor, Spacing};
use crate::view_models::tag_update::{
    TagUpdateButtonDisplay, TagUpdateCheck, TagUpdateCheckDisplay, TagUpdateCheckState,
    TagUpdateFileDisplay, TagUpdateMarkRole, TagUpdatePopupDisplay, TagUpdateReportDisplay,
    TagUpdateReportRole,
};

/// Renders the "Update n file(s)" button from its typed display.
pub(crate) fn render_tag_update_button(
    display: TagUpdateButtonDisplay,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> AnyElement {
    Button::styled(display.button_id, ControlStyle::Neutral)
        .label(display.label)
        .a11y_label(display.a11y_label)
        .disabled(!display.enabled)
        .on_click(on_click)
        .into_any_element()
}

/// Renders the report of the latest confirm. Stacked text uses
/// `overflow_hidden()` and no truncation (ADR 0063).
pub(crate) fn render_tag_update_report(report: TagUpdateReportDisplay, cx: &App) -> AnyElement {
    div()
        .id("tag-update-result")
        .flex()
        .flex_col()
        .gap(Spacing::XXS.scaled(cx))
        .w_full()
        .min_w_0()
        .overflow_hidden()
        .px(Spacing::MD.scaled(cx))
        .py(Spacing::XS.scaled(cx))
        .child(
            Label::new(report.summary)
                .size(FontSize::Micro)
                .color(SemanticColor::SecondaryLabel),
        )
        .children(report.lines.into_iter().map(|line| {
            let color = match line.role {
                TagUpdateReportRole::Written => SemanticColor::SecondaryLabel,
                TagUpdateReportRole::NotWritten => SemanticColor::WarningLabel,
                TagUpdateReportRole::Failed => SemanticColor::DangerLabel,
            };
            Label::new(line.text).size(FontSize::Micro).color(color)
        }))
        .into_any_element()
}

/// Opens the popup with one confirm button and one cancel button.
///
/// `initial` is the display at the time of the open. `current` reads the
/// display again at each render of the dialog, so the check boxes show the
/// selection that `on_check` changes. The handlers change the state of `T`.
/// The shell then refreshes the window.
pub(crate) fn open_tag_update_confirmation_dialog<T>(
    window: &mut Window,
    cx: &mut Context<T>,
    initial: TagUpdatePopupDisplay,
    current: impl Fn(&T) -> Option<TagUpdatePopupDisplay> + 'static,
    handlers: TagUpdateDialogHandlers<T>,
) where
    T: 'static,
{
    let TagUpdateDialogHandlers {
        check: on_check,
        toggle_skipped: on_toggle_skipped,
        confirm: on_confirm,
        cancel: on_cancel,
    } = handlers;
    let on_check = Rc::new(on_check);
    let on_toggle_skipped = Rc::new(on_toggle_skipped);
    let cancel_entity = cx.weak_entity();
    let confirm_entity = cx.weak_entity();
    let check_entity = cx.weak_entity();
    let toggle_entity = cx.weak_entity();
    let display_entity = cx.weak_entity();
    let handlers = ConfirmationDialogHandlers::new(
        move |_, cx: &mut App| {
            let _ = cancel_entity.update(cx, |this, cx| on_cancel(this, cx));
        },
        move |_, cx: &mut App| {
            let _ = confirm_entity.update(cx, |this, cx| on_confirm(this, cx));
        },
    )
    .with_check_handler(move |box_, window, cx: &mut App| {
        let Some(check) = TagUpdateCheck::parse(&box_.id) else {
            return;
        };
        let on_check = Rc::clone(&on_check);
        let _ = check_entity.update(cx, |this, cx| {
            on_check(this, check);
            cx.notify();
        });
        window.refresh();
    })
    .with_section_toggle_handler(move |window, cx: &mut App| {
        let on_toggle_skipped = Rc::clone(&on_toggle_skipped);
        let _ = toggle_entity.update(cx, |this, cx| {
            on_toggle_skipped(this);
            cx.notify();
        });
        window.refresh();
    });
    // The dialog can render once more after the selection ends. It then
    // shows the last display.
    let last = RefCell::new(confirmation_display(initial));
    open_confirmation_dialog_from(
        window,
        cx,
        false,
        move |cx: &App| {
            let display = display_entity
                .read_with(cx, |this, _| current(this))
                .ok()
                .flatten();
            if let Some(display) = display {
                *last.borrow_mut() = confirmation_display(display);
            }
            last.borrow().clone()
        },
        handlers,
    );
}

/// The handlers of the popup. Each one changes the state of the owner `T`.
pub(crate) struct TagUpdateDialogHandlers<T> {
    pub(crate) check: Box<CheckAction<T>>,
    pub(crate) toggle_skipped: Box<StateAction<T>>,
    pub(crate) confirm: Box<ContextAction<T>>,
    pub(crate) cancel: Box<ContextAction<T>>,
}

type CheckAction<T> = dyn Fn(&mut T, TagUpdateCheck) + 'static;
type StateAction<T> = dyn Fn(&mut T) + 'static;
type ContextAction<T> = dyn Fn(&mut T, &mut Context<T>) + 'static;

fn check_state(state: TagUpdateCheckState) -> CheckboxState {
    match state {
        TagUpdateCheckState::Checked => CheckboxState::Checked,
        TagUpdateCheckState::Mixed => CheckboxState::Mixed,
        TagUpdateCheckState::NotChecked => CheckboxState::NotChecked,
    }
}

fn dialog_check(check: TagUpdateCheckDisplay) -> ConfirmationDialogCheck {
    ConfirmationDialogCheck {
        id: SharedString::from(check.id),
        state: check_state(check.state),
        a11y_label: SharedString::from(check.a11y_label),
    }
}

fn dialog_item(file: TagUpdateFileDisplay) -> ConfirmationDialogItem {
    ConfirmationDialogItem {
        title: SharedString::from(file.title),
        subtitle: file.album.map(SharedString::from),
        mark: file.mark.map(|(label, role)| {
            let color = match role {
                TagUpdateMarkRole::InUse => SemanticColor::WarningLabel,
                TagUpdateMarkRole::Unreadable => SemanticColor::DangerLabel,
            };
            (SharedString::from(label), color)
        }),
        lines: Vec::new(),
        check: file.check.map(dialog_check),
        checked_lines: file
            .lines
            .into_iter()
            .map(|line| ConfirmationDialogCheckedLine {
                text: SharedString::from(line.text),
                check: line.check.map(dialog_check),
            })
            .collect(),
    }
}

fn confirmation_display(display: TagUpdatePopupDisplay) -> ConfirmationDialogDisplay {
    ConfirmationDialogDisplay {
        title: SharedString::from(display.title),
        message: SharedString::from(display.message),
        cancel_button_id: SharedString::from(display.cancel_button_id),
        cancel_label: SharedString::from(display.cancel_label),
        cancel_a11y_label: SharedString::from(display.cancel_a11y_label),
        confirm_button_id: SharedString::from(display.confirm_button_id),
        confirm_label: SharedString::from(display.confirm_label),
        confirm_a11y_label: SharedString::from(display.confirm_a11y_label),
        destructive: false,
        confirm_enabled: display.confirm_enabled,
        form: None,
        items: display.files.into_iter().map(dialog_item).collect(),
        section: display.skipped.map(|skipped| ConfirmationDialogSection {
            id: SharedString::from(skipped.id),
            label: SharedString::from(skipped.label),
            a11y_label: SharedString::from(skipped.a11y_label),
            folded: skipped.folded,
            items: skipped.files.into_iter().map(dialog_item).collect(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_models::tag_update::{TagUpdateLineDisplay, IN_USE_MARK};

    #[test]
    fn adr_0076_tag_update_presenter_maps_files_to_dialog_items() {
        let display = TagUpdatePopupDisplay {
            title: "Update 1 file?".into(),
            message: "Message".into(),
            cancel_button_id: "cancel",
            cancel_label: "Cancel",
            cancel_a11y_label: "Close",
            confirm_button_id: "confirm",
            confirm_label: "Write Tags",
            confirm_a11y_label: "Write".into(),
            confirm_enabled: true,
            files: vec![TagUpdateFileDisplay {
                title: "Song 1".into(),
                album: Some("Album One".into()),
                lines: vec![TagUpdateLineDisplay {
                    text: "TIT2: Song 1 (file: Old title)".into(),
                    check: None,
                }],
                mark: Some((IN_USE_MARK, TagUpdateMarkRole::InUse)),
                check: None,
            }],
            skipped: None,
        };

        let dialog = confirmation_display(display);

        assert!(!dialog.destructive);
        assert_eq!(dialog.confirm_label, SharedString::from("Write Tags"));
        assert_eq!(dialog.items.len(), 1);
        let item = &dialog.items[0];
        assert_eq!(item.subtitle, Some(SharedString::from("Album One")));
        assert_eq!(
            item.mark,
            Some((SharedString::from(IN_USE_MARK), SemanticColor::WarningLabel))
        );
    }
}
