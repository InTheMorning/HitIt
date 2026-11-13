//! Confirmation dialog composite for focused, content-affecting choices.
//!
//! The composite owns the app's dialog body contract: token-driven text,
//! shared button primitives, explicit Cancel affordance, and no domain logic.
//! Screens provide display-ready strings and callbacks.
//!
//! The composite also owns the order of the button row and the keyboard of a
//! destructive confirmation (musicindex ADR 0064 R-ACT-8 and R-ACT-9).

#![warn(clippy::pedantic)]

use std::rc::Rc;

use gpui::{div, prelude::*, App, FocusHandle, FontWeight, SharedString, Window};
use gpui_component::{dialog::Dialog, WindowExt};

use crate::ui::control_styles::ControlStyle;
use crate::ui::primitives::{Button, ButtonSize, Label, LabelVariant};
use crate::ui::tokens::{SemanticColor, Size, Spacing};

type DialogActionHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfirmationDialogDisplay {
    pub title: SharedString,
    pub message: SharedString,
    pub cancel_button_id: SharedString,
    pub cancel_label: SharedString,
    pub cancel_a11y_label: SharedString,
    pub confirm_button_id: SharedString,
    pub confirm_label: SharedString,
    pub confirm_a11y_label: SharedString,
    pub destructive: bool,
    /// The items that the choice affects. The dialog lists them in a
    /// scrolling column below the message. An empty list shows no column.
    pub items: Vec<ConfirmationDialogItem>,
}

/// One affected item of a confirmation. The screen supplies display-ready
/// text. The mark text states its meaning, so its color is never the only
/// signal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfirmationDialogItem {
    pub title: SharedString,
    pub subtitle: Option<SharedString>,
    pub mark: Option<(SharedString, SemanticColor)>,
    pub lines: Vec<SharedString>,
}

#[derive(Clone)]
pub struct ConfirmationDialogHandlers {
    on_cancel: DialogActionHandler,
    on_confirm: DialogActionHandler,
}

impl ConfirmationDialogHandlers {
    pub fn new(
        on_cancel: impl Fn(&mut Window, &mut App) + 'static,
        on_confirm: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            on_cancel: Rc::new(on_cancel),
            on_confirm: Rc::new(on_confirm),
        }
    }
}

/// One button of the confirmation row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfirmationButton {
    /// The safe action. It closes the dialog with no change.
    Cancel,
    /// The action that the dialog asks about.
    Confirm,
}

/// The order and the first focus of the confirmation row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConfirmationButtonRow {
    /// The buttons from left to right.
    pub order: [ConfirmationButton; 2],
    /// A larger space follows the first button (musicindex ADR 0064 R-ACT-8
    /// item 4).
    pub space_after_first: bool,
    /// The button that has the focus when the dialog opens (musicindex ADR
    /// 0064 R-ACT-9 item 1).
    pub first_focus: Option<ConfirmationButton>,
}

/// Gives the button row of a confirmation.
///
/// A destructive confirmation puts the destructive button first, a larger
/// space after it, and Cancel last. Cancel has the first focus (musicindex
/// ADR 0064 R-ACT-8 and R-ACT-9). Each other confirmation puts Cancel first
/// and the primary action last.
#[must_use]
pub const fn confirmation_button_row(destructive: bool) -> ConfirmationButtonRow {
    if destructive {
        ConfirmationButtonRow {
            order: [ConfirmationButton::Confirm, ConfirmationButton::Cancel],
            space_after_first: true,
            first_focus: Some(ConfirmationButton::Cancel),
        }
    } else {
        ConfirmationButtonRow {
            order: [ConfirmationButton::Cancel, ConfirmationButton::Confirm],
            space_after_first: false,
            first_focus: None,
        }
    }
}

/// Opens a confirmation dialog.
///
/// Escape and Enter with no focused button run the safe action. When the
/// row gives a first focus, that button has the focus after the dialog
/// opens (musicindex ADR 0064 R-ACT-9).
pub fn open_confirmation_dialog(
    window: &mut Window,
    cx: &mut App,
    display: ConfirmationDialogDisplay,
    handlers: ConfirmationDialogHandlers,
) {
    open_confirmation_dialog_with_focus(window, cx, display, handlers);
}

