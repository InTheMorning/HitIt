//! Show screen shell.
//!
//! ADR 0060 gives playback its own screen mount instead of a workspace frame.
//! ADR 0063 arranges Show as a dashboard: the shell composes the summary, card
//! grid, transport deck, and trailing detail panel without taking ownership of
//! section detail rendering.

#![warn(clippy::pedantic)]

use std::rc::Rc;

use gpui::{
    div, prelude::*, App, ClickEvent, FontWeight, IntoElement, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, ParentElement, RenderOnce, SharedString, Styled, Window,
};

use crate::ui::composites::{ListRow, ListRowA11yLabel};
use crate::ui::composites::{
    ShowCard, ShowDetailPanel, ShowDetailPanelDisplay, ShowDetailPanelSlots, ShowLogPane,
    ShowLogPaneSlots,
};
use crate::ui::primitives::{Button, Label, LabelVariant};
use crate::ui::shells::queue_now_playing::{render_queue_transport, QueueNowPlayingSlots};
use crate::ui::tokens::{color, FontSize, SemanticColor, Spacing};
use crate::view_models::queue_now_playing::CueMoveTarget;
use crate::view_models::show::{
    EndShowAction, EventControlIntent, OnAirAction, OnAirBandDisplay, PublisherServiceRole,
    ShowCardDisplay, ShowCardKind, ShowEmptyStateDisplay, ShowNowPlayingDisplay, ShowPageVm,
    ShowPanelMode, ShowPlaylistRowDisplay, ShowWidthClass,
};

/// Callback slots supplied by the application-owned Show screen.
#[must_use]
pub(crate) struct ShowSlots {
    queue: QueueNowPlayingSlots,
    card: ShowCardSlots,
    panel: ShowDetailPanelSlots,
    log: ShowLogPaneSlots,
    load_playlist: Option<ShowLoadPlaylistHandler>,
    go_on_air: Option<ShowGoOnAirHandler>,
    on_air_action: Option<ShowOnAirActionHandler>,
    toggle_show_information: Option<ShowGoOnAirHandler>,
    end_show: Option<ShowEndActionHandler>,
}

/// The handler of the row action "Load into Show cue". It gets the playlist id.
type ShowLoadPlaylistHandler = Rc<dyn Fn(i64, &mut Window, &mut App) + 'static>;
type ShowGoOnAirHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;
/// The handler of the one action of an on-air row.
type ShowOnAirActionHandler = Rc<dyn Fn(OnAirAction, &mut Window, &mut App) + 'static>;
type ShowEndActionHandler = Rc<dyn Fn(EndShowAction, &mut Window, &mut App) + 'static>;

impl Default for ShowSlots {
    fn default() -> Self {
        Self {
            queue: QueueNowPlayingSlots::new(),
            card: ShowCardSlots::default(),
            panel: ShowDetailPanelSlots::new(),
            log: ShowLogPaneSlots::default(),
            load_playlist: None,
            go_on_air: None,
            on_air_action: None,
            toggle_show_information: None,
            end_show: None,
        }
    }
}

type ShowCardClickHandler = Rc<dyn Fn(ShowCardKind, &ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(Default)]
struct ShowCardSlots {
    select: Option<ShowCardClickHandler>,
}

impl ShowSlots {
    pub(crate) fn log_frames(
        mut self,
        frames: crate::ui::composites::log_frame::LogFrames,
    ) -> Self {
        self.log.frames = frames;
        self
    }

