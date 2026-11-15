# ADR 0101 Task 003: The Art Radius And The Art Shadow

Status: Waits - 2026-11-15. This packet starts after [task 001](adr-0101-task-001-shared-tokens.md) passes its checks. It does not wait for task 002.

## Goal

The corner radius of each image of art is 6 % of its edge, rounded to a full pixel, with a minimum of 4 px. A larger image has a larger shadow. Only art has an art shadow.

## Authority

- [ADR 0101](../adr/0101-the-design-language-follows-the-shared-rules.md) Decision 4 owns this packet. Read Decisions 4 and 12 before the first edit.
- musicindex ADR 0021 R-ART-4 (`../musicindex/docs/adr/0021-cross-app-artwork-rules.md` lines 67 to 87, at musicindex `785ae28`):
  - Item 2: the radius is 6 % of the edge, rounded to a full pixel, with a minimum of 4 px. 40 px gets 4 px, 84 px gets 5 px, 152 px gets 9 px and 200 px gets 12 px.
  - Item 3: only artwork has a shadow. A larger image has a larger shadow.
  - Item 4: an app keeps its own sizes.
- The shared token `art-radius` of musicindex-assets `b8399cc` gives `fraction-of-edge` 0.06, `minimum-px` 4 and `rounding` "nearest pixel". `src/assets/tokens.json` is the HitIt copy. Do not edit it.
- ADR 0039 owns the UI scale. The radius is 6 % of the scaled edge.
- The `rust-dev` skill applies to each Rust change.

## Recorded Facts

Each fact is from HitIt `ad1fcd9`, read on 2026-11-15. A ChatGPT search found them, and the main session examined the items marked "examined".

- `ThumbnailSize` (`src/ui/composites/thumbnail.rs:43`) gives Sm 32, Md 48, Lg 80, Xl 152 and XXl 200 px.
- `ThumbnailSize::radius` (`src/ui/composites/thumbnail.rs:88`) maps Sm and Md to `Radius::SM`, Lg and Xl to `Radius::MD`, and XXl to `Radius::LG`. Examined. Task 001 changes these role values to 6, 10 and 12.
- `Thumbnail::render` (`src/ui/composites/thumbnail.rs:126`) gives the radius to the image primitive.
- `src/ui/composites/content_tile.rs:32` draws a tile image of size Xl with `Radius::MD`. It does not use `Thumbnail`.
- `CollectionArt` (`src/ui/composites/collection_art.rs:127`) forwards its size. At XXl it asks for a baked shadow (`:57`). `src/media/logo_frame.rs:131` bakes it.
- `src/ui/primitives/image.rs:74` gives an art shadow only to Xl and XXl. Line 148 draws it. `ArtworkShadow` (`src/ui/tokens.rs:771`) owns the shadow values, and line 813 builds the GPUI shadows.
- Tests: `adr_0083_thumbnail_sizes_match_their_image_sizes` (`src/ui/composites/thumbnail.rs:169`) and `only_xl_and_xxl_resolve_an_artwork_shadow` (`src/ui/primitives/image.rs:163`). Guard: `adr_0083_only_tokens_builds_a_shadow` (`tests/architecture_tests.rs:20575`).

Main session choice, 2026-11-15: the radius is a function of the scaled edge, not a `Radius` role. Thus a change of a `Radius` role does not change the art. The function rounds half up.

Main session choice, 2026-11-15: Sm and Md keep no shadow. An image of 32 px or 48 px in a row has no room for one. Lg gets a shadow smaller than Xl. Xl and XXl keep their shadow values.

The website shadow is `0 16px 48px` at `rgba(0, 0, 0, .5)` dark and `.18` light. Its source is `search.html` at musicindex `785ae28`. It is a reference for XXl, not a rule.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Art size and radius | extend | `src/ui/composites/thumbnail.rs` | `ThumbnailSize` owns the sizes. Replace `radius` with the 6 % function there. |
| Image primitive | extend | `src/ui/primitives/image.rs` | It draws the radius and the art shadow. Add the Lg shadow there. |
| Shadow tokens | extend | `src/ui/tokens.rs` | `ArtworkShadow` owns the shadow values. Add the Lg value. |
| Tile image | extend | `src/ui/composites/content_tile.rs` | It draws its own image. Give it the radius of the function. |
| Collection art | extend | `src/ui/composites/collection_art.rs` | It forwards its size. Its baked image uses the same radius. |
| Logo frame raster | extend | `src/media/logo_frame.rs` | It bakes the collection art. Give it the radius of the function. |

## Required Changes

1. **The radius function.** Write one function that takes the scaled edge in pixels and gives the radius: 6 % of the edge, rounded half up to a full pixel, with a minimum of 4 px. Put the fraction and the minimum in `src/ui/tokens.rs`, with a comment that cites R-ART-4 item 2.
2. **The callers.** `Thumbnail`, `ContentTile`, `CollectionArt` and the logo frame raster use the function. Find each other image of art that sets a `Radius` role, for example in `src/ui/composites/track_header.rs` and `src/ui/composites/file_header.rs`. Report each one.
3. **The shadow.** Give Lg an art shadow that is smaller than the Xl shadow. A test proves that the shadow grows with the size. Do not give a shadow to a non-art image, for example a logo or an icon.
4. **The token test.** A test reads `art-radius` in `src/assets/tokens.json` and compares the fraction and the minimum. Its message names R-ART-4 item 2 and the fix.

## Do Not Change

- No change of an art size. R-ART-4 item 4 lets HitIt keep its sizes.
- No change of a `Radius` role value.
- No change of the header glow. Task 002 does that.
- No `git commit`, `git add` or `git stash`.

## Acceptance Criteria

Mechanical:

- M1: a test of the function gives 4 for 40, 5 for 84, 9 for 152 and 12 for 200, and 4 for 32 and 48.
- M2: a test proves that each `ThumbnailSize` gives the radius of the function for its scaled edge at each UI scale step.
- M3: a test proves that `ContentTile` and `CollectionArt` use the same function.
- M4: a test proves that the art shadow of Lg is smaller than Xl, and Xl is not larger than XXl. Sm and Md have none.
- M5: the token test of step 4 passes.
- M6: the shadow guard passes.
- M7: the fit tests of ADR 0097 pass.
- M8: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and `cargo build --bin hitit` pass.

Visual, for the operator, in Dark and Light at M:

- V1: the corners of a row image, a tile and a page header image look the same in proportion. A tile corner is not round like a button.
- V2: a header image has a larger shadow than a tile. A row image has no shadow.

## Test Commands

- `cargo test --lib ui::composites::thumbnail`
- `cargo test --lib ui::primitives::image`
- `cargo test --lib ui::tokens`
- `cargo test --test architecture_tests`
- `cargo test app::fit_check::`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Report

Give the files changed and the tests of M1 to M5 by name. Give each caller of step 2 with its size and its radius before and after.

A delegate does not commit. It leaves its change in the working tree and reports it. Write each comment and the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if `art-radius` in `src/assets/tokens.json` does not give 0.06 and 4.
- Stop if the GPUI image corner radius cannot take a value that is not a `Radius` role.
- Stop if an image is art in one view and an icon in a different view.
