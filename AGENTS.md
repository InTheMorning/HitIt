# HitIt Agent Guidelines

HitIt (`hitit`) is a Linux-first GPUI desktop app for Value4Value music curation and
show production. A curator finds music, takes it into a local library, keeps
that library ready, and uses it to run or record a show.

This file describes the present. It carries no history and no superseded rule.
The [source map](docs/architecture/source-map.md) lists the owner file of each area. Read it to find a file.

Governance model: ADR 0061.

## Where The Work Stands

[ADR 0096](docs/adr/0096-the-app-name-is-hitit.md), Accepted on 2026-11-11, renames the app HitIt with the machine name `hitit`. The GitHub repository is `InTheMorning/HitIt` and the directory is `~/build/HitIt`.

The [overhaul plan](docs/plans/design-and-cleanup-overhaul-plan.md) is the current priority. It orders the
design and cleanup work in six phases. Phases 1 and 2 are complete. The design language is
[ADR 0101](docs/adr/0101-the-design-language-follows-the-shared-rules.md), Accepted on 2026-11-15. It supersedes the archived ADR 0083.

Phase 3, the tokens, is complete. Phase 4, the Library, is in progress. Its first two packets, one track page
and one album page for the Library and the Index, are complete. Phase 4A, flow and information architecture, decided the
Music structure in [ADR 0084](docs/adr/archive/0084-music-column-browser.md).

Phase 4A builds ADR 0084 in the packets of the
[ADR 0084 phase plan](docs/plans/adr-0084-music-column-browser-phase-plan.md). [ADR 0089](docs/adr/archive/0089-the-shell-destinations-and-list-pages.md) replaces its columns, destinations and Discover.
Stophammer release 0.13.0 delivered request 9 on 2026-10-21, so packets 026 and 029 wait for nothing outside this repository.
Stophammer release 0.15.0 of 2026-10-25 delivered items 1 to 7 of request 10 (Stophammer ADRs 0078 and 0080). Stophammer release 0.19.0 of 2026-11-11 delivered requests 14 and 15 and item 1 of request 11 (Stophammer ADRs 0084, 0085 and 0094). The tag v0.19.0 is at git `9fcbdb5`, and on 2026-11-11 the live API gives version 0.19.0.

Packets 001 to 009, 013 to 015, 018, 019, 036 and 037 are complete. Packets 010 to 012, 016, 017, 020 to 035 are implemented,
and so are ADR 0083 tasks 006 to 008. The run of 2026-10-20 closed on 2026-10-23. Its unwalked visual checks stay open in
[pending human checks](docs/pending-human-checks.md), and the [run record](docs/reviews/orchestrated-run-2026-10-20.md) lists them.

