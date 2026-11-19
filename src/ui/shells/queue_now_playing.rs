//! Queue/Now Playing workspace-frame shell.
//!
//! ADR 0046 Phase 4 gives playback status and transport controls their own
//! surface. The global toolbar remains a compact status affordance.

#![warn(clippy::pedantic)]

use std::rc::Rc;

use gpui::{
    div, prelude::*, App, ClickEvent, Div, FontWeight, IntoElement, ParentElement, RenderOnce,
    SharedString, Stateful, Styled, Window,
};

use crate::ui::composites::track_drag::{attach_cue_row_drag, CueRowDragPayload};
use crate::ui::control_styles::ControlStyle;
use crate::ui::icons::{Icon, IconName, IconSize};
use crate::ui::primitives::{
    Button, ContextMenu, ContextMenuItem, ContextMenuItemDisplay, ContextMenuScope, Tooltip,
};
use crate::ui::tokens::{color, FontSize, Radius, SemanticColor, Size, Spacing};
use crate::view_models::queue_now_playing::{
    CueMoveTarget, QueueNowPlayingPageVm, QueueRowDisplay, TransportDisplay, TransportState,
};
use crate::view_models::workspace::FrameChromeButtonDisplay;

type QueueClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
type QueueSelectHandler = Rc<dyn Fn(i64, &ClickEvent, &mut Window, &mut App) + 'static>;
/// Moves a cue entry (ADR 0106 Decision 2). It gets the entry id and where
/// the entry goes.
type QueueMoveHandler = Rc<dyn Fn(i64, CueMoveTarget, &mut Window, &mut App) + 'static>;

/// Callback slots supplied by the application-owned queue frame.
#[derive(Clone, Default)]
#[must_use]
pub(crate) struct QueueNowPlayingSlots {
    skip_previous: Option<QueueClickHandler>,
    play_pause: Option<QueueClickHandler>,
    skip_next: Option<QueueClickHandler>,
    select_entry: Option<QueueSelectHandler>,
    move_entry: Option<QueueMoveHandler>,
    stop: Option<QueueClickHandler>,
    clear: Option<QueueClickHandler>,
}

impl QueueNowPlayingSlots {
    /// Creates empty queue-frame slots.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Supplies the previous-track callback.
    pub(crate) fn on_skip_previous(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.skip_previous = Some(Rc::new(handler));
        self
    }

    /// Supplies the play/pause callback.
    pub(crate) fn on_play_pause(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.play_pause = Some(Rc::new(handler));
        self
    }

    /// Supplies the next-track callback.
    pub(crate) fn on_skip_next(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.skip_next = Some(Rc::new(handler));
        self
    }

    pub(crate) fn on_select_entry(
        mut self,
        handler: impl Fn(i64, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.select_entry = Some(Rc::new(handler));
        self
    }

    /// Supplies the callback that moves a cue entry (ADR 0106).
    pub(crate) fn on_move_entry(
        mut self,
        handler: impl Fn(i64, CueMoveTarget, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.move_entry = Some(Rc::new(handler));
        self
    }

    pub(crate) fn on_stop(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.stop = Some(Rc::new(handler));
        self
    }

    pub(crate) fn on_clear(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.clear = Some(Rc::new(handler));
        self
    }
}

/// Queue cuelist shell element.
#[derive(IntoElement)]
#[must_use]
pub(crate) struct QueueCuelistShell {
    vm: QueueNowPlayingPageVm,
    slots: QueueNowPlayingSlots,
}

/// Queue transport shell element.
#[derive(IntoElement)]
#[must_use]
pub(crate) struct QueueTransportShell {
    transport: TransportDisplay,
    slots: QueueNowPlayingSlots,
}

/// Creates a queue-only shell for panel cuelist mode.
pub(crate) fn render_queue_cuelist(
    vm: QueueNowPlayingPageVm,
    slots: QueueNowPlayingSlots,
) -> QueueCuelistShell {
    QueueCuelistShell { vm, slots }
}

/// Creates a transport-only shell for the Show surface.
pub(crate) fn render_queue_transport(
    transport: TransportDisplay,
    slots: QueueNowPlayingSlots,
) -> QueueTransportShell {
    QueueTransportShell { transport, slots }
}

impl RenderOnce for QueueCuelistShell {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let QueueNowPlayingPageVm {
            rows,
            empty_label,
            actions,
            cue_move_hint,
            ..
        } = self.vm;
        let clear = if actions.clear.availability
            == crate::view_models::queue_now_playing::ShowActionAvailability::Available
        {
            let mut button = Button::styled("show-clear-cue", ControlStyle::Destructive)
                .label(actions.clear.label)
                .a11y_label(actions.clear.a11y_label);
            if let Some(handler) = self.slots.clear {
                button = button.on_click(move |event, window, cx| handler(event, window, cx));
            } else {
                button = button.disabled(true);
            }
            Some(button)
        } else {
            None
        };
        div()
            .flex()
            .flex_col()
            .gap(Spacing::SM.scaled(cx))
            .children(clear)
            .child(render_queue_list(
                rows,
                empty_label,
                (
                    self.slots.select_entry.as_ref(),
                    self.slots.move_entry.as_ref(),
                ),
                cx,
            ))
            .children(cue_move_hint.map(|hint| {
                div()
                    .px(Spacing::MD.scaled(cx))
                    .text_size(FontSize::Caption.scaled(cx))
                    .text_color(color(cx, SemanticColor::TertiaryLabel))
                    .child(SharedString::from(hint))
            }))
    }
}

impl RenderOnce for QueueTransportShell {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        render_control_deck(self.transport, self.slots, cx)
    }
}

