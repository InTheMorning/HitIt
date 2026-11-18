# ADR 0074 Task 001: The Restore Band Scrolls At 390 px

Status: Implemented - operator visual checks open, 2026-11-18. The orchestrator asked for this packet on 2026-11-18, with the approval of the operator.

## Goal

This packet applies to Settings, Diagnostics, Database, at a window width of 390 px. Each button of the action band shows, or comes into view when the band scrolls.

## Authority

- [ADR 0074](../adr/0074-repair-and-diagnostics-pages.md): at narrow widths, the actions occupy a scrollable band above the content.
- [ADR 0097](../adr/0097-the-fit-check-at-two-widths.md): the narrowest window width is 390 px.
- [ADR 0101](../adr/0101-the-design-language-follows-the-shared-rules.md) Decision 5 gives the sequence of the buttons in the band. This packet does not change the sequence.

## Defect

The [second agent walk of 2026-11-18](../reviews/agent-walk-2026-11-18-second.md), defect 1: at 390 px the band shows two rows of buttons. The third row ("Copy database report") is cut at the bottom edge of the band. The band does not scroll. At 480 px the three rows show. Screenshot: `docs/reviews/screenshots/agent-walk-2026-11-18-second/restore-band-390-cut.png`.

## Probable Cause

This is an inference from the code. The implementer confirms it with the failing test first.

- `MaintenancePage::render_layout` (`src/ui/composites/maintenance_page.rs`) gives the band `overflow_y_scrollbar()` and, in the stacked layout, `max_h(relative(MAINTENANCE_ACTION_BAND_FRACTION))` (0.25, `src/ui/layouts.rs`).
- The gpui-component scroll wrapper (`scroll/scrollable.rs`, `root_style_from` and `Scrollable::render`) copies the maximum size to its outer element and keeps it on the inner element. Thus the inner content can have the same height limit as the viewport, and the scroll range does not include the third row.
- `database_tools` (`src/ui/composites/maintenance_forms.rs`) adds Cancel and "Copy database report" after the task actions. At 390 px these wrap to a third row.

## Change

- Keep the height limit of the band on the viewport only. Give the scroll content its full height, so the band scrolls to its last row.
- Make the change in `MaintenancePage::render_layout`, the shared owner of the band. Do not change `database_tools`, the sequence of the buttons or the band fraction.
- If the cause is different, stop and report the cause before a different change.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Maintenance page band | extend | src/ui/composites/maintenance_page.rs | The shared owner of the band of each maintenance page. |
| Database tools | reuse | src/ui/composites/maintenance_forms.rs | It gives the buttons of the Database page. No change. |
| Band fraction | reuse | src/ui/layouts.rs | `MAINTENANCE_ACTION_BAND_FRACTION` stays 0.25. |
| Scroll content | reuse | src/ui/composites/page_scroll_content.rs | The content element of the band. |

## Mechanical Criteria

- A new GPUI test `adr_0074_task_001_the_restore_band_scrolls_at_390_px` in `src/ui/composites/maintenance_page.rs` draws the Database page in the stacked layout at 390 px with the buttons of `database_tools`. After a scroll of the band to its end, the bounds of the last button are fully inside the bounds of the band. The test fails before the change.
- `adr_0074_short_windows_reserve_space_for_task_content` and `adr_0074_reports_do_not_capture_instruction_scrolling` pass without a change.
- The commit checks of AGENTS.md pass.

## Visual Criteria

- V1: in Light and Dark at 390 px, Settings, Diagnostics, Database: the band scrolls with the wheel and shows "Copy database report" fully.
- V2: Tab from the first button of the band to "Copy database report". The band scrolls to show each focused button with its full focus ring.
- V3: at 480 px and at 1440 px the band and the action column look as before.
- Incorrect: a button that is cut and does not come into view, or a band that is higher than before at 480 px.

## Not In This Packet

- The other maintenance pages use the same band. The test covers the shared owner. A person checks only the Database page.
