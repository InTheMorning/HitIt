//! Show screen adapter.
//!
//! ADR 0060 moves queue and transport presentation into a screen mount. This
//! adapter binds existing playback projection and transport callbacks to the
//! Show shell without introducing a workspace frame.
//! ADR 0113 places event registration and retryable liveness checks inside Live Metadata.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use gpui::{AppContext as _, Context, Entity};
use gpui_component::input::{InputEvent, InputState};
use rusqlite::Connection;

use crate::application::capability::Dependency;
use crate::application::capability_recovery::{RecoveryAction, RecoveryIntent};
use crate::application::commands::playlist::ClearShowCue;
use crate::application::{ApplicationCommand, CommandContext, CommandError, CommandOutcome};
use crate::broadcast::control::{self, ServiceState, UnitRef};
use crate::broadcast::encoder::{self, EncoderTarget};
use crate::broadcast::publisher_targets::{self, PublisherTargetCommandError, PublisherTargetList};
use crate::broadcast::registry::BroadcastRegistry;
use crate::broadcast::transport::Transport;
use crate::config::BroadcastHostConfig;
use crate::presentation::present_command;
use crate::presentation::{bridge_watch, bridge_watch_with_redraw, RuntimeHost};
use crate::runtime::{
    BroadcastEncoderWatchTarget, BroadcastInstanceWatch, BroadcastReadinessSnapshot,
    BroadcastReadinessWatchHandle, BroadcastServiceRole, BroadcastServiceWatchHandle,
    BroadcastServiceWatchPace, BroadcastServiceWatchSnapshot, BroadcastServiceWatchUnit,
    BroadcastWatchInstance,
};
use crate::ui::composites::{
    open_confirmation_dialog, open_confirmation_dialog_from, ConfirmationDialogDisplay,
    ConfirmationDialogHandlers,
};
use crate::ui::shells::show::{render_show, ShowShell, ShowSlots};
use crate::view_models::queue_now_playing::QueueNowPlayingPageVm;
use crate::view_models::show::{
    EventCheckResponse, EventCommandFeedback, EventCommandState, EventControlIntent, EventKind,
    EventRegistryAction, EventRegistryInput, EventRegistryStatus, EventReportOperation,
    EventSectionInput, EventSelectionInput, EventState, EventTargetInput, EventTargetListInput,
    PublisherServiceOperation, PublisherServiceRole, ReservedAddOutcome, SelectedInstance,
    ShowCardKind, ShowCommandId, ShowLogRequestId, ShowPageVm, ShowPlayer, ShowPlayerFacts,
    StreamEncoderOperation, PUBLISHER_LOG_LINE_COUNT,
};
use crate::view_models::workspace::{FrameNavigationEntry, MusicDestination};
use crate::{config, db};

use super::queue_now_playing::{
    queue_now_playing_input, queue_now_playing_vm, queue_select_entry_action,
    queue_transport_action,
};
use super::show_mixxx::{self, MixxxStep, MixxxStepCommand, MixxxStepDone, ShowUnit};
use super::{AppTab, TopApp};

impl TopApp {
    fn confirm_go_on_air(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) {
        let Some(band) = self.show_page.on_air.as_ref() else {
            return;
        };
        if !band.go_available {
            return;
        }
        let message = if band.show_information_off {
            format!(
                "{}\nShow information: off. Podcast apps do not show this show.",
                band.confirmation_message
            )
        } else {
            band.confirmation_message.clone()
        };
        let entity = cx.weak_entity();
        open_confirmation_dialog(
            window,
            cx,
            ConfirmationDialogDisplay {
                confirm_enabled: true,
                form: None,
                section: None,
                title: band.confirmation_title.into(),
                message: message.into(),
                cancel_button_id: "show-go-on-air-cancel".into(),
                cancel_label: "Cancel".into(),
                cancel_a11y_label: "Cancel going on air".into(),
                confirm_button_id: "show-go-on-air-confirm".into(),
                confirm_label: "Go on air".into(),
                confirm_a11y_label: "Start the show stream".into(),
                destructive: false,
                items: Vec::new(),
            },
            ConfirmationDialogHandlers::new(
                |_, _: &mut gpui::App| {},
                move |_, cx: &mut gpui::App| {
                    let _ = entity.update(cx, TopApp::start_on_air_sequence);
                },
            ),
        );
    }

    fn end_show_action(
        &mut self,
        action: crate::view_models::show::EndShowAction,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        use crate::view_models::show::EndShowAction;
        match action {
            EndShowAction::Confirm => self.confirm_end_show(window, cx),
            EndShowAction::EndNow => {
                self.end_band = None;
                self.start_end_show_sequence(cx);
            }
            EndShowAction::KeepOnAir => {
                self.end_band = None;
                self.reproject_show_page_from_current_queue();
                cx.notify();
            }
        }
    }

    fn confirm_end_show(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) {
        use crate::ui::composites::{ConfirmationDialogDisplay, ConfirmationDialogItem};
        let Some(band) = self.show_page.on_air.as_ref() else {
            return;
        };
        if !band.end_show_available {
            return;
        }
        let parts = band
            .end_confirmation
            .parts
            .iter()
            .map(|part| ConfirmationDialogItem {
                check: None,
                checked_lines: Vec::new(),
                title: (*part).into(),
                subtitle: None,
                mark: None,
                lines: Vec::new(),
            })
            .collect();
        let notes = band.end_confirmation.notes.join("\n");
        let entity = cx.weak_entity();
        open_confirmation_dialog(
            window,
            cx,
            ConfirmationDialogDisplay {
                confirm_enabled: true,
                form: None,
                section: None,
                title: "End show?".into(),
                message: notes.into(),
                cancel_button_id: "show-end-show-cancel".into(),
                cancel_label: "Cancel".into(),
                cancel_a11y_label: "Keep the show on air".into(),
                confirm_button_id: "show-end-show-confirm".into(),
                confirm_label: "End show".into(),
                confirm_a11y_label: "End the show".into(),
                destructive: false,
                items: parts,
            },
            ConfirmationDialogHandlers::new(
                |_, _: &mut gpui::App| {},
                move |_, cx: &mut gpui::App| {
                    let _ = entity.update(cx, TopApp::start_end_show_sequence);
                },
            ),
        );
    }

    pub(super) fn start_end_show_sequence(&mut self, cx: &mut Context<Self>) {
        if self
            .end_show_sequence
            .as_ref()
            .is_some_and(|sequence| sequence.next_step().is_some())
            || self
                .on_air_sequence
                .as_ref()
                .is_some_and(|sequence| sequence.next_step().is_some())
        {
            return;
        }
        self.end_band = None;
        let music_is_on = self.show_page.queue.transport.play_pause_state.is_active();
        let stream_is_on = self.stream_is_connected() && self.started_parts.stream;
        let services_that_are_on = self
            .publisher_service_snapshot
            .as_ref()
            .map(|snapshot| {
                snapshot
                    .units
                    .iter()
                    .filter(|unit| {
                        unit.state == ServiceState::Active
                            && self.started_parts.services.contains(&unit.role)
                    })
                    .map(|unit| unit.role)
                    .collect()
            })
            .unwrap_or_default();
        self.end_show_sequence = Some(
            if self.show_player == ShowPlayer::Mixxx || self.started_parts.private_stopped.is_some()
            {
                crate::view_models::show::EndShowSequence::for_mixxx(
                    &self.started_parts,
                    stream_is_on,
                )
            } else {
                crate::view_models::show::EndShowSequence::new(
                    music_is_on,
                    stream_is_on,
                    services_that_are_on,
                )
            },
        );
        self.dispatch_next_end_show_step(cx);
    }

    fn dispatch_next_end_show_step(&mut self, cx: &mut Context<Self>) {
        use crate::view_models::show::EndShowStep;
        let Some(step) = self
            .end_show_sequence
            .as_ref()
            .and_then(crate::view_models::show::EndShowSequence::next_step)
        else {
            self.reproject_show_page_from_current_queue();
            cx.notify();
            return;
        };
        match step {
            EndShowStep::StopMusic => {
                self.end_show_stop_started.set(false);
                self.end_show_stop_pending.set(true);
                self.stop_show(cx);
                if !self.end_show_stop_started.get() {
                    self.end_show_stop_pending.set(false);
                    self.advance_on_air(None, Err("Show playback could not stop.".into()), cx);
                }
            }
            EndShowStep::DisconnectStream => {
                self.run_stream_encoder_command(StreamEncoderOperation::Disconnect, cx);
            }
            EndShowStep::StopService(role) => {
                self.run_publisher_service_command(role, PublisherServiceOperation::Stop, cx);
            }
            EndShowStep::StopRecording => {
                if let Some(target) = self.mixxx_encoder_target() {
                    self.run_mixxx_step(MixxxStep::StopRecording { target }, cx);
                }
            }
            EndShowStep::StopShowProducer | EndShowStep::StopShowPublisher => {
                let unit = if step == EndShowStep::StopShowProducer {
                    ShowUnit::Producer
                } else {
                    ShowUnit::Publisher
                };
                if let (Some(private_instance), Some(show_instance)) = (
                    self.private_instance_name(),
                    self.started_parts.show_instance.clone(),
                ) {
                    self.run_mixxx_step(
                        MixxxStep::StopShowUnit {
                            private_instance,
                            show_instance,
                            unit,
                        },
                        cx,
                    );
                }
            }
            EndShowStep::StartPrivateProducer => {
                let private_instance = self
                    .started_parts
                    .private_stopped
                    .clone()
                    .or_else(|| self.private_instance_name());
                if let Some(private_instance) = private_instance {
                    self.run_mixxx_step(MixxxStep::StartPrivateProducer { private_instance }, cx);
                }
            }
        }
        let waiting_for_command = self
            .end_show_sequence
            .as_ref()
            .and_then(crate::view_models::show::EndShowSequence::next_step)
            == Some(step)
            && (self.on_air_step_id.is_some()
                || self.end_show_stop_pending.get()
                || self.mixxx_step_pending);
        if self
            .end_show_sequence
            .as_ref()
            .and_then(crate::view_models::show::EndShowSequence::next_step)
            == Some(step)
            && !waiting_for_command
        {
            let reason = if self.settings_status.is_empty() {
                "This step could not start.".to_owned()
            } else {
                self.settings_status.clone()
            };
            self.advance_on_air(None, Err(reason), cx);
        }
    }

    fn private_instance_name(&self) -> Option<String> {
        self.broadcast
            .selected_host()
            .ok()
            .map(|host| host.instance_name.clone())
    }

    fn mixxx_encoder_target(&self) -> Option<EncoderTarget> {
        self.broadcast
            .encoder
            .as_ref()
            .ok()?
            .as_ref()?
            .target()
            .ok()
    }

