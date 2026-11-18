# ADR 0098 Task 002: A Role Row Dot Has The Color Of Its First Role

Status: Implemented - operator visual checks V1 and V2 open, 2026-11-18. The orchestrator asked for this packet on 2026-11-18, with the approval of the operator.

This packet brings HitIt into agreement with musicindex ADR 0037 R-WORD-7 item 2: "The dot has the color of the first role in that sequence." The website (`roleLabel` in `search.html`) and Studio (`mediumKind` in `crates/api/ui/shell.js`) already give a role row this dot, as the orchestrator reported on 2026-11-18. The packet replaces the choice of [ADR 0084 task 029](adr-0084-task-029-search-link-facts.md) that `entity_kind_for_badge` maps `Roles` to the feed color.

## Goal

The search row of a feed with known roles and its feed page show the dot in the same color. For "Ph@fe", the row dot and the page dot have the label color.

## Authority

- musicindex ADR 0037 R-WORD-7 item 2: the dot has the color of the first role in the R-ROLE-1 sequence. musicindex ADR 0066 R-WORD-8 gives the fill.
- [ADR 0098](../adr/0098-the-music-shell-pages-lists-and-search.md) Decision 11.7: the badge has one dot. The dot has the color of the first role in the label, in the R-ROLE-1 sequence (musicindex ADR 0066 R-WORD-8).
- [ADR 0098](../adr/0098-the-music-shell-pages-lists-and-search.md): each list row shows its type dot and type word (musicindex ADR 0038 R-DOT-4).

## Defect

The [third agent walk of 2026-11-18](../reviews/agent-walk-2026-11-18-third.md) question 2: the dot of the "Ph@fe" row in the search results has the feed color (`#C4965B`). The dot of its page has the label color (`#875CCF`). Screenshot: `docs/reviews/screenshots/agent-walk-2026-11-18-third/role-search-row-dot.png`.

## Cause

The main session read this in the code on 2026-11-18:

- `SearchRowBadge::Roles(String)` (`src/view_models/search_results/rows.rs`) holds only the type label text, for example "Label · Host".
- `entity_kind_for_badge` (`src/ui/shells/search_result_rows.rs`) maps `SearchRowBadge::Roles(_)` to `EntityKind::Feed`.
- The feed page reads `PublisherPageVm::header_badge` (`src/view_models/publisher_page.rs`). It gives a `HeaderRole` of the first known role. `src/ui/shells/publisher.rs` maps `HeaderRole::Label` to `EntityKind::Label`.

## Change

- `SearchRowBadge::Roles` also holds the first known role, in the R-ROLE-1 sequence. The view model selects it with the same function as the type label (`known_roles` or `type_label` of `src/view_models/role_words.rs`).
- `entity_kind_for_badge` maps the role to its `EntityKind`, with the same map as the feed page. Move the map of `src/ui/shells/publisher.rs` to one shared place, and use it in the two renderers. Do not copy it.
- The filled or hollow dot follows R-WORD-8, as on the feed page, if the row has the agreement facts. When nothing confirms the roles, the dot is hollow (musicindex ADR 0033 R-WORD-3 item 3), if HitIt has that state for the row. If the row does not have the agreement facts, keep the current dot fill and write this in the report.
- Do not change the color of the "Feed" badge, the "Artist" badge or a release badge.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Search row badge | extend | src/view_models/search_results/rows.rs | It holds the badge facts of a feed row. |
| Role words | reuse | src/view_models/role_words.rs | It gives the known roles in the R-ROLE-1 sequence. |
| Search row renderer | extend | src/ui/shells/search_result_rows.rs | `entity_kind_for_badge` selects the dot color. |
| Feed page badge color map | extend | src/ui/shells/publisher.rs | The map of `HeaderRole` to `EntityKind`. It becomes the shared map. |

## Mechanical Criteria

- A view model test gives a feed row with the agreed roles `host`, `label` and `studio`. The badge text is "Label · Host", and the badge role is label.
- A renderer test, or a unit test of the shared map, proves that a label role badge maps to `EntityKind::Label`. Before the change, it maps to `EntityKind::Feed`. The implementer records that failure.
- The commit checks of AGENTS.md pass.

## Visual Criteria

- V1: search "Ph@fe". The row dot and the dot on the page of "Ph@fe" have the same label color, in Light and Dark.
- V2: a "Feed" row and an "Artist" row keep their colors.
- Incorrect: one feed shows two dot colors, or a "Feed" row changes its color.

## Stop And Ask

- Do not change a shared rule. Stop and report if a shared rule seems to disagree with this packet.

A delegate does not commit. It leaves the change in the working tree and reports it. Apply the STE skill at `/home/citizen/.agents/skills/asd-ste100/SKILL.md` to each comment and report.

## Report

- `SearchRowBadge::Roles` now has the fields `words` and `first`. `first` is the `HeaderRole` of the first known role, from `header_role_for`.
- The shared map is `EntityKind::from_header_role` in `src/ui/composites/tag_badge.rs`. The feed page, the curator badge and `entity_kind_for_badge` use it. The old function in `src/ui/shells/publisher.rs` is deleted.
- The row has only agreed roles in `FeedLinkClass::OtherRoles`. The dot keeps its filled state, as before. The row has no state for a hollow dot.
- Failure before the change: `entity_kind_for_badge` gave `Feed` where the test expects `Label`.
- Visual criteria V1 and V2 are open. A person must judge them.