/// Opens the dialog and gives the focus handle of Cancel.
fn open_confirmation_dialog_with_focus(
    window: &mut Window,
    cx: &mut App,
    display: ConfirmationDialogDisplay,
    handlers: ConfirmationDialogHandlers,
) -> FocusHandle {
    let row = confirmation_button_row(display.destructive);
    let cancel_focus = cx.focus_handle();
    let dialog_focus = cancel_focus.clone();
    window.open_dialog(cx, move |dialog, _window, cx| {
        confirmation_dialog(dialog, display.clone(), handlers.clone(), &dialog_focus, cx)
    });
    if row.first_focus == Some(ConfirmationButton::Cancel) {
        // `open_dialog` focuses the dialog container. This call moves the
        // focus to Cancel, inside the focus trap of the dialog.
        cancel_focus.focus(window, cx);
    }
    cancel_focus
}

fn confirmation_dialog(
    dialog: Dialog,
    display: ConfirmationDialogDisplay,
    handlers: ConfirmationDialogHandlers,
    cancel_focus: &FocusHandle,
    cx: &App,
) -> Dialog {
    let ConfirmationDialogHandlers {
        on_cancel,
        on_confirm,
    } = handlers;
    let escape_cancel = on_cancel.clone();
    let enter_cancel = on_cancel.clone();
    let row = confirmation_button_row(display.destructive);
    let confirm_style = if display.destructive {
        ControlStyle::Destructive
    } else {
        ControlStyle::Primary
    };

    let mut cancel_button = Some(
        Button::styled(display.cancel_button_id, ControlStyle::Neutral)
            .size(ButtonSize::Md)
            .label(display.cancel_label)
            .a11y_label(display.cancel_a11y_label)
            .track_focus(cancel_focus)
            .on_click(move |_, window, cx| {
                on_cancel(window, cx);
                window.close_dialog(cx);
            }),
    );
    let mut confirm_button = Some(
        Button::styled(display.confirm_button_id, confirm_style)
            .size(ButtonSize::Md)
            .label(display.confirm_label)
            .a11y_label(display.confirm_a11y_label)
            .on_click(move |_, window, cx| {
                on_confirm(window, cx);
                window.close_dialog(cx);
            }),
    );
    let mut buttons = div()
        .flex()
        .flex_row()
        .justify_end()
        .gap(Spacing::SM.scaled(cx));
    for (index, which) in row.order.into_iter().enumerate() {
        let button = match which {
            ConfirmationButton::Cancel => cancel_button.take(),
            ConfirmationButton::Confirm => confirm_button.take(),
        };
        buttons = buttons.children(button);
        if index == 0 && row.space_after_first {
            buttons = buttons.child(destructive_button_space(cx));
        }
    }

    dialog
        .title(display.title)
        .overlay_closable(false)
        .close_button(false)
        .on_cancel(move |_, window, cx| {
            escape_cancel(window, cx);
            true
        })
        // Enter with no focused button runs the safe action (musicindex ADR
        // 0064 R-ACT-9 item 2). A focused button takes Enter first.
        .on_ok(move |_, window, cx| {
            enter_cancel(window, cx);
            true
        })
        .child(
            div()
                .w(Size::ColumnRegular.scaled(cx))
                .max_w(Size::ColumnTall.scaled(cx))
                .flex()
                .flex_col()
                .gap(Spacing::MD.scaled(cx))
                .child(
                    Label::new(display.message)
                        .variant(LabelVariant::Body)
                        .color(SemanticColor::SecondaryLabel),
                )
                .when(!display.items.is_empty(), |el| {
                    el.child(confirmation_items(display.items, cx))
                })
                .child(buttons),
        )
}

/// The space after the destructive button of a button row. With the gap of
/// the row on each side, it is larger than the space between the other
/// buttons (musicindex ADR 0064 R-ACT-8 item 4).
pub fn destructive_button_space(cx: &App) -> impl IntoElement {
    div().flex_shrink_0().w(Spacing::LG.scaled(cx))
}

