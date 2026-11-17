# ADR 0101: The Design Language Follows The Shared Rules

## Status

Accepted - 2026-11-15. The operator accepted this ADR on 2026-11-15, and the orchestrator session relayed the decision. This ADR is the HitIt record of the shared design rules. The ADRs of Decision 11 that adopt other shared rules stay their owners.

- Bar: crosses repositories. musicindex owns each rule that this ADR adopts, and musicindex.org and Studio obey the same rules.
- Supersedes: [ADR 0083](archive/0083-design-language.md), the design language.
- Revised: 2026-11-16, Decision 7, by operator decision relayed by the orchestrator session: the left side of the status bar shows the latest event of one bounded app event list. The list holds the show event reports and the events with no visible action of musicindex ADR 0026 R-NOTICE-1 item 2. [ADR 0101 task 008](../tasks/adr-0101-task-008-status-bar-and-notices.md) builds it.

Operator choice, 2026-11-15: HitIt adopts musicindex ADR 0045 R-ACT-7 for the main action ([Decision 5](#5-actions-have-a-hierarchy)). HitIt keeps the focus ring of 2 px with no gap ([Decision 1](#1-the-palette)). musicindex ADR 0073 R-FOCUS-2 (musicindex commit `785ae28`) gives this ring to Studio and HitIt. It replaces musicindex ADR 0068 R-SCALE-4 item 6.

Operator choice, 2026-11-15, relayed by the orchestrator session: HitIt adopts musicindex ADR 0075 R-VALUE-4 and R-VALUE-5 (musicindex commit `bf58554`). R-VALUE-5 replaces R-VALUE-1 item 1. HitIt also adopts musicindex ADR 0076 R-ADDRESS-1 to R-ADDRESS-3 (musicindex commit `a08dc01`) and musicindex ADR 0077 R-SHEET-1 and R-SHEET-2 (musicindex commit `d3db352`). The crate `musicindex-value` gives the split arithmetic and the short form of an address ([Decision 2](#2-color-roles-do-not-mix), [Decision 11](#11-hitit-adopts-these-shared-rules)).

## Context

Step 4 of the [overhaul plan](../plans/design-and-cleanup-overhaul-plan.md#phase-7-visual-overhaul-1), Phase 7, says: "First, a HitIt ADR adopts the rules. It amends or supersedes ADR 0083 and the Phase 3 tokens." A shared rule binds HitIt only through this ADR. Step 4 waits for the operator. The three defect fixes of 2026-11-14 come first: ADR 0090 task 005, ADR 0092 task 001 and ADR 0085 task 001.

The research note `~/build/musicindex-research/research_notes/ui-design/hitit-conformance-2026-11-15.md` (musicindex-research commit `77fcefa`) compares HitIt `1866ba9` with musicindex `3baa6f0` and musicindex-assets `dc564f9`. It is advisory. It says that the shared rules are stable, and it gives these facts:

- HitIt agrees with most rules of the shell, the back control, the lists, the search, the role words and the destructive button.
- The base layer of HitIt does not agree. The focus color, the type sizes, the radius values and the spacing scale are earlier values.
- ADR 0083 Decision 5 and musicindex ADR 0045 R-ACT-7 give a different main action.
- ADR 0083 line 161 gives the light page `#f2f2f7`. musicindex ADR 0045 R-ELEVATION-2 makes the page white.

The [check of the open rows](../reviews/shared-ui-rules-gap-check-2026-11-15.md) examines the rows that only a ChatGPT run found. It gives a result for each row. Some results depend on the content of a shared rule, which that check did not examine.

ADR 0083 has four "Changed by" lines, three "Amended" lines and one "Revised" line. The limit is three "Changed by" lines. Thus this ADR does not change ADR 0083. It restates each live rule of ADR 0083 and of its changes, and it supersedes ADR 0083.

This session did not read a musicindex file. The rule IDs and the short descriptions of shared rules come from the research note and its ChatGPT log. musicindex owns the text of each shared rule. Where this ADR gives a shared value, the musicindex record is correct if the two differ.

## Decision

Decisions 1 to 10 keep the numbers of ADR 0083, because 214 lines in `src/` and `tests/` cite "ADR 0083 Decision N". Each decision gives the live rule of HitIt and the shared rule that owns a part of it. Decisions 11 to 13 adopt the shared rules and give the sequence of the work.

### 1. The Palette

The theme profiles use these values. Dark is the default. Each value is a named token in `src/ui/tokens.rs`.

| Role | Dark | Light |
|---|---|---|
| Sidebar | `#141417` | `#f5f5f7` |
| Text | `#f5f5f7` | `#1d1d1f` |
| Muted text | `#a1a1a6` | `#5c5c62` |
| Separator | white at 10 % | `#3c3c43` at 16 % |
| Accent | `#2d7bff` | `#0a5bd6` |

- **Dark levels.** The dark levels stay: background `#0b0b0d`, surface `#1c1c1f` and raised surface `#26262a`.
- **Light levels.** The light levels obey musicindex ADR 0045 R-ELEVATION-2. The light page is white. This corrects ADR 0083 line 161, which gave the page `#f2f2f7` from the earlier R-ELEVATION-1.
- **Light level values.** R-ELEVATION-2 gives three levels: the page `#ffffff`, a group or a card `#f2f2f7`, and an open entry in a group `#ffffff`. In light, a popover, a menu and a sheet have `#ffffff` with the shadow of R-ELEVATION-1. The open entry is not `#ececef`, because the status text `#1a7f37` has only 4.31:1 on it. The light raised surface is not a level. The orchestrator session read these values in musicindex at `3baa6f0` on 2026-11-15.
- **Focus.** One focus token, `#2d7bff` dark and `#005bd3` light (musicindex ADR 0080 R-TOKEN-1). The accent is not the focus color.
- **Focus ring.** The focus ring is 2 px with no gap, an inset border (`src/ui/layouts.rs:29`). musicindex ADR 0073 R-FOCUS-2 (musicindex commit `785ae28`) gives this ring to Studio and HitIt. The token `focus-ring.hitit` of musicindex-assets `b8399cc` gives width 2 and offset 0.
- **High contrast.** The high-contrast profiles keep their own values and their contrast tests. The system setting selects them (Decision 11, R-A11Y-1).

### 2. Color Roles Do Not Mix

Each color has one meaning:

- **Accent** marks the selected item and a link.
- **Status** colors mark success, warning, danger and information. musicindex ADR 0068 R-STATUS-1 gives the values, and musicindex ADR 0080 R-TOKEN-1 gives danger and its tint. Danger also marks a destructive action. In the dark profile, the warning color is orange, as in the light profile.
- **Entity** colors mark a stated entity kind: feed, track, playlist, artist, publisher, label and the other kinds of the website. They show as a dot, and musicindex ADR 0032 R-DOT-1 sets the dimensions of the dot and the hollow dot. `src/assets/tokens.json` gives the entity values that R-DOT-2 checks. A dot is never a filled text badge. Live red is reserved for a live item.
- **Value** marks the shares of a payment split. musicindex ADR 0080 R-VALUE-2 gives one row for each recipient and one summary bar in the share color, `#2d7bff` dark and `#1a56c0` light. musicindex ADR 0075 R-VALUE-5 gives the percentage of each share, with a fee share in the total. musicindex ADR 0076 R-ADDRESS-1 gives the short form of a node key or an npub in a row. HitIt calls the crate `musicindex-value` for both, and it has no copy of the code (R-VALUE-4, R-ADDRESS-3).

An entity color appears only when the source states the kind. A payment recipient has no stated kind, so a payment split never uses entity colors. Text always carries the meaning: a name, a word or a number. Color alone never does.

### 3. Type

- The app embeds Figtree. It uses Figtree on Linux and on each system that is not an Apple device. It uses the system font on an Apple device (musicindex ADR 0068 R-TYPE-1 item 4).
- The sizes are the Desktop sizes of musicindex ADR 0068 R-TYPE-1. A page title obeys R-TYPE-2: an album, track or playlist page title is Large Title bold, 26 px. An artist or label name is Display bold, 42 px.
- The UI scale steps scale each size.
- Identifiers such as GUIDs and URLs in an inspection view use a monospace font.

### 4. Artwork Leads

- A cover shows at the sizes of the artwork tokens: 152 pixels for a tile and 200 pixels for a page header at M (musicindex ADR 0021 R-ART-4 item 4).
- The corner radius of art is 6 % of the edge, with a minimum of 4 px (R-ART-4 item 2). The image order is R-ART-3.
- Art has a shadow (R-ART-4 item 3). A larger image has a larger shadow. A popover, a menu and a sheet have one shadow (musicindex ADR 0068 R-ELEVATION-1 item 2).
- A page header shows a blurred glow from the cover with film grain, and a veil that keeps the header text at 4.5:1 (musicindex ADR 0021 R-ART-5).
- A missing cover shows the logo shape in 45 % of the entity color at each size, with no letters and no icon (R-ART-2). It never shows an emoji.
- A collection has the logo frame of [ADR 0098](0098-the-music-shell-pages-lists-and-search.md). Its item map gives the place of ADR 0084 Decision 12.
- Show displays the cover of the current track.

The renderer gets the header art from a view-model fact, for example a prepared image. The view model carries no renderer type. GPUI 0.3.1 has no blur filter, no blend mode and only two stops in a gradient. Thus the app prepares the glow image outside the renderer: it blurs and saturates the cover and adds the grain with the `image` crate. GPUI draws that image below a veil of two stacked gradients. The first glow packet must prove this method, also for an animated cover.

### 5. Actions Have A Hierarchy

| Kind | Presentation | Example |
|---|---|---|
| Main action | musicindex ADR 0045 R-ACT-7. A track page fills its audio action. A release page fills nothing until it has a main action of its own. The header button has the text color, not the accent | Play |
| Frequent action | At most two buttons. Each has a fill and one of the four roles (musicindex ADR 0042 R-ACT-4 and R-ACT-6) | Add to Playlist |
| Navigation | A link on the text | Artist name, publisher name |
| Identity and provenance | Quiet icon links, always visible | Website, Nostr, RSS |
| State | A small icon and a word, always visible | Tags differ, removed from feed, ready for a show |
| Rare, technical or destructive action | The "⋯" menu and the right-click menu | MusicBrainz lookup, download again, remove |

- A download action is a button of the role Secondary, because a download changes stored data (R-ACT-4 item 3, `ControlStyle::by_result` in `src/ui/control_styles.rs:139`). Until a track page has an audio action, it has no primary button. The download state icon of Decision 6 shows the download step.
- Buttons and menu items use title case (musicindex ADR 0023 R-CASE-1), for example "Add to Playlist" and "Download Album". A label names a release by its kind ([ADR 0098](0098-the-music-shell-pages-lists-and-search.md) item map, ADR 0084 Decision 5).
- A destructive item is last in its menu, after a divider. Its label ends with "…" (musicindex ADR 0026 R-ELLIPSIS-1 item 2). It has a red label on a red tint and is never filled. Its label names the result (musicindex ADR 0080 R-ACT-3).
- A destructive item asks for a confirmation, except when the person can undo in the same view (R-ACT-3). The confirmation is a sheet with "Cancel" (musicindex ADR 0023 R-CONFIRM-1).
- In a horizontal row of buttons, musicindex ADR 0064 R-ACT-8 applies. This includes a dialog, a bar and the buttons of a list entry. The destructive button is the leftmost button, a larger space follows it, and the safe action is the rightmost button. When a row has two destructive buttons and other buttons, the two destructive buttons are at the left end.
- A row with only destructive buttons shows one button "Remove…" with a menu (R-ACT-10), for example the playlist row of a removed track.
- The action column of a maintenance page ([ADR 0066](0066-configuration-and-startup-failure-recovery.md)) puts its destructive controls last, after a divider. When the page is narrow, the column wraps into a band, and the band puts them first (R-ACT-11).
- "Use Defaults…" in Settings is destructive: it has the red tint and a confirmation (R-ACT-12).
- A destructive confirmation obeys R-ACT-9: Cancel has the first focus, Enter does not start the destructive action, and Escape closes the dialog with no action.

These items of ADR 0083 stay open for the operator:

- Recovery has "Retry" and "Dismiss". In one condition, "Dismiss" can discard a retained conversion file. HitIt does not identify "Dismiss" as destructive.
- The staged edit row of Inspect Sources has no Enter binding, and the focus stays on the sheet. The row is not a confirmation, so R-ACT-9 item 1 has no clear safe button there.
- In a destructive confirmation, Cancel has the first focus, but its focus ring shows only after a key press. The operator decides if the ring must show when the dialog opens.

### 6. Rows Show Information And Hide Most Actions

- A row shows its information and its state at all times.
- A track row and a release row show their download state as an icon in one column:
  - Not downloaded: `CloudDownload` in the accent color, the one state that is a button.
  - Queued: `Clock`.
  - Downloading: `LoaderCircle`, which turns.
  - Downloaded: `CloudCheck` in the status color Success of R-STATUS-1.
- Each download state icon has a tooltip and an accessibility label, so color is never the only signal. A partly downloaded release also shows a short count beside its icon, for example "3 of 9".
- A row shows its actions on hover and on keyboard focus.
- The right-click menu of a row holds the same items as its "⋯" menu.
- A list with selection has the shared check box and the action bar of [ADR 0098](0098-the-music-shell-pages-lists-and-search.md) (item map, ADR 0084 Decision 10). The check box obeys musicindex ADR 0063 (Decision 11).
- A track can be dragged onto "My playlists" in the sidebar, the destination of ADR 0098 Decision 1. While a track is held over it, the page opens My playlists, and the track drops on a playlist row there.
- In selection mode, a drag that starts on a checked track row carries each checked track, in list order. The drag image shows the count, for example "3 tracks". A drag that starts on a row that is not checked does nothing. The drop adds each track, and the status names the count.
- Each action is reachable with the mouse. A keyboard shortcut is optional ([ADR 0067](archive/0067-platform-shortcut-modifiers.md) stays).

### 7. One Status Bar

The window has one status bar at the bottom, the same on every section. One shared composite owns it.

- **Left:** the latest event of one bounded app event list. The list holds the show event reports ([ADRs 0059 and 0063](0059-broadcast-control-surface.md)) and the events with no visible action of musicindex ADR 0026 R-NOTICE-1 item 2: a feed check, a download or a tag write that fails. Each event keeps the words of its owner and its recorded time.
- **Right:** small items with a simple icon and a short word. Each item opens its detail. The items are feed state, files to update, reports with a count, and show state.
- **The show item appears only while a show is active.** With no active show, Music and Settings show no broadcast state ([ADR 0098](0098-the-music-shell-pages-lists-and-search.md) Decision 18).

The status bar replaces the live status band. The reports item opens the normal-shell notice of ADR 0066. Its full subjects, repair actions and retained actions stay reachable there and in Settings. A notice obeys musicindex ADR 0026 R-NOTICE-1 (Decision 11).

### 8. Inspection Is One Step Away

- An album or track page has one "Inspect Sources" entry.
- The inspection view shows the RSS value, the MusicIndex value and the file tag side by side. Each row gives its source element and a result in words.
- RSS is labeled the source of truth, and MusicIndex the cache ([ADR 0075](0075-metadata-ownership-and-completeness.md) Decision I).
- The inspection view of a track holds the content of the earlier track compare grid.

### 9. A Tile Is A Release

The tile view shows one tile for each release, not for each track. [ADR 0062](0062-music-content-surface.md) stays for recency order and paging. [ADR 0098](0098-the-music-shell-pages-lists-and-search.md) owns the start page of the app.

### 10. Simple Icons

- One icon set with one stroke style. The app uses the icons of `gpui-kit-assets` ([ADR 0025](0025-theme-icon-style-boundary.md) icon boundary). The sizes obey musicindex ADR 0068 R-ICON-1 and R-ICON-2.
- An icon with a word is the default. An icon alone needs a tooltip and an accessibility label (musicindex ADR 0026 R-TOOLTIP-1).

### 11. HitIt Adopts These Shared Rules

HitIt adopts the rules in this table, at musicindex commit `3baa6f0`. musicindex owns their text. A change to one of them needs a superseding record in `../musicindex`, not a change here. A new shared rule binds HitIt only when a HitIt ADR adopts it.

| musicindex ADR | Adopted | Not for HitIt |
|---|---|---|
| 0068, the foundations | R-PROFILE-1, R-SCALE-1, R-TYPE-1, R-TYPE-2, R-SPACE-1, R-TARGET-1, R-RADIUS-1, R-STATUS-1, R-ELEVATION-1 with the light levels of R-ELEVATION-2, R-ICON-1, R-ICON-2, R-A11Y-1 | R-SCALE-2 and R-SCALE-3, the web scales. R-SCALE-4, the Studio scale |
| 0073, the focus ring of Studio and HitIt | R-FOCUS-2, at musicindex commit `785ae28` | None |
| 0069, the interface rules and the shell | R-WORD-1, R-VALUE-1 items 2 and 3, R-VALUE-2, R-ACT-1, R-ACT-2, R-ACT-3, R-TOKEN-1, R-SHELL-1, R-NAV-1, R-NAV-2, R-NAV-4, R-LIST-1, R-TOOLBAR-1, R-FIT-1 | R-SHELL-2, the editor outline. R-NAV-3, the touch back control |
| 0021, the artwork | R-ART-1 to R-ART-5 | None |
| 0022, the shared assets | Decision items 1 to 7. HitIt copies the fonts and `tokens.json` from musicindex-assets | None |
| 0023, the words, links and messages | R-LINK-1, R-CASE-1, R-SCOPE-1, R-DATE-1, R-MSG-1, R-CONFIRM-1, R-WORD-2, R-WORD-8 (the product name), R-WORD-9, R-WORD-10 | None |
| 0028, the item pages | R-PAGE-1, R-PAGE-2, R-ART-6. ADR 0098 applies the sections and the header actions | None |
| 0032, the type dot | R-DOT-1, R-DOT-2 | None |
| 0042, a button with a label has a fill | R-ACT-4, R-ACT-5, R-ACT-6 | None |
| 0064, the destructive button | R-ACT-8 to R-ACT-12 | None |
| 0026, the controls and the feedback | R-FOCUS-1, R-FIELD-1, R-SEARCH-1, R-TOOLTIP-1, R-LOADING-1, R-EMPTY-1, R-NOTICE-1, R-ELLIPSIS-1 | The ring width and gap of R-FOCUS-1. musicindex ADR 0073 R-FOCUS-2 gives the HitIt ring |
| 0045, the main action, the live action and the light page | R-SYSTEM-1, R-ACT-7, R-LIVE-1, R-ELEVATION-2 | R-SCALE-5, the web artist title |
| 0051, the segmented control and the selected look | R-SEG-1, R-SELECT-1 | None |
| 0056, the sidebar glow | R-SIDE-1, R-SIDE-2 | None |
| 0063, the check box | R-CHECK-1 to R-CHECK-8 | None |
| 0075, the split code and the fee of a split | R-VALUE-4, R-VALUE-5, at musicindex commit `bf58554`. R-VALUE-5 replaces R-VALUE-1 item 1 | None |
| 0076, the short form of a long address | R-ADDRESS-1 to R-ADDRESS-3, at musicindex commit `a08dc01` | None |
| 0077, a sheet fits the window | R-SHEET-1, R-SHEET-2, at musicindex commit `d3db352`. R-SHEET-2 replaces R-FIT-2 item 4 of musicindex ADR 0065. ADR 0097 task 005 makes the check | None |
| 0071, the edge indicator | R-EDGE-3, the token copy | R-EDGE-1 and R-EDGE-2. HitIt keeps its desktop tab bar |

[ADR 0098](0098-the-music-shell-pages-lists-and-search.md), [ADR 0091](0091-search-parser-moves-to-the-shared-crate.md), [ADR 0093](0093-settings-has-a-tab-row-and-one-save-bar.md), [ADR 0094](0094-one-fold-header.md) and [ADR 0097](0097-the-fit-check-at-two-widths.md) adopt other shared rules. They stay the owners of those adoptions. musicindex ADRs 0022, 0028, 0032, 0042, 0056, 0064 and 0073 are not in the list of the orchestrator brief. ADR 0073 records the operator choice of the focus ring. ADR 0083 adopted 0022, 0028, 0032, 0042 and 0064, so this ADR restates those adoptions. It adopts 0056, because HitIt builds R-SIDE-1 and no HitIt ADR adopts it.

### 12. The Known Differences And Their Packets

The code differs from the adopted rules in these points. The [check of the open rows](../reviews/shared-ui-rules-gap-check-2026-11-15.md) and the research note give the evidence.

| Rule | HitIt now | Packet |
|---|---|---|
| R-TOKEN-1, the focus color | `#5795ff` dark, `#3b7cde` light (`src/ui/tokens.rs:256` and `:365`) | Token packet |
| R-FOCUS-2, the focus ring | 2 px with no gap (`src/ui/layouts.rs:29`). The ring agrees. No test compares it with `focus-ring.hitit` | Token packet: the test only |
| R-TYPE-1 and R-TYPE-2 | Headline 15, Title 24, Display 30 (`src/ui/tokens.rs:632` to `:641`) | Token packet |
| R-RADIUS-1 | 4, 6, 10, 14 (`src/ui/tokens.rs:583` to `:587`) | Token packet |
| R-SPACE-1 | No 20 and no 48 (`src/ui/tokens.rs:538`) | Token packet |
| R-ELEVATION-2, the light levels | No token gives the group level and the open entry level (`src/ui/tokens.rs:331` to `:334`) | Token packet |
| R-ART-5, the header glow | A flat backdrop (`src/ui/composites/cover_backdrop.rs:31`) | Glow packet |
| R-ART-4, the art radius | A fixed map from size to radius (`src/ui/composites/thumbnail.rs:88`) | Art radius packet |
| R-VALUE-1, R-VALUE-2, R-VALUE-5 and R-ADDRESS-1 | No payment section on an item page | Payment split packet |
| R-A11Y-1 | No code reads the three system settings (`src/ui/theme_bridge.rs:85`) | System settings packet |
| R-DATE-1, R-LOADING-1, R-NOTICE-1 | A fixed English date, a skeleton with no animation, no shared notice | One packet for each rule |
| R-LINK-1 | No link to the musicindex.org page of an item | Link packet |
| R-SELECT-1 | A tint and an outline (`src/ui/control_styles.rs:129`) | Selected look packet |
| R-ACT-7 | The next curation step of ADR 0083 Decision 5 is in the code, for example "Download release" (`src/view_models/track_detail.rs:893`) | Main action packet |
| R-CHECK-5 | The token test waits, because `tokens.json` has no check box tokens | When musicindex-assets has the tokens |

The sequence of the work:

1. The token packet. It changes the tokens of each row that names the token packet, in `src/ui/tokens.rs`. HitIt commit `77420fc` copied `tokens/tokens.json` of musicindex-assets `b8399cc` to `src/assets/tokens.json`. A test compares the values with `src/assets/tokens.json`. The three defect fixes of 2026-11-14 come before it.
2. One packet for each surface. The sequence starts with the header glow, the art radius, the payment split and the system settings. Then come the dates, the loading state, the notices, the link, the selected look and the main action.
3. The Ready packets of ADR 0093, ADR 0069 task 002, ADR 0094 and ADR 0097 tasks 003 to 006 come after the token packet. Thus each one is built one time in the new design.

Step 4 of the overhaul plan asks for a result for each [visual requirement moved from pending checks](../plans/design-and-cleanup-overhaul-plan.md#visual-requirements-moved-from-pending-checks---2026-10-07) of size and color. The token packet records if each one stays or if this ADR replaces it.

### 13. Changes To Other ADRs

This ADR restates the three changes of ADR 0083. Each changed ADR keeps its "Changed by" line for the archived ADR 0083, and that line names this ADR:

- [ADR 0025](0025-theme-icon-style-boundary.md), Decision 2: entity kinds get their own palette and stop using status colors (Decision 2 here).
- [ADR 0062](0062-music-content-surface.md), Decision 9: a tile is a release (Decision 9 here).
- [ADR 0066](0066-configuration-and-startup-failure-recovery.md), Decision 7: the normal-shell notice opens from the status bar (Decision 7 here).

## Consequences

- HitIt gets one source for its design: this ADR and the musicindex records that it names.
- Each surface packet changes many views. The token packet changes the size of most text and radii, so the fit tests of ADR 0097 and many visual checks run again.
- The Ready packets of the Settings editor, the fold header and the fit check wait for the token packet.
- `EntityKind` resolves no status color. Code that colors a payment recipient by kind must change.
- Pages lose most visible buttons. Users find rare actions in the "⋯" and right-click menus.

## Alternatives Considered

- **Add a fifth "Changed by" line to ADR 0083.** Rejected. The limit is three, and ADR 0083 has four.
- **Adopt the shared rules and keep ADR 0083 current.** Rejected. Two current records would state the palette, the type and the actions, and ADR 0083 line 161 would stay incorrect.
- **One HitIt ADR for each musicindex ADR.** Rejected. The reading path grows with each record.
- **Keep ADR 0083 Decision 5 for the main action.** A primary "Download" button with the kind of the release, then "Add to Playlist". Rejected by the operator on 2026-11-15. HitIt would differ from musicindex.org and Studio, and R-ACT-7 is a shared rule.
- **The shared ring of R-FOCUS-1, 3 px with a gap of 2 px.** Rejected by the operator on 2026-11-15. A ring outside the control needs free space, and a parent that clips its content can cut it. The reason of archived musicindex ADR 0041 item 6 also applies at the desktop scale of HitIt: a ring of 3 px with a gap "looked too heavy at this scale, and it did not match the field".

The alternatives of ADR 0083 stay in its text. They are mockup version 1, the website sizes, a command palette, identity links in "⋯" and recipient colors by entity kind.

## Verification

Mechanical, at the owning layer:

- `tokens.rs` exposes each palette role of Decision 1 for each theme profile. The contrast tests cover each pair of each role: surfaces, entity colors, value colors, the display size and the artwork shadow.
- A token test compares the focus color, the type sizes, the radii, the spacing and the light levels of `tokens.rs` with `src/assets/tokens.json`. The token packet adds it.
- A test compares `focus-ring.hitit` of `src/assets/tokens.json` with the HitIt ring (`src/ui/layouts.rs:29`). musicindex ADR 0073 asks for it, and the token packet adds it.
- `EntityKind` resolves no status color. A guard blocks a status token in the entity palette.
- The payment split view model exposes one row for each recipient and one summary bar.
- The theme view model selects a profile from the three system settings of R-A11Y-1. A test gives the settings through a fake source.
- The date formatter gives the date in the language of the system. A test gives two languages.
- A status bar view model exposes no show item when no show is active.
- The status bar has one composite owner, and no screen renders a second status band.
- The Library tile view model exposes one tile for each release.
- Each row action exists in the row's "⋯" items and in its right-click items.
- The R-CHECK-5 token test, when musicindex-assets has the check box tokens.

Visual, for the operator:

- Each section at M, in Dark and Light, agrees with musicindex.org in hierarchy, color roles and artwork. This replaces the check of ADR 0083 against the accepted mockups of 2026-10-03, because the shared rules replace those mockups.
- The page header shows a blurred glow from the cover, with grain. The glow fades into the background before the content. It agrees with the album page of musicindex.org for the same cover.
- Tab through a HitIt page. The focus ring of 2 px shows fully on a list row, a tile and a dialog button at 390 px and at 940 px.
- The status bar reads clearly at normal and narrow widths.

## Item Map

| ADR 0083 item | Place here |
|---|---|
| Decisions 1 to 8 and 10 | Decisions 1 to 8 and 10, with the same numbers |
| Decision 9, superseded except the tile rule | Decision 9 |
| Amendments of 2026-10-07, 2026-10-20, 2026-10-21 and the revision of 2026-10-22 | Decisions 5 and 6 |
| "Changed by" musicindex ADRs 0021, 0022, 0023, 0026, 0032, 0042, 0064, 0068 and 0069 | Decisions 1 to 10 cite each rule at its place, and Decision 11 adopts them |
| Rules That MusicIndex Replaces | Decisions 1 to 8 and Decision 11 |
| Line 161, the light page | Decision 1, corrected by R-ELEVATION-2 |
| The "Changes" lines for ADRs 0025, 0062 and 0066 | Decision 13 |
| Consequences: the glow method | Decision 4 |
| Verification | Verification |