fn render_queue_list(
    rows: Vec<QueueRowDisplay>,
    empty_label: &'static str,
    handlers: (Option<&QueueSelectHandler>, Option<&QueueMoveHandler>),
    cx: &App,
) -> impl IntoElement {
    let (select_entry, move_entry) = handlers;
    let row_gap = Spacing::XXS.scaled(cx);

    if rows.is_empty() {
        return div()
            .flex()
            .flex_1()
            .min_h_0()
            .items_center()
            .justify_center()
            .text_size(FontSize::Caption.scaled(cx))
            .text_color(color(cx, SemanticColor::TertiaryLabel))
            .child(SharedString::from(empty_label))
            .into_any_element();
    }

    div()
        .id("queue-now-playing-list")
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .min_w_0()
        .overflow_y_scroll()
        .p(Spacing::MD.scaled(cx))
        .gap(row_gap)
        .children(
            rows.into_iter()
                .map(|row| render_queue_row(row, (select_entry.cloned(), move_entry.cloned()), cx)),
        )
        .into_any_element()
}

fn render_queue_row(
    row: QueueRowDisplay,
    handlers: (Option<QueueSelectHandler>, Option<QueueMoveHandler>),
    cx: &App,
) -> impl IntoElement {
    let (select_entry, move_entry) = handlers;
    let show_play_next = row.play_next();
    let tertiary_label = color(cx, SemanticColor::TertiaryLabel);
    let accent = color(cx, SemanticColor::Accent);
    let fill = color(cx, SemanticColor::TertiaryFill);

    div()
        .id(SharedString::from(row.id.clone()))
        .debug_selector(|| "queue-now-playing-row".to_owned())
        .min_h(Size::RowLg.scaled(cx))
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::SM.scaled(cx))
        .px(Spacing::SM.scaled(cx))
        .py(Spacing::XS.scaled(cx))
        .rounded(Radius::MD.scaled(cx))
        .when(row.now_playing, |el| el.bg(fill))
        .when(row.selected, |el| el.border_1().border_color(accent))
        .when_some(
            row.entry_id
                .filter(|_| row.selectable || row.can_move())
                .zip(select_entry),
            |el, (entry_id, handler)| {
                el.on_click(move |event, window, cx| handler(entry_id, event, window, cx))
            },
        )
        .when_some(
            row.entry_id
                .filter(|_| row.can_move())
                .zip(move_entry.clone()),
            |el, (entry_id, handler)| {
                let drag = CueRowDragPayload {
                    entry_id,
                    label: SharedString::from(row.title.clone()),
                };
                let to_index = row.cue_index;
                let drop: CueRowDropHandler =
                    Rc::new(move |dragged: i64, window: &mut Window, cx: &mut App| {
                        handler(dragged, CueMoveTarget::Index(to_index), window, cx);
                    });
                cue_row_drop_target(attach_cue_row_drag(el, drag), entry_id, true, accent, drop)
            },
        )
        .tooltip({
            let label = SharedString::from(row.a11y_label);
            move |window, cx| Tooltip::new(label.clone()).build(window, cx)
        })
        .child(
            div()
                .w(Size::ButtonSm.scaled(cx))
                .flex()
                .items_center()
                .justify_center()
                .when(row.now_playing, |el| {
                    el.child(
                        Icon::new(IconName::Play)
                            .size(IconSize::Action)
                            .color(accent),
                    )
                }),
        )
        .child(row_text(row.title, row.artist, row.now_playing, cx))
        .when_some(row.duration_label, |el, duration| {
            el.child(
                div()
                    .text_size(FontSize::Caption.scaled(cx))
                    .text_color(tertiary_label)
                    .child(SharedString::from(duration)),
            )
        })
        .children(
            [row.unavailable_label.map(str::to_owned), row.state_label]
                .into_iter()
                .flatten()
                .map(|text| {
                    div()
                        .text_size(FontSize::Micro.scaled(cx))
                        .text_color(tertiary_label)
                        .child(SharedString::from(text))
                }),
        )
        .when_some(
            row.entry_id.filter(|_| show_play_next).zip(move_entry),
            |el, (entry_id, handler)| el.child(play_next_controls(entry_id, &handler)),
        )
}

