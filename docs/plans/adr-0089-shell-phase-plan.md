# ADR 0089 Shell Phase Plan

## Status

Active - 2026-11-05. This plan is advisory. It states no rule. [ADR 0089](../adr/0089-the-shell-destinations-and-list-pages.md) owns the rules of v4vmm, and musicindex ADRs 0027 and 0028 own the shared rules. The [overhaul plan](design-and-cleanup-overhaul-plan.md#phase-7-visual-overhaul) puts this work before the new tokens.

## Goal

Music becomes a stack. One sidebar holds each destination, a list is a page at full width, and an item opens over its list. Each destination opens on content or an empty state.

## Non-Goals

- No new tokens and no new design rules ADR (overhaul plan step 4).
- No folders, moves or renames of My playlists.
- No audition player and no cue. ADR 0068 owns them, and packet 016 waits for it.
- No change to item page content, apart from Inspect Sources (packet 015). The sequence of R-PAGE-1 is not in this plan.
- No new Stophammer endpoint. Open request 13 stays a draft. A packet that needs an endpoint stops.

## Assumptions

Each fact below was read on 2026-11-05 at commit `5942db3`.

- Music already has a one-page mode. `MusicColumns::for_width` gives `SidebarList` or `SidebarDetail` under `MUSIC_SINGLE_COLUMN_BREAKPOINT`. `FrameNavigationEntry::opens_detail_page` already tells a list root from a page. Packet 001 makes this mode the only mode.
- `Settings` is a navigation entry of the content frame. `Show` is a whole-screen mount. The sidebar lives in the source-list frame, so it shows only in Music. Packet 003 moves it out of the frames.
- `TopApp::select_tab` is the one section switch. About 15 sites call it.
- The app stores no release state "removed". A removed release is a release that is not in the Library, so it shows only in All. Packet 004 needs no new state for this.
- `ReleaseDownloadState` has no failed state. A failed download is a transient queue outcome. Packet 004 adds the fact that In Library needs.
- The app has no value "show playlist". The cuelist is projected from the source playlist of the active playback session. Packet 012 uses that playlist.
- Music has no Play action on a row, and the guard `music_flow_map_dead_artist_view_and_track_play_action_stay_deleted` keeps `EntityActionKind::Play` deleted. ADR 0068 reserves Music playback for audition. Thus the main row action Play of ADR 0089 Decision 6 waits with the panel (packet 016).
- `saved_searches` has `id`, `query`, `start_tab` and `created_at`. It has no name and no seen state. Schema version 21 is current.
- `playlist_tracks.track_id` references `tracks`. A `tracks` row can exist with no file and with `is_in_library` false. A public playlist item needs such a row before a local playlist can hold it.
- No code reads `GET /v1/live-items`, and no code sends `medium`.

## Packet Register

| Packet | Scope | Depends on | State |
|---|---|---|---|
| [001](../tasks/adr-0089-task-001-one-page-stack.md) | The stack: one page at full width, and no "Select an item" | None | Implemented 2026-11-05. V1 and V2 are open |
| [002](../tasks/adr-0089-task-002-toolbar-history-and-marked-row.md) | The breadcrumb goes. Back, Forward and the title move to the toolbar. Back keeps the scroll and marks the row | 001 | Implemented 2026-11-05. V1 to V3 are open. |
| [003](../tasks/adr-0089-task-003-one-sidebar.md) | One sidebar on each mount, no section tabs, the start page Releases, and the destination Tracks goes | 002 | Implemented 2026-11-05. V1 to V3 are open. |
| [004](../tasks/adr-0089-task-004-list-page-state.md) | The state of each list page, its title row and the scope "All, In Library" | 003 | Implemented 2026-11-05. V1 to V4 are open. |
| [005](../tasks/adr-0089-task-005-releases-sort.md) | Sort on Releases and Artists, window groups with Newest, the letter bar, and the "New releases" view goes. Parts A to D, one commit each | 004 | Implemented 2026-11-06. Parts A to D. V1 to V5 are open. |
| [006](../tasks/adr-0089-task-006-search-page.md) | The Search page lists the saved searches, and Discover goes | 005 | Implemented 2026-11-06. V1 to V4 are open. |
| [007](../tasks/adr-0089-task-007-saved-search-edits-and-new-counts.md) | Edit Query, Rename, Duplicate and the count of new results (migration 22) | 006 | Implemented 2026-11-06. Parts A to C. V1 to V4 are open. |
| [008](../tasks/adr-0089-task-008-results-rows.md) | The two rows of the results page: the scope writes the query, and the type tabs show counts | 007 | Implemented 2026-11-06. Parts A to C. V1 to V4 are open. |
| [009](../tasks/adr-0089-task-009-public-playlists.md) | The Public playlists page and its item page | 008 | Implemented 2026-11-06. V1 to V3 are open. |
| [010](../tasks/adr-0089-task-010-copy-to-my-playlists.md) | "Copy to My Playlists…" | 009 | Implemented 2026-11-06. V1 to V3 are open. |
| [011](../tasks/adr-0089-task-011-live-page.md) | The Live page | 008 | Implemented 2026-11-06. V1 to V3 are open. |
| [012](../tasks/adr-0089-task-012-inbox.md) | The Inbox, and Needs attention goes | 008 | Implemented 2026-11-06. V1 to V4 are open. |
| [013](../tasks/adr-0089-task-013-narrow-tab-bar.md) | The tab bar and More in a narrow window | 011, 012 | Ready |
| [014](../tasks/adr-0089-task-014-row-rules.md) | The rows of Decision 6, apart from Play | 005 | Ready |
| [015](../tasks/adr-0089-task-015-inspect-sources-sheet.md) | Inspect Sources is a sheet | 003 | Ready |
| [016](../tasks/adr-0089-task-016-now-playing-panel.md) | The now-playing panel and the row action Play | The operator decision on ADR 0068 | Waits for the operator |

## Sequence

The packets run in register order, one at a time, because most of them change `src/app.rs`, `src/library/app_impl.rs` and `src/view_models/library.rs`. Between two packets the sidebar is correct for the pages that exist: each packet adds its own row, and Discover and Needs attention stay until their replacements land.

The [run record](../reviews/orchestrated-run-2026-11-05-shell.md) holds the decisions and the final walk.

## Rules For Each Packet

These restate rules of `AGENTS.md`, ADR 0061 and the review practice. They create no rule.

- Keep the gate Green. Run `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, then `cargo build --bin v4vmm`.
- Never run the app.
- Delete each guard and test of a rule that the packet replaces, in the same change. A new guard is situational and cites ADR 0089. Do not weaken a guard of a rule that stays.
- Delete the code that no path reaches after the change.
- Update `docs/architecture/source-map.md` for each new owner file, and remove the rows of deleted files.
- A view model carries no renderer type. A new visible word or number comes from a view model or a token.
- Write comments and documents in Simplified Technical English. Name ADR 0089 and the decision in each new module comment.
- Do not commit. The orchestrator reviews and commits.

## Schema And API

- Migration 22 (packet 007) adds the name and the seen state of a saved search.
- Packet 005 adds `medium` to `fetch_recent_feeds` and `fetch_feeds_by_name`.
- Packet 009 reads `GET /v1/feeds/{guid}` with `include=remote_items`.
- Packet 011 adds `GET /v1/live-items`.
- The stored contract stays version 0.18.1.

## Risk Areas

- `tests/architecture_tests.rs` asserts source text. Packets 001 to 003 delete many guards. Each deletion must name the decision that replaces its rule.
- A saved `config.toml` layout can hold the old frames. `WorkspaceLayout::from_config` must keep loading it.
- Keyboard focus: the handlers of `src/app/keyboard.rs` choose a list column or a detail column. After packet 001 only one shows.
- Packet 004 changes the default scope to All, so the start page sends Index requests at start. The Library rows must show before the response.

## Test Strategy

Each packet states mechanical criteria at the view model layer and lists its visual checks apart. An agent cannot run the app. The operator walks the visual checks at the end of the run, from section 11 of [pending human checks](../pending-human-checks.md).

## Rollback

Each packet is one commit. `git revert` of a packet and the packets after it restores the state before it. Migration 22 is forward only under the ADR 0016 registry, and a revert of packet 007 needs a restore of the database backup.

## Open Findings

- Open points of ADR 0089 keep their present behavior: the verb "Download", no upcoming items in Live, no other actions of a public playlist, `in:index` only as typed text, and the row action on hover and focus.
- R-TOOLBAR-1 puts the page title in the toolbar, and R-LIST-1 item 4 gives a list page a title row. Both show, so a list page shows its title twice. The final walk asks the operator about this.
- 2026-11-06: an open window group page keeps the rows of its key, so a fresh load of the Releases list does not empty it (`WindowGroupSource::Releases` in `src/view_models/window_groups.rs`). No packet owns this.
- 2026-11-06: 37 comments and strings in `src/` still say "New releases". Packet 006 changes each string that a person sees.
- 2026-11-06: the other `open::that` sites of `src/` pass a URL with no scheme check. RSS and MusicIndex responses are untrusted input. Only `live_link_url` checks the scheme. No packet owns the other sites.
