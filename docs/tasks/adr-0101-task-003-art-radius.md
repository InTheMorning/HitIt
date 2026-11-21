# ADR 0101 Task 003: The Art Radius And The Art Shadow

Status: Implemented - operator visual checks open, 2026-11-16.

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

Each fact is from HitIt `ba43138`, read on 2026-11-15. A ChatGPT search found them, and the main session examined the items marked "examined".

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

- V1: the corners of a row image, a tile and a page header image look the same in proportion. A tile corner does not have the curve of a button.
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

## Implementation Report - 2026-11-16

Status: the mechanical checks M1 to M8 pass. The visual checks V1 and V2 stay open for the operator.

Changes:

- `src/ui/tokens.rs`: new `ArtworkRadius` with `FRACTION_OF_EDGE` 0.06, `MINIMUM_PX` 4, `px_for_edge` (rounds half up) and `for_edge`. New `ArtworkShadow::Lg` (blur 8, offset 2, alpha 0.22).
- `src/ui/primitives/image.rs`: the primitive takes its radius from the edge. The `radius` field and method are removed. `ImageSize::Lg` has the Lg shadow.
- `src/ui/composites/thumbnail.rs`: `ThumbnailSize::radius(cx)` and `radius_at(scale)` use the function.
- `src/ui/composites/content_tile.rs`, `src/ui/composites/collection_art.rs`, `src/ui/shells/library/thumbnail.rs`: callers use the function.
- `tests/architecture_tests.rs`: the guard `playlist_rows_scale_through_design_tokens` requires `ArtworkRadius::for_edge(size)` in place of `Radius::SM.scaled(cx)`. Its purpose (the radius scales) is unchanged.

Callers at UI scale Medium:

| Caller | Edge | Radius before | Radius after |
|---|---|---|---|
| `Thumbnail` Sm | 32 | 6 (SM, task 001 value) | 4 |
| `Thumbnail` Md | 48 | 6 | 4 |
| `Thumbnail` Lg | 80 | 10 (MD) | 5 |
| `Thumbnail` Xl | 152 | 10 | 9 |
| `Thumbnail` XXl | 200 | 12 (LG) | 12 |
| `ContentTile` image | 152 | 10 | 9 |
| `CollectionArt` pending square | size of `ThumbnailSize` | role value | same as `Thumbnail` |
| `render_album_thumb` (playlist row) | 24 | 6 | 4 |

The logo frame raster (`src/media/logo_frame.rs`) draws a polygon shape with no corner radius. It needed no change. `track_header.rs` and `file_header.rs` set no image radius.

Tests: M1 `ui::tokens::tests::r_art_4_art_radius_is_six_percent_of_the_edge_with_a_minimum`, M2 `ui::composites::thumbnail::tests::r_art_4_each_thumbnail_size_has_the_art_radius_of_its_scaled_edge`, M3 `ui::composites::content_tile::tests::r_art_4_content_tile_uses_the_art_radius_function` and `ui::composites::collection_art::tests::r_art_4_collection_art_uses_the_art_radius_function`, M4 `ui::primitives::image::tests::the_art_shadow_grows_with_the_image_size` and `adr_0083_artwork_shadow_grows_with_image_size`, M5 `ui::tokens::tests::r_art_4_art_radius_matches_the_shared_token`.

Primary agent choice, 2026-11-16: the Lg art shadow has blur 8, offset 2 and alpha 0.22. Each value is approximately half of the Xl value or less, so the shadow grows from Lg to Xl to XXl. The packet gave only "smaller than Xl".

The primary agent examined the diff and ran the gate. Each user of the image primitive shows art, so no icon or logo gets the art radius.

## Walk Result - 2026-11-16

The primary agent walked V1 on an Xvfb display with the debug binary, in Dark at M.

- The first walk found square corners on each image. The container of the image primitive clips to its rectangle, not to its corners. The code before this task had the same fault. Correction: the `img` element of `src/ui/primitives/image.rs` gets the same radius, because GPUI paints an image with the corner radii of its own style.
- After the correction, V1: Pass. A row image has a corner of approximately 4 px, a tile approximately 9 px and the page header image approximately 12 px. A tile corner does not have the curve of a button.
- V2 is open. On the dark page background, the agent cannot compare the shadow sizes in a screenshot. The operator does V2 in Light.

The operator confirmation stays open.
