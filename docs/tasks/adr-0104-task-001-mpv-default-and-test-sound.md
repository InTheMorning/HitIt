# ADR 0104 Task 001: The mpv Default And The Test Sound

## Status

Ready - 2026-11-19. It waits for no other task.

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

Empty until the work is done.
