# Pending Human Checks

## Scheduling

The operator resumed visual checks on 2026-10-02. Walk a new check right after its packet.
The older checks below stay open until a person walks them. The operator walked part of a visual batch on 2026-10-02 and stopped after its Part A step 8.
The [agent walk of 2026-11-17](reviews/agent-walk-2026-11-17.md) walked sections 6 to 9, 11 and 22 to 33 on Xvfb. It lists the checks that only a person can do.
The [agent walk of 2026-11-18](reviews/agent-walk-2026-11-18.md) walked sections 11, 13, 15 to 17, 25 and 33. It took the items that the earlier walk left open for an agent.

The current evidence fixture is `/tmp/hitit-governance.ie6k8TQf`. Its cleanup remains unconfirmed.

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

## Operator Run Sheet - 2026-11-09

This section is not a check. It gives the commands that each check below uses.
Run them in a terminal of the i3 desktop session. Each numbered section has a `Run:` block that refers to these commands.

### Start The App

Close each open HitIt window first. Then build and start the app:

```bash
cd ~/build/HitIt
cargo build --bin hitit
target/debug/hitit &
```

Run `cargo build --bin hitit` again after each `cargo test`, because a test build can replace the binary.

### Before A Migration

The build of 2026-11-09 has schema version 24. Migration 24 adds `playlists.position`. Before the first start of a new build, read the schema version of the database:

```bash
sqlite3 -readonly ~/.local/share/hitit/hitit.sqlite 'SELECT max(version) FROM schema_migrations;'
```

If the result is less than 24, close the app and make a backup:

```bash
sqlite3 ~/.local/share/hitit/hitit.sqlite ".backup '$HOME/.local/share/hitit/hitit-before-24.sqlite'"
```

To go back to an older build, close the app and copy the backup over `hitit.sqlite`.

### Set The Window Width

Type this function one time in each terminal. It sets the dimensions of the HitIt window:

```bash
v4w() { local wid; wid=$(xdotool search --onlyvisible --pid "$(pgrep -x hitit | head -1)" | head -1); i3-msg -q "[id=$wid] floating enable, resize set $1 $2"; }
```

Examples: `v4w 1440 845` for full width, `v4w 900 845` for a narrow window, `v4w 300 845` for the smallest width of section 12.

### Light And Dark

Open Settings with the sidebar row "Settings". Select "Theme", then "Light" or "Dark". The theme changes immediately.

### What The Agent Could Not Run

The agent checks run on an Xvfb display with scratch `XDG_CONFIG_HOME`, `XDG_DATA_HOME` and `XDG_CACHE_HOME` folders. They do not touch the Library of the operator. These items need the operator:

1. **The Library downloads.** The scratch Library has no release with 8 tracks or more and no feed with 8 releases or more. Section 12 packets 5 and 6 need them. Find candidates in your Library:

   ```bash
   sqlite3 -readonly ~/.local/share/hitit/hitit.sqlite "SELECT f.title, count(*) FROM tracks t JOIN feeds f ON f.id = t.feed_id WHERE t.is_in_library = 1 GROUP BY f.id HAVING count(*) >= 8 ORDER BY count(*) DESC LIMIT 10;"
   sqlite3 -readonly ~/.local/share/hitit/hitit.sqlite "SELECT album_artist_name, count(DISTINCT feed_id) FROM tracks WHERE is_in_library = 1 GROUP BY album_artist_name HAVING count(DISTINCT feed_id) >= 8 ORDER BY 2 DESC LIMIT 10;"
   ```

   If a query gives no row, download one Index release with 8 tracks or more before the check.
2. **The packet 3 migration on the operator database.** Migration 24 ran only on scratch databases. Only the operator database shows that the stored playlist sequence stays. Use "Before A Migration" first.
3. **A hardware keyboard with the Menu key.** The agent sends keys with `xdotool`. ADR 0092 needs a person to push Shift+F10 and the Menu key on a hardware keyboard.
4. **A feed with more than 1 shared release.** The agent saw only the row text "1 shared release". The plural row text needs a feed page with 2 or more. On 2026-11-09, `/v1/artist-names` of the live node gives no `two_way_link_count`, so no command finds such a feed. If you know such a feed, open its page, or give its name to the agent.
5. **Names with numbers in the Library.** The scratch Library has no names such as "Track 2" and "Track 10". Packet 7 of section 12 needs them. Find candidates:

   ```bash
   sqlite3 -readonly ~/.local/share/hitit/hitit.sqlite "SELECT DISTINCT track_title FROM tracks WHERE is_in_library = 1 AND track_title GLOB '*[0-9]*' LIMIT 20;"
   ```

6. **Audio, mpv, a macOS desktop and `musicbrainz.org`.** Sections 1, 4 and 5 need them. The agent display has no audio, and the agent has no macOS desktop.

### Decisions For The Operator

Each item gives the recommendation of the agent. The operator decides.

1. **The badge of Sir Libre Records.** Must the header badge ignore a role word that it does not know, such as `recordLabel`? Recommendation: yes, and the orchestrator agrees. Then the badge reads "Label", and the section heading is "Artists on these releases". musicindex ADR 0023 R-WORD-3, as musicindex ADR 0033 changed it, owns this rule. A change needs a record in `../musicindex` first.
   Answer, 2026-11-14: yes. The operator accepted the recommendation. musicindex ADR 0066 (commit `df500dc`) records the rule. [ADR 0085 task 001](tasks/adr-0085-task-001-type-label-of-known-roles.md) builds it.
