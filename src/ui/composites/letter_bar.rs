//! The letter bar of a Music list page (ADR 0089 Decision 6, musicindex
//! ADR 0027 R-LIST-1 item 2).
//!
//! The bar is a row of letter buttons, from A to Z and then "#". The view
//! model gives each label, each id, each accessibility label and the
//! selected letter. This composite only lays them out. The selected letter
//! uses the primary control style and says "selected" in its accessibility
//! label.
//!
//! For the keyboard, the bar is one tab stop (WAI-ARIA toolbar pattern, walk of
//! 2026-11-06). Left, Right, Home and End move a cursor in the bar, and a move
//! sends no request. Enter or Space runs the letter of the cursor.

#![warn(clippy::pedantic)]

use std::rc::Rc;

use gpui::{
    div, prelude::*, AnyElement, App, FocusHandle, IntoElement, KeyDownEvent, ParentElement,
    SharedString, Styled, Window,
};

use crate::ui::control_styles::ControlStyle;
use crate::ui::focus_ring::focus_ring_shows;
use crate::ui::primitives::Button;
use crate::ui::tokens::{resolve_color, Radius, SemanticColor, Spacing};
use crate::view_models::letter_bar::{LetterBarVm, LetterKey, LetterStep};

/// The callback of a letter click. It gets the key of the letter.
pub(crate) type LetterHandler = Rc<dyn Fn(&LetterKey, &mut Window, &mut App) + 'static>;

/// The callback of one cursor move.
pub(crate) type LetterStepHandler = Rc<dyn Fn(LetterStep, &mut Window, &mut App) + 'static>;

/// The callback that runs the letter of the cursor.
pub(crate) type LetterActivateHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

/// The key context of the bar. The active pane bindings exclude it, so the bar
/// gets Left, Right, Home, End, Enter and Space (`src/app/keyboard.rs`).
const LETTER_BAR_KEY_CONTEXT: &str = "LetterBar";

/// The keyboard side of the letter bar.
#[derive(Clone)]
pub(crate) struct LetterBarKeys {
    /// The one tab stop of the bar.
    pub(crate) focus: FocusHandle,
    /// The focus after a mouse click on a letter, so the arrow keys act on
    /// the list.
    pub(crate) return_focus: FocusHandle,
    /// Moves the cursor of the bar.
    pub(crate) on_step: LetterStepHandler,
    /// Runs the letter of the cursor.
    pub(crate) on_activate: LetterActivateHandler,
}

/// Gives the keys of the bar to `keys`. Returns `true` when the bar used the key.
fn handle_letter_key(
    event: &KeyDownEvent,
    keys: &LetterBarKeys,
    window: &mut Window,
    cx: &mut App,
) -> bool {
    let stroke = &event.keystroke;
    if stroke.modifiers.modified() {
        return false;
    }
    let step = match stroke.key.as_str() {
        "left" => LetterStep::Previous,
        "right" => LetterStep::Next,
        "home" => LetterStep::First,
        "end" => LetterStep::Last,
        "enter" | "space" => {
            if !event.is_held {
                (keys.on_activate)(window, cx);
            }
            return true;
        }
        _ => return false,
    };
    (keys.on_step)(step, window, cx);
    true
}

/// Renders the letter bar. `on_letter` is the click of a letter. A `None`
/// callback leaves each letter without a click. `keys` makes the bar one tab
/// stop.
#[must_use]
pub(crate) fn letter_bar(
    display: LetterBarVm,
    on_letter: Option<&LetterHandler>,
    keys: Option<&LetterBarKeys>,
    cx: &App,
) -> AnyElement {
    let mut row = div()
        .id(SharedString::from(display.control_id))
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(Spacing::XXS.scaled(cx));
    if let Some(keys) = keys {
        let key_side = keys.clone();
        row = row
            .key_context(LETTER_BAR_KEY_CONTEXT)
            .track_focus(&keys.focus)
            .on_key_down(move |event, window, cx| {
                if handle_letter_key(event, &key_side, window, cx) {
                    cx.stop_propagation();
                }
            });
    }
    let focus_color = resolve_color(cx, SemanticColor::Focus, None);
    for letter in display.letters {
        let style = if letter.selected {
            ControlStyle::Primary
        } else {
            ControlStyle::Ghost
        };
        let mut button = Button::styled(SharedString::from(letter.id), style)
            .narrow_hit_target()
            .label(SharedString::from(letter.label))
            .a11y_label(SharedString::from(letter.a11y_label.clone()))
            .tooltip(SharedString::from(letter.a11y_label));
        if let Some(handler) = on_letter {
            let key = letter.key;
            let handler = handler.clone();
            let return_focus = keys.map(|keys| keys.return_focus.clone());
            button = button.on_click(move |_, window, cx| {
                handler(&key, window, cx);
                if let (Some(list), false) = (&return_focus, focus_ring_shows(cx)) {
                    window.focus(list, cx);
                }
            });
        }
        // The border always takes its space, so the cursor moves no letter.
        row = row.child(
            div()
                .rounded(Radius::SM.scaled(cx))
                .border_2()
                .border_color(if letter.cursor {
                    focus_color
                } else {
                    gpui::transparent_black().into()
                })
                .child(button),
        );
    }
    row.into_any_element()
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use gpui::{Context, FocusHandle, Render, TestAppContext, Window};

    use super::*;
    use crate::view_models::list_page::ListPageKey;

    struct BarRoot {
        focus: FocusHandle,
        steps: Rc<RefCell<Vec<LetterStep>>>,
        runs: Rc<RefCell<usize>>,
    }

    impl Render for BarRoot {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let steps = self.steps.clone();
            let runs = self.runs.clone();
            let keys = LetterBarKeys {
                focus: self.focus.clone(),
                return_focus: self.focus.clone(),
                on_step: Rc::new(move |step, _, _| steps.borrow_mut().push(step)),
                on_activate: Rc::new(move |_, _| *runs.borrow_mut() += 1),
            };
            letter_bar(
                LetterBarVm::new(ListPageKey::Artists, None),
                None,
                Some(&keys),
                cx,
            )
        }
    }

    /// Walk of 2026-11-06: the letters work as one unit from the keyboard. A
    /// move runs nothing, and Enter or Space runs the letter.
    #[gpui::test]
    fn adr_0089_letter_bar_is_one_unit_for_the_keyboard(cx: &mut TestAppContext) {
        cx.update(gpui_component::init);
        let steps = Rc::new(RefCell::new(Vec::new()));
        let runs = Rc::new(RefCell::new(0));
        let (root_steps, root_runs) = (steps.clone(), runs.clone());
        let (root, cx) = cx.add_window_view(|_, cx| BarRoot {
            focus: cx.focus_handle(),
            steps: root_steps,
            runs: root_runs,
        });
        cx.update(|window, cx| {
            window.activate_window();
            let handle = root.read(cx).focus.clone();
            window.focus(&handle, cx);
            window.draw(cx).clear(cx);
        });
        cx.simulate_keystrokes("right left home end");
        assert_eq!(
            *steps.borrow(),
            [
                LetterStep::Next,
                LetterStep::Previous,
                LetterStep::First,
                LetterStep::Last
            ]
        );
        assert_eq!(*runs.borrow(), 0, "a move sends no request");
        cx.simulate_keystrokes("enter space");
        assert_eq!(*runs.borrow(), 2);
    }
}
