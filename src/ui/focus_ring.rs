//! The owner of the "show the focus ring" state (musicindex ADR 0026
//! R-FOCUS-1 item 3, ADR 0089 Decision 9).
//!
//! The ring shows for keyboard focus, and a mouse click does not show it. GPUI
//! gives `Window::last_input_was_keyboard`, but any mouse move clears that
//! fact. A resting mouse can then hide the ring while the keyboard user moves
//! the focus. Thus a key press shows the ring, a mouse button hides it, and a
//! mouse move changes nothing.
//!
//! The key press comes from a keystroke interceptor. A key listener on an
//! element does not see a key that has a binding, for example Tab, because
//! GPUI gives that key to its action and stops.

#![warn(clippy::pedantic)]

use gpui::{App, Global, InteractiveElement, Window};

/// `true` after a key press, until the next mouse button press.
#[derive(Clone, Copy, Debug, Default)]
struct FocusRingShows(bool);

impl Global for FocusRingShows {}

/// Tells if the focused control or row shows its focus ring now.
#[must_use]
pub(crate) fn focus_ring_shows(cx: &App) -> bool {
    cx.try_global::<FocusRingShows>()
        .is_some_and(|shows| shows.0)
}

fn set_focus_ring_shows(shows: bool, window: &mut Window, cx: &mut App) {
    if focus_ring_shows(cx) != shows {
        cx.set_global(FocusRingShows(shows));
        window.refresh();
    }
}

/// Starts the key press side of the state, once for the app.
pub(crate) fn init(cx: &mut App) {
    cx.intercept_keystrokes(|_, window, cx| set_focus_ring_shows(true, window, cx))
        .detach();
}

/// Puts the mouse button listener on the root element of the window.
pub(crate) fn track_focus_ring_mouse<E: InteractiveElement>(root: E) -> E {
    root.capture_any_mouse_down(|_, window, cx| set_focus_ring_shows(false, window, cx))
}

#[cfg(test)]
mod tests {
    use gpui::{
        actions, div, point, prelude::*, px, Context, FocusHandle, KeyBinding, Modifiers,
        MouseButton, TestAppContext, Window,
    };

    use super::{focus_ring_shows, init, track_focus_ring_mouse};

    actions!(focus_ring_test, [TabAction]);

    struct Root(FocusHandle);

    impl Render for Root {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            track_focus_ring_mouse(
                div()
                    .id("root")
                    .key_context("FocusRingRoot")
                    .size_full()
                    .track_focus(&self.0)
                    .on_action(|_: &TabAction, _, _| {}),
            )
        }
    }

    /// Walk of 2026-11-06: Tab moved the focus and no ring showed. Tab has a
    /// binding here, as it has in the app, so an element key listener would
    /// not see it. A mouse move must not hide the ring. Only a mouse button
    /// does.
    #[gpui::test]
    fn r_focus_1_key_shows_ring_move_keeps_it_and_click_hides_it(cx: &mut TestAppContext) {
        cx.update(|cx| {
            init(cx);
            cx.bind_keys([KeyBinding::new("tab", TabAction, Some("FocusRingRoot"))]);
        });
        let (root, cx) = cx.add_window_view(|_, cx| Root(cx.focus_handle()));
        let at = point(px(10.0), px(10.0));
        cx.update(|window, cx| {
            window.activate_window();
            let handle = root.read(cx).0.clone();
            window.focus(&handle, cx);
            window.draw(cx).clear(cx);
        });
        assert!(!cx.update(|_, cx| focus_ring_shows(cx)));

        cx.simulate_keystrokes("tab");
        assert!(cx.update(|_, cx| focus_ring_shows(cx)));

        cx.simulate_mouse_move(at, None, Modifiers::none());
        assert!(cx.update(|_, cx| focus_ring_shows(cx)));

        cx.simulate_mouse_down(at, MouseButton::Left, Modifiers::none());
        assert!(!cx.update(|_, cx| focus_ring_shows(cx)));
    }
}
