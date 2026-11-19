# ADR 0101 Task 016: The Search Error Has A "Details" Fold

Status: Implemented - operator visual checks open, 2026-11-19. The orchestrator asked for this packet on 2026-11-19, with the approval of the operator.

## Goal

The notice of a failed Music Index search shows the technical report in one fold with the label "Details". The fold is closed at the start. The citations of musicindex ADR 0023 change to musicindex ADR 0085.

## Authority

- musicindex ADR 0085 R-MSG-1 item 4 (`../musicindex/docs/adr/0085-the-words-links-and-messages-of-the-three-apps.md`, musicindex commit `4025eba`): the text of a message never shows raw text from a server, a database or a library. An app can show that text in a fold with the label "Details" below the message. The fold is closed at the start. The app also keeps that text in its log. The fold is permitted, not necessary.
- musicindex ADR 0085 replaces musicindex ADR 0023 and keeps each rule ID. [ADR 0101](../adr/0101-the-design-language-follows-the-shared-rules.md) Decision 11 adopts it.
- [ADR 0066](../adr/0066-configuration-and-startup-failure-recovery.md): the search failure report and "Copy report".

## Change

1. `src/view_models/search_results/failure.rs`: the two labels "Show details" and "Hide details" become one fold control with the label "Details". The view model gives the open condition. The accessibility label names it: "Details, closed" or "Details, open".
2. The renderer draws the fold control with a chevron that points right when the fold is closed and down when it is open. "Copy report" stays a button. It comes before the fold, so that its position does not change when the fold opens.
3. The app writes the redacted error text of a failed Music Index search to its log (`eprintln!`), as the other failures of the app do.
4. Each live citation of musicindex ADR 0023 changes to musicindex ADR 0085. Archived ADRs, reviews, plans and finished task files keep ADR 0023 as history. HitIt ADR 0023 (design system and view models) has a different subject and does not change.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Search failure view model | extend | src/view_models/search_results/failure.rs | It gives the labels and the open condition of the report. |
| Button | reuse | src/ui/primitives/button.rs | `leading_icon` draws the chevron. The fold control is one Button with its own Tab stop. |
| Search notice renderer | extend | src/ui/shells/search_results_inspector.rs | `empty_state_content` draws the failure actions and the report. |
| Fold header | reuse | docs/adr/0094-one-fold-header.md | `FoldHeader` of ADR 0094 is not built yet. Its task waits for the operator. This packet uses the chevron of R-FOLD-3 item 1 on a Button. When `FoldHeader` exists, its task replaces this control. |
| Endpoint redaction | reuse | src/diagnostics.rs | `redact_endpoint_details` removes secrets before the log line. |

## Mechanical Criteria

- A view model test: the fold control has the label "Details" when it is closed and when it is open. The accessibility label is "Details, closed" at the start and "Details, open" after one activation. The report is not visible at the start.
- The tests of ADR 0066 for the report and "Copy report" pass with no change to their report assertions.
- No source file has the labels "Show details" or "Hide details".
- No live file outside the history set cites musicindex ADR 0023.
- The commit checks of AGENTS.md pass.

## Visual Criteria

- V1: in Light and Dark, make a Music Index search fail. The notice shows "Copy report" and below it the closed fold "Details" with a chevron that points right.
- V2: click "Details" or press Enter on it. The report shows below the fold, and the chevron points down. A second activation closes it.
- Incorrect: the report shows at the start, or the label changes to "Show details" or "Hide details".

## Report

- The view model field `fold_open` of `SearchFailureActionDisplay` gives the open condition of the fold control. It is `None` for "Copy report".
- The fold control uses `ControlStyle::RowAction`, the quiet style of a row control, because R-FOLD-1 item 1 gives a closed fold header the look of a list row.
- Operator choice, open: "Copy report" now comes before the fold. Before this packet the two buttons were in one row, with "Show details" first.
