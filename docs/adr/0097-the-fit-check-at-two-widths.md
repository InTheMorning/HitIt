# ADR 0097: The Fit Check At Two Widths

## Status

Accepted - 2026-11-13. The operator selected the narrowest width and the text system of the tests. The orchestrator session relayed the two decisions and selected the scope of phase 1.

- Revised: 2026-11-14: each citation of a live rule of the six archived Music shell ADRs names the item of [ADR 0098](0098-the-music-shell-pages-lists-and-search.md) that restates it. No rule changes.

## Context

[musicindex ADR 0065](../../../musicindex/docs/adr/0065-each-app-checks-its-fit-at-two-widths.md), Accepted on 2026-11-13, makes each app check its fit at two widths. R-FIT-2 gives the two widths. R-FIT-3 gives the faults: a break inside a word, an edge cut, an overflow and an overlap. The overlap, and the overflow of a text out of its own box, came with the R-FIT-3 change of 2026-11-13. R-FIT-4 item 2 makes HitIt do the check as GPUI layout tests. musicindex ADR 0065 leaves the narrowest width of HitIt to this app (musicindex ADR 0069 R-FIT-1 item 1).

These facts about HitIt and gpui-pre 0.3.1 control the design:

- No HitIt ADR stated a narrowest width. The window set no minimum size in `src/app/bootstrap.rs`.
- Below `SHELL_TAB_BAR_BREAKPOINT` (940 px, `src/ui/layouts.rs`) the tab bar replaces the sidebar ([ADR 0098](0098-the-music-shell-pages-lists-and-search.md) Decision 1). Thus 940 px is the first width with the sidebar.
- A GPUI test uses `NoopTextSystem`. Each character is 0.6 em wide, and the test does not use Figtree. `TestAppContext::build` always makes this text system, and `App::new_app` is not public. Thus a test cannot find the correct width of a label.
- A GPUI test can read the bounds of an element only through its `debug_selector`. GPUI gives a test no walk of all elements.
- GPUI gives the wrap boundaries of a text through `TextLayout::line_layouts`. During prepaint and paint, `Window::content_mask` gives the area that the window, the panels and the scroll areas let the element show.
- No test mounted the full shell, `TopApp`. A test can make it with an in-memory database and no runtime host (`src/app/search_field_list.rs` makes `LibraryApp` with this procedure).

## Decision

### 1. The narrowest supported width is 390 px

1. The narrowest supported window width of HitIt is 390 px. This is the narrowest width of musicindex ADR 0069 R-FIT-1 item 1 for HitIt.
2. The window sets `window_min_size` to a width of 390 px, so the app cannot become narrower. The constant is in `src/ui/layouts.rs`. This record gives no minimum height.

### 2. The two widths of the check

1. The layout tests run at 390 px and at `SHELL_TAB_BAR_BREAKPOINT` (musicindex ADR 0065 R-FIT-2).
2. The tests read the two constants. When a constant changes, the tests change with it (R-FIT-2 item 3).

### 3. The tests use Figtree

1. A patch of `gpui-pre` adds a public `TestAppContext` constructor that takes a `PlatformTextSystem`. The patch is in the `InTheMorning/gpui-kit` fork. `Cargo.toml` pins it by rev in `[patch.crates-io]`, with the same method as the `gpui-base` patch.
2. The fit tests give that constructor the `CosmicTextSystem` of `gpui-pre-wgpu`. Then they install the embedded Figtree files through `crate::ui::fonts::install`.
3. When gpui-pre has a public constructor that gives a test a text system, a superseding change removes the patch.
4. The fork rev holds the gpui-base correction of [ADR 0072](archive/0072-pinned-gpui-base-selection-corrections.md). Thus `gpui-base` and `gpui-pre` use one rev. The ADR 0072 guard in `tests/architecture_tests.rs` allows these two patches and that rev only.

### 4. The shared primitives record fit probes

