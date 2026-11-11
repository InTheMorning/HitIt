# Research Request: The Shell, The Destinations And The Search Of HitIt

## Status

Partly answered - 2026-11-11. ADR 0089 answers sections 1 to 4. musicindex
ADR 0062 answers the `host:` field of section 4. Five questions of section 5
are open. Each section gives its status.

Received - 2026-10-24. This request comes from the research repository at
`~/build/musicindex-research`. That repository has no Git remote. Its index is
`AGENTS.md`.

This document binds nothing in HitIt. HitIt records each decision in its own
ADR.

**The hold is lifted.** On 2026-10-25 the operator lifted the hold. musicindex
ADR 0027 (the shell, sort, view, the back control and the toolbar), ADR 0028
(the sections and the header actions of an item page, and the animated
cover) and ADR 0029 (the web scale, which HitIt does not use) state the
shared rules. HitIt can now write its own ADRs for sections 1 to 4 of this
request. They cite the musicindex rules and do not restate them.

The research examined HitIt at HEAD `58ce1ab`. The mockups are research
commits `6bacd1c`, `4fedc27`, `8d02b00`, `10fd7eb` and `e2cd3a3`. Commits
`10fd7eb` and `e2cd3a3` give the corrections of sections 2 to 4.

[ADR 0089](../adr/0089-the-shell-destinations-and-list-pages.md), Accepted on 2026-11-05, records sections 1 to 4.

## The Data

| Data | Path from this repository |
| --- | --- |
| The shell analysis and the decision of the operator | `../musicindex-research/research_notes/ui-design/shell-layout-analysis.md` |
| The HitIt mockup | `../musicindex-research/research_notes/ui-design/mockups/v4vmm.html`, with `mockups/v4vmm/` |
| The shared kit of the mockups, with the search | `../musicindex-research/research_notes/ui-design/mockups/kit/` (`search.js` parses the query text of ADR 0087) |
| The decisions that the mockups show | `../musicindex-research/research_notes/ui-design/mockups/README.md` |
| The journeys study of HitIt, the layout reference | `../musicindex-research/sources/captures/2026-10-24-v4vmm-journeys/journeys/` |
| The study compared with the cross-app rules | `../musicindex-research/research_notes/ui-design/v4vmm-journeys-comparison.md` |
| The cross-app rules | `../musicindex/docs/adr/0020-cross-app-interface-rules.md` to `0026-cross-app-controls-and-feedback.md` |
| The search filters of the Index | `docs/plans/hitit-open-requests.md`, request 10 |
| The alternatives for Discover and for the edit of a saved search | `../musicindex-research/research_notes/ui-design/mockups/discover-options.html` |
| Three feeds of the Index in the mockups, with their tracks and covers | `../musicindex-research/sources/captures/2026-10-24-heycitizen-feeds/` |

## 1. The Shell Is A Stack With Two Exceptions

**Section status - 2026-11-11: Answered** by
[ADR 0089](../adr/0089-the-shell-destinations-and-list-pages.md) Decisions 1,
7 and 8. [ADR 0068](../adr/0068-show-cue-and-audition-isolation.md), in its
revision of 2026-11-11, gives the cue and the two players of the panel. ADR
0089 packet 016 builds the panel after ADR 0068 packets 003 and 004.

The operator selected this shell for the three apps on 2026-10-24. It changes
ADR 0084 Decision 1 (three columns).

- The sidebar holds the destinations. A list is a page at the full width of
  the content area. An item opens over the list.
- Back returns to the list at the same scroll position, and marks the row of
  the item that was open.
- No page is a placeholder. Today Releases, Artists, Tracks, Needs attention
  and Playlists open with "Select an item in the list."
  (`src/view_models/workspace/nav.rs:65-69`, `src/view_models/library.rs:813`).
- Exception 2: the right side is reserved for an expandable now-playing
  playlist. It shows only while something plays or a show is live. It holds
  the current track, the transport, a seek control and Up Next. In a narrow
  window it is a bar above the tab bar. Up Next and the transport move there
  from the Show page.
