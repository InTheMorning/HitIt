# Third Agent Walk Of The Open Checks - 2026-11-18

Status: record of an agent walk. This walk closes no gate. Each visual gate stays open until a person walks it.
[Pending human checks](../pending-human-checks.md) owns each check, and this record gives the agent results.

## Method

- Binary: the release build of HEAD `d323ffd`.
- Two walker agents used two Xvfb displays (:91 and :92) with openbox. Each walker had its own scratch `HOME`,
  `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_CACHE_HOME` and music folder. No walker touched the Library of the operator.
- The walk took the items that the [second walk of 2026-11-18](agent-walk-2026-11-18-second.md) left open for an
  agent. It also took the "Agent check" lines of sections 35 and 36.
- The main agent read each report. It examined the screenshots of each new defect and of each Pass that closes an
  agent item. This folder keeps only the screenshots of the defects and the open points, in
  [screenshots/agent-walk-2026-11-18-third](screenshots/agent-walk-2026-11-18-third/).
- The main agent found the test data for section 16 in the live MusicIndex API. The publisher feed "Ph@fe"
  (`a9ce0ee9-fe9f-4495-b7b8-ebae25ff2112`) has the agreed roles `host`, `label` and `studio`. It was the only feed
  with two known roles in the `/v1/artist-names?include=link_facts` list.
- The audio of the Sir Libre releases is on `cdn.kolomona.com`. The earlier walks allowed only
  `api.musicindex.org`, so their download failed. With `cdn.kolomona.com` and `sirlibre.com` in `allowed_domains`, the
  download of "Motive to Kill" (10 tracks) worked.
- In this session, `api.musicindex.org` was reachable without `allowed_domains`. For the search failure, walker A
  started the app with `HTTPS_PROXY`, `HTTP_PROXY` and `ALL_PROXY` set to `http://127.0.0.1:9` for that process only.

## Defects Found

1. **The Sir Libre Records page takes the name of a release after a download.** Before the download, the release page
   of "Motive to Kill" shows "Label: Sir Libre Records". After the download, it shows
   "Label: 34bf795f-f551-4db1-8877-ad19aa5f89c8". The Sir Libre Records page shows the title "Motive to Kill". The main
   agent confirmed it. Screenshots: `label-guid-after-download.png`, `publisher-page-after-download.png`.

   A probable cause is that the RSS data of the download replaces the MusicIndex name. This is an inference.
2. **The "Use Defaults" dialog is cut at 390 px.** In Settings, General, in Light, the title ("se the default
   settings?") and the text are cut at the left edge of the window. The main agent confirmed it. Screenshot:
   `use-defaults-dialog-390-cut.png`. [ADR 0097 task 005](../tasks/adr-0097-task-005-sheets-dialogs-and-menus-fit.md),
   which is Ready, checks the fit of the dialogs.
3. **"Load more" takes the Tab focus and shows no ring.** In the Tiles view of Releases, after a click on the Tiles
   button, the first Tab moved the grid to its end. Return then loaded more tiles, and no release opened. "Load more"
   shows no ring in Dark and in Light, and it looks dimmed.

   The main agent confirmed the screenshot. Screenshot:
   `load-more-focus-no-ring.png`.

## Seen By One Walker

The main agent did not confirm these items.

- In the Tiles view, Tab to the list shows no ring on a tile. The ring shows only after an arrow key. ADR 0098
  Invariants say: "When a list with rows gets the keyboard focus, a row shows it."
- After a download of one single on "Jon Ross - 8 singles", the title of the group page changed to "9 singles".

## Answer: Where Tab Goes After The Tiles Button

The second walk could not prove where the focus goes after the Tiles button. The code and walker B agree:

- Code: each page header button has Tab index 0 (`Button::new`, `src/ui/primitives/button.rs`). The list column has Tab
  index 1 (`render_list_column`, `src/library/app_impl.rs`). "Load more" has Tab index 1 after the list
  (`action_button`, `src/ui/shells/library/index_list_status.rs`). GPUI sorts the Tab stops by index, then by
  insertion order.
- Shot: with the keyboard from the top, walker B saw Newest, List, Tiles, Select, All, In Library, then two stops in
  the grid area with no ring, then the sidebar.
- Thus, from the Tiles button, Tab goes to Select. After a pointer click on Tiles, the first Tab went to "Load more".
  An inference is that a click on Tiles does not move the focus to it, and the focus stayed on the list.

## Questions For The Operator

1. Section 16 V3 says: "Inspect sources of Sir Libre Records still shows `recordLabel`." Inspect Sources opens only from
   a track. On the track "Rapeseed" of "Motive to Kill", the sheet has an RSS column only, and no row shows a role
   word. `inspect_sources_sheet.rs` has no role field.

   `rel="recordLabel"` shows on the publisher page, in the "Role
   not confirmed" rows of V5. Screenshot: `inspect-sources-no-role.png`. The operator decides if the check text or the
   sheet changes.
2. The dot of the "Ph@fe" row in the search results has the feed color (`#C4965B`). The dot of its page has the
   label color (`#875CCF`). Screenshot: `role-search-row-dot.png`. The operator decides if the row dot follows the
   first role.
3. In Light at 480 px, the pinned list bar shows a white band of about 30 px above its buttons. The second walk saw it
   in Dark. Screenshot: `pinned-bar-480-band-light.png`. A person decides if this is correct.

## Results By Section

| Section | Pass | Open for the agent | Open for a person |
| --- | --- | --- | --- |
| ADR 0090 task 002 | Light: a row download and "Add to playlist" from the pinned bar, with the bar pinned | - | The band at 480 px (question 3) |
| 15 | V3 at 600 px and at 330 px high: the notice "Music Index search unavailable", Tab to "Copy report" with its full ring | - | - |
| 16 | V2 on "Ph@fe": "Label · Host", the label color on the page dot, no "studio" | - | V3: question 1. The row dot: question 2 |
| 25 | V1 for a tile in Dark and Light at 940 px, after an arrow key. Light at 390 px: the page buttons and the dialog buttons | A list row at 390 px waits for [ADR 0092 task 004](../tasks/adr-0092-task-004-the-list-comes-before-the-tab-bar.md) | - |
| 35 | Light at 1440 and 390 px on the Diagnostics row of `d323ffd`: V1, a drag selection, Copy, step 7 | - | All visual criteria |
| 36 | Light at 1440 and 390 px: V1 to V3. "Delete" on one scratch cache file at 390 px: the count went to 0 and the file went from the cache folder | - | All visual criteria |
