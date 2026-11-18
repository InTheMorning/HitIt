# Second Agent Walk Of The Open Checks - 2026-11-18

Status: record of an agent walk. This walk closes no gate. Each visual gate stays open until a person walks it.
[Pending human checks](../pending-human-checks.md) owns each check, and this record gives the agent results.

## Method

- Binary: the release build of HEAD `343f5ab`.
- Three walker agents used three Xvfb displays (:91 to :93) with openbox. Each walker had its own scratch
  `HOME`, `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_CACHE_HOME` and music folder. No walker touched the
  Library of the operator.
- The Library rows came from `fountain.fm` feeds.
- The walk took the items that the [first walk of 2026-11-18](agent-walk-2026-11-18.md) left open for an agent.
- The main agent read each report. It examined the screenshot of each Fail, each new defect and each observation.
  This folder keeps only those screenshots, in
  [screenshots/agent-walk-2026-11-18-second](screenshots/agent-walk-2026-11-18-second/).
- W3 wrote a "Removed from feed" mark into its scratch database with `sqlite3`, because no feed removed an item. The
  R-ACT-10 result uses that mark.

## Defects Found

1. **The restore band is cut at 390 px.** In Settings, Diagnostics, Database, at 390 px, the button band shows two
   rows. The third row ("Copy database report") is cut at the bottom edge of the band. The band does not scroll.

   At 480 px, the three rows show. The main agent confirmed it. Screenshot: `restore-band-390-cut.png`.
2. **At 390 px, the Releases list is not a Tab stop.** In the Rows view, Tab goes from "In Library" to "Search" in
   the bottom bar. The main agent confirmed it. Screenshot: `rows-390-tab-skips-list.png`.
3. **At 390 px, the Releases title breaks.** The title shows "Relea" and "ses" on two lines. In the Tiles view, the
   tiles are one column in the left half of the window. The main agent confirmed it. Screenshot:
   `tiles-390-title-breaks.png`.

## Not Confirmed

- **Tab reaches "Load more" before the tiles.** In the Tiles view of Releases, the first Tab after the Tiles button
  moved the grid to its end. Return then loaded more tiles. Thus the walker thinks that the focus was on "Load more".

  After shift+Tab and Return, the first release of the list opened, but no ring showed on it. The screenshot
  `tiles-tab-reaches-load-more.png` shows the grid at its end with no ring. It cannot prove which control has the
  focus.
- **A band above the pinned list bar at 480 px.** The pinned bar shows a white band of about 30 px above its
  buttons. Screenshot: `pinned-bar-480-band.png`. A person must decide if this is correct.

## Questions For The Operator

1. Section 33 V2 says: "An open playlist entry is white in its gray group." ADR 0101 task 012 gives the unavailable
   playlist row the open entry level. In Light, that row is white (`#ffffff`) on the white page. No gray group is
   around it, and only its border shows it.

   Screenshot: `unavailable-row-light.png`. The operator decides if the
   check text, the level of the row, or nothing changes.
2. W1 saw these items. They are not in the check text:
   - The search field in the toolbar keeps the last query on other pages.
   - The single "Wake Up" shows a download icon in the results of a saved search with `in:library`. The Releases
     scope "In Library" lists it.

## Results By Section

| Section | Pass | Open for the agent | Open for a person |
| --- | --- | --- | --- |
| 11 | Light 900 px: 002 V2, 004 V3, 006 V2, 007 V2, 009 V3, 014 V2. 007 V3: "3 new" showed and went after an open (`saved-search-new-count.png`) | - | As in the walk of 2026-11-17 |
| ADR 0090 task 002 | Light at 1440, 900 and 480 px. A click on the empty part of the bar does nothing | A row action with the bar pinned, in Light | The band at 480 px |
| 13 | Light: the Library remove dialog, the playlist selection bar, Inspect Sources with a staged edit, R-ACT-10 (with the written mark), R-ACT-11 at 1440, 700, 480 and 390 px, the playlist delete dialog and the playlist page bar | - | The focus of Cancel when a dialog opens. Defect 1 |
| 15 | - | V3: a search failure needs a failed network request | - |
| 16 | V1 heading "Artists on these releases" | V2: no page with two known roles was found. V3: no Sir Libre track can enter the Library | - |
| 17 | V3 in Rows: after "Show more", the ring is on the first new row | - | - |
| 25 | V1 for a list row in Light at 1120 px | V1 for a tile (see "Not Confirmed"), a list row at 390 px (defect 2), Light at 390 px | - |
| 33 | - | - | V2: question 1 |
| 35 | Light at 1440 and 390 px on the About tab of `343f5ab`: the line, a drag selection, Copy | - | The Diagnostics row of `bebaf08` in Light. All visual criteria |

Section 35 changed after this walk. In `bebaf08`, the version line is the first row of Diagnostics, and the About tab
is gone. The main agent checked that row in Dark at 1440 and 390 px.
