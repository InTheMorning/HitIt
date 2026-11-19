//! Top-level playback command binding.

use std::sync::Arc;

use gpui::Context;

use crate::application::capability::Dependency;
use crate::application::capability_recovery::{PlaybackOperation, RecoveryAction, RecoveryIntent};
use crate::application::commands::playback::{
    NextShowCueEntry, PauseShow, PlayTestSound, PreviousShowCueEntry, ResumeShow,
    SelectShowCueEntry, StartShow, StopShow,
};
use crate::application::{ApplicationCommand, CommandContext};
use crate::presentation::present_command;
use crate::view_models::queue_now_playing::{ShowToggleIntent, TransportState};

use super::TopApp;

impl TopApp {
    pub(super) fn stop_show(&mut self, cx: &mut Context<Self>) {
        if !self.show_page.queue.can_stop_show() {
            return;
        }
        let action = self.playback_action(PlaybackOperation::Stop);
        let Some(owner) = self.available_playback_owner(&action, cx) else {
            return;
        };
        let command = StopShow::new(Arc::clone(&self.conn), owner);
        self.run_playback_command(command, action, None, cx);
    }

    pub(super) fn select_show_cue_entry(&mut self, entry_id: i64, cx: &mut Context<Self>) {
        let command = SelectShowCueEntry::new(Arc::clone(&self.conn), entry_id);
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            |this, result, cx| {
                this.playback_presentation_override = None;
                result.message().clone_into(&mut this.settings_status);
                this.refresh_show_page(cx);
            },
            |this, error, cx| {
                this.settings_status = format!("Show selection error: {error:#}");
                this.refresh_show_page(cx);
            },
        );
    }

    pub(super) fn skip_playback_next(&mut self, cx: &mut Context<Self>) {
        if !self.show_page.queue.can_skip_next() {
            return;
        }
        let action = self.playback_action(PlaybackOperation::Next);
        let Some(owner) = self.available_playback_owner(&action, cx) else {
            return;
        };
        let command = NextShowCueEntry::new(Arc::clone(&self.conn), Arc::clone(&owner));
        self.run_playback_command(command, action, None, cx);
    }

    pub(super) fn skip_playback_previous(&mut self, cx: &mut Context<Self>) {
        if !self.show_page.queue.can_skip_previous() {
            return;
        }
        let action = self.playback_action(PlaybackOperation::Previous);
        let Some(owner) = self.available_playback_owner(&action, cx) else {
            return;
        };
        let command = PreviousShowCueEntry::new(Arc::clone(&self.conn), Arc::clone(&owner));
        self.run_playback_command(command, action, None, cx);
    }

    pub(super) fn toggle_playback_paused(&mut self, cx: &mut Context<Self>) {
        // ADR 0068: the view model decides the Show intent. No cue gives no intent.
        let Some((intent, _)) = self.show_page.queue.toggle_intent() else {
            return;
        };
        let action = self.playback_action(match intent {
            ShowToggleIntent::Start => PlaybackOperation::Start,
            ShowToggleIntent::Pause => PlaybackOperation::Pause,
            ShowToggleIntent::Resume => PlaybackOperation::Resume,
        });
        let Some(owner) = self.available_playback_owner(&action, cx) else {
            return;
        };
        let conn = Arc::clone(&self.conn);
        match intent {
            ShowToggleIntent::Start => {
                if self.refuse_show_play(&owner, cx).is_some() {
                    return;
                }
                let entry_id = self.show_page.queue.selected_entry_id();
                self.run_playback_command(StartShow::new(conn, owner, entry_id), action, None, cx);
            }
            ShowToggleIntent::Pause => {
                self.run_playback_command(PauseShow::new(conn, owner), action, None, cx);
            }
            ShowToggleIntent::Resume => {
                self.run_playback_command(ResumeShow::new(conn, owner), action, None, cx);
            }
        }
    }

    /// Refuses a Show Play when the player is the `null` driver (ADR 0104 Decision 2).
    /// The refusal comes before the "Starting" override, so that state does not occur.
    fn refuse_show_play(
        &mut self,
        owner: &Arc<
            std::sync::Mutex<
                crate::playback_owner::PlaybackOwner<
                    crate::playback_driver::ConfiguredPlaybackDriver,
                >,
            >,
        >,
        cx: &mut Context<Self>,
    ) -> Option<&'static str> {
        let driver_is_live = match owner.lock() {
            Ok(owner) => owner.driver().is_live_driver(),
            Err(_) => true,
        };
        let reason = crate::view_models::show::show_play_refusal(driver_is_live)?;
        self.playback_presentation_override = None;
        reason.clone_into(&mut self.settings_status);
        self.reproject_show_page_from_current_queue();
        cx.notify();
        Some(reason)
    }

    /// Plays the test sound through its own driver (ADR 0104 Decision 4).
    pub fn play_test_sound(&mut self, cx: &mut Context<Self>) {
        let state = self.show_page.queue.transport.play_pause_state;
        if let Err(reason) = self.test_sound.begin(state) {
            reason.clone_into(&mut self.settings_status);
            self.reproject_show_page_from_current_queue();
            cx.notify();
            return;
        }
        let config = match crate::config::ConfigSnapshot::read_existing(&self.cfg_path)
            .map_err(|error| format!("{error:#}"))
            .and_then(|snapshot| snapshot.playback().map_err(|error| format!("{error:?}")))
        {
            Ok(config) => config,
            Err(error) => {
                self.test_sound.fail();
                self.settings_status =
                    format!("App could not read the player settings for the test sound: {error}");
                cx.notify();
                return;
            }
        };
        let tone_directory = match crate::config::app_cache_dir() {
            Ok(directory) => directory,
            Err(error) => {
                self.test_sound.fail();
                self.settings_status = format!("Test sound error: {error:#}");
                cx.notify();
                return;
            }
        };
        let command = PlayTestSound::new(
            config,
            tone_directory,
            |config: &crate::config::PlaybackConfig| {
                crate::playback_driver::ConfiguredPlaybackDriver::from_config_isolated(
                    config,
                    "test-sound",
                )
            },
        );
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            |this, config, cx| {
                this.test_sound.record_played(config);
                "The player played the test sound.".clone_into(&mut this.settings_status);
                cx.notify();
            },
            |this, error, cx| {
                this.test_sound.fail();
                this.settings_status = format!("Test sound error: {error:#}");
                cx.notify();
            },
        );
    }

    /// Records the answer to "Did you hear the test sound?" (ADR 0104 Decision 4).
    pub fn answer_test_sound(&mut self, heard: bool, cx: &mut Context<Self>) {
        self.test_sound.answer(heard);
        cx.notify();
    }

    /// Gives the last test sound result for the current player.
    #[must_use]
    pub fn test_sound_outcome(&self) -> crate::view_models::audio_setup::TestSoundOutcome {
        self.test_sound.outcome()
    }

    fn playback_action(&self, operation: PlaybackOperation) -> RecoveryAction {
        RecoveryAction::Playback {
            operation,
            track_id: self.show_page.queue.current_track_id(),
            queue: self.show_page.queue.track_ids(),
        }
    }

    pub(super) fn retry_playback(&mut self, intent: RecoveryIntent, cx: &mut Context<Self>) {
        let Some(owner) = self.playback_owner.clone() else {
            self.capability_vm.pending.finish(
                intent.id,
                "App has no verified player. Check playback again.".into(),
            );
            return;
        };
        let RecoveryAction::Playback { operation, .. } = intent.action else {
            return;
        };
        match operation {
            PlaybackOperation::Pause => {
                self.run_retained_playback(PauseShow::new(self.conn.clone(), owner), intent, cx);
            }
            PlaybackOperation::Start => {
                if let Some(reason) = self.refuse_show_play(&owner, cx) {
                    self.capability_vm.pending.finish(intent.id, reason.into());
                    return;
                }
                self.run_retained_playback(
                    StartShow::new(
                        self.conn.clone(),
                        owner,
                        self.show_page.queue.selected_entry_id(),
                    ),
                    intent,
                    cx,
                );
            }
            PlaybackOperation::Stop => {
                self.run_retained_playback(StopShow::new(self.conn.clone(), owner), intent, cx);
            }
            PlaybackOperation::Resume => {
                self.run_retained_playback(ResumeShow::new(self.conn.clone(), owner), intent, cx);
            }
            PlaybackOperation::Next => self.run_retained_playback(
                NextShowCueEntry::new(self.conn.clone(), owner),
                intent,
                cx,
            ),
            PlaybackOperation::Previous => self.run_retained_playback(
                PreviousShowCueEntry::new(self.conn.clone(), owner),
                intent,
                cx,
            ),
        }
    }

    fn run_retained_playback<C>(&self, command: C, intent: RecoveryIntent, cx: &mut Context<Self>)
    where
        C: ApplicationCommand<
            Output = crate::application::commands::playback::PlaybackCommandResult,
        >,
    {
        let id = intent.id;
        let action = intent.action.clone();
        self.run_playback_command(self.retry_command(command, intent), action, Some(id), cx);
    }

    pub(super) fn playback_availability(
        &self,
    ) -> Result<(), crate::application::capability::ExecutionUnavailable> {
        self.feature_availability()
            .require(crate::application::capability::Dependency::Playback)
    }

    pub(super) fn feature_availability(
        &self,
    ) -> crate::application::capability::FeatureAvailability {
        crate::application::capability::FeatureAvailability::from_resources(
            &self.musicindex_endpoint,
            &self.broadcast,
            self.playback_owner.is_some(),
        )
        .with_runtime(self.command_runner.availability())
        .with_observations(&self.capability_observations.snapshot())
        .with_checking(self.capability_vm.running_dependency())
    }

    fn available_playback_owner(
        &mut self,
        action: &RecoveryAction,
        cx: &mut Context<Self>,
    ) -> Option<
        Arc<
            std::sync::Mutex<
                crate::playback_owner::PlaybackOwner<
                    crate::playback_driver::ConfiguredPlaybackDriver,
                >,
            >,
        >,
    > {
        if let Err(reason) = self.playback_availability() {
            self.retain_failed_action(action.clone(), reason.dependency, cx);
            self.settings_status = reason.to_string();
            self.reproject_show_page_from_current_queue();
            cx.notify();
            return None;
        }
        self.playback_owner.clone()
    }

    fn run_playback_command<C>(
        &self,
        command: C,
        action: RecoveryAction,
        retry_id: Option<u64>,
        cx: &mut Context<Self>,
    ) where
        C: ApplicationCommand<
            Output = crate::application::commands::playback::PlaybackCommandResult,
        >,
    {
        let is_start = matches!(
            action,
            RecoveryAction::Playback {
                operation: PlaybackOperation::Start,
                ..
            }
        );
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, result, cx| {
                this.playback_presentation_override = presentation_after_command(is_start, true);
                if let Some(id) = retry_id {
                    this.capability_vm
                        .pending
                        .succeed(id, result.message().to_owned());
                }
                this.maybe_start_playback_polling(cx);
                this.settings_status = result.message().to_string();
                this.refresh_show_page(cx);
                this.library.update(cx, |library, cx| {
                    library.reload_playlists(cx);
                });
            },
            move |this, error, cx| {
                let missing_mpv_path = match &error {
                    crate::application::CommandError::Playback(message) => message
                        .strip_prefix(crate::playback_driver::MPV_NOT_INSTALLED_PREFIX)
                        .map(std::path::PathBuf::from),
                    _ => None,
                };
                if let Some(path) = missing_mpv_path {
                    this.capability_observations.record(
                        crate::application::capability::CapabilityObservation::new(
                            Dependency::Playback,
                            Some(crate::application::capability::CapabilityFailure::PlaybackProgramMissing),
                        )
                        .at_path(&path),
                    );
                    this.capability_vm.observations = this.capability_observations.snapshot();
                }
                if let Some(id) = retry_id {
                    this.capability_vm.pending.finish(
                        id,
                        crate::diagnostics::redact_endpoint_details(&format!(
                            "App could not retry playback: {error}"
                        )),
                    );
                } else {
                    this.retain_failed_action(action, Dependency::Playback, cx);
                }
                if is_start {
                    this.playback_presentation_override = presentation_after_command(true, false);
                    this.reproject_show_page_from_current_queue();
                }
                this.settings_status = format!("Playback error: {error:#}");
                this.library.update(cx, |library, cx| {
                    library.reload_playlists(cx);
                });
            },
        );
    }
}

/// Gives the Show state that the page shows after a playback command (ADR 0068).
/// An accepted Start is `Starting` until an observation of the driver proves playback.
/// A Start that failed is `Failed`. Each other command shows the stored session.
const fn presentation_after_command(is_start: bool, accepted: bool) -> Option<TransportState> {
    match (is_start, accepted) {
        (true, true) => Some(TransportState::Starting),
        (true, false) => Some(TransportState::Failed),
        (false, _) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_accepted_start_is_starting_and_not_playing() {
        assert_eq!(
            presentation_after_command(true, true),
            Some(TransportState::Starting)
        );
        assert_eq!(
            presentation_after_command(true, false),
            Some(TransportState::Failed)
        );
        assert_eq!(presentation_after_command(false, true), None);
    }
}
