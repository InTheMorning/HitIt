# ADR 0104 Task 002: The Group "Audio" And The First Start

## Status

Ready - 2026-11-19. It waits for [task 001](adr-0104-task-001-mpv-default-and-test-sound.md).

## Goal

Settings has the group "Audio" with the Show player, the state of mpv and the test sound. The first start opens it.

## Authority

[ADR 0104](../adr/0104-the-show-player-and-the-sound-setup.md) Decisions 3, 4 and 5. [ADR 0093](../adr/0093-settings-has-a-tab-row-and-one-save-bar.md) Decision 2 gives the position of the group.

## Change

1. `src/view_models/settings.rs`: add the group "Audio" between General and Library.
2. The group has one choice row with "Speakers of this computer (mpv)" and "No sound", with the descriptions of ADR 0104. The choice saves `playback.driver`. It uses the save path of the General group at the time of the build. When ADR 0069 task 002 delivers the draft, the row joins it.
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

Empty until the work is done.
