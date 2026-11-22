# Current Decisions

Every ADR here still constrains a judgment call. This is the answer to what
binds a change today.

Superseded and fully guarded decisions live in `archive/`. Read those for
research, never to find a live rule. ADR 0061 owns this split.

Status values are the ADR 0057 vocabulary: `Proposed`, `Accepted`,
`Implemented`, `Superseded by ADR NNNN`.

## Governance

| ADR | Scope | Status |
|---|---|---|
| [0001](0001-record-architecture-decisions.md) | ADRs record decisions. Sequential numbering, Nygard structure | Accepted |
| [0057](0057-adr-status-vocabulary-and-amendment-policy.md) | Status vocabulary and in-place amendment rules | Accepted |
| [0061](0061-executable-governance.md) | Reading path, durable and situational guards, archive rule | Accepted |

## Foundation And Storage

| ADR | Scope | Status |
|---|---|---|
| [0012](0012-root-desktop-crate.md) | One root desktop crate | Accepted |
| [0095](0095-the-license-is-agpl-3-0-only.md) | The license is AGPL-3.0-only, with the text in `LICENSE` and the field in `Cargo.toml` | Accepted |
| [0096](0096-the-app-name-is-hitit.md) | The app name is HitIt. The machine name is `hitit`. At the first start the app moves the v4vmm configuration and data directories one time | Accepted |
| [0100](0100-a-removal-moves-the-audio-file-to-the-audio-cache.md) | A removal from the Library moves each audio file to the audio cache, `~/.cache/hitit/audio`. The table `cached_files` holds the cache files. A download of a cached track takes the file from the cache, and each start completes the move | Accepted |
| [0028](0028-local-identity-source-fact-persistence.md) | Source links, ids, and contributors persist as source facts. ADR 0075 replaces its replacement key | Implemented |
| [0053](0053-local-detail-source-fact-parity.md) | Source-fact route for parity gaps not locally durable | Accepted |
| [0054](0054-local-metadata-source-fact-persistence.md) | Feed and track metadata facts persist by source; hydration visual checks remain open. ADR 0075 replaces its replacement key | Accepted, partial |
| [0066](0066-configuration-and-startup-failure-recovery.md) | Core startup requirements; in-app repair and optional-tool retry; [001–003 and 005–013 complete; 004 implemented, operator gate open](../plans/adr-0066-startup-recovery-phase-plan.md) | Accepted |

## Metadata