2. **The tiles icon at 300 px (section 12, packet 4).** Is the cut tiles icon a fault? Recommendation: yes. A control that a person cannot see is not available. Let the bar go on two lines when one line cannot hold the two ends.
3. **The cut second line of an artist tile (section 12, musicindex ADR 0057).** Is "2 albu…" satisfactory? Recommendation: accept it in tiles, because the rows view gives the full text.
4. **The stale strip in Dark (section 12, packet 4).** The agent saw one stale strip after a scroll. Recommendation: walk the step on your display. If the strip shows there, open a fix packet.
5. **Tab does not go to an artist tile (section 12, packet 8).** Recommendation: open a fix packet in ADR 0092 and ADR 0089 Decision 9. Do not close packet 8 before the fix.
6. **Section 3, the migration repair report.** Its fixture needs schema 12, and the build has schema 24. Recommendation: revise the fixture for schema 24 in a packet, then walk the check. The agent did not examine if a database below schema 12 exists.
7. **Section 5, macOS.** Recommendation: keep it open until a macOS desktop is available.
8. **The questions of the [shell run record](reviews/orchestrated-run-2026-11-05-shell.md#questions-for-the-final-walk).** The [ADR 0089 amendment of 2026-11-08](adr/archive/0089-the-shell-destinations-and-list-pages.md#amendment---2026-11-08-the-stored-curator-order) answers question 2. Questions 1 and 3 to 7 stay open. Recommendation: answer them during the walk of section 11.

## 1. Optional Tool Isolation — ADR 0066 Task 004

Open - implementation and mechanical checks recorded 2026-09-11.

- Owner: [task 004](tasks/adr-0066-task-004-optional-tool-isolation.md).
- Run: the runbook section [Task 004](runbooks/startup-recovery-check.md#task-004-optional-tool-isolation) gives each command. It starts with this fixture setup:

```bash
cd ~/build/HitIt
recovery_dir="$(python3 docs/runbooks/startup-recovery-fixture.py setup)"
python3 docs/runbooks/startup-recovery-fixture.py verify "$recovery_dir"
```

  Expected: the runbook gives the result of each case. Do not walk the playback cases before ADR 0068 is implemented. You can walk the report cases at this time.
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
- [ADR 0068](adr/0068-show-cue-and-audition-isolation.md), Accepted on 2026-11-11, decides that separation.
  Its visual/audio check cannot run before implementation. The decision closes
  no gate and reopens no accepted case.
- Task 005's completion closes none of this packet's remaining checks.
  The phase plan retains its scheduling exception.
- The three groups that used to precede this packet moved to the [overhaul plan](plans/design-and-cleanup-overhaul-plan.md#visual-requirements-moved-from-pending-checks---2026-10-07) on 2026-10-07.

## 2. Final Walk Of The Run Of 2026-10-20

Open - the run closed on 2026-10-23. The Walk Log of the run record shows the checks that the operator passed. Each other check of these packets stays open until a person walks it.

- Owner: the [run record](reviews/orchestrated-run-2026-10-20.md#final-walk).
- Run: do "Before A Migration" of the Operator Run Sheet, then start the app:

```bash
cd ~/build/HitIt
cargo build --bin hitit
target/debug/hitit &
```

  Then open each packet file in the list below. Do the steps of its "Operator Visual Check" section in the sequence of the list. Each packet states what to look at and the expected result.
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
- Run: no command is available. The fixture needs schema 12, and the build has schema 24. Decision 6 of the Operator Run Sheet applies.
- Check: separate version-11 repair and version-12 readiness reports, retained backup paths, recorded times, report copy, and normal/narrow presentation.
- Run the packet's temporary-fixture procedure only after mechanical review and the operator resumes visual checks.
- Preserve configuration, audio files, secret files, playlists, source records, and the selected event during the fixture check.
- Retain a failing fixture. Confirm successful preservation and fixture cleanup before closing this gate.
- This check has no runnable procedure and no owner (triage of 2026-10-20). The fixture checks a migration-11 ledger and migration-12 rows, and expects a separate version-12 candidate. Schema version 19 is current. The fixture needs revision for schema 19 before anyone can walk this check.

## 4. MusicBrainz URL Relations By Type — ADR 0080 Task 002

Open and paused - implementation and mechanical checks are complete on 2026-09-29. Operator inspection is pending.

- Owner: [task 002](tasks/adr-0080-task-002-musicbrainz-url-relations-by-type.md#operator-visual-check).
- Run: build the app, then do the steps of the [Operator Visual Check](tasks/adr-0080-task-002-musicbrainz-url-relations-by-type.md#operator-visual-check) of task 002. They make an isolated database copy:

```bash
cd ~/build/HitIt
cargo build --bin hitit
```

  Look at: the `WOAR` and `WCOP` frames of the test copy in an external tag reader. Expected: plain URLs.
- Scheduling: run the procedure only after the operator resumes visual checks. It uses an isolated database copy and never writes to the Library. It needs network access to `musicbrainz.org`.
- V3: after a write of the homepage and the license on a test copy, an external tag reader shows plain URLs in `WOAR` and `WCOP`.
- V1, V2 and V4 moved to the [overhaul plan](plans/design-and-cleanup-overhaul-plan.md#visual-requirements-moved-from-pending-checks---2026-10-07) on 2026-10-07, under Phase 4: Inspect sources.

## 5. Forward Navigation — ADR 0046 Task 015

Open for macOS only - the operator passed V1, V3 and the Linux part of V2 on 2026-10-07, in Dark and Light, at UI scale L and at narrow width.

- Owner: [task 015](tasks/adr-0046-task-015-forward-navigation.md#operator-visual-check).
- Run: on a macOS desktop, build and start the app:

```bash
cd ~/build/HitIt
cargo build --bin hitit
target/debug/hitit &
```

  Open a release, then a second release. Push `cmd-[`, then `cmd-]`. Expected: Back, then Forward. Then put the cursor in a text box and push `cmd-]` and `cmd-[`. Expected: indent and outdent work, and the page does not change.
- Scheduling: it needs a macOS desktop session.
- V2, macOS part: `cmd-[` and `cmd-]` outside a text box do the same as Back and Forward. Indent and outdent still work in a macOS text box.

## 6. Final Walk Of The Run Of 2026-10-23

Open - the run closed on 2026-10-24 before the end of its walk. Each check below stays open. The fix packets of the walk come first. Walk the checks of a changed screen after its fix packet.

- Owner: the [run record](reviews/orchestrated-run-2026-10-23.md#final-walk).
- Run: start the app:

```bash
cd ~/build/HitIt
cargo build --bin hitit
target/debug/hitit &
```

  Then open each packet file in the list below. Do the steps of its "Operator Visual Check" section for the named checks only.
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

The [agent walk of 2026-11-17](reviews/agent-walk-2026-11-17.md) passed ADR 0086 packet 002 V4, V6 and V8, packet 003 V4 to V7 in List, packet 004 V4, V9 and V10, packet 005 V3, and ADR 0087 packet 002 V6 and V8, packet 003 V8, packet 004 V9 and packet 005 V5. ADR 0098 replaced the screens of packet 004 V2, ADR 0087 packet 002 V2 to V4 and V9, and packet 003 V5. The record lists the checks that stay open. Most of them need downloads, timing or a network cut.

## 7. Final Walk Of The Run Of 2026-10-24

Open - the run closed on 2026-10-24 after its walk. The operator passed ADR 0084 packets 036 and 037 and ADR 0087 packet 008. Those packets are Complete and archived. The items below stay open. Walk a check of a changed screen after its fix packet.

- Owner: the [run record](reviews/orchestrated-run-2026-10-24.md#final-walk).
- Run: start the app:

```bash
cd ~/build/HitIt
cargo build --bin hitit
target/debug/hitit &
```

  Then do the named checks in each packet file below. ADR 0059 packet 019 V4 also needs a published show feed.
- The open checks. Each item names its packet:
  - [ADR 0084 packet 035](tasks/adr-0084-task-035-download-stores-feed-links.md) V2 and V3. Packet 038 implements the fix of the V1 defect. Its checks V2 to V4 walk packet 035 V1 to V3 again (section 8). V1 walks packet 026 V6 again
  - [ADR 0087 packet 007](tasks/adr-0087-task-007-live-run-index-rows.md) V4. Packet 009 replaces this check (section 9). With MusicIndex not reachable, a live run gave two errors, and they show as retained actions in the top bar
  - [ADR 0059 packet 019](tasks/adr-0059-task-019-livevalue-uri-is-a-url.md) V2 to V4. The operator removed the fixture before the steps of V2 and V3. V4 needs a real show feed. Its step 9 replaces the tag in each feed that holds the old form, outside the app

The [agent walk of 2026-11-17](reviews/agent-walk-2026-11-17.md) passed ADR 0059 packet 019 V1 to V3 on the local fixture. Packet 009 V1 replaces ADR 0087 packet 007 V4, and it passed. Packet 035 V2 and V3 need downloads. Packet 019 step 8 and V4 need a real show feed.

## 8. Packets 038 And 039 - Implemented 2026-10-25

Open - the packets are implemented. The mechanical suite is Green. No person checked these packets. Walk each check at a terminal in a desktop session. Each packet gives the commands.

- Owner: the packet files [038](tasks/adr-0084-task-038-downloaded-single-joins-its-group.md#operator-visual-check) and [039](tasks/adr-0084-task-039-download-queue-host-limit.md#operator-visual-check). Each file holds its steps and its cleanup.
- Run: start the app:

```bash
cd ~/build/HitIt
cargo build --bin hitit
target/debug/hitit &
```

  For packet 038 V5, push Ctrl+F, type `haleen d` and push Enter. For packet 039 V1, the app needs a connection to `https://api.musicindex.org`.
- [ADR 0084 packet 038](tasks/adr-0084-task-038-downloaded-single-joins-its-group.md): V1 to V5. V2, V3 and V4 walk [packet 035](tasks/adr-0084-task-035-download-stores-feed-links.md) V1, V2 and V3 again. V1 is the group rows of "Everything", and V5 is the search "haleen d".
- [ADR 0084 packet 039](tasks/adr-0084-task-039-download-queue-host-limit.md): V1 to V4, and the step 4 observation of the lag (responsiveness item 32). V1 needs one group of 30 or more Wavlake singles that are not in the Library.

The [agent walk of 2026-11-17](reviews/agent-walk-2026-11-17.md) could not download from `wavlake.com` (HTTP 403 in the sandbox). Each check of this section stays open for the operator desktop. Packet 038 V1 also needs feeds 109 to 128 of the operator database.

## 9. ADR 0087 Packets 009 And 010 - 2026-10-25

Open - both packets are implemented on 2026-10-25. The visual checks are not walked. A gate that no person walks is not met.

- Owner: the packet files of [packet 009](tasks/adr-0087-task-009-one-error-for-the-latest-text.md) and
  [packet 010](tasks/adr-0087-task-010-field-list-row-height.md). Each packet has its steps in "Operator Visual Check".
  [packet 010](tasks/adr-0087-task-010-field-list-row-height.md). Each packet has its steps in "Operator Visual Check".
- Run: for packet 009, close the app, then make MusicIndex not reachable and start the app:

```bash
cd ~/build/HitIt
cargo build --bin hitit
cp ~/.config/hitit/config.toml ~/.config/hitit/config.toml.v009-check
sed -i 's|^musicindex_endpoint = .*|musicindex_endpoint = "http://127.0.0.1:9"|' ~/.config/hitit/config.toml
grep musicindex_endpoint ~/.config/hitit/config.toml
target/debug/hitit &
```

  If `grep` shows no line, add the line `musicindex_endpoint = "http://127.0.0.1:9"` to the file. Then do steps 6 to 8 of the packet: type `haleen`, then ` d`. Expected: one note for `haleen d`, and no retained item. Push Enter. Expected: one retained item with "Edit endpoint". Cleanup: close the app, then put back the configuration:

```bash
mv ~/.config/hitit/config.toml.v009-check ~/.config/hitit/config.toml
```

  For packet 010, start the app with the usual configuration. Click the empty search field and scroll the field list to its end.
- The open checks:
  - [ADR 0087 packet 009](tasks/adr-0087-task-009-one-error-for-the-latest-text.md) V1 and V2. V1 replaces packet 007 V4. With MusicIndex not reachable, a live run of `haleen` then `haleen d` shows one note for `haleen d` and no retained action for a live run. V2 checks that Enter keeps one retained action with "Edit endpoint".
  - [ADR 0087 packet 010](tasks/adr-0087-task-010-field-list-row-height.md) V1 to V5. V1 walks packet 004 V1 again. V3 walks packet 005 V7 again. Each row keeps its height, the list scrolls to its end, and the scroll bar covers no text.

The [agent walk of 2026-11-17](reviews/agent-walk-2026-11-17.md) passed packet 009 V1 and V2 and packet 010 V1, V2, V4 and V5. Packet 010 V3 and the Library rows of packet 009 V1 need Library names. Step 7 of packet 010 must say 12 Down presses, not 11.

## 10. ADR 0080 Packets 007 To 013 And ADR 0076 Packet 011 - Implemented 2026-11-05

Open - the packets are implemented, and the mechanical suite is Green. No person checked these packets. The [run record](reviews/orchestrated-run-2026-11-05.md#final-walk) gives the order and the setup.

- Owner: the "Operator Visual Check" section of each packet file. Each file holds its steps and its cleanup.
- Owner: the "Operator Visual Check" section of each packet file. Each file holds its steps and its cleanup.
- Run: start the app:

```bash
cd ~/build/HitIt
cargo build --bin hitit
target/debug/hitit &
```

  Then do the steps of each packet below in the sequence of the run record. Packets that write tags use a test copy of a file, as the packet states.
- [Packet 007](tasks/adr-0080-task-007-contributor-role-frames.md) V1: a `remixer` credit goes into `TPE4`.
- [Packet 008](tasks/adr-0080-task-008-rss-readers-agree.md) V1: a feed with a banner `podcast:image` shows its cover.
- [Packet 009](tasks/adr-0080-task-009-funding-link-frame.md) V1 and V2 passed on MP3 files on 2026-11-05. The close of the run removes this line.
- [Packet 010](tasks/adr-0080-task-010-license-frames.md) V1 and V2: the RSS license in its frames, and on its own compare rows.
- [Packet 011](tasks/adr-0080-task-011-confirmed-label-frame.md) V1 and V2 passed on 2026-11-05. The run record owns the results, and the close of the run removes this line.
- [Packet 012](tasks/adr-0080-task-012-language-frame.md) V1 passed on 2026-11-05. The close of the run removes this line.
- [Packet 014](tasks/adr-0080-task-014-reader-keeps-each-frame.md) V1 passed on 2026-11-05. The run record owns the result, and the close of the run removes this line.
- [Packet 013](tasks/adr-0080-task-013-mp4-date-shares-one-value.md) V1: an MP4 file settles after one write. The run record also asks for the format of the two files of the observation of 2026-10-16.
- [ADR 0076 packet 011](tasks/adr-0076-task-011-new-reader-reads-each-feed-once.md) V1 and V2: the first Check RSS of a feed reads its full document, and its license reaches the "Update n files" list.

The second [agent walk of 2026-11-17](reviews/agent-walk-2026-11-17.md#second-walk) passed packets 007, 008, 010 and 013 and ADR 0076 packet 011 on a local copy of a real feed. A person must still check a real remixer credit, a real MP4 file and the compare rows of packet 010.

## 11. ADR 0089 The Shell - Implemented 2026-11-06

Open - packets 001 to 015 are implemented, and the mechanical suite is Green. No person checked these packets. The [run record](reviews/orchestrated-run-2026-11-05-shell.md) lists the decisions and seven questions for the operator.

- Owner: the "Visual" list of each packet file in `docs/tasks/adr-0089-task-*.md`. Each check runs in Light and Dark, at normal width and below 940 px.
- Run: do "Before A Migration" of the Operator Run Sheet, then start the app:

```bash
cd ~/build/HitIt
cargo build --bin hitit
target/debug/hitit &
```

  The checks change no stored data, apart from the checks of packets 006, 007 and 010, and their cleanup in those packets. Use `v4w 1440 845` and `v4w 900 845` for the two widths, and Settings, Theme for Light and Dark.
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

The [agent walk of 2026-11-17](reviews/agent-walk-2026-11-17.md) passed most checks of packets 001 to 015 in Light and Dark at 1440 and 900 px. **Packet 015 V3 fails**: a long value in the sheet ends in an ellipsis and does not wrap. The record lists the open checks, the evidence for questions 1 and 3 to 7 and the defects that the walk found.

The [agent walk of 2026-11-18](reviews/agent-walk-2026-11-18.md) passed most open items of packets 002 to 014 in Light at 1440 px. It passed most of them in Dark at 900 px too. **Packet 003 V3 fails at 900 px**: the section shortcuts stop on Show and Settings. The pinned list bar of ADR 0090 task 002 passed in Dark at 1440 px. A Space closes the New Playlist popover.

## 12. musicindex ADR 0053 The List Controls - 2026-11-08

Open - the packets below are implemented, and the mechanical suite is Green. The main agent checked each packet on its own Xvfb display in Light and Dark. No person checked them. The [phase plan](plans/musicindex-adr-0053-list-controls-phase-plan.md) owns the packets.

- Run: do "Before A Migration" of the Operator Run Sheet, then start the app:

```bash
cd ~/build/HitIt
cargo build --bin hitit
target/debug/hitit &
```

  The checks change no stored data, apart from migration 24 and the playlist moves of packet 3. Use these steps to open each page:

  | Item | How to open it |
  |---|---|
  | Packet 1 | Click the sidebar rows Releases, Artists and Public playlists. Select "All". Open the sort menu |
  | Packet 2 | Push Ctrl+F, type `haleen` and push Enter. Open the sort menu |
  | Packet 4 | Click Releases, then the row "Advocateworldwide · 6 singles". Use `v4w 900 845`, `v4w 420 845` and `v4w 300 845` |
  | Packet 5 | Click Releases and open "Reward". For the public playlist, click Public playlists, then "Boostagram Ball Playlist 1 to 25" |
  | Packet 6 | On the release page "Reward", click the artist link "Advocateworldwide" |
  | Packet 7 | Use item 5 of "What The Agent Could Not Run" to find names with numbers. Open a Library list that shows them, and select A to Z |
  | Packets 8 and 9, R-WORD-9, musicindex ADR 0057 | Click Artists and select "All". For R-WORD-9, click the letter `#` |
  | Packet 3 and ADR 0092 | Click My playlists. Push Tab until a row shows the focus |
  | The search pins | Click the empty search field. For `kind:foo` and `-remix`, push Ctrl+F, type the text and push Enter |
  | R-WORD-10 | Push Ctrl+F, type `Sir Libre Records` and push Enter. Open the Artists tab and the feed row. Do the same for `Crash Landing` |
  | R-WORD-9, search | Push Ctrl+F, type `olamide` and push Enter. Open the Artists tab. Do the same for `jsmovie` |
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
- [musicindex ADR 0057](adr/archive/0085-artists-by-confirmation.md) R-ARTIST-1: Artists in All, its tiles and the Artists tab of a search show three groups with a note each. The groups are "Stated artists · n", "Inferred artists" and "Artist names with no feed", in this sequence.
  - The agent Xvfb run of 2026-11-08 shows this in Light and Dark. "Show more" in the inferred group adds rows, and the other groups stay. A search with no stated artist, such as `haleen`, shows no empty group. The tiles cut their second line, for example "2 albu…" and "by na…". A person checks that the groups are clear and that the cut text is satisfactory.
- Wrong: a control that does nothing, rows in an incorrect sequence, or no selection after a sort. A sort control on a list that R-LIST-9 does not give one is also wrong.

## 13. musicindex ADR 0064 The Destructive Button - 2026-11-12

- Delete a playlist. The dialog shows "Delete" on the left, a space, and "Cancel" on the right. Cancel has the focus ring. Enter and Escape close the dialog, and the playlist stays.
  - The agent Xvfb run of 2026-11-12, Dark: the dialog shows "Delete", a space and "Cancel". Enter and Escape close it, and the playlist stays. Tab moves the focus from Cancel to Delete. The page bar and the Settings row show the new sequence.
  - Open: the focus ring of Cancel shows only after a key press. A person decides if Cancel must show the ring when the dialog opens.
- Remove a track from the Library and update the tags of a file. Each dialog has the same sequence and the same focus.
- Settings: "Use Defaults" is on the left with a space after it, and "Save" is on the right.
- A playlist page: "Delete" is the first button of the page bar, and a space follows it.
- Select rows in the Library and in a playlist. The remove buttons are the first buttons of the selection bar, and a space follows them.
- Inspect Sources with a staged edit: "Discard staged" is on the left with a space, and "Apply" is on the right.
- R-ACT-12, Settings: "Use Defaults…" is red. A click opens the dialog "Use the default settings?" with "Use Defaults" on the left, a space, and "Cancel" on the right. Cancel and Escape keep the settings. "Use Defaults" resets and saves them.
- R-ACT-10, a playlist with a track that the RSS check marks "Removed from feed": the row shows one red button "Remove…". Its menu lists "Remove from playlist", then "Remove from all playlists…". Each item does its action, and the second item opens its confirmation.
- R-ACT-11, Settings, Diagnostics, Database, the task Restore: in a wide window, "Restore database" is the last control of the column, after a divider. Make the window narrower than 760 px. The controls wrap into a band, and "Restore database" is first, with a space after it.
- Wrong: a red or destructive button at the right end of a row, or Enter that removes data.

The [agent walk of 2026-11-18](reviews/agent-walk-2026-11-18.md) passed most items in Dark. **The Library selection bar fails**: no space follows the remove button. The record gives three questions about the text of this section.

## 14. ADR 0097 The Fit Check At Two Widths - 2026-11-13

- Open HitIt at a window width of 940 px. The sidebar shows, and no label breaks inside a word (musicindex ADR 0065).
- Make the window as narrow as possible. It stops at 390 px, and the tab bar shows each tab.
- The agent test of 2026-11-13 draws the shell at 390 px and at 940 px with Figtree and finds no fault. It does not use a real window manager, so a person checks the two items above.
- Wrong: a window narrower than 390 px, or a label with a part of a word on the next line.

## 15. ADR 0090 Task 005 Each Music Column Shows The Focus - 2026-11-14

[ADR 0090 task 005](tasks/adr-0090-task-005-each-music-column-shows-the-focus.md) gives the procedure. Make the window 600 px high, and use Tab only.

- V1: Tab through the destinations list. Each focused entry is in view with its focus ring.
  - Walk of 2026-11-15: V1 did not pass. The feed area hid the focused Inbox row. The correction of 2026-11-15 is in the task file. Do V1 again with the new build.
- V2: on an Index detail pane, Tab to a button below the visible part. The pane scrolls to show it.
- V3: on a search notice page, Tab to the last button. The page scrolls to show it.
- V4: in Settings, Tab to a control below the visible part. The page scrolls to show it. The padding and the scrollbar of Settings look as before.
- Wrong: a focused control is out of view, or its focus ring is cut at the edge of the column.

The [agent walk of 2026-11-18](reviews/agent-walk-2026-11-18.md) passed V1, V2 and V4 at a window height of 600 px. V3 stays open, because no notice page with buttons was found.

## 16. ADR 0085 Task 001 The Type Label Of Known Roles - 2026-11-14

[ADR 0085 task 001](tasks/adr-0085-task-001-type-label-of-known-roles.md) gives the procedure.

- V1: the page of Sir Libre Records shows the badge and the title "Label". Its shared releases heading is "Artists on these releases".
- V2: a feed page with two or more known roles shows each role with " · ", and the dot has the color of the first role.
- V3: Inspect sources of Sir Libre Records still shows `recordLabel`.
- V4: on a feed page with an agreed role other than artist, the "Roles" fact and the "As" heading show only known roles, in the sequence artist, label, hosting, producer, host, author, network, sponsor (musicindex ADR 0066 R-ROLE-6).
- V5: a "Role not confirmed" row of a feed page or a release page shows an unknown word as `rel="…"`, for example "Sir Libre Records: label · the album: rel="recordLabel"" (R-ROLE-5 item 4).
- Wrong: a page shows "Publisher" for a feed with a known role, Inspect sources hides a role word, a "Roles" fact or an "As" heading shows an unknown word, or a "Role not confirmed" row shows an unknown word without `rel="…"`.

The [agent walk of 2026-11-18](reviews/agent-walk-2026-11-18.md) passed the badge and the title of V1, and V4 and V5, on Sir Libre Records. The heading of V1, V2 and V3 stay open.

## 17. ADR 0092 Task 001 Artists Takes The List Focus - 2026-11-14

[ADR 0092 task 001](tasks/adr-0092-task-001-artists-list-focus.md) gives the procedure. Open Artists in the scope All.

- V1: in the Rows view, Tab reaches the list in one stop. Up and Down move the focus ring across the three groups. Enter opens the artist.
  - Walk of 2026-11-15: the keys had no fault. The ring had a fault: with the focus on the sort menu, the marked row also showed a ring. The correction of 2026-11-15 is in the task file. Do V1 again with the new build.
- V2: the same in the Tiles view. The focused tile shows the focus outline and is in view.
  - Walk of 2026-11-15: Tab went to each "Show more" first, and the grid did not scroll to the tile with the cursor. The correction of 2026-11-15 is in the task file. Do V2 again with the new build.
- V3: press "Show more" with the keyboard. The focus goes to the first new row of that group, not to the sidebar.
- Wrong: Tab skips the list, the focus ring does not show, or the focus goes to the sidebar.

The [agent walk of 2026-11-18](reviews/agent-walk-2026-11-18.md) passed V1 with the focus on the sort menu, V2, and V3 in Tiles. V3 in Rows stays open. The record gives a case of two rings that is not confirmed.

## 18. ADR 0097 Task 007 The List Rows Truncate A Long Name - 2026-11-14

- Make the window 390 px wide. Open the Inbox, Search and a search with results.
- A long name shows its first words and an ellipsis on one line. No part of a word goes to the next line.
- The agent test draws these pages with names of 40 letters and finds no fault. It cannot see an ellipsis with no words (ADR 0063), so a person checks this.
- Wrong: a row that shows only "...", or a name on two lines.

## 19. ADR 0068 Task 002 The Stored Show Cue - 2026-11-14

Use a database copy with a populated playlist. Use a Linux desktop session.

1. Compile HitIt with `cargo build --locked --offline --bin hitit`.
2. Set `database_path` from `db_path` with `database_path="$(sed -n 's/^db_path = \"\(.*\)\"/\1/p' ~/.config/hitit/config.toml)"`.
3. Make a scratch directory with `scratch="$(mktemp -d)"`.
4. Make its data folder with `mkdir -p "$scratch/data/hitit"`.
5. Use `cp "$database_path" "$scratch/data/hitit/hitit.sqlite"` to write the configured database to the scratch folder.
6. Start HitIt with `XDG_CONFIG_HOME="$scratch/config" XDG_DATA_HOME="$scratch/data" XDG_CACHE_HOME="$scratch/cache" target/debug/hitit`.
7. Open My playlists. Open the populated playlist menu, then select "Load into Show Cue".
8. Open Show. Make sure that each cue row appears and no audio plays.
9. Open the playlist menu again. Make sure that it says "Replace Show Cue". Select it and make sure that Show lists the playlist.
10. Make sure that an unavailable row stays in the list and names its cause. A track with no download reads "Not downloaded". A track with a file that is not on the disk reads "File missing".
11. Make an empty playlist. Open its menu, then select "Replace Show Cue". Make sure that the status reads "The playlist has no tracks. The Show cue did not change."
12. Open Show and start the cue with the Start button. Open a playlist menu and make sure that "Replace Show Cue" is not available.
13. Stop the playback. Open the playlist menu again and make sure that "Replace Show Cue" is available.
14. Close HitIt. Use `rm -rf "$scratch"` to remove the database copy and scratch files.

- Incorrect: no cue rows appear, a missing file hides its row, the menu label is incorrect, or audio starts.

## 20. ADR 0068 Task 003 Show Plays The Cue - 2026-11-15

Use the scratch setup of section 19, steps 1 to 6. Use a playlist with three or more downloaded tracks. The check needs mpv and an audio output.

1. Open My playlists. Open the playlist menu, then select "Load into Show Cue". Make sure that no row of the playlist page has a Play control.
2. Open Show. Make sure that the first cue row has the selection border, that the play control shows the play icon, and that no audio plays.
3. Select the second cue row. Make sure that the selection border moves and no audio plays.
4. Select Start. Make sure that the second row has the current mark, that audio plays, and that the play control shows the pause icon (V1).
5. Select Pause, then Next. Make sure that the third row has the current mark and that no sound occurs. Select Resume and make sure that the third track plays from its start (V2).
6. Select Previous. Make sure that the second track plays. Make sure that Previous is not available on the first row and that Next is not available on the last row (V2).
7. Select Stop. Make sure that the audio stops and that the cue rows and their order stay (V2).
8. Select Start, then open My playlists. Press `ctrl-alt-p` two times. On macOS the key is `cmd-alt-p`. Make sure that the show pauses and then resumes (V3).
9. Close HitIt while the track plays. Start HitIt again with the command of section 19 step 6. Open Show. Make sure that the cue shows, that the play control shows the play icon, and that no audio plays (V4).
10. Select Start. Make sure that the Clear button above the cuelist does not show. Select Stop.
11. Select Clear. Make sure that a confirmation opens with the focus on Cancel. Select Clear in the confirmation. Make sure that the cuelist is empty (V5).
12. Open My playlists and a playlist menu. Make sure that it says "Load into Show Cue" (V5).
13. Press `ctrl-alt-p`. Make sure that nothing plays.

- Incorrect: a sound at step 5 before Resume, a move by two entries at the end of a track, audio after the restart, a Clear button while a track plays, or a cue row that changes its order.
- Also examine: the selection border of a cue row is a new mark. Say if the border is acceptable next to the current mark.
- Also examine: after a Next, the current mark moves and the selection border stays on its row.
- Also examine: the status line of Show uses the error color for each message, for example "Playback stopped". After a Next in pause it reads "Playing" and the track title.
- Also examine: the tooltip of the play control stays "Start Show" until the pointer leaves the control.
- Also examine: the empty cuelist reads "Queue is empty".

The agent check of 2026-11-15 ran these steps on an Xvfb display with a null audio output. It read the stored session after each step. It cannot hear audio, so each audio statement stays open for a person.

## 21. ADR 0092 Task 002 The Detail List Follows The Page Controls - 2026-11-16

[ADR 0092 task 002](tasks/adr-0092-task-002-detail-list-follows-the-page-controls.md) gives the procedure. Use a Linux desktop session with the Index and Library release pages available.

1. Compile HitIt with `cargo build --release --bin hitit`.
2. Start HitIt with `target/release/hitit`.
3. Open an Index release page with tracks.
4. Push Tab until “Reverse” has focus. Push Tab again. Make sure the first track row has the focus ring (V1).
5. Push Down. Make sure keyboard focus moves in the track list.
6. Repeat steps 3 to 5 on a Library release page with tracks (V2).
7. Close HitIt. Remove scratch folders that you created for this check.

- Incorrect: Tab skips the first track row, or Down does not move keyboard focus in the list.

The agent walk of 2026-11-16 ran steps 3 to 5 on an Xvfb display with the release binary, on the Index release "Waltz Tunes Volume 2". Tab went from "Reverse" to the first track row, Down moved the ring, and Shift+Tab went back to "Reverse". V1 passed there. The scratch Library had no tracks, so V2 stays open for a person.

## 22. ADR 0092 Task 003 Keyboard Entry Puts The Cursor On The Marked Row - 2026-11-16

[ADR 0092 task 003](tasks/adr-0092-task-003-entry-puts-the-cursor-on-the-marked-row.md) gives the procedure.

1. Compile HitIt with `cargo build --release --bin hitit`.
2. Start HitIt with `target/release/hitit`.
3. On Releases, scroll down and open a release. Push Back.
4. Scroll the list to the top, so the marked row is out of view.
5. Click "Search" in the sidebar. Push Shift+Tab. Make sure the marked row has the focus ring and is in view (V2).
6. On Artists in the Tiles view, open an artist and push Back. Put the focus in the grid with Tab or Shift+Tab. Make sure the tile of that artist has the focus outline and is in view (V1).
7. Do step 6 again in the Rows view (V2).

- Incorrect: no row has the ring after keyboard entry, or the list does not scroll to the marked row.

The agent walk of 2026-11-16 ran steps 3 to 7 on an Xvfb display with the release binary. On Releases, entry scrolled to "The Dark Plains - 00001" and showed the ring. On Artists, entry showed "2140" in the Rows view and "888childish" in the Tiles view. All three passed there.

The [agent walk of 2026-11-17](reviews/agent-walk-2026-11-17.md) could not walk V2, because the scratch Library had one release only.

## 23. ADR 0084 Task 040 The Release Page Shows Only The Rows In View - 2026-11-16

[ADR 0084 task 040](tasks/adr-0084-task-040-release-page-draws-only-the-rows-in-view.md) gives the procedure.

1. Compile HitIt with `cargo build --release --bin hitit`.
2. Start HitIt with `target/release/hitit`.
3. Search for "Global Fedpilled Industrial Complex". Open the release "GLOBAL FEDPILLED INDUSTRIAL COMPLEX".
4. Scroll from the top to the end with the mouse wheel. Make sure that the scroll has no lag and that the scrollbar thumb keeps its length (V1).
5. Push Tab until "Reverse" has the focus. Push Tab again. Push End. Make sure that the last row has the ring and is in view (V2).
6. Scroll to the middle. Open a track, then push Back. Make sure that the release page shows the same rows at the same position (V3).
7. Scroll down until the list bar goes out of view. Make sure that the pinned list bar shows. Scroll to the top and make sure that it does not show (V4).

- Incorrect: the scroll lags, the thumb changes its length, End does not show the last row, Back does not keep the position, or the pinned bar stays at the top.

The agent walk of 2026-11-16 ran steps 3 to 7 on an Xvfb display with the release binary. V1, V3 and V4 passed there, and End in V2 passed. Home in V2 did not show the first row, because of defect E in the task file. [ADR 0084 task 041](tasks/adr-0084-task-041-the-row-reveal-clears-the-pinned-bar.md) corrects defect E. The agent walk of task 041 passed Home in V2.

## 24. ADR 0084 Task 041 The Row Reveal Clears The Pinned Bar - 2026-11-16

[ADR 0084 task 041](tasks/adr-0084-task-041-the-row-reveal-clears-the-pinned-bar.md) gives the procedure.

1. Compile HitIt with `cargo build --release --bin hitit`.
2. Start HitIt with `target/release/hitit`.
3. Search for "Global Fedpilled Industrial Complex". Open the release "GLOBAL FEDPILLED INDUSTRIAL COMPLEX".
4. Push Tab until "Reverse" has the focus. Push Tab again. Make sure that row 1 has the ring and that the full ring is in view (V1).
5. Push End, then Home. Make sure that row 1 has the ring and that the full ring is below the pinned list bar (V2).
6. Push Down 14 times. Then push Up until row 1 has the ring. Make sure that each row with the ring is fully below the pinned list bar (V3).

- Incorrect: the ring is out of view after Tab, or the pinned list bar covers a part of the ring.

The agent walk of 2026-11-16 ran steps 3 to 6 on an Xvfb display with the release binary. V1, V2 and V3 passed there.

## 25. ADR 0101 Task 001 The Shared Tokens - 2026-11-16

[ADR 0101 task 001](tasks/adr-0101-task-001-shared-tokens.md) gives the procedure. Do each step in Dark and Light at M, at the usual width and at a narrow width.

- V1: Tab through a page. The focus ring is 2 px with no space between the ring and the control, and has the new focus color. It shows fully on a list row, a tile and a dialog button at 390 px and at 940 px.
- V2: an album, a track and a playlist page title is 26 px bold. An artist and a label page name is 42 px bold. No title is clipped. The title of the track inspector is also 26 px bold.
- V3: chips, fields, rows, popovers and cards show the new radii.
- V4: the text of each list and page keeps its position, with no clipped text and no overlap.
- V5: Settings at XL in a narrow window. The title and the two menus are on one line, with a small space between them.
- Incorrect: a ring is cut or has a space in it, a title has an incorrect dimension or is clipped, or text overlaps.

Agent walk, 2026-11-16, Dark only. The task file gives the details.

- V2, V4 and V5 pass.
- V3 passes for the HitIt radii. A gpui-component field keeps a corner of approximately 4 px. This is an open point of the task.
- V1 fails: a space shows between the ring and a button. [ADR 0101 task 014](tasks/adr-0101-task-014-the-button-ring-has-no-gap.md) corrects it.
- Task 014 is implemented. Its agent walk passed in Dark and Light: the ring of "Download album" is outside the fill with a gap, and the ring touches "⋯" and "Website".


The [agent walk of 2026-11-17](reviews/agent-walk-2026-11-17.md) passed V1 for a row at 1440 px, V2, V3 for the HitIt radii and V4 at 1440 and 440 px in Light. At XL and 390 px the page title shows one letter on each line, and the tab bar labels overlap. V5 shows the "Check" menu on its own line.

The [agent walk of 2026-11-18](reviews/agent-walk-2026-11-18.md) passed V1 for the dialog buttons and a page button at 390 px. A tile, a list row at 390 px and Light stay open.

## 26. ADR 0101 Task 003 The Art Radius And The Art Shadow - 2026-11-16

Task: [ADR 0101 task 003](tasks/adr-0101-task-003-art-radius.md). Do the check in Dark and in Light at M.

- V1: on Releases, compare a row image, a tile and the page header image of a release. Each corner has a curve in proportion to the image. A tile corner does not have the curve of a button.
- V2: in Light, the page header image has a larger shadow than a tile. A row image has no shadow.
- Incorrect: an image has square corners, a small image has a large curve, or a row image has a shadow.

Agent walk, 2026-11-16, Dark only. V1 passes after a correction in the task. V2 is open, because the shadow does not show clearly on the dark background.

The [agent walk of 2026-11-17](reviews/agent-walk-2026-11-17.md) passed V2 in Light.

## 27. ADR 0101 Task 005 The System Accessibility Settings - 2026-11-16

Task: [ADR 0101 task 005](tasks/adr-0101-task-005-system-accessibility-settings.md). Each step needs a change of a desktop setting while HitIt runs.

- V1: set the HitIt profile to System. Turn on high contrast in the desktop settings. HitIt changes to its high-contrast profile with no restart.
- V2: turn on reduced animation in the desktop settings. Start a download. The download icon does not turn, and its word stays.
- V3: set the HitIt profile to Dark, with high contrast on in the desktop settings. The separators and the control outlines are clear.
- Incorrect: HitIt needs a restart, an icon continues to turn, or a separator is not clear.

Agent check, 2026-11-16: the mechanical tests pass. The Xvfb display has no desktop settings, so the agent did not walk V1 to V3.

## 28. ADR 0101 Task 006 Dates In The System Language - 2026-11-16

Task: [ADR 0101 task 006](tasks/adr-0101-task-006-dates-in-the-system-language.md).

- V1: with the system language English, a release page shows its date as "Nov 12, 2026". The day is the day in your time zone.
- V2: start HitIt with `LANG=fr_FR.UTF-8`. A release page shows the date in French, for example "12 nov. 2026".
- V3: a playlist of more than one hour shows "1 h 12 min" and not "1h 12m".
- Incorrect: a date is in English with a French system language, or a total time shows "1h 12m".

Agent walk, 2026-11-16: V2 passes on the Xvfb display. V1 and V3 are open.

The [agent walk of 2026-11-17](reviews/agent-walk-2026-11-17.md) passed V3: "1 h 16 min" and "7 h 40 min". V1 stays open for the day in the time zone of the operator.

## 29. ADR 0101 Task 007 Loading States - 2026-11-16

Task: [ADR 0101 task 007](tasks/adr-0101-task-007-loading-states.md).

- V1: in My playlists, open a playlist of more than 100 tracks and scroll down fast. The gray rows that wait for their tracks show a soft light band that moves from left to right. The tracks replace them in the same places.
- V3: download an album of more than one track. Below the title "Downloading 2 of 9", a bar and a count such as "1 of 9 done" show. You can see the track of the bar in Dark and in Light.
- V4: turn on reduced animation in the desktop settings and do V1 again. The band does not move. The words stay.
- Incorrect: you cannot see the track of the bar, or the band moves with reduced animation on.

Agent walk, 2026-11-16: V1 and V3 could not be walked with the test data. V2 of the packet ("Adding…") is not possible, because the add has no busy condition.

The [agent walk of 2026-11-17](reviews/agent-walk-2026-11-17.md) could not walk V1 or V3: a local playlist has no loading rows, and only one release was downloaded.

## 30. ADR 0101 Task 002 The Page Header Glow - 2026-11-16

Task: [ADR 0101 task 002](tasks/adr-0101-task-002-header-glow.md).

- V1: in Dark and in Light, open an album page. A soft glow from the cover, with small grain, fills the top of the page. It fades into the background before the track list. Compare it with the same album on musicindex.org.
- V2: open a track page, an artist page and a label page. Each one shows the same glow.
- V3: open a page with an animated cover. The glow moves with the cover.
- V4: open an item with no image. The glow has the color of the item type.
- V5: open a page with a white cover and a page with a black cover. The header text is easy to read on each.
- V6: open a page. The page shows immediately, and the glow comes in after it.
- Incorrect: the glow has a hard edge, the text is hard to read, or the page waits for the glow.

Agent walk, 2026-11-16: V1 passes in Dark and Light. The agent did not compare it with musicindex.org. V2 passes for the artist page and the track page. V5 passes for the walked pages. V3, V4 and V6 are open.

The [agent walk of 2026-11-17](reviews/agent-walk-2026-11-17.md) found no animated cover and no item with no image for V3 and V4. V6 stays open.

## 31. ADR 0101 Task 009 Open On musicindex.org - 2026-11-16

[ADR 0101 task 009](tasks/adr-0101-task-009-open-on-musicindex.md) gives the steps. Walk them in Dark and Light at M.

- V1: a release page shows the ExternalLink icon button after RSS. Its tooltip shows immediately and reads "Open on musicindex.org".
- V2: the release and track buttons open the same item on musicindex.org.
- V3: the artist feed and name pages open the same item on musicindex.org.
- Incorrect: a link opens an incorrect item, button sequence changes, the tooltip does not show immediately, or a page without a GUID shows a button.

Agent walk, 2026-11-16: V1 passes in Light. The button shows on the artist page and the track page. V2 and V3 are open, because the test display has no browser.

## 32. ADR 0101 Task 010 The Selected Look - 2026-11-16

Task: [ADR 0101 task 010](tasks/adr-0101-task-010-selected-look.md).

- V1: do this step in Dark, Light and the two high-contrast profiles. Look at the selected sidebar item, the selected segment "All" and a checked row in the selection mode of Releases. The three items have the same look: a soft blue fill that is lighter at the top, and a thin edge.
- V2: the label of each selected item has the text color, not blue. Only the icon is blue.
- V3: push Tab until the ring is on the selected sidebar item. The full ring shows over the look. Do this again on a selected list row.
- Incorrect: two selected items have a different look, a label is blue, or the look hides a part of the ring.

Agent walk, 2026-11-16: V1 and V2 pass in Light and Dark. V3 passes for the sidebar in Dark. The high-contrast profiles and V3 on a list row are open.

The [agent walk of 2026-11-17](reviews/agent-walk-2026-11-17.md) passed V1 and V2 in both high-contrast profiles, and V3 for the sidebar and for a row in High Contrast Dark.

## 33. ADR 0101 Task 012 The Levels On The Surfaces - 2026-11-16

Task: [ADR 0101 task 012](tasks/adr-0101-task-012-light-levels-on-surfaces.md).

- V1: in Light, the page is white. The toolbar, the cards of Show and the Cuelist panel are light gray. Tell if the gray toolbar is correct.
- V2: in Light, open My playlists with one playlist or more. An open playlist entry is white in its gray group.
- V3: open the sort menu of Releases and click in the search field. Each layer has one soft shadow, in Light and in Dark.
- V4: move the pointer over a row of Releases. The hover fill shows in Light and in Dark.
- V5: the Dark theme looks as before.
- V6: make the window 300 px high and click in the search field. The field list stops at the window bottom and scrolls.
- Incorrect: a group is white in Light, or a layer has no shadow or two shadows.
- Incorrect: a hover does not show on a list row, or the field list goes below the window.

Agent walk, 2026-11-16: V1, V3, V4, V5 and V6 pass. V2 is open, because the test data has no playlist.

The [agent walk of 2026-11-17](reviews/agent-walk-2026-11-17.md) moved a file out of the scratch music folder, but the screenshot does not show the playlist row of that file. V2 stays open.

The [agent walk of 2026-11-18](reviews/agent-walk-2026-11-18.md) did not walk V2. V2 is about the unavailable row of a playlist page, which has the open entry level.

## 34. ADR 0101 Task 011 The Main Action - 2026-11-16

Task: [ADR 0101 task 011](tasks/adr-0101-task-011-main-action.md).

- V1: open an Index release page. "Download Album" is a Secondary button with a light tint, not a blue fill.
- V2: open a downloaded track. "Add to Playlist" is a Secondary button. Open the "⋯" menu. "MusicBrainz Lookup" is in the menu.
- V3: compare the header of a release page and a track page with the same page on musicindex.org. The two pages give the same weight to each button.
- Incorrect: a header button has a blue fill, or a button label does not use title case, for example "Add to playlist".

Agent walk, 2026-11-16: V1 and V2 pass in Light. V3 is open, because the test display has no browser.

## 35. musicindex ADR 0083 The Version Line - 2026-11-18

Task: [musicindex ADR 0083 task 001](tasks/musicindex-adr-0083-task-001-version-line.md).

1. Compile HitIt with `cargo build --release --bin hitit`.
2. Start HitIt with `target/release/hitit`.
3. Open Settings. Make sure that the last line of the page shows the version, for example "HitIt 0.1.0 (0dd3e95, 2026-11-18)", with "Copy" at its right (V1).
4. Drag the pointer across the line. Make sure that the text shows as selected (V2).
5. Click "Copy". Paste in a text editor. Make sure that the paste is the full line (V3).
6. Open Library and Diagnostics. Make sure that each page ends with the same line.
- Incorrect: no line, a line that overlaps "Copy", text that you cannot select, or a paste that is not the line.
- Open point for the operator: the line shows on each tab of Settings. Tell if it must show on one tab only.

Agent check, 2026-11-18: V1 to V3 pass on Xvfb in Dark and Light at 1440 px, and in Dark at 390 px.
