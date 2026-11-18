# ADR 0093 Task 003: The Audio Cache In Library

## Status

Implemented - 2026-11-18. The mechanical criteria pass. The visual criteria wait for a person, in section 36 of
[pending human checks](../pending-human-checks.md).

The agent check of 2026-11-18 on Xvfb passed in Dark at 1440 px and at 390 px. Library showed the section "Audio
cache" after "Files". The Diagnostics page menu showed Database, Configuration, Background tools and App session.
Light was not examined.

## Operator Choice

Operator choice, 2026-11-18: the cached files are settings of the Library, not a Diagnostics tool. Library gets the
section "Audio cache" after "Files". Its row is "Cached files (N)" with a hint and "Delete All Cached". The file list
is below that row, with "Delete" on each file. Diagnostics loses its "Cached files" page.
[ADR 0093](../adr/0093-settings-has-a-tab-row-and-one-save-bar.md) Decisions 4 and 7 record it.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Settings view model | extend | src/view_models/settings.rs | The Library sections get "Audio cache" with the row `CachedFiles`. `DiagnosticPage` loses `CachedFiles`. |
| Settings render | extend | src/app/settings.rs | The sections loop renders the `CachedFiles` row with `render_cached_files`. |
| Settings row composites | reuse | src/ui/composites/settings.rs | `settings_row` and `settings_cached_row` give the row and the file rows. `settings_plain_page` had no other user, so it is deleted. |
| Cached files view model | extend | src/view_models/cached_files.rs | The status text names "Diagnostics, Background tools" as the location of "Check again". |

## Acceptance Criteria

Mechanical:

1. The Settings view model gives the Library section "Audio cache" with the row `CachedFiles`, after "Files"
   (test `adr_0093_sections_give_the_rows_of_decision_7`).
2. `DiagnosticPage::ALL` holds Database, Configuration, Background tools and App session.
3. `cargo test --locked`, `cargo clippy --all-targets --locked -- -D warnings` and
   `cargo test --test architecture_tests` pass.

Visual, for a person:

1. Library shows the section "Audio cache" after "Files", in Light and Dark, at 1440 px and at 390 px. The row shows
   "Cached files (N)", the hint and "Delete All Cached". The file list is below the row.
2. "Delete" on one file removes that file from the list.
3. The Diagnostics page menu does not show "Cached files".