    /// Run one step of a show that Mixxx plays. The result goes to `finish_mixxx_step`.
    fn run_mixxx_step(&mut self, step: MixxxStep, cx: &mut Context<Self>) {
        let transport = match selected_broadcast_host(&self.broadcast) {
            Ok(host) => host.transport,
            Err(error) => {
                self.mixxx_step_pending = true;
                self.finish_mixxx_step(MixxxStepDone::failed(format!("{error:#}")), cx);
                return;
            }
        };
        self.mixxx_step_pending = true;
        let command = MixxxStepCommand {
            conn: Arc::clone(&self.conn),
            transport,
            step,
        };
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            TopApp::finish_mixxx_step,
            move |this, error, cx| {
                this.finish_mixxx_step(MixxxStepDone::failed(error.to_string()), cx);
            },
        );
    }

    /// A Mixxx step ended. Update the started parts, then go to the next step.
    fn finish_mixxx_step(&mut self, done: MixxxStepDone, cx: &mut Context<Self>) {
        use crate::view_models::show::{EndShowStep, OnAirStep};
        self.mixxx_step_pending = false;
        let ok = done.result.is_ok();
        let end_step = self
            .end_show_sequence
            .as_ref()
            .and_then(crate::view_models::show::EndShowSequence::next_step);
        if let Some(step) = end_step {
            match step {
                EndShowStep::StopRecording if ok => self.started_parts.recording = false,
                EndShowStep::StopShowPublisher if ok => self.started_parts.show_instance = None,
                EndShowStep::StartPrivateProducer if ok => {
                    self.started_parts.private_stopped = None;
                }
                _ => {}
            }
        } else if let Some(OnAirStep::SwitchProducer) = self
            .on_air_sequence
            .as_ref()
            .and_then(crate::view_models::show::OnAirSequence::next_step)
        {
            if ok {
                self.started_parts.show_instance =
                    selected_show_target(&self.broadcast).map(|(name, _)| name);
            }
            // A failed switch can leave the private producer stopped. The stored fact says so.
            let stored = self
                .conn
                .lock()
                .ok()
                .and_then(|conn| show_mixxx::started_parts_from_store(&conn));
            if let Some(stored) = stored {
                self.started_parts.private_stopped = stored.private_stopped;
                if self.started_parts.show_instance.is_none() {
                    self.started_parts.show_instance = stored.show_instance;
                }
            }
        } else if let Some(OnAirStep::StartRecording) = self
            .on_air_sequence
            .as_ref()
            .and_then(crate::view_models::show::OnAirSequence::next_step)
        {
            if ok {
                self.started_parts.recording = true;
                self.started_parts.recording_started_at = done.recording_started_at;
            }
        }
        self.advance_on_air(None, done.result, cx);
        self.invalidate_publisher_service_snapshot();
    }

    /// The Check step of "Go on air" with Mixxx ended. Go on only when the event is Live.
    fn finish_mixxx_check(&mut self, cx: &mut Context<Self>) {
        if !self.mixxx_step_pending
            || self
                .on_air_sequence
                .as_ref()
                .and_then(crate::view_models::show::OnAirSequence::next_step)
                != Some(crate::view_models::show::OnAirStep::CheckEvent)
        {
            return;
        }
        let Some(input) = self.event_section_input.as_ref() else {
            return;
        };
        let result = match &input.feedback.check {
            EventCommandState::Working => return,
            EventCommandState::Failed { detail } => Err(detail.clone()),
            _ if input.liveness_confirmed
                && input
                    .selected_event
                    .as_ref()
                    .is_some_and(|event| event.state == EventState::Live) =>
            {
                Ok(())
            }
            _ => Err("The event is not live.".to_owned()),
        };
        self.mixxx_step_pending = false;
        self.advance_on_air(None, result, cx);
    }

    pub(super) fn stream_is_connected(&self) -> bool {
        self.publisher_service_snapshot
            .as_ref()
            .is_some_and(|snapshot| {
                snapshot.encoder.status.state == crate::broadcast::encoder::EncoderState::Connected
            })
    }

    fn start_on_air_sequence(&mut self, cx: &mut Context<Self>) {
        if self
            .end_show_sequence
            .as_ref()
            .is_some_and(|sequence| sequence.next_step().is_some())
        {
            return;
        }
        if let Some(sequence) = self.on_air_sequence.as_ref() {
            if sequence.failure.is_none() {
                return;
            }
            self.on_air_sequence = None;
        }
        self.end_show_sequence = None;
        let running = self
            .publisher_service_snapshot
            .as_ref()
            .map(|snapshot| {
                snapshot
                    .units
                    .iter()
                    .filter(|unit| unit.state == ServiceState::Active)
                    .map(|unit| unit.role)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        self.on_air_sequence = Some(if self.show_player == ShowPlayer::Mixxx {
            crate::view_models::show::OnAirSequence::for_mixxx()
        } else {
            crate::view_models::show::OnAirSequence::new(
                crate::view_models::show::services_to_start(&running),
                self.show_information.is_on(),
            )
        });
        self.dispatch_next_on_air_step(cx);
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one match runs each step of both players and keeps the failure handling together"
    )]
    fn dispatch_next_on_air_step(&mut self, cx: &mut Context<Self>) {
        use crate::view_models::show::OnAirStep;
        let Some(step) = self
            .on_air_sequence
            .as_ref()
            .and_then(crate::view_models::show::OnAirSequence::next_step)
        else {
            if self
                .on_air_sequence
                .as_ref()
                .is_some_and(|sequence| sequence.failure.is_none())
            {
                self.on_air_sequence = None;
            }
            self.reproject_show_page_from_current_queue();
            cx.notify();
            return;
        };
        match step {
            OnAirStep::StartService(role) => {
                self.run_publisher_service_command(role, PublisherServiceOperation::Start, cx);
            }
            OnAirStep::AttachEvent => {
                // ADR 0110 Decision 7: this step must not replace a different event.
                if let Some(message) = self
                    .event_section_input
                    .as_ref()
                    .and_then(on_air_attach_refusal)
                {
                    if let Some(input) = &mut self.event_section_input {
                        input.feedback.target_mutation = EventCommandState::Failed {
                            detail: message.clone(),
                        };
                        input.record_event_report(
                            EventReportOperation::TargetMutation,
                            chrono::Utc::now(),
                        );
                    }
                    self.advance_on_air(None, Err(message), cx);
                    return;
                }
                self.dispatch_event_target_command(EventTargetOperation::Attach, None, cx);
            }
            OnAirStep::ConnectStream => {
                if self.stream_is_connected() {
                    if let Some(sequence) = self.on_air_sequence.as_mut() {
                        sequence.complete(Ok(()));
                    }
                    self.dispatch_next_on_air_step(cx);
                    return;
                }
                self.run_stream_encoder_command(StreamEncoderOperation::Connect, cx);
            }
            OnAirStep::PlayCue => {
                self.start_on_air_cue(cx);
                return;
            }
            OnAirStep::SwitchProducer => {
                let (Some((show_instance, _)), Some(private_instance)) = (
                    selected_show_target(&self.broadcast),
                    self.private_instance_name(),
                ) else {
                    self.advance_on_air(None, Err("Make or select a show instance.".into()), cx);
                    return;
                };
                let private_was_active = show_mixxx::private_producer_was_active(
                    self.publisher_service_snapshot.as_ref(),
                    &private_instance,
                );
                self.run_mixxx_step(
                    MixxxStep::Switch {
                        private_instance,
                        show_instance,
                        private_was_active,
                    },
                    cx,
                );
            }
            OnAirStep::CheckEvent => {
                self.mixxx_step_pending = true;
                self.dispatch_event_registry_command(EventRegistryAction::Check, None, cx);
                let working = self.event_section_input.as_ref().is_some_and(|input| {
                    matches!(input.feedback.check, EventCommandState::Working)
                });
                if !working {
                    self.mixxx_step_pending = false;
                }
            }
            OnAirStep::StartRecording => {
                let (Some(target), Some((show_instance, _)), Some(private_instance)) = (
                    self.mixxx_encoder_target(),
                    selected_show_target(&self.broadcast),
                    self.private_instance_name(),
                ) else {
                    self.advance_on_air(
                        None,
                        Err("The recording could not start: no encoder or show instance.".into()),
                        cx,
                    );
                    return;
                };
                self.run_mixxx_step(
                    MixxxStep::StartRecording {
                        target,
                        private_instance,
                        show_instance,
                    },
                    cx,
                );
            }
        }
        // A dispatch that returned before it set a step id did not start the
        // command. Without this, the sequence would wait for all time.
        let still_waiting = self
            .on_air_sequence
            .as_ref()
            .and_then(crate::view_models::show::OnAirSequence::next_step)
            == Some(step)
            && self.on_air_step_id.is_none()
            && !self.mixxx_step_pending;
        if still_waiting {
            let reason = if self.settings_status.is_empty() {
                "This step could not start.".to_owned()
            } else {
                self.settings_status.clone()
            };
            self.advance_on_air(None, Err(reason), cx);
        }
    }

    fn start_on_air_cue(&mut self, cx: &mut Context<Self>) {
        use crate::view_models::show::OnAirRow;
        let action =
            self.playback_action(crate::application::capability_recovery::PlaybackOperation::Start);
        let Some(owner) = self.available_playback_owner(&action, cx) else {
            self.fail_on_air_play(
                OnAirRow::Sound,
                "Show playback is not available.".into(),
                cx,
            );
            return;
        };
        let Some(entry_id) = self.show_page.queue.selected_entry_id() else {
            self.fail_on_air_play(
                OnAirRow::ShowCue,
                "The Show cue has no selected entry.".into(),
                cx,
            );
            return;
        };
        if let Some(reason) = self.refuse_show_play(&owner, cx) {
            self.fail_on_air_play(OnAirRow::Sound, reason.into(), cx);
            return;
        }
        self.on_air_play_pending.set(true);
        self.run_playback_command(
            crate::application::commands::playback::StartShow::new(
                Arc::clone(&self.conn),
                owner,
                Some(entry_id),
            ),
            action,
            None,
            cx,
        );
    }

    /// The play step ended well. Only the play command of the run calls this.
    pub(super) fn finish_on_air_play(
        &mut self,
        result: Result<(), String>,
        cx: &mut Context<Self>,
    ) {
        if self.on_air_play_step_is_next() {
            self.advance_on_air(None, result, cx);
        }
    }

    /// The play step failed. The error goes to the row that the caller names.
    pub(super) fn fail_on_air_play(
        &mut self,
        row: crate::view_models::show::OnAirRow,
        error: String,
        cx: &mut Context<Self>,
    ) {
        if !self.on_air_play_step_is_next() {
            return;
        }
        if let Some(sequence) = self.on_air_sequence.as_mut() {
            sequence.fail(row, error);
        }
        self.on_air_step_id = None;
        self.on_air_connect_started = None;
        self.reproject_show_page_from_current_queue();
        cx.notify();
    }

    fn on_air_play_step_is_next(&self) -> bool {
        self.on_air_step_id.is_none()
            && self
                .on_air_sequence
                .as_ref()
                .and_then(crate::view_models::show::OnAirSequence::next_step)
                == Some(crate::view_models::show::OnAirStep::PlayCue)
    }

    fn toggle_show_information(&mut self, cx: &mut Context<Self>) {
        // A failed run permits the switch, so that the host can try again without the attach.
        if self
            .on_air_sequence
            .as_ref()
            .is_some_and(|sequence| sequence.failure.is_none())
        {
            return;
        }
        self.show_information = self.show_information.toggled();
        self.reproject_show_page_from_current_queue();
        cx.notify();
    }

    pub(super) fn advance_on_air(
        &mut self,
        command_id: Option<ShowCommandId>,
        result: Result<(), String>,
        cx: &mut Context<Self>,
    ) {
        if self
            .end_show_sequence
            .as_ref()
            .is_some_and(|sequence| sequence.next_step().is_some())
        {
            if let Some(expected) = self.on_air_step_id {
                if command_id != Some(expected) {
                    return;
                }
            } else if command_id.is_some() {
                return;
            }
            if let Some(sequence) = self.end_show_sequence.as_mut() {
                sequence.complete(result);
            }
            self.on_air_step_id = None;
            self.on_air_connect_started = None;
            self.reproject_show_page_from_current_queue();
            self.dispatch_next_end_show_step(cx);
            return;
        }
        if let Some(expected) = self.on_air_step_id {
            if command_id != Some(expected) {
                return;
            }
        } else if command_id.is_some() {
            return;
        }
        let Some(sequence) = self.on_air_sequence.as_mut() else {
            return;
        };
        sequence.complete(result);
        self.on_air_step_id = None;
        self.on_air_connect_started = None;
        self.reproject_show_page_from_current_queue();
        self.dispatch_next_on_air_step(cx);
    }

    fn check_on_air_connection(&mut self, cx: &mut Context<Self>) {
        let Some(command_id) = self.on_air_step_id else {
            return;
        };
        let Some(started_at) = self.on_air_connect_started else {
            return;
        };
        let connected = self
            .publisher_service_snapshot
            .as_ref()
            .is_some_and(|snapshot| {
                snapshot.encoder.status.state == crate::broadcast::encoder::EncoderState::Connected
            });
        if connected {
            self.advance_on_air(Some(command_id), Ok(()), cx);
        } else if started_at.elapsed() >= crate::view_models::show::STREAM_CONNECT_LIMIT {
            // The stream did not connect, so HitIt does not own a later connection.
            self.started_parts
                .record_stream(false, std::time::Instant::now());
            self.advance_on_air(
                Some(command_id),
                Err("The stream did not connect.".into()),
                cx,
            );
        }
    }

    fn confirm_clear_show_cue(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) {
        if !self.show_page.queue.can_clear_show_cue() {
            return;
        }
        let entity = cx.weak_entity();
        let handlers = ConfirmationDialogHandlers::new(
            |_, _: &mut gpui::App| {},
            move |window, cx: &mut gpui::App| {
                let _ = entity.update(cx, |this, cx| this.clear_show_cue(window, cx));
            },
        );
        open_confirmation_dialog(
            window,
            cx,
            ConfirmationDialogDisplay {
                confirm_enabled: true,
                form: None,
                section: None,
                title: "Clear the Show cue?".into(),
                message: "This removes the stored cue.".into(),
                cancel_button_id: "show-clear-cancel".into(),
                cancel_label: "Cancel".into(),
                cancel_a11y_label: "Keep the Show cue".into(),
                confirm_button_id: "show-clear-confirm".into(),
                confirm_label: "Clear".into(),
                confirm_a11y_label: "Clear the Show cue".into(),
                destructive: true,
                items: Vec::new(),
            },
            handlers,
        );
    }

    /// The row action "Load into Show cue" of the empty Show page. It calls
    /// the load command of the Music page (ADR 0106 Decision 4).
    fn load_playlist_into_show_cue(&mut self, playlist_id: i64, cx: &mut Context<Self>) {
        self.library
            .update(cx, |library, cx| library.load_show_cue(playlist_id, cx));
    }

    fn clear_show_cue(&mut self, _window: &mut gpui::Window, cx: &mut Context<Self>) {
        if !self.show_page.queue.can_clear_show_cue() {
            return;
        }
        // ADR 0068: the store checks the stopped state again in the Clear transaction.
        // A Clear while playback plays or is paused changes nothing and gives the reason.
        let command = ClearShowCue::new(Arc::clone(&self.conn));
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            |this, (), cx| {
                this.playback_presentation_override = None;
                this.settings_status.clear();
                this.library
                    .update(cx, crate::library::LibraryApp::reload_playlists);
                this.refresh_show_page(cx);
            },
            |this, error, cx| {
                this.settings_status = format!("Show cue clear error: {error:#}");
                this.refresh_show_page(cx);
            },
        );
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "the Show slot builder binds each callback to its existing app command"
)]
pub(super) fn build_show_screen(
    app: &TopApp,
    window_width: f32,
    cx: &mut Context<TopApp>,
) -> ShowShell {
    let entity = cx.entity();
    let service_entity = entity.clone();
    let stop_entity = entity.clone();
    let clear_entity = entity.clone();
    let move_entity = entity.clone();
    let reset_entity = entity.clone();
    let select_card_entity = entity.clone();
    let open_panel_entity = entity.clone();
    let close_panel_entity = entity.clone();
    let show_cuelist_entity = entity.clone();
    let connect_stream_entity = entity.clone();
    let disconnect_stream_entity = entity.clone();
    let readiness_entity = entity.clone();
    let change_stream_entity = entity.clone();
    let load_playlist_entity = entity.clone();
    let on_air_entity = entity.clone();
    let end_show_entity = entity.clone();
    let information_entity = entity.clone();
    let retry_entity = entity.clone();
    let slots = with_log_slots(ShowSlots::new().log_frames(app.log_frames.clone()), &entity);
    let slots = with_event_registry_slots(slots, &entity);
    render_show(
        app.show_page.clone().with_window_width(window_width),
        slots
            .on_skip_previous(queue_transport_action(
                entity.clone(),
                TopApp::skip_playback_previous,
            ))
            .on_play_pause(queue_transport_action(
                entity.clone(),
                TopApp::toggle_playback_paused,
            ))
            .on_stop_show(queue_transport_action(entity.clone(), TopApp::stop_show))
            .on_clear_cue(move |_, window, cx| {
                clear_entity.update(cx, |this, cx| this.confirm_clear_show_cue(window, cx));
            })
            .on_skip_next(queue_transport_action(
                entity.clone(),
                TopApp::skip_playback_next,
            ))
            .on_select_cue_entry(queue_select_entry_action(entity.clone()))
            .on_move_cue_entry(move |entry_id, target, _, cx| {
                move_entity.update(cx, |this, cx| {
                    this.move_show_cue_entry(entry_id, target, cx);
                });
            })
            .on_load_playlist(move |playlist_id, _, cx| {
                load_playlist_entity.update(cx, |this, cx| {
                    this.load_playlist_into_show_cue(playlist_id, cx);
                });
            })
            .on_go_on_air(move |window, cx| {
                on_air_entity.update(cx, |this, cx| this.confirm_go_on_air(window, cx));
            })
            .on_toggle_show_information(move |_, cx| {
                information_entity.update(cx, TopApp::toggle_show_information);
            })
            .on_retry_show(move |_, _, cx| {
                retry_entity.update(cx, |this, cx| {
                    if this
                        .on_air_sequence
                        .as_ref()
                        .is_some_and(|sequence| sequence.failure.is_some())
                    {
                        this.start_on_air_sequence(cx);
                    } else {
                        this.start_end_show_sequence(cx);
                    }
                });
            })
            .on_end_show(move |action, window, cx| {
                end_show_entity.update(cx, |this, cx| this.end_show_action(action, window, cx));
            })
            .on_open_broadcast_readiness(move |_, _, cx| {
                readiness_entity.update(cx, |this, cx| {
                    this.open_broadcast_readiness_in_music(cx);
                });
            })
            .on_change_stream(move |_, window, cx| {
                change_stream_entity.update(cx, |this, cx| {
                    this.settings_action(
                        crate::view_models::settings::SettingsAction::OpenRepair,
                        window,
                        cx,
                    );
                    this.select_tab(AppTab::Settings, window, cx);
                });
            })
            .on_select_card(move |kind, _, _, cx| {
                select_card_entity.update(cx, |this, cx| {
                    this.select_show_card_detail(kind, cx);
                });
            })
            .on_open_show_panel(move |_, _, cx| {
                open_panel_entity.update(cx, |this, cx| {
                    this.open_show_panel(cx);
                });
            })
            .on_close_show_panel(move |_, _, cx| {
                close_panel_entity.update(cx, |this, cx| {
                    this.close_show_panel(cx);
                });
            })
            .on_show_cuelist(move |_, _, cx| {
                show_cuelist_entity.update(cx, |this, cx| {
                    this.show_cuelist_panel(cx);
                });
            })
            .on_start_publisher_service(move |role, _, _, cx| {
                service_entity.update(cx, |this, cx| {
                    this.run_publisher_service_command(role, PublisherServiceOperation::Start, cx);
                });
            })
            .on_stop_publisher_service(move |role, _, window, cx| {
                stop_entity.update(cx, |this, cx| {
                    this.request_publisher_stop(role, window, cx);
                });
            })
            .on_reset_publisher_service(move |role, _, _, cx| {
                reset_entity.update(cx, |this, cx| {
                    this.run_publisher_service_command(role, PublisherServiceOperation::Reset, cx);
                });
            })
            .on_connect_stream(move |_, _, cx| {
                connect_stream_entity.update(cx, |this, cx| {
                    this.run_stream_encoder_command(StreamEncoderOperation::Connect, cx);
                });
            })
            .on_disconnect_stream(move |_, _, cx| {
                disconnect_stream_entity.update(cx, |this, cx| {
                    this.run_stream_encoder_command(StreamEncoderOperation::Disconnect, cx);
                });
            }),
    )
}

/// Binds the event row's explicit registry actions (ADR 0113).
fn with_event_registry_slots(slots: ShowSlots, entity: &Entity<TopApp>) -> ShowSlots {
    let select_event_entity = entity.clone();
    let event_logs_entity = entity.clone();
    let event_control_entity = entity.clone();
    let player_entity = entity.clone();
    let instance_entity = entity.clone();
    let make_instance_entity = entity.clone();
    slots
        .on_select_player(move |player, _, cx| {
            player_entity.update(cx, |this, cx| this.set_show_player(player, cx));
        })
        .on_select_show_instance(move |name, _, cx| {
            instance_entity.update(cx, |this, cx| this.select_show_instance(name, cx));
        })
        .on_make_instance(move |_, window, cx| {
            make_instance_entity.update(cx, |this, cx| this.open_make_instance_dialog(window, cx));
        })
        .on_select_event(move |event_id, _, cx| {
            select_event_entity.update(cx, |this, cx| this.select_show_event(event_id, cx));
        })
        .on_open_event_logs(move |_, _, cx| {
            event_logs_entity.update(cx, |this, cx| {
                if this.show_page.log_pane.shows_event() {
                    this.close_publisher_logs(cx);
                } else if let Some(input) = &this.event_section_input {
                    this.show_page.log_pane.show_event(input);
                    this.reproject_show_page_from_current_queue();
                    cx.notify();
                }
            });
        })
        .on_event_control(move |intent, _, window, cx| {
            event_control_entity.update(cx, |this, cx| match intent {
                EventControlIntent::AddReserved => this.open_add_reserved_dialog(window, cx),
                EventControlIntent::Attach => this.attach_event_after_question(window, cx),
                _ => this.run_event_control(intent, cx),
            });
        })
}

/// Binds independent log reads and shared split resizing to the Show owner (ADR 0063).
fn with_log_slots(slots: ShowSlots, entity: &Entity<TopApp>) -> ShowSlots {
    let logs_entity = entity.clone();
    let close_logs_entity = entity.clone();
    let log_layout_entity = entity.clone();
    let log_resize_start_entity = entity.clone();
    let log_resize_move_entity = entity.clone();
    let log_resize_end_entity = entity.clone();
    slots
        .on_open_publisher_logs(move |role, _, _, cx| {
            logs_entity.update(cx, |this, cx| {
                this.open_publisher_logs(role, cx);
            });
        })
        .on_close_publisher_logs(move |_, _, cx| {
            close_logs_entity.update(cx, |this, cx| {
                this.close_publisher_logs(cx);
            });
        })
        .on_log_layout(move |height, handle_height, _, cx| {
            log_layout_entity.update(cx, |this, cx| {
                if this
                    .show_page
                    .log_pane
                    .update_geometry(height, handle_height)
                {
                    cx.notify();
                }
            });
        })
        .on_log_resize_start(move |event, _, cx| {
            // ADR 0039: un-scaling pointer drag Y is geometry — CHROME domain.
            let scale = crate::ui::tokens::ScaleFactor::current(cx).chrome_multiplier();
            log_resize_start_entity.update(cx, |this, _| {
                this.show_log_resize = Some((
                    f32::from(event.position.y) / scale,
                    this.show_page.log_pane.height,
                ));
            });
        })
        .on_log_resize_move(move |event, _, cx| {
            // ADR 0039: un-scaling pointer drag Y is geometry — CHROME domain.
            let scale = crate::ui::tokens::ScaleFactor::current(cx).chrome_multiplier();
            log_resize_move_entity.update(cx, |this, cx| {
                if !event.dragging() {
                    this.show_log_resize = None;
                } else if let Some((start_y, start_height)) = this.show_log_resize {
                    this.show_page
                        .log_pane
                        .resize(start_height + start_y - f32::from(event.position.y) / scale);
                    cx.notify();
                }
            });
        })
        .on_log_resize_end(move |_, _, cx| {
            log_resize_end_entity.update(cx, |this, _| this.show_log_resize = None);
        })
}

impl TopApp {
    pub(super) fn maybe_start_broadcast_readiness_watch(&mut self, cx: &mut Context<Self>) {
        if self.broadcast_readiness_watch.is_some() {
            return;
        }
        let Some(host) = self.runtime_host.clone() else {
            return;
        };

        let handle = start_broadcast_readiness_watch(&host, Arc::clone(&self.conn));
        self.broadcast_readiness_snapshot = Some(handle.latest());
        bridge_watch(
            handle.subscribe(),
            |this: &mut Self, snapshot, cx| {
                this.apply_broadcast_readiness_snapshot(snapshot, cx);
            },
            cx,
        );
        self.broadcast_readiness_watch = Some(handle);
    }

    pub(super) fn maybe_start_broadcast_service_watch(&mut self, cx: &mut Context<Self>) {
        if self.publisher_service_watch.is_some() {
            return;
        }
        let Some(host) = self.runtime_host.clone() else {
            return;
        };

        let (units, encoder) = broadcast_watch_inputs(&self.broadcast);
        if self.publisher_watch_generation == 0 {
            self.show_commands.clear();
        }
        self.publisher_watch_generation += 1;
        let generation = self.publisher_watch_generation;
        let instances = broadcast_instance_watch(&self.broadcast);
        let handle = start_broadcast_service_watch(&host, units, instances, encoder);
        self.publisher_service_snapshot = Some(handle.latest());
        bridge_watch_with_redraw(
            handle.subscribe(),
            move |this: &mut Self, snapshot, cx| {
                if this.publisher_watch_generation != generation {
                    return false;
                }
                this.apply_publisher_service_snapshot(snapshot, cx)
            },
            cx,
        );
        self.publisher_service_watch = Some(handle);
        // The fresh watch always starts at Background; correct it for the
        // current section and any command still waiting (ADR 0113).
        self.broadcast_service_watch_pace = BroadcastServiceWatchPace::Background;
        self.sync_broadcast_service_watch_pace();
    }

    /// Send the broadcast service watch the pace its current viewer needs
    /// (ADR 0113). A command resolving outside Show can still lower the
    /// pace, so this also runs after each snapshot's `observe`.
    pub(super) fn sync_broadcast_service_watch_pace(&mut self) {
        let pace = crate::view_models::show::broadcast_service_watch_pace(
            self.tab == AppTab::Show,
            &self.show_commands,
        );
        if pace == self.broadcast_service_watch_pace {
            return;
        }
        self.broadcast_service_watch_pace = pace;
        if let Some(handle) = self.publisher_service_watch.as_ref() {
            let _ = handle.set_pace(pace);
        }
    }

