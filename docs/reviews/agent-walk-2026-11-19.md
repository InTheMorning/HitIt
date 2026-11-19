# Agent Walk Of The Open Checks - 2026-11-19

Status: record of an agent walk. This walk closes no gate. Each visual gate stays open until a person walks it.
[Pending human checks](../pending-human-checks.md) owns each check, and this record gives the agent results.

## Method

- Binary: the release build of HEAD `89f0883`.
- Two walker agents used two Xvfb displays (:91 and :92) with openbox. Each walker had its own scratch `HOME`,
  `XDG_CONFIG_HOME`, `XDG_DATA_HOME` and `XDG_CACHE_HOME`. No walker touched the Library of the operator.
- `allowed_domains` had `api.musicindex.org`, `cdn.kolomona.com` and `sirlibre.com`.
- Walker A walked [ADR 0098 task 001](../tasks/adr-0098-task-001-each-named-feed-keeps-its-own-title.md),
  [ADR 0098 task 002](../tasks/adr-0098-task-002-a-role-row-dot-has-the-color-of-its-first-role.md),
  [ADR 0101 task 015](../tasks/adr-0101-task-015-load-more-shows-the-focus-ring.md), section 16 V3 and the tile ring.
- Walker B walked sections 37, 38 and 39, section 25 V1 for a list row at 390 px, and the count of a singles group.
- The main agent examined the shots of each defect and of these Pass results: task 001 V1 and V2, task 002 V1 in
  Light, and task 015 V1 in Light. This folder keeps only the shots of the defect and the open points, in
  [screenshots/agent-walk-2026-11-19](screenshots/agent-walk-2026-11-19/).

## Defects Found

1. **Tab moves the focus to a hidden button of the restore band at 390 px.** Section 37 V2, in Dark and Light. Tab
   from "Review restore" goes to "Upgrade backup" and then to "Copy database report". The band does not scroll, and
   no ring shows. The wheel scrolls the band (V1 passes).

   The main agent confirmed that the shot shows no ring after
   the Tab. Screenshots: `restore-band-tab-hidden-dark.png`, `restore-band-tab-hidden-light.png`.

   Preparation: the buttons are disabled in a new state. The walker typed two scratch paths in the Restore fields and
   chose "Review restore". The app refused the review, and then "Upgrade backup" and "Copy database report" were
   enabled. The walker did not choose "Restore database".

## Open Points

1. **A singles group gets a "Matched by name" row after a download.** Before the download, the group page of
   "Jon Ross · 8 singles" lists 8 singles and has no "Matched by name" section. After the download of
   "Never Be Another You", the title is "Jon Ross · 9 singles". A "Matched by name" section shows "Rock The Halls".

   The button still says "Download 8 Singles". The main agent confirmed the shots. Screenshots:
   `singles-group-before-download.png`, `singles-group-after-download.png`.

   An inference is that the Library track lets the name match find a by-name single. The count then includes that
   row, and the button counts only the artist feed singles. The operator decides if the count or the button is
   incorrect.
2. **Task 015 V3: the walk cannot prove which tile is new.** Enter on "Load more" loads more tiles, and the ring goes
   to a tile in the second row of the new part. In the sort Newest, the new tiles can sort between the old tiles. The
   code selects the first tile in screen order that the list did not show before the press, and
   `adr_0101_load_more_focuses_the_first_new_row` tests it. Screenshot: `load-more-enter-ring-dark.png`.
3. **A release page title breaks inside a word at 390 px.** "Red Pill Revolution" shows as "Revoluti" and "on". This
   is a detail page, not a list page, so section 39 does not include it. Only walker B saw it, with no shot in this
   folder.

## Results By Item

| Item | Pass | Fail or open |
| --- | --- | --- |
| ADR 0098 task 001 | V1 and V2 after the download of "Motive to Kill": "Label: Sir Libre Records", and the page title "Sir Libre Records" | - |
| Section 16 V3 | The Sir Libre Records page has "Role not confirmed" rows with `rel="recordLabel"` | - |
| ADR 0098 task 002 | V1: the "Ph@fe" row dot and page dot are `#875CCF` in Dark and `#8D62D5` in Light. V2: the Feed dot and the Artist dot keep their colors | - |
| ADR 0101 task 015 | V1 in Light and Dark: the 2 px ring, the fill of Select, and no load in 6 s. V2 in Light and Dark: after a click on Tiles, Tab goes to Select | V3: open point 2 |
| Tile ring | Tab into the Tiles grid shows the ring on the first tile at once, in Light and Dark | - |
| Section 25 V1 | A list row at 390 px has the full 2 px ring, in Light and Dark | - |
| Section 37 | Step 3, V1 and V3 in Light and Dark | V2: defect 1 |
| Section 38 | V1 to V4 in Light and Dark. The status bar has no Tab stop at 390 px, so Tab goes from the list to the tab bar | - |
| Section 39 | V1 to V4 in Light and Dark | - |
| Singles count | The download worked with the allowed hosts | Open point 1 |

Only a walker saw the Pass results that the main agent did not examine. The Method section names the results that the
main agent examined.
