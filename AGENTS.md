# HitIt Agent Guidelines

HitIt (`hitit`) is a Linux-first GPUI desktop app for Value4Value music curation and
show production. A curator finds music, takes it into a local library, keeps
that library ready, and uses it to run or record a show.

This file describes the present. It carries no history and no superseded rule.
The [source map](docs/architecture/source-map.md) lists the owner file of each area. Read it to find a file.

Governance model: ADR 0061.

## Where The Work Stands

The app name is HitIt, with the machine name `hitit`
([ADR 0096](docs/adr/0096-the-app-name-is-hitit.md)). The GitHub repository is
`InTheMorning/HitIt` and the directory is `~/build/HitIt`.

### Order Of The Work

- The [overhaul plan](docs/plans/design-and-cleanup-overhaul-plan.md) is the
  current priority. Its
  [remaining order](docs/plans/design-and-cleanup-overhaul-plan.md#remaining-order---2026-10-23)
  gives the order of the next work. The visual overhaul comes before each new
  screen. Phases 1 to 3 are complete. Phase 4, the Library, and Phase 7, the
  visual overhaul, are open.
- Phase 4 and 5 packets carry the
  [visual requirements moved from pending checks](docs/plans/design-and-cleanup-overhaul-plan.md#visual-requirements-moved-from-pending-checks---2026-10-07).
- The defect fixes of ADR 0090 task 005, ADR 0092 task 001 and ADR 0085 task
  001 come before the ADR 0068 packets (operator choice).
- The [ADR 0101 phase plan](docs/plans/adr-0101-design-language-phase-plan.md)
  gives the position of the Ready packets of other ADRs and the open points for
  the operator.

### Documents And Checks

- Each file in `docs/tasks/` is open: it is Ready, or it waits for an operator
  check. Finished tasks are in `docs/tasks/archive/`. Each file in
  `docs/plans/` is an active plan. Finished plans are in `docs/plans/archive/`.
- [Pending human checks](docs/pending-human-checks.md) holds each open check.
  Its [operator run sheet](docs/pending-human-checks.md#operator-run-sheet---2026-11-09)
  gives the commands, the needs that the agent could not supply and the open
  operator decisions. Each run record in `docs/reviews/` lists the checks of
  its run.
- Walk a new check right after its packet. The procedures in `docs/runbooks/`
  stay regression checks.

### Design Language

- [ADR 0101](docs/adr/0101-the-design-language-follows-the-shared-rules.md) is
  the design language. Tasks 001 to 003, 005 to 012 and 014 are implemented.
  Their checks are open in sections 25 to 34 of the pending human checks. The
  checks of [task 008](docs/tasks/adr-0101-task-008-status-bar-and-notices.md)
  are open in its task file.
- [Task 004](docs/tasks/adr-0101-task-004-payment-split.md), the payment split,
  is blocked until task 001 passes its checks.
  [Task 013](docs/tasks/adr-0101-task-013-sheets-fit-the-window.md) waits for
  ADR 0097 task 005.
- The destructive button of musicindex ADR 0064 is implemented. ADR 0101
  Decision 5 and the ADR 0093 amendment of 2026-11-12 own its open items. Its
  checks are open in section 13.
- [ADR 0097](docs/adr/0097-the-fit-check-at-two-widths.md) owns the fit check.
  Phases 1 and 2 of its [phase plan](docs/plans/adr-0097-fit-check-phase-plan.md)
  are complete. [Tasks 003 to 006](docs/tasks/adr-0097-task-003-item-pages-fit.md)
  are Ready.

### Music Shell

- [ADR 0098](docs/adr/0098-the-music-shell-pages-lists-and-search.md) is the
  current record for the Music pages, lists and search. Its item map gives the
  new place of each item of the archived ADRs 0084 to 0087, 0089 and 0090.
- The packets of the [ADR 0084 phase plan](docs/plans/adr-0084-music-column-browser-phase-plan.md)
  and the [ADR 0089 phase plan](docs/plans/adr-0089-shell-phase-plan.md) are
  implemented, but not ADR 0089 packet 016. Their checks are open in sections
  8, 9 and 11. Packet 016, the now-playing panel and the row action Play, waits
  for the ADR 0068 packets.
- The list controls of musicindex ADR 0053 and ADR 0054, and the release counts
  of musicindex ADR 0085 R-WORD-9, are implemented. Their checks are open in
  section 12. The [phase plan](docs/plans/musicindex-adr-0053-list-controls-phase-plan.md)
  gives the packets.
- [ADR 0092](docs/adr/0092-the-keys-of-a-list-row.md) owns the keys of a list
  row. Public playlists, Live and Search do not have the list focus yet. A
  subsequent task gives it (ADR 0092).
- [ADR 0091](docs/adr/0091-search-parser-moves-to-the-shared-crate.md): the
  crate `musicindex-search` owns the search parser, the Index plan, the field
  list, the notes, the `kind:` requests and the sort parameters. HitIt builds
  no copy of these (ADR 0091 Decision 7).
- Ready: [ADR 0102 task 001](docs/tasks/adr-0102-task-001-tiles-arrow-keys.md),
  which waits until the operator opens it,
  [ADR 0093 task 001](docs/tasks/adr-0093-task-001-settings-tab-row-and-rows.md),
  [ADR 0069 task 002](docs/tasks/adr-0069-task-002-shared-guarded-editor.md),
  [ADR 0094 tasks 001](docs/tasks/adr-0094-task-001-fold-header-composite.md)
  and [002](docs/tasks/adr-0094-task-002-singles-group-row-keys.md),
  [ADR 0088 task 001](docs/tasks/adr-0088-task-001-rss-check-follows-a-move.md)
  and [ADR 0064 task 002](docs/tasks/adr-0064-task-002-repair-report-surface.md).
  ADR 0069 task 002 gives the Save bar of ADR 0093. The operator decides the
  priority.

### Library And Tags

- [ADR 0100](docs/adr/0100-a-removal-moves-the-audio-file-to-the-audio-cache.md):
  a removal moves the audio file to the audio cache `~/.cache/hitit/audio`.
- The tag frames of the [ADR 0080 phase plan](docs/plans/adr-0080-tag-frames-phase-plan.md)
  are implemented. Their checks are open in section 10. The next full write of
  "Update n files" waits for more playlist checks (open point 3 of the
  [run record](docs/reviews/orchestrated-run-2026-11-05.md)).
- [ADR 0108](docs/adr/0108-the-tag-update-selects-each-change.md), the check
  boxes of "Update n files": tasks 001 and 002 are implemented. Their checks
  are open in section 48.
- [ADR 0109](docs/adr/0109-check-all-feeds-reads-rss-of-changed-feeds.md),
  RSS reads of changed feeds: task 001 is implemented. Its checks are open in
  section 49.
- HitIt uses the crate `musicindex-tags` (musicindex ADRs 0086 and 0090). The
  operator retag of the library is complete. The R-TAG-6 episode part is open.
  It waits for the HitIt station ADR and the push of stophammer-parser
  d105468 ([task file](docs/tasks/musicindex-adr-0086-task-001-tag-crate-and-retag.md)).

### Show And Broadcast

- [ADR 0113](docs/adr/0113-broadcast-control-surface.md) is the current record
  of the broadcast control surface. A guard test can keep the prefix `adr_0059_`
  in its name.
- [ADR 0068](docs/adr/0068-show-cue-and-audition-isolation.md): packets 001 to
  003 of its [phase plan](docs/plans/adr-0068-show-cue-and-audition-phase-plan.md)
  are implemented. Their checks are open in sections 19 and 20. Packets 004 and
  005 are Ready.
- ADRs [0104](docs/adr/0104-the-show-player-and-the-sound-setup.md),
  [0105](docs/adr/0105-go-on-air-and-end-show.md) and
  [0106](docs/adr/0106-the-cue-changes-during-a-show.md): the six tasks are
  implemented. The agent walked sections 44 to 46. Some operator checks of
  those sections are open.
- [ADR 0107](docs/adr/0107-the-cards-hold-the-on-air-status.md):
  [task 001](docs/tasks/adr-0107-task-001-the-cards-hold-the-on-air-status.md)
  is implemented. Its checks are open in section 47. ADR 0107 names the second
  record, which gives the list of shows.
- [ADR 0110](docs/adr/0110-a-reserved-event-enters-the-registry.md) and
  [ADR 0111](docs/adr/0111-show-instances-and-the-producer-switch.md) are items
  1, 2 and 5 of the [live rehearsal goal request](docs/plans/live-rehearsal-goal-request.md).
  Their tasks are implemented. The checks of ADR 0110 task 002 are open, and
  the checks of ADR 0111 are open in section 51. An agent check does not start,
  stop or change the private units (ADR 0111 Decision 4).
- [ADR 0112](docs/adr/0112-a-station-host-with-a-liquidsoap-producer.md), the
  station host: tasks 001 to 003 are implemented, and the agent walked them on
  Xvfb. The operator check on a real radio host is open. No agent configures a
  radio host.

### Known Open Defects

- The live status shows one song that does not change. The ADR 0068 packets own
  the fix.
- The MP4 `TYER` difference in "Update n files": the fix is implemented.
  [ADR 0080 packet 013](docs/tasks/adr-0080-task-013-mp4-date-shares-one-value.md)
  owns the open operator check.
- A Music column must bring a focused control into view (ADR 0098 Invariants).
  [ADR 0090 task 005](docs/tasks/adr-0090-task-005-each-music-column-shows-the-focus.md)
  implements the fix. Its checks are open in section 15.
- A feed page with two or more roles shows "Publisher", and Sir Libre Records
  shows "Publisher" for `rel="recordLabel"`. ADR 0098 Decision 11.7 and
  musicindex ADR 0066 own the rules.
  [ADR 0085 task 001](docs/tasks/adr-0085-task-001-type-label-of-known-roles.md)
  implements the fix. Its checks are open in section 16.
- Tab does not go to an artist tile on Artists, and after "Show more" the focus
  goes to the sidebar.
  [ADR 0092 task 001](docs/tasks/adr-0092-task-001-artists-list-focus.md)
  implements the fix. Its checks are open in section 17.

A defect stays in this list until the operator passes its checks.

### Stored Facts That Agents Need

- Schema version 33 is current. The migration registry of ADR 0016 owns each
  schema change.
- `tests/fixtures/musicindex-openapi-0.21.0.json` is the stored MusicIndex
  contract of Stophammer release 0.21.0. After each Stophammer release, replace
  it in the same change as the decode changes (ADR 0075 section 6).
- [Open Stophammer requests](docs/plans/hitit-open-requests.md) has an equal
  copy in the Stophammer repository. Update both copies together.
- `docs/plans/broadcast-chain-delivery-order.md` is the only cross-repository
  order. Read it before a session on broadcast work. When a real show is
  scheduled, publisher show-log task 001 takes priority.
- Do not delete the evidence fixture `/tmp/v4vmm-governance.ie6k8TQf`. Its
  cleanup is unconfirmed.

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
make the subject and consequence clear. ADRs 0113/0063 own Event report data.

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

[ADR 0103](docs/adr/0103-the-linux-packages-and-the-release-workflow.md) defines
the Arch package, the `.deb` and the AppImage. The release workflow builds the
`.deb` and the AppImage on `ubuntu-22.04`. `Cargo.toml` holds the `.deb`
metadata. A new run-time library or external program also changes that metadata. The [Linux packages runbook](docs/runbooks/linux-packages.md) gives
the local build steps. The [ADR 0103 plan](docs/plans/adr-0103-linux-packages-plan.md)
gives the open tasks.

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