    pub(super) fn refresh_show_page(&self, cx: &mut Context<Self>) {
        let requested_event_input = self.event_section_input.clone();
        let command = RefreshShowPage::new(
            Arc::clone(&self.conn),
            self.music_dir.clone(),
            self.queue_text_filter.clone(),
            self.broadcast.clone(),
        );
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, projection, cx| {
                let applied = apply_event_refresh(
                    &mut this.event_section_input,
                    requested_event_input.as_ref(),
                    projection.event_section_input,
                );
                this.show_playlists = projection.playlists;
                this.reproject_show_page(projection.queue);
                let selection_changed = requested_event_input.as_ref().map(event_input_identity)
                    != this.event_section_input.as_ref().map(event_input_identity);
                if applied
                    && selection_changed
                    && this
                        .event_section_input
                        .as_ref()
                        .is_some_and(|input| input.selected_event.is_some())
                {
                    this.run_event_registry_command(EventRegistryAction::Check, cx);
                }
            },
            |this, error, _cx| {
                this.settings_status = format!("Show status error: {error:#}");
            },
        );
    }

    /// Apply one service-watch snapshot. Returns whether the Show page must
    /// redraw for it: a changed unit or encoder state, a changed command
    /// progress, or a started log read (ADR 0113). A changed read time alone
    /// never redraws.
    fn apply_publisher_service_snapshot(
        &mut self,
        snapshot: BroadcastServiceWatchSnapshot,
        cx: &mut Context<Self>,
    ) -> bool {
        let command_progress_changed = self.show_commands.observe(&snapshot);
        self.sync_broadcast_service_watch_pace();
        let previous_snapshot = self.publisher_service_snapshot.replace(snapshot);
        let stream_on = self.stream_is_connected();
        let services_on = self
            .publisher_service_snapshot
            .as_ref()
            .map(|snapshot| {
                snapshot
                    .units
                    .iter()
                    .filter(|unit| unit.state == ServiceState::Active)
                    .map(|unit| unit.role)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        self.started_parts
            .reconcile(stream_on, &services_on, std::time::Instant::now());
        self.check_on_air_connection(cx);
        self.reproject_show_page_from_current_queue();
        let mut log_read_started = false;
        if self.tab == AppTab::Show {
            if let Some((role, request)) = self.show_page.log_pane.refresh_request() {
                self.read_publisher_logs(role, request, cx);
                log_read_started = true;
            }
        }
        let next = self
            .publisher_service_snapshot
            .as_ref()
            .expect("the snapshot this call just stored");
        crate::view_models::show::broadcast_service_snapshot_redraw(
            previous_snapshot.as_ref(),
            next,
            command_progress_changed,
            log_read_started,
        )
    }

    fn apply_broadcast_readiness_snapshot(
        &mut self,
        snapshot: BroadcastReadinessSnapshot,
        cx: &mut Context<Self>,
    ) {
        let mounted_report = self
            .music_readiness_list_is_current()
            .then(|| snapshot.report.clone())
            .flatten();
        self.broadcast_readiness_snapshot = Some(snapshot);
        self.sync_inbox_setup(cx);
        if let Some(report) = mounted_report {
            self.library.update(cx, |library, cx| {
                library.show_broadcast_readiness_report(&report, cx);
            });
        }
        self.reproject_show_page_from_current_queue();
        cx.notify();
    }

    fn reproject_show_page(&mut self, mut queue: QueueNowPlayingPageVm) {
        if let Some(state) = self.playback_presentation_override {
            queue.set_transport_state(state);
        }
        if let Some(input) = &self.event_section_input {
            self.event_session.remember(input);
            // ADR 0101 Decision 7: each show event report goes to the app
            // event list, which the status bar shows.
            self.app_events.record_show_reports(input);
            if self.show_page.log_pane.shows_event() {
                self.show_page.log_pane.show_event(input);
            }
        }
        let panel_mode = self.show_page.panel_mode;
        let panel_open = self.show_page.panel_open;
        let private_instance = self.private_instance_name().unwrap_or_default();
        let selected_host = self.broadcast.selected_host().ok();
        let producer = selected_host.map(|host| host.producer).unwrap_or_default();
        let instance_names: Vec<String> = selected_host
            .map(|host| {
                host.show_instances
                    .iter()
                    .map(|instance| instance.name.clone())
                    .collect()
            })
            .unwrap_or_default();
        let show_selection = match self.broadcast.selected_show_instance.as_deref() {
            None => SelectedInstance::None,
            Some(name) if name == private_instance => SelectedInstance::Private(name),
            Some(name) => SelectedInstance::Show(name),
        };
        let drop_problem = self
            .broadcast
            .show_drop_problem(config::runtime_dir_from_env().as_deref());
        let music_is_on = queue.transport.play_pause_state.is_active();
        let services_that_are_on = self
            .publisher_service_snapshot
            .as_ref()
            .map(|snapshot| {
                snapshot
                    .units
                    .iter()
                    .filter(|unit| unit.state == ServiceState::Active)
                    .map(|unit| unit.role)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        self.show_page = ShowPageVm::from_queue_publisher_readiness_and_event(
            queue,
            self.publisher_service_snapshot.as_ref(),
            self.show_page.log_pane.clone(),
            self.broadcast_readiness_snapshot.as_ref(),
            self.event_section_input.as_ref(),
        )
        .with_command_state(&self.show_commands)
        .with_status_message(&self.settings_status)
        .with_execution_availability(self.command_runner.availability())
        .with_feature_availability(
            self.feature_availability(),
            &self.capability_observations.snapshot(),
        )
        .with_panel_state(panel_mode, panel_open)
        .with_cue_playlists(crate::view_models::playlist_curation::show_playlist_rows(
            &self.show_playlists,
        ))
        .with_on_air_band(crate::view_models::show::OnAirFacts {
            player_is_mpv: self.audio_driver == config::PlaybackDriver::Mpv,
            mpv_ping_succeeded: matches!(
                self.mpv_status,
                crate::view_models::audio_setup::MpvStatus::Installed(_)
            ),
            show_information: self.show_information,
            sequence: self.on_air_sequence.as_ref(),
            run_active: self
                .on_air_sequence
                .as_ref()
                .is_some_and(|sequence| sequence.next_step().is_some())
                || self
                    .end_show_sequence
                    .as_ref()
                    .is_some_and(|sequence| sequence.next_step().is_some()),
            end: crate::view_models::show::EndShowFacts {
                music_is_on,
                stream_is_on: self.stream_is_connected(),
                services_that_are_on: &services_that_are_on,
                started: &self.started_parts,
                sequence: self.end_show_sequence.as_ref(),
                end_band: self.end_band,
                now: std::time::Instant::now(),
            },
            started: &self.started_parts,
            show_player: ShowPlayerFacts {
                player: self.show_player,
                selected: show_selection,
                private_instance: &private_instance,
                instances: &instance_names,
                drop_folder_problem: drop_problem.as_deref(),
                producer,
            },
        });
    }

    pub(super) fn reproject_show_page_from_current_queue(&mut self) {
        self.capability_vm.repair_blocked = self.show_commands.repair_blocked();
        self.reproject_show_page(self.show_page.queue.clone());
    }

    fn select_show_card_detail(&mut self, kind: ShowCardKind, cx: &mut Context<Self>) {
        self.show_page = self.show_page.clone().select_card(kind);
        cx.notify();
    }

    fn open_show_panel(&mut self, cx: &mut Context<Self>) {
        self.show_page = self.show_page.clone().open_panel();
        cx.notify();
    }

    fn close_show_panel(&mut self, cx: &mut Context<Self>) {
        self.show_page = self.show_page.clone().close_panel();
        cx.notify();
    }

    fn show_cuelist_panel(&mut self, cx: &mut Context<Self>) {
        self.show_page = self.show_page.clone().show_cuelist_panel();
        cx.notify();
    }

    /// Stop of a station unit asks first (ADR 0112 Decision 4). Another host stops at once.
    fn request_publisher_stop(
        &mut self,
        role: PublisherServiceRole,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        use crate::ui::composites::ConfirmationDialogDisplay;
        let Some(question) = self
            .show_page
            .publisher
            .as_ref()
            .and_then(|section| section.stop_confirmation.clone())
        else {
            self.run_publisher_service_command(role, PublisherServiceOperation::Stop, cx);
            return;
        };
        let entity = cx.weak_entity();
        open_confirmation_dialog(
            window,
            cx,
            ConfirmationDialogDisplay {
                confirm_enabled: true,
                form: None,
                section: None,
                title: question.title.into(),
                message: question.message.into(),
                cancel_button_id: "show-station-stop-cancel".into(),
                cancel_label: "Cancel".into(),
                cancel_a11y_label: "Keep the station unit running".into(),
                confirm_button_id: "show-station-stop-confirm".into(),
                confirm_label: "Stop".into(),
                confirm_a11y_label: format!(
                    "Stop the {} of station instance {}",
                    role_label(role),
                    question.instance
                )
                .into(),
                destructive: true,
                items: Vec::new(),
            },
            ConfirmationDialogHandlers::new(
                |_, _: &mut gpui::App| {},
                move |_, cx: &mut gpui::App| {
                    let _ = entity.update(cx, |this, cx| {
                        this.run_publisher_service_command(
                            role,
                            PublisherServiceOperation::Stop,
                            cx,
                        );
                    });
                },
            ),
        );
    }

    fn run_publisher_service_command(
        &mut self,
        role: PublisherServiceRole,
        operation: PublisherServiceOperation,
        cx: &mut Context<Self>,
    ) {
        let action = RecoveryAction::Publisher {
            role,
            operation,
            host: self.broadcast.selected_host().ok().cloned(),
            event_id: self.event_section_input.as_ref().and_then(|input| {
                input
                    .selected_event
                    .as_ref()
                    .map(|event| event.event_id.clone())
            }),
        };
        if let Err(reason) = self.feature_availability().require(Dependency::Publisher) {
            self.retain_failed_action(action, reason.dependency, cx);
            self.settings_status = reason.to_string();
            return;
        }
        let selected_host = match selected_broadcast_host(&self.broadcast) {
            Ok(host) => host,
            Err(error) => {
                self.settings_status = format!("Relay sender command error: {error:#}");
                cx.notify();
                return;
            }
        };
        let command = match PublisherServiceCommand::new(
            &selected_host,
            role,
            operation,
            self.broadcast.selected_show_instance.as_deref(),
        ) {
            Ok(command) => command,
            Err(error) => {
                self.settings_status = format!("Relay sender command error: {error}");
                cx.notify();
                return;
            }
        };
        self.dispatch_service_command(role, operation, command, action, None, cx);
    }

    fn dispatch_service_command<C>(
        &mut self,
        role: PublisherServiceRole,
        operation: PublisherServiceOperation,
        command: C,
        action: RecoveryAction,
        retry_id: Option<u64>,
        cx: &mut Context<Self>,
    ) where
        C: ApplicationCommand<Output = ShowCommandCompletion>,
    {
        let Some(command_id) = self.show_commands.begin_service(role, operation) else {
            if let Some(id) = retry_id {
                self.capability_vm.pending.finish(id, "App is already running a command for this relay sender. Check again when it finishes.".into());
            }
            return;
        };
        let on_air_start = self
            .on_air_sequence
            .as_ref()
            .and_then(crate::view_models::show::OnAirSequence::next_step)
            == Some(crate::view_models::show::OnAirStep::StartService(role));
        let end_show_stop = operation == PublisherServiceOperation::Stop
            && self
                .end_show_sequence
                .as_ref()
                .and_then(crate::view_models::show::EndShowSequence::next_step)
                == Some(crate::view_models::show::EndShowStep::StopService(role));
        if on_air_start || end_show_stop {
            self.on_air_step_id = Some(command_id);
        }
        self.dispatch_show_command(command, command_id, action, retry_id, cx);
    }

    fn dispatch_show_command<C>(
        &mut self,
        command: C,
        command_id: ShowCommandId,
        action: RecoveryAction,
        retry_id: Option<u64>,
        cx: &mut Context<Self>,
    ) where
        C: ApplicationCommand<Output = ShowCommandCompletion>,
    {
        // ADR 0113: a command that begins outside Show, for example from a
        // Settings recovery action, needs the Viewed pace at once. Else its
        // first fresh read can come only after the Background interval.
        self.sync_broadcast_service_watch_pace();
        self.settings_status.clear();
        self.reproject_show_page_from_current_queue();
        let error_action = action.clone();
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, completion, cx| {
                this.record_show_retry_result(action, retry_id, &completion.result, cx);
                this.finish_show_command(command_id, completion, cx);
            },
            move |this, error, cx| {
                let completion = ShowCommandCompletion::new(Err(error));
                this.record_show_retry_result(error_action, retry_id, &completion.result, cx);
                this.finish_show_command(command_id, completion, cx);
            },
        );
    }

    fn record_show_retry_result(
        &mut self,
        action: RecoveryAction,
        retry_id: Option<u64>,
        result: &Result<(), CommandError>,
        cx: &mut Context<Self>,
    ) {
        if let Some(id) = retry_id {
            match result {
                Ok(()) => self.capability_vm.pending.succeed(id, "App completed the original service command. Check the service report for its observed state.".into()),
                Err(error) => self.capability_vm.pending.finish(id, crate::diagnostics::redact_endpoint_details(&format!("App could not complete the original service command: {error}"))),
            }
        } else if result.is_err() {
            let dependency = action.dependency();
            self.retain_failed_action(action, dependency, cx);
        }
    }

    pub(super) fn retry_show_action(&mut self, intent: RecoveryIntent, cx: &mut Context<Self>) {
        let id = intent.id;
        match intent.action.clone() {
            RecoveryAction::Publisher {
                role, operation, ..
            } => {
                let command = self
                    .broadcast
                    .selected_host()
                    .map_err(publisher_command_error)
                    .and_then(|host| {
                        PublisherServiceCommand::new(
                            host,
                            role,
                            operation,
                            self.broadcast.selected_show_instance.as_deref(),
                        )
                    });
                match command {
                    Ok(command) => {
                        let action = intent.action.clone();
                        self.dispatch_service_command(
                            role,
                            operation,
                            self.retry_command(command, intent),
                            action,
                            Some(id),
                            cx,
                        );
                    }
                    Err(error) => self.capability_vm.pending.finish(id, error.to_string()),
                }
            }
            RecoveryAction::Encoder { operation, .. } => {
                match StreamEncoderCommand::new(&self.broadcast, operation) {
                    Ok(command) => {
                        let Some(command_id) = self.show_commands.begin_stream(operation) else {
                            self.capability_vm
                                .pending
                                .finish(id, "App is already running an encoder command.".into());
                            return;
                        };
                        let action = intent.action.clone();
                        self.dispatch_show_command(
                            self.retry_command(command, intent),
                            command_id,
                            action,
                            Some(id),
                            cx,
                        );
                    }
                    Err(error) => self.capability_vm.pending.finish(id, error.to_string()),
                }
            }
            RecoveryAction::Event { operation, .. } => match operation {
                EventControlIntent::Create => self.dispatch_event_registry_command(
                    EventRegistryAction::Create,
                    Some(intent),
                    cx,
                ),
                EventControlIntent::Replace => self.dispatch_event_registry_command(
                    EventRegistryAction::Replace,
                    Some(intent),
                    cx,
                ),
                EventControlIntent::Check => self.dispatch_event_registry_command(
                    EventRegistryAction::Check,
                    Some(intent),
                    cx,
                ),
                EventControlIntent::Attach => self.dispatch_event_target_command(
                    EventTargetOperation::Attach,
                    Some(intent),
                    cx,
                ),
                EventControlIntent::Detach => self.dispatch_event_target_command(
                    EventTargetOperation::Detach,
                    Some(intent),
                    cx,
                ),
                _ => self.capability_vm.pending.finish(
                    id,
                    "Use the current event's Check action to refresh its report.".into(),
                ),
            },
            _ => {}
        }
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "the async completion callback owns the command result"
    )]
    fn finish_show_command(
        &mut self,
        command_id: ShowCommandId,
        completion: ShowCommandCompletion,
        cx: &mut Context<Self>,
    ) {
        if completion.result.is_ok() {
            if let Some((role, started)) = self.show_commands.intent(command_id) {
                if let Some(role) = role {
                    self.started_parts.record_service(role, started);
                } else if started {
                    self.started_parts
                        .record_stream(true, std::time::Instant::now());
                } else {
                    self.started_parts.release_stream(std::time::Instant::now());
                }
            }
        }
        if !self.show_commands.complete(
            command_id,
            completion.returned_at,
            completion.result.is_ok(),
        ) {
            return;
        }
        self.settings_status = completion
            .result
            .as_ref()
            .err()
            .map_or_else(String::new, ToString::to_string);
        self.reproject_show_page_from_current_queue();
        self.invalidate_publisher_service_snapshot();
        let is_on_air_connect = self
            .on_air_sequence
            .as_ref()
            .and_then(crate::view_models::show::OnAirSequence::next_step)
            == Some(crate::view_models::show::OnAirStep::ConnectStream)
            && self.on_air_step_id == Some(command_id);
        if is_on_air_connect && completion.result.is_ok() {
            self.on_air_connect_started = Some(std::time::Instant::now());
            self.check_on_air_connection(cx);
        } else {
            self.advance_on_air(
                Some(command_id),
                completion
                    .result
                    .as_ref()
                    .copied()
                    .map_err(ToString::to_string),
                cx,
            );
        }
        cx.notify();
    }

    fn open_publisher_logs(&mut self, role: PublisherServiceRole, cx: &mut Context<Self>) {
        self.show_log_resize = None;
        let Some(request_id) = self.show_page.toggle_publisher_logs(role) else {
            cx.notify();
            return;
        };
        cx.notify();
        self.read_publisher_logs(role, request_id, cx);
    }

    fn read_publisher_logs(
        &mut self,
        role: PublisherServiceRole,
        request_id: ShowLogRequestId,
        cx: &mut Context<Self>,
    ) {
        let selected_host = match selected_broadcast_host(&self.broadcast) {
            Ok(host) => host,
            Err(error) => {
                self.show_page.log_pane.apply_result(
                    request_id,
                    Err(format!("Relay sender log error: {error:#}")),
                );
                cx.notify();
                return;
            }
        };
        self.show_page.log_pane.bind_service_source(&selected_host);
        let instance = service_instance(
            &selected_host,
            self.broadcast.selected_show_instance.as_deref(),
        );
        let command = match ReadPublisherLogs::new(&selected_host, role, request_id, &instance) {
            Ok(command) => command,
            Err(error) => {
                self.show_page
                    .log_pane
                    .apply_result(request_id, Err(format!("Relay sender log error: {error}")));
                cx.notify();
                return;
            }
        };
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            |this, logs, cx| {
                if this
                    .show_page
                    .log_pane
                    .apply_result(logs.request_id, Ok(logs.text))
                {
                    cx.notify();
                }
            },
            move |this, error, cx| {
                if this.show_page.log_pane.apply_result(
                    request_id,
                    Err(format!("Relay sender log error: {error:#}")),
                ) {
                    cx.notify();
                }
            },
        );
    }

    fn close_publisher_logs(&mut self, cx: &mut Context<Self>) {
        self.show_page.close_publisher_logs();
        self.show_log_resize = None;
        cx.notify();
    }

    fn run_event_registry_command(&mut self, action: EventRegistryAction, cx: &mut Context<Self>) {
        self.dispatch_event_registry_command(action, None, cx);
    }

    fn event_recovery_action(&self, operation: EventControlIntent) -> Option<RecoveryAction> {
        let input = self.event_section_input.as_ref()?;
        let target = match operation {
            EventControlIntent::Attach => {
                event_target_name_for_operation(EventTargetOperation::Attach, input).ok()
            }
            EventControlIntent::Detach => {
                event_target_name_for_operation(EventTargetOperation::Detach, input).ok()
            }
            _ => None,
        };
        Some(RecoveryAction::Event {
            operation,
            event_id: input
                .selected_event
                .as_ref()
                .map(|event| event.event_id.clone()),
            selection_revision: input.registry.revision,
            host: self.broadcast.selected_host().ok().cloned(),
            target,
        })
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one adapter keeps readiness, feedback, retry and the Mixxx check hook together"
    )]
    fn dispatch_event_registry_command(
        &mut self,
        action: EventRegistryAction,
        retry: Option<RecoveryIntent>,
        cx: &mut Context<Self>,
    ) {
        let Some(original) = retry
            .as_ref()
            .map(|intent| intent.action.clone())
            .or_else(|| {
                self.event_recovery_action(match action {
                    EventRegistryAction::Create => EventControlIntent::Create,
                    EventRegistryAction::Replace => EventControlIntent::Replace,
                    EventRegistryAction::Check => EventControlIntent::Check,
                })
            })
        else {
            return;
        };
        let retry_id = retry.as_ref().map(|intent| intent.id);
        if let Err(reason) = self.feature_availability().require(original.dependency()) {
            if let Some(id) = retry_id {
                self.capability_vm.pending.finish(id, reason.to_string());
            } else {
                self.retain_failed_action(original, reason.dependency, cx);
            }
            return;
        }

        let available = self
            .show_page
            .publisher
            .as_ref()
            .and_then(|section| section.event.as_ref())
            .is_some_and(|event| !event.actions.registry_action(action).disabled());
        if !available {
            if let Some(id) = retry_id {
                self.capability_vm.pending.finish(id, "App did not retry this event action because its current readiness no longer permits it. Check the original event.".into());
            }
            return;
        }
        let Some(input) = self.event_section_input.as_mut() else {
            return;
        };
        self.event_session.next_request += 1;
        input.request = self.event_session.next_request;
        let request = input.request;
        let request_context = input.context.clone();
        let error_context = request_context.clone();
        let command = EventRegistryCommand {
            selection_revision: input.registry.revision,
            conn: Arc::clone(&self.conn),
            cfg_path: self.cfg_path.clone(),
            action,
            selected_event: input.selected_event.clone(),
        };
        begin_event_registry_action(input, action);
        let command = self.command_with_retry(command, retry);
        if action == EventRegistryAction::Check {
            self.read_event_targets(request, cx);
        }
        self.reproject_show_page_from_current_queue();
        cx.notify();
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, event, cx| {
                if let Some(id) = retry_id {
                    record_event_retry_result(&mut this.capability_vm.pending, id, &event);
                }
                if !this.event_request_is_current(request, &request_context) {
                    return;
                }
                let Some(input) = this.event_section_input.as_mut() else {
                    return;
                };
                let next = apply_event_registry_result(input, action, Ok(event));
                this.reproject_show_page_from_current_queue();
                cx.notify();
                if let Some(next) = next {
                    this.run_event_registry_command(next, cx);
                } else {
                    this.finish_mixxx_check(cx);
                }
            },
            move |this, error, cx| {
                if let Some(id) = retry_id {
                    this.capability_vm.pending.finish(
                        id,
                        crate::diagnostics::redact_endpoint_details(&error.to_string()),
                    );
                } else {
                    let dependency = original.dependency();
                    this.retain_failed_action(original, dependency, cx);
                }
                if !this.event_request_is_current(request, &error_context) {
                    return;
                }
                if let Some(input) = this.event_section_input.as_mut() {
                    apply_event_registry_result(input, action, Err(error));
                }
                this.reproject_show_page_from_current_queue();
                cx.notify();
                this.finish_mixxx_check(cx);
            },
        );
    }

    fn run_event_target_command(
        &mut self,
        operation: EventTargetOperation,
        cx: &mut Context<Self>,
    ) {
        self.dispatch_event_target_command(operation, None, cx);
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one event-target adapter keeps existing readiness, command feedback and retry completion together"
    )]
    fn dispatch_event_target_command(
        &mut self,
        operation: EventTargetOperation,
        retry: Option<RecoveryIntent>,
        cx: &mut Context<Self>,
    ) {
        let Some(original) = retry
            .as_ref()
            .map(|intent| intent.action.clone())
            .or_else(|| {
                self.event_recovery_action(match operation {
                    EventTargetOperation::Attach => EventControlIntent::Attach,
                    EventTargetOperation::Detach => EventControlIntent::Detach,
                })
            })
        else {
            return;
        };
        let retry_id = retry.as_ref().map(|intent| intent.id);
        if let Err(reason) = self.feature_availability().require(Dependency::Publisher) {
            if let Some(id) = retry_id {
                self.capability_vm.pending.finish(id, reason.to_string());
            } else {
                self.retain_failed_action(original, reason.dependency, cx);
            }
            return;
        }

        let available = self
            .show_page
            .publisher
            .as_ref()
            .and_then(|section| section.event.as_ref())
            .is_some_and(|event| match operation {
                EventTargetOperation::Attach => !event.actions.attach.disabled(),
                EventTargetOperation::Detach => !event.actions.detach.disabled(),
            });
        if !available {
            if let Some(id) = retry_id {
                self.capability_vm.pending.finish(id, "App did not retry this target mutation because current event readiness no longer permits it. Check the original event and target.".into());
            }
            return;
        }
        let Some(input) = self.event_section_input.clone() else {
            if let Some(id) = retry_id {
                self.capability_vm.pending.finish(id, "App did not retry the event target because its event state is not loaded. Check the original event again.".into());
            }
            "Event target command error: event state is not loaded"
                .clone_into(&mut self.settings_status);
            cx.notify();
            return;
        };
        let Some(event) = input.selected_event.clone() else {
            if let Some(id) = retry_id {
                self.capability_vm.pending.finish(id, "App did not retry the event target because its original event is no longer selected.".into());
            }
            "Event target command error: no broadcast event selected"
                .clone_into(&mut self.settings_status);
            cx.notify();
            return;
        };
        let selected_host = match selected_broadcast_host(&self.broadcast) {
            Ok(host) => host,
            Err(error) => {
                if let Some(id) = retry_id {
                    self.capability_vm.pending.finish(id, "App did not retry the event target because its original relay sender or target is unavailable. Check it again.".into());
                }
                self.settings_status = format!("Event target command error: {error:#}");
                cx.notify();
                return;
            }
        };
        let Some(selected_instance) = self.broadcast.selected_show_instance.clone() else {
            if let Some(id) = retry_id {
                self.capability_vm.pending.finish(
                    id,
                    "App did not retry the event target because no show instance is selected."
                        .into(),
                );
            }
            "Event target command error: make or select a show instance"
                .clone_into(&mut self.settings_status);
            cx.notify();
            return;
        };
        let requested_target = match &retry {
            Some(intent) => intent
                .action
                .event_target_name()
                .map_err(|error| event_registry_error(error.to_string())),
            None => event_target_name_for_operation(operation, &input),
        };
        let target_name = match requested_target {
            Ok(name) => name,
            Err(error) => {
                if let Some(id) = retry_id {
                    self.capability_vm.pending.finish(id, "App did not retry the event target because its original relay sender or target is unavailable. Check it again.".into());
                }
                self.settings_status = format!("Event target command error: {error}");
                cx.notify();
                return;
            }
        };
        let Some(command_id) = self.show_commands.begin_service(
            PublisherServiceRole::Publisher,
            PublisherServiceOperation::Start,
        ) else {
            if let Some(id) = retry_id {
                self.capability_vm.pending.finish(
                    id,
                    "App is already running a relay sender command. Check again after it finishes."
                        .into(),
                );
            }
            return;
        };
        if operation == EventTargetOperation::Attach
            && self
                .on_air_sequence
                .as_ref()
                .and_then(crate::view_models::show::OnAirSequence::next_step)
                == Some(crate::view_models::show::OnAirStep::AttachEvent)
        {
            self.on_air_step_id = Some(command_id);
        }
        // ADR 0113: the waiting command needs the Viewed pace at once.
        self.sync_broadcast_service_watch_pace();
        self.event_session.next_request += 1;
        let request = self.event_session.next_request;
        let context = input.context.clone();
        let error_context = context.clone();
        let command = EventTargetCommand {
            conn: Arc::clone(&self.conn),
            selection_revision: input.registry.revision,
            operation,
            transport: selected_host.transport,
            private_instance_name: selected_host.instance_name,
            instance_name: selected_instance,
            target_name,
            event_id: event.event_id,
            token_path: event.token_path,
        };
        let command = self.command_with_retry(command, retry);
        let error_original = original.clone();
        if let Some(input) = &mut self.event_section_input {
            input.request = request;
            input.feedback.target_mutation = EventCommandState::Working;
            input.feedback.active_action = Some(match operation {
                EventTargetOperation::Attach => EventControlIntent::Attach,
                EventTargetOperation::Detach => EventControlIntent::Detach,
            });
            input.record_event_report(EventReportOperation::TargetMutation, chrono::Utc::now());
            input.targets = EventTargetListInput::Unknown;
        }
        self.reproject_show_page_from_current_queue();
        cx.notify();
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, completion, cx| {
                this.record_show_retry_result(original, retry_id, &completion.result, cx);
                this.finish_event_target(request, &context, command_id, completion, cx);
            },
            move |this, error, cx| {
                this.record_show_retry_result(error_original, retry_id, &Err(error.clone()), cx);
                this.finish_event_target(
                    request,
                    &error_context,
                    command_id,
                    ShowCommandCompletion::new(Err(error)),
                    cx,
                );
            },
        );
    }

    fn run_stream_encoder_command(
        &mut self,
        operation: StreamEncoderOperation,
        cx: &mut Context<Self>,
    ) {
        let action = RecoveryAction::Encoder {
            operation,
            target: self.broadcast.encoder.as_ref().ok().cloned().flatten(),
        };
        if let Err(reason) = self.feature_availability().require(Dependency::Encoder) {
            self.retain_failed_action(action, reason.dependency, cx);
            self.settings_status = reason.to_string();
            return;
        }
        let command = match StreamEncoderCommand::new(&self.broadcast, operation) {
            Ok(command) => command,
            Err(error) => {
                self.settings_status = format!("Stream command error: {error:#}");
                cx.notify();
                return;
            }
        };
        let Some(command_id) = self.show_commands.begin_stream(operation) else {
            return;
        };
        let on_air_connect = operation == StreamEncoderOperation::Connect
            && self
                .on_air_sequence
                .as_ref()
                .and_then(crate::view_models::show::OnAirSequence::next_step)
                == Some(crate::view_models::show::OnAirStep::ConnectStream);
        let end_show_disconnect = operation == StreamEncoderOperation::Disconnect
            && self
                .end_show_sequence
                .as_ref()
                .and_then(crate::view_models::show::EndShowSequence::next_step)
                == Some(crate::view_models::show::EndShowStep::DisconnectStream);
        if on_air_connect || end_show_disconnect {
            self.on_air_step_id = Some(command_id);
        }
        self.dispatch_show_command(command, command_id, action, None, cx);
    }

    fn invalidate_publisher_service_snapshot(&self) {
        if let Some(handle) = self.publisher_service_watch.as_ref() {
            let _ = handle.refresh_now();
        }
    }

    pub(super) fn invalidate_broadcast_readiness_snapshot(&self) {
        if let Some(handle) = self.broadcast_readiness_watch.as_ref() {
            let _ = handle.refresh_now();
        }
    }

    fn open_broadcast_readiness_in_music(&mut self, cx: &mut Context<Self>) {
        let Some(report) = self
            .broadcast_readiness_snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.report.clone())
        else {
            "Broadcast readiness is not available yet".clone_into(&mut self.settings_status);
            cx.notify();
            return;
        };

        self.tab = AppTab::Music;
        self.sync_broadcast_service_watch_pace();
        if let Some(content_list_id) = self.content_list_frame_id() {
            // ADR 0084 Decision 9: the Inbox starts a new path, as one
            // history entry.
            if let Err(error) = self.workspace_layout.start_nav_path(
                content_list_id,
                FrameNavigationEntry::Destination(MusicDestination::Inbox),
                None,
            ) {
                self.settings_status = format!("Error opening broadcast readiness: {error}");
            }
        }
        self.library.update(cx, |library, cx| {
            library.show_broadcast_readiness_report(&report, cx);
        });
        cx.notify();
    }

    fn music_readiness_list_is_current(&self) -> bool {
        matches!(self.tab, AppTab::Music)
            && self
                .content_list_frame_id()
                .and_then(|id| self.workspace_layout.frame_nav(id))
                .is_some_and(|nav| {
                    matches!(
                        nav.current(),
                        FrameNavigationEntry::Destination(MusicDestination::Inbox)
                    )
                })
    }
}

