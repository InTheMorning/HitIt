# ADR 0101 Task 011: The Main Action

Status: Implemented - operator visual checks open, 2026-11-16.

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

Each fact is from HitIt `ba43138`, read on 2026-11-15. A ChatGPT search found them, and the main session examined the items marked "examined".

- `TrackDetail::page_actions` (`src/view_models/track_detail.rs:593`) gives one `filled` action in each state: DownloadAlbum for an Index track, DownloadTrack for a Library track with no audio, and AddToPlaylist for a downloaded track. Examined.
- `render_track_page_actions` (`src/ui/shells/track_page.rs:106`) draws `filled` with `ControlStyle::Primary` and each `plain` action with `ControlStyle::Neutral`. Examined.
- `AlbumPageAction::label` (`src/view_models/album_page.rs:36`) gives "Download {kind}", "Add to playlist", "Copy feed URL", "MusicBrainz lookup" and "Remove {kind}…". Examined. These are not in title case.
- `album_page_actions` (`src/view_models/album_page.rs:114`) gives DownloadAlbum as `filled` for an Index release. Examined.
- `src/ui/shells/release_page.rs:231` draws the release action with `ControlStyle::Primary`. Examined.
- `TrackPageAction::label` (`src/view_models/track_detail.rs:893`) gives "Download release" when no kind is known.
- `ControlStyle::by_result` (`src/ui/control_styles.rs:141`) gives Secondary for a command that changes stored data, else Neutral. Examined.
- This is an inference: no header has a Play or an Audition action. ADR 0068 task 004, the audition player, is Ready.
- Tests: `adr_0083_r83_42_index_track_actions` (`src/view_models/track_detail.rs:1627`) and `adr_0083_album_page_actions_follow_the_state_table` (`src/view_models/album_page.rs:218`).

Main session choice, 2026-11-15: a page header has no filled button after this packet. The style of R-ACT-7 item 1 comes with its first user, the audition button of ADR 0068 task 004. Operator choice, 2026-11-15: its label is "Audition" (open point 1 of the [phase plan](../plans/adr-0101-design-language-phase-plan.md#open-points)).

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

## Implementation Report - 2026-11-16

The delegate did not commit. The change is in the working tree.

### Files changed

- `src/view_models/track_detail.rs`: `TrackPageActions::filled` is `Option` and is `None` in each state. The download and the add to a playlist are plain buttons, with Download first. New `TrackPageAction::changes_stored_data`. The labels use title case.
- `src/view_models/album_page.rs`: the same change for `AlbumPageActions`. New `AlbumPageAction::changes_stored_data`. The kind in a label comes from `ReleaseKind::label`.
- `src/view_models/track_page.rs` and `src/view_models/release_page.rs`: `filled` is `Option` in the two action row models. The row labels of the track menu use title case.
- `src/ui/shells/track_page.rs` and `src/ui/shells/release_page.rs`: each plain button has the role `ControlStyle::by_result`. A filled button, when one exists, has the role Primary.
- `src/ui/control_styles.rs`: test for the roles.
- `src/view_models/singles_group.rs`, `src/ui/shells/library/singles_group.rs`: see the choices below.
- `src/view_models/library.rs`, `src/view_models/playlist_rss_check.rs`, `src/view_models/release_row_actions.rs`: labels in title case, and the tests that assert them.
- `tests/architecture_tests.rs`: the guard of the playlist row menu labels follows the new labels. The guard of ADR 0084 R7-07 also forbids the literals "Download Album" and "Remove Album…".
- `docs/tasks/adr-0068-task-004-audition-player.md`: required change 10 and criterion M9 (step 5).

### Tests of M1 to M5

- M1: `adr_0101_decision_5_index_track_actions_give_no_filled_action`, `adr_0101_decision_5_library_track_not_downloaded_actions`, `adr_0101_decision_5_downloaded_track_actions` (`track_detail.rs`).
- M2: `adr_0101_decision_5_album_page_actions_give_no_filled_action` (`album_page.rs`). It covers an Index release and Library releases.
- M3: `adr_0101_decision_5_header_download_and_playlist_buttons_are_secondary` (`control_styles.rs`).
- M4: `adr_0101_decision_5_track_buttons_are_secondary_and_at_most_two` and `adr_0101_decision_5_album_page_buttons_are_secondary_and_at_most_two`.
- M5: `adr_0084_release_kind_r7_06_track_download_label_names_the_known_kind` (track) and `adr_0084_release_kind_r7_05_download_and_remove_name_the_kind` (release).

### Labels, before and after

- "Download release" to "Download Release".
- "Download track" to "Download Track".
- "Download single/EP/album" to "Download Single/EP/Album".
- "Add to playlist" to "Add to Playlist".
- "Copy feed URL" to "Copy Feed URL".
- "MusicBrainz lookup" to "MusicBrainz Lookup".
- "Remove track…" to "Remove Track…".
- "Remove single/album…" to "Remove Single/Album…".
- Row menu: "Add to playlist" to "Add to Playlist".
- "Move up/down" to "Move Up/Down".
- "Remove from playlist" to "Remove from Playlist".
- "Remove from all playlists…" to "Remove from All Playlists…".
- Singles group header: "Download n singles" to "Download n Singles".
- "Add to playlist" to "Add to Playlist".

### Choices that the packet does not give

- Order and roles: a Library track that is not downloaded shows "Download Track", then "Add to Playlist". An Index release or track shows only its download button. A downloaded item shows "Add to Playlist".
- `filled` stays as an `Option` that is always `None`, as step 1 says. The shells draw a `Some` value with Primary. No state gives one.
- Singles group page and window group page: their header had a Primary button. This packet does not name them. The delegate changed the role to Secondary and the labels to title case, to agree with the goal. The main session can reverse this.
- Not changed: the selection action bar labels, the playlist popover test fixture and the tooltip test text.

### Check results

`cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (3236 library tests, 416 architecture tests, `app::fit_check::` 3 tests) and `cargo build --bin hitit` pass. The visual criteria V1 to V3 are open.

## Primary Agent Check - 2026-11-16

The primary agent read the diff and ran the four checks of M8. All pass: 3236 library tests and 416 architecture tests.

The primary agent accepts the decision of the delegate on the singles group pages. Their header button is Secondary after this change, because the goal says that no page header fills a button.

## Walk Result - 2026-11-16

The agent walked on an Xvfb display with the debug binary in Light at M.

- V1: Pass. The Index release "Fox Trot Tunes Volume 2" shows "Download Album" as a Secondary button, with no blue fill. Its "⋯" menu shows "Copy Feed URL".
- V2: Pass. The downloaded track "Let Us Waltz As We Say Goodbye" shows "Add to Playlist" as a Secondary button. Its "⋯" menu shows "Copy Feed URL", "Inspect Sources", "MusicBrainz Lookup" and "Remove Track…". The Library release "Waltz Tunes Volume 2" also shows "Add to Playlist" as a Secondary button.
- V3: Open. The test display has no browser for musicindex.org.

The operator confirmation stays open.
