# ADR 0093: Settings Has A Tab Row And One Save Bar

## Status

Accepted - 2026-11-10. The operator accepted [musicindex ADR 0059](../../../musicindex/docs/adr/0059-settings-has-a-tab-row-and-one-save-bar.md) (musicindex `5c4375c`) on 2026-11-10, and decided that v4vmm uses the same Settings layout as Studio. The orchestrator session relayed the decision. This record applies R-SET-1 to R-SET-5 to v4vmm. Decisions 3, 6, 9, 11 and 12 are choices of the planning session. This record changes no code.

Applies: musicindex ADR 0059 R-SET-1 to R-SET-5. Uses: musicindex ADR 0051 R-SEG-1 for the tab row, and musicindex ADR 0032 R-DOT-1 for the type dot.

Supersedes: the Settings group menu and the shared Settings header of [ADR 0074](0074-repair-and-diagnostics-pages.md). Amends: [ADR 0069](0069-grouped-settings-and-selective-presets.md), the group navigation, "leaving Settings retains the working draft" and Cancel. Amends: [ADR 0069 task 002](../tasks/adr-0069-task-002-shared-guarded-editor.md), the names of the commands, the leave and the quit.

## Context

On 2026-11-10 v4vmm Settings has the groups General, Library and Diagnostics. A compact menu selects the group (ADR 0074). Diagnostics has a second menu for its five tool pages. General and Library show a label above each control and a hint below it.

Save and Use Defaults are below the fields of General and Library. Diagnostics does not show them. Use Defaults resets the fields and saves them immediately. There is no Discard.

ADR 0069 keeps the draft when the user leaves Settings. ADR 0069 task 002 adds the shared guarded draft with Save, Cancel and Use Defaults. It discards the draft on quit. Task 002 is Ready, and its implementation has not started.

The examination of v4vmm for musicindex ADR 0059 found these differences on 2026-11-10:

| Rule | v4vmm on 2026-11-10 |
| --- | --- |
| R-SET-1 | A compact menu of three groups. ADR 0069 plans five groups: General, Live Metadata, Audio, Library and Diagnostics |
| R-SET-2 | Differs. A label above the control and a hint below it. UI scale and Theme are rows of buttons that wrap |
| R-SET-3 | One Save for two groups, no Discard, no mark. Use Defaults saves immediately. A leave keeps the draft, and a quit discards it with no question |
| R-SET-4 | Agrees. The scale and the theme apply immediately. Cancel of task 002 puts them back, and it is not implemented |
| R-SET-5 | Agrees, with two differences. Use Defaults saves immediately. End session discards the draft with no question |

On 2026-11-10 Settings has these actions: Converter setup, Test converters and Configuration repair. Diagnostics adds the Database tasks, the Background tools checks, End session, and Delete in Cached files. Task 002 adds the explicit MusicIndex check.

## Decision

### The tab row (R-SET-1)

1. A segmented control of musicindex ADR 0051 R-SEG-1, below the page bar, selects the group. It replaces the group menu of ADR 0074. The labels are General, Library and Diagnostics, in that sequence.
2. ADR 0069 adds Live Metadata and Audio between General and Library when their owners exist. The row then has 5 groups, which is the limit of R-SET-1 item 2. A sixth group needs a change to this record.
3. v4vmm has no narrow layout for a tab row. On a narrow page, the tab row fills the width (musicindex ADR 0051 R-SEG-1 item 5). It does not become a menu or a side column.
4. Diagnostics holds tools, not settings. It keeps its page menu for Database, Configuration, Background tools, App session and Cached files (ADR 0074). Each tool is an action of R-SET-5.
5. A change of the group runs no command (ADR 0069 Invariant 1). It keeps the draft, and the focus and the scroll position of each group.

### The rows (R-SET-2)