/// The handler of a drop of a cue row. It gets the entry id of the dragged row.
type CueRowDropHandler = Rc<dyn Fn(i64, &mut Window, &mut App)>;

/// Makes `el` a drop target of a dragged cue row (ADR 0106 Decision 2).
/// A row that cannot take the entry (`accepts` is false) takes no drop. A row
/// does not take its own drag. An accepting row shows a 2 px accent border
/// while the pointer is on it. `on_drop` gets the entry id of the dragged row.
fn cue_row_drop_target(
    el: Stateful<Div>,
    own_entry_id: i64,
    accepts: bool,
    accent: gpui::Rgba,
    on_drop: CueRowDropHandler,
) -> Stateful<Div> {
    el.can_drop(move |drag, _window, _cx| {
        accepts
            && drag
                .downcast_ref::<CueRowDragPayload>()
                .is_some_and(|payload| payload.entry_id != own_entry_id)
    })
    .drag_over::<CueRowDragPayload>(move |el, _, _, _| {
        if accepts {
            el.border_2().border_color(accent)
        } else {
            el
        }
    })
    .on_drop(
        move |payload: &CueRowDragPayload, window: &mut Window, cx: &mut App| {
            if payload.entry_id != own_entry_id {
                on_drop(payload.entry_id, window, cx);
            }
        },
    )
}

/// The title and the artist of a queue row.
fn row_text(
    title: String,
    artist: Option<String>,
    now_playing: bool,
    cx: &App,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .child(
            div()
                .text_size(FontSize::Body.scaled(cx))
                .text_color(color(cx, SemanticColor::Label))
                .font_weight(if now_playing {
                    FontWeight::MEDIUM
                } else {
                    FontWeight::NORMAL
                })
                .truncate()
                .child(SharedString::from(title)),
        )
        .when_some(artist, |el, artist| {
            el.child(
                div()
                    .text_size(FontSize::Micro.scaled(cx))
                    .text_color(color(cx, SemanticColor::SecondaryLabel))
                    .truncate()
                    .child(SharedString::from(artist)),
            )
        })
}

/// The "Play next" button and the row menu with the same item (ADR 0106
/// Decision 2).
fn play_next_controls(entry_id: i64, handler: &QueueMoveHandler) -> impl IntoElement {
    let button_handler = Rc::clone(handler);
    let menu_handler = Rc::clone(handler);
    let button = Button::styled(
        SharedString::from(format!("queue-play-next-{entry_id}")),
        ControlStyle::RowAction,
    )
    .label("Play next")
    .a11y_label("Play next")
    .on_click(move |_, window, cx| {
        button_handler(entry_id, CueMoveTarget::AfterCurrent, window, cx);
    });
    let menu = ContextMenu::new(
        SharedString::from(format!("queue-row-menu-{entry_id}")),
        ContextMenuScope::TrackList,
        "Cue row actions",
    )
    .trigger_icon(IconName::More)
    .item(
        ContextMenuItem::new(ContextMenuItemDisplay {
            id: SharedString::from(format!("queue-row-menu-play-next-{entry_id}")),
            label: "Play next".into(),
            a11y_label: "Play next".into(),
            destructive: false,
            disabled: false,
        })
        .on_select(move |window, cx| {
            menu_handler(entry_id, CueMoveTarget::AfterCurrent, window, cx);
        }),
    );
    div()
        .flex()
        .flex_row()
        .items_center()
        .child(button)
        .child(menu)
}

fn render_control_deck(
    transport: TransportDisplay,
    slots: QueueNowPlayingSlots,
    cx: &mut App,
) -> impl IntoElement {
    let border = color(cx, SemanticColor::Separator);

    div()
        .flex()
        .flex_col()
        .flex_shrink_0()
        .gap(Spacing::MD.scaled(cx))
        .border_t_1()
        .border_color(border)
        .p(Spacing::MD.scaled(cx))
        .child(render_transport(transport, slots, cx))
}