/// The scrolling item column. Stacked text uses `overflow_hidden()` and no
/// truncation (ADR 0063).
///
/// Each item carries `flex_shrink_0()`.
///
/// A flex child with `overflow_hidden()` and no `flex_shrink_0()` has a
/// minimum height of zero. When the items are taller than `max_h`, the
/// layout shrinks each item to zero height. `flex_shrink_0()` keeps each
/// item at its own height. The column then scrolls to show the items that
/// do not fit (ADR 0076 Decision 8).
fn confirmation_items(items: Vec<ConfirmationDialogItem>, cx: &App) -> impl IntoElement {
    div()
        .id("confirmation-dialog-items")
        .debug_selector(|| "confirmation-dialog-items".to_owned())
        .flex()
        .flex_col()
        .gap(Spacing::SM.scaled(cx))
        .max_h(Size::ColumnRegular.scaled(cx))
        .overflow_y_scroll()
        .children(items.into_iter().enumerate().map(|(index, item)| {
            div()
                .debug_selector(move || format!("confirmation-dialog-item-{index}"))
                .flex()
                .flex_col()
                .flex_shrink_0()
                .gap(Spacing::XXS.scaled(cx))
                .min_w_0()
                .overflow_hidden()
                .child(
                    Label::new(item.title)
                        .variant(LabelVariant::Body)
                        .weight(FontWeight::SEMIBOLD),
                )
                .when_some(item.subtitle, |el, subtitle| {
                    el.child(
                        Label::new(subtitle)
                            .variant(LabelVariant::Caption)
                            .color(SemanticColor::SecondaryLabel),
                    )
                })
                .when_some(item.mark, |el, (mark, color)| {
                    el.child(
                        Label::new(mark)
                            .variant(LabelVariant::Caption)
                            .weight(FontWeight::SEMIBOLD)
                            .color(color),
                    )
                })
                .children(item.lines.into_iter().map(|line| {
                    Label::new(line)
                        .variant(LabelVariant::Caption)
                        .color(SemanticColor::TertiaryLabel)
                }))
        }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirmation_display_keeps_cancel_and_confirm_identity() {
        let display = ConfirmationDialogDisplay {
            title: SharedString::from("Remove Track from Library?"),
            message: SharedString::from("This track is in a playlist."),
            cancel_button_id: SharedString::from("cancel"),
            cancel_label: SharedString::from("Cancel"),
            cancel_a11y_label: SharedString::from("Cancel removing track from library"),
            confirm_button_id: SharedString::from("remove"),
            confirm_label: SharedString::from("Remove"),
            confirm_a11y_label: SharedString::from("Remove track from library"),
            destructive: true,
            items: Vec::new(),
        };

        assert_eq!(display.cancel_label, "Cancel");
        assert_eq!(
            display.cancel_a11y_label,
            SharedString::from("Cancel removing track from library")
        );
        assert_eq!(display.confirm_label, "Remove");
        assert_eq!(
            display.confirm_a11y_label,
            SharedString::from("Remove track from library")
        );
        assert!(display.destructive);
    }

    // -------------------------------------------------------------------
    // ADR 0076 task 008: the item column scrolls instead of shrinking each
    // item to zero height.
    // -------------------------------------------------------------------

    use gpui::{px, AppContext, Context, Render, TestAppContext};
    use gpui_component::Root;

    fn sample_items(count: usize) -> Vec<ConfirmationDialogItem> {
        (0..count)
            .map(|index| ConfirmationDialogItem {
                title: SharedString::from(format!("Track {index}.mp3")),
                subtitle: Some(SharedString::from("Example Album")),
                mark: Some((
                    SharedString::from("Title, Artist"),
                    SemanticColor::SuccessLabel,
                )),
                lines: vec![
                    SharedString::from("TIT2: title"),
                    SharedString::from("TPE1: artist"),
                ],
            })
            .collect()
    }

    fn empty_display() -> ConfirmationDialogDisplay {
        ConfirmationDialogDisplay {
            title: SharedString::from("Remove Track from Library?"),
            message: SharedString::from("This track is in a playlist."),
            cancel_button_id: SharedString::from("cancel"),
            cancel_label: SharedString::from("Cancel"),
            cancel_a11y_label: SharedString::from("Cancel removing track from library"),
            confirm_button_id: SharedString::from("remove"),
            confirm_label: SharedString::from("Remove"),
            confirm_a11y_label: SharedString::from("Remove track from library"),
            destructive: true,
            items: Vec::new(),
        }
    }

    /// Leaks a selector name so a name built at test time satisfies
    /// `VisualTestContext::debug_bounds`'s `'static` key. The test process
    /// ends after the test, so the leak carries no lasting cost.
    fn leaked_selector(name: String) -> &'static str {
        Box::leak(name.into_boxed_str())
    }

    struct ConfirmationItemsTest {
        items: Vec<ConfirmationDialogItem>,
    }

    impl Render for ConfirmationItemsTest {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .child(confirmation_items(self.items.clone(), cx))
        }
    }

    /// R76-8-01 and R76-8-02 (ADR 0076 Decision 8).
    ///
    /// Each item keeps its height when 40 items are taller than the
    /// scrolling column. The first item's height stays the same as in a
    /// one-item dialog. The column does not grow past its `max_h`.
    #[gpui::test]
    fn adr_0076_confirmation_list_keeps_item_height_with_many_items(cx: &mut TestAppContext) {
        cx.update(gpui_component::init);

        let (view, cx) = cx.add_window_view(|_, _| ConfirmationItemsTest {
            items: sample_items(1),
        });
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let one_item_height = cx
            .debug_bounds(leaked_selector("confirmation-dialog-item-0".to_owned()))
            .expect("the single item renders")
            .size
            .height;

        view.update(cx, |view, cx| {
            view.items = sample_items(40);
            cx.notify();
        });
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });

        let mut first_item_height = None;
        for index in 0..40 {
            let selector = leaked_selector(format!("confirmation-dialog-item-{index}"));
            let bounds = cx
                .debug_bounds(selector)
                .unwrap_or_else(|| panic!("item {index} did not render"));
            assert!(
                bounds.size.height > px(0.0),
                "item {index} has zero height with 40 items"
            );
            if index == 0 {
                first_item_height = Some(bounds.size.height);
            }
        }
        assert_eq!(
            first_item_height,
            Some(one_item_height),
            "the first item's height changed between one item and 40 items"
        );

        let max_height = cx.update(|_, cx| Size::ColumnRegular.scaled(cx));
        let column = cx
            .debug_bounds(leaked_selector("confirmation-dialog-items".to_owned()))
            .expect("the item column renders");
        assert!(
            column.size.height <= max_height,
            "the item column grew past its max_h: {:?} > {max_height:?}",
            column.size.height
        );
    }

    struct ConfirmationDialogTest {
        display: ConfirmationDialogDisplay,
        cancel_focus: FocusHandle,
    }

    impl Render for ConfirmationDialogTest {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let handlers = ConfirmationDialogHandlers::new(|_, _| {}, |_, _| {});
            div().size_full().child(confirmation_dialog(
                Dialog::new(cx),
                self.display.clone(),
                handlers,
                &self.cancel_focus,
                cx,
            ))
        }
    }

    /// R76-8-03: a confirmation with no item shows no item column. The fix
    /// for many items must not change this present behavior.
    #[gpui::test]
    fn adr_0076_confirmation_list_with_no_item_shows_no_column(cx: &mut TestAppContext) {
        cx.update(gpui_component::init);

        let (_, cx) = cx.add_window_view(|window, cx| {
            let inner = cx.new(|cx| ConfirmationDialogTest {
                display: empty_display(),
                cancel_focus: cx.focus_handle(),
            });
            Root::new(inner, window, cx)
        });
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });

        assert!(
            cx.debug_bounds(leaked_selector("confirmation-dialog-items".to_owned()))
                .is_none(),
            "an empty confirmation rendered an item column"
        );
    }

    // -------------------------------------------------------------------
    // musicindex ADR 0064: the order of the row and the keyboard of a
    // destructive confirmation.
    // -------------------------------------------------------------------

    use crate::view_models::button_row::{adr_0064_violation, ButtonRowRole};
    use std::cell::Cell;

    fn row_roles(destructive: bool) -> Vec<ButtonRowRole> {
        confirmation_button_row(destructive)
            .order
            .iter()
            .map(|button| match button {
                ConfirmationButton::Cancel => ButtonRowRole::Safe,
                ConfirmationButton::Confirm if destructive => ButtonRowRole::Destructive,
                ConfirmationButton::Confirm => ButtonRowRole::Other,
            })
            .collect()
    }

    /// musicindex ADR 0064 R-ACT-8 and R-ACT-9 item 1. Each destructive
    /// confirmation of this app (library removal, selection removal, remove from
    /// all playlists, playlist delete and saved search delete) uses this row.
    #[test]
    fn adr_0064_r_act_8_destructive_confirmation_puts_the_safe_action_last() {
        for destructive in [true, false] {
            if let Some(violation) = adr_0064_violation(&row_roles(destructive)) {
                panic!("{violation}");
            }
        }
        let row = confirmation_button_row(true);
        assert!(
            row.space_after_first,
            "musicindex ADR 0064 R-ACT-8 item 4: a larger space follows the destructive button. \
             Set `space_after_first` in `confirmation_button_row`."
        );
        assert_eq!(
            row.first_focus,
            Some(ConfirmationButton::Cancel),
            "musicindex ADR 0064 R-ACT-9 item 1: the safe action has the focus when the dialog \
             opens. Give Cancel the first focus in `confirmation_button_row`."
        );
    }

    /// The app shell draws the dialog layer of `Root` (`render_window_layers`).
    /// This host does the same, so the dialog gets the keys.
    struct DialogHost;

    impl Render for DialogHost {
        fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .children(Root::render_dialog_layer(window, cx))
        }
    }

    struct Calls {
        cancel: Rc<Cell<usize>>,
        confirm: Rc<Cell<usize>>,
    }

    impl Calls {
        fn new() -> Self {
            Self {
                cancel: Rc::new(Cell::new(0)),
                confirm: Rc::new(Cell::new(0)),
            }
        }

        fn handlers(&self) -> ConfirmationDialogHandlers {
            let cancel = self.cancel.clone();
            let confirm = self.confirm.clone();
            ConfirmationDialogHandlers::new(
                move |_, _| cancel.set(cancel.get() + 1),
                move |_, _| confirm.set(confirm.get() + 1),
            )
        }
    }

    fn open_in_root<'a>(
        cx: &'a mut TestAppContext,
        display: ConfirmationDialogDisplay,
        calls: &Calls,
    ) -> (FocusHandle, &'a mut gpui::VisualTestContext) {
        cx.update(gpui_component::init);
        let (_, cx) = cx.add_window_view(|window, cx| {
            let inner = cx.new(|_| DialogHost);
            Root::new(inner, window, cx)
        });
        let handlers = calls.handlers();
        let cancel_focus = cx.update(|window, cx| {
            open_confirmation_dialog_with_focus(window, cx, display, handlers)
        });
        cx.run_until_parked();
        cx.update(|window, cx| window.draw(cx).clear(cx));
        (cancel_focus, cx)
    }

    /// musicindex ADR 0064 R-ACT-9 items 1 and 2: Cancel has the focus when
    /// a destructive confirmation opens, and Enter runs Cancel, not the
    /// destructive action.
    #[gpui::test]
    fn adr_0064_r_act_9_enter_in_a_new_destructive_confirmation_cancels(cx: &mut TestAppContext) {
        let calls = Calls::new();
        let (cancel_focus, cx) = open_in_root(cx, empty_display(), &calls);

        assert!(
            cx.update(|window, _| cancel_focus.is_focused(window)),
            "musicindex ADR 0064 R-ACT-9 item 1: the safe action has the focus when the dialog \
             opens. Focus Cancel in `open_confirmation_dialog`."
        );

        cx.simulate_keystrokes("enter");
        cx.run_until_parked();

        assert_eq!(
            calls.confirm.get(),
            0,
            "musicindex ADR 0064 R-ACT-9 item 2: Enter does not start the destructive action. \
             Give the first focus to Cancel and send Enter with no focus to the safe action."
        );
        assert_eq!(
            calls.cancel.get(),
            1,
            "musicindex ADR 0064 R-ACT-9 item 2: Enter runs the safe action once. Make `on_ok` of the confirmation dialog run Cancel."
        );
        assert!(
            !cx.update(WindowExt::has_active_dialog),
            "Enter closes the dialog"
        );
    }

    /// musicindex ADR 0064 R-ACT-9 item 2: Enter with no focused button
    /// runs the safe action.
    #[gpui::test]
    fn adr_0064_r_act_9_enter_with_no_focused_button_cancels(cx: &mut TestAppContext) {
        let calls = Calls::new();
        let display = ConfirmationDialogDisplay {
            destructive: false,
            ..empty_display()
        };
        let (_, cx) = open_in_root(cx, display, &calls);

        cx.simulate_keystrokes("enter");
        cx.run_until_parked();

        assert_eq!(
            calls.confirm.get(),
            0,
            "musicindex ADR 0064 R-ACT-9 item 2: Enter with no focused button starts the safe \
             action. Make `on_ok` of the confirmation dialog run Cancel."
        );
        assert_eq!(
            calls.cancel.get(),
            1,
            "musicindex ADR 0064 R-ACT-9 item 2: Enter runs the safe action once. Make `on_ok` of the confirmation dialog run Cancel."
        );
    }

    /// musicindex ADR 0064 R-ACT-9 item 3: Escape closes the dialog with no
    /// action.
    #[gpui::test]
    fn adr_0064_r_act_9_escape_closes_with_no_action(cx: &mut TestAppContext) {
        let calls = Calls::new();
        let (_, cx) = open_in_root(cx, empty_display(), &calls);

        cx.simulate_keystrokes("escape");
        cx.run_until_parked();

        assert_eq!(
            calls.confirm.get(),
            0,
            "musicindex ADR 0064 R-ACT-9 item 3: Escape closes the dialog with no action."
        );
        assert!(
            !cx.update(WindowExt::has_active_dialog),
            "Escape closes the dialog"
        );
    }
}
