# ADR 0089: The Shell, The Destinations And The List Pages

## Status

Accepted - 2026-11-05. The operator made the selections below on 2026-10-24 in the [research request](../plans/research-ui-shell-and-search-request.md), and lifted its hold on 2026-10-25. The operator asked for this record, answered its five questions and accepted it on 2026-11-05 (see [Operator Answers](#operator-answers---2026-11-05)). Packets 001 to 015 of the [phase plan](../plans/adr-0089-shell-phase-plan.md) are implemented on 2026-11-06, and their visual checks are open. Packet 016, the panel and the row action Play, waits for ADR 0068.

This record makes these changes:

- It supersedes [ADR 0084](0084-music-column-browser.md) Decisions 2, 3 and 8. It amends Decision 9: the location breadcrumb goes. ADR 0084 Decision 1 is already replaced by musicindex ADR 0027 R-SHELL-1.
- It amends [ADR 0060](0060-workflow-surface-structure.md): the invariants of [Amended Rules](#amended-rules).
- It amends [ADR 0086](0086-new-releases-singles-window.md): New releases is not a page, and Releases uses the window groups with the sort Newest.
- It amends [ADR 0087](0087-search-fields-and-live-results.md): saved searches move from Discover to the Search page.
- It amends [ADR 0046](0046-workspace-frame-architecture.md): Music has no separate list frame and detail frame.
- `AGENTS.md` and the [overhaul plan](../plans/design-and-cleanup-overhaul-plan.md) state this record. Each packet deletes the guards of the rules that it replaces, in the same change.

## Context

musicindex ADR 0027 owns the shell, the back control, sort, view and the toolbar of the three apps. musicindex ADR 0028 owns the sections and the header actions of an item page. This record cites them, and does not state their rules again. It states only what those records leave to v4vmm: the destinations, the start page, the scopes and the defaults (R-LIST-1 item 6).

These facts control the design:

- Five of the six Music destinations open with the text "Select an item in the list." (`src/view_models/library.rs:813`). R-SHELL-1 item 3 forbids a placeholder.
- The operator found that "New releases" was the same as Releases with All and Newest.
- The operator found that Needs attention showed the same tracks as the Inbox section for the show playlist.
- The app has no Inbox. A search of `src/` found no such destination.
- The stored MusicIndex contract is version 0.18.1. Section [The Contract](#the-contract) records what it gives.

The [v4vmm mockup](../../../musicindex-research/research_notes/ui-design/mockups/v4vmm.html) shows these selections at research commit `f22ab12`. The mockup is advisory. When the mockup and this record differ, this record is correct.

## Decision

### 1. One Sidebar Holds Each Destination

The sidebar is one list with no group labels. The toolbar has no section tabs.

| Destination | Page |
| --- | --- |
| Search | The saved searches (Decision 4) |
| Releases | The releases, and the start page of the app |
| Artists | The artists and labels |
| My playlists | The playlists of the curator, in the sequence that the curator sets |
| Public playlists | The public playlists of the Index (musicL feeds) |
| Live | What is live now on the Index |

A line follows these destinations. Below the line are Show, Inbox and Settings.

- Show stays a screen mount (ADR 0060). Settings keeps its pages.
- The feed state and "Check All Feeds" stay at the bottom of the sidebar until the status bar of [ADR 0083](0083-design-language.md) Decision 7 takes them.
- "Copy to My Playlists…" is in the item menu of a public playlist.

In a narrow window, the tab bar of R-SHELL-1 item 7 has Search, Releases, Show, Inbox and More. More opens a menu with each other destination.

### 2. Three Destinations Go

- **Discover** is not a destination. Releases takes its "New releases" view, and the Search page takes its saved searches.
- **Tracks** is not a destination. A track stays reachable from a release page, a playlist and the Tracks tab of the results.
- **Needs attention** is not a destination. The Inbox takes its tracks (Decision 3).

### 3. The Inbox Holds What Needs A Decision

The Inbox holds the tracks that a show cannot use as they are, the updates and the setup that a show needs. The sidebar shows its count, in words for the accessibility label, so the count does not rely on color alone. An empty Inbox shows an empty state of musicindex ADR 0026 R-EMPTY-1.

The Inbox lists each track that a show cannot use as it is. The tracks of the show playlist come first.

### 4. The Search Page Lists The Saved Searches

One search field stays in the toolbar on each page (R-TOOLBAR-1, ADR 0087). A click on the Search destination opens the Search page and puts the focus in that field.

- The page lists each saved search, with its query and the count of its new results. "New Saved Search" makes one.
- "How to Search" gives four steps and example queries. The steps show when no search is saved.
- A saved search opens over the list, with the same results as the results page. A click on its query makes it a field, and the results change while the curator types. "Save Query" or Enter keeps the change. Escape cancels it.
- The item menu of a saved search has Edit Query, Rename…, Duplicate and Delete Saved Search…. A saved search keeps its text (ADR 0087 Decision 6).

The results page has two rows below its title:

1. "All | In Library" is part of the query. A click writes `in:library` or removes it, so this control shows no counts.
2. The type tabs are All, Releases, Tracks, Artists and Labels. They select a group of one run and do not change the query, so each tab shows its count.

A selection goes into the query text in three conditions:

- It must stay with a saved search.
- It needs a new run or the Index.
- A field gives the same selection.

In each other condition, it can be a view of one run.

### 5. Each List Page Has Its Defaults

R-LIST-1 gives the form of sort and view. This table gives the defaults (R-LIST-1 item 6). Each page keeps its own sort, view and scope.

| Page | Sort at the start | View at the start | Scope |
| --- | --- | --- | --- |
| Releases | Newest | Rows | All |
| Artists | A to Z | Rows | All |
| My playlists | None. The curator sets the sequence | Rows | None |
| Public playlists | Newest | Tiles | None |
| The results and a saved search | Newest | Rows | In the query (Decision 4) |

The scope of Releases and Artists is the segmented control "All | In Library", below the title at the left. All is the default.

- All holds the Index items and the Library items. Each Library item keeps its state.
- In Library holds what is downloaded, downloading or failed.
- A release that the curator removed shows only in All.
- In Library shows a count. All shows no count, because the Index gives no total. [Open request 13](../plans/v4vmm-open-requests.md) asks for a total.
- With the sort Newest, Releases groups the singles of one artist by the window of ADR 0086. With the sort A to Z, it uses the group of ADR 0084 Decision 6.

### 6. A Row Keeps One Main Action

- Each list row shows its type dot and type word (musicindex ADR 0038 R-DOT-4). A numbered track row on a release page does not, because its section heading gives the type.
- The library icon and the time each have one column, so they align in a list.
- The library icon is muted. A warning or a danger keeps its status color.
- The main action of an album or a track in the Library is Play.
- Adding to a playlist is in the item menu only.
- The selection model of ADR 0084 Decision 10 stays. Its "Select" button moves to the title row of the list page.

### 7. The Now-Playing Panel Is On The Right

The right side holds the panel of R-SHELL-1 item 5, only while something plays or a show is live. With neither, the panel is absent.

- The panel holds the current track, the transport, a seek control and Up Next. The transport and Up Next move there from the Show page.
- Play stays on the current page and opens the panel. It does not move to the Show page.
- The curator can expand and collapse the panel.

ADR 0068 decides the audio route of Play. The panel packet waits for that decision.

### 8. Inspect Sources Is A Sheet

"Inspect Sources" opens as a sheet (R-SHELL-1 item 6). ADR 0083 Decision 8 keeps its content. The app has no general inspector.

### 9. What Stays

- ADR 0084 Decisions 4 to 7 and 10 to 12 stay. Their words "list column" and "detail column" now mean the list page and the item page.
- The one navigation stack of ADR 0084 Decision 9 stays, and R-NAV-2 gives Back and Forward. Its location breadcrumb goes. A page shows the back icons and its title (R-TOOLBAR-1).
- ADR 0087 stays, apart from the place of the saved searches.
- musicindex ADR 0029 R-SCALE-1 item 2 stays: v4vmm uses the macOS values.
- The letter bar is one tab stop. Left, Right, Home and End move a cursor in it, and Enter or Space runs the letter. A move sends no request. This follows the WAI-ARIA toolbar pattern.
- One page shows at a time, so a frame draws no border and no focus outline. The old outline read a stored flag, not the keyboard focus, so it could mark the wrong place. The focused control or row shows the focus (musicindex ADR 0026 R-FOCUS-1). When a list with rows gets the keyboard focus, a row shows it. The walk of 2026-11-06 added this rule.

## Amended Rules

This record changes these ADR 0060 rules:

| ADR 0060 rule | The change |
| --- | --- |
| "The toolbar exposes exactly three app sections" | The sidebar holds each destination (Decision 1) |
| "Search is a command. No app section is a search destination" | Search stays a toolbar command. The Search destination holds the saved searches, not the search (Decision 4) |
| "Queue and transport render only inside `Show`" | They also render in the now-playing panel (Decision 7) |
| "`Music` and `Settings` render no operational surface when no show is active" | The panel shows while something plays. It stays absent when nothing plays (Decision 7) |

"A section that does not apply to the selected mode is absent, not disabled" stays. So do the words `Music` and `Library`, and the rule that `Show` is a screen mount.

## The Contract

The stored contract, version 0.18.1, gives these sources:

| Need | Source | Result |
| --- | --- | --- |
| Releases and Public playlists, Newest | `GET /v1/feeds/recent` with `medium` | Available |
| Releases and Public playlists, A to Z | `GET /v1/feeds/by-name` with `medium` and `from` | Available |
| Artists, A to Z | `GET /v1/artist-names` and `GET /v1/feeds/by-name` | Available |
| Artists, Newest | `GET /v1/feeds/recent` with `medium` gives the feeds. A name has no source | A packet must examine it |
| Live | `GET /v1/live-items` with `status` | Available |
| The count of All | `Pagination` gives only `cursor` and `has_more` | Not available (open request 13) |

## Operator Answers - 2026-11-05

The operator accepted each recommendation:

1. **The breadcrumb goes.** A stack page has one title and Back.
2. **ADR 0068 comes first for the panel.** [ADR 0068](0068-show-cue-and-audition-isolation.md) is Proposed. It reserves Music playback for audition, and audition never sets now-playing state. The operator decides it before the panel packet.
3. **The Inbox lists each track that a show cannot use.** The "Keep the Library ready" job needs the full list. The tracks of the show playlist come first. The mockup lists only the tracks that block the show playlist.
4. **Releases uses two groups.** The window group of ADR 0086 applies with the sort Newest, and the group of ADR 0084 Decision 6 applies with the sort A to Z. The mockup uses only Decision 6.
5. **All shows no count.** The Index gives no total. In Library shows its count, and open request 13 asks for a total.

## Open Points

The research request leaves these open, and this record does not decide them:

- the verbs "Download" or "Get" and "Drop"
- upcoming items in Live
- the other actions of a public playlist
- `in:index` on the results page, which only typed text can reach
- the main row action, shown on each row or only on hover and focus

## Invariants

- No view model exposes the text "Select an item".
- Each destination opens on content or an empty state.
- The now-playing panel renders only while something plays or a show is live.
- Each list page has its own sort, view and scope state.

## Non-Goals

- The new tokens and the design rules ADR. They come after this record, as the overhaul plan orders.
- The folders, moves and renames of My playlists. The operator plans them for later.
- The audition player and the cue of ADR 0068.

## Consequences

Positive:

- Each destination opens on content, and a list has the full width.
- One sidebar replaces the section tabs and the destinations.
- The app, the website and Studio have one shell.

Negative:

- The change is large. Each Music surface changes its layout, and the three-column frames, Discover and the destination Tracks go.
- A list and an item do not show at the same time, so Back has more work. The marked row of R-NAV-2 item 2 makes this smaller.
- The Inbox is a new surface. It needs its own view model.
- The count of the new results of a saved search needs stored state. Its packet adds a migration through the ADR 0016 registry.
- The panel puts a transport on Music pages, so curation is no longer free of playback controls while something plays.

## Alternatives Considered

- **Keep the three columns of ADR 0084.** The list stays at 320 px, and the detail is empty until a selection. The operator selected the stack (musicindex ADR 0027).
- **Keep Discover.** Its first view was the same as Releases. The operator removed it on 2026-10-24.
- **Keep the three section tabs.** The curator then has two navigation levels for one list of places. The operator selected one list.
- **A general inspector on the right.** The operator reserved the right side for the now-playing panel.

## Verification

Mechanical criteria, each at the layer that owns it:

- The sidebar view model exposes the destinations of Decision 1 in that sequence, and Releases is the default.
- The tab bar view model exposes Decision 1 in a narrow window, and More gives each other destination.
- No view model exposes "Select an item". A situational guard that cites this ADR blocks it.
- The list page view model of each page gives the defaults of Decision 5.
- The scope model puts a downloaded, downloading or failed release in In Library, and a removed release only in All.
- A click on the results scope writes or removes `in:library`, and a type tab does not change the query.
- The panel view model is absent while nothing plays and no show is live.
- The navigation model gives R-NAV-2 items 2 to 4, and no view model exposes a breadcrumb.
- The Releases list view model gives window groups with the sort Newest, and Decision 6 groups with the sort A to Z.
- The Inbox view model lists each track that a show cannot use, with the tracks of the show playlist first.

Visual criteria, for the operator, in Light and Dark, at normal and narrow width:

- The app starts on Releases, and an item opens over its list.
- Back returns to the list at the same scroll position, with the row marked.
- The sidebar and the tab bar reach each destination.
- The Search page, the results page and a saved search show their rows in the sequence of Decision 4.
- The panel opens on Play, and the page does not change.
- Inspect Sources opens as a sheet.

## Follow-Up Work

A phase plan divides this record into packets. The packets come before the new tokens, because the stack changes the layout of each surface.