fn render_transport(
    transport: TransportDisplay,
    slots: QueueNowPlayingSlots,
    cx: &App,
) -> impl IntoElement {
    let icon = match transport.play_pause_state {
        TransportState::Playing => IconName::Pause,
        TransportState::Paused
        | TransportState::Stopped
        | TransportState::Empty
        | TransportState::Starting
        | TransportState::Failed => IconName::Play,
    };

    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .gap(Spacing::SM.scaled(cx))
        .child(transport_button(
            transport.skip_previous,
            IconName::Previous,
            slots.skip_previous,
        ))
        .child(transport_button(
            FrameChromeButtonDisplay::new(
                transport.play_pause_id,
                transport.play_pause_a11y_label,
                transport.disabled,
            ),
            icon,
            slots.play_pause,
        ))
        .child(transport_button(
            transport.skip_next,
            IconName::Next,
            slots.skip_next,
        ))
        .child(transport_button(transport.stop, IconName::Stop, slots.stop))
}

fn transport_button(
    display: FrameChromeButtonDisplay,
    icon: IconName,
    handler: Option<QueueClickHandler>,
) -> Button {
    let disabled = display.disabled;
    let mut button = Button::styled(SharedString::from(display.id), ControlStyle::ToolbarIcon)
        .leading_icon(icon)
        .a11y_label(display.a11y_label)
        .tooltip(display.a11y_label)
        .disabled(disabled);

    if !disabled {
        if let Some(handler) = handler {
            button = button.on_click(move |event, window, cx| {
                handler(event, window, cx);
            });
        }
    }

    button
}

// ---------------------------------------------------------------------------
// ADR 0039 task 002 M1: fixed geometry does not depend on string length.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use gpui::{div, prelude::*, px, TestAppContext};

    use super::render_queue_row;
    use crate::view_models::queue_now_playing::QueueRowDisplay;

    fn row(
        title: &str,
        artist: Option<&str>,
        duration: Option<&str>,
        now_playing: bool,
    ) -> QueueRowDisplay {
        QueueRowDisplay {
            track_id: 1,
            entry_id: None,
            selected: false,
            selectable: false,
            id: "queue-row-1".to_owned(),
            title: title.to_owned(),
            artist: artist.map(str::to_owned),
            duration_label: duration.map(str::to_owned),
            now_playing,
            unavailable_label: None,
            a11y_label: title.to_owned(),
            cue_index: 0,
            state_label: None,
            cue_move: crate::view_models::queue_now_playing::CueRowMove::Locked,
            duration_seconds: None,
        }
    }

    struct RowHeightTest {
        display: QueueRowDisplay,
    }

    impl gpui::Render for RowHeightTest {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            cx: &mut gpui::Context<Self>,
        ) -> impl gpui::IntoElement {
            div()
                .w(px(320.0))
                .child(render_queue_row(self.display.clone(), (None, None), cx))
        }
    }

    /// ADR 0039 task 002 M1: the queue row is a floor
    /// (`.min_h(Size::RowLg.scaled(cx))`), not a cap, but its height must
    /// still stay independent of title/artist string length and unaffected
    /// by which optional slots (artist, duration, now-playing) are present —
    /// title/artist clip via `.truncate()` inside their `flex_1()` ancestor
    /// rather than growing the row.
    #[gpui::test]
    fn adr_0039_queue_row_height_is_independent_of_title_length(cx: &mut TestAppContext) {
        cx.update(gpui_component::init);
        let short = row("A", None, None, false);
        let (view, cx) = cx.add_window_view(|_, _| RowHeightTest {
            display: short.clone(),
        });

        let long_title = "A very long queue row title that would otherwise wrap this row".repeat(3);
        let cases = [
            short.clone(),
            row(&long_title, None, None, false),
            row("A", Some("Artist"), None, false),
            row(
                &long_title,
                Some("A very long artist name too"),
                None,
                false,
            ),
            row("A", Some("Artist"), Some("3:45"), false),
            row(
                &long_title,
                Some("A very long artist name too"),
                Some("3:45"),
                true,
            ),
        ];

        let mut heights = Vec::new();
        for display in cases {
            view.update(cx, |this, cx| {
                this.display = display;
                cx.notify();
            });
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            let bounds = cx
                .debug_bounds("queue-now-playing-row")
                .expect("queue row bounds recorded");
            heights.push(bounds.size.height);
        }

        // Toggling `now_playing`, `artist` or `duration_label` legitimately
        // changes the row (a highlighted background, an extra line, a
        // trailing label) — those are named optional states, not a
        // string-length dependency. What must never move is the height
        // *within* a fixed set of present slots when only string length
        // changes: cases 0 vs 1 (title only) and cases 2 vs 3 (title+artist).
        assert_eq!(
            heights[0], heights[1],
            "queue row height changed with title length alone: {heights:?}"
        );
        assert_eq!(
            heights[2], heights[3],
            "queue row height changed with title/artist length alone: {heights:?}"
        );
    }
}