/// Registry commands have no publisher transport or configuration mutation capability (ADR 0113).
struct EventRegistryCommand {
    conn: Arc<Mutex<Connection>>,
    cfg_path: PathBuf,
    action: EventRegistryAction,
    selected_event: Option<EventSelectionInput>,
    selection_revision: i64,
}

impl ApplicationCommand for EventRegistryCommand {
    type Output = EventRegistryResult;

    fn execute(self, context: &CommandContext) -> crate::application::CommandResult<Self::Output> {
        if context.cancellation().is_cancelled() {
            return Err(CommandError::Cancelled);
        }
        let conn = self
            .conn
            .lock()
            .map_err(|_| CommandError::Query("database lock poisoned".to_owned()))?;
        // Revalidate stored identity and liveness before any relay mutation.
        context.validate_retry_subject(&conn)?;
        let selection = db::broadcast_event_selection(&conn).map_err(event_registry_error)?;
        let selected = selected_event_input(&conn).map_err(event_registry_error)?;
        if selection.revision != self.selection_revision
            || selection.event_id.as_deref()
                != self
                    .selected_event
                    .as_ref()
                    .map(|event| event.event_id.as_str())
        {
            return Err(CommandError::Other(
                "Selected event changed; refresh Show before trying again.".to_owned(),
            ));
        }
        let allowed = match self.action {
            EventRegistryAction::Create => {
                selection.event_id.is_none()
                    && db::broadcast_events(&conn)
                        .map_err(event_registry_error)?
                        .is_empty()
            }
            EventRegistryAction::Replace => selected.as_ref().is_some_and(|event| {
                event.state == EventState::Dead && event.kind == EventKind::Created
            }),
            EventRegistryAction::Check => selected.is_some(),
        };
        if !allowed {
            return Err(CommandError::Other(
                "Event action is unavailable in the current state.".to_owned(),
            ));
        }
        let endpoint = match &selected {
            Some(event) if self.action == EventRegistryAction::Check => event.endpoint.clone(),
            _ => config::load_musicindex_endpoint(&self.cfg_path).map_err(event_registry_error)?,
        };
        let directory = self
            .cfg_path
            .parent()
            .ok_or_else(|| CommandError::Other("Config path has no directory".to_owned()))?
            .join("broadcast")
            .join("tokens");
        let registry = BroadcastRegistry::with_token_directory(&conn, &endpoint, directory)
            .map_err(event_registry_error)?;
        let event = match self.action {
            EventRegistryAction::Create | EventRegistryAction::Replace => {
                registry
                    .create_event(None)
                    .map_err(event_registry_error)?
                    .event
            }
            EventRegistryAction::Check => {
                let event = selected
                    .as_ref()
                    .ok_or_else(|| CommandError::Other("No event selected".to_owned()))?;
                match registry.check_event(&event.event_id) {
                    Ok(checked) => checked.event,
                    Err(error) => {
                        return Ok(CommandOutcome::without_events(
                            EventRegistryResult::CheckFailed {
                                response: event_check_response(&error),
                                detail: format!("{error:#}"),
                            },
                        ));
                    }
                }
            }
        };
        let (selection, selection_error) = if self.action == EventRegistryAction::Check {
            (Some(selection), None)
        } else {
            match db::select_broadcast_event(&conn, &event.event_id) {
                Ok(selection) => (Some(selection), None),
                Err(error) => (
                    None,
                    Some(format!(
                        "Event registered, but saving the choice failed: {error:#}"
                    )),
                ),
            }
        };
        let events = db::broadcast_events(&conn)
            .map_err(event_registry_error)?
            .into_iter()
            .map(event_selection_input)
            .collect();
        Ok(CommandOutcome::without_events(
            EventRegistryResult::Complete(EventRegistrySuccess {
                event: event_selection_input(event),
                selection,
                selection_error,
                events,
            }),
        ))
    }
}

fn event_check_response(error: &anyhow::Error) -> EventCheckResponse {
    if let Some(error) = error.downcast_ref::<crate::api::LiveMetadataReadError>() {
        error
            .response_status
            .map_or(EventCheckResponse::NoResponse, EventCheckResponse::Http)
    } else if let Some(error) =
        error.downcast_ref::<crate::broadcast::registry::EventCheckSaveError>()
    {
        EventCheckResponse::SaveFailed(event_state(Some(error.observed_status)))
    } else {
        EventCheckResponse::Unclassified
    }
}

fn event_registry_error(error: impl std::fmt::Display) -> CommandError {
    CommandError::Other(format!("{error:#}"))
}

fn begin_event_registry_action(input: &mut EventSectionInput, action: EventRegistryAction) {
    input.feedback.active_action = Some(match action {
        EventRegistryAction::Create => EventControlIntent::Create,
        EventRegistryAction::Replace => EventControlIntent::Replace,
        EventRegistryAction::Check => EventControlIntent::Check,
    });
    match action {
        EventRegistryAction::Create | EventRegistryAction::Replace => {
            input.feedback.registration = EventCommandState::Working;
            input.feedback.check = EventCommandState::Idle;
            input.record_event_report(EventReportOperation::Check, chrono::Utc::now());
            input.record_event_report(EventReportOperation::Registration, chrono::Utc::now());
        }
        EventRegistryAction::Check => {
            input.feedback.check = EventCommandState::Working;
            input.record_event_report(EventReportOperation::Check, chrono::Utc::now());
        }
    }
}

enum EventRegistryResult {
    Complete(EventRegistrySuccess),
    CheckFailed {
        response: EventCheckResponse,
        detail: String,
    },
}

fn record_event_retry_result(
    pending: &mut crate::application::capability_recovery::RecoveryIntents,
    id: u64,
    result: &EventRegistryResult,
) {
    match result {
        EventRegistryResult::Complete(_) => pending.succeed(id, "App completed the original event command. See the Event report for the command and selection results.".into()),
        EventRegistryResult::CheckFailed { detail, .. } => pending.finish(id, crate::diagnostics::redact_endpoint_details(&format!("App could not check the original event: {detail}"))),
    }
}

struct EventRegistrySuccess {
    event: EventSelectionInput,
    selection: Option<db::BroadcastEventSelection>,
    selection_error: Option<String>,
    events: Vec<EventSelectionInput>,
}

/// Registration and selection storage have independent results (ADR 0113).
fn apply_event_registry_result(
    input: &mut EventSectionInput,
    action: EventRegistryAction,
    result: Result<EventRegistryResult, CommandError>,
) -> Option<EventRegistryAction> {
    match result {
        Ok(EventRegistryResult::CheckFailed { response, detail }) => {
            input.feedback.check_response = response;
            input.feedback.verification_failure = Some(detail.clone());
            input.feedback.check = EventCommandState::Failed { detail };
            input.record_event_report(EventReportOperation::Check, chrono::Utc::now());
            None
        }
        Ok(EventRegistryResult::Complete(result)) => {
            input.registry.events = result.events;
            if action == EventRegistryAction::Check {
                input.feedback.check = EventCommandState::Succeeded;
            } else {
                input.feedback.registration = EventCommandState::Succeeded;
                input.record_event_report_for(
                    EventReportOperation::Registration,
                    chrono::Utc::now(),
                    Some(result.event.clone()),
                );
                if let Some(detail) = result.selection_error {
                    input.feedback.selection = EventCommandState::Failed { detail };
                    input.record_event_report_for(
                        EventReportOperation::Selection,
                        chrono::Utc::now(),
                        Some(result.event),
                    );
                    return None;
                }
                input.feedback.selection = EventCommandState::Succeeded;
                input.record_event_report_for(
                    EventReportOperation::Selection,
                    chrono::Utc::now(),
                    Some(result.event.clone()),
                );
                input.targets = EventTargetListInput::Unknown;
            }
            if let Some(selection) = result.selection {
                input.registry.selected_id = selection.event_id;
                input.registry.revision = selection.revision;
            }
            input.selected_event = Some(result.event);
            input.liveness_confirmed = action == EventRegistryAction::Check;
            input.feedback.verification_failure = None;
            if action == EventRegistryAction::Check {
                input.record_event_report(EventReportOperation::Check, chrono::Utc::now());
            }
            (action != EventRegistryAction::Check).then_some(EventRegistryAction::Check)
        }
        Err(error) => {
            let failure = EventCommandState::Failed {
                detail: error.to_string(),
            };
            match action {
                EventRegistryAction::Check => {
                    input.feedback.verification_failure = Some(error.to_string());
                    input.feedback.check = failure;
                    input.feedback.check_response = EventCheckResponse::Unclassified;
                    input.record_event_report(EventReportOperation::Check, chrono::Utc::now());
                }
                EventRegistryAction::Create | EventRegistryAction::Replace => {
                    input.feedback.registration = failure;
                    input.record_event_report(
                        EventReportOperation::Registration,
                        chrono::Utc::now(),
                    );
                }
            }
            None
        }
    }
}

/// A queued refresh cannot overwrite a registration or check completed since it was requested.
fn apply_event_refresh(
    current: &mut Option<EventSectionInput>,
    requested: Option<&EventSectionInput>,
    mut refreshed: Option<EventSectionInput>,
) -> bool {
    if current.as_ref() != requested
        || current.as_ref().is_some_and(|input| {
            input.feedback.working()
                && refreshed
                    .as_ref()
                    .is_none_or(|new| new.context == input.context)
        })
    {
        return false;
    }
    if let (Some(old), Some(new)) = (current.as_ref(), refreshed.as_mut()) {
        if old.selected_event.as_ref().map(|event| &event.event_id)
            == new.selected_event.as_ref().map(|event| &event.event_id)
            && new.context == old.context
            && new.registry.revision == old.registry.revision
        {
            new.liveness_confirmed = old.liveness_confirmed;
            new.feedback = old.feedback.clone();
            new.targets = old.targets.clone();
            new.request = old.request;
        }
    }
    *current = refreshed;
    true
}

struct RefreshShowPage {
    conn: Arc<Mutex<Connection>>,
    music_dir: PathBuf,
    queue_text_filter: Option<String>,
    broadcast: config::BroadcastCapabilities,
}

impl RefreshShowPage {
    fn new(
        conn: Arc<Mutex<Connection>>,
        music_dir: PathBuf,
        queue_text_filter: Option<String>,
        broadcast: config::BroadcastCapabilities,
    ) -> Self {
        Self {
            conn,
            music_dir,
            queue_text_filter,
            broadcast,
        }
    }
}

impl ApplicationCommand for RefreshShowPage {
    type Output = ShowPageProjection;

    fn execute(self, context: &CommandContext) -> crate::application::CommandResult<Self::Output> {
        if context.cancellation().is_cancelled() {
            return Err(CommandError::Cancelled);
        }

        let (queue_input, event_section_input, playlists) = {
            let conn = self
                .conn
                .lock()
                .map_err(|_| CommandError::Query("database lock poisoned".to_string()))?;
            let queue = queue_now_playing_input(&conn, self.queue_text_filter);
            let input = load_event_section(&conn, &self.broadcast);
            // ADR 0106 Decision 4: the same read as My playlists, in the
            // Curator Order. A failed read shows no rows.
            let playlists = db::playlists_list(&conn).unwrap_or_default();
            (queue, Some(input), playlists)
        };
        let queue = queue_now_playing_vm(queue_input, &self.music_dir);
        Ok(CommandOutcome::without_events(ShowPageProjection {
            queue,
            event_section_input,
            playlists,
        }))
    }
}

struct ShowPageProjection {
    queue: QueueNowPlayingPageVm,
    event_section_input: Option<EventSectionInput>,
    playlists: Vec<db::Playlist>,
}

/// Capture completion in the worker, before presentation callback scheduling (ADR 0113).
struct ShowCommandCompletion {
    returned_at: Instant,
    result: Result<(), CommandError>,
}

impl ShowCommandCompletion {
    fn new(result: Result<(), CommandError>) -> Self {
        Self {
            returned_at: Instant::now(),
            result,
        }
    }
}

struct PublisherServiceCommand {
    role: PublisherServiceRole,
    transport: crate::broadcast::transport::Transport,
    unit: UnitRef,
    operation: PublisherServiceOperation,
}

impl PublisherServiceCommand {
    fn new(
        host: &BroadcastHostConfig,
        role: PublisherServiceRole,
        operation: PublisherServiceOperation,
        selected: Option<&str>,
    ) -> Result<Self, CommandError> {
        // ADR 0111 Decision 4: refuse before any command runs.
        let instance = refuse_private_service_instance(host, selected)?;
        Ok(Self {
            role,
            transport: host.transport.clone(),
            unit: broadcast_service_unit(host.producer, &instance, role)
                .map_err(publisher_command_error)?,
            operation,
        })
    }
}

impl ApplicationCommand for PublisherServiceCommand {
    type Output = ShowCommandCompletion;

    fn execute(self, context: &CommandContext) -> crate::application::CommandResult<Self::Output> {
        if context.cancellation().is_cancelled() {
            return Err(CommandError::Cancelled);
        }
        let result = match self.operation {
            PublisherServiceOperation::Start => control::start(&self.transport, &self.unit),
            PublisherServiceOperation::Stop => control::stop(&self.transport, &self.unit),
            PublisherServiceOperation::Reset => control::reset(&self.transport, &self.unit),
        }
        .map_err(|error| {
            publisher_command_error(format!(
                "{} {}: {error:#}",
                operation_label(self.operation),
                role_label(self.role)
            ))
        });

        Ok(CommandOutcome::without_events(ShowCommandCompletion::new(
            result,
        )))
    }
}

struct ReadPublisherLogs {
    request_id: ShowLogRequestId,
    role: PublisherServiceRole,
    transport: crate::broadcast::transport::Transport,
    unit: UnitRef,
    line_count: usize,
}