| ADR | Scope | Status |
|---|---|---|
| [0075](0075-metadata-ownership-and-completeness.md) | Shared metadata rules cover ownership, response completeness, storage by provider, each field with an accepted rule, and labeled identity sections. Decision I makes RSS the only provenance and MusicIndex a cache of RSS. The phase plan records packets and gates. ADR 0076 supersedes Decisions F to H and the source-selection rules | Accepted |
| [0076](0076-playlist-rss-check-for-stale-musicindex-records.md) | MusicIndex stays the normal checker. A playlist RSS check, also run when a show selects the playlist, applies and reports RSS differences. Tags and payment routes in files change only on operator confirmation. Supersedes ADR 0075 Decisions F to H and its source-selection rules. Amends ADRs 0059 and 0065. Amended 2026-10-03: a download writes the stored values. ADR 0108 changes Decision 8 | Accepted |
| [0099](0099-the-rss-path-keeps-only-lightning-payment-routes.md) | The RSS path keeps the first Lightning `podcast:value` block and only `lnaddress` and valid `node` recipients, the rule of stophammer ADR 0098. HitIt discards the routes of other payment systems. Changes ADR 0076 Decision 9 | Accepted |
| [0077](0077-publisher-feed-artist-binding.md) | An artist page is keyed on the publisher feed GUID that an album names. Supersedes ADR 0045. Each proposal needs individual operator review. ADR 0078 supersedes its Decision 4 | Accepted |
| [0079](0079-remove-musicindex-artist-subject-storage.md) | MusicIndex artist subject storage is deleted. The publisher feed is the only artist identity. Person identity stays deferred. Supersedes ADR 0029 | Accepted |
| [0080](0080-tag-frames-follow-their-owner.md) | Tag frames follow their owner: one resolved Nostr key, the item page in `WOAF` and the channel website in `WOAR`, a separate album description frame, idempotent writes that keep MusicBrainz values, plain URLs in URL frames, a compare that uses the resolution of the writer, the artwork URL in `TXXX:MusicIndex Image`, the funding link in `WPAY`, the RSS license in `WCOP` or `TXXX:LICENSE`, a confirmed label in `TPUB` and the language in `TLAN`. `podcast:season` waits for Studio | Accepted |
| [0088](0088-a-feed-move-follows-musicindex.md) | The RSS check reads the MusicIndex `feed_url` of a stored feed. The feed moves to that URL only when the RSS document there has the same `podcast:guid` | Accepted |
| [0081](0081-remove-the-staged-frame-model.md) | Remove the staged frame model: no frame add, remove, detach or dock model with no caller, no reserved slots, and Forward completes ADR 0046 Invariant 2. Supersedes ADR 0046 Invariant 8 | Implemented |
| [0082](0082-publisher-roles-belong-to-each-album-link.md) | Publisher roles belong to each album link: no page type, albums grouped by `album_names_as` and by role agreement, and the feeds that share albums. Revised for Stophammer 0.7.0. Supersedes ADR 0078. ADR 0085 supersedes Decisions 1 to 3 and 6. Decisions 4 and 5 stay | Implemented |
| [0101](0101-the-design-language-follows-the-shared-rules.md) | Design language: adopts the current shared rules of musicindex ADRs 0021, 0022, 0026, 0028, 0032, 0042, 0045, 0051, 0056, 0063, 0064, 0068, 0069, 0071, 0073 and 0085, and restates the live rules of the archived ADR 0083 with the same decision numbers. The main action follows R-ACT-7. The focus ring is 2 px with no gap (R-FOCUS-2). The token packet comes first | Accepted |
| [0004](0004-format-neutral-audio-tag-boundary.md) | Format-neutral audio tag boundary | Accepted |
| [0005](0005-musicbrainz-metadata-lookup.md) | Metadata-based MusicBrainz lookup. No fingerprinting | Accepted |
| [0006](0006-musicbrainz-release-detail-enrichment.md) | MusicBrainz release detail enrichment | Accepted |
| [0007](0007-metadata-compare-table-drag-copy.md) | Compare table drag copy | Accepted |
| [0008](0008-explicit-id3v24-write-boundary.md) | Only explicit ID3v2.4 frames are written | Accepted |

## Services And Boundaries

| ADR | Scope | Status |
|---|---|---|
| [0010](0010-musicindex-endpoint-setting.md) | Configurable MusicIndex endpoint | Accepted |
| [0015](0015-non-ui-service-boundaries.md) | Services stay free of UI concerns | Accepted |
| [0017](0017-cli-debug-contracts.md) | CLI debug contract and JSON output | Accepted |
| [0024](0024-command-query-event-application-layer.md) | Command, query, and event application layer | Implemented |
| [0040](0040-async-vm-runtime.md) | Async view-model runtime. Actors in `src/runtime/` | Implemented |
| [0041](0041-windowed-paged-view-models.md) | Windowed paged view models | Implemented |
| [0056](0056-remote-media-fetch-validation-boundary.md) | Remote media fetch validation and redirect policy. Amended 2026-10-03: no enclosure length check | Implemented |
| [0058](0058-outbound-http-client-policy.md) | All blocking HTTP clients built in `src/http_client.rs` | Implemented |

## Playback And Broadcast