- Exception 1, an outline in an editor, does not apply to HitIt.
- Inspect Sources opens as a sheet. No general inspector exists.

## 2. The Destinations

**Section status - 2026-11-11: Answered** by
[ADR 0089](../adr/0089-the-shell-destinations-and-list-pages.md) Decisions 1
to 3 and 6. Folders, moves and renames in My Playlists stay a Non-Goal of ADR
0089.

The operator corrected this section on 2026-10-24. It changes ADR 0084
Decision 2 (the destinations), Decision 3 (Music opens on Discover) and
Decision 8 (two scopes).

On 2026-11-05 the operator decided that the sidebar items and the page titles
are "My playlists" and "Public playlists" (musicindex ADR 0023 R-CASE-1 item
2). This section keeps the words that the research received. A menu item keeps
title case, for example "Copy to My Playlists…".

- The sidebar is one list with no group labels: **Search, Releases, Artists,
  My Playlists, Public Playlists, Live**. Then Show, Inbox and Settings
  below the line.
- **Needs attention is not a destination.** It showed the same tracks as the
  Inbox section for the show playlist. The Inbox keeps them. The operator
  decided this on 2026-10-24.
- **Discover is not a destination.** Its "New releases" view was the same as
  Releases with All and Newest. Releases is the start page.
- **Search** opens the saved searches, and puts the focus in the toolbar
  field. Section 4 gives the page.
- **Tracks is not a destination.** Tracks stay reachable from a release page,
  a playlist and the Tracks tab of the results.
- **My Playlists** holds the playlists of the curator in the sequence that the
  curator sets. The operator plans folders, moves and renames, as in a file
  manager. The mockup does not show folders.
- **Public Playlists** lists the public playlists of the Index (musicL feeds),
  as the website does. Its item menu has "Copy to My Playlists…".
- **Live** lists what is live now on the Index, as the website does. Red is
  only for an item that is live now (musicindex ADR 0012 item 3).
- **Play stays on the current page** and opens the now-playing playlist. It
  does not move to the Show page.
- Adding to a playlist is in the item menu only. A row has no "+ Episode 42".
  The main action of an album or a track in the Library is Play.

## 3. Sort, View And Scope

**Section status - 2026-11-11: Answered** by
[ADR 0089](../adr/0089-the-shell-destinations-and-list-pages.md) Decisions 5
and 6, and its amendment of 2026-11-08 for the Curator Order.
[ADR 0090](../adr/0090-one-list-bar-above-the-rows.md) moves the sort
controls into one list bar.

- **Sort** is a plain button that names the current sort, for example
  "Newest ⌄". It opens a menu with "Newest" and "A to Z", with a check mark
  on the current one. A to Z also shows the letter bar.
- **View** is two icons, tiles and rows. The current icon has a tint.
- Sort and view are at the right end of the title row. They are not a scope,
  so they do not use the segmented control (musicindex ADR 0023 R-SCOPE-1).
- **Scope** "All | In Library" is a segmented control below the title, at the
  left, with a count for each of the two. All is the default. In Library holds
  what is downloaded, downloading or failed. A dropped release shows only in
  All, so the mockup has no "Dropped" link.
- Each page keeps its own sort, view and scope.

| Page | Sort | View at the start | Scope |
| --- | --- | --- | --- |
| Releases | Newest | Rows | All |
| Artists | A to Z | Rows | All |
| My Playlists | None. The curator sets the sequence. | Rows | None |
| Public Playlists | Newest | Tiles | None |
| The results and a saved search | Newest | Rows | In the query, section 4 |

- **Each row shows the entity dot and the type word**, for example "Album",
  "Track" or "Artist" (musicindex ADR 0012 item 8). A numbered track row on a
  release page does not, because the section heading gives the type.
- **The library icon is muted.** A warning or a danger keeps its status
  color. The icon and the time each have one column, so they align in a
  list of releases and tracks.