[ADR 0086](docs/adr/archive/0086-new-releases-singles-window.md), Accepted on 2026-10-23, groups the singles of one artist in New releases by a window.
[ADR 0087](docs/adr/archive/0087-search-fields-and-live-results.md), Accepted on 2026-10-23, gives search its fields, a field list and live results.
The [remaining order](docs/plans/design-and-cleanup-overhaul-plan.md#remaining-order---2026-10-23)
of the overhaul plan is the order of the next work. The visual overhaul comes before each new screen.

The fix packets of the walk of 2026-10-24 are implemented on 2026-10-25. They are ADR 0084 packets 038 and 039 and ADR 0087 packets 009 and 010. Their visual checks are open in sections 8 and 9 of [pending human checks](docs/pending-human-checks.md). The next step starts after those checks.

Phase 4 and 5 packets carry the
[visual requirements moved from pending checks](docs/plans/design-and-cleanup-overhaul-plan.md#visual-requirements-moved-from-pending-checks---2026-10-07).

Each file in `docs/tasks/` is open: it is Ready, or it waits for an operator check. Finished tasks are in
`docs/tasks/archive/`. Each file in `docs/plans/` is an active plan. Finished plans are in `docs/plans/archive/`.

The operator resumed visual checks on 2026-10-02. [Pending human checks](docs/pending-human-checks.md)
holds each open check. Its [operator run sheet](docs/pending-human-checks.md#operator-run-sheet---2026-11-09) gives the commands, the needs that the agent could not supply and the open operator decisions. Walk a new check right after its packet. The procedures in `docs/runbooks/` stay
regression checks.

The run of 2026-10-23 implemented ADR 0086 packets 001 to 005 and ADR 0087 packets 001 to 005. It closed on 2026-10-24. Its unwalked checks stay open in
[pending human checks](docs/pending-human-checks.md), and the [run record](docs/reviews/orchestrated-run-2026-10-23.md) lists them.

The run of 2026-10-24 implemented ADR 0084 packets 035 to 037, ADR 0087 packets 007 and 008 and ADR 0059 packet 019. It closed on 2026-10-24 after its walk.
Its open checks stay in [pending human checks](docs/pending-human-checks.md), and the [run record](docs/reviews/orchestrated-run-2026-10-24.md) lists them.

On 2026-11-14 the operator put the defect fixes first: [ADR 0090 task 005](docs/tasks/adr-0090-task-005-each-music-column-shows-the-focus.md), [ADR 0092 task 001](docs/tasks/adr-0092-task-001-artists-list-focus.md) and [ADR 0085 task 001](docs/tasks/adr-0085-task-001-type-label-of-known-roles.md) come before the ADR 0068 packets.
On 2026-11-15 the operator opened step 4 of the overhaul plan, Phase 7, the visual overhaul. [ADR 0101](docs/adr/0101-the-design-language-follows-the-shared-rules.md) adopts the shared rules. [ADR 0102](docs/adr/0102-the-arrow-keys-move-in-a-grid-of-tiles.md), Accepted on 2026-11-15, uses musicindex ADR 0078 R-GRIDKEY-1 for the arrow keys of the Tiles views. Its [task 001](docs/tasks/adr-0102-task-001-tiles-arrow-keys.md) waits until the operator opens it. It comes after the walk of sections 15 to 17 of [pending human checks](docs/pending-human-checks.md), because it corrects a fault of that walk. The ADR 0101 token packet, [task 001](docs/tasks/adr-0101-task-001-shared-tokens.md), comes after the three defect fixes and after ADR 0102 task 001. The surface packets of ADR 0101 Decision 12 are tasks 002 to 012. They follow the token packet. The [ADR 0101 phase plan](docs/plans/adr-0101-design-language-phase-plan.md) gives their sequence, the place of the Ready packets of other ADRs and the open points for the operator.
Ready packets include
[ADR 0064 task 002](docs/tasks/adr-0064-task-002-repair-report-surface.md) and
[ADR 0069 task 002](docs/tasks/adr-0069-task-002-shared-guarded-editor.md).
[ADR 0093](docs/adr/0093-settings-has-a-tab-row-and-one-save-bar.md), Accepted on 2026-11-10, applies the Settings layout of musicindex ADR 0059. [ADR 0093 task 001](docs/tasks/adr-0093-task-001-settings-tab-row-and-rows.md), the tab row and the rows, is Ready. ADR 0069 task 002 delivers its Save bar, its marks and its leave question. The operator decides the priority.
[ADR 0094](docs/adr/0094-one-fold-header.md), Accepted on 2026-11-10, applies the fold header of musicindex ADR 0060. [ADR 0094 task 001](docs/tasks/adr-0094-task-001-fold-header-composite.md) and [task 002](docs/tasks/adr-0094-task-002-singles-group-row-keys.md), the keys of the singles group row, are Ready.
The [run of 2026-11-05](docs/reviews/orchestrated-run-2026-11-05.md) implemented [ADR 0080 packets 006 to 016](docs/plans/adr-0080-tag-frames-phase-plan.md), the tag frames of the review of 2026-11-04, and [ADR 0076 packet 011](docs/tasks/adr-0076-task-011-new-reader-reads-each-feed-once.md). The open checks of these packets are in section 10 of [pending human checks](docs/pending-human-checks.md). [Packet 014](docs/tasks/adr-0080-task-014-reader-keeps-each-frame.md) corrects a reader fault that the walk found. A write of 143 Library files ran on 2026-11-05. The next full write waits for more playlist checks, as the operator decided (open point 3 of the run record). 26 HeyCitizen files hold `TPUB: HeyCitizen`. The operator confirmed that this value is correct. The live HeyCitizen feeds state only `rel="artist"`.
[ADR 0088](docs/adr/0088-a-feed-move-follows-musicindex.md), the feed move, is Accepted on 2026-11-11. A stored feed follows the MusicIndex `feed_url` when the RSS document proves the same `podcast:guid`. [ADR 0088 task 001](docs/tasks/adr-0088-task-001-rss-check-follows-a-move.md) is Ready.
[ADR 0089](docs/adr/archive/0089-the-shell-destinations-and-list-pages.md), the shell, is Accepted on 2026-11-05. The [shell run](docs/reviews/orchestrated-run-2026-11-05-shell.md) implemented packets 001 to 015 of its [phase plan](docs/plans/adr-0089-shell-phase-plan.md) on 2026-11-06. Their visual checks and the questions of the run record are open in section 11 of [pending human checks](docs/pending-human-checks.md). Packet 016, the now-playing panel and the row action Play, waits for the packets of ADR 0068, Accepted on 2026-11-11. The new tokens of the overhaul plan start after the walk.

[musicindex ADR 0042](../musicindex/docs/adr/0042-a-button-with-a-label-has-a-fill.md), the neutral button, is implemented here on 2026-11-06 in the [run record](docs/reviews/orchestrated-run-2026-11-06-neutral-button.md). The operator passed its check on 2026-11-07.

[musicindex ADR 0053](../musicindex/docs/adr/0053-which-lists-have-a-sort-a-reverse-and-a-view.md), the sorts, the direction and the views of each list, is built in the packets of its [phase plan](docs/plans/musicindex-adr-0053-list-controls-phase-plan.md). Packets 1 to 9 are implemented on 2026-11-08, packet 6 with ADR 0090 task 004: Releases All, Public playlists and Artists All reverse through the node when the node lists the capability `sort`, the results page and the saved search page sort by Relevance, A to Z and Newest, the group pages have a sort, a reverse and views in the list bar, a release page and a public playlist page reverse their tracks or entries from 8 rows, each release section of a feed page and a name level page has its own sort, reverse and views from 8 rows, a local A to Z compares names in the language of the reader with numbers as numbers, Artists has rows and tiles and the sort Newest in each scope, and My playlists has the stored Curator Order of the [ADR 0089 amendment of 2026-11-08](docs/adr/archive/0089-the-shell-destinations-and-list-pages.md#amendment---2026-11-08-the-stored-curator-order). Their checks are open in section 12 of [pending human checks](docs/pending-human-checks.md).

[musicindex ADR 0054](../musicindex/docs/adr/0054-the-sort-menu-names-both-directions.md), Accepted on 2026-11-08, replaces the reverse icon of musicindex ADR 0043 R-LIST-7. It is implemented on 2026-11-08: each sort menu lists the usual and the reversed entry of each sort that can reverse, a list with one fixed sequence has a text toggle "Reverse", and the "Add to playlist" menus follow the Curator Order. [ADR 0092](docs/adr/0092-the-keys-of-a-list-row.md), the keys of a list row, is implemented on 2026-11-08. It uses musicindex ADR 0061 R-LISTKEY-1. Artists is one Tab stop in its Rows view and its Tiles view ([ADR 0092 task 001](docs/tasks/adr-0092-task-001-artists-list-focus.md)). Public playlists, Live and Search are not one Tab stop yet, and a subsequent task gives them the list focus. musicindex ADR 0056, the accent glow at the top of the sidebar, is implemented on 2026-11-08. The shared releases section of a feed page uses the heading, the note and the row text of musicindex ADR 0023 R-WORD-10. An Index row of Artists All and a feed row of the search Artists tab count the releases of each kind from `release_track_counts`, for example "14 albums · 38 singles" (musicindex ADR 0023 R-WORD-9). A row with no valid map gives the total. Their visual checks are open in section 12 of [pending human checks](docs/pending-human-checks.md).

[musicindex ADR 0064](../musicindex/docs/adr/0064-a-destructive-button-is-never-the-rightmost-button.md), Accepted on 2026-11-12, is implemented here on 2026-11-12. In each horizontal row of buttons, list entries included, the destructive button is leftmost, a larger space follows it, and the safe action is rightmost. A destructive confirmation gives the first focus to Cancel, and Enter and Escape close it with no action. The amendment in musicindex commit `ab11758` adds R-ACT-10 to R-ACT-12. A playlist row of a removed track shows one "Remove…" menu button. A maintenance column puts its destructive controls last, and its narrow band puts them first. "Use Defaults…" in Settings is red and asks for confirmation. [ADR 0101](docs/adr/0101-the-design-language-follows-the-shared-rules.md) Decision 5 and the [ADR 0093](docs/adr/0093-settings-has-a-tab-row-and-one-save-bar.md) amendment of 2026-11-12 record this and their open items. The visual checks are open in section 13 of [pending human checks](docs/pending-human-checks.md).

[ADR 0098](docs/adr/0098-the-music-shell-pages-lists-and-search.md), Accepted on 2026-11-14, restates the live rules of ADRs 0084, 0085, 0086, 0087, 0089 and 0090. It makes no new decision. Those six ADRs are in `docs/adr/archive/`. The item map of ADR 0098 gives the new place of each item that code, tests and documents cite.

[ADR 0097](docs/adr/0097-the-fit-check-at-two-widths.md), Accepted on 2026-11-13, applies musicindex ADR 0065. The narrowest window width is 390 px. GPUI layout tests with Figtree check the fit at 390 px and at 940 px. Phases 1 and 2 of its [phase plan](docs/plans/adr-0097-fit-check-phase-plan.md) are complete: `src/app/fit_check/` checks the shell, the list bar and each list page, `src/library/fit_fixtures.rs` gives the fixture rows, and `src/ui/primitives/fit_probe.rs` finds the faults. [ADR 0097 task 007](docs/tasks/adr-0097-task-007-list-rows-truncate-long-names.md) truncates a long name in the rows of the Inbox, Search and the search results, and its visual check is open in section 18 of [pending human checks](docs/pending-human-checks.md). Phases 3 to 6 are open, and their packets, [ADR 0097 tasks 003 to 006](docs/tasks/adr-0097-task-003-item-pages-fit.md), are Ready. A new text or control primitive records a fit probe.

[ADR 0087 packet 011](docs/tasks/archive/adr-0087-task-011-index-receives-the-fields.md) sends each search field to the Index as its own filter, and ADR 0098 Known Differences list the differences from the Library. [Responsiveness task 001](docs/tasks/archive/responsiveness-task-001-no-ui-thread-database-wait.md) removes the hang of a download or a removal. The operator confirmed both on 2026-11-06.

[ADR 0091](docs/adr/0091-search-parser-moves-to-the-shared-crate.md), Accepted on 2026-11-07, moves the search parser, the Index plan and the kind rule to the `musicindex-search` crate. The crate is pinned at `4761148`. It gives the placeholder of each text field and `quote_field_value` for a runnable example (musicindex ADR 0058). It gives the rows of the field list (`field_list_rows`), the note of a text that sends no Index request (`search_note`) and the note of a tab that gets no request (`scope_note`), from musicindex ADR 0052 R-QUERY-7 and R-QUERY-8. HitIt adds only its sentence about its Library rows. The crate builds the `kind:` requests, the text of `q` and the sort parameters. HitIt builds no `kind:` mapping and no sort parameter of its own. musicindex ADR 0062 owns the `host:` field. The crate sends `host` and `not_host`, and HitIt filters its Library rows with `host_keeps_row` and the host of the feed URL. It passes the crate's sort parameters for a sort that the person selected, when the node lists `sort`.

[ADR 0100](docs/adr/0100-a-removal-moves-the-audio-file-to-the-audio-cache.md), Accepted on 2026-11-15, moves the audio file of a removed track from the music folder to the audio cache, `~/.cache/hitit/audio`. The music folder holds only Library tracks. The table `cached_files` holds the cache files, and a `local_files` row is a file below `music_dir`. [ADR 0100 task 001](docs/tasks/adr-0100-task-001-removal-moves-the-file-to-the-audio-cache.md) implements it on 2026-11-15, and the operator passed its visual checks on 2026-11-15.

Known open defects:

- The live status shows one song that does not change. Playback work waits for
  [ADR 0068](docs/adr/0068-show-cue-and-audition-isolation.md), which is Accepted on 2026-11-11. Its five packets are written in its [phase plan](docs/plans/adr-0068-show-cue-and-audition-phase-plan.md). [Packet 001](docs/tasks/adr-0068-task-001-mpv-ipc-read-error.md) fixes the mpv IPC read error on 2026-11-14: a read timeout is not an error before a reply deadline of 10 s. [Packet 002](docs/tasks/adr-0068-task-002-stored-show-cue.md), the stored Show cue, is implemented on 2026-11-14, and its visual checks are open in section 19 of [pending human checks](docs/pending-human-checks.md). [Packet 003](docs/tasks/adr-0068-task-003-show-plays-the-cue.md), Show plays the cue, is implemented on 2026-11-15, and its visual checks are open in section 20. Only typed Show commands start program playback, and no page of Music starts it. Packets 004 and 005 are Ready.
- The fix for the MP4 `TYER` difference in "Update n files" is implemented. [ADR 0080 packet 013](docs/tasks/adr-0080-task-013-mp4-date-shares-one-value.md)
  owns the operator check, which is open.
- Each Music column scrolls with a held handle, so a focused control comes into view ([ADR 0098](docs/adr/0098-the-music-shell-pages-lists-and-search.md) Invariants). [ADR 0090 task 005](docs/tasks/adr-0090-task-005-each-music-column-shows-the-focus.md) implements this on 2026-11-14. The defect stays in this list until the operator passes its visual checks in section 15 of [pending human checks](docs/pending-human-checks.md).
- A feed page with two or more roles shows "Publisher". [ADR 0098](docs/adr/0098-the-music-shell-pages-lists-and-search.md) Decision 11.7 lists each role. [ADR 0085 task 001](docs/tasks/adr-0085-task-001-type-label-of-known-roles.md) implements the fix on 2026-11-14. The defect stays in this list until the operator passes the checks in section 16 of [pending human checks](docs/pending-human-checks.md).
- Sir Libre Records shows the badge "Publisher", because one side gives `rel="recordLabel"`. On 2026-11-14 the operator decided that the badge ignores an unknown role word. Decision 1 of the [operator run sheet](docs/pending-human-checks.md#operator-run-sheet---2026-11-09) records it. [musicindex ADR 0066](../musicindex/docs/adr/0066-a-role-word-that-is-not-known-is-ignored.md) owns the rule. ADR 0085 task 001 implements the fix on 2026-11-14, and its checks are open in section 16.
- Tab does not go to an artist tile on Artists, and after "Show more" the focus goes to the sidebar. [ADR 0092 task 001](docs/tasks/adr-0092-task-001-artists-list-focus.md) implements the fix on 2026-11-14. The defect stays in this list until the operator passes its visual checks in section 17 of [pending human checks](docs/pending-human-checks.md).

Stored facts that agents need:

- Schema version 27 is current. The migration registry of ADR 0016 owns each schema change.
- `tests/fixtures/musicindex-openapi-0.19.0.json` is the stored MusicIndex contract. Since 2026-11-08 it holds the live contract of Stophammer candidate v0.19.0-rc.7 (git `398d10f`). After each Stophammer
  release, replace it in the same change as the decode changes (ADR 0075 section 6).
- [Open Stophammer requests](docs/plans/hitit-open-requests.md) has an equal copy in the Stophammer
  repository. Update both copies together.
- `docs/plans/broadcast-chain-delivery-order.md` is the only cross-repository order. Read it before a session
  on broadcast work. When a real show is scheduled, publisher show-log task 001 takes priority.
- Do not delete the evidence fixture `/tmp/v4vmm-governance.ie6k8TQf`. Its cleanup is unconfirmed.

## Current Design Philosophy

**One sidebar.** It lists Search, Releases, Artists, My playlists, Public
playlists and Live, then Show, Inbox and Settings. Music is curation and holds
index results and the library together. `Show` is broadcasting. Search is a
toolbar command on each page, and the Search destination holds the saved
searches. ADR 0098 Decision 1.

**One list bar above the rows.** A list of selectable rows has one bar directly above its rows, with selection at its left and sort and view at its right. ADR 0098 Decision 13. The singles group page and the window group page have it, and the [phase plan](docs/plans/adr-0090-list-bar-phase-plan.md) moves each other page.

**Curation shows nothing operational.** While nothing plays and no show is
active, `Music` and `Settings` show no queue, no transport, and no broadcast
status. A surface that cannot act is absent, not disabled. ADR 0098 Decision 18.

**One page for each item.** A list is a page, and an item opens over it (musicindex ADR 0080, ADR 0098 Decision 6). Each release, track, artist and playlist has one
page, whatever its origin. The Library is a state that each page and list shows, not a place. ADR 0098 Decision 6.

**Artists, not publishers.** A feed shows its role word, not the RSS word. The word rules are musicindex ADR 0033
R-WORD-3 and R-WORD-4, and ADR 0098 Decision 11 applies them here. A feed that albums name through a confirmed link is an artist, confirmed or unverified.
Each artist list shows the stated artists, then the inferred artists, then the names with no feed (musicindex ADR 0057 R-ARTIST-1, ADR 0098 Decision 11.8).

**Shared interface rules come from musicindex.** The musicindex ADRs of ADR 0101 Decision 11 own the interface rules that
musicindex.org, Studio and HitIt share. ADR 0101 adopts them and gives the HitIt design language. A change to a
shared rule needs a superseding record in `../musicindex`.

**Provenance first.** Provenance is the element in the RSS document that
asserted a value: the channel, the item, or a person. MusicIndex is a cache of
RSS, never a peer source. Embedded tags and MusicBrainz stay separate. Never
collapse them into one inferred truth. Placeholder-looking source text is a
source-boundary problem, never a renderer problem. ADR 0075 Decision I.

**Only Lightning payment routes.** The RSS path keeps the first `podcast:value` block whose type is
`lightning` or absent, and only `lnaddress` and valid `node` recipients. The stored route, the file frame and the
drop file hold no route of a different payment system.
[ADR 0099](docs/adr/0099-the-rss-path-keeps-only-lightning-payment-routes.md), with stophammer ADR 0098.
Migration 26 corrects the stored routes.

**The broadcast chain runs when this app is closed.** This app is a control
surface and a status display, never a required part of the chain. The built-in
player is the one exception.

**Real time is canonical.** This app, the encoder, and the relay sender all
present real time. Delivery delay serves the live path only and never reaches a
recording or an interface.

**Current-view state must update in place.** A mutation refreshes the mounted view.
Never require navigating away and back to reveal a result.

## The Durable Set

These are permanent. They describe how this project is built, not what any
screen looks like. Adding to this set requires an ADR. ADR 0061.

**Renderer portability.** View models carry no renderer type. Presentation
facts, default labels, availability, and command intent live in
`src/view_models` or `src/views.rs`. Reusable chrome, row layout, and
interaction geometry live in `src/ui/primitives` or `src/ui/composites`.
Screens stay thin: they compose views, resolve images, wire callbacks, manage
focus, and dispatch commands.

The reason is portability. These rules are what let the interface move to or
from GPUI without rewriting product logic. A violation welds product logic to
one renderer.

**Element hierarchy.** Every element has a defined place. Title, subtitle,
metadata, state, and actions have predictable placement, weight, and
visibility. Nothing is positioned ad hoc.

**Apple HIG structure.** Predictable hierarchy and disclosure, clear state,
comfortable density, accessibility, and cautious destructive actions. HIG is
structural guidance, not visual imitation.

**Token discipline.** Size, spacing, color, typography, icons, and roles come
from named tokens. No raw literal and no glyph string in a renderer. Never rely
on color alone.

**Typed action state.** Every action carries typed availability and an
accessibility label before it renders. No renderer decides whether a control is
enabled.

**UI change acceptance gate.** A user-visible UI change is accepted only when
the shared owner, view model, token path, and verification path are all clear.
ADR 0033 and ADR 0061.

**No isolated visual tweaks.** Fix repeated visual affordances at their shared
primitive, composite, shell, or view-model owner. A renderer patch is only for
screen-local composition.

**Button and action discipline.** Buttons render from typed action state,
named icons, accessibility labels, and named control styles. A renderer does
not invent action availability.

**Workflow-first requirement.** UI structure follows the curator workflow
surface first, then the component hierarchy. Component availability does not
earn screen space by itself.

**HIG product polish is separate from restructuring.** HIG product-completeness
gaps live in the product polish backlog. Do not mix them into structural
rework unless the active ADR says to do so.

## Working Rules

**ADR first.** Do not implement an architectural change before an ADR records
it. Use the status vocabulary of ADR 0057.

**One phase for each session.** Complete a phase, verify it, then start a fresh
session. Do not chain phases.

**Zero warnings.** Every commit passes `cargo check`, `cargo fmt --check`, and
`cargo clippy --all-targets -- -D warnings`.

**Silence success.** Report an error. When a check passes, say "Green" and
continue.

**Write reports for the operator.** When writing reports, log messages, or
status text, name the subject, action, and object. Explain what happened and
what the app did with the result. Name the event, endpoint, service, or file
being checked instead of an internal phase such as "liveness". Timestamp
reported actions and observations using their actual recorded times. Do not
invent times during rendering or turn an unanswered question into a fact.
Omit idle actions and repeated filler. Put technical details after the useful
explanation. Short badges may stay concise, but their surrounding report must
make the subject and consequence clear. ADRs 0059/0063 own Event report data.

**Every fix gets a guard.** A user-confirmed fix is incomplete without a test,
an architecture guard, or a documented manual check that blocks the same class
of regression.

**Reconcile human gates by meaning.** Read ADR status sections and review gate
statements, including prose. Checkbox syntax is not a gate inventory. Before
listing an inherited check, retire replaced requirements with the superseding
ADR named, and preserve each surviving requirement in the pending-check index.
Scope a claim of no open checks to the packet or ADR actually reviewed.

**A guard names its class and its ADR.** A durable guard enforces the set
above. A situational guard cites one ADR and is deleted when that ADR is
superseded. When the class is unclear, choose situational.

**Prose that a guard replaces is deleted** in the same change.

**Delete dead code.** Code no composition root reaches is removed, not parked.
Copy any pattern worth keeping into the live surface first.

**Visual proof.** A user-visible layout, hierarchy, or presentation fix is not
fixed until a person inspects it in the running app.

**Make your own visual check first.** Before you ask a person to check a
user-visible result, the main agent runs the app on its own Xvfb display and
judges screenshots. Use scratch `XDG_CONFIG_HOME`, `XDG_DATA_HOME` and
`XDG_CACHE_HOME` folders, so the run cannot touch the real Library. Use this
display:

```bash
Xvfb :77 -screen 0 1440x900x24 -nolisten tcp -extension GLX &
DISPLAY=:77 openbox &
DISPLAY=:77 env -u WAYLAND_DISPLAY target/debug/hitit
```

The agent environment sets `WAYLAND_DISPLAY`. Without `env -u WAYLAND_DISPLAY`,
the app tries Wayland and stops with `NoCompositor`.

In auto mode, the Bash call that starts the app sets
`allowed_domains: ["api.musicindex.org"]`. Without it, the sandbox proxy
refuses each MusicIndex request, and the Index pages show errors.

Openbox makes the window active. Without a window manager, the search field list does not open. Never send input to the operator's display. A packet worker does not run the
app. An agent result does not pass a visual gate.

**Write operator instructions for what stays.** End the report with an
`Operator visual check` section that holds only the steps that need a person,
and that a person can follow at a terminal in a desktop session:

- numbered steps, including the commands that reach the state
- what to look at, and what would count as wrong
- any hardware or system state the check needs, named plainly, such as a failed
  unit, an unreachable host, or an installed binary
- the cleanup that undoes any state the steps created

Leave the gate open in the packet `Status:` line, in
`docs/plans/broadcast-chain-delivery-order.md`, and in
`docs/pending-human-checks.md`, which holds every check that is open today. A
gate that no person has walked is never met, and a green mechanical suite is not
evidence about it.

**An acceptance criterion states how it is checked.** It is mechanical, proved
by a test and phrased at the layer that owns it, or it is visual and listed
separately. `The section shows six states` is untestable. `The view model
exposes six states` is the same requirement where a test can reach it. A
criterion that is neither passes by omission.

**Collect responsiveness concerns.** When you find a slow response, or background work with no
visible feedback, such as a download or a fetch with no progress, loading state or motion, add it
to "Throughout: Responsiveness And Background Feedback" in the
[overhaul plan](docs/plans/design-and-cleanup-overhaul-plan.md), with its date and its owner file.
Do not fix it inside an unrelated packet unless the operator asks.

**Column text does not call `truncate()`.** ADR 0063 decides this, and
`docs/troubleshooting/column-text-truncation.md` explains why. Read them before
you style stacked text.

## Build, Test, Lint

After running tests, run `cargo build --bin hitit` before handing a desktop
binary to the operator. `cargo test` can replace `target/debug/hitit` with a
binary linked to GPUI test-support and its synchronous drawing loop. The startup
fixture's `run` command rebuilds the normal binary before opening it.

```bash
cargo build                               # Debug build
cargo build --release                     # Production build: target/release/hitit
cargo test                                # All tests
cargo test --test architecture_tests      # The guards
cargo fmt -- --check                      # Check formatting
cargo clippy --all-targets -- -D warnings # Strict clippy, every target
```

The agent runs build, test, clippy and format check locally before a commit.
The guard `shared_adr_link_check_passes` runs the ADR link check of the
project-baseline skill, section "A Binding Rule Lives In A Decision Record".
It gives `--bar-from 0100` to the check, so each current ADR from ADR 0100 has
one `- Bar:` line.
The check is in the shared skill folder, so the guard does not run in CI.
`.github/workflows/ci.yml` runs the "Zero warnings" checks and `cargo test` on
each push and each pull request. It does not replace the local checks. A
change to the "Zero warnings" commands also changes that file.

`packaging/arch/PKGBUILD` builds the `hitit-git` Arch Linux package from
GitHub. The [Arch package runbook](docs/runbooks/arch-package.md) tells how to
build it. A new run-time library, program or file also changes the PKGBUILD.

## Conventions

**Naming.** PascalCase types and enums, snake_case functions, variables and
modules, SCREAMING_SNAKE_CASE constants.

**Imports.** Group std, external, then `crate::`. No wildcard outside tests.

**Errors.** `anyhow::Result` with `.context(...)` in services, the CLI, and I/O.
Typed enums in the application layer and the view models, because callers match
on variants. A view model returns a display state, never a transport error.

**Async.** Background work uses the ADR 0040 runtime. Actors live in
`src/runtime/`, and `src/runtime/playback_polling.rs` is the reference. Never
call `cx.spawn` from a screen. Blocking database access uses
`Arc<Mutex<Connection>>`.

**Logging.** This crate has no `tracing` dependency. The CLI prints JSON on
stdout and errors on stderr. UI code reports failure through view-model display
state.

**Docs.** Module-level `//!` comments naming the owning ADR. Most modules carry
`#![warn(clippy::pedantic)]`. An opt-out uses `#![expect(...)]` with a reason.

**Serde.** `#[serde(default)]` for optional config so an older `config.toml`
keeps loading. `rename_all = "snake_case"` for enums.

**Tests.** Unit tests in `#[cfg(test)] mod tests` beside the code.
`tests/architecture_tests.rs` is the only integration file and holds the
guards. There is no shared test helper module.

**Secrets.** Broadcaster tokens are files with mode `0600`. Never in the
database, a log line, a command argument, or `Debug` output. Construct blocking
HTTP clients only in `src/http_client.rs`. Treat RSS and MusicIndex responses as
untrusted input.

## What Binds You

`docs/adr/README.md` lists every current ADR with its status and scope. It is
the answer to what binds a change today.

`docs/adr/archive/` holds superseded and fully guarded decisions. Read it for
research, never to find a live rule.

This file owns how an agent works. An ADR owns how the code is shaped. The two
do not overlap, and neither one holds the other's rules.

- **Agent behaviour lives here.** What to run, what never to run, what a report
  must contain, how to leave a gate open. These need no ADR, and this file is
  their owner.
- **Code shape lives in an ADR.** Layering, ownership, element hierarchy, a
  contract between two modules, how a surface is built. This file may point at
  such a rule and name the ADR that decides it. It never states one as its own.
