# ADR 0068 Task 004: The Audition Player

## Status

Ready after packets 001 and 003 - 2026-11-11. Implementation has not started.

## Goal

A second driver instance plays an audition on a monitor output. The audition owner has its own state and commands. It cannot write the Show session or the broadcast metadata.

## Authority

- [ADR 0068](../adr/0068-show-cue-and-audition-isolation.md), "Audition Is One Independent Track", "Independent Audio Outputs And Driver Instances", "Publication And Authoritative State", and Answers 2 and 3 of its [revision of 2026-11-11](../adr/0068-show-cue-and-audition-isolation.md#revision-for-the-adr-0089-shell---2026-11-11).
- [ADR 0069](../adr/0069-grouped-settings-and-selective-presets.md): Show and Audition have their own output. Its phase 005 adds the Settings controls later.
- [ADR 0064](../adr/archive/0064-local-file-addressing.md): an audition plays a validated local file.

## Recorded Facts - 2026-11-11

- The socket path is `runtime_dir.join(format!("mpv-{}.sock", std::process::id()))` (`src/playback_driver/mpv.rs:159`). `remove_stale_socket` refuses a socket that answers.
- mpv starts with `--idle`, `--no-video` and `--input-ipc-server` only.
- `PlaybackConfig` (`src/config.rs:101`) has `driver` and `mpv_path`. The table is `[playback]`.
- The polling actor (`src/runtime/playback_polling.rs`) and `SessionDrain` (`src/application/session_lifecycle.rs`) hold the Show owner.

## Files To Inspect

- `src/playback_driver/mod.rs`, `src/playback_driver/mpv.rs`, `src/playback_driver/null.rs`
- `src/playback_owner.rs`, `src/runtime/playback_polling.rs`, `src/application/session_lifecycle.rs`
- `src/config.rs`, `src/startup.rs` (`prepare_playback_driver`), `src/app/bootstrap.rs`
- `src/application/capability_recovery/setup.rs`

## Scope

1. The socket name includes the purpose (`show` or `audition`) and a nonce of the instance, after the process id. Each instance has its own child, connection, reply state and lock.
2. Configuration:
   - `[playback]` stays the Show player. An optional `audio_device` gives mpv `--audio-device`. With no value, Show keeps the behavior of today.
   - A new table `[audition]` has `driver`, `mpv_path` and `audio_device`. Audition needs `audio_device`. With no value, audition is not available, and the app gives the reason. Audition never uses the Show output or the system default.
   - An old configuration file stays valid and is not written again at startup.
3. `AuditionOwner` has the states idle, starting, playing, paused and failed, a position, and the commands Start, Pause, Resume, Seek and Stop. It holds no session id, no cue cursor, no producer and no publisher credential.
4. Answer 2: Start takes one track or the tracks of one album in their order. The end of a track goes to the next track of that album. At the last track the audition stops. A new Start replaces the audition.
5. Answer 3: a page change does not stop an audition. Stop, a new Start, the end, a failure and the end of the app stop it. A Show start does not stop it.
6. The audition has its own polling watch. A blocked audition command cannot stop the Show watch. A late reply from an earlier audition cannot change the current one.
7. A failure of one player leaves the other usable. A missing mpv binary can affect the two players, and each report names its player. A configured mpv failure never becomes the Null driver.
8. The two owners take part in the app shutdown and in `SessionDrain`. The audition does not resume after a restart or a recovery.
9. The view model gives the audition state, its labels and the availability of each command. This packet adds no control on a page. ADR 0089 packet 016 adds them.

## Do Not Touch

- The Settings page. ADR 0069 phase 005 owns the audio controls.
- The Show cue and the Show commands, apart from the shared driver code.
- The CLI.

## Mechanical Criteria

- M4 of ADR 0068: with recording fakes, Start, Replace, Seek, Pause, the end of a track, an error and Stop make zero Show writes and zero producer calls. A Show that plays at the same time continues, with its own metadata updates.
- M5 of ADR 0068: two instances have different children and sockets and independent replies. A timeout, a shutdown or an output failure of one cannot control or block the other.
- M6 of ADR 0068: an old configuration file reads with no change. An invalid audition output affects only the audition. No missing output falls back to a default, and no failure becomes the Null driver. A simulation with the Null driver has a label.
- M8 of ADR 0068, audition part: the two owners drain and release their resources. The audition does not resume.
- The commit checks of AGENTS.md pass.

## Visual Criteria

None in this packet. The [final gate](../plans/adr-0068-show-cue-and-audition-phase-plan.md#final-gate) listens to the two outputs after ADR 0089 packet 016.

## Test Commands

```bash
cargo fmt -- --check
cargo test --locked --offline --quiet
cargo test --test architecture_tests
cargo clippy --locked --offline --quiet --all-targets -- -D warnings
cargo build --locked --offline --quiet --bin hitit
git diff --check
```

## Report

Give the configuration keys, the files changed and the tests. Give the mpv arguments of each player.

## Stop And Ask

Stop if mpv needs a device name that the configuration cannot hold as one text value. ADR 0068 requires a device identity of the audio backend, not a position in a list.
