# ADR 0104 Task 002: The Group "Audio" And The First Start

## Status

Implemented - 2026-11-19. The mechanical criteria pass. The visual criteria wait for a person, in section 44 of the pending human checks.

## Goal

Settings has the group "Audio" with the Show player, the state of mpv and the test sound. The first start opens it.

## Authority

[ADR 0104](../adr/0104-the-show-player-and-the-sound-setup.md) Decisions 3, 4 and 5. [ADR 0093](../adr/0093-settings-has-a-tab-row-and-one-save-bar.md) Decision 2 gives the position of the group.

## Change

1. `src/view_models/settings.rs`: add the group "Audio" between General and Library.
2. The group has one choice row with "Speakers of this computer (mpv)" and "No sound", with the descriptions of ADR 0104. The choice saves `playback.driver`. When ADR 0069 task 002 delivers the draft, the row joins it.

   Main session choice, 2026-11-19: the General group has no save path, because the theme choice changes only the window. A click on a player item saves at once through the configuration correction of `src/config/correction.rs`, with only `playback.driver` changed. The save then runs the same capability check as a save from the configuration editor (`CorrectionEvent::Saved` in `src/app/startup.rs`). Thus the Show driver is made again, and the test result is cleared. A failed save shows its error on the row and keeps the old choice.
3. Below the choice, a row shows "mpv is installed" with the version from `ping`, or "mpv is not installed" with the text of ADR 0104 Decision 3 item 4 and "Check again".
4. A row with "Play test sound". After the tone, the row asks "Did you hear the test sound?" with "Yes" and "No". "No" shows the checks of ADR 0104 Decision 4 item 4.
5. When `created_defaults()` is true at the start (`src/startup.rs:213`), the app opens Settings on "Audio" with the title "Set up sound", its text and "Continue". "Continue" opens Music. The first start text shows only in that run.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Settings view model | extend | src/view_models/settings.rs | It gets the group and its rows. |
| Settings render | extend | src/app/settings.rs | It renders the rows of the new group. |
| Settings row composites | reuse | src/ui/composites/settings_row.rs | Each row of the group is a settings row. |
| Configuration correction | reuse | src/config/correction.rs | It already edits `playback.driver`. |
| Start destination | extend | src/app.rs | The start page is Music Releases (`src/app.rs:543`). The first start selects Settings, Audio. |

## Mechanical Criteria

1. The Settings view model gives the groups General, Audio, Library and Diagnostics in this order.
2. The "Audio" choice row gives the two labels and descriptions of ADR 0104, and no typed value.
3. A test: with a fake `ping` failure, the view model gives "mpv is not installed", the help text and "Check again".
4. A test: after the test sound, the view model gives the question and the two answers. "No" gives the checks.
5. A test: a start with `created_defaults()` true selects Settings, Audio with the title "Set up sound". A later start selects Music.
6. The fit check of ADR 0097 passes for the group at 390 px and at 940 px.
7. The commit checks of AGENTS.md pass.

## Visual Criteria

These are in section 44 of [pending human checks](../pending-human-checks.md).

- V1: the group "Audio" shows the two player items, the mpv state and "Play test sound", in Light and Dark.
- V2: "Play test sound" plays a tone of 2 s that the person hears. Then the question shows.
- V3: a start with no configuration file opens "Set up sound". "Continue" opens Music.

## Report

Changed files:

- `src/view_models/settings.rs`: added Audio and the two player choices.
- `src/app/settings.rs`: added the Audio rows, player save, mpv check and test sound answers.
- `src/app.rs`: retained the selected player and first start state.
- `src/app/bootstrap.rs` and `src/startup.rs`: opened Audio after default configuration creation.
- `src/ui/composites/settings.rs`: added the Audio scroll handle.
- `src/app/fit_check/mod.rs`: added the Audio fit check at 390 px and 940 px.
- `src/playback_driver/mpv.rs` and `src/playback_driver/mod.rs`: returned the mpv version from ping.
- `src/view_models/audio_setup.rs`: added the failed ping display rule and its test.
- `docs/pending-human-checks.md`: kept the visual gate open and updated its status.
- This task file: recorded the result and checks.

Mechanical criteria:

1. The Settings view model test checks the group sequence.
2. The player row test checks its labels and descriptions. It checks that no typed value appears.
3. `failed_mpv_ping_gives_the_missing_state_and_help` checks that a failed `ping` gives `MpvStatus::Missing`, the help text and "Check again".
4. `an_answer_needs_a_played_test_sound` checks the question and each answer. The Audio row shows `NOT_HEARD_HELP` after "No".
5. `only_a_start_that_wrote_the_configuration_opens_set_up_sound` checks `StartPage::for_start`. The bootstrap uses this function. The title and the text show only on the group "Audio".
6. `adr_0097_audio_settings_fit_at_the_two_widths` checks the Audio group at 390 px and 940 px.
7. `cargo build` and `cargo test` passed. `cargo clippy --all-targets -- -D warnings` passed.
8. `cargo fmt -- --check` passed. `cargo test --test architecture_tests` passed.

Main session review, 2026-11-19:

- The state of mpv is the typed `MpvStatus`, not a compared string.
- The first start title and text show only on the group "Audio".
- A save that cannot start also shows its error on the player row.
- The row help shows the description of "No sound" from ADR 0104 Decision 2 item 1.

Open point: a save of the player while the Show plays makes the Show driver again through the capability check. The configuration editor has the same behavior. No decision covers it.

The visual checks remain open in section 44 of `docs/pending-human-checks.md`.