    pub(crate) fn on_select_event(
        mut self,
        handler: impl Fn(String, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.panel = self.panel.on_select_event(handler);
        self
    }
    pub(crate) fn on_select_cue_entry(
        mut self,
        handler: impl Fn(i64, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.panel = self.panel.on_select_cue_entry(handler);
        self
    }
    pub(crate) fn on_move_cue_entry(
        mut self,
        handler: impl Fn(i64, CueMoveTarget, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.panel = self.panel.on_move_cue_entry(handler);
        self
    }
    pub(crate) fn on_clear_cue(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.panel = self.panel.on_clear_cue(handler);
        self
    }
    pub(crate) fn on_stop_show(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.queue = self.queue.on_stop(handler);
        self
    }
    pub(crate) fn on_open_event_logs(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.panel = self.panel.on_open_event_logs(handler);
        self
    }
    pub(crate) fn on_event_control(
        mut self,
        handler: impl Fn(EventControlIntent, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.panel = self.panel.on_event_control(handler);
        self
    }

    /// Supplies the callback of the row action "Load into Show cue" of the
    /// empty Show page (ADR 0106 Decision 4).
    pub(crate) fn on_load_playlist(
        mut self,
        handler: impl Fn(i64, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.load_playlist = Some(Rc::new(handler));
        self
    }

    /// Supplies the Go on air action.
    pub(crate) fn on_go_on_air(
        mut self,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.go_on_air = Some(Rc::new(handler));
        self
    }

    fn on_air_handlers(&self) -> OnAirHandlers {
        OnAirHandlers {
            go: self.go_on_air.clone(),
            action: self.on_air_action.clone(),
            toggle_information: self.toggle_show_information.clone(),
            end_show: self.end_show.clone(),
        }
    }

    /// Supplies the action of an on-air row.
    pub(crate) fn on_on_air_action(
        mut self,
        handler: impl Fn(OnAirAction, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_air_action = Some(Rc::new(handler));
        self
    }

    /// Supplies the switch "Show information".
    pub(crate) fn on_toggle_show_information(
        mut self,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.toggle_show_information = Some(Rc::new(handler));
        self
    }

    /// Supplies the End show and last-track actions.
    pub(crate) fn on_end_show(
        mut self,
        handler: impl Fn(EndShowAction, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.end_show = Some(Rc::new(handler));
        self
    }

    /// Creates empty Show-screen slots.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Supplies the previous-track callback.
    pub(crate) fn on_skip_previous(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.queue = self.queue.on_skip_previous(handler);
        self
    }

    /// Supplies the play/pause callback.
    pub(crate) fn on_play_pause(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.queue = self.queue.on_play_pause(handler);
        self
    }

    /// Supplies the next-track callback.
    pub(crate) fn on_skip_next(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.queue = self.queue.on_skip_next(handler);
        self
    }

    /// Supplies the Source readiness open callback.
    pub(crate) fn on_open_broadcast_readiness(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.panel = self.panel.on_open_broadcast_readiness(handler);
        self
    }

    /// Supplies the dashboard card selection callback.
    pub(crate) fn on_select_card(
        mut self,
        handler: impl Fn(ShowCardKind, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.card.select = Some(Rc::new(handler));
        self
    }

    /// Supplies the panel-open callback.
    pub(crate) fn on_open_show_panel(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.panel = self.panel.on_open_panel(handler);
        self
    }

    /// Supplies the panel-close callback.
    pub(crate) fn on_close_show_panel(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.panel = self.panel.on_close_panel(handler);
        self
    }

    /// Supplies the callback that returns detail mode to the cuelist.
    pub(crate) fn on_show_cuelist(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.panel = self.panel.on_show_cuelist(handler);
        self
    }

    /// Supplies the publisher service start callback.
    pub(crate) fn on_start_publisher_service(
        mut self,
        handler: impl Fn(PublisherServiceRole, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.panel = self.panel.on_start_publisher_service(handler);
        self
    }

    /// Supplies the publisher service stop callback.
    pub(crate) fn on_stop_publisher_service(
        mut self,
        handler: impl Fn(PublisherServiceRole, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.panel = self.panel.on_stop_publisher_service(handler);
        self
    }

    /// Supplies the publisher service reset callback.
    pub(crate) fn on_reset_publisher_service(
        mut self,
        handler: impl Fn(PublisherServiceRole, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.panel = self.panel.on_reset_publisher_service(handler);
        self
    }

    /// Supplies the publisher log open callback.
    pub(crate) fn on_open_publisher_logs(
        mut self,
        handler: impl Fn(PublisherServiceRole, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.panel = self.panel.on_open_publisher_logs(handler);
        self
    }

    /// Supplies the publisher log close callback.
    pub(crate) fn on_close_publisher_logs(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.log.close = Some(Rc::new(handler));
        self
    }

    /// Supplies shared log split geometry and resize callbacks.
    pub(crate) fn on_log_layout(
        mut self,
        handler: impl Fn(f32, f32, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.log.layout = Some(Rc::new(handler));
        self
    }

    pub(crate) fn on_log_resize_start(
        mut self,
        handler: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.log.resize_start = Some(Rc::new(handler));
        self
    }

    pub(crate) fn on_log_resize_move(
        mut self,
        handler: impl Fn(&MouseMoveEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.log.resize_move = Some(Rc::new(handler));
        self
    }

    pub(crate) fn on_log_resize_end(
        mut self,
        handler: impl Fn(&MouseUpEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.log.resize_end = Some(Rc::new(handler));
        self
    }

    /// Supplies the stream connect callback.
    pub(crate) fn on_connect_stream(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.panel = self.panel.on_connect_stream(handler);
        self
    }

    /// Supplies the stream disconnect callback.
    pub(crate) fn on_disconnect_stream(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.panel = self.panel.on_disconnect_stream(handler);
        self
    }
}

/// Show screen element.
#[derive(IntoElement)]
#[must_use]
pub(crate) struct ShowShell {
    vm: ShowPageVm,
    slots: ShowSlots,
}

/// Creates the Show screen shell.
pub(crate) fn render_show(vm: ShowPageVm, slots: ShowSlots) -> ShowShell {
    ShowShell { vm, slots }
}

impl RenderOnce for ShowShell {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let ShowPageVm {
            title,
            state_label,
            now_playing,
            active: _,
            empty_state,
            cue_playlists,
            source,
            publisher,
            stream,
            cards,
            width_class,
            panel_mode,
            panel_open,
            panel_chrome,
            log_pane,
            queue,
            status_message,
            on_air,
        } = self.vm;
        let transport = queue.transport.clone();

        div()
            .id("show-screen")
            .size_full()
            .flex()
            .flex_col()
            .min_h_0()
            .min_w_0()
            .overflow_hidden()
            .bg(color(cx, SemanticColor::SystemBackground))
            .child(render_show_summary(
                title,
                state_label,
                now_playing,
                empty_state,
                cue_playlists,
                self.slots.load_playlist.clone(),
                cx,
            ))
            .when_some(on_air, |el, band| {
                el.child(render_on_air_band(band, self.slots.on_air_handlers(), cx))
            })
            // A command that fails on this screen says so on this screen.
            .when_some(status_message, |el, message| {
                el.child(render_status_message(message, cx))
            })
            .child(
                div()
                    .id("show-dashboard-body")
                    .flex()
                    .flex_row()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .overflow_hidden()
                    .child(
                        div()
                            .id("show-dashboard-main")
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_h_0()
                            .min_w_0()
                            .overflow_hidden()
                            .child(
                                ShowLogPane::new(
                                    log_pane,
                                    render_show_card_grid(
                                        cards,
                                        width_class,
                                        panel_mode,
                                        panel_open,
                                        &self.slots.card,
                                        cx,
                                    )
                                    .into_any_element(),
                                )
                                .slots(self.slots.log),
                            )
                            .child(render_queue_transport(transport, self.slots.queue)),
                    )
                    .child(ShowDetailPanel::new(
                        ShowDetailPanelDisplay {
                            panel_mode,
                            panel_open,
                            panel_chrome,
                            queue,
                            source,
                            publisher,
                            stream,
                        },
                        self.slots.panel,
                    )),
            )
    }
}

fn render_status_message(message: String, cx: &App) -> impl IntoElement {
    div()
        .px(Spacing::LG.scaled(cx))
        .pb(Spacing::SM.scaled(cx))
        .text_size(FontSize::Caption.scaled(cx))
        .text_color(color(cx, SemanticColor::DangerLabel))
        .child(SharedString::from(message))
}

struct OnAirHandlers {
    go: Option<ShowGoOnAirHandler>,
    action: Option<ShowOnAirActionHandler>,
    toggle_information: Option<ShowGoOnAirHandler>,
    end_show: Option<ShowEndActionHandler>,
}

#[expect(
    clippy::too_many_lines,
    reason = "This function composes the rows and actions of one band."
)]
fn render_on_air_band(
    band: OnAirBandDisplay,
    handlers: OnAirHandlers,
    cx: &App,
) -> impl IntoElement {
    let OnAirBandDisplay {
        rows,
        go_available,
        go_label,
        show_information_off,
        signal,
        listeners,
        end_show_available,
        end_band,
        end_failures,
        confirmation_title: _,
        confirmation_message: _,
        end_confirmation: _,
    } = band;
    let action_handler = handlers.action;
    let rows = rows.into_iter().map(|row| {
        // One line for each row: the words at the left, the action at the right.
        let mut text = div().flex().flex_col().flex_1().min_w_0().child(
            div()
                .flex()
                .flex_row()
                .flex_wrap()
                .gap_x(Spacing::SM.scaled(cx))
                .child(Label::new(row.title).variant(LabelVariant::Body))
                .child(
                    Label::new(row.state)
                        .variant(LabelVariant::Body)
                        .color(SemanticColor::SecondaryLabel),
                ),
        );
        if row.warning {
            text = text.child(
                Label::new("Check this before you go on air.")
                    .variant(LabelVariant::Caption)
                    .color(SemanticColor::WarningLabel),
            );
        }
        if let Some(error) = row.error {
            text = text.child(
                Label::new(error)
                    .variant(LabelVariant::Caption)
                    .color(SemanticColor::DangerLabel),
            );
        }
        let mut line = div()
            .id(format!("on-air-row-{:?}", row.row))
            .flex()
            .flex_row()
            .items_center()
            .min_w_0()
            .gap(Spacing::MD.scaled(cx))
            .child(text);
        if let Some(action) = row.action {
            let handler = action_handler.clone();
            line = line.child(
                Button::plain(format!("on-air-action-{:?}", row.row))
                    .label(action.label())
                    .on_click(move |_, window, app| {
                        if let Some(handler) = &handler {
                            handler(action, window, app);
                        }
                    }),
            );
        }
        line
    });
    let callback = handlers.go;
    let toggle = handlers.toggle_information;
    let end_handler = handlers.end_show;
    let information_label = if show_information_off {
        "Show information: off. Turn on"
    } else {
        "Show information: on. Turn off"
    };
    div()
        .id("show-on-air-band")
        .flex()
        .flex_col()
        .flex_shrink_0()
        .gap(Spacing::XS.scaled(cx))
        .px(Spacing::XL.scaled(cx))
        .py(Spacing::SM.scaled(cx))
        .border_b_1()
        .border_color(color(cx, SemanticColor::Separator))
        .children(rows)
        .child(render_on_air_buttons(
            information_label,
            go_label,
            go_available,
            end_show_available,
            OnAirHandlers {
                go: callback,
                action: None,
                toggle_information: toggle,
                end_show: end_handler.clone(),
            },
            cx,
        ))
        .when_some(end_band, |el, end_band| {
            let end_now = end_handler.clone();
            let keep_on_air = end_handler.clone();
            el.child(Label::new(end_band.message).variant(LabelVariant::Headline))
                .child(
                    Label::new(format!("{} seconds left", end_band.seconds_left))
                        .variant(LabelVariant::Body),
                )
                .child(
                    Button::plain("show-end-show-now")
                        .label("End show now")
                        .on_click(move |_, window, app| {
                            if let Some(handler) = &end_now {
                                handler(EndShowAction::EndNow, window, app);
                            }
                        }),
                )
                .child(
                    Button::plain("show-keep-on-air")
                        .label("Keep on air")
                        .on_click(move |_, window, app| {
                            if let Some(handler) = &keep_on_air {
                                handler(EndShowAction::KeepOnAir, window, app);
                            }
                        }),
                )
        })
        .children(
            end_failures
                .into_iter()
                .enumerate()
                .map(|(index, failure)| {
                    let part = match failure.part {
                        crate::view_models::show::EndShowPart::Music => "The music",
                        crate::view_models::show::EndShowPart::Stream => "The stream",
                        crate::view_models::show::EndShowPart::ShowInformation => {
                            "The track names to the relay"
                        }
                    };
                    div()
                        .id(format!("end-show-failure-{index}"))
                        .flex()
                        .flex_col()
                        .child(Label::new(part).variant(LabelVariant::Headline))
                        .child(
                            Label::new(failure.error)
                                .variant(LabelVariant::Caption)
                                .color(SemanticColor::DangerLabel),
                        )
                }),
        )
        .when_some(signal, |el, signal| {
            el.child(Label::new(signal.label).variant(LabelVariant::Caption))
        })
        .when_some(listeners, |el, listeners| {
            el.child(Label::new(listeners.label).variant(LabelVariant::Caption))
        })
}

/// The switch of the show information, "Go on air" and "End show" on one line.
fn render_on_air_buttons(
    information_label: &'static str,
    go_label: &'static str,
    go_available: bool,
    end_show_available: bool,
    handlers: OnAirHandlers,
    cx: &App,
) -> impl IntoElement {
    let OnAirHandlers {
        go,
        toggle_information,
        end_show,
        action: _,
    } = handlers;
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .gap(Spacing::SM.scaled(cx))
        .pt(Spacing::XS.scaled(cx))
        .child(
            Button::filled("show-go-on-air")
                .label(go_label)
                .disabled(!go_available)
                .on_click(move |_, window, app| {
                    if let Some(callback) = &go {
                        callback(window, app);
                    }
                }),
        )
        .when(end_show_available, |el| {
            el.child(Button::plain("show-end-show").label("End show").on_click(
                move |_, window, app| {
                    if let Some(handler) = &end_show {
                        handler(EndShowAction::Confirm, window, app);
                    }
                },
            ))
        })
        .child(
            Button::plain("show-information-switch")
                .label(information_label)
                .on_click(move |_, window, app| {
                    if let Some(toggle) = &toggle_information {
                        toggle(window, app);
                    }
                }),
        )
}

fn render_show_summary(
    title: &'static str,
    state_label: &'static str,
    now_playing: Option<ShowNowPlayingDisplay>,
    empty_state: Option<ShowEmptyStateDisplay>,
    cue_playlists: Vec<ShowPlaylistRowDisplay>,
    load_playlist: Option<ShowLoadPlaylistHandler>,
    cx: &App,
) -> impl IntoElement {
    let secondary_label = color(cx, SemanticColor::SecondaryLabel);
    let tertiary_label = color(cx, SemanticColor::TertiaryLabel);

    let mut summary = div()
        .id("show-summary")
        .flex()
        .flex_col()
        .flex_shrink_0()
        .gap(Spacing::SM.scaled(cx))
        .px(Spacing::XL.scaled(cx))
        .py(Spacing::LG.scaled(cx))
        .child(
            div()
                .text_size(FontSize::Caption.scaled(cx))
                .font_weight(FontWeight::MEDIUM)
                .text_color(tertiary_label)
                .child(SharedString::from(title)),
        );

    if let Some(now_playing) = now_playing {
        summary = summary.child(render_now_playing_summary(
            now_playing,
            state_label,
            secondary_label,
            tertiary_label,
            cx,
        ));
    } else if let Some(empty_state) = empty_state {
        summary = summary.child(render_empty_summary(
            empty_state,
            state_label,
            secondary_label,
            tertiary_label,
            cx,
        ));
        if !cue_playlists.is_empty() {
            summary = summary.child(render_cue_playlists(cue_playlists, load_playlist, cx));
        }
    }

    summary
}

/// The playlists of the empty Show page. Each row is a `ListRow` with the
/// row action "Load into Show cue" (ADR 0106 Decision 4).
fn render_cue_playlists(
    rows: Vec<ShowPlaylistRowDisplay>,
    load_playlist: Option<ShowLoadPlaylistHandler>,
    cx: &App,
) -> impl IntoElement {
    div()
        .id("show-cue-playlists")
        .flex()
        .flex_col()
        .gap(Spacing::XXS.scaled(cx))
        .children(rows.into_iter().map(move |row| {
            let ShowPlaylistRowDisplay {
                playlist_id,
                element_id,
                name,
                track_count_label,
                action_label,
                action_a11y_label,
            } = row;
            let handler = load_playlist.clone();
            ListRow::new(SharedString::from(element_id.clone()))
                .a11y_label(ListRowA11yLabel {
                    label: format!("Playlist: {name}").into(),
                })
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_w_0()
                        .child(Label::new(name).truncated())
                        .child(
                            Label::new(track_count_label)
                                .variant(LabelVariant::Caption)
                                .color(SemanticColor::SecondaryLabel),
                        ),
                )
                .child(
                    Button::tinted(SharedString::from(format!("{element_id}-load")))
                        .label(action_label)
                        .a11y_label(action_a11y_label)
                        .on_click(move |_event, window, cx| {
                            if let Some(handler) = &handler {
                                handler(playlist_id, window, cx);
                            }
                        }),
                )
        }))
}

fn render_now_playing_summary(
    now_playing: ShowNowPlayingDisplay,
    state_label: &'static str,
    secondary_label: gpui::Rgba,
    tertiary_label: gpui::Rgba,
    cx: &App,
) -> impl IntoElement {
    div()
        .id("show-now-playing-summary")
        .tooltip({
            let label = SharedString::from(now_playing.a11y_label);
            move |window, cx| crate::ui::primitives::Tooltip::new(label.clone()).build(window, cx)
        })
        .flex()
        .flex_col()
        .gap(Spacing::XS.scaled(cx))
        .child(
            div()
                .text_size(FontSize::Title.scaled(cx))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(color(cx, SemanticColor::Label))
                .truncate()
                .child(SharedString::from(now_playing.title)),
        )
        .child(render_summary_subtitle(
            now_playing.artist,
            now_playing.duration_label,
            state_label,
            secondary_label,
            tertiary_label,
            cx,
        ))
}

fn render_empty_summary(
    empty_state: ShowEmptyStateDisplay,
    state_label: &'static str,
    secondary_label: gpui::Rgba,
    tertiary_label: gpui::Rgba,
    cx: &App,
) -> impl IntoElement {
    div()
        .id(empty_state.id)
        .flex()
        .flex_col()
        .gap(Spacing::XS.scaled(cx))
        .child(
            div()
                .text_size(FontSize::Title.scaled(cx))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(color(cx, SemanticColor::Label))
                .child(SharedString::from(empty_state.title)),
        )
        .when_some(empty_state.subtitle, |el, subtitle| {
            el.child(
                div()
                    .text_size(FontSize::Headline.scaled(cx))
                    .text_color(secondary_label)
                    .child(SharedString::from(subtitle)),
            )
        })
        .child(
            div()
                .text_size(FontSize::Caption.scaled(cx))
                .text_color(tertiary_label)
                .child(SharedString::from(state_label)),
        )
}

fn render_summary_subtitle(
    artist: Option<String>,
    duration_label: Option<String>,
    state_label: &'static str,
    secondary_label: gpui::Rgba,
    tertiary_label: gpui::Rgba,
    cx: &App,
) -> impl IntoElement {
    let mut subtitle = div()
        .flex()
        .flex_row()
        .gap(Spacing::SM.scaled(cx))
        .items_center()
        .min_w_0()
        .text_size(FontSize::Headline.scaled(cx))
        .text_color(secondary_label);

    if let Some(artist) = artist {
        subtitle = subtitle.child(div().min_w_0().truncate().child(SharedString::from(artist)));
    }
    if let Some(duration) = duration_label {
        subtitle = subtitle.child(
            div()
                .flex_shrink_0()
                .text_color(tertiary_label)
                .child(SharedString::from(duration)),
        );
    }

    subtitle.child(
        div()
            .flex_shrink_0()
            .text_size(FontSize::Caption.scaled(cx))
            .text_color(tertiary_label)
            .child(SharedString::from(state_label)),
    )
}

fn render_show_card_grid(
    cards: Vec<ShowCardDisplay>,
    width_class: ShowWidthClass,
    panel_mode: ShowPanelMode,
    panel_open: bool,
    slots: &ShowCardSlots,
    cx: &App,
) -> impl IntoElement {
    let select_handler = slots.select.clone();
    div()
        .id("show-card-grid")
        .flex_shrink_0()
        .grid()
        .grid_cols(width_class.columns())
        .gap(Spacing::MD.scaled(cx))
        .px(Spacing::XL.scaled(cx))
        .pb(Spacing::LG.scaled(cx))
        .children(cards.into_iter().map(move |card| {
            let selected = show_card_selected(panel_mode, panel_open, card.kind);
            let mut card = ShowCard::new(card).selected(selected);
            if let Some(handler) = select_handler.clone() {
                card = card.on_select(move |kind, event, window, cx| {
                    handler(kind, event, window, cx);
                });
            }
            card
        }))
}

fn show_card_selected(panel_mode: ShowPanelMode, panel_open: bool, kind: ShowCardKind) -> bool {
    matches!(panel_mode, ShowPanelMode::Detail(selected) if panel_open && selected == kind)
}