| ADR | Scope | Status |
|---|---|---|
| [0014](0014-playback-session-authoritative-state.md) | `PlaybackSession` is the authoritative now-playing state | Accepted |
| [0020](0020-simulated-playlist-playback.md) | Simulated playlist transport for relay smoke tests | Accepted |
| [0021](0021-mpv-playback-driver.md) | mpv driver behind the `PlaybackDriver` trait | Implemented |
| [0068](0068-show-cue-and-audition-isolation.md) | Persisted Show cue, explicit Show transport, independent audition state/audio/publication boundaries. Revised on 2026-11-11 for the ADR 0089 shell. The Show cue and its transport are built. Audition isolation is not built. ADRs 0105 and 0106 change the last entry and the cue lock | Accepted |
| [0104](0104-the-show-player-and-the-sound-setup.md) | The default Show player is mpv. "No sound" refuses a Show Play. Settings has the group "Audio" with the player, the mpv state and a test sound. The first start opens it. Changes ADR 0069 | Accepted |
| [0105](0105-go-on-air-and-end-show.md) | One on air band on the Show page: a checklist in order, "Go on air" and "End show" with confirmations, the signal state and the listener count of butt, and an end band after the last track. Changes ADRs 0063, 0068 and 0069. ADR 0107 changes Decisions 1, 2, 4 and 5. ADR 0111 changes Decisions 3 and 5 | Accepted |
| [0106](0106-the-cue-changes-during-a-show.md) | Cue entries after the current entry move during a show, by a drag, Alt+Up and Alt+Down, or "Play next". A load opens Show, and the empty Show lists the playlists. Changes ADR 0068 | Accepted |
| [0107](0107-the-cards-hold-the-on-air-status.md) | The band has one button row. Cards and inspectors show status. "End show" stops only parts recorded during this HitIt session. Changes ADR 0105. ADR 0111 changes Decision 3 | Accepted |
| [0110](0110-a-reserved-event-enters-the-registry.md) | The operator adds a reserved relay event from its event ID and its token file. HitIt does not reserve events. Only `event_not_found` makes a row Dead, and a reserved row has no Replace. `Forget` keeps a token file that a target uses or that HitIt did not write, and Attach asks before it replaces a different event. Changes ADR 0059 | Accepted |
| [0111](0111-show-instances-and-the-producer-switch.md) | Each Mixxx producer unit has an instance. HitIt makes a show instance with the publisher setup helper, switches the one Mixxx producer between the private instance and a show instance, and shows the state of each unit. HitIt writes no unit file and protects the private instance. A show of either player uses a selected show instance. A show that Mixxx plays has its own "Go on air" and "End show" steps with the encoder recording. Changes ADRs 0105 and 0107 | Accepted |
| [0112](0112-a-station-host-with-a-liquidsoap-producer.md) | A broadcast host has a producer kind, `mixxx` or `liquidsoap`. HitIt shows and starts the units of a station instance on a radio host over SSH. It does not set up or attach the station instance, and it copies no token file to the radio host. The station event is a reserved event, and a DJ set is a show that Mixxx plays. | Accepted |
| [0113](0113-broadcast-control-surface.md) | The app is a control surface for the broadcast chain: the event registry with created and reserved rows, the saved event selection, the configured target readiness, token files, SSH control, the Show sections and the pace of the service watch. Supersedes ADR 0059 | Accepted |
| [0108](0108-the-tag-update-selects-each-change.md) | The "Update n files" popup has a check box for each file and each change. A cleared change becomes a skip until its stored value changes. Skips stay in a folded section. Changes ADR 0076 Decision 8 | Accepted |
| [0109](0109-check-all-feeds-reads-rss-of-changed-feeds.md) | "Check all feeds" reads RSS only for a feed that musicindex marks as changed, with one conditional RSS check of ADR 0076 that applies the values. The RSS merge for each track goes. Musicindex requests run a maximum of four at the same time. Changes the scope of ADR 0075 task 039 | Accepted |

## UI Architecture

