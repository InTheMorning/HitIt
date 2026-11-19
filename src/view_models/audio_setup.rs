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
/// The help of ADR 0104 Decision 3 item 4 when mpv is not installed.
pub(crate) const MPV_MISSING_HELP: &str = "mpv is a free program that plays audio. HitIt needs it for sound. Install mpv with the software tool of your system, then select Check again.";

/// The help of ADR 0104 Decision 4 item 4 after the answer "No".
pub(crate) const NOT_HEARD_HELP: &str =
    "Check the volume, the system output device and the mpv state.";

/// The title of the group "Audio" at the first start (ADR 0104 Decision 5).
pub(crate) const FIRST_START_TITLE: &str = "Set up sound";

/// The text of the group "Audio" at the first start (ADR 0104 Decision 5).
pub(crate) const FIRST_START_TEXT: &str =
    "HitIt plays your show through a player program on this computer.";

/// The state of mpv in the group "Audio" (ADR 0104 Decision 3 item 3).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum MpvStatus {
    /// The app did not check mpv yet.
    #[default]
    Unchecked,
    /// `ping` gave this version.
    Installed(String),
    /// The system has no mpv program at the configured path.
    Missing,
    /// mpv did not start or did not answer `ping`. The text is the error.
    DidNotStart(String),
}

impl MpvStatus {
    /// The state from the result of `ping`. An error that starts with
    /// `MPV_NOT_INSTALLED_PREFIX` is a missing program. Each other error is a
    /// failed start, so that the row does not name a wrong cause.
    pub(crate) fn from_ping(result: Result<Option<String>, String>) -> Self {
        match result {
            Ok(Some(version)) => Self::Installed(version),
            Ok(None) => Self::Missing,
            Err(error) if error.contains(crate::playback_driver::MPV_NOT_INSTALLED_PREFIX) => {
                Self::Missing
            }
            Err(error) => Self::DidNotStart(error),
        }
    }

    /// The text of the state row.
    pub(crate) fn label(&self) -> String {
        match self {
            Self::Unchecked => "The app checks mpv.".into(),
            Self::Installed(version) => format!("mpv is installed: {version}"),
            Self::Missing => format!("mpv is not installed. {MPV_MISSING_HELP}"),
            Self::DidNotStart(error) => format!("mpv did not start: {error}"),
        }
    }

    /// True when the row shows "Check again".
    pub(crate) const fn offers_check_again(&self) -> bool {
        matches!(self, Self::Missing | Self::DidNotStart(_))
    }
}

/// The page that the app opens after the start (ADR 0104 Decision 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StartPage {
    /// The usual start page.
    Music,
    /// Settings on the group "Audio", with the first start text.
    SetUpSound,
}

impl StartPage {
    /// The page for a start. Only a start that wrote the configuration file opens "Set up sound".
    pub(crate) const fn for_start(created_defaults: bool) -> Self {
        if created_defaults {
            Self::SetUpSound
        } else {
            Self::Music
        }
    }
}

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

/// What the test sound row of the group "Audio" shows (ADR 0104 Decision 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TestSoundDisplay {
    /// Only the button "Play test sound".
    Play,
    /// The question "Did you hear the test sound?" with "Yes" and "No".
    Question,
    /// The result text, then the button "Play test sound".
    ResultAndPlay(&'static str),
}

impl TestSoundOutcome {
    /// Gives what the test sound row shows for this result.
    #[must_use]
    pub(crate) const fn display(self) -> TestSoundDisplay {
        match self {
            Self::NotTested => TestSoundDisplay::Play,
            Self::Played => TestSoundDisplay::Question,
            Self::Heard => TestSoundDisplay::ResultAndPlay("You heard the test sound."),
            Self::NotHeard => TestSoundDisplay::ResultAndPlay(NOT_HEARD_HELP),
        }
    }
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

    /// A defect of 2026-11-19: after "Yes" the row showed the question again.
    #[test]
    fn an_answer_shows_the_result_and_the_play_button_again() {
        assert_eq!(
            TestSoundOutcome::NotTested.display(),
            TestSoundDisplay::Play
        );
        assert_eq!(
            TestSoundOutcome::Played.display(),
            TestSoundDisplay::Question
        );
        assert_eq!(
            TestSoundOutcome::Heard.display(),
            TestSoundDisplay::ResultAndPlay("You heard the test sound.")
        );
        assert_eq!(
            TestSoundOutcome::NotHeard.display(),
            TestSoundDisplay::ResultAndPlay(NOT_HEARD_HELP)
        );
    }

    #[test]
    fn failed_mpv_ping_gives_the_missing_state_and_help() {
        let status = MpvStatus::from_ping(Err(format!(
            "{}/usr/bin/mpv",
            crate::playback_driver::MPV_NOT_INSTALLED_PREFIX
        )));
        assert_eq!(status, MpvStatus::Missing);
        assert!(status.label().starts_with("mpv is not installed. "));
        assert!(status.label().contains(MPV_MISSING_HELP));
        assert!(status.offers_check_again());
        let found = MpvStatus::from_ping(Ok(Some("mpv 0.40.0".into())));
        assert_eq!(found.label(), "mpv is installed: mpv 0.40.0");
        assert!(!found.offers_check_again());
    }

    #[test]
    fn a_failed_mpv_start_does_not_say_that_mpv_is_not_installed() {
        let status = MpvStatus::from_ping(Err("connect mpv IPC socket before timeout".into()));
        assert_eq!(
            status.label(),
            "mpv did not start: connect mpv IPC socket before timeout"
        );
        assert!(!status.label().contains("not installed"));
        assert!(status.offers_check_again());
    }

    #[test]
    fn only_a_start_that_wrote_the_configuration_opens_set_up_sound() {
        assert_eq!(StartPage::for_start(true), StartPage::SetUpSound);
        assert_eq!(StartPage::for_start(false), StartPage::Music);
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