## 4. The Search

**Section status - 2026-11-11: Answered.**
[ADR 0089](../adr/0089-the-shell-destinations-and-list-pages.md) Decision 4
and [ADR 0087](../adr/0087-search-fields-and-live-results.md) answer each
item, apart from the `host:` field. musicindex ADR 0062 answers the `host:`
field (R-QUERY-13 to R-QUERY-17). By
[ADR 0091](../adr/0091-search-parser-moves-to-the-shared-crate.md), musicindex
owns the syntax. The `musicindex-search` crate at `4761148` gives the field.
HitIt sends `host` and `not_host` to Stophammer (open request 10 item 7), and
it filters the Library rows by the host of the feed URL.

- **One wide field in the center of the toolbar, on each page**, as ADR 0060
  and ADR 0087 state. It is at most 640 px wide. Its width does not change
  when it gets the focus. The operator does not want the animation.
- **The toolbar** has the `‹ ›` icons and the page title at the left, as
  musicindex ADR 0020 R-NAV-1 gives for HitIt. The website uses the same
  form.
  The field list of ADR 0087 Decision 3 opens under it. ⌘K and ⌘F focus it.
- The website uses the same field and the same query text, with the Index
  fields only.
- The pause is **300 ms** in each app. musicindex ADR 0026 R-SEARCH-1 now
  gives 300 ms, the value of ADR 0087.
- **Request: a `host:` field.** On 2026-10-24 the operator asked for a search
  by host. It includes or excludes the feeds of one hosting service, for
  example `host:wavlake.com` and `-host:wavlake.com`. Request 10 item 7 asks
  Stophammer for `host` and `not_host`. ADR 0087 Decision 2 needs an
  amendment for the field when HitIt writes ADRs again.
- **A selection goes into the query text in three conditions.** It must stay
  with a saved search, it needs a new run or the Index, or a field gives the
  same selection. In all other conditions, it can be a view of one run.
- **"All | In Library" on the results is part of the query.** A click writes
  `in:library` or removes it (ADR 0087 Decision 2), so it shows no counts. It
  is the first row below the title.
- **The type tabs** (All, Releases, Tracks, Artists, Labels) are the second
  row. They select a group of one run and do not change the query, so each
  tab shows its count.
- **The Search page** lists the saved searches, each with its query and the
  count of its new results. "New Saved Search" makes one. "How to Search"
  gives four steps and example queries. The steps show when no search is
  saved.
- **A saved search** opens over the list with the same results as the
  results page. A click on its query makes it a field, and the results change
  while the curator types. "Save Query" or Enter keeps the change. Escape
  cancels it.
- The item menu of a saved search has Edit Query, Rename…, Duplicate and
  Delete Saved Search…. ADR 0087 Decision 6 stays: a saved search keeps its
  text.
- The mockup parser gives the results that the mechanical criteria of ADR 0087
  name, for example five terms for
  `artist:jsmovie kind:single after:2026-10-01 -title:remix "night drive"`.

## 5. Open Questions For The Operator

**Section status - 2026-11-11: Open.** Question 2 is Answered by ADR 0089
Decision 3. Questions 1, 3, 4, 5 and 6 are Open: they wait for an operator
decision. ADR 0089 lists them in its Open Points, and the app keeps its
present behavior until then. For question 4, ADR 0089 Decision 1 decides
"Copy to My Playlists…", and the other actions stay open.

The operator has not decided these:

1. The verbs: ADR 0083 says "Download". The journeys study says "Get" and
   "Drop".
2. Answered on 2026-10-24: one Inbox. Needs attention is not a destination.
3. Does Live also list upcoming items, as the website does?
4. The actions of a public playlist, for example "Copy to My Playlists…".
5. "All | In Library" on the results page maps to no `in:` field and to
   `in:library`. `in:index` is reachable only when the curator types it.
6. The row actions show only on hover or keyboard focus (ADR 0083). The
   study shows the main row action on each row.
