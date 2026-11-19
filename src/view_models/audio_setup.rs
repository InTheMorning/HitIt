//! The state of the test sound of the Show player (ADR 0104 Decision 4).
//!
//! The app keeps the last result for the current player configuration.
//! A change of `playback.driver` or `playback.mpv_path` clears the result.

use crate::config::PlaybackConfig;
use crate::view_models::queue_now_playing::TransportState;

/// The reason that a Show Play and the test sound give with the `null` driver.
pub(crate) const NO_SOUND_REASON: &str =
    "No sound is selected. Select a player in Settings, Audio.";

/// The reason that the test sound gives while the Show plays or is paused.
pub(crate) const TEST_SOUND_SHOW_ACTIVE_REASON: &str =
    "The test sound is not available while the Show plays or is paused. Stop the Show first.";

/// The reason that the test sound gives while an earlier test sound runs.
pub(crate) const TEST_SOUND_RUNNING_REASON: &str = "The test sound is already playing.";

/// The last result of the test sound for the current player.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TestSoundOutcome {
    /// The host did not play the test sound with this player.
    #[default]
    NotTested,
    /// The player played the test sound. The host did not answer yet.
    Played,
    /// The host heard the test sound.
    Heard,
    /// The host did not hear the test sound.
    NotHeard,
}

/// The test sound result with the player configuration that it belongs to.
#[derive(Clone, Debug, Default)]
pub(crate) struct TestSoundState {
    config: Option<PlaybackConfig>,
    outcome: TestSoundOutcome,
    running: bool,
}

impl TestSoundState {
    /// Gives the last result for the current player.
    #[must_use]
    pub(crate) const fn outcome(&self) -> TestSoundOutcome {
        self.outcome
    }

    /// Starts a test sound. It gives the refusal reason while the Show is active
    /// or while another test sound runs.
    pub(crate) fn begin(&mut self, show: TransportState) -> Result<(), &'static str> {
        if show.is_active() {
            return Err(TEST_SOUND_SHOW_ACTIVE_REASON);
        }
        if self.running {
            return Err(TEST_SOUND_RUNNING_REASON);
        }
        self.running = true;
        Ok(())
    }

    /// Records that the test sound ended with an error.
    pub(crate) fn fail(&mut self) {
        self.running = false;
    }

    /// Records that the player played the test sound.
    pub(crate) fn record_played(&mut self, config: PlaybackConfig) {
        self.running = false;
        self.config = Some(config);
        self.outcome = TestSoundOutcome::Played;
    }

    /// Records the answer of the host to "Did you hear the test sound?".
    pub(crate) fn answer(&mut self, heard: bool) {
        if self.outcome != TestSoundOutcome::NotTested {
            self.outcome = if heard {
                TestSoundOutcome::Heard
            } else {
                TestSoundOutcome::NotHeard
            };
        }
    }

    /// Clears the result when the player configuration is not the one that was tested.
    pub(crate) fn reconcile(&mut self, current: &PlaybackConfig) {
        if self.config.as_ref() != Some(current) {
            self.config = None;
            self.outcome = TestSoundOutcome::NotTested;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::config::PlaybackDriver;

    fn mpv(path: Option<&str>) -> PlaybackConfig {
        PlaybackConfig {
            driver: PlaybackDriver::Mpv,
            mpv_path: path.map(PathBuf::from),
        }
    }

    #[test]
    fn test_sound_is_refused_while_the_show_plays_or_is_paused() {
        for state in [
            TransportState::Starting,
            TransportState::Playing,
            TransportState::Paused,
        ] {
            let mut sound = TestSoundState::default();
            assert_eq!(sound.begin(state), Err(TEST_SOUND_SHOW_ACTIVE_REASON));
            assert!(!sound.running);
        }
        for state in [
            TransportState::Empty,
            TransportState::Stopped,
            TransportState::Failed,
        ] {
            assert_eq!(TestSoundState::default().begin(state), Ok(()));
        }
    }

    #[test]
    fn test_sound_is_refused_while_another_test_sound_runs() {
        let mut sound = TestSoundState::default();
        sound.begin(TransportState::Stopped).unwrap();
        assert_eq!(
            sound.begin(TransportState::Stopped),
            Err(TEST_SOUND_RUNNING_REASON)
        );
        sound.fail();
        assert_eq!(sound.begin(TransportState::Stopped), Ok(()));
    }

    #[test]
    fn a_player_change_clears_the_test_result() {
        let mut sound = TestSoundState::default();
        sound.begin(TransportState::Stopped).unwrap();
        sound.record_played(mpv(None));
        sound.answer(true);
        sound.reconcile(&mpv(None));
        assert_eq!(sound.outcome(), TestSoundOutcome::Heard);

        sound.reconcile(&mpv(Some("/opt/mpv")));
        assert_eq!(sound.outcome(), TestSoundOutcome::NotTested);
        sound.reconcile(&mpv(None));
        assert_eq!(sound.outcome(), TestSoundOutcome::NotTested);

        sound.record_played(mpv(None));
        sound.reconcile(&PlaybackConfig {
            driver: PlaybackDriver::Null,
            mpv_path: None,
        });
        assert_eq!(sound.outcome(), TestSoundOutcome::NotTested);
    }

    #[test]
    fn an_answer_needs_a_played_test_sound() {
        let mut sound = TestSoundState::default();
        sound.answer(true);
        assert_eq!(sound.outcome(), TestSoundOutcome::NotTested);
        sound.record_played(mpv(None));
        assert_eq!(sound.outcome(), TestSoundOutcome::Played);
        sound.answer(false);
        assert_eq!(sound.outcome(), TestSoundOutcome::NotHeard);
    }
}