1. In a test binary, the shared text and control primitives of `src/ui/primitives` record fit probes. A probe holds the bounds of the element, the content mask at prepaint, and the text layout of a text. A control also records a probe of its text, linked to the control by its element id. A production binary records no probe.
2. The test finds the faults of musicindex ADR 0065 R-FIT-3 from the probes:
   - A break inside a word: a wrap boundary of a text is between two characters that are letters or digits.
   - An edge cut: an edge of the content mask crosses the element on the horizontal axis. Or an edge of the window crosses the element on the vertical axis.
   - An overflow: the element is wider than its content mask, or the element is fully out of its content mask on the horizontal axis. A text that is wider than its own box is also an overflow. So is a text that goes out of the box of its control on the horizontal axis.
   - An overlap: the boxes of two controls intersect by more than 1 px on each axis in the part that shows, and no box of the two holds the other.
3. The attribute of R-FIT-3 item 3 is `Label::break_anywhere`. A text with this attribute, for example a URL or a GUID, can break inside a word, and the test does not count that break.
4. A vertical scroll area cuts its rows on the vertical axis during a scroll. Thus the test does not count a vertical cut by a content mask that is not the window.

### 5. The failure message

The message of each fault names musicindex ADR 0065 R-FIT-3, the fault, the width, the text or the element, and the fix. An example:

```text
musicindex ADR 0065 R-FIT-3: at 940 px the label "Public playlists" breaks inside a word.
Give the label more width, make the text shorter, or let the row wrap between the controls (musicindex ADR 0069 R-FIT-1 item 2).
```

### 6. The phases

The [ADR 0097 phase plan](../plans/adr-0097-fit-check-phase-plan.md) orders the work. Phase 1 covers the shell: the sidebar, the tab bar, the Settings navigation and the list bar. The subsequent phases add each page, sheet and dialog of R-FIT-2 item 4. The plan keeps the subsequent phases open until each is complete.

## Consequences

- HitIt depends on a patched `gpui-pre`. Each upgrade of gpui-pre needs the patch again, or a gpui-pre release with the same public constructor.
- The fit tests install system fonts through `CosmicTextSystem`. A label in Figtree gets the same width on each machine. A glyph that Figtree does not have uses a system font, and its width can change between machines.
- A test binary renders one more element in each probed primitive. The probe has no dimension and no paint. A production binary does not have it.
- Until the last phase, only part of R-FIT-2 item 4 is met. The phase plan names the pages, sheets and dialogs that have no check.

## How This Decision Is Checked

The rule broke in Studio rc.22 on 2026-11-13 (musicindex ADR 0065). Thus it gets a guard.

Mechanical:

- A unit test makes a label with a long word in a narrow box. The test must report a break inside a word. The same test with `Label::break_anywhere` must report no break.
- A unit test puts a button across the right edge of a box with `overflow_hidden`. The test must report an edge cut. A button fully out of the box must give an overflow.
- A unit test puts a button with a long label in a narrow box, and a label with no wrap in a narrow box. The test must report an overflow of the text for each.
- A unit test puts two buttons on top of each other. The test must report an overlap. Two buttons that do not touch must give no fault.
- The phase 1 layout test mounts `TopApp` at 390 px and at 940 px. It finds no fault in the start page, the tab bar or the sidebar, and the Settings navigation. The test mounts the list bar at the content width of each window width.
- A unit test reads the window options of `src/app/bootstrap.rs` and requires a minimum width of 390 px.

Visual, for the operator:

- Open HitIt at a window width of 940 px. Make sure that the sidebar shows and that no label breaks inside a word (musicindex ADR 0065).
- Make the window as narrow as possible. Make sure that it stops at 390 px and that the tab bar shows each tab.

## Alternatives Considered

### Keep the no-op text system

The test then uses 0.6 em for each character. Figtree is narrower for most letters and wider for some, for example "W" and "M". The test then reports breaks that do not occur and misses breaks that occur. The operator selected Figtree.

### A separate width test for each fixed label

A test can shape each fixed label with Figtree and compare it with its token width. That test does not use the layout of GPUI. It cannot find an edge cut or an overflow. It also misses a label that gets its width from the layout.

### A full shell check in one phase

R-FIT-2 item 4 asks for each page, sheet and dialog. Many pages need fixture data in the Library database. One phase for all of them is too large to examine. The orchestrator selected phases.
