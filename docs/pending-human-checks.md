# Pending Human Checks

## Scheduling

The operator resumed visual checks on 2026-10-02. Walk a new check right after its packet.
The older checks below stay open until a person walks them. The operator walked part of a visual batch on 2026-10-02 and stopped after its Part A step 8.

The current evidence fixture is `/tmp/v4vmm-governance.ie6k8TQf`. Its cleanup remains unconfirmed.

## Purpose

Some acceptance criteria need a person. The main agent first makes its own
check on a private display (`AGENTS.md`), and an agent result does not pass a
check here.

This file holds the checks that are open today. It is not a history. When a
check passes, do all three in the same change:

1. Record it in the `Status:` line of the owning packet in `docs/tasks/`.
2. Record it in the row in `docs/plans/broadcast-chain-delivery-order.md`.
3. Remove the section from this file, and number the sections that stay, so the
   order is still `1` to `n`.

When no check is open, this file keeps the function and the method sections
only. A closed check leaves no entry here.

## 1. Optional Tool Isolation — ADR 0066 Task 004

Open - implementation and mechanical checks recorded 2026-09-11.

- Owner: [task 004](tasks/adr-0066-task-004-optional-tool-isolation.md).
- Check: [Optional Tool Isolation](runbooks/startup-recovery-check.md#task-004-optional-tool-isolation).
- Needs a Linux desktop, Python 3.11+, this checkout's debug binary, installed
  mpv and working desktop audio for the producer-failure case. The fixture
  supplies local tracks, broken paths and external-service stubs.
- For paired Index/player failure, check local search, playlist visibility and
  retained reports. Check that Play and Ctrl+Alt+P cannot execute playback from Show.
  Task 007 supplies enabled repair routes while retaining the execution restriction.
- Check producer failure with audible playback. Check publisher failure with
  an independent producer and encoder. Check partially applied path repair.
- Check report copy and preservation for the publisher and path-repair cases.
  Record final fixture cleanup. Producer preservation passed on 2026-09-11.
  Its permitted workspace preference changes do not establish playback acceptance.
- Playback checks remain paused. The operator requires cue loading and playback
  from Show, with other Play buttons using a separate audition path.
  Current Music Play shares the Show session. The producer screenshot also
  reports an unresolved mpv IPC read error. Report-only checks cannot establish
  audio or publication behavior. See [task 004's correction](tasks/adr-0066-task-004-optional-tool-isolation.md#playback-workflow-correction--2026-09-11).
- [ADR 0068](adr/0068-show-cue-and-audition-isolation.md) proposes that separation.
  Its visual/audio check cannot run before implementation. The proposal closes
  no gate and reopens no accepted case.
- Task 005's completion closes none of this packet's remaining checks.
  The phase plan retains its scheduling exception.
- The three groups that used to precede this packet moved to the [overhaul plan](plans/design-and-cleanup-overhaul-plan.md#visual-requirements-moved-from-pending-checks---2026-10-07) on 2026-10-07.

## 2. Final Walk Of The Run Of 2026-10-20

Open - the run closed on 2026-10-23. The Walk Log of the run record shows the checks that the operator passed. Each other check of these packets stays open until a person walks it.

- Owner: the [run record](reviews/orchestrated-run-2026-10-20.md#final-walk).
- The packets of the run. Each item names its packet and its open checks:
  - [ADR 0084 packet 021](tasks/adr-0084-task-021-one-release-page-assembly.md) V1 to V3
  - [ADR 0084 packet 022](tasks/adr-0084-task-022-download-queue.md) V1 to V6
  - [ADR 0084 packet 025](tasks/adr-0084-task-025-playlist-selection.md) V1 to V7
  - [ADR 0084 packet 011](tasks/adr-0084-task-011-logo-frame.md) V1 to V7
  - [ADR 0084 packet 024](tasks/adr-0084-task-024-artist-feed-image.md) V1 to V3
  - [ADR 0083 task 006](tasks/adr-0083-task-006-row-hover-actions.md) V1 to V12
  - [ADR 0084 packet 027](tasks/adr-0084-task-027-stop-queued-downloads.md) V1 to V3
  - [ADR 0084 packet 023](tasks/adr-0084-task-023-one-track-page-assembly.md) V1 to V5
  - [ADR 0083 task 007](tasks/adr-0083-task-007-list-and-playlist-row-actions.md) V1 to V11
  - [ADR 0083 task 008](tasks/adr-0083-task-008-drag-track-to-playlist.md) V1 to V9
  - [ADR 0084 packet 016](tasks/adr-0084-task-016-library-and-everything-scopes.md) V1 to V8
  - [ADR 0084 packet 017](tasks/adr-0084-task-017-name-level-artist-page.md) V1 to V11
  - [ADR 0084 packet 010](tasks/adr-0084-task-010-search-results-list-column.md) V1 to V7
  - [ADR 0084 packet 012](tasks/adr-0084-task-012-saved-searches.md) V1 to V7
  - [ADR 0084 packet 028](tasks/adr-0084-task-028-search-open-defects.md) V1 to V4
  - [ADR 0084 packet 029](tasks/adr-0084-task-029-search-link-facts.md) V1 to V6
  - [ADR 0084 packet 026](tasks/adr-0084-task-026-index-singles-groups.md) V2 (the "You Are Enough" row only), V6, V7, V9 and V10. V6 and V10 failed on 2026-10-23 (run record, question 2 and walk log). Packet 035 V1 walks V6 again (section 7). The visual overhaul takes V10
  - [ADR 0084 packet 030](tasks/adr-0084-task-030-music-open-defects.md) V1 to V6
  - [ADR 0084 packet 031](tasks/adr-0084-task-031-release-row-actions-and-selection-drag.md) V1 to V9
  - [ADR 0084 packet 032](tasks/adr-0084-task-032-singles-group-rows.md) V1 to V7
  - [ADR 0084 packet 033](tasks/adr-0084-task-033-download-state-icons.md) V1 to V8
  - [ADR 0084 packet 034](tasks/adr-0084-task-034-store-feed-links-on-download.md) V1 to V4
- Packet 024 adds migration 20, and packet 012 adds migration 21. Before the first start of the new build, make the backup of the Rollback section of packet 012.
- Part B holds the deferred visual requirements of the overhaul plan that present screens can show.

## Method: Reach A Publisher Service State

This section is not a check. It is the method that each publisher check needs.
It stays here when every check above is closed.

A temporary drop-in file makes the unit enter a test state. The cleanup removes
only the named test files. Before each setup, check whether `zz-force-fail.conf`
already exists at that path. If it exists, stop. Do not overwrite an existing drop-in.

| State | How to reach it |
|---|---|
| `Active` | override `ExecStart=/bin/sleep infinity` |
| `Inactive` | `systemctl --user stop <unit>` |
| `Starting` | override `ExecStartPre=/bin/sleep 60`, then look while it starts |
| `Stopping` | stop a unit that takes time to shut down |
| `Failed` | see the two variants below |
| `NotInstalled` | the unit files are not linked into `~/.config/systemd/user/` |
| `Unknown` | no normal operation reaches this state |

### Failed With A Start Limit

A revoked token drives the unit to `failed` with `Result=start-limit-hit`. The
unit file sets `StartLimitBurst=5` and `RestartSec=5s`. Systemd starts the unit
five times, then stops. The unit sits in `Starting` between attempts.

```bash
mkdir -p ~/.config/systemd/user/musicindex-live-publisher@mixxx.service.d
printf '[Service]\nExecStart=\nExecStart=/bin/false\n' \
  > ~/.config/systemd/user/musicindex-live-publisher@mixxx.service.d/zz-force-fail.conf
systemctl --user daemon-reload
systemctl --user start musicindex-live-publisher@mixxx.service
```

Wait about 25 seconds.

### Failed Immediately

Use this alternative when you need an immediate failure and do not need the
start limit. `Restart=no` stops the restart loop.

```bash
mkdir -p ~/.config/systemd/user/mixxx-now-playing.service.d
printf '[Service]\nExecStart=\nExecStart=/bin/false\nRestart=no\n' \
  > ~/.config/systemd/user/mixxx-now-playing.service.d/zz-force-fail.conf
systemctl --user daemon-reload
systemctl --user start mixxx-now-playing.service
systemctl --user show mixxx-now-playing.service \
  --property=LoadState,ActiveState,SubState,Result
```

The result is `ActiveState=failed` and `Result=exit-code`.

### Cleanup

Use only the cleanup for the method you ran. The methods do not change the
app configuration. They require no app configuration restore. Other drop-ins
must remain intact.

For **Failed With A Start Limit**:

```bash
rm -f -- ~/.config/systemd/user/musicindex-live-publisher@mixxx.service.d/zz-force-fail.conf
systemctl --user daemon-reload
systemctl --user reset-failed musicindex-live-publisher@mixxx.service
```

For **Failed Immediately**:

```bash
rm -f -- ~/.config/systemd/user/mixxx-now-playing.service.d/zz-force-fail.conf
systemctl --user daemon-reload
systemctl --user reset-failed mixxx-now-playing.service
```

## References

- `docs/adr/0061-executable-governance.md`, for the mechanical and visual rule
- `docs/plans/broadcast-chain-delivery-order.md`

## 3. Metadata Migration Repair Report And Readiness — ADR 0075 Task 012

Open and paused - implementation, technical review, and mechanical checks are complete on 2026-09-20. Operator inspection is pending.

- Owner: [packet 012](tasks/adr-0075-task-012-provider-snapshot-migration.md#operator-visual-check).
- Check: separate version-11 repair and version-12 readiness reports, retained backup paths, recorded times, report copy, and normal/narrow presentation.
- Run the packet's temporary-fixture procedure only after mechanical review and the operator resumes visual checks.
- Preserve configuration, audio files, secret files, playlists, source records, and the selected event during the fixture check.
- Retain a failing fixture. Confirm successful preservation and fixture cleanup before closing this gate.
- This check has no runnable procedure and no owner (triage of 2026-10-20). The fixture checks a migration-11 ledger and migration-12 rows, and expects a separate version-12 candidate. Schema version 19 is current. The fixture needs revision for schema 19 before anyone can walk this check.

## 4. MusicBrainz URL Relations By Type — ADR 0080 Task 002

Open and paused - implementation and mechanical checks are complete on 2026-09-29. Operator inspection is pending.

- Owner: [task 002](tasks/adr-0080-task-002-musicbrainz-url-relations-by-type.md#operator-visual-check).
- Scheduling: run the procedure only after the operator resumes visual checks. It uses an isolated database copy and never writes to the Library. It needs network access to `musicbrainz.org`.
- V3: after a write of the homepage and the license on a test copy, an external tag reader shows plain URLs in `WOAR` and `WCOP`.
- V1, V2 and V4 moved to the [overhaul plan](plans/design-and-cleanup-overhaul-plan.md#visual-requirements-moved-from-pending-checks---2026-10-07) on 2026-10-07, under Phase 4: Inspect sources.

## 5. Forward Navigation — ADR 0046 Task 015

Open for macOS only - the operator passed V1, V3 and the Linux part of V2 on 2026-10-07, in Dark and Light, at UI scale L and at narrow width.

- Owner: [task 015](tasks/adr-0046-task-015-forward-navigation.md#operator-visual-check).
- Scheduling: it needs a macOS desktop session.
- V2, macOS part: `cmd-[` and `cmd-]` outside a text box do the same as Back and Forward. Indent and outdent still work in a macOS text box.

## 6. Final Walk Of The Run Of 2026-10-23

Open - the run closed on 2026-10-24 before the end of its walk. Each check below stays open. The fix packets of the walk come first. Walk the checks of a changed screen after its fix packet.

- Owner: the [run record](reviews/orchestrated-run-2026-10-23.md#final-walk).
- The walk starts with ADR 0084 packet 026 (section 2). The operator passed its V1 to V5 and V8 on 2026-10-23, apart from the "You Are Enough" row of V2. The operator passed packet 020 V1 to V6 on 2026-10-23.
- The packets of the run. Each item names its packet and its open checks:
  - [ADR 0086 packet 001](tasks/adr-0086-task-001-parallel-new-releases-detail-reads.md) V1, V2 and V4
  - [ADR 0086 packet 002](tasks/adr-0086-task-002-new-releases-window-groups.md) V4, V6 and V8. Packet 004 replaced V5 and V7
  - [ADR 0086 packet 003](tasks/adr-0086-task-003-pager-closes-windows.md) V1 and V4 to V10
  - [ADR 0086 packet 004](tasks/adr-0086-task-004-window-group-page.md) V4, V6, V9 and V10
  - [ADR 0086 packet 005](tasks/adr-0086-task-005-saved-search-window-groups.md) V1 to V7
  - [ADR 0087 packet 002](tasks/adr-0087-task-002-library-fields-and-index-words.md) V8 and V9
  - [ADR 0087 packet 004](tasks/adr-0087-task-004-field-list.md) V9. Packet 010 V1 replaces V1 (section 9). Packet 008 V2 replaces V6 (section 7)
  - [ADR 0087 packet 003](tasks/adr-0087-task-003-live-results.md) V2, V8 and V9. Packet 007 V1 and V3 replace V3, and packet 008 V1 replaces V6 (section 7)
  - [ADR 0087 packet 005](tasks/adr-0087-task-005-field-list-values.md) V3 and V6. Packet 010 V3 replaces V7 (section 9)

## 7. Final Walk Of The Run Of 2026-10-24

Open - the run closed on 2026-10-24 after its walk. The operator passed ADR 0084 packets 036 and 037 and ADR 0087 packet 008. Those packets are Complete and archived. The items below stay open. Walk a check of a changed screen after its fix packet.

- Owner: the [run record](reviews/orchestrated-run-2026-10-24.md#final-walk).
- The open checks. Each item names its packet:
  - [ADR 0084 packet 035](tasks/adr-0084-task-035-download-stores-feed-links.md) V2 and V3. Packet 038 implements the fix of the V1 defect. Its checks V2 to V4 walk packet 035 V1 to V3 again (section 8). V1 walks packet 026 V6 again
  - [ADR 0087 packet 007](tasks/adr-0087-task-007-live-run-index-rows.md) V4. Packet 009 replaces this check (section 9). With MusicIndex not reachable, a live run gave two errors, and they show as retained actions in the top bar
  - [ADR 0059 packet 019](tasks/adr-0059-task-019-livevalue-uri-is-a-url.md) V2 to V4. The operator removed the fixture before the steps of V2 and V3. V4 needs a real show feed. Its step 9 replaces the tag in each feed that holds the old form, outside the app

## 8. Packets 038 And 039 - Implemented 2026-10-25

Open - the packets are implemented. The mechanical suite is Green. No person checked these packets. Walk each check at a terminal in a desktop session. Each packet gives the commands.

- Owner: the packet files [038](tasks/adr-0084-task-038-downloaded-single-joins-its-group.md#operator-visual-check) and [039](tasks/adr-0084-task-039-download-queue-host-limit.md#operator-visual-check). Each file holds its steps and its cleanup.
- [ADR 0084 packet 038](tasks/adr-0084-task-038-downloaded-single-joins-its-group.md): V1 to V5. V2, V3 and V4 walk [packet 035](tasks/adr-0084-task-035-download-stores-feed-links.md) V1, V2 and V3 again. V1 is the group rows of "Everything", and V5 is the search "haleen d".
- [ADR 0084 packet 039](tasks/adr-0084-task-039-download-queue-host-limit.md): V1 to V4, and the step 4 observation of the lag (responsiveness item 32). V1 needs one group of 30 or more Wavlake singles that are not in the Library.

## 9. ADR 0087 Packets 009 And 010 - 2026-10-25

Open - both packets are implemented on 2026-10-25. The visual checks are not walked. A gate that no person walks is not met.

- Owner: the packet files of [packet 009](tasks/adr-0087-task-009-one-error-for-the-latest-text.md) and
  [packet 010](tasks/adr-0087-task-010-field-list-row-height.md). Each packet has its steps in "Operator Visual Check".
- The open checks:
  - [ADR 0087 packet 009](tasks/adr-0087-task-009-one-error-for-the-latest-text.md) V1 and V2. V1 replaces packet 007 V4. With MusicIndex not reachable, a live run of `haleen` then `haleen d` shows one note for `haleen d` and no retained action for a live run. V2 checks that Enter keeps one retained action with "Edit endpoint".
  - [ADR 0087 packet 010](tasks/adr-0087-task-010-field-list-row-height.md) V1 to V5. V1 walks packet 004 V1 again. V3 walks packet 005 V7 again. Each row keeps its height, the list scrolls to its end, and the scroll bar covers no text.

## 10. ADR 0080 Packets 007 To 013 And ADR 0076 Packet 011 - Implemented 2026-11-05

Open - the packets are implemented, and the mechanical suite is Green. No person checked these packets. The [run record](reviews/orchestrated-run-2026-11-05.md#final-walk) gives the order and the setup.

- Owner: the "Operator Visual Check" section of each packet file. Each file holds its steps and its cleanup.
- [Packet 007](tasks/adr-0080-task-007-contributor-role-frames.md) V1: a `remixer` credit goes into `TPE4`.
- [Packet 008](tasks/adr-0080-task-008-rss-readers-agree.md) V1: a feed with a banner `podcast:image` shows its cover.
- [Packet 009](tasks/adr-0080-task-009-funding-link-frame.md) V1 and V2 passed on MP3 files on 2026-11-05. The close of the run removes this line.
- [Packet 010](tasks/adr-0080-task-010-license-frames.md) V1 and V2: the RSS license in its frames, and on its own compare rows.
- [Packet 011](tasks/adr-0080-task-011-confirmed-label-frame.md) V1 and V2 passed on 2026-11-05. The run record owns the results, and the close of the run removes this line.
- [Packet 012](tasks/adr-0080-task-012-language-frame.md) V1 passed on 2026-11-05. The close of the run removes this line.
- [Packet 014](tasks/adr-0080-task-014-reader-keeps-each-frame.md) V1 passed on 2026-11-05. The run record owns the result, and the close of the run removes this line.
- [Packet 013](tasks/adr-0080-task-013-mp4-date-shares-one-value.md) V1: an MP4 file settles after one write. The run record also asks for the format of the two files of the observation of 2026-10-16.
- [ADR 0076 packet 011](tasks/adr-0076-task-011-new-reader-reads-each-feed-once.md) V1 and V2: the first Check RSS of a feed reads its full document, and its license reaches the "Update n files" list.

## 11. ADR 0089 The Shell - Implemented 2026-11-06

Open - packets 001 to 015 are implemented, and the mechanical suite is Green. No person checked these packets. The [run record](reviews/orchestrated-run-2026-11-05-shell.md) lists the decisions and seven questions for the operator.

- Owner: the "Visual" list of each packet file in `docs/tasks/adr-0089-task-*.md`. Each check runs in Light and Dark, at normal width and below 940 px.
- Build: run `cargo build --bin v4vmm` first, then open `target/debug/v4vmm` in a desktop session. The checks change no stored data, apart from migration 22, the checks of packets 006, 007 and 010, and their cleanup in those packets. Back up the database before migration 22, because a return to an older build needs the backup.
- `~/.claude/scripts/v4vmm-shots.sh` holds click points of the old layout, so its scenarios need new points before use.
- [Packet 001](tasks/adr-0089-task-001-one-page-stack.md) V1 and V2: one page at full width, and no empty pane.
- [Packet 002](tasks/adr-0089-task-002-toolbar-history-and-marked-row.md) V1 to V3: the toolbar, the scroll and the mark after Back, and Forward.
- [Packet 003](tasks/adr-0089-task-003-one-sidebar.md) V1 to V3: the start page, the sidebar on each mount, and the shortcuts.
- [Packet 004](tasks/adr-0089-task-004-list-page-state.md) V1 to V4: the scope, the title row, the state of each page and a failed download.
- [Packet 005](tasks/adr-0089-task-005-releases-sort.md) V1 to V5: window groups, the letter bar, tiles, Load more and smooth scroll.
- [Packet 006](tasks/adr-0089-task-006-search-page.md) V1 to V4: the Search page.
- [Packet 007](tasks/adr-0089-task-007-saved-search-edits-and-new-counts.md) V1 to V4: the menu, the query field and the counts.
- [Packet 008](tasks/adr-0089-task-008-results-rows.md) V1 to V4: the rows of the results page.
- [Packet 009](tasks/adr-0089-task-009-public-playlists.md) V1 to V3: Public playlists.
- [Packet 010](tasks/adr-0089-task-010-copy-to-my-playlists.md) V1 to V3: the copy. Delete the test playlist after the check.
- [Packet 011](tasks/adr-0089-task-011-live-page.md) V1 to V3: Live.
- [Packet 012](tasks/adr-0089-task-012-inbox.md) V1 to V4: the Inbox.
- [Packet 013](tasks/adr-0089-task-013-narrow-tab-bar.md) V1 to V3: the tab bar.
- [Packet 014](tasks/adr-0089-task-014-row-rules.md) V1 to V4: the rows.
- [Packet 015](tasks/adr-0089-task-015-inspect-sources-sheet.md) V1 to V4: the sheet.
- The search field list (ADR 0087 Decision 3), reported in the walk of 2026-11-06: a click on the page or the sidebar closes the list. A click on a list row keeps it open. A click on Search opens no list, and no list opens when the window gets the focus again.
- The keyboard focus (ADR 0089 Decision 9): Tab reaches the sidebar rows and the list. A row shows the focus when the list gets it. A mouse click shows no ring.
- The letter bar (ADR 0089 Decision 9): Tab stops once at the bar. Left, Right, Home and End move the cursor with no request. Enter or Space runs the letter.
- Edit Query (ADR 0089 Decision 4): the toolbar field holds the query with its field list. The title shows "Editing: <name>" with Save Query and Cancel. Enter saves, Escape cancels, and leaving the page or a new search cancels.
- The list bar stays in view ([ADR 0090 task 002](tasks/adr-0090-task-002-list-bar-stays-in-view.md)): the screenshot run of 2026-11-07 shows the pinned bar, "Done" and "Select" on it, a click on its empty part, and a narrow window. The agent screenshot run of 2026-11-07 shows "Add to playlist" and a download from the pinned bar without a fault. It found a fault: with the bar pinned, Tab passed 7 stops above the viewport that showed no focus. A key that moves the focus to a button now scrolls the column to show it (ADR 0090 Invariants).
- The list bar on the release page ([ADR 0090 task 003](tasks/adr-0090-task-003-release-page-list-bar.md)): the Library form and the Index form, in its visual list.
- The search corrections of ADR 0091 step 2: a sort selected on the results page reloads the rows in that order, `-remix` alone shows its note and no error, and `haleen kind:single` shows Index rows. A live run keeps relevance. The operator accepted the look of the sort rows, of `in:library` with the reverse icon and of the scope All on 2026-11-07. The orchestrator session relayed this answer from the agent screenshots. The operator also accepted that each sort can change the rows two times.
  - The agent screenshot run of 2026-11-07 also shows three corrections: a saved search with no name of its own names what its query reads ("Library", "Index", "Library and Index" or "No results"), the two entries of a row menu have one text size in Light and Dark, and Tab scrolls a page to show a focused button. A later run shows two more: Tab reaches "Add to playlist", "…", "Website" and "RSS" of a page header, and Enter or Space opens the menus. Rows, tiles and page headers with no cover show the logo shape of musicindex ADR 0021 R-ART-2 in Light and Dark.
- Wrong: a pane with no content, a control that does nothing, a page change after a mutation, or a lost keyboard focus.

## 12. musicindex ADR 0053 The List Controls - 2026-11-08

Open - the packets below are implemented, and the mechanical suite is Green. The main agent checked each packet on its own Xvfb display in Light and Dark. No person checked them. The [phase plan](plans/musicindex-adr-0053-list-controls-phase-plan.md) owns the packets.

- Build: run `cargo build --bin v4vmm`, then open `target/debug/v4vmm` in a desktop session. The checks change no stored data.
- musicindex ADR 0054 replaces the reverse icon of packets 1 to 9 on 2026-11-08. A sort menu lists the usual and the reversed direction of each sort. The tracks of a release and the entries of a public playlist have a text toggle "Reverse". The steps below use these controls.
- Packet 1: the sort menus of Releases All, Artists All and Public playlists list "Oldest" and "Z to A". "Oldest" starts at the oldest release. "Z to A" starts at Z, and a letter starts at the last name of that letter.
- Packet 2: search `haleen` shows "Relevance". The menu lists Relevance, A to Z and Newest, with no reversed entry in the scope All. "In Library" adds "Z to A" and "Oldest". `kind:single` starts with Newest. A saved search with a word, such as "haleen", opens with Newest.
- Packet 4: open a window group from Releases, for example "6 Singles". The list bar has Select at the left, and the sort menu, the rows icon and the tiles icon at the right. A to Z, Z to A and tiles change the page in place.
- Packet 4, tiles and Select: the tiles show the download icon. In Select mode the right end stays. The singles group page keeps its own sort and view.
- Packet 4, the narrow window: make the window narrower than 940 px. When the two ends of the bar are too wide for one line, each end goes on its own line. Scroll down until the bar pins, and use the sort menu on the pinned bar.
  - Agent Xvfb run of 2026-11-08, Light and Dark: at 900, 420 and 340 px the bar of "Advocateworldwide · 6 singles" stays on one line. At 300 px the tiles icon is cut off, and the bar stays on one line. The bar pins, and "Z to A" on the pinned bar changes the rows. In Dark, one stale strip of the row "Come As You Are" stayed below the rows after the scroll. A person decides if the cut at 300 px is a fault.
- Packet 5: open "Reward" from Releases (9 tracks). The list bar shows the toggle "Reverse" at the right. Pressed, it has the selected look, and the tracks come last to first, each with its own number. A release with 7 tracks shows Select only.
- Packet 5, public playlist: open "Boostagram Ball Playlist 1 to 25". The heading above the entries shows "278 items" and the toggle "Reverse". Pressed, the entries come last to first, each with its own number.
- musicindex ADR 0054: the toggle "Reverse" reads "Reverse, now Feed Order" or "Reverse, now Feed Order, Reversed" to a screen reader. The "Add to playlist" menus list the playlists in the Curator Order of My playlists.
- Packet 5, not checked by the agent: a Library release with 8 tracks or more. Select some tracks in the reversed sequence, and use "Add to playlist". The tracks go in the sequence of the selection.
- Packet 6 and [ADR 0090 task 004](tasks/adr-0090-task-004-feed-and-name-page-list-bar.md): open "Reward", then the artist link "Advocateworldwide". The Select bar is above "Albums and EPs", and the page has no bottom bar. "Singles" has "A to Z", rows and tiles at the right of its heading. "Albums and EPs" (4 rows) has no control.
- Packet 6, selection: select a row in "Albums and EPs" and a tile in "Singles": the bar shows "2 releases selected".
- Packet 6, not checked by the agent: a Library feed page with 8 releases or more shows Newest. A name level page with 8 releases or more.
  - Agent Xvfb run of 2026-11-08: at 900 and 400 px, the Select bar of the Advocateworldwide page stays on its own line with no overlap.
- Packet 7: on a Library list with A to Z, a name with a number such as "Track 2" comes before "Track 10", and "abba" and "ABBA" are adjacent. The agent screenshots of 2026-11-08 show no such names in the seeded Library.
- Packet 8: on Artists, the tiles icon shows the artists as tiles in All and In Library. A name with no feed shows the logo shape. The letter M, Z to A and Load more work with tiles.
- Packet 8, the end of the tiles and the keyboard: the agent Xvfb run of 2026-11-08 shows "Show more" at the end of the tiles in All, and more tiles come after a click.
  - Fault found by the agent run of 2026-11-08: Tab does not go to a tile. After "Show more", the focus goes back to the sidebar. Only the letter A of the letter bar takes the focus. This item stays open until a fix and a person check.
- Packet 3 (ADR 0089 amendment of 2026-11-08): back up the database first, because migration 24 adds `playlists.position`. My playlists keeps the sequence of today and starts with Curator Order.
- Packet 3, the drag: drag a playlist row onto a different row. It moves there, and it stays there after a restart of the app. A new playlist goes last. A to Z, Newest and tiles work. The row menu has "Move up" and "Move down" in Curator Order.
- Packet 3, not checked by the agent: the migration on the database of the operator, "Move up" and "Move down", and a drag with the keyboard focus on a row.
- [ADR 0092](adr/0092-the-keys-of-a-list-row.md): on My playlists, Tab reaches the list. Up and Down move the focus, and Return opens a playlist. Shift+F10 or the Menu key opens the row menu with its first entry focused. Down, Up and Return choose and run an entry, and Escape gives the focus back to the row.
- ADR 0092, more keys: Alt+Up and Alt+Down move a playlist in Curator Order. On Releases and the Inbox, Shift+F10 opens the menu of the focused row. The agent screenshots of 2026-11-08 show each of these. A person checks them with a hardware keyboard, also the Menu key.
- Packet 9: on Artists in All, the sort menu lists A to Z and Newest. Newest starts with the artists of the newest releases and shows no letter bar. The menu entry "Oldest" reverses it. In Library, Newest puts the artist of the newest Library release first.
- Packet 9, Show more and the scope: the agent Xvfb run of 2026-11-08 shows, in Light and Dark, that "Show more" with Newest in All adds rows after the last row. With Newest selected, "In Library" shows "No artists yet" in the empty Library, and "All" shows the rows again with Newest.
- The pin of `musicindex-search` at `634e4cc` (ADR 0091): Enter on `kind:foo` shows "This search cannot run". Enter on `-remix` shows the note "Music Index needs a word or a field to search for" above the Library rows. The Artists tab of `kind:single` shows "kind: does not apply to artists and labels". The agent screenshots of 2026-11-08 show these. A person checks that the words are clear.
- The pin at `e84bab5` (musicindex ADR 0058 R-QUERY-11 and R-QUERY-12): click the empty search field. The rows `artist:`, `title:` and `release:` show a muted placeholder such as `artist:"artist name"`. `artist:` and `release:` also show a darker example from the Library.
- The pin at `e84bab5`, the clicks: click the placeholder. The field shows `artist:"artist name"` with "artist name" selected, and no search runs. When you type a name, the search runs. When you click a darker example, the search runs.
- The pin at `e84bab5`, the visits: close and open the list again. The darker examples can change. The agent screenshots of 2026-11-08 show these in Light and Dark. A person checks that the placeholder and the example are easy to tell apart.
- musicindex ADR 0023 R-WORD-10: open the feed pages of Sir Libre Records and Crash Landing. Each page has a shared releases section below the releases.
  - Agent Xvfb run of 2026-11-08, Light and Dark: Crash Landing (badge "Artist") shows "Labels and other feeds" and the note. Its row reads "1 shared release · releases give rel="label"". This part agrees.
  - Fault found by the same run: Sir Libre Records lists "Roles: label" and "As label", but its header badge reads "Publisher". So the section has the heading "Shares releases with", not "Artists on these releases".
  - Cause: the badge joins the agreed role words with the words of one side (musicindex ADR 0033 R-WORD-3 item 3). Three albums of this feed give `rel="recordLabel"` on the album side, so the badge sees two roles. The orchestrator decides if the badge uses a role word that it does not know.
- R-WORD-10, the words: a label page has the heading "Artists on these releases". An artist page has "Labels and other feeds". A page of a different type has "Shares releases with".
- R-WORD-10, the note and the rows: the note below the heading is "The same release names each feed here and this feed. The two feeds of each link confirm it." A row reads "1 shared release · releases give rel="…"". Examine the section in Light and Dark.
- musicindex ADR 0023 R-WORD-9: open Artists, select All, and go to the letter #. The agent Xvfb run of 2026-11-08 shows these two rows with the text below in Dark. The row "$2 Holla" reads "Not in Library · by name · 1 album · 2 singles". The row "0$" reads "Not in Library · 1 album · 18 singles".
- R-WORD-9, search: search `olamide` and open the Artists tab. The feed row of OLAMIDE reads "14 albums · 38 singles". Search `jsmovie`. Its feed row reads "17 singles". Make sure that the text does not overflow in a narrow window, in Light and Dark.
  - The agent Xvfb run of 2026-11-08 shows the two rows with this text in Dark. It shows OLAMIDE in Light. Each row is correct at full width and at 700 px.
- [musicindex ADR 0057](adr/0085-artists-by-confirmation.md) R-ARTIST-1: Artists in All, its tiles and the Artists tab of a search show three groups with a note each. The groups are "Stated artists · n", "Inferred artists" and "Artist names with no feed", in this sequence.
  - The agent Xvfb run of 2026-11-08 shows this in Light and Dark. "Show more" in the inferred group adds rows, and the other groups stay. A search with no stated artist, such as `haleen`, shows no empty group. The tiles cut their second line, for example "2 albu…" and "by na…". A person checks that the groups are clear and that the cut text is satisfactory.
- Wrong: a control that does nothing, rows in an incorrect sequence, or no selection after a sort. A sort control on a list that R-LIST-9 does not give one is also wrong.
