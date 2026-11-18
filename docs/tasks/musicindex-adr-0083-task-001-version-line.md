# musicindex ADR 0083 Task 001: The Version Line

Status: Implemented on 2026-11-18. The mechanical criteria pass. The visual criteria wait for a person, in section 35
of [pending human checks](../pending-human-checks.md). The orchestrator sent this task on 2026-11-18.

The first agent check of 2026-11-18 on Xvfb showed the line on each tab. It passed visual criteria 1 to 3 in Dark and
Light at 1440 px, and in Dark at 390 px. "Copy" put the exact line on the clipboard. A drag marked the text, and
Ctrl+C copied the marked part. The agent check of the About tab, also on 2026-11-18, passed in Dark at 1440 px and at 390 px. The tab row showed
four tabs. About showed the line and "Copy", and "Copy" put the line on the clipboard. General, Library and
Diagnostics showed no line. Light was not examined again.

## Operator Choice

2026-11-18: the line shows on one Settings tab only, and that tab is a new tab "About". This choice replaces the
first main session choice "one line at the end of each page". [ADR 0093](../adr/0093-settings-has-a-tab-row-and-one-save-bar.md)
Decision 1 and Decision 2 record the fourth group.

## Goal

HitIt shows its build values in the shared form of
[musicindex ADR 0083](../../../musicindex/docs/adr/0083-each-app-shows-its-version-in-one-form.md), Accepted on
2026-11-18. The build values are the version of `Cargo.toml`, the git revision and the build time in UTC
(R-VERSION-1). This task cites the rules. It does not write them again.

The rules that apply are R-VERSION-1, R-VERSION-2, R-VERSION-4, R-VERSION-6, R-VERSION-8 and R-VERSION-9.

## Orchestrator Choices

Each choice is from 2026-11-18.

1. The crate `musicindex-build-info` comes from musicindex-lib at rev
   `e6d11118d33069c26c01368f7de71524988039c6`. It is a dependency and a build dependency.
2. `build.rs` calls `musicindex_build_info::build::emit("HITIT")`.
3. The app name in the line is "HitIt". The code uses `build_info!("HitIt")`.
4. R-VERSION-3 does not apply. HitIt has no local server route. Each `TcpListener` in `src` is in a test.
5. R-VERSION-7: the version in `Cargo.toml` stays `0.1.0`. The operator selects the first release version and
   makes the tag.

## Main Session Choices

1. The About tab is the last tab, after Diagnostics. It has the heading "About", the line and "Copy". It has no
   Save bar, because it has no setting.
2. The line text is `version_line()`. It is the same text as `hitit --version`, so a bug report has one form.
3. `hitit --version` runs before the user data move of ADR 0096 Decision 6. A version request reads no user
   data and changes no user data.
4. "Copy" copies the line text. It uses the button of `settings_actions`. It does not use a new button.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Build values and the version line | reuse | Cargo.toml | The `musicindex-build-info` crate of musicindex-lib gives `build::emit`, `build_info!` and `BuildInfo::version_line` (R-VERSION-8). |
| Command line entry | extend | src/cli.rs | `run` gets the `--version` arm next to `--help`. |
| Settings view model | extend | src/view_models/settings.rs | `SettingsGroup::About` is the fourth group. `SettingsVm` gives the version line text, its Copy action and `shows_version`. |
| Settings render | extend | src/app/settings.rs | `render_settings` adds the line when `shows_version` is true. |
| Settings scroll handles | extend | src/ui/composites/settings.rs | `SettingsScrollHandles` gets a handle for About. |
| Selectable text | reuse | src/ui/composites/selectable_text.rs | `SelectableText` gives selection and the context menu Copy. |
| Settings row composites | reuse | src/ui/composites/settings.rs | `settings_value_and_action` and `settings_actions` give the line and the button. |
| Clipboard write | reuse | src/app/search_dispatch.rs | The pattern `cx.write_to_clipboard(ClipboardItem::new_string(...))`. |
| User-Agent | reuse | src/http_client.rs | `USER_AGENT` keeps `hitit/` and `CARGO_PKG_VERSION` (R-VERSION-6). No change. |

## Changes

1. `Cargo.toml`: `musicindex-build-info` in `[dependencies]` and in `[build-dependencies]`, both at the rev of
   choice 1. Add a comment that cites musicindex ADR 0083 R-VERSION-8. `Cargo.lock` changes with it.
2. `build.rs` (new, at the repository root): it calls `musicindex_build_info::build::emit("HITIT")`. Add a comment
   that cites musicindex ADR 0083 R-VERSION-8.
3. A small module, for example `src/build_values.rs`, gives one function that returns
   `musicindex_build_info::build_info!("HitIt")`. Each surface uses this function. Do not call the macro in more than
   one place.
4. `src/cli.rs`: `hitit --version` prints `version_line()` and ends with success (R-VERSION-2). `print_help` lists
   `--version`.
5. `src/view_models/settings.rs`: the view model gives the version line text and a Copy action. The action type
   follows the action pattern of the Settings view model.
6. `src/app/settings.rs`: the version line shows on the About tab only (operator choice). The text is a
   `SelectableText`. "Copy" writes the line to the clipboard (R-VERSION-4).
7. Tests (R-VERSION-9 item 1):
   - A test of the `--version` output: run the binary with `--version` (for example with
     `env!("CARGO_BIN_EXE_hitit")` in `tests/`). The line starts with `HitIt 0.1.0`. When the build has a git
     revision and a build time, the parentheses hold them.
   - A view model test: the Settings version line text equals the version line of the build values. It holds
     the version, the git revision and the build date when they are known.
   - A unit test with a fixed `BuildInfo::new("HitIt", "0.1.0", Some("a1b2c3d"), Some("2026-11-18T12:00:00Z"))`
     gives `HitIt 0.1.0 (a1b2c3d, 2026-11-18)` through the HitIt line function or the view model.

Do not change `src/http_client.rs`, `src/feed_service.rs` or `src/application/commands/metadata.rs`. Their
User-Agent keeps the form `hitit/0.1.0` (R-VERSION-6).

## Acceptance Criteria

Mechanical:

1. `cargo test --locked` passes, with the three tests of change 7.
2. `cargo clippy --all-targets --locked -- -D warnings` passes.
3. `cargo test --test architecture_tests` passes.
4. `target/debug/hitit --version` prints one line in the form of R-VERSION-2.
5. The Settings view model exposes the version line text and the Copy action. `shows_version` is true for About
   only, and the group control lists General, Library, Diagnostics and About.

Visual, for a person (R-VERSION-9 item 2):

1. The About tab of Settings shows the line, in Light and Dark, at 1440 px and at 390 px. The text does not overlap
   "Copy". The other tabs do not show the line.
2. A mouse drag selects the text of the line.
3. "Copy" puts the line on the clipboard.