impl ReadPublisherLogs {
    fn new(
        host: &BroadcastHostConfig,
        role: PublisherServiceRole,
        request_id: ShowLogRequestId,
        instance: &str,
    ) -> Result<Self, CommandError> {
        Ok(Self {
            request_id,
            role,
            transport: host.transport.clone(),
            unit: broadcast_service_unit(host.producer, instance, role)
                .map_err(publisher_command_error)?,
            line_count: PUBLISHER_LOG_LINE_COUNT,
        })
    }
}

impl ApplicationCommand for ReadPublisherLogs {
    type Output = PublisherLogsResult;

    fn execute(self, context: &CommandContext) -> crate::application::CommandResult<Self::Output> {
        if context.cancellation().is_cancelled() {
            return Err(CommandError::Cancelled);
        }
        let text =
            control::logs(&self.transport, &self.unit, self.line_count).map_err(|error| {
                publisher_command_error(format!("read {} logs: {error:#}", role_label(self.role)))
            })?;
        Ok(CommandOutcome::without_events(PublisherLogsResult {
            request_id: self.request_id,
            text,
        }))
    }
}

struct PublisherLogsResult {
    request_id: ShowLogRequestId,
    text: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EventTargetOperation {
    Attach,
    Detach,
}

struct EventTargetCommand {
    conn: Arc<Mutex<Connection>>,
    selection_revision: i64,
    operation: EventTargetOperation,
    transport: Transport,
    /// The private instance of the host. ADR 0111 Decision 4 refuses it.
    private_instance_name: String,
    instance_name: String,
    target_name: String,
    event_id: String,
    token_path: String,
}

impl ApplicationCommand for EventTargetCommand {
    type Output = ShowCommandCompletion;

    fn execute(self, context: &CommandContext) -> crate::application::CommandResult<Self::Output> {
        if context.cancellation().is_cancelled() {
            return Err(CommandError::Cancelled);
        }
        publisher_targets::refuse_private_instance(
            &self.private_instance_name,
            &self.instance_name,
        )
        .map_err(|error| event_target_command_error(error.to_string()))?;
        let conn = self
            .conn
            .lock()
            .map_err(|_| event_registry_error("database lock poisoned"))?;
        context.validate_retry_subject(&conn)?;
        context.validate_retry_event_target(
            &self.event_id,
            &self.target_name,
            &self.transport,
            &self.instance_name,
        )?;
        let selection = db::broadcast_event_selection(&conn).map_err(event_registry_error)?;
        if selection.revision != self.selection_revision
            || selection.event_id.as_deref() != Some(&self.event_id)
        {
            return Err(event_registry_error(
                "Selected event changed before target mutation",
            ));
        }
        let selected =
            db::broadcast_event_by_event_id(&conn, &self.event_id).map_err(event_registry_error)?;
        if self.operation == EventTargetOperation::Attach
            && selected
                .as_ref()
                .is_none_or(|event| event.last_status != Some(db::BroadcastEventStatus::Live))
        {
            return Err(event_registry_error("Event is no longer confirmed Live"));
        }
        if self.operation == EventTargetOperation::Detach {
            let targets = publisher_targets::list_targets(&self.transport, &self.instance_name)
                .map_err(event_registry_error)?;
            if !targets
                .targets
                .iter()
                .any(|target| target.name == self.target_name && target.event_id == self.event_id)
            {
                return Err(event_registry_error(
                    "Configured target changed before Detach; check it again",
                ));
            }
        }
        let result = match self.operation {
            EventTargetOperation::Attach => publisher_targets::attach_event(
                &self.transport,
                &self.private_instance_name,
                &self.instance_name,
                &self.target_name,
                &self.event_id,
                Path::new(&self.token_path),
            ),
            EventTargetOperation::Detach => publisher_targets::detach_target(
                &self.transport,
                &self.private_instance_name,
                &self.instance_name,
                &self.target_name,
            ),
        }
        .map_err(|error| {
            if let PublisherTargetCommandError::RestartFailed { detail } = error {
                return event_target_command_error(format!(
                    "Target configuration saved; relay sender restart failed: {detail}"
                ));
            }
            event_target_command_error(format!(
                "{} event target: {error}",
                event_target_operation_label(self.operation)
            ))
        });
        Ok(CommandOutcome::without_events(ShowCommandCompletion::new(
            result,
        )))
    }
}

fn event_target_name_for_operation(
    operation: EventTargetOperation,
    input: &EventSectionInput,
) -> Result<String, CommandError> {
    match operation {
        EventTargetOperation::Attach => {
            let target_name = input.attach_target_name.trim();
            if target_name.is_empty() {
                return Err(CommandError::Other(
                    "broadcast target name is not configured".to_owned(),
                ));
            }
            Ok(target_name.to_owned())
        }
        EventTargetOperation::Detach => attached_target_name(input).ok_or_else(|| {
            CommandError::Other("selected broadcast event is not attached".to_owned())
        }),
    }
}

fn attached_target_name(input: &EventSectionInput) -> Option<String> {
    let event_id = input.selected_event.as_ref()?.event_id.as_str();
    let EventTargetListInput::Loaded { targets } = &input.targets else {
        return None;
    };
    targets
        .iter()
        .find(|target| {
            target.name == input.attach_target_name.trim() && target.event_id == event_id
        })
        .map(|target| target.name.trim().to_owned())
        .filter(|target_name| !target_name.is_empty())
}

fn start_broadcast_service_watch(
    host: &RuntimeHost,
    units: Vec<BroadcastServiceWatchUnit>,
    instances: BroadcastInstanceWatch,
    encoder: BroadcastEncoderWatchTarget,
) -> BroadcastServiceWatchHandle {
    let _enter = host.handle().enter();
    crate::runtime::broadcast_service_watch::start(units, instances, encoder, host.bus().session())
}

fn start_broadcast_readiness_watch(
    host: &RuntimeHost,
    conn: Arc<Mutex<Connection>>,
) -> BroadcastReadinessWatchHandle {
    let _enter = host.handle().enter();
    crate::runtime::broadcast_readiness::start(conn, host.bus().session())
}

struct StreamEncoderCommand {
    target: EncoderTarget,
    server_name: Option<String>,
    operation: StreamEncoderOperation,
}

impl StreamEncoderCommand {
    fn new(
        broadcast: &crate::config::BroadcastCapabilities,
        operation: StreamEncoderOperation,
    ) -> Result<Self, CommandError> {
        let encoder = broadcast
            .encoder
            .as_ref()
            .map_err(stream_command_error)?
            .as_ref()
            .ok_or_else(|| CommandError::Other("broadcast.encoder is not configured".to_owned()))?;
        Ok(Self {
            target: encoder.target().map_err(stream_command_error)?,
            server_name: encoder
                .default_server_name
                .as_deref()
                .map(|name| name.trim().to_owned()),
            operation,
        })
    }
}

impl ApplicationCommand for StreamEncoderCommand {
    type Output = ShowCommandCompletion;

    fn execute(self, context: &CommandContext) -> crate::application::CommandResult<Self::Output> {
        if context.cancellation().is_cancelled() {
            return Err(CommandError::Cancelled);
        }
        let result = match self.operation {
            StreamEncoderOperation::Connect => {
                encoder::connect(&self.target, self.server_name.as_deref())
            }
            StreamEncoderOperation::Disconnect => encoder::disconnect(&self.target),
        }
        .map_err(|error| {
            stream_command_error(format!(
                "{} stream encoder: {error:#}",
                stream_operation_label(self.operation)
            ))
        });

        Ok(CommandOutcome::without_events(ShowCommandCompletion::new(
            result,
        )))
    }
}

fn selected_broadcast_host(
    broadcast: &crate::config::BroadcastCapabilities,
) -> anyhow::Result<BroadcastHostConfig> {
    Ok(broadcast.selected_host()?.clone())
}

fn selected_event_input(conn: &Connection) -> anyhow::Result<Option<EventSelectionInput>> {
    let selection = db::broadcast_event_selection(conn)?;
    selection
        .event_id
        .as_deref()
        .map(|id| db::broadcast_event_by_event_id(conn, id))
        .transpose()
        .map(|event| event.flatten().map(event_selection_input))
}

const fn event_kind(kind: db::BroadcastEventKind) -> EventKind {
    match kind {
        db::BroadcastEventKind::Created => EventKind::Created,
        db::BroadcastEventKind::Reserved => EventKind::Reserved,
    }
}

fn event_selection_input(event: db::BroadcastEventRow) -> EventSelectionInput {
    EventSelectionInput {
        kind: event_kind(event.kind),
        created_at: event.created_at,
        last_checked_at: event.last_checked_at,
        label: event.label,
        event_id: event.event_id,
        endpoint: event.endpoint,
        token_file_missing: token_file_missing(&event.token_path),
        token_path: event.token_path,
        state: event_state(event.last_status),
    }
}

fn load_event_section(
    conn: &Connection,
    broadcast: &config::BroadcastCapabilities,
) -> EventSectionInput {
    let mut input = empty_event_section(broadcast);
    let result = (|| -> anyhow::Result<()> {
        let selection = db::broadcast_event_selection(conn)?;
        input.registry.events = db::broadcast_events(conn)?
            .into_iter()
            .map(event_selection_input)
            .collect();
        input.selected_event = selected_event_input(conn)?;
        input.registry.selected_id = selection.event_id;
        input.registry.revision = selection.revision;
        Ok(())
    })();
    input.registry.status = match result {
        Ok(()) => EventRegistryStatus::Loaded,
        Err(error) => EventRegistryStatus::Failed(format!("{error:#}")),
    };
    input
}

fn empty_event_section(broadcast: &config::BroadcastCapabilities) -> EventSectionInput {
    EventSectionInput {
        liveness_confirmed: false,
        registry: EventRegistryInput {
            status: EventRegistryStatus::Loading,
            ..EventRegistryInput::default()
        },
        feedback: EventCommandFeedback::default(),
        selected_event: None,
        targets: EventTargetListInput::Unknown,
        attach_target_name: selected_show_target(broadcast)
            .map(|(_, target)| target)
            .unwrap_or_default(),
        remote_host: broadcast
            .selected_host()
            .is_ok_and(|host| matches!(host.transport, Transport::Ssh { .. })),
        context: event_context(broadcast),
        request: 0,
    }
}

fn event_context(broadcast: &config::BroadcastCapabilities) -> String {
    broadcast.selected_host().map_or_else(
        |error| error.to_string(),
        |host| {
            let (instance, target) = selected_show_target(broadcast).unwrap_or_default();
            format!(
                "{} / {} / {:?} / {}",
                host.name, instance, host.transport, target
            )
        },
    )
}

/// The selected show instance and its target name. The private instance is
/// never a show instance (ADR 0111 Decision 4), so it gives `None`.
pub(super) fn selected_show_target(
    broadcast: &config::BroadcastCapabilities,
) -> Option<(String, String)> {
    let host = broadcast.selected_host().ok()?;
    let name = broadcast.selected_show_instance.as_deref()?;
    host.show_instances
        .iter()
        .find(|instance| instance.name == name && instance.name != host.instance_name)
        .map(|instance| (instance.name.clone(), instance.target.clone()))
}

fn read_targets(broadcast: &config::BroadcastCapabilities) -> EventTargetListInput {
    let host = match selected_broadcast_host(broadcast) {
        Ok(host) => host,
        Err(error) => {
            return EventTargetListInput::Failed {
                detail: format!("{error:#}"),
            }
        }
    };
    // The list is read only, so the private instance may be listed.
    let Some(instance) = broadcast.selected_show_instance.as_deref() else {
        return EventTargetListInput::Unknown;
    };
    match publisher_targets::list_targets(&host.transport, instance) {
        Ok(targets) => event_targets_loaded(targets),
        Err(PublisherTargetCommandError::CommandsUnavailable) => {
            EventTargetListInput::CommandsUnavailable
        }
        Err(PublisherTargetCommandError::NotReachable) => EventTargetListInput::NotReachable,
        Err(error) => EventTargetListInput::Failed {
            detail: error.to_string(),
        },
    }
}

fn event_targets_loaded(targets: PublisherTargetList) -> EventTargetListInput {
    EventTargetListInput::Loaded {
        targets: targets
            .targets
            .into_iter()
            .map(|target| EventTargetInput {
                name: target.name,
                event_id: target.event_id,
            })
            .collect(),
    }
}

const fn event_state(status: Option<db::BroadcastEventStatus>) -> EventState {
    match status {
        Some(db::BroadcastEventStatus::Live) => EventState::Live,
        Some(db::BroadcastEventStatus::Dead) => EventState::Dead,
        Some(db::BroadcastEventStatus::Unknown) | None => EventState::Unknown,
    }
}

fn token_file_missing(path: &str) -> bool {
    fs::metadata(path).is_err()
}

// ADR 0066: neither independent observer requires the other configuration.
fn broadcast_watch_inputs(
    broadcast: &config::BroadcastCapabilities,
) -> (Vec<BroadcastServiceWatchUnit>, BroadcastEncoderWatchTarget) {
    let units = selected_broadcast_host(broadcast)
        .and_then(|host| {
            let instance = service_instance(&host, broadcast.selected_show_instance.as_deref());
            broadcast_service_units(&host, &instance)
        })
        .unwrap_or_default();
    let encoder = broadcast_encoder_watch_target(broadcast)
        .unwrap_or_else(|_| BroadcastEncoderWatchTarget::unavailable());
    (units, encoder)
}

/// The private instance and each show instance of the selected host
/// (ADR 0111 Decision 8). An invalid host gives no instances.
fn broadcast_instance_watch(broadcast: &config::BroadcastCapabilities) -> BroadcastInstanceWatch {
    selected_broadcast_host(broadcast)
        .and_then(|host| broadcast_host_instances(&host))
        .unwrap_or_default()
}

fn broadcast_host_instances(host: &BroadcastHostConfig) -> anyhow::Result<BroadcastInstanceWatch> {
    // ADR 0112 Decision 4: a station host has the station instance only.
    let show_instances: &[_] = if host.producer == config::ProducerKind::Liquidsoap {
        &[]
    } else {
        &host.show_instances
    };
    let names = std::iter::once(host.instance_name.as_str())
        .chain(show_instances.iter().map(|show| show.name.as_str()));
    let instances = names
        .map(|name| {
            Ok(BroadcastWatchInstance {
                name: name.to_owned(),
                publisher: UnitRef::publisher(name)?,
                producer: UnitRef::producer(host.producer, name)?,
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    Ok(BroadcastInstanceWatch {
        transport: host.transport.clone(),
        instances,
    })
}

fn broadcast_encoder_watch_target(
    broadcast: &crate::config::BroadcastCapabilities,
) -> anyhow::Result<BroadcastEncoderWatchTarget> {
    let Some(encoder) = broadcast.encoder.as_ref().map_err(|issue| *issue)? else {
        return Ok(BroadcastEncoderWatchTarget::not_configured());
    };
    Ok(BroadcastEncoderWatchTarget::configured(
        encoder
            .default_server_name
            .as_deref()
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map_or_else(|| "Encoder default".to_owned(), ToOwned::to_owned),
        encoder.target()?,
    ))
}

fn broadcast_service_units(
    host: &BroadcastHostConfig,
    instance: &str,
) -> anyhow::Result<Vec<BroadcastServiceWatchUnit>> {
    let host_name = host.name.trim().to_owned();
    Ok(vec![
        BroadcastServiceWatchUnit::new(
            BroadcastServiceRole::Publisher,
            host_name.clone(),
            host.transport.clone(),
            broadcast_service_unit(host.producer, instance, BroadcastServiceRole::Publisher)?,
        ),
        BroadcastServiceWatchUnit::new(
            BroadcastServiceRole::Producer,
            host_name,
            host.transport.clone(),
            broadcast_service_unit(host.producer, instance, BroadcastServiceRole::Producer)?,
        ),
    ])
}

fn broadcast_service_unit(
    kind: config::ProducerKind,
    instance: &str,
    role: PublisherServiceRole,
) -> anyhow::Result<UnitRef> {
    match role {
        PublisherServiceRole::Publisher => UnitRef::publisher(instance),
        PublisherServiceRole::Producer => UnitRef::producer(kind, instance),
    }
}

/// The instance whose role units the Show page reads: the selected instance, or the private
/// instance when none is selected. The private units are read for display only.
fn service_instance(host: &BroadcastHostConfig, selected: Option<&str>) -> String {
    // ADR 0112 Decision 4: a station host has the station instance only.
    if host.producer == config::ProducerKind::Liquidsoap {
        return host.instance_name.clone();
    }
    selected
        .filter(|name| {
            *name == host.instance_name || host.show_instances.iter().any(|show| show.name == *name)
        })
        .unwrap_or(&host.instance_name)
        .to_owned()
}

/// The instance that a Start, Stop or Reset command may act on.
///
/// ADR 0111 Decision 4: the private instance, and no selection, are refused. Only the producer
/// switch and the End show step `StartPrivateProducer` start or stop the private producer.
fn refuse_private_service_instance(
    host: &BroadcastHostConfig,
    selected: Option<&str>,
) -> Result<String, CommandError> {
    // ADR 0112 Decision 4: on a station host the commands act on `instance_name`.
    if host.producer == config::ProducerKind::Liquidsoap {
        return Ok(host.instance_name.clone());
    }
    match selected {
        None => Err(publisher_command_error(
            "No show instance is selected. Make or select a show instance, then try again.",
        )),
        Some(name) if name == host.instance_name => Err(publisher_command_error(format!(
            "ADR 0111 Decision 4: HitIt does not start, stop or reset a unit of the private \
             instance {name}. Select a show instance."
        ))),
        Some(name) => Ok(name.to_owned()),
    }
}

fn publisher_command_error(error: impl std::fmt::Display) -> CommandError {
    CommandError::Other(error.to_string())
}

fn stream_command_error(error: impl std::fmt::Display) -> CommandError {
    CommandError::Other(error.to_string())
}

fn event_target_command_error(error: impl std::fmt::Display) -> CommandError {
    CommandError::Other(error.to_string())
}

const fn role_label(role: PublisherServiceRole) -> &'static str {
    match role {
        PublisherServiceRole::Publisher => "relay sender",
        PublisherServiceRole::Producer => "producer",
    }
}

const fn operation_label(operation: PublisherServiceOperation) -> &'static str {
    match operation {
        PublisherServiceOperation::Start => "start",
        PublisherServiceOperation::Stop => "stop",
        PublisherServiceOperation::Reset => "reset",
    }
}

const fn stream_operation_label(operation: StreamEncoderOperation) -> &'static str {
    match operation {
        StreamEncoderOperation::Connect => "connect",
        StreamEncoderOperation::Disconnect => "disconnect",
    }
}

const fn event_target_operation_label(operation: EventTargetOperation) -> &'static str {
    match operation {
        EventTargetOperation::Attach => "attach",
        EventTargetOperation::Detach => "detach",
    }
}

#[cfg(test)]
mod tests {
    fn adr_0111_broadcast() -> config::BroadcastCapabilities {
        let temp = tempfile::tempdir().unwrap();
        let cfg_path = temp.path().join("config.toml");
        fs::write(
            &cfg_path,
            r#"
music_dir = "/tmp/music"
db_path = "/tmp/hitit.sqlite"

[[broadcast.hosts]]
name = "Local"
transport = "local"
instance_name = "mixxx"

[[broadcast.hosts.show_instances]]
name = "show-1"
target = "show-target"
config_file = "/fixture/show-1/config.toml"
publisher_unit = "musicindex-live-publisher@show-1.service"
producer_unit = "mixxx-now-playing@show-1.service"
"#,
        )
        .unwrap();
        config::ConfigSnapshot::read_existing(&cfg_path)
            .unwrap()
            .broadcast()
    }

    /// The path function takes the runtime folder as an argument.
    #[test]
    fn adr_0111_show_drop_directory_is_below_the_runtime_folder() {
        let temp = tempfile::tempdir().unwrap();
        assert_eq!(
            config::show_drop_directory(temp.path(), "show-1"),
            temp.path()
                .join("musicindex-live-publisher")
                .join("show-1")
                .join("nowplaying")
        );
    }

    /// The producer follows the selection: no selection keeps `broadcast.drop_directory`, a show
    /// instance gives its folder and its target, and a missing runtime folder gives an error.
    #[test]
    fn adr_0111_producer_folder_follows_the_selection() {
        let temp = tempfile::tempdir().unwrap();
        let mut broadcast = adr_0111_broadcast();
        assert!(broadcast
            .drop_file_producer_in(Some(temp.path()))
            .unwrap()
            .is_none());
        broadcast.selected_show_instance = Some("show-1".to_owned());
        let producer = broadcast
            .drop_file_producer_in(Some(temp.path()))
            .unwrap()
            .unwrap();
        assert_eq!(
            producer.path(),
            config::show_drop_directory(temp.path(), "show-1").join("show-target.nowplaying.json")
        );
        assert!(broadcast
            .show_drop_problem(None)
            .is_some_and(|text| text.contains("XDG_RUNTIME_DIR")));
        assert!(broadcast.drop_file_producer_in(None).is_err());
        assert_eq!(broadcast.show_drop_problem(Some(temp.path())), None);
        broadcast.selected_show_instance = Some("mixxx".to_owned());
        assert!(broadcast
            .drop_file_producer_in(Some(temp.path()))
            .unwrap()
            .is_none());
    }

