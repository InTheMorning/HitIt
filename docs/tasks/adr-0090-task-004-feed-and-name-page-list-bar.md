# ADR 0090 Task 004: Feed Page And Name Level Page List Bar

## Status

Implemented - 2026-11-08. Visual checks are open. It is built together with packet 6 of the [musicindex ADR 0053 phase plan](../plans/musicindex-adr-0053-list-controls-phase-plan.md).

This task is packet 004 of the [ADR 0090 phase plan](../plans/adr-0090-list-bar-phase-plan.md). It implements Decision 1 of [ADR 0090](../adr/0090-one-list-bar-above-the-rows.md) on the feed page and on the name level page.

## Goal

On the feed page and on the name level page, show one list bar above the release sections. Remove the band above the title and the bottom action bar. The bar stays in view during a scroll.

## Scope

- Only the feed page (`PublisherPageVm`, `src/ui/shells/publisher.rs`) and the name level page (`NameLevelPageVm`, `src/ui/shells/name_level_page.rs`) change, in each context that shows them.
- The bar uses the parts of packets 001 to 003: `has_list_bar`, `ListBarVm`, `ListBarCommand`, `render_list_bar_copies` and the pinned copy.
- The bar has no sort control and no view icon. Each release section has its own control at the right end of its heading (musicindex ADR 0053 R-LIST-11 item 3). Packet 6 of the ADR 0053 phase plan adds it.

## Position Of The Bar

The bar sits after the header, the facts and the notices, and before the first release section. Each section heading stays with its rows below the bar. The section "Shares albums with" is not a release list, so it keeps its position.

## Mechanical Criteria

Each criterion is checked by a unit test or a guard.

- `has_list_bar` gives `true` for the feed page and the name level page.
- A page with no selectable release has no list bar.
- While selecting, the bar gives the actions that the bottom bar of the page gives today.
- The guard `adr_0090_list_bar_pages_have_no_frame_header_action_or_bottom_bar` covers the two pages.
- The guard `adr_0090_pinned_copy_comes_from_the_one_list_bar_renderer` covers the two pages.

## Visual Criteria

These criteria need a person in a desktop session. They are open.

- Open a feed page with releases from Artists. The bar is directly above the first section heading. The page has no band above its title and no bottom bar.
- Open a name level page. The same points apply.
- Select releases in two sections. The count and the actions in the bar include the releases of the two sections.
- Scroll a long feed page. The bar stays at the top of the rows.
- In a narrow window, the bar wraps. Its two ends stay on two lines.
