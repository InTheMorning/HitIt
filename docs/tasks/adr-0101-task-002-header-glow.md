# ADR 0101 Task 002: The Page Header Glow

Status: Waits - 2026-11-15. This packet starts after [task 001](adr-0101-task-001-shared-tokens.md) passes its checks.

## Goal

An album page, a track page, an artist page and a label page show a blurred glow from the cover. The glow has film grain and a veil. The veil keeps the header text at 4.5:1. An animated cover moves in the glow. An item with no image has a glow in its entity color.

## Authority

- [ADR 0101](../adr/0101-the-design-language-follows-the-shared-rules.md) Decision 4 owns this packet. Its last paragraph gives the method: the app prepares the glow image outside the renderer with the `image` crate, and GPUI draws it below a veil of two stacked gradients. Read Decisions 2 and 4 before the first edit.
- musicindex ADR 0021 R-ART-5 (`../musicindex/docs/adr/0021-cross-app-artwork-rules.md` lines 67 to 87, at musicindex `785ae28`). It gives a glow with grain on the album, track, artist and label page, and a veil at 4.5:1. An item with no image gets a glow in its entity color. An editor header gets no glow.
- musicindex ADR 0028 R-ART-6 (`../musicindex/docs/adr/0028-cross-app-item-pages.md` lines 57 to 73). An animated cover moves in the glow. With Reduce Motion, the glow shows the first frame.
- musicindex ADR 0068 R-A11Y-1 item 3: with Reduce Transparency, the grain and the glow blur are replaced with opaque fills.
- Do not read a musicindex file. This packet gives each value.
- The `rust-dev` skill applies to each Rust change.

## Recorded Facts

Each fact is from HitIt `ad1fcd9`, read on 2026-11-15. A ChatGPT search found them, and the main session examined the items marked "examined".

- `CoverBackdrop::render` (`src/ui/composites/cover_backdrop.rs:30`) draws a color gradient behind the header. It draws no image and no grain. Examined.
- `CoverBackdrop` tokens (`src/ui/tokens.rs:898`) give a height of 320 px, an opacity and luminance limits.
- `src/ui/shells/entity.rs:77` is the one caller of `CoverBackdrop::new`. It wraps `render_contract_header`.
- The release page gives `hero_image` and `cover_color` through `ReleaseDetailBehaviorSlots` (`src/ui/shells/release_page.rs:126`). The Library gives them in `src/ui/shells/library/feed_detail.rs:64`, and the Index in `src/app/search_dispatch.rs:1492`.
- The Library asks for a cover color only for `LibraryDetail::Album` (`src/library/app_impl.rs:6669`).
- The track header (`src/ui/composites/track_header.rs:80`) has no backdrop. The artist and label pages are `src/ui/shells/name_level_page.rs` and `src/ui/shells/publisher.rs`. They have no backdrop.
- `read_or_download` (`src/media/image_cache.rs:359`) reads or downloads the cover bytes. `cover_color` (`src/media/cover_color.rs:30`) decodes them with `image::load_from_memory`.
- `Cargo.toml:43` enables the `image` decoders JPEG, PNG, GIF and WebP.
- `fetch_blocking` (`src/media/image_cache.rs:268`) keeps the bytes of an animated GIF. Line 285 gives a static preview from the first frame. `src/ui/primitives/image.rs:134` gives a GIF a stable ID for the GPUI animation.
- The Index detail asks for the static variant (`src/app/search_dispatch.rs:1779`).
- `Environment` (`src/ui/tokens.rs:1276`) has `reduce_motion`. `src/ui/theme_bridge.rs:85` sets it to `false`. Examined. No field gives Reduce Transparency.
- Tests: `adr_0083_cover_backdrop_keeps_header_text_contrast` (`src/ui/tokens.rs:1716`) and `adr_0083_cover_color_of_a_known_image_is_stable` (`src/media/cover_color.rs:105`). Guard: `adr_0083_cover_backdrop_uses_its_tokens` (`tests/architecture_tests.rs:21234`).

## The Values

No shared rule and no shared token gives a number for the blur, the saturation, the grain or the veil. Main session choice, 2026-11-15: HitIt copies the values of the website code, `search.html` at musicindex `785ae28`, lines 88 to 1266. The orchestrator session sent them on 2026-11-15. Each token comment says "copied from the musicindex.org page header, `search.html` at `785ae28`".

| Part | Dark | Light |
|---|---|---|
| Glow art | Blur 90 px, saturation 1.7, opacity 0.85. The art extends 200 px above, 80 px at each side and 40 px below the header | Same |
| Veil | The background color at 25 % at the top, 60 % at 55 % of the height, 100 % at the bottom | 55 %, 80 %, 100 % |
| Grain | Fractal noise, base frequency 0.85, 3 octaves, a tile of 240 px, no color, overlay blend, opacity 0.22 | Opacity 0.28 |
| Black and white art | The entity color at 40 %, and the art at opacity 0.35 with luminosity blend | Same |

GPUI 0.3.1 has no blur, no blend mode and two stops in a gradient. Thus:

- The app makes the blur, the saturation, the grain and the black and white blend in the prepared image. The overlay of the grain depends on the theme, so the cache key holds the theme.
- The app blurs a small copy of the cover, for example 96 px, with the blur scaled to that size. GPUI scales the image to the header.
- The veil is two stacked gradients of two stops: 0 to 55 % and 55 % to 100 % of the height.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Header backdrop | extend | `src/ui/composites/cover_backdrop.rs` | It draws the backdrop of a page header. Draw the prepared glow image and the veil there. |
| Backdrop tokens | extend | `src/ui/tokens.rs` | `CoverBackdrop` holds the backdrop values. Add the glow, veil and grain values there. |
| Cover bytes | reuse | `src/media/image_cache.rs` | `read_or_download` and `fetch_blocking` give the bytes and keep the GIF frames. |
| Cover decode | reuse | `src/media/cover_color.rs` | It shows the decode with the `image` crate and the color of a cover. |
| Glow image preparation | new | `src/media/header_glow.rs` | Searched `src/media/`, `src/ui/composites/` and `src/ui/primitives/image.rs` in HitIt. No code blurs an image or adds grain. The orchestrator answers for `~/build/musicindex-lib`. |
| Environment flags | extend | `src/ui/tokens.rs` | `Environment` holds `reduce_motion`. Add `reduce_transparency` there, with the value `false`. Task 005 sets it from the system. |

## Required Changes

1. **The glow image.** In `src/media/header_glow.rs`, write a pure function that takes the decoded cover frames and the theme and gives the prepared frames. It scales down, blurs, saturates, and adds the grain with a fixed seed. A black and white cover gets the blend of "The Values". Keep the frame delays of an animated cover.
2. **The preparation path.** Prepare the glow on the background executor. Cache it by the image key and the theme. Follow the path of the cover color (`src/library/app_impl.rs:6669`, `src/app/search_dispatch.rs:1492`). Each page kind of R-ART-5 asks for it: the release page in the Library and the Index, the track page, the name level page and the publisher page.
3. **The view model fact.** The header view model gives a glow fact: the image key, or the entity color when the item has no image. It holds no renderer type.
4. **The backdrop.** `CoverBackdrop` draws the prepared frames with opacity 0.85, at the extents of "The Values", and the veil below the header text. The header that has no prepared image yet draws the veil on the background, with no glow. An editor header gets no glow.
5. **No image.** An item with no image gets a glow in its entity color (R-ART-5 item 3). It has the same veil and grain.
6. **Reduce Motion.** With `reduce_motion`, the glow of an animated cover shows the first frame.
7. **Reduce Transparency.** Add `reduce_transparency` to `Environment` with the value `false` in `src/ui/theme_bridge.rs`. With it, the backdrop draws an opaque fill of the cover color with no grain and no blur.
8. **The tokens and the guard.** Put each value in `CoverBackdrop` in `src/ui/tokens.rs`. Change `adr_0083_cover_backdrop_uses_its_tokens` so that it also covers the new values. Its message names ADR 0101 Decision 4.

## Do Not Change

- No change of the cover color function or its test.
- No glow in an editor header, a sheet, a row or a tile.
- No new dependency. The `image` crate does the work.
- No change of the art radius. Task 003 does that.
- No change of the header buttons. Task 011 does that.
- No `git commit`, `git add` or `git stash`.

## Acceptance Criteria

Mechanical:

- M1: a test of `header_glow` proves that a still cover gives one frame of the prepared size, and that the same input gives the same output.
- M2: a test proves that an animated cover of three frames gives three frames with the same delays.
- M3: a test proves that a black and white cover gets the entity color blend.
- M4: a test of the view model proves that an album, a track, an artist and a label page give a glow fact. An item with no image gives its entity color.
- M5: a contrast test proves that the header text and the muted text have 4.5:1 or more on the veil. The test uses a white glow and a black glow in each theme.
- M6: a test of the environment proves that `reduce_transparency` gives the opaque fill and no grain, and that `reduce_motion` gives the first frame.
- M7: the backdrop guard passes with the new values.
- M8: the fit tests of ADR 0097 pass.
- M9: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and `cargo build --bin hitit` pass.

Visual, for the operator, in Dark and Light at M:

- V1: an album page shows a blurred glow from the cover, with grain. The glow fades into the background before the content. It agrees with the album page of musicindex.org for the same cover.
- V2: a track page, an artist page and a label page show the same glow.
- V3: an animated cover moves in the glow.
- V4: an item with no image shows a glow in its entity color.
- V5: the header text reads clearly on a light cover and on a dark cover.
- V6: a page opens with no visible wait for the glow. The glow comes in after the page.

## Test Commands

- `cargo test --lib media::header_glow`
- `cargo test --lib ui::tokens`
- `cargo test --lib ui::composites::cover_backdrop`
- `cargo test --test architecture_tests`
- `cargo test app::fit_check::`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Report

Give the files changed and the tests of M1 to M7 by name. Give the time to prepare one still cover and one animated cover of 20 frames in a release build. Give the contrast ratios of M5.

A delegate does not commit. It leaves its change in the working tree and reports it. Write each comment and the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if GPUI 0.3.1 cannot draw the prepared frames of an animated cover.
- Stop if the preparation of one cover takes more than 200 ms in a release build.
- Stop if a contrast pair of M5 is below 4.5:1 with the veil values of "The Values".
- Stop if a page kind of R-ART-5 has no header that can hold the backdrop.
