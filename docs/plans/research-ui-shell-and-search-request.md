# Research Request: The Shell, The Destinations And The Search Of v4vmm

## Status

Received - 2026-10-24. This request comes from the research repository at
`~/build/musicindex-research`. That repository has no Git remote. Its index is
`AGENTS.md`.

This document binds nothing in v4vmm. v4vmm records each decision in its own
ADR.

**Wait before an ADR.** On 2026-10-24 the operator decided that v4vmm writes
no ADR from this request yet. First the operator fixes the research mockups.
Then a musicindex ADR codifies the shared rules. Read the request now. Do not draft
an ADR yet.

The research examined v4vmm at HEAD `58ce1ab`. The mockups are research
commits `6bacd1c`, `4fedc27` and `8d02b00`.

## The Data

| Data | Path from this repository |
| --- | --- |
| The shell analysis and the decision of the operator | `../musicindex-research/research_notes/ui-design/shell-layout-analysis.md` |
| The v4vmm mockup | `../musicindex-research/research_notes/ui-design/mockups/v4vmm.html`, with `mockups/v4vmm/` |
| The shared kit of the mockups, with the search | `../musicindex-research/research_notes/ui-design/mockups/kit/` (`search.js` parses the query text of ADR 0087) |
| The decisions that the mockups show | `../musicindex-research/research_notes/ui-design/mockups/README.md` |
| The journeys study of v4vmm, the layout reference | `../musicindex-research/sources/captures/2026-10-24-v4vmm-journeys/journeys/` |
| The study compared with the cross-app rules | `../musicindex-research/research_notes/ui-design/v4vmm-journeys-comparison.md` |
| The cross-app rules | `../musicindex/docs/adr/0020-cross-app-interface-rules.md` to `0026-cross-app-controls-and-feedback.md` |
| The search filters of the Index | `docs/plans/v4vmm-open-requests.md`, request 10 |

## 1. The Shell Is A Stack With Two Exceptions

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
- Exception 1, an outline in an editor, does not apply to v4vmm.
- Inspect Sources opens as a sheet. No general inspector exists.

## 2. The Destinations

- Music: Discover, Releases, Artists, Playlists, Live, Needs attention. Then
  Show, Inbox and Settings.
- **Tracks is not a destination.** Tracks stay reachable from a release page,
  a playlist and the Tracks scope of search.
- **Playlists has two scopes, "Yours" and "Public".** Yours is the default.
  Public lists the public playlists of the Index (musicL feeds), as the website
  does.
- **Live** lists what is live now on the Index, as the website does. Red is
  only for an item that is live now (musicindex ADR 0012 item 3).
- **The start page is Discover**, with the new releases as tiles.
- **Play stays on the current page** and opens the now-playing playlist. It
  does not move to the Show page.
- Adding to a playlist is in the item menu only. A row has no "+ Episode 42".
  The main action of an album or a track in the Library is Play.

## 3. The Search

- **One wide field in the toolbar of the content area, on each page**, as
  ADR 0060 and ADR 0087 state. It is 360 px, and 520 px while it has focus.
  The field list of ADR 0087 Decision 3 opens under it. ⌘K and ⌘F focus it.
- The website uses the same field and the same query text, with the Index
  fields only.
- The pause is **300 ms** in each app. musicindex ADR 0026 R-SEARCH-1 now
  gives 300 ms, the value of ADR 0087.
- **Request: a `host:` field.** On 2026-10-24 the operator asked for a search
  by host. It includes or excludes the feeds of one hosting service, for
  example `host:wavlake.com` and `-host:wavlake.com`. Request 10 item 7 asks
  Stophammer for `host` and `not_host`. ADR 0087 Decision 2 needs an
  amendment for the field when v4vmm writes ADRs again.
- The mockup parser gives the results that the mechanical criteria of ADR 0087
  name, for example five terms for
  `artist:jsmovie kind:single after:2026-10-01 -title:remix "night drive"`.

## 4. Open Questions For The Operator

The operator has not decided these:

1. The verbs: ADR 0083 says "Download". The journeys study says "Get" and
   "Drop".
2. One Inbox, or the Inbox and the Needs attention destination.
3. Does Live also list upcoming items, as the website does?
4. The actions of a public playlist, for example "Copy to Yours…".
5. "Library | Everything" on the results page maps to `in:library` and to no
   `in:` field. `in:index` is then reachable only by typing it.
6. The row actions show only on hover or keyboard focus (ADR 0083). The
   study shows the main row action on each row.
