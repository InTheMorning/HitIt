//! The letter bar of a Music list page (ADR 0089 Decision 6, musicindex
//! ADR 0027 R-LIST-1 item 2).
//!
//! The bar is a row of letter buttons, from A to Z and then "#". The view
//! model gives each label, each id, each accessibility label and the
//! selected letter. This composite only lays them out. The selected letter
//! uses the primary control style and says "selected" in its accessibility
//! label.

#![warn(clippy::pedantic)]

use std::rc::Rc;

use gpui::{
    div, prelude::*, AnyElement, App, IntoElement, ParentElement, SharedString, Styled, Window,
};

use crate::ui::control_styles::ControlStyle;
use crate::ui::primitives::Button;
use crate::ui::tokens::Spacing;
use crate::view_models::letter_bar::{LetterBarVm, LetterKey};

/// The callback of a letter click. It gets the key of the letter.
pub(crate) type LetterHandler = Rc<dyn Fn(&LetterKey, &mut Window, &mut App) + 'static>;

/// Renders the letter bar. `on_letter` is the click of a letter. A `None`
/// callback leaves each letter without a click.
#[must_use]
pub(crate) fn letter_bar(
    display: LetterBarVm,
    on_letter: Option<&LetterHandler>,
    cx: &App,
) -> AnyElement {
    let mut row = div()
        .id(SharedString::from(display.control_id))
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(Spacing::XXS.scaled(cx));
    for letter in display.letters {
        let style = if letter.selected {
            ControlStyle::Primary
        } else {
            ControlStyle::Ghost
        };
        let mut button = Button::styled(SharedString::from(letter.id), style)
            .label(SharedString::from(letter.label))
            .a11y_label(SharedString::from(letter.a11y_label.clone()))
            .tooltip(SharedString::from(letter.a11y_label));
        if let Some(handler) = on_letter {
            let key = letter.key;
            let handler = handler.clone();
            button = button.on_click(move |_, window, cx| {
                handler(&key, window, cx);
            });
        }
        row = row.child(button);
    }
    row.into_any_element()
}