    /// A host that uses SSH has no local drop folder for a show instance.
    #[test]
    fn adr_0111_ssh_host_has_no_drop_folder() {
        let mut broadcast = adr_0111_broadcast();
        let host = &mut broadcast.hosts.as_mut().unwrap()[0];
        host.transport =
            crate::broadcast::transport::Transport::ssh("operator@example.test").unwrap();
        broadcast.selected_show_instance = Some("show-1".to_owned());
        let temp = tempfile::tempdir().unwrap();
        let problem = broadcast.show_drop_problem(Some(temp.path())).unwrap();
        assert!(problem.contains("local") && problem.contains("SSH"));
        assert!(broadcast.drop_file_producer_in(Some(temp.path())).is_err());
    }

    /// Start, Stop and Reset and the watch act on the units of the selected show instance. The
    /// private instance and no selection are refused when the command is made, before any
    /// command can run.
    #[test]
    fn adr_0111_service_commands_target_the_show_instance_and_refuse_the_private_one() {
        let broadcast = adr_0111_broadcast();
        let host = broadcast.selected_host().unwrap().clone();
        for role in [
            PublisherServiceRole::Publisher,
            PublisherServiceRole::Producer,
        ] {
            for operation in [
                PublisherServiceOperation::Start,
                PublisherServiceOperation::Stop,
                PublisherServiceOperation::Reset,
            ] {
                let command =
                    PublisherServiceCommand::new(&host, role, operation, Some("show-1")).unwrap();
                assert!(command.unit.unit().contains("@show-1."));
                let error = PublisherServiceCommand::new(&host, role, operation, Some("mixxx"))
                    .err()
                    .unwrap()
                    .to_string();
                assert!(error.contains("ADR 0111 Decision 4") && error.contains("mixxx"));
                assert!(PublisherServiceCommand::new(&host, role, operation, None).is_err());
            }
        }
        // The watch follows the selection and reads the private units when none is selected.
        let names = |selected: Option<&str>| {
            let mut broadcast = broadcast.clone();
            broadcast.selected_show_instance = selected.map(str::to_owned);
            broadcast_watch_inputs(&broadcast)
                .0
                .iter()
                .map(|unit| unit.unit.unit().to_owned())
                .collect::<Vec<_>>()
        };
        assert!(names(None).iter().all(|unit| unit.contains("@mixxx.")));
        assert!(names(Some("show-1"))
            .iter()
            .all(|unit| unit.contains("@show-1.")));
    }

    fn adr_0112_station_broadcast() -> config::BroadcastCapabilities {
        let temp = tempfile::tempdir().unwrap();
        let cfg_path = temp.path().join("config.toml");
        fs::write(
            &cfg_path,
            r#"
music_dir = "/tmp/music"
db_path = "/tmp/hitit.sqlite"

[[broadcast.hosts]]
name = "Radio"
transport = "local"
instance_name = "my-station"
producer = "liquidsoap"
"#,
        )
        .unwrap();
        config::ConfigSnapshot::read_existing(&cfg_path)
            .unwrap()
            .broadcast()
    }

    /// ADR 0112 Decision 4: on a station host the watch reads the two station units, the
    /// commands act on `instance_name` with any selection, and the instance list has one entry.
    #[test]
    fn adr_0112_station_units_are_watched_and_commanded_on_the_station_instance() {
        let mut broadcast = adr_0112_station_broadcast();
        let host = broadcast.selected_host().unwrap().clone();
        for selected in [None, Some("my-station"), Some("other")] {
            broadcast.selected_show_instance = selected.map(str::to_owned);
            let units: Vec<_> = broadcast_watch_inputs(&broadcast)
                .0
                .iter()
                .map(|unit| unit.unit.unit().to_owned())
                .collect();
            assert_eq!(
                units,
                [
                    "musicindex-live-publisher@my-station.service",
                    "liquidsoap-now-playing@my-station.service"
                ]
            );
            for role in [
                PublisherServiceRole::Publisher,
                PublisherServiceRole::Producer,
            ] {
                for operation in [
                    PublisherServiceOperation::Start,
                    PublisherServiceOperation::Stop,
                    PublisherServiceOperation::Reset,
                ] {
                    let command =
                        PublisherServiceCommand::new(&host, role, operation, selected).unwrap();
                    assert!(command.unit.unit().contains("@my-station."));
                }
            }
        }
        let watch = broadcast_instance_watch(&broadcast);
        let names: Vec<_> = watch.instances.iter().map(|i| i.name.as_str()).collect();
        assert_eq!(names, ["my-station"]);
        assert_eq!(
            watch.instances[0].producer.unit(),
            "liquidsoap-now-playing@my-station.service"
        );
    }

    /// ADR 0112 Decision 3: Attach and Detach have no target on a station instance.
    #[test]
    fn adr_0112_station_instance_has_no_attach_target() {
        let mut broadcast = adr_0112_station_broadcast();
        broadcast.selected_show_instance = Some("my-station".to_owned());
        assert_eq!(selected_show_target(&broadcast), None);
        assert_eq!(empty_event_section(&broadcast).attach_target_name, "");
    }

    /// ADR 0111 criterion 12: Attach goes to the target of the selected show
    /// instance. With no selection, or with the private instance, the section has
    /// no target name, so Attach and Detach are not available.
    #[test]
    fn adr_0111_attach_uses_the_target_of_the_selected_show_instance() {
        let mut broadcast = adr_0111_broadcast();
        assert_eq!(selected_show_target(&broadcast), None);
        assert_eq!(empty_event_section(&broadcast).attach_target_name, "");
        broadcast.selected_show_instance = Some("show-1".to_owned());
        assert_eq!(
            selected_show_target(&broadcast),
            Some(("show-1".to_owned(), "show-target".to_owned()))
        );
        assert_eq!(
            empty_event_section(&broadcast).attach_target_name,
            "show-target"
        );
        broadcast.selected_show_instance = Some("mixxx".to_owned());
        assert_eq!(selected_show_target(&broadcast), None);
        assert_eq!(empty_event_section(&broadcast).attach_target_name, "");
    }

    /// ADR 0111 criterion 5 (Attach and Detach): the app command refuses the
    /// private instance with an error that names Decision 4, before it reads the
    /// database or runs a command.
    #[test]
    fn adr_0111_event_target_command_refuses_the_private_instance() {
        let temp = tempfile::tempdir().unwrap();
        let conn = Arc::new(Mutex::new(show_event_db(&temp)));
        for operation in [EventTargetOperation::Attach, EventTargetOperation::Detach] {
            let command = EventTargetCommand {
                conn: Arc::clone(&conn),
                selection_revision: 0,
                operation,
                transport: Transport::local(),
                private_instance_name: "mixxx".to_owned(),
                instance_name: "mixxx".to_owned(),
                target_name: "default".to_owned(),
                event_id: "event-one".to_owned(),
                token_path: "/fixture/one.token".to_owned(),
            };
            let error = match command.execute(&CommandContext::next()) {
                Ok(_) => panic!("the private instance must be refused"),
                Err(error) => error.to_string(),
            };
            assert!(error.contains("ADR 0111 Decision 4"), "{error}");
            assert!(error.contains("mixxx"), "{error}");
        }
    }

    #[test]
    fn adr_0066_failed_event_check_keeps_its_recovery_controls() {
        let mut pending = crate::application::capability_recovery::RecoveryIntents::default();
        let id = pending.retain(
            RecoveryAction::Event {
                operation: EventControlIntent::Check,
                event_id: Some("original-event".into()),
                selection_revision: 1,
                host: None,
                target: None,
            },
            Dependency::MusicIndex,
            1,
        );
        record_event_retry_result(
            &mut pending,
            id,
            &EventRegistryResult::CheckFailed {
                response: EventCheckResponse::NoResponse,
                detail: "No relay response.".into(),
            },
        );
        assert!(!pending.entries()[0].completed());
        assert!(pending.entries()[0]
            .result
            .as_ref()
            .unwrap()
            .message()
            .contains("could not check the original event"));
    }

    #[test]
    fn adr_0066_broadcast_observers_and_commands_are_independent() {
        let snapshot = config::ConfigSnapshot::from_bytes(
            std::path::Path::new("config.toml"),
            b"[broadcast]\nhosts = false\n[broadcast.encoder]\naddress = '127.0.0.1'\n".to_vec(),
        )
        .unwrap();
        let (units, encoder) = broadcast_watch_inputs(&snapshot.broadcast());
        assert!(units.is_empty());
        assert!(encoder.target.is_some());
        assert!(
            StreamEncoderCommand::new(&snapshot.broadcast(), StreamEncoderOperation::Connect)
                .is_ok()
        );
        let snapshot = config::ConfigSnapshot::from_bytes(
            std::path::Path::new("config.toml"),
            b"[broadcast]\nencoder = false\n".to_vec(),
        )
        .unwrap();
        let (units, encoder) = broadcast_watch_inputs(&snapshot.broadcast());
        assert_eq!(units.len(), 2);
        assert!(encoder.target.is_none());
        assert_eq!(encoder.server_name, "Configuration unavailable");
        assert!(
            StreamEncoderCommand::new(&snapshot.broadcast(), StreamEncoderOperation::Connect)
                .is_err()
        );
    }

    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::time::{Duration, Instant};

    use crate::broadcast::control::ServiceState;
    use crate::runtime::broadcast_service_watch::{
        BroadcastEncoderSnapshot, BroadcastServiceUnitSnapshot,
    };
    use crate::view_models::show::{EventSectionDisplay, ShowLogPaneDisplay};

    fn show_event_db(temp: &tempfile::TempDir) -> Connection {
        db::open_db(&temp.path().join("app.sqlite"))
            .unwrap()
            .connection
    }