| ADR | Scope | Status |
|---|---|---|
| [0023](0023-design-system-and-view-models.md) | Design system and view-model architecture | Implemented |
| [0025](0025-theme-icon-style-boundary.md) | Theme, icon and style boundary. All eleven packets and recorded visual checks are complete. | Implemented |
| [0026](0026-shared-entity-projection-layer.md) | Shared entity projection layer | Implemented |
| [0027](0027-shared-entity-action-state.md) | Typed action state for shared entities | Implemented |
| [0032](0032-ui-backend-boundary-and-popover-contracts.md) | UI and backend boundary, popover contracts | Implemented |
| [0033](0033-hig-ui-architecture-governance.md) | HIG structural governance for UI work | Implemented |
| [0042](0042-layer-consolidation.md) | Primitive versus composite versus shell | Implemented |
| [0046](0046-workspace-frame-architecture.md) | Workspace frames, history, and chrome. ADR 0098 restates the Music frames of the archived ADRs 0084 and 0089 | Implemented |
| [0047](0047-library-search-unification.md) | One content surface for library and index rows | Implemented |
| [0060](0060-workflow-surface-structure.md) | Music, Show and Settings structure is delivered. Independent audition remains unfinished under ADR 0068. | Accepted |
| [0062](0062-music-content-surface.md) | Five Music packets are complete. The default Index order returns releases. Broader mixed-row search/expansion still needs an evidence review. | Accepted |
| [0098](0098-the-music-shell-pages-lists-and-search.md) | The Music shell, its pages, its lists and its search. It holds the sidebar, the Inbox, the Search page, the list defaults and the item pages. It also holds the groups of singles, the artist levels, the list bar, the download queue, the now-playing panel and the search fields. It restates ADRs 0084, 0085, 0086, 0087, 0089 and 0090, and makes no new decision | Accepted |
| [0091](0091-search-parser-moves-to-the-shared-crate.md) | The parser, the Index plan and the kind rule move to the `musicindex-search` crate in `../musicindex-lib`, and musicindex ADR 0049 owns the syntax. It amends ADR 0087 | Accepted |
| [0092](0092-the-keys-of-a-list-row.md) | My playlists takes the list focus of Releases. Shift+F10 and the Menu key open a row menu. Alt+Up and Alt+Down move a playlist in Curator Order. It applies the keyboard reach invariant of ADR 0098 and uses musicindex ADR 0061 R-LISTKEY-1. Artists, Public playlists, Live and Search are not one Tab stop yet. ADR 0102 changes the arrow keys of a Tiles view | Accepted |
| [0102](0102-the-arrow-keys-move-in-a-grid-of-tiles.md) | The Tiles views of Releases and Artists use musicindex ADR 0078 R-GRIDKEY-1: each arrow key moves to the tile in its direction, and the lines come from the layout at each key press. [Task 001](../tasks/adr-0102-task-001-tiles-arrow-keys.md) waits. Changes ADR 0092 | Accepted |
| [0103](0103-the-linux-packages-and-the-release-workflow.md) | HitIt ships the PKGBUILD, a `.deb` made by cargo-deb and an AppImage made by appimagetool, under musicindex ADR 0084. A release workflow on `ubuntu-22.04` (glibc floor 2.35) makes a draft Release on a tag. The app must name a missing `mpv` and a missing `ssh` | Accepted |
| [0063](0063-show-dashboard-layout.md) | Show cards and diagnostics; [shared log frames and following](../tasks/archive/adr-0063-task-005-shared-log-frames-and-following.md) complete with operator acceptance, preservation and cleanup | Implemented |
| [0069](0069-grouped-settings-and-selective-presets.md) | Grouped Settings, live metadata resource selection, independent audio configuration and selective preset snapshots. Task 001 is complete. [Task 002 is Ready](../tasks/adr-0069-task-002-shared-guarded-editor.md). Implementation of task 002 has not started. ADR 0093 amends the group navigation, Cancel and the leave | Accepted |
| [0093](0093-settings-has-a-tab-row-and-one-save-bar.md) | Settings applies musicindex ADR 0059: a segmented control for the groups, one row for each setting, one draft with a Save bar, Discard and a not-saved mark, the leave question, a preview of the scale and the theme, and actions outside Save. [Task 001](../tasks/adr-0093-task-001-settings-tab-row-and-rows.md) is Ready, and ADR 0069 task 002 delivers the draft. Amends ADRs 0069 and 0074 | Accepted |
| [0094](0094-one-fold-header.md) | One fold header applies musicindex ADR 0060 R-FOLD-1 to R-FOLD-3: the look of a list row, the chevron at the start, then the name in Body text, then the count, one control with its own Tab stop. The singles group row is a tree row, and it keeps Right and Left (musicindex ADR 0061 R-LISTKEY-1). A value cell is not a fold header. [Task 001](../tasks/adr-0094-task-001-fold-header-composite.md) and [task 002](../tasks/adr-0094-task-002-singles-group-row-keys.md) are Ready | Accepted |
| [0097](0097-the-fit-check-at-two-widths.md) | The narrowest window width is 390 px. GPUI layout tests with Figtree run at 390 px and at 940 px and find the three faults of musicindex ADR 0065 R-FIT-3 through fit probes in the shared primitives. The [phase plan](../plans/adr-0097-fit-check-phase-plan.md) adds the pages, sheets and dialogs | Accepted |
| [0074](0074-repair-and-diagnostics-pages.md) | Separate repair/diagnostics pages, action columns, report views and consistent text. Visual and preservation gates accepted; task 013 cleanup confirmed. ADR 0093 supersedes the Settings group menu | Implemented |

