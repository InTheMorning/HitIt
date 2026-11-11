# ADR 0068 Task 005: The CLI Writer Lease

## Status

Ready after packet 003 - 2026-11-11. Implementation has not started.

## Goal

While the desktop app owns the Show session, a CLI command that changes the session fails with a clear message. A read and a simulation on a different database continue to work.

## Authority

- [ADR 0068](../adr/0068-show-cue-and-audition-isolation.md), "Persistence, Restart And CLI Compatibility".
- [ADR 0020](../adr/0020-simulated-playlist-playback.md) and [ADR 0021](../adr/0021-mpv-playback-driver.md): the CLI stays a session simulator. Its JSON contracts stay.

## Recorded Facts - 2026-11-11

- These CLI commands change the session with `DEFAULT_SESSION_ID`: `playlist play`, `playback set-track`, `playback position`, `playback next`, `playback previous`, `playback pause`, `playback resume` and `playback stop` (`src/cli.rs`).
- `now-playing --json` only reads the session.
- No lease or lock exists between the desktop app and the CLI.

## Files To Inspect

- `src/cli.rs`, `src/playback.rs`
- `src/app/bootstrap.rs`, `src/startup.rs`, `src/application/session_lifecycle.rs`

## Scope

1. The desktop Show owner takes a writer lease for its database and session. The operating system releases the lease when the process ends, also after a crash. A stored boolean is not a lease.
2. Each CLI command that changes the session takes the same lease first. If the desktop holds it, the command fails with a message that names the cause and the fix, and changes nothing.
3. `now-playing --json` needs no lease.
4. A CLI command on a different database, for example a fixture database, works with no desktop app.

## Do Not Touch

- The JSON output of each CLI command.
- The players and the cue.

## Mechanical Criteria

- M9 of ADR 0068: a simulation works alone. A command that changes the same session fails while the desktop holds the lease. `now-playing --json` works. The end of the process, also a forced end, releases the lease.
- The commit checks of AGENTS.md pass.

## Visual Criteria

None.

## Test Commands

```bash
cargo fmt -- --check
cargo test --locked --offline --quiet
cargo clippy --locked --offline --quiet --all-targets -- -D warnings
cargo build --locked --offline --quiet --bin v4vmm
git diff --check
```

## Report

Give the lease method, the files changed and the tests. Tell how a test proves the release after a forced end.

## Stop And Ask

Stop if the lease method does not work on macOS. ADR 0067 keeps macOS a supported desktop.