    fn show_event_relay(statuses: Vec<u16>) -> (String, std::thread::JoinHandle<Vec<String>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let thread = std::thread::spawn(move || {
            let mut requests = Vec::new();
            for status in statuses {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut request = Vec::new();
                let mut byte = [0];
                while !request.ends_with(b"\r\n\r\n") {
                    stream.read_exact(&mut byte).unwrap();
                    request.push(byte[0]);
                }
                let request = String::from_utf8(request).unwrap();
                let body_length = request
                    .lines()
                    .find_map(|line| {
                        line.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .map(|length| length.trim().parse::<usize>().unwrap())
                    })
                    .unwrap_or(0);
                stream.read_exact(&mut vec![0; body_length]).unwrap();
                let line = request.lines().next().unwrap().to_owned();
                let body = if line.starts_with("POST ") {
                    serde_json::json!({
                        "event_id": "event-new", "broadcaster_token": "fixture-secret",
                        "metadata_url": "/v1/liveitems/event-new/metadata", "events_url": "/events",
                    })
                    .to_string()
                } else if status == 404 {
                    // ADR 0110 Decision 4: only this code means Dead.
                    r#"{"error":"event_not_found"}"#.to_owned()
                } else {
                    serde_json::json!({
                        "event_id": "event-new", "seq": 1, "updated_at": "2026-09-09T00:00:00Z", "metadata": {},
                    }).to_string()
                };
                requests.push(line);
                write!(stream, "HTTP/1.1 {status} Fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
            }
            requests
        });
        (endpoint, thread)
    }

    fn show_event_project(input: &EventSectionInput) -> EventSectionDisplay {
        let snapshot = BroadcastServiceWatchSnapshot {
            read_started_at: Instant::now(),
            at: Instant::now(),
            units: [
                PublisherServiceRole::Producer,
                PublisherServiceRole::Publisher,
            ]
            .into_iter()
            .map(|role| BroadcastServiceUnitSnapshot {
                role,
                host_name: "Local".to_owned(),
                unit_name: "fixture.service".to_owned(),
                state: ServiceState::Active,
            })
            .collect(),
            instances: Vec::new(),
            mixxx_holder: None,
            encoder: BroadcastEncoderSnapshot {
                server_name: "Not configured".to_owned(),
                configured: false,
                status: encoder::EncoderStatus::not_installed(),
            },
        };
        ShowPageVm::from_queue_publisher_readiness_and_event(
            QueueNowPlayingPageVm::builder().build(),
            Some(&snapshot),
            ShowLogPaneDisplay::closed(),
            None,
            Some(input),
        )
        .publisher
        .unwrap()
        .event
        .unwrap()
    }

    fn show_event_execute(
        command: EventRegistryCommand,
        input: &mut EventSectionInput,
    ) -> Option<EventRegistryAction> {
        let action = command.action;
        assert!(!show_event_project(input)
            .actions
            .registry_action(action)
            .disabled());
        begin_event_registry_action(input, action);
        let working = show_event_project(input);
        assert!(
            working.actions.create.disabled()
                && working.actions.replace.disabled()
                && working.actions.check.disabled()
        );
        let result = command
            .execute(&CommandContext::next())
            .map(|outcome| outcome.into_parts().0);
        apply_event_registry_result(input, action, result)
    }

    /// Situational ADR 0113: Replace preserves the dead registry entry and its token.
    fn seed_dead_recovery_event(
        conn: &Connection,
        directory: &Path,
        endpoint: &str,
    ) -> db::BroadcastEventRow {
        let token_path = directory.join("old.token");
        crate::broadcast::tokens::write_token_file(&token_path, "old-secret").unwrap();
        db::insert_broadcast_event(
            conn,
            &db::BroadcastEventInput {
                event_id: "event-old".to_owned(),
                label: None,
                endpoint: endpoint.to_owned(),
                token_path: token_path.to_string_lossy().into_owned(),
                created_at: 1,
                last_checked_at: Some(2),
                last_status: Some(db::BroadcastEventStatus::Dead),
                kind: db::BroadcastEventKind::Created,
            },
        )
        .unwrap();
        db::broadcast_event_by_event_id(conn, "event-old")
            .unwrap()
            .unwrap()
    }

    /// Situational ADR 0113: registration saves a private token and reports the new identity.
    fn assert_event_registration(input: &EventSectionInput) -> EventSelectionInput {
        let registered = input.selected_event.clone().unwrap();
        assert_eq!(registered.event_id, "event-new");
        assert_eq!(registered.state, EventState::Unknown);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            assert_eq!(
                fs::metadata(&registered.token_path)
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
        let row = show_event_project(input);
        assert_eq!(row.event.event_id.as_deref(), Some("event-new"));
        assert_eq!(row.event.token_path.as_ref(), Some(&registered.token_path));
        assert!(row.registration_message.unwrap().contains("registered"));
        registered
    }

    /// Situational ADR 0113: queue refresh preserves recovery state and never exposes token contents.
    fn assert_recovered_event_refresh(
        input: &mut EventSectionInput,
        expected: EventState,
        registered: &EventSelectionInput,
    ) {
        // The independent target read supplies current configuration; queue refresh preserves it.
        input.targets = EventTargetListInput::Loaded { targets: vec![] };
        let mut refreshed = input.clone();
        refreshed.feedback = EventCommandFeedback::default();
        refreshed.targets = EventTargetListInput::Loaded { targets: vec![] };
        let requested = Some(input.clone());
        let mut current = requested.clone();
        apply_event_refresh(&mut current, requested.as_ref(), Some(refreshed));
        let row = show_event_project(current.as_ref().unwrap());
        assert_eq!(row.event.state.state, expected);
        assert_eq!(!row.actions.attach.disabled(), expected == EventState::Live);
        assert_eq!(
            !row.actions.replace.disabled(),
            expected == EventState::Dead
        );
        assert!(!row.actions.check.disabled());
        assert_eq!(row.event.event_id.as_deref(), Some("event-new"));
        assert_eq!(row.event.token_path.as_ref(), Some(&registered.token_path));
        assert!(!format!("{row:?}").contains("fixture-secret"));
    }

    /// Situational ADR 0113: a failed check preserves registration success and offers a retry.
    fn assert_event_check_retry(input: &EventSectionInput) {
        let failed = show_event_project(input);
        assert_eq!(input.feedback.check_response, EventCheckResponse::Http(503));
        assert!(failed
            .diagnostics
            .contains("answered HTTP 503 for event event-new"));
        assert!(failed.diagnostics.contains("App created event event-new"));
        assert!(failed
            .diagnostics
            .contains("App kept the saved status of event event-new as Unknown"));
        assert!(failed.registration_message.unwrap().contains("registered"));
        assert!(failed.check_message.unwrap().contains("failed"));
        assert_eq!(failed.actions.check.label, "Retry check");
        assert!(!failed.actions.check.disabled());
        assert!(
            failed.actions.create.disabled()
                && failed.actions.replace.disabled()
                && failed.actions.attach.disabled()
        );
    }

    /// Situational ADR 0113: run real registry commands and the mounted-row presenter transitions.
    fn show_event_recovery_sequence(action: EventRegistryAction, retry_status: u16) {
        let temp = tempfile::tempdir().unwrap();
        let conn = Arc::new(Mutex::new(show_event_db(&temp)));
        let (endpoint, relay) = show_event_relay(vec![200, 503, retry_status]);
        let cfg_path = temp.path().join("config.toml");
        let config_before = format!(
            "musicindex_endpoint = {endpoint:?}\n[broadcast]\ndrop_file_target = \"default\"\n"
        );
        fs::write(&cfg_path, &config_before).unwrap();
        // A sentinel publisher configuration must survive registration/checking byte for byte.
        let publisher_path = temp.path().join("publisher-config.toml");
        fs::write(&publisher_path, "targets = []\n").unwrap();
        let old = if action == EventRegistryAction::Replace {
            Some(seed_dead_recovery_event(
                &conn.lock().unwrap(),
                temp.path(),
                &endpoint,
            ))
        } else {
            None
        };
        let mut input = EventSectionInput {
            liveness_confirmed: true,
            registry: EventRegistryInput::default(),
            context: String::new(),
            request: 0,
            feedback: EventCommandFeedback::default(),
            selected_event: selected_event_input(&conn.lock().unwrap()).unwrap(),
            targets: EventTargetListInput::Loaded { targets: vec![] },
            attach_target_name: "default".to_owned(),
            remote_host: false,
        };
        let selection = db::broadcast_event_selection(&conn.lock().unwrap()).unwrap();
        input.registry.selected_id = selection.event_id;
        input.registry.revision = selection.revision;
        let command = |action, input: &EventSectionInput| EventRegistryCommand {
            selection_revision: input.registry.revision,
            conn: Arc::clone(&conn),
            cfg_path: cfg_path.clone(),
            action,
            selected_event: input.selected_event.clone(),
        };
        let before_registration = Some(input.clone());
        let next = show_event_execute(command(action, &input), &mut input);
        assert_eq!(next, Some(EventRegistryAction::Check));
        let registered = assert_event_registration(&input);
        let token_before = fs::read(&registered.token_path).unwrap();
        // A refresh begun before registration must not erase the new identity or feedback.
        let mut current = Some(input.clone());
        apply_event_refresh(
            &mut current,
            before_registration.as_ref(),
            before_registration.clone(),
        );
        assert_eq!(current, Some(input.clone()));
        show_event_execute(command(next.unwrap(), &input), &mut input);
        assert_eq!(input.selected_event.as_ref(), Some(&registered));
        assert_event_check_retry(&input);
        show_event_execute(command(EventRegistryAction::Check, &input), &mut input);
        let stored = selected_event_input(&conn.lock().unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(stored.event_id, registered.event_id);
        assert_eq!(stored.token_path, registered.token_path);
        assert_eq!(fs::read(&registered.token_path).unwrap(), token_before);
        let expected = match retry_status {
            200 => EventState::Live,
            404 => EventState::Dead,
            _ => EventState::Unknown,
        };
        assert_eq!(stored.state, expected);
        assert_eq!(input.selected_event.as_ref(), Some(&stored));
        assert_recovered_event_refresh(&mut input, expected, &registered);
        assert_eq!(fs::read_to_string(&cfg_path).unwrap(), config_before);
        assert_eq!(
            fs::read_to_string(&publisher_path).unwrap(),
            "targets = []\n"
        );
        let locked = conn.lock().unwrap();
        assert_eq!(
            db::broadcast_events(&locked).unwrap().len(),
            if old.is_some() { 2 } else { 1 }
        );
        assert_eq!(
            db::broadcast_event_by_event_id(&locked, "event-old").unwrap(),
            old
        );
        if let Some(old) = old {
            assert_eq!(fs::read_to_string(old.token_path).unwrap(), "old-secret");
        }
        assert_eq!(
            relay.join().unwrap(),
            [
                "POST /v1/liveitems HTTP/1.1",
                "GET /v1/liveitems/event-new/metadata HTTP/1.1",
                "GET /v1/liveitems/event-new/metadata HTTP/1.1",
            ]
        );
    }

    #[test]
    fn show_event_create_initial_failure_then_retry_live() {
        show_event_recovery_sequence(EventRegistryAction::Create, 200);
    }
    #[test]
    fn show_event_replace_initial_failure_then_retry_live() {
        show_event_recovery_sequence(EventRegistryAction::Replace, 200);
    }
    #[test]
    fn show_event_create_failed_retry_remains_retryable() {
        show_event_recovery_sequence(EventRegistryAction::Create, 503);
    }
    #[test]
    fn show_event_replace_failed_retry_remains_retryable() {
        show_event_recovery_sequence(EventRegistryAction::Replace, 503);
    }
    #[test]
    fn show_event_create_retry_404_stores_dead_before_offering_replace() {
        show_event_recovery_sequence(EventRegistryAction::Create, 404);
    }
    #[test]
    fn show_event_replace_retry_404_stores_dead_before_offering_replace() {
        show_event_recovery_sequence(EventRegistryAction::Replace, 404);
    }
    /// Situational ADR 0113: an observed Live response is not success until its status is stored.
    #[test]
    fn show_event_status_write_failure_retains_unknown_and_retry() {
        let temp = tempfile::tempdir().unwrap();
        let conn = Arc::new(Mutex::new(show_event_db(&temp)));
        let (endpoint, relay) = show_event_relay(vec![200, 200, 200]);
        let cfg_path = temp.path().join("config.toml");
        fs::write(&cfg_path, format!("musicindex_endpoint = {endpoint:?}\n")).unwrap();
        let mut input = EventSectionInput {
            liveness_confirmed: true,
            registry: EventRegistryInput::default(),
            context: String::new(),
            request: 0,
            feedback: EventCommandFeedback::default(),
            selected_event: None,
            targets: EventTargetListInput::Unknown,
            attach_target_name: "default".to_owned(),
            remote_host: false,
        };
        let selection = db::broadcast_event_selection(&conn.lock().unwrap()).unwrap();
        input.registry.selected_id = selection.event_id;
        input.registry.revision = selection.revision;
        let command = |action, input: &EventSectionInput| EventRegistryCommand {
            selection_revision: input.registry.revision,
            conn: Arc::clone(&conn),
            cfg_path: cfg_path.clone(),
            action,
            selected_event: input.selected_event.clone(),
        };
        assert_eq!(
            show_event_execute(command(EventRegistryAction::Create, &input), &mut input),
            Some(EventRegistryAction::Check)
        );
        let registered = input.selected_event.clone();
        conn.lock().unwrap().execute_batch("CREATE TRIGGER reject_event_check BEFORE UPDATE ON broadcast_events BEGIN SELECT RAISE(FAIL, 'status write failed'); END;").unwrap();
        show_event_execute(command(EventRegistryAction::Check, &input), &mut input);
        assert_eq!(input.selected_event, registered);
        assert_eq!(
            selected_event_input(&conn.lock().unwrap()).unwrap(),
            registered
        );
        let row = show_event_project(&input);
        assert!(row.check_message.unwrap().contains("status write failed"));
        assert_eq!(
            input.feedback.check_response,
            EventCheckResponse::SaveFailed(EventState::Live)
        );
        assert!(row
            .diagnostics
            .contains("returned metadata for event event-new"));
        assert!(row
            .diagnostics
            .contains("could not finish saving that answer"));
        assert!(!row.diagnostics.contains("received no HTTP response"));
        assert_eq!(row.actions.check.label, "Retry check");
        assert!(!row.actions.check.disabled());
        assert!(row.actions.replace.disabled() && row.actions.attach.disabled());
        conn.lock()
            .unwrap()
            .execute_batch("DROP TRIGGER reject_event_check;")
            .unwrap();
        show_event_execute(command(EventRegistryAction::Check, &input), &mut input);
        assert_eq!(
            show_event_project(&input).event.state.state,
            EventState::Live
        );
        assert_eq!(relay.join().unwrap().len(), 3);
    }

    /// Situational ADR 0113: an in-flight registry command owns its row until completion.
    #[test]
    fn show_event_refresh_cannot_clear_progress_or_failure_feedback() {
        let mut input = EventSectionInput {
            liveness_confirmed: true,
            registry: EventRegistryInput::default(),
            context: String::new(),
            request: 0,
            feedback: EventCommandFeedback::default(),
            selected_event: None,
            targets: EventTargetListInput::Unknown,
            attach_target_name: "default".to_owned(),
            remote_host: false,
        };
        begin_event_registry_action(&mut input, EventRegistryAction::Create);
        let requested = Some(input.clone());
        let mut current = requested.clone();
        let mut refreshed = input.clone();
        refreshed.feedback = EventCommandFeedback::default();
        apply_event_refresh(&mut current, requested.as_ref(), Some(refreshed));
        assert_eq!(current, requested);
        apply_event_registry_result(
            &mut input,
            EventRegistryAction::Create,
            Err(CommandError::Other("registration rejected".to_owned())),
        );
        let row = show_event_project(&input);
        assert!(row
            .registration_message
            .unwrap()
            .contains("registration rejected"));
        assert!(!row.actions.create.disabled());
        assert!(row.actions.replace.disabled() && row.actions.check.disabled());
        assert_eq!(row.event.state.state, EventState::None);
    }
    fn stored_test_event(
        conn: &Connection,
        directory: &Path,
        id: &str,
        endpoint: &str,
        created: i64,
    ) {
        let token = directory.join(format!("{id}.token"));
        crate::broadcast::tokens::write_token_file(&token, "preserve-secret").unwrap();
        db::insert_broadcast_event(
            conn,
            &db::BroadcastEventInput {
                event_id: id.to_owned(),
                label: None,
                endpoint: endpoint.to_owned(),
                token_path: token.to_string_lossy().into_owned(),
                created_at: created,
                last_checked_at: Some(created),
                last_status: Some(db::BroadcastEventStatus::Dead),
                kind: db::BroadcastEventKind::Created,
            },
        )
        .unwrap();
    }

    /// Situational ADR 0113: chosen older rows remain operable; changing revision rejects stale commands.
    #[test]
    fn compact_event_saved_choice_drives_commands_and_refresh() {
        let temp = tempfile::tempdir().unwrap();
        let conn = Arc::new(Mutex::new(show_event_db(&temp)));
        let (endpoint, relay) = show_event_relay(vec![404, 200]);
        let cfg_path = temp.path().join("config.toml");
        fs::write(&cfg_path, format!("musicindex_endpoint = {endpoint:?}\n")).unwrap();
        {
            let conn = conn.lock().unwrap();
            stored_test_event(&conn, temp.path(), "older", &endpoint, 1);
            db::select_broadcast_event(&conn, "older").unwrap();
            stored_test_event(&conn, temp.path(), "newer", &endpoint, 2);
        }
        let mut input = load_event_section(
            &conn.lock().unwrap(),
            &config::ConfigSnapshot::from_bytes(std::path::Path::new("fixture.toml"), Vec::new())
                .unwrap()
                .broadcast(),
        );
        assert_eq!(input.registry.events[0].event_id, "newer");
        assert_eq!(input.selected_event.as_ref().unwrap().event_id, "older");
        for action in [EventRegistryAction::Check, EventRegistryAction::Replace] {
            let command = EventRegistryCommand {
                conn: Arc::clone(&conn),
                cfg_path: cfg_path.clone(),
                action,
                selected_event: input.selected_event.clone(),
                selection_revision: input.registry.revision,
            };
            let result = command
                .execute(&CommandContext::next())
                .unwrap()
                .into_parts()
                .0;
            apply_event_registry_result(&mut input, action, Ok(result));
        }
        assert_eq!(input.selected_event.as_ref().unwrap().event_id, "event-new");
        assert_eq!(
            fs::read_to_string(temp.path().join("older.token")).unwrap(),
            "preserve-secret"
        );
        assert_eq!(
            fs::read_to_string(temp.path().join("newer.token")).unwrap(),
            "preserve-secret"
        );
        let stale = EventRegistryCommand {
            conn: Arc::clone(&conn),
            cfg_path,
            action: EventRegistryAction::Check,
            selected_event: input.selected_event.clone(),
            selection_revision: input.registry.revision,
        };
        db::select_broadcast_event(&conn.lock().unwrap(), "older").unwrap();
        assert!(stale.execute(&CommandContext::next()).is_err());
        assert_eq!(
            relay.join().unwrap(),
            [
                "GET /v1/liveitems/older/metadata HTTP/1.1",
                "POST /v1/liveitems HTTP/1.1"
            ]
        );
        assert_eq!(
            selected_event_input(&conn.lock().unwrap())
                .unwrap()
                .unwrap()
                .event_id,
            "older"
        );
    }

    /// Situational ADR 0113: failed preference storage cannot erase successful registration or repeat POST.
    #[test]
    fn compact_event_registration_survives_selection_save_failure() {
        let temp = tempfile::tempdir().unwrap();
        let conn = Arc::new(Mutex::new(show_event_db(&temp)));
        let (endpoint, relay) = show_event_relay(vec![200]);
        let cfg_path = temp.path().join("config.toml");
        fs::write(&cfg_path, format!("musicindex_endpoint = {endpoint:?}\n")).unwrap();
        conn.lock().unwrap().execute_batch("CREATE TRIGGER reject_choice BEFORE INSERT ON broadcast_event_selection BEGIN SELECT RAISE(FAIL, 'choice disk failure'); END;").unwrap();
        let mut input = load_event_section(
            &conn.lock().unwrap(),
            &config::ConfigSnapshot::from_bytes(std::path::Path::new("fixture.toml"), Vec::new())
                .unwrap()
                .broadcast(),
        );
        let command = EventRegistryCommand {
            conn: Arc::clone(&conn),
            cfg_path,
            action: EventRegistryAction::Create,
            selected_event: None,
            selection_revision: 0,
        };
        let result = command
            .execute(&CommandContext::next())
            .unwrap()
            .into_parts()
            .0;
        let EventRegistryResult::Complete(ref success) = result else {
            panic!("registration must return its stored event");
        };
        let token_path = success.event.token_path.clone();
        assert_eq!(
            apply_event_registry_result(&mut input, EventRegistryAction::Create, Ok(result)),
            None
        );
        assert_eq!(input.feedback.registration, EventCommandState::Succeeded);
        assert!(matches!(
            input.feedback.selection,
            EventCommandState::Failed { .. }
        ));
        assert_eq!(input.registry.events.len(), 1);
        assert!(show_event_project(&input).actions.create.disabled());
        assert_eq!(fs::read_to_string(token_path).unwrap(), "fixture-secret");
        assert_eq!(relay.join().unwrap().len(), 1);
    }

    /// Situational ADR 0113: a missing saved entry and a failed registry read never offer Create.
    #[test]
    fn compact_event_missing_choice_and_registry_errors_are_explicit() {
        let temp = tempfile::tempdir().unwrap();
        let conn = show_event_db(&temp);
        stored_test_event(&conn, temp.path(), "old", "http://127.0.0.1:1", 1);
        db::select_broadcast_event(&conn, "old").unwrap();
        stored_test_event(&conn, temp.path(), "new", "http://127.0.0.1:1", 2);
        let old = db::broadcast_event_by_event_id(&conn, "old")
            .unwrap()
            .unwrap();
        db::delete_broadcast_event(&conn, old.id).unwrap();
        let input = load_event_section(
            &conn,
            &config::ConfigSnapshot::from_bytes(std::path::Path::new("fixture.toml"), Vec::new())
                .unwrap()
                .broadcast(),
        );
        assert!(input.selected_event.is_none());
        assert_eq!(input.registry.selected_id.as_deref(), Some("old"));
        assert_eq!(show_event_project(&input).badge.label, "Event unavailable");
        assert!(show_event_project(&input).actions.create.disabled());
        conn.execute_batch("DROP TABLE broadcast_events").unwrap();
        let input = load_event_section(
            &conn,
            &config::ConfigSnapshot::from_bytes(std::path::Path::new("fixture.toml"), Vec::new())
                .unwrap()
                .broadcast(),
        );
        assert!(matches!(
            input.registry.status,
            EventRegistryStatus::Failed(_)
        ));
        let event = show_event_project(&input);
        assert_eq!(event.badge.label, "Events unavailable");
        assert!(event.actions.create.disabled());
        assert_eq!(event.primary.unwrap().intent, EventControlIntent::Refresh);
    }

    /// Situational ADR 0113: refreshes cannot resurrect old confirmation or another context's results.
    #[test]
    fn compact_event_context_revision_and_configured_detach_are_scoped() {
        let mut input = empty_event_section(
            &config::ConfigSnapshot::from_bytes(std::path::Path::new("fixture.toml"), Vec::new())
                .unwrap()
                .broadcast(),
        );
        input.context = "host / instance / default".to_owned();
        input.request = 7;
        input.registry.status = EventRegistryStatus::Loaded;
        input.registry.selected_id = Some("a".to_owned());
        input.registry.revision = 3;
        input.selected_event = Some(EventSelectionInput {
            kind: EventKind::Created,
            created_at: 1,
            last_checked_at: Some(2),
            label: None,
            event_id: "a".to_owned(),
            endpoint: "http://localhost".to_owned(),
            token_path: "/fixture/token".to_owned(),
            state: EventState::Dead,
            token_file_missing: false,
        });
        input.attach_target_name = "default".to_owned();
        input.targets = EventTargetListInput::Loaded {
            targets: vec![EventTargetInput {
                name: "unused".to_owned(),
                event_id: "a".to_owned(),
            }],
        };
        assert!(attached_target_name(&input).is_none());
        let EventTargetListInput::Loaded { targets } = &mut input.targets else {
            unreachable!()
        };
        targets.push(EventTargetInput {
            name: "default".to_owned(),
            event_id: "a".to_owned(),
        });
        assert_eq!(
            event_target_name_for_operation(EventTargetOperation::Detach, &input).unwrap(),
            "default"
        );
        let requested = Some(input.clone());
        input.request += 1;
        let mut current = Some(input.clone());
        assert!(!apply_event_refresh(
            &mut current,
            requested.as_ref(),
            requested.clone()
        ));
        assert_eq!(current.as_ref().unwrap().request, 8);
        let mut new_context = input.clone();
        new_context.context = "another host".to_owned();
        new_context.feedback = EventCommandFeedback::default();
        new_context.targets = EventTargetListInput::Unknown;
        input.feedback.check = EventCommandState::Working;
        current = Some(input.clone());
        apply_event_refresh(&mut current, Some(&input), Some(new_context.clone()));
        assert_eq!(current, Some(new_context));
    }

    /// ADR 0110 Decision 2: the dialog command stores a Reserved row, selects it and keeps the file.
    #[cfg(unix)]
    #[test]
    fn adr_0110_add_reserved_command_stores_selects_and_keeps_the_token_file() {
        use std::os::unix::fs::PermissionsExt as _;

        let temp = tempfile::tempdir().unwrap();
        let conn = Arc::new(Mutex::new(show_event_db(&temp)));
        let (endpoint, relay) = show_event_relay(vec![200]);
        let cfg_path = temp.path().join("config.toml");
        fs::write(&cfg_path, format!("musicindex_endpoint = {endpoint:?}\n")).unwrap();
        let token = temp.path().join("show.token");
        fs::write(&token, "fixture-token\n").unwrap();
        fs::set_permissions(&token, fs::Permissions::from_mode(0o600)).unwrap();
        let command = |event_id: &str, path: &Path| AddReservedEventCommand {
            conn: Arc::clone(&conn),
            cfg_path: cfg_path.clone(),
            event_id: event_id.to_owned(),
            token_path: path.to_path_buf(),
            label: Some("Friday".to_owned()),
        };

        let refused = command("event-one", &temp.path().join("missing.token"))
            .execute(&CommandContext::next())
            .map(|_| ())
            .unwrap_err()
            .to_string();
        assert!(refused.contains("check 3"), "{refused}");
        assert!(db::broadcast_events(&conn.lock().unwrap())
            .unwrap()
            .is_empty());

        let added = command("event-one", &token)
            .execute(&CommandContext::next())
            .unwrap()
            .into_parts()
            .0;
        assert_eq!(added.event_id, "event-one");
        assert_eq!(added.check_failure, None);
        assert_eq!(added.selection_error, None);
        let conn = conn.lock().unwrap();
        let row = db::broadcast_event_by_event_id(&conn, "event-one")
            .unwrap()
            .unwrap();
        assert_eq!(row.kind, db::BroadcastEventKind::Reserved);
        assert_eq!(row.token_path, token.to_string_lossy());
        assert_eq!(
            db::broadcast_event_selection(&conn)
                .unwrap()
                .event_id
                .as_deref(),
            Some("event-one")
        );
        assert_eq!(fs::read_to_string(&token).unwrap(), "fixture-token\n");
        assert_eq!(
            fs::metadata(&token).unwrap().permissions().mode() & 0o777,
            0o600
        );
        let loaded = event_selection_input(row);
        assert_eq!(loaded.kind, EventKind::Reserved);
        relay.join().unwrap();
    }

    /// ADR 0110 Decision 5: the command refuses Replace for a Dead reserved row.
    #[test]
    fn adr_0110_replace_command_refuses_a_dead_reserved_row() {
        let temp = tempfile::tempdir().unwrap();
        let conn = Arc::new(Mutex::new(show_event_db(&temp)));
        let cfg_path = temp.path().join("config.toml");
        fs::write(&cfg_path, "musicindex_endpoint = \"http://127.0.0.1:9\"\n").unwrap();
        {
            let conn = conn.lock().unwrap();
            db::insert_broadcast_event(
                &conn,
                &db::BroadcastEventInput {
                    event_id: "event-old".to_owned(),
                    label: None,
                    endpoint: "http://127.0.0.1:9".to_owned(),
                    token_path: temp.path().join("old.token").to_string_lossy().into_owned(),
                    created_at: 1,
                    last_checked_at: Some(2),
                    last_status: Some(db::BroadcastEventStatus::Dead),
                    kind: db::BroadcastEventKind::Reserved,
                },
            )
            .unwrap();
            db::select_broadcast_event(&conn, "event-old").unwrap();
        }
        let (selected, revision) = {
            let conn = conn.lock().unwrap();
            (
                selected_event_input(&conn).unwrap(),
                db::broadcast_event_selection(&conn).unwrap().revision,
            )
        };
        let error = EventRegistryCommand {
            conn: Arc::clone(&conn),
            cfg_path,
            action: EventRegistryAction::Replace,
            selected_event: selected,
            selection_revision: revision,
        }
        .execute(&CommandContext::next())
        .map(|_| ())
        .unwrap_err();
        assert!(error.to_string().contains("unavailable"), "{error}");
        assert_eq!(
            db::broadcast_events(&conn.lock().unwrap()).unwrap().len(),
            1
        );
    }

    /// ADR 0110 Decision 7: the Go on air step refuses to replace a different event, and runs for the same event.
    #[test]
    fn adr_0110_go_on_air_attach_step_refuses_a_different_event() {
        let input = |target_event: &str, targets: Option<EventTargetListInput>| {
            let mut input = EventSectionInput {
                liveness_confirmed: true,
                registry: EventRegistryInput::default(),
                context: String::new(),
                request: 0,
                feedback: EventCommandFeedback::default(),
                selected_event: Some(EventSelectionInput {
                    kind: EventKind::Created,
                    created_at: 0,
                    last_checked_at: None,
                    label: None,
                    event_id: "event-one".to_owned(),
                    endpoint: "https://relay.example".to_owned(),
                    token_path: "/fixture/one.token".to_owned(),
                    state: EventState::Live,
                    token_file_missing: false,
                }),
                targets: EventTargetListInput::Loaded {
                    targets: vec![EventTargetInput {
                        name: "default".to_owned(),
                        event_id: target_event.to_owned(),
                    }],
                },
                attach_target_name: "default".to_owned(),
                remote_host: false,
            };
            if let Some(targets) = targets {
                input.targets = targets;
            }
            input
        };
        let message = on_air_attach_refusal(&input("event-other", None)).expect("refusal");
        assert!(message.contains("event-other") && message.contains("event-one"));
        assert!(message.contains("Attach in the Live Metadata inspector"));
        assert_eq!(on_air_attach_refusal(&input("event-one", None)), None);
        assert_eq!(on_air_attach_refusal(&input("", None)), None);
        let unread = input("event-other", Some(EventTargetListInput::Unknown));
        assert_eq!(on_air_attach_refusal(&unread), None);
    }
}

/// The dialog "Add reserved event" (ADR 0110 Decision 2). The screen owns the
/// inputs. The dialog composite owns only the layout.
pub(super) struct AddReservedDialog {
    show_name: Entity<InputState>,
    event_id: Entity<InputState>,
    token_path: Entity<InputState>,
    label: Entity<InputState>,
    state: Entity<ReservedDialogState>,
    _subscriptions: Vec<gpui::Subscription>,
}

/// What the dialog shows besides its inputs.
struct ReservedDialogState {
    /// The token folder of `HitIt`. `None` when the config folder is unknown.
    token_folder: Option<PathBuf>,
    /// The path that `HitIt` last put in the token file field.
    suggested: String,
    outcome: ReservedAddOutcome,
}

/// The display of the dialog at each render. It reads only the dialog entities.
fn add_reserved_display(
    inputs: &[Entity<InputState>; 4],
    state: &Entity<ReservedDialogState>,
    cx: &gpui::App,
) -> ConfirmationDialogDisplay {
    use crate::ui::composites::{
        ConfirmationDialogCopyText, ConfirmationDialogField, ConfirmationDialogForm,
    };
    let [show_name, event_id, token_path, label] = inputs;
    let state = state.read(cx);
    let name = show_name.read(cx).value().trim().to_owned();
    let command = state
        .token_folder
        .as_deref()
        .and_then(|folder| crate::view_models::show::suggested_token_path(folder, &name))
        .and_then(|path| crate::view_models::show::relay_command(&name, &path.to_string_lossy()));
    let command_text = command.clone().unwrap_or_else(|| {
        if name.is_empty() {
            "Type a show name to see the relay command.".to_owned()
        } else if state.token_folder.is_none() {
            "The token folder of HitIt is not known.".to_owned()
        } else {
            "Use only letters, digits, - and _ in the show name. Do not start it with a dot."
                .to_owned()
        }
    });
    let field = |label: &str, input: &Entity<InputState>| ConfirmationDialogField {
        label: label.to_owned().into(),
        input: input.clone(),
    };
    let finished = matches!(
        state.outcome,
        ReservedAddOutcome::Working | ReservedAddOutcome::Added { .. }
    );
    ConfirmationDialogDisplay {
        title: "Add reserved event".into(),
        message: "Reserve the event with the relay command. Then enter its event ID and the token file here. HitIt does not copy or change the token file.".into(),
        cancel_button_id: "show-add-reserved-close".into(),
        cancel_label: "Close".into(),
        cancel_a11y_label: "Close the dialog without adding a reserved event".into(),
        confirm_button_id: "show-add-reserved-add".into(),
        confirm_label: "Add".into(),
        confirm_a11y_label: "Add the reserved event to the event list".into(),
        destructive: false,
        confirm_enabled: !finished,
        items: Vec::new(),
        section: None,
        form: Some(ConfirmationDialogForm {
            fields: vec![
                field("Show name", show_name),
                field("Event ID", event_id),
                field("Token file path", token_path),
                field("Label (optional)", label),
            ],
            copy_text: Some(ConfirmationDialogCopyText {
                label: "Relay command".into(),
                text: command_text.into(),
                button_id: "show-add-reserved-copy".into(),
                button_label: "Copy".into(),
                button_a11y_label: "Copy the relay command".into(),
                enabled: command.is_some(),
            }),
            result: state
                .outcome
                .message()
                .map(|(text, failure)| (text.into(), failure)),
            keeps_open: true,
        }),
    }
}

impl TopApp {
    /// Opens the dialog "Add reserved event" (ADR 0110 Decision 2).
    pub(super) fn open_add_reserved_dialog(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        let available = self
            .show_page
            .publisher
            .as_ref()
            .and_then(|section| section.event.as_ref())
            .is_some_and(|event| !event.actions.add_reserved.disabled());
        if !available || self.add_reserved_dialog.is_some() {
            return;
        }
        let new_input =
            |placeholder: &'static str, window: &mut gpui::Window, cx: &mut Context<Self>| {
                cx.new(|cx: &mut Context<InputState>| {
                    InputState::new(window, cx).placeholder(placeholder)
                })
            };
        let show_name = new_input("Show name", window, cx);
        let event_id = new_input("Event ID", window, cx);
        let token_path = new_input("Absolute path of the token file", window, cx);
        let label = new_input("Label", window, cx);
        let state = cx.new(|_| ReservedDialogState {
            token_folder: crate::broadcast::tokens::default_token_directory().ok(),
            suggested: String::new(),
            outcome: ReservedAddOutcome::Idle,
        });
        let subscriptions = vec![
            cx.subscribe_in(&show_name, window, Self::on_reserved_show_name_event),
            cx.subscribe_in(&event_id, window, Self::on_reserved_field_event),
            cx.subscribe_in(&token_path, window, Self::on_reserved_field_event),
            cx.subscribe_in(&label, window, Self::on_reserved_field_event),
        ];
        let inputs = [
            show_name.clone(),
            event_id.clone(),
            token_path.clone(),
            label.clone(),
        ];
        let display_state = state.clone();
        self.add_reserved_dialog = Some(AddReservedDialog {
            show_name,
            event_id,
            token_path,
            label,
            state,
            _subscriptions: subscriptions,
        });
        let cancel_entity = cx.weak_entity();
        let confirm_entity = cx.weak_entity();
        open_confirmation_dialog_from(
            window,
            cx,
            false,
            move |cx| add_reserved_display(&inputs, &display_state, cx),
            ConfirmationDialogHandlers::new(
                move |_, cx: &mut gpui::App| {
                    let _ = cancel_entity.update(cx, |this, _| this.add_reserved_dialog = None);
                },
                move |_, cx: &mut gpui::App| {
                    let _ = confirm_entity.update(cx, TopApp::submit_add_reserved);
                },
            ),
        );
    }

    /// A change of the show name puts the suggested path in the token file
    /// field while the operator has not changed that field (ADR 0110 Decision 3).
    fn on_reserved_show_name_event(
        &mut self,
        _input: &Entity<InputState>,
        event: &InputEvent,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        if !matches!(event, InputEvent::Change) {
            return;
        }
        let Some(dialog) = &self.add_reserved_dialog else {
            return;
        };
        let name = dialog.show_name.read(cx).value().trim().to_owned();
        let current = dialog.token_path.read(cx).value().to_string();
        let (folder, suggested) = {
            let state = dialog.state.read(cx);
            (state.token_folder.clone(), state.suggested.clone())
        };
        let next = folder
            .as_deref()
            .and_then(|folder| crate::view_models::show::suggested_token_path(folder, &name))
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_default();
        if current.is_empty() || current == suggested {
            dialog
                .token_path
                .update(cx, |input, cx| input.set_value(next.clone(), window, cx));
            dialog.state.update(cx, |state, _| state.suggested = next);
        }
        cx.refresh_windows();
    }

    /// A change in another field clears an old result and redraws the dialog.
    fn on_reserved_field_event(
        &mut self,
        _input: &Entity<InputState>,
        event: &InputEvent,
        _window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        if !matches!(event, InputEvent::Change) {
            return;
        }
        if let Some(dialog) = &self.add_reserved_dialog {
            dialog.state.update(cx, |state, _| {
                if matches!(state.outcome, ReservedAddOutcome::Failed(_)) {
                    state.outcome = ReservedAddOutcome::Idle;
                }
            });
        }
        cx.refresh_windows();
    }

    /// The button "Add" of the dialog.
    fn submit_add_reserved(&mut self, cx: &mut Context<Self>) {
        let Some(dialog) = &self.add_reserved_dialog else {
            return;
        };
        let event_id = dialog.event_id.read(cx).value().trim().to_owned();
        let token_path = dialog.token_path.read(cx).value().trim().to_owned();
        let label = dialog.label.read(cx).value().trim().to_owned();
        let state = dialog.state.clone();
        state.update(cx, |state, _| state.outcome = ReservedAddOutcome::Working);
        cx.refresh_windows();
        let command = AddReservedEventCommand {
            conn: Arc::clone(&self.conn),
            cfg_path: self.cfg_path.clone(),
            event_id,
            token_path: PathBuf::from(token_path),
            label: (!label.is_empty()).then_some(label),
        };
        let error_state = state.clone();
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, result, cx| {
                state.update(cx, |state, _| {
                    state.outcome = ReservedAddOutcome::Added {
                        event_id: result.event_id,
                        check_failure: result.check_failure,
                        selection_error: result.selection_error,
                    };
                });
                cx.refresh_windows();
                this.refresh_show_page(cx);
            },
            move |_, error, cx| {
                error_state.update(cx, |state, _| {
                    state.outcome = ReservedAddOutcome::Failed(error.to_string());
                });
                cx.refresh_windows();
            },
        );
    }

    /// Attach asks before it replaces a different event on the target
    /// (ADR 0110 Decision 7). With no different event it runs at once.
    pub(super) fn attach_event_after_question(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        use crate::ui::composites::ConfirmationDialogItem;
        let event = self
            .show_page
            .publisher
            .as_ref()
            .and_then(|section| section.event.as_ref());
        if event.is_none_or(|event| event.actions.attach.disabled()) {
            return;
        }
        let Some(question) = event.and_then(|event| event.attach_confirmation.clone()) else {
            self.run_event_control(EventControlIntent::Attach, cx);
            return;
        };
        let item = |title: &str, id: String| ConfirmationDialogItem {
            check: None,
            checked_lines: Vec::new(),
            title: title.to_owned().into(),
            subtitle: None,
            mark: None,
            lines: vec![id.into()],
        };
        let entity = cx.weak_entity();
        open_confirmation_dialog(
            window,
            cx,
            ConfirmationDialogDisplay {
                confirm_enabled: true,
                form: None,
                section: None,
                title: "Replace the event on the target?".into(),
                message: format!(
                    "Target {} carries a different event. Attach replaces it with the selected event and restarts the relay sender.",
                    question.target_name
                )
                .into(),
                cancel_button_id: "show-attach-replace-cancel".into(),
                cancel_label: "Cancel".into(),
                cancel_a11y_label: "Keep the event on the target".into(),
                confirm_button_id: "show-attach-replace-confirm".into(),
                confirm_label: "Replace event".into(),
                confirm_a11y_label: format!(
                    "Replace the event on target {} with the selected event",
                    question.target_name
                )
                .into(),
                destructive: true,
                items: vec![
                    item("Event on the target", question.target_event_id),
                    item("Selected event", question.selected_event_id),
                ],
            },
            ConfirmationDialogHandlers::new(
                |_, _: &mut gpui::App| {},
                move |_, cx: &mut gpui::App| {
                    let _ = entity.update(cx, |this, cx| {
                        this.run_event_control(EventControlIntent::Attach, cx);
                    });
                },
            ),
        );
    }
}

/// The error of the Go on air step "Attach" when the configured target carries a
/// different event (ADR 0110 Decision 7). `None` when the step can run. It also
/// runs when `HitIt` could not read the target list.
fn on_air_attach_refusal(input: &EventSectionInput) -> Option<String> {
    let question = crate::view_models::show::attach_confirmation(input)?;
    Some(format!(
        "Go on air stopped. Target {} carries event {}, and the selected event is {}. Use Attach in the Live Metadata inspector to replace it.",
        question.target_name, question.target_event_id, question.selected_event_id
    ))
}

/// Adds a reserved event and selects it (ADR 0110 Decision 2).
struct AddReservedEventCommand {
    conn: Arc<Mutex<Connection>>,
    cfg_path: PathBuf,
    event_id: String,
    token_path: PathBuf,
    label: Option<String>,
}

struct AddedReservedEventResult {
    event_id: String,
    check_failure: Option<String>,
    selection_error: Option<String>,
}

impl ApplicationCommand for AddReservedEventCommand {
    type Output = AddedReservedEventResult;

