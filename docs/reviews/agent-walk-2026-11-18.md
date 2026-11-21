# Agent Walk Of The Open Checks - 2026-11-18

Status: record of an agent walk. This walk closes no gate. Each visual gate stays open until a person walks it.
[Pending human checks](../pending-human-checks.md) owns each check, and this record gives the agent results.

## Method

- Binary: the release build of HEAD `c655762`. No source file changed after the build of 2026-11-17 21:51.
- Three walker agents used three Xvfb displays (:86 to :88) with openbox. Each walker had its own scratch
  `HOME`, `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_CACHE_HOME` and music folder. No walker touched the
  Library of the operator.
- The app calls declared `api.musicindex.org`, `fountain.fm`, `headstarts.uk` and `op3.dev`. The Library rows came
  from `fountain.fm` feeds.
- The walk took the items that the [walk of 2026-11-17](agent-walk-2026-11-17.md) left open for an agent.
  It also took the sections that had no agent walk after their correction of 2026-11-15.
- The main agent read each report and examined the screenshot of each Fail and each new defect. This folder keeps
  only those screenshots, in [screenshots/agent-walk-2026-11-18](screenshots/agent-walk-2026-11-18/).

## Defects Found

1. **The section shortcuts stop at 900 px.** On Show and on Settings, ctrl+1, ctrl+2 and ctrl+3 do nothing. A click
   on the page does not help. The shortcuts work again after More opens and closes. This is section 11, packet 003
   V3.

   Inference: the app moves the focus to a sidebar row, and no sidebar shows at 900 px. The screenshot
   `shortcuts-stop-900.png` agrees with the report. It cannot prove which keys the app got.
2. **A Space closes the New Playlist popover.** The walker typed "W1 test" in "Playlist name". The popover closed,
   and the field held "W1". This occurs from the pinned list bar and from the page header. Screenshot:
   `space-closes-popover.png`.
3. **The Library selection bar has no space after the remove button.** On a Library release page, the bar shows
   "Done", the count, then "Remove 2 tracks…". The space after the remove button is the same as the other spaces.

   The playlist selection bar has the space. Section 13 needs it. The main agent confirmed it. Screenshot:
   `library-selection-bar-gap.png`.

## Not Confirmed

- **Two rings on Artists.** After Enter on an Artists row, Back and a sidebar click, the marked row shows a blue
  border and an accent bar. At the same time, the Tiles button has the focus ring. This can be the fault of
  2026-11-15 of ADR 0092 task 001 V1 in a different condition. It can also be the usual mark of the last opened row.

  Screenshot: `artists-two-rings.png`.
- **No ring on a Releases tile.** In Dark at 940 px, no tile and no "Load more" showed a ring. The screenshots do
  not prove where the focus is. This agrees with the third walk of 2026-11-17.
- **No ring at 390 px in Settings.** Tab showed no ring in 30 stops until the walker activated the window. The cause
  can be a lost window focus.

## Questions For The Operator

These items come from the text of section 13. The operator decides if the check text or the app changes.

1. The tag dialog ("Update 2 files?") has "Cancel" at the left and "Write Tags" at the right. That is the order of a
   primary dialog. Section 13 says that the tag dialog has the order of the destructive dialogs.
2. In Settings, the red "Use Defaults…" is at the right end of its row. R-ACT-12 puts it there. The "Wrong" line of
   section 13 forbids a red button at the right end of a row. R-ACT-10 also puts the red "Remove…" at the right end
   of its strip.
3. The Settings line of section 13 ("Use Defaults" left, "Save" right) names an older layout.

## Results By Section

| Section | Pass | Open for the agent | Open for a person |
| --- | --- | --- | --- |
| 11 | Light 1440 px and Dark 900 px: 002 V2, 004 V3, 006 V2, 007 V2, 009 V3, 014 V2. Light 1440 px: 003 V3. 006 V4 at 900 px. 007 V4: the More menu opens with no delay. **Fail: 003 V3 at 900 px (defect 1)** | Light 900 px. 007 V3 (no count showed) | As in the walk of 2026-11-17 |
| ADR 0090 task 002 | Dark 1440 px: the rows do not jump when the bar pins, "Add to playlist" from the pinned bar, a row action with the bar pinned, the window group page and the singles group page | Light, a narrow window, a click on the empty part of the bar | - |
| 13 | Dark: the Library remove dialog, the playlist page bar, the playlist selection bar, Inspect Sources with a staged edit, R-ACT-10, R-ACT-11 at three widths. Dark and Light: R-ACT-12. Light: the playlist page bar and the delete dialog. **Fail: the Library selection bar (defect 3)** | Light for the other items | The focus of Cancel when a dialog opens, and the three questions above |
| 15 | V1 (the Inbox row), V2, V4 | V3: no notice page with buttons | - |
| 16 | V1 for the badge and the title, V4 and V5 on Sir Libre Records | V1 heading "Artists on these releases", V2, V3 | - |
| 17 | V1 with the focus on the sort menu, V2, V3 in Tiles | V3 in Rows. The two rings above | - |
| 25 | V1 for the dialog buttons and a page button at 390 px | V1 for a tile, a list row at 390 px and Light | - |
| 33 | - | V2: the walker examined a list row of My playlists. V2 is about the unavailable row of a playlist page, which has the open entry level | - |
