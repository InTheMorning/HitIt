# ADR 0101 Design Language Phase Plan

## Status

Active - 2026-11-15. This plan is advisory. It states no rule. [ADR 0101](../adr/0101-the-design-language-follows-the-shared-rules.md) owns each rule of HitIt, and musicindex owns the text of each shared rule that ADR 0101 adopts.

## Goal

The surfaces of HitIt obey the shared rules that ADR 0101 Decision 11 adopts. Each difference in the table of ADR 0101 Decision 12 gets one packet. After the last packet, each section at M agrees with musicindex.org in hierarchy, color roles and artwork, in Dark and in Light.

## Non-Goals

- No new screen. The new screens of the [overhaul plan](design-and-cleanup-overhaul-plan.md#remaining-order---2026-10-23) come after this phase.
- No change of a shared rule. A change needs a superseding record in `../musicindex`.
- No rename of a token role. The token packet keeps each role name.
- The R-CHECK-5 token test. It waits for the check box tokens in musicindex-assets.

## Sources

- The shared rule text: the orchestrator session sent it on 2026-11-15 as a copy of musicindex commit `785ae28` and musicindex-assets commit `b8399cc`. ADR 0101 adopts the rules at musicindex `3baa6f0`. Each packet cites the musicindex path and the line range at `785ae28`. If the two commits differ in a rule, the packet stops and asks.
- The code facts: a ChatGPT search at HitIt `ad1fcd9` on 2026-11-15. The main session examined the facts that each packet uses in its "Recorded Facts".
- No shared rule and no shared token gives a number for the glow blur, the saturation, the grain or the veil. None gives the art shadow of each size. The website code (`search.html` at musicindex `785ae28`) has values. The glow packet copies them, and it says so.

## Sequence

ADR 0101 Decision 12 gives the sequence. The packet numbers follow it.

| Packet | Content | Shared rule | State |
|---|---|---|---|
| [001](../tasks/adr-0101-task-001-shared-tokens.md) | The base tokens: focus color, type sizes, radii, spacing, light levels, the ring test | R-TOKEN-1, R-TYPE-1, R-TYPE-2, R-RADIUS-1, R-SPACE-1, R-ELEVATION-2, R-FOCUS-2 | Waits for the three defect fixes of 2026-11-14 and for [ADR 0102 task 001](../tasks/adr-0102-task-001-tiles-arrow-keys.md) |
| [002](../tasks/adr-0101-task-002-header-glow.md) | The page header glow with grain and a veil. An animated cover moves in the glow | R-ART-5, R-ART-6 | Waits for 001 |
| [003](../tasks/adr-0101-task-003-art-radius.md) | The art radius of 6 % of the edge. A larger image has a larger shadow | R-ART-4 | Waits for 001 |
| [004](../tasks/adr-0101-task-004-payment-split.md) | The payment split section of a track page and a release page | R-VALUE-1, R-VALUE-2, R-VALUE-5, R-ADDRESS-1 | Waits for 001 |
| [005](../tasks/adr-0101-task-005-system-accessibility-settings.md) | The app reads Reduce Motion, Increase Contrast and Reduce Transparency from the system | R-A11Y-1 | Waits for 001 |
| [006](../tasks/adr-0101-task-006-dates-in-the-system-language.md) | Dates in the language of the system. Durations and total times | R-DATE-1 | Waits for 001 |
| [007](../tasks/adr-0101-task-007-loading-states.md) | The skeleton sheen, the spinner with a busy label, the progress bar with a count | R-LOADING-1 | Waits for 005 |
| [008](../tasks/adr-0101-task-008-status-bar-and-notices.md) | One status bar replaces the live status band. A result shows where the action was. No toast | R-NOTICE-1, ADR 0101 Decision 7 | Waits for 001 |
| [009](../tasks/adr-0101-task-009-open-on-musicindex.md) | "Open on musicindex.org" on a release, track and artist page | R-LINK-1, R-PAGE-2 item 1 | Waits for 001 |
| [010](../tasks/adr-0101-task-010-selected-look.md) | The selected look: a gradient, an inner edge and a highlight | R-SELECT-1 | Waits for 001 |
| [011](../tasks/adr-0101-task-011-main-action.md) | The main action in a glow header uses the text color. No next curation step is filled | R-ACT-7, R-PAGE-2 item 2 | Waits for 002 |
| [012](../tasks/adr-0101-task-012-light-levels-on-surfaces.md) | The group level and the open entry level on the surfaces. The floating layer in Light | R-ELEVATION-1, R-ELEVATION-2 | Waits for 001 |
| [013](../tasks/adr-0101-task-013-sheets-fit-the-window.md) | Each sheet, dialog, menu and popover fits the window less the window margin | R-SHEET-1 | Waits for ADR 0097 task 005 |

Each packet changes many views. Thus one coding agent works at a time on this phase (CLAUDE.md, one agent for each set of files). Packets 002 to 012 can run in a different order when the operator asks, but these orders are fixed:

- 002 comes before 011, because R-ACT-7 applies to a header with a glow.
- 005 comes before 007, because the sheen and the spinner stop with Reduce Motion.
- 002 adds the Reduce Transparency field to the environment with the value `false`. 005 sets it from the system.

## The Ready Packets Of Other ADRs

ADR 0101 Decision 12 item 3 puts these packets after the token packet. This plan gives their place:

1. [ADR 0097 tasks 003 to 005](../tasks/adr-0097-task-003-item-pages-fit.md), the fit tests of the item pages, the Settings and the sheets, come right after packet 001. They find the faults of the new type sizes, for example a clipped page title, before the surface packets change the same views.
2. Packet 013 comes after ADR 0097 task 005. Task 005 adds the sheet check of musicindex ADR 0077 R-SHEET-2 and marks the width faults that it finds. Packet 013 corrects them.
3. [ADR 0097 task 006](../tasks/adr-0097-task-006-each-text-and-control-records-a-probe.md) comes after packet 013.
4. [ADR 0093 task 001](../tasks/adr-0093-task-001-settings-tab-row-and-rows.md), the Settings tab row, comes after packet 010, because the tab row uses the selected look.
5. [ADR 0069 task 002](../tasks/adr-0069-task-002-shared-guarded-editor.md), the editor, comes after ADR 0093 task 001. Its Save bar shows "Saved, not written" in the bar (R-NOTICE-1 item 1).
6. [ADR 0094 tasks 001 and 002](../tasks/adr-0094-task-001-fold-header-composite.md) wait until the operator or the orchestrator gives their sequence.

## Visual Defects That Move Into This Phase

The overhaul plan step 4 puts two visual defects into this phase, with no separate fix:

- ADR 0084 packet 026 V10, the clipped titles. Packet 001 V2 carries the walk. ADR 0097 task 003 adds the fit test of the page headers. The ChatGPT search found that `DetailHeader` (`src/ui/composites/detail_header.rs:139`) and `TrackHeader` (`src/ui/composites/track_header.rs:80`) draw the title as plain text beside the art. This is an inference: a long title with no space can go past its width.
- The max height of the field list (ADR 0087 packet 010). The list has `max_h(Size::ColumnRegular)`, 320 px (`src/ui/composites/search_field_list.rs:114`). Operator choice, 2026-11-15: the list is at most 320 px high, and it stops at the bottom of a lower window. Packet 012 makes this change.

## Open Points

The operator answered each open point on 2026-11-15. The operator also approved the new crate `icu_datetime` for packet 006.

1. **The label of the audio action.** Operator choice, 2026-11-15: the label is "Audition". Packet 011 fills no button on a track page. ADR 0068 task 004, the audition player, gives the label and the fill.
2. **Increase Contrast with an explicit profile.** Operator choice, 2026-11-15: with the profile System, Increase Contrast selects the high-contrast profile of the appearance. An explicit profile that the person selected stays.
3. **The report times.** Operator choice, 2026-11-15: packet 006 changes only the dates of items. The reports keep their recorded time, for example `2026-11-15 22:27:35 UTC`.
4. **A Library item and the index.** Operator choice, 2026-11-15: a Library page shows the link when it has the feed GUID. The page of musicindex.org gives its own message when it does not find the item.

## Risks

- Each packet changes the look of many views. The fit tests of ADR 0097 run after each packet.
- GPUI 0.3.1 has no blur filter, no blend mode and only two stops in a gradient (ADR 0101 Decision 4). The glow and the selected look prepare their effect outside the renderer or with two stops.
- A blur of each frame of an animated cover can be slow. Packet 002 measures it and stops if one cover takes more than 200 ms.
- The portal keys of the system settings are an inference from the xdg-desktop-portal specification. Packet 005 examines them before it uses them. No portal key gives Reduce Transparency, so that setting can stay `false` on Linux.
- Packet 006 adds a date crate. A crate that is not in `Cargo.lock` needs the operator.