6. Each setting is one row. The name and a hint of one line are at the start, and the control is at the end. The rows of a group are in sections, with the section heading of v4vmm. When the row is too narrow for its control, the control goes below its name.
7. These are the rows:

    | Group | Section | Row | Control |
    | --- | --- | --- | --- |
    | General | Appearance | UI scale | The choices XS, S, M, L and XL |
    | General | Appearance | Theme | The five theme choices |
    | General | Defaults | Default values | The action Use Defaults |
    | Library | Music Index | Music Index endpoint | A text field below its name (R-SET-2 item 3) |
    | Library | Files | Music directory | The current path, and the action Configuration repair |
    | Library | Files | Audio converter | The saved FLAC path, and the action Converter setup |

8. The FLAC path is a setting of the draft (ADR 0069 task 002 step 2). The user edits it in Converter setup, and the row shows its value and its mark. The Library row has no text field for it.

### The draft and the Save bar (R-SET-3)

9. The page bar shows the state of the draft, Discard and Save on each group, Diagnostics included. The state texts are those of R-SET-3 item 2: "No changes", "1 change not saved", "N changes not saved" and "Saved". Discard replaces Cancel of ADR 0069 and its task 002.
10. The settings of the draft are UI scale, Theme, Music Index endpoint and FLAC path (ADR 0069 task 002 step 2). Use Defaults puts the default value of these four settings in the draft. It does not save.
11. When v4vmm cannot save the draft, Save is off and the page bar gives the cause. These are the causes:
    - ADR 0066 pauses ordinary saves while a saved optional value is not correct.
    - Configuration repair has edits that are not saved (ADR 0069 task 002 step 2).
12. A changed row shows a pencil in the accent color after its name, with the accessible name "Not saved". A tab with a pending change shows the same pencil after its label. Studio ADR 0166 item 9 uses the same mark. The mark is not the type dot.
13. The leave question opens before each of these with a pending change: a move to a different destination of the sidebar, End session, and the close of the window. The question is "Leave without saving?". It names the number of changes. The buttons are "Keep Editing" and "Discard Changes" (destructive). A change of the group or of the Diagnostics page does not ask.

### The preview (R-SET-4)

14. UI scale and Theme show immediately as a preview. Save keeps them. Discard, or "Discard Changes" of the leave question, puts back the saved values.

### The actions (R-SET-5)

15. The actions run immediately with their own confirmation, and Save and Discard do not include them. Configuration repair keeps its own review and Save of ADR 0066, because a core path needs the recovery route.
16. End session asks the leave question first (R-SET-5 item 3). Its explanation does not say that it discards the edits.

## Invariants

1. Settings has one tab row of at most 5 groups, and one Save bar.
2. A group change runs no command and loses no edit.
3. Save is the only route that writes a setting of the draft. No action writes the draft, and no save runs an action.
4. Nothing leaves Settings with a pending change and no question.

## How This Decision Is Checked

Mechanical, in the view-model tests of the tasks below:

- The view model gives one segmented group control with at most 5 groups, and the rows of one group.
- A change of a setting gives the state "1 change not saved", the mark on its row and on its tab, and no write.
- Save writes each pending value and gives "Saved". Discard puts back each saved value and the preview.
- Use Defaults changes the draft and does not write.
- A leave, End session and the close of the window with a pending change give the leave question.
- Each action runs with no Save, and Save runs no action.

Visual, for the operator: open Settings in the light and the dark theme, at the usual and the narrow width and at XL. Change a setting in two groups, then select Discard. Change them again, then select Save. Change a setting, then select a destination in the sidebar.

## Delivery

[ADR 0093 task 001](../tasks/adr-0093-task-001-settings-tab-row-and-rows.md) delivers Decisions 1 to 8. It needs no draft, and it is Ready. [ADR 0069 task 002](../tasks/adr-0069-task-002-shared-guarded-editor.md) delivers the draft with Decisions 9 to 16. The operator decides the priority.

## Alternatives Considered

musicindex ADR 0059 gives the alternatives: a list of groups, tabs in a column at the side, and one page with jump links.

### Keep the draft when the user leaves Settings

ADR 0069 kept the draft across a leave. Then the preview of the scale shows on Music pages with no save, and the workspace writes of task 002 wait for an indefinite time. R-SET-3 item 5 asks the leave question.

### A text field for the FLAC path in Library

The row then has a second editor of the same value. Converter setup tests the path, and it is the owner of the edit (ADR 0069 task 002).
