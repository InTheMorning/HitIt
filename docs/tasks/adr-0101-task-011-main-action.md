# ADR 0101 Task 011: The Main Action

Status: Waits - 2026-11-15. This packet starts after [task 002](adr-0101-task-002-header-glow.md), because R-ACT-7 applies to a page header with a glow.

## Goal

A page header fills no button for the next curation step. A track page fills only its audio action, and it has none yet. A release page fills nothing. A download and an add to a playlist are buttons of the role Secondary. Each button and menu item uses title case.

## Authority

- [ADR 0101](../adr/0101-the-design-language-follows-the-shared-rules.md) Decision 5 owns this packet. Read Decision 5 fully before the first edit. Its first row and its first item:
  - Main action: musicindex ADR 0045 R-ACT-7. A track page fills its audio action. A release page fills nothing until it has a main action of its own. The header button has the text color, not the accent.
  - A download action is a button of the role Secondary. Until a track page has an audio action, it has no primary button.
  - Buttons and menu items use title case (musicindex ADR 0023 R-CASE-1), for example "Add to Playlist" and "Download Album".
- musicindex ADR 0045 R-ACT-7 (`../musicindex/docs/adr/0045-the-primary-action-the-live-action-and-the-light-page.md` lines 64 to 72, at musicindex `785ae28`):
  1. In a page header with a glow, a Filled button has the fill `color.text` and the label `color.background`.
  2. Each other Filled button keeps the accent. A form and a sheet are examples.
  3. This button counts toward the limit of R-ACT-1 item 2.
  4. A track page fills its audio action. A release page fills nothing until it has a main action of its own.
- musicindex ADR 0028 R-PAGE-2 items 2 and 3 (`../musicindex/docs/adr/0028-cross-app-item-pages.md` lines 57 to 73): the primary action is a text button. "Open API JSON", the copy actions and "Inspect Sources" are in the "…" menu.
- musicindex ADR 0042 R-ACT-4 item 3: the role of a command follows its result (`ControlStyle::by_result`).
- The `rust-dev` skill applies to each Rust change.

## Recorded Facts

Each fact is from HitIt `ad1fcd9`, read on 2026-11-15. A ChatGPT search found them, and the main session examined the items marked "examined".

- `TrackDetail::page_actions` (`src/view_models/track_detail.rs:593`) gives one `filled` action in each state: DownloadAlbum for an Index track, DownloadTrack for a Library track with no audio, and AddToPlaylist for a downloaded track. Examined.
- `render_track_page_actions` (`src/ui/shells/track_page.rs:106`) draws `filled` with `ControlStyle::Primary` and each `plain` action with `ControlStyle::Neutral`. Examined.
- `AlbumPageAction::label` (`src/view_models/album_page.rs:36`) gives "Download {kind}", "Add to playlist", "Copy feed URL", "MusicBrainz lookup" and "Remove {kind}…". Examined. These are not in title case.
- `album_page_actions` (`src/view_models/album_page.rs:114`) gives DownloadAlbum as `filled` for an Index release. Examined.
- `src/ui/shells/release_page.rs:231` draws the release action with `ControlStyle::Primary`. Examined.
- `TrackPageAction::label` (`src/view_models/track_detail.rs:893`) gives "Download release" when no kind is known.
- `ControlStyle::by_result` (`src/ui/control_styles.rs:141`) gives Secondary for a command that changes stored data, else Neutral. Examined.
- This is an inference: no header has a Play or an Audition action. ADR 0068 task 004, the audition player, is Ready.
- Tests: `adr_0083_r83_42_index_track_actions` (`src/view_models/track_detail.rs:1627`) and `adr_0083_album_page_actions_follow_the_state_table` (`src/view_models/album_page.rs:218`).

Main session choice, 2026-11-15: a page header has no filled button after this packet. The style of R-ACT-7 item 1 comes with its first user, the audition button of ADR 0068 task 004. Its label is open point 1 of the [phase plan](../plans/adr-0101-design-language-phase-plan.md#open-points). Recommendation: "Audition".

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Track page actions | extend | `src/view_models/track_detail.rs` | `page_actions` gives the header actions of a track. Make `filled` optional there. |
| Release page actions | extend | `src/view_models/album_page.rs` | `album_page_actions` gives the header actions of a release. Make `filled` optional there. |
| Track header render | extend | `src/ui/shells/track_page.rs` | It draws the header actions of a track. |
| Release header render | extend | `src/ui/shells/release_page.rs` | It draws the header actions of a release. |
| Command role | reuse | `src/ui/control_styles.rs` | `by_result` gives the role of each command. |
| Audition packet | extend | `docs/tasks/adr-0068-task-004-audition-player.md` | It builds the first audio action. Add the R-ACT-7 item 1 style to it. |

## Required Changes

1. **No filled next step.** `filled` becomes optional in the track and release actions. No state gives a filled action now.
2. **The roles.** Each header button gets its role from `by_result`. A download and an add to a playlist are Secondary. At most two buttons show in the header. Each other action is in the "…" menu.
3. **The order.** Download comes first when the item is not downloaded. "Add to Playlist" comes first when it is.
4. **The words.** Each label of a header button and a menu item uses title case, for example "Add to Playlist", "Download Single", "Copy Feed URL", "MusicBrainz Lookup", "Remove Single…" and "Download Release". Find each other label of a page header and a row menu, and change it. Keep each accessibility label as a sentence.
5. **The audition packet.** Add one required change to ADR 0068 task 004: in a page header with a glow, the audio action is a Filled button with the fill `color.text` and the label `color.background` (R-ACT-7 item 1). Add its test to the acceptance criteria of that packet.
6. **The tests.** Change the ADR 0083 action tests so that they prove the new roles and the absence of a filled action. Rename them to cite ADR 0101 Decision 5.

## Do Not Change

- No change of the menu of a destructive action (ADR 0101 Decision 5, R-ACT-8 to R-ACT-12).
- No change of a filled button in a form or a sheet.
- No audio action. ADR 0068 task 004 adds it.
- No `git commit`, `git add` or `git stash`.

## Acceptance Criteria

Mechanical:

- M1: a test proves that the track actions give no filled action in each state.
- M2: a test proves that the release actions give no filled action for an Index release and a Library release.
- M3: a test proves that DownloadTrack, DownloadAlbum and AddToPlaylist have the role Secondary.
- M4: a test proves that each header shows at most two buttons.
- M5: a test proves the title case of each label of step 4.
- M6: ADR 0068 task 004 has the change and the test of step 5.
- M7: the fit tests of ADR 0097 pass.
- M8: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and `cargo build --bin hitit` pass.

Visual, for the operator, in Dark and Light at M:

- V1: an Index release page shows "Download Album" as a Secondary button, with no blue fill.
- V2: a downloaded track shows "Add to Playlist" as a Secondary button. "MusicBrainz Lookup" is in the "…" menu.
- V3: the header of a release page and a track page agrees with musicindex.org in hierarchy.

## Test Commands

- `cargo test --lib view_models::track_detail`
- `cargo test --lib view_models::album_page`
- `cargo test --test architecture_tests`
- `cargo test app::fit_check::`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Report

Give the files changed and the tests of M1 to M5 by name. Give each label of step 4, before and after.

A delegate does not commit. It leaves its change in the working tree and reports it. Write each comment and the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if a header has three actions that change stored data and none can go into the "…" menu.
- Stop if a label is the text of a stored value or a test fixture of a different packet.