## UI Presentation

| ADR | Scope | Status |
|---|---|---|
| [0030](0030-discovery-library-ui-fixes.md) | Current Music/Settings scroll check survives earlier surface fixes | Accepted, partial |
| [0031](0031-release-detail-presentation-contract.md) | Release detail composition | Implemented |
| [0035](0035-track-surface-consolidation.md) | One track detail surface | Implemented |
| [0037](0037-same-entity-surface-parity.md) | Same-entity surface parity. Feed task 001 is complete, including both themes and cleanup. Track task 002 retains its visual gate. | Accepted, partial |
| [0049](0049-inspector-source-ownership.md) | Inspector source tree and filter ownership | Implemented |

## Review Candidates

ADR 0060 replaced the Discover surface with Music. Legacy module records
0003 and 0013 are decided: Superseded by ADR 0060, archived 2026-10-03. This
record cleanup does not put back their earlier screens.

The [2026-09-10 reconciliation](../reviews/2026-09-10-governance-reconciliation.md)
retired replaced requirements before restoring current human checks:

- 0030, 0037, and 0054 have surviving checks in
  [pending human checks](../pending-human-checks.md).
- ADR 0043 is Implemented on 2026-09-18. The operator accepted normal/narrow
  toolbar checks in both themes and confirmed fixture cleanup.
- ADR 0044 is Implemented on 2026-09-19. The operator accepted playlist
  interaction, library removal and immediate row updates in both themes.
  Fixture cleanup is confirmed.
- ADR 0039 is Implemented on 2026-09-18. All three packets and numerical
  ratification are complete. The operator passed thirteen visual checks and
  confirmed fixture removal. The review records preservation as an inference
  from the conditional cleanup command. The dated reconciliation retains its
  original historical result.
- ADR 0025 is Implemented after the 2026-09-18 status correction. All eleven
  packets record implementation, and the review records the required visual passes.
  Further theme work uses bounded packets.
- ADR 0055 is Implemented after the 2026-09-18 module review and focused checks.
- ADRs 0060 and 0062 retain the unfinished scope named in their status sections.
  Their completed structural packets are not an implementation backlog.

The [2026-09-18 review](../reviews/2026-09-18-adr-status-and-remaining-work.md)
records these corrections and the remaining work. Historical reviews retain
their dated findings.

## Status Verification

The situational ADR 0057 guard `adr_0057_status_headers_are_canonical` checks
every current and archived numbered decision. The
[2026-09-11 reconciliation](../reviews/2026-09-11-pending-work-reconciliation.md)
closes the header-format drift. The guard verifies format, vocabulary and date;
implementation and human acceptance still require their named evidence.