    fn execute(self, context: &CommandContext) -> crate::application::CommandResult<Self::Output> {
        if context.cancellation().is_cancelled() {
            return Err(CommandError::Cancelled);
        }
        let conn = self
            .conn
            .lock()
            .map_err(|_| CommandError::Query("database lock poisoned".to_owned()))?;
        let endpoint =
            config::load_musicindex_endpoint(&self.cfg_path).map_err(event_registry_error)?;
        let directory = self
            .cfg_path
            .parent()
            .ok_or_else(|| CommandError::Other("Config path has no directory".to_owned()))?
            .join("broadcast")
            .join("tokens");
        let registry = BroadcastRegistry::with_token_directory(&conn, &endpoint, directory)
            .map_err(event_registry_error)?;
        let added = registry
            .add_reserved_event(&self.event_id, &self.token_path, self.label.as_deref())
            .map_err(event_registry_error)?;
        let selection_error = db::select_broadcast_event(&conn, &added.event.event_id)
            .err()
            .map(|error| format!("{error:#}"));
        Ok(CommandOutcome::without_events(AddedReservedEventResult {
            event_id: added.event.event_id,
            check_failure: added.check_failure,
            selection_error,
        }))
    }
}

/// Session-only latest results; no token contents or persistent audit trail (ADR 0113).
#[derive(Default)]
pub(super) struct EventSession {
    next_request: u64,
    feedback: HashMap<(String, String), EventCommandFeedback>,
}

impl EventSession {
    fn remember(&mut self, input: &EventSectionInput) {
        if let Some(event) = &input.selected_event {
            self.feedback.insert(
                (input.context.clone(), event.event_id.clone()),
                input.feedback.clone(),
            );
        }
    }
}

impl TopApp {
    fn event_request_is_current(&self, request: u64, context: &str) -> bool {
        context == event_context(&self.broadcast)
            && self
                .event_section_input
                .as_ref()
                .is_some_and(|input| input.request == request && input.context == context)
    }

    fn run_event_control(&mut self, intent: EventControlIntent, cx: &mut Context<Self>) {
        match intent {
            EventControlIntent::Create => {
                self.run_event_registry_command(EventRegistryAction::Create, cx);
            }
            EventControlIntent::Replace => {
                self.run_event_registry_command(EventRegistryAction::Replace, cx);
            }
            EventControlIntent::Check => {
                self.run_event_registry_command(EventRegistryAction::Check, cx);
            }
            EventControlIntent::Attach => {
                self.run_event_target_command(EventTargetOperation::Attach, cx);
            }
            EventControlIntent::Detach => {
                self.run_event_target_command(EventTargetOperation::Detach, cx);
            }
            EventControlIntent::Refresh => self.refresh_show_page(cx),
            EventControlIntent::ReadTargets => {
                if self
                    .event_section_input
                    .as_ref()
                    .is_none_or(|input| input.feedback.working())
                {
                    return;
                }
                self.event_session.next_request += 1;
                if let Some(input) = self.event_section_input.as_mut() {
                    input.request = self.event_session.next_request;
                    input.feedback.active_action = Some(intent);
                }
                self.read_event_targets(self.event_session.next_request, cx);
            }
            // The dialog needs the window. `with_event_registry_slots` opens it.
            EventControlIntent::CopyFeedTag | EventControlIntent::AddReserved => {}
        }
    }

    fn select_show_event(&mut self, event_id: String, cx: &mut Context<Self>) {
        let Some(input) = &mut self.event_section_input else {
            return;
        };
        if input.feedback.working()
            || !input
                .registry
                .events
                .iter()
                .any(|event| event.event_id == event_id)
        {
            return;
        }
        self.event_session.remember(input);
        self.event_session.next_request += 1;
        input.request = self.event_session.next_request;
        input.feedback.selection = EventCommandState::Working;
        let requested_event = input
            .registry
            .events
            .iter()
            .find(|event| event.event_id == event_id)
            .cloned();
        input.record_event_report_for(
            EventReportOperation::Selection,
            chrono::Utc::now(),
            requested_event.clone(),
        );
        let request = input.request;
        let context = input.context.clone();
        let error_context = context.clone();
        let command = SelectShowEvent {
            conn: Arc::clone(&self.conn),
            broadcast: self.broadcast.clone(),
            event_id,
            expected_revision: input.registry.revision,
        };
        self.reproject_show_page_from_current_queue();
        cx.notify();
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, mut input, cx| {
                if !this.event_request_is_current(request, &context) {
                    return;
                }
                if let Some(event) = &input.selected_event {
                    input.feedback = this
                        .event_session
                        .feedback
                        .get(&(context.clone(), event.event_id.clone()))
                        .cloned()
                        .unwrap_or_default();
                }
                input.feedback.selection = EventCommandState::Succeeded;
                input.record_event_report(EventReportOperation::Selection, chrono::Utc::now());
                input.request = request;
                this.event_section_input = Some(input);
                this.reproject_show_page_from_current_queue();
                this.run_event_registry_command(EventRegistryAction::Check, cx);
            },
            move |this, error, cx| {
                if !this.event_request_is_current(request, &error_context) {
                    return;
                }
                if let Some(input) = &mut this.event_section_input {
                    input.feedback.selection = EventCommandState::Failed {
                        detail: error.to_string(),
                    };
                    input.record_event_report_for(
                        EventReportOperation::Selection,
                        chrono::Utc::now(),
                        requested_event.clone(),
                    );
                }
                this.reproject_show_page_from_current_queue();
                cx.notify();
            },
        );
    }

    fn read_event_targets(&mut self, request: u64, cx: &mut Context<Self>) {
        let Some(input) = &mut self.event_section_input else {
            return;
        };
        input.feedback.target_read = EventCommandState::Working;
        input.record_event_report(EventReportOperation::TargetRead, chrono::Utc::now());
        let context = input.context.clone();
        let error_context = context.clone();
        let command = ReadEventTargets {
            broadcast: self.broadcast.clone(),
        };
        self.reproject_show_page_from_current_queue();
        cx.notify();
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, targets, cx| {
                if !this.event_request_is_current(request, &context) {
                    return;
                }
                this.apply_target_read(targets, cx);
            },
            move |this, error, cx| {
                if !this.event_request_is_current(request, &error_context) {
                    return;
                }
                this.apply_target_read(
                    EventTargetListInput::Failed {
                        detail: error.to_string(),
                    },
                    cx,
                );
            },
        );
    }

    fn apply_target_read(&mut self, targets: EventTargetListInput, cx: &mut Context<Self>) {
        if let Some(input) = &mut self.event_section_input {
            input.feedback.target_read = match &targets {
                EventTargetListInput::Loaded { .. } => EventCommandState::Succeeded,
                EventTargetListInput::Failed { detail } => EventCommandState::Failed {
                    detail: detail.clone(),
                },
                EventTargetListInput::CommandsUnavailable => EventCommandState::Failed {
                    detail: "Relay sender target commands unavailable".to_owned(),
                },
                EventTargetListInput::NotReachable => EventCommandState::Failed {
                    detail: "Relay sender not reachable".to_owned(),
                },
                EventTargetListInput::Unknown => EventCommandState::Failed {
                    detail: "No target observation".to_owned(),
                },
            };
            input.targets = targets;
            input.record_event_report(EventReportOperation::TargetRead, chrono::Utc::now());
            if input.feedback.target_mutation == EventCommandState::Working {
                input.feedback.target_mutation = EventCommandState::Succeeded;
                input.record_event_report(EventReportOperation::TargetMutation, chrono::Utc::now());
            }
        }
        self.reproject_show_page_from_current_queue();
        cx.notify();
    }

    fn finish_event_target(
        &mut self,
        request: u64,
        context: &str,
        command_id: ShowCommandId,
        completion: ShowCommandCompletion,
        cx: &mut Context<Self>,
    ) {
        // Even a failed restart requires fresh service observation: the target write may have succeeded.
        if !self
            .show_commands
            .complete(command_id, completion.returned_at, true)
        {
            return;
        }
        self.invalidate_publisher_service_snapshot();
        if !self.event_request_is_current(request, context) {
            return;
        }
        let on_air_result = completion
            .result
            .as_ref()
            .copied()
            .map_err(ToString::to_string);
        if let Some(input) = &mut self.event_section_input {
            if let Err(error) = completion.result {
                input.feedback.target_mutation = EventCommandState::Failed {
                    detail: error.to_string(),
                };
                input.record_event_report(EventReportOperation::TargetMutation, chrono::Utc::now());
            }
        }
        self.advance_on_air(Some(command_id), on_air_result, cx);
        self.read_event_targets(request, cx);
    }
}

struct SelectShowEvent {
    conn: Arc<Mutex<Connection>>,
    broadcast: config::BroadcastCapabilities,
    event_id: String,
    expected_revision: i64,
}

impl ApplicationCommand for SelectShowEvent {
    type Output = EventSectionInput;
    fn execute(self, context: &CommandContext) -> crate::application::CommandResult<Self::Output> {
        if context.cancellation().is_cancelled() {
            return Err(CommandError::Cancelled);
        }
        let conn = self
            .conn
            .lock()
            .map_err(|_| event_registry_error("database lock poisoned"))?;
        let current = db::broadcast_event_selection(&conn).map_err(event_registry_error)?;
        if current.revision != self.expected_revision {
            return Err(event_registry_error(
                "Event choice changed; refresh the events list",
            ));
        }
        db::select_broadcast_event(&conn, &self.event_id).map_err(event_registry_error)?;
        Ok(CommandOutcome::without_events(load_event_section(
            &conn,
            &self.broadcast,
        )))
    }
}

struct ReadEventTargets {
    broadcast: config::BroadcastCapabilities,
}
impl ApplicationCommand for ReadEventTargets {
    type Output = EventTargetListInput;
    fn execute(self, context: &CommandContext) -> crate::application::CommandResult<Self::Output> {
        if context.cancellation().is_cancelled() {
            return Err(CommandError::Cancelled);
        }
        Ok(CommandOutcome::without_events(read_targets(
            &self.broadcast,
        )))
    }
}

fn event_input_identity(input: &EventSectionInput) -> (&str, Option<&str>, i64) {
    (
        &input.context,
        input.registry.selected_id.as_deref(),
        input.registry.revision,
    )
}
