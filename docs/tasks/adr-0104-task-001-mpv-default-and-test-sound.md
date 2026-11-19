# ADR 0104 Task 001: The mpv Default And The Test Sound

## Status

Implemented - 2026-11-19. The mechanical criteria pass. The visual criteria have no section, because task 002 shows the rows.

## Goal

The default Show player is mpv. A Show Play with the `null` driver does not start and gives a reason. HitIt can play a test sound through the Show player and keep the result.

## Authority

[ADR 0104](../adr/0104-the-show-player-and-the-sound-setup.md) Decisions 1, 2 and 4.

## Change

1. `src/config.rs`: when `playback.driver` is absent, the driver is `PlaybackDriver::Mpv`. The commented template of the first start says that the default is `mpv`. A file with `driver = "null"` keeps `null`.
2. The Show Play command refuses the `null` driver before it sets "Starting" (`src/app/playback_bar.rs:286`). The Show view model gives the reason "No sound is selected. Select a player in Settings, Audio." The CLI does not change.
3. Move the tone writer of `src/startup/fixture.rs:324` to a live module below `src/playback_driver/`. It writes a WAV sine tone of 440 Hz, 2 s long, at -18 dBFS, to the cache folder of the app. The fixture uses the same writer with its own length and wave.
4. Add a test sound command. It makes its own driver instance for the selected player and loads the tone. It waits for EOF or for 5 s, then it stops the instance. It does not use the Show playback owner, the session or the drop file. It is refused while the Show plays or is paused.
5. Keep the last test result in app state: not tested, played, heard or not heard, for the current player configuration. A change of `playback.driver` or `playback.mpv_path` clears it. Expose it to view models.
6. Examine if the app makes a new Show driver after a player change while the Show is stopped. Record the result in the Report. If it does not, make the change take effect at the next Show Play. Stop and report if this needs a change to the playback owner design.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Playback configuration | extend | src/config.rs | The default of `PlaybackConfig` changes. |
| Playback driver trait | reuse | src/playback_driver/mod.rs | The test sound uses `PlaybackDriver` with its own instance. |
| Tone writer | extend | src/startup/fixture.rs | The debug fixture has the only tone writer. It moves to a live module, and the fixture calls it. |
| Idle driver preparation | reuse | src/application/capability_recovery/setup.rs | "Check player" already prepares an idle driver. The test sound uses the same preparation. |
| Show Play admission | extend | src/app/playback_bar.rs | The `null` refusal comes before the "Starting" override. |

## Mechanical Criteria

1. A configuration with no `[playback]` table gives `PlaybackDriver::Mpv`. A configuration with `driver = "null"` gives `PlaybackDriver::Null`.
2. A test: a Show Play with the `null` driver gives the refusal reason, and the Show state is not "Starting".
3. A test with a fake driver: the test sound loads one file, stops its instance, and does not change the Show session row or the drop file.
4. A test: the test sound is refused while the Show session is playing or paused.
5. A test: a change of the player clears the test result.
6. The written WAV has 2 s of 440 Hz at the stated level (a test reads the header and the samples).
7. The commit checks of AGENTS.md pass.

## Visual Criteria

None. Task 002 shows the rows.

## Report

Changed files:

- `src/config.rs`: the default driver is `Mpv`. The template says so. New `app_cache_dir()`.
- `src/playback_driver/tone.rs` (new): the tone writer.
- `src/playback_driver/test_sound.rs` (new): plays one file through a driver, then stops it.
- `src/playback_driver/mod.rs` and `src/playback_driver/mpv.rs`: a driver with its own runtime folder.
- `src/application/commands/playback.rs`: the command `PlayTestSound`.
- `src/view_models/audio_setup.rs` (new, `src/view_models/mod.rs`): the test result state and the refusal reasons.
- `src/view_models/show.rs`: `show_play_refusal`.
- `src/app.rs`, `src/app/playback_bar.rs`, `src/app/capabilities.rs`: the Show Play refusal, the test sound methods and the state.
- `src/startup/fixture.rs`: the fixture uses the tone writer with a square wave.
- Tests changed for the new default: `src/startup.rs`, `src/app/bootstrap.rs`, `src/application/capability_recovery/setup.rs`.

Tests, by mechanical criterion:

1. `config::tests::load_config_defaults_missing_playback_to_mpv_driver`, `config::tests::load_config_keeps_an_explicit_null_driver`.
2. `view_models::show::tests::show_play_with_the_null_driver_gives_the_no_sound_reason`. The test covers the reason at the view model. `refuse_show_play` returns before `run_playback_command`, the only code that sets "Starting". No test covers the app method. The app tests need a window.
3. `application::commands::playback::tests::test_sound_leaves_the_show_session_and_the_drop_file_unchanged`, with `playback_driver::test_sound::tests::test_sound_loads_one_file_and_stops_its_instance`. The command has no database handle and no Show owner.
4. `view_models::audio_setup::tests::test_sound_is_refused_while_the_show_plays_or_is_paused`. It also refuses "Starting".
5. `view_models::audio_setup::tests::a_player_change_clears_the_test_result`. `install_checked_capability` calls `reconcile` after each Playback check.
6. `playback_driver::tone::tests::test_tone_is_two_seconds_of_440_hz_at_minus_18_dbfs`, `square_wave_keeps_the_fixture_shape`.

Other tests: the null command refusal, the refusal while a test runs, the timeout and load error paths of `play`, the answer rule.

Item 6, driver rebuild after a player change: the app already does it. A saved change of `playback.driver` or `playback.mpv_path` runs `check_saved_capabilities`. The Playback check calls `replace_idle_driver` when the owner has no loaded track. No change to the playback owner was necessary.

Decisions in the work:

- The test sound uses `ConfiguredPlaybackDriver::from_config_isolated` and not `prepare_playback_driver`. The mpv socket name uses the process ID. A Show mpv that stays alive after Stop would block a second driver in the same folder. The test sound has its own runtime folder `test-sound`.
- The tone goes to `<app cache>/test-sound.wav`, not into the `audio` folder.
- `play_test_sound`, `answer_test_sound` and `test_sound_outcome` are `pub` methods of `TopApp`. They have no caller yet. The ADR 0060 guard forbids `dead_code` attributes. Task 002 binds them.
- The Show page shows the refusal reason as its status message. The link to Settings, Audio waits for task 002.

Checks: `cargo build` and `cargo test` Green (lib 3318 passed). `cargo clippy --all-targets -- -D warnings` Green. `cargo fmt -- --check` Green. `cargo test --test architecture_tests` Green (417 passed).
