# ADR 0101 Task 002: The Page Header Glow

Status: Implemented - operator visual checks open, 2026-11-16.

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
| Glow image preparation | new | `src/media/header_glow.rs` | Searched `src/media/`, `src/ui/composites/` and `src/ui/primitives/image.rs` in HitIt. No code blurs an image or adds grain. Searched musicindex-lib 953e464 (search/, fit/) and Studio 44d1895 through the orchestrator on 2026-11-15: no glow code. |
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

## Stop Report - 2026-11-16

A delegate stopped before the first edit. The primary agent examined the first value by hand and got the same result.

### M5 Contrast With The Veil Of "The Values"

Model: a white glow or a black glow at opacity 0.85 on the page background, then the veil. The veil alpha changes linearly between its stops. The text colors are `Label` and `SecondaryLabel` of `src/ui/tokens.rs`. The height is a fraction of the veil height.

| Condition | 0 % | 40 % | 55 % | 70 % | 85 % |
|---|---|---|---|---|---|
| Dark, white glow, `Label` | 2.22 | 4.43 | 5.95 | 9.17 | 13.76 |
| Dark, white glow, `SecondaryLabel` | 1.06 | 1.87 | 2.52 | 3.88 | 5.82 |
| Light, black glow, `Label` | 6.24 | 9.74 | 11.32 | 13.00 | 14.84 |
| Light, black glow, `SecondaryLabel` | 2.46 | 3.84 | 4.46 | 5.13 | 5.86 |

A black glow in Dark and a white glow in Light give 4.5:1 or more at each height.

- In Dark with a white glow, `Label` gets 4.5:1 only below approximately 41 % of the veil height. `SecondaryLabel` gets it only below approximately 78 %.
- In Light with a black glow, `SecondaryLabel` gets 4.5:1 only below approximately 56 %.

### Questions

1. The veil values. Is the stack of the website different from "The Values"? For example, does a layer below the text have its own fill? Or do the veil values change so that M5 passes at the top of the header?
2. The image of an artist page and a label page. These pages show `CollectionArt`: a mosaic of logos or a logo in the entity color. They have no one cover. Which image gives the glow: the provided image of `CollectionArtVm::ProvidedImage`, one cover of the mosaic, or the entity color?

### Facts For The Next Attempt

- GPUI animates a `RenderImage` of more than one frame in `gpui::img(ImageSource::Render(..))` when the element has an id. `RenderImage::new` takes `image::Frame` values with the bytes B, G, R, A. Thus Stop And Ask item 1 does not apply.
- The track page header (`TrackHeader`) has no backdrop and no cover color path.
- The `Environment` struct literals in `src/ui/theme_bridge.rs` and in a test of `src/ui/tokens.rs` need the new field.
- A grain in a prepared image of 96 px is approximately 10 times larger on the screen. A prepared image of 240 px, the grain tile, is better.

## Answers - 2026-11-16

The orchestrator session sent these answers.

Operator choice line, 2026-11-16, question 1 (the veil): "Limit the glow".

- Keep the blur, the saturation, the grain, the veil and the extents of "The Values".
- In `header_glow`, limit the luminance of the prepared glow for each theme: a maximum in Dark and a minimum in Light. Use the method of `CoverBackdrop::tone` (`DARK_MAX_LUMINANCE` and `LIGHT_MIN_LUMINANCE` in `src/ui/tokens.rs`). Keep the hue.
- Calculate each limit so that M5 passes for Label and SecondaryLabel at the top of the veil (0 %). Use a white glow and a black glow in each theme.
- Put each limit in `CoverBackdrop` with a comment that names ADR 0101 Decision 4 and R-ART-5 item 2.
- The report gives the limits and the M5 ratios.
- Website fact (`search.html` at musicindex `804589b`): the website has no layer below the text, and it has the same contrast fault. A subsequent task corrects the website. This packet does not wait for it.

Operator choice line, 2026-11-16, change of the method of question 1: the website applied "Limit the glow" (musicindex `84ad724`, `docs/tasks/site-glow-task-001-limit-the-glow.md`). HitIt uses the website method, so that the two apps agree (V1). This replaces the luminance limits for each pixel above.

- One opacity value limits the glow of each page. Do not limit each pixel. A lower opacity mixes the glow with the background, and the hue stays.
- The opacity is the highest value, with a maximum of 0.85, at which Label and SecondaryLabel keep 4.5:1 at the top of the veil. `heroGlowOpacity()` in `search.html` calculates it. It reads the mean color of each 6 x 6 block of a 24 x 24 sample after the saturation of 1.7.
- The black and white blend and the entity tint are calculated in the same procedure, with their layer color.
- Values (custom properties of `.hero-glow`): `--glow-max` 0.85, `--glow-saturate` 1.7, `--glow-tint` 0.45, `--glow-muted-tint` 40 %, `--glow-muted-art` 0.35, `--veil-top` 25 % in Dark and 55 % in Light. `--glow-safe` is 0.24 in Dark and 0.37 in Light.
- HitIt has the pixels, so the safe value is not necessary after the image is prepared. Use the safe value only before the preparation ends, or show no glow until it ends.
- The website opacities for its test covers: Dark, white 0.49 (muted blend), yellow 0.27, black 0.85. Light, black 0.73 (muted blend), white 0.85.
- M5 tests the calculated opacity with a white glow and a black glow in each theme.

Answer to question 2 (the image of an artist page and a label page). This is not an operator question. R-ART-3 item 2 and ADR 0067 item 20 give the answer, and "The Values" copy the website. Do as `search.html` does (`heroHtml` line 3289, `tuneHeroGlow` line 3260):

- The candidates are the own image when it exists (`CollectionArtVm::ProvidedImage`). When there is no own image, the candidates are the first 8 covers of the mosaic.
- Measure the colorfulness of each candidate with the Hasler and Suesstrunk metric on a sample of 24 x 24 px, as `imageColorfulness` does. Keep each candidate with a score of 18 or more. The glow uses the one or two highest, adjacent in two equal columns.
- When all candidates are black and white, use the muted blend of "The Values": the entity color at 40 %, and the art at 0.35 with luminosity.
- When there is no image, use the entity color at opacity 0.45 (`.hero-glow-tint`) below the art opacity of 0.85.

The glow box (website fact):

- The glow box is 560 px high, and 460 px at a window width of 1100 px or less. The box starts at the top of the page. Its height does not change with the header height.
- The veil fractions are fractions of that box, not of `CoverBackdrop::HEIGHT_BASE` (320).
- Use the 560 px box, or tell in the report why the header height is better.

The track page:

- The track page has no backdrop. Add one to `TrackHeader`, as R-ART-5 item 1 requires.
- Its image is the track image. When the track has no image, it is the album cover (R-ART-3 item 1).

When the packet is complete or stops again, send a report to the orchestrator session.

## Implementation Report - 2026-11-16

A delegate wrote this change. It is not committed. The main agent checks it.

### Method

- GPUI has no blur, no blend mode, no tiling and only two-stop gradients. `media::header_glow` thus prepares the glow in pixels with the `image` crate. The view draws the prepared frames as opaque images at one calculated opacity.
- Steps: reduce each candidate to 96 x 96, blur, saturate (1.7), blend (muted blend or entity color), enlarge to 240 px for each column, add grain (overlay, hashed noise).
- One opacity limits each page (operator choice of 2026-11-16). The opacity is the highest value, with a maximum of 0.85, at which `Label` and `SecondaryLabel` keep 4.5:1 at the top of the veil. The sample is the mean color of each 6 x 6 block of a 24 x 24 sample. The worst block and the worst frame decide.
- The safe value (0.24 Dark, 0.37 Light) is not used. Selection of this delegate: the page shows no glow until the glow is prepared. The header does not move, because the padding stays.
- Selection of covers: the colorfulness of Hasler and Suesstrunk on 24 x 24, with a limit of 18. The glow keeps the one or two highest covers, in page order. If no cover reaches 18, the glow takes the highest and uses the muted blend. If no cover decodes, the glow has the entity color.
- An animated GIF keeps its frames (maximum 48) only when it is the one candidate.
- Glow box: 560 px high (460 px at a window width of 1100 px or less) from the top of the page. The veil fractions are fractions of that box. The header height is not used.
- Reduce Transparency draws an opaque fill of the main glow color, with no grain and no blur. Reduce Motion draws the first frame.

### Files changed

New:

- `src/media/header_glow.rs` (preparation, opacity, cache request, `GlowImage`)
- `src/view_models/header_glow.rs` (`HeaderGlowVm`, `GlowEntity`)
- `src/presentation/header_glow.rs` (`header_glow`, `HeaderGlowLoads`)

Changed:

- Tokens and composites: `src/ui/tokens.rs`, `src/ui/composites/cover_backdrop.rs`, `src/ui/composites/track_header.rs`, `src/ui/composites/track_detail_surface.rs`
- Shells: `src/ui/shells/release_page.rs`, `track_page.rs`, `track.rs`, `entity.rs`, `name_level_page.rs`, `publisher.rs`, `library/detail.rs`, `library/feed_detail.rs`, `library/track_detail.rs`
- View models: `src/view_models/mod.rs`, `name_level_page.rs`, `publisher_page.rs`, `release_page.rs`, `track_page.rs`
- Media and application: `src/media/mod.rs`, `src/media/image_cache.rs`, `src/application/queries/images.rs` (`FetchCoverColor` replaced by `FetchHeaderGlow`)
- Presentation and hosts: `src/presentation/mod.rs`, `src/presentation/collection_art.rs`, `src/app.rs`, `src/app/search_dispatch.rs`, `src/app/name_level_dispatch.rs`, `src/app/publisher_dispatch.rs`, `src/library.rs`, `src/library/app_impl.rs`
- Guard: `tests/architecture_tests.rs` (`adr_0083_cover_backdrop_uses_its_tokens` now requires the new token calls and names ADR 0101 Decision 4)
- This packet (this section only; the Status line is not changed)

The old cover-color path is removed (`cover_colors`, `peek_cover_color`, `fetch_cover_color_blocking`, `CoverColorState`, `cover_color_for_url`, `index_cover_color`). The old tokens `HEIGHT_BASE`, `STRENGTH`, `height()` and `background()` are removed. `src/media/cover_color.rs` and its test are not changed, as the packet says. `cover_color()` has no caller now, except its own test. The main agent decides if it is deleted.

### Tests of M1 to M7

- M1: `media::header_glow::tests::adr_0101_header_glow_m1_still_cover_gives_one_stable_frame`
- M2: `media::header_glow::tests::adr_0101_header_glow_m2_animated_cover_keeps_its_frames_and_delays`
- M3: `media::header_glow::tests::adr_0101_header_glow_m3_black_and_white_cover_gets_the_entity_blend`
- M4: `view_models::header_glow::tests::adr_0101_header_glow_m4_own_image_is_the_one_candidate`, `..._m4_covers_give_eight_candidates_at_most`, `..._m4_no_image_gives_the_entity_color`. For each page kind: `view_models::release_page::tests::adr_0101_header_glow_m4_album_page_glow`, `view_models::track_page::tests::adr_0101_header_glow_m4_track_page_glow`, `view_models::name_level_page::tests::adr_0101_header_glow_m4_artist_page_uses_its_covers`, `view_models::publisher_page::tests::adr_0101_header_glow_m4_feed_page_glow` (it covers the label)
- M5: `ui::tokens::tests::adr_0101_header_glow_m5_text_keeps_contrast_in_each_theme` (real theme colors). Also `media::header_glow::tests::adr_0101_header_glow_m5_text_keeps_contrast_at_the_calculated_opacity` and `..._prepared_opacity_keeps_text_contrast`
- M6: `ui::composites::cover_backdrop::tests::adr_0101_header_glow_m6_reduce_motion_draws_the_first_frame` and `..._m6_reduce_transparency_draws_an_opaque_fill`
- M7: `adr_0083_cover_backdrop_uses_its_tokens` in `tests/architecture_tests.rs`
- Colorfulness: `media::header_glow::tests::adr_0101_header_glow_colorfulness_separates_gray_from_color` and `..._choice_keeps_the_one_or_two_most_colorful`
- Website agreement: `ui::tokens::tests::adr_0101_header_glow_opacity_agrees_with_the_website` (each value within 0.03)
- Cache and query: `application::queries::images::tests::fetch_header_glow_prepares_and_keeps_the_glow`

### Contrast ratios of M5 (calculated opacity, top of the veil)

| Theme | Glow | Opacity | Contrast of the worst text color |
| --- | --- | --- | --- |
| Dark | white | 0.250 | 4.50 |
| Dark | black | 0.850 | 7.97 |
| Light | white | 0.850 | 6.64 |
| Light | black | 0.371 | 4.50 |

A white glow in Dark and a black glow in Light are limited to the exact 4.5:1 level. The other two reach the maximum of 0.85.

### Opacities and the website values

| Case | This app | Website | Difference |
| --- | --- | --- | --- |
| Dark, white cover (muted blend) | 0.494 | 0.49 | 0.004 |
| Dark, yellow | 0.262 | 0.27 | 0.008 |
| Dark, black | 0.850 | 0.85 | 0.000 |
| Light, black cover (muted blend) | 0.729 | 0.73 | 0.001 |
| Light, white | 0.850 | 0.85 | 0.000 |

No difference is more than 0.03. The muted blend uses the entity color of an album (`EntityFeed`) as the tint. The website tint is not known to this report, and the values agree with it.

### Time to prepare, release build

Build: `CARGO_TARGET_DIR=<scratchpad>/target-002 cargo build --release`. A temporary example (deleted after the run) timed `prepare_from_encoded`, three runs for each input. The inputs are synthetic.

- One still cover, 1400 x 1400 JPEG: 22, 18 and 19 ms.
- One animated cover, GIF of 20 frames at 600 x 600: 114, 110 and 111 ms.

All times are below the limit of 200 ms. A real GIF has other content and size. A GIF with the 48 frame cap takes about 2.4 times the 20 frame time (inference).

### Risks and open points

- The glow box is absolute and 560 px high. On a short page it can extend the scroll range. It is `overflow_hidden`, so the extension is of the box, not of the art.
- `TrackHeader` now sits in `CoverBackdrop`, which adds padding of `Spacing::MD` around the header.
- The grain is hashed noise, not the `feTurbulence` of the website. Grain pixels are not in the sample of the opacity.
- The veil color is `SystemBackground`.
- A feed page with a role other than label has the artist color.
- The Library and Index glow animates for one animated cover. The Index detail page used static covers before.
- Token-level tests use the real theme colors. The unit tests in `media::header_glow` use theme colors that were copied by hand.
- The STE checker reports many findings on `.rs` files, because it reads code as prose. Comments were written to the STE rules directly. The checker was not used as a gate on Rust files.

### Visual gates

All stay OPEN. No agent walked them. V1 to V6 need a person, or a walk on Xvfb by the main agent.

- V1 OPEN
- V2 OPEN
- V3 OPEN
- V4 OPEN
- V5 OPEN
- V6 OPEN (the page shows no glow until the glow is prepared, so the page does not wait)

### Checks

See the final message of the delegate for the result of each command.

## Corrections After The Review - 2026-11-16

The primary agent walked V1 on the Xvfb display (debug binary, Dark, M) and got a second agent to examine the diff. The primary agent examined findings 1, 2, 4, 5 and 6 in the code. Primary agent choices, 2026-11-16:

1. The loads map. `HeaderGlowLoads` keeps a maximum of 16 entries, the capacity of the LRU of `image_cache.rs`. Remove the oldest entry first. When an entry goes, call `window.drop_image` for each of its frames, so the atlas tiles go too.
2. The glow box. Walk result: the glow is a rectangle with hard edges at the top, the left and the right, at the padding of the content column. The box starts at the top of the scroll content of the page. It has the full width of the column, with no padding at its sides. The veil and the window edges stop it. Remove the `p(Spacing::MD)` of `CoverBackdrop`, so the header keeps its position from before this task.
3. The grain. Do not put the grain in the scaled glow image. Prepare one grain tile of 240 px for each theme. Show it at its own dimension, in a pattern of tiles, above the glow and below the veil. The page opacity does not change the grain. Correct the comment. If the tiles add more than 2 ms to a frame, stop and report.
4. Frames. Measure a GIF of 48 frames in a release build. If it takes more than 200 ms, make `MAX_FRAMES` smaller. A GIF with more frames than `MAX_FRAMES` gets a glow from its first frame only. Thus the glow does not move out of step with the cover.
5. A key change on artist and label pages. While a new key prepares, the page keeps the glow of its last key.
6. A failed read. Do not keep a result with a failed candidate in the LRU or in the loads map. The next visit of the page tries again. When the own image of a feed fails, the candidates are the mosaic covers (answer 2).
7. The theme. The flatten color, the tint and the contrast target use the live theme. The cache key holds the profile and Increase Contrast.
8. The tests. The test of the opacity of `prepare` calculates the contrast. The track test has a test of the album cover. The Library album and track glow facts come from the view models, and the host uses `vm.header_glow`.
9. Dead code. Delete `cover_color()` and its test, because no entry point reaches them (CLAUDE.md, Delete Dead Things). This replaces the line "No change of the cover color function or its test". Keep `main_color`. Correct the module comment and the comment of the guard `adr_0083_cover_backdrop_uses_its_tokens`. The guard also covers the values of `glow_style`.
10. The publisher glow uses the sequence of `album_image_urls()`, the sequence of the mosaic.

## Corrections Report - 2026-11-16

A delegate made these corrections. They are not committed. The main agent checks them. The Status line is not changed.

### Result of each item

1. Done. `HeaderGlowLoads` (`src/presentation/header_glow.rs`) keeps at most `HEADER_GLOW_CAPACITY` (16) entries, the capacity of the LRU of `image_cache.rs`, which is now `pub`. A read of an entry makes it the newest. The oldest entry goes first. The renderer images of a removed glow (the frames image and the still copy; the shared grain tile stays) wait in a list. The completion callback of the next load, success or error, calls `cx.drop_image(image, None)` for each. The callback runs outside a window update, so each window is in the window list of the app. Test: `adr_0101_header_glow_loads_keep_sixteen_entries`.
2. Done in code, visual result not checked. `CoverBackdrop` has no padding. The box is at `top(-LG)`, `left(-LG)` and `right(-(LG + overlay scrollbar + SM))` from the header, so it starts at the top of the scroll content and has the full width of the column. These are the paddings of the scroll container and of `page_scroll_content`. The values are in the tokens `page_inset_start` and `page_inset_end`. The window edge and the scroll area clip the box, and the veil is inside it. Test: `adr_0101_header_glow_backdrop_keeps_the_header_in_place` (the header has the same place with and without a glow, at 0, 0). This depends on the shared paddings of the pages (inference from the code, no walk). V1 needs a new walk.
3. Done. The grain is not in the glow image. `grain_tile` makes one 240 px tile for each grain opacity (one for Dark, one for Light). `GlowImage` holds the shared tile. `CoverBackdrop` shows it at 240 px in a grid of tiles above the glow image and below the veil. The tiles have no opacity, so the page opacity does not change them. The comments are corrected. Cost: a draw test with the real composite (test platform, no GPU, release build) took 0.021 ms per frame without the grain and 0.12 ms with it (about 0.1 ms more; 20 to 36 tiles). The limit is 2 ms. The GPU cost is not measured. Tests: `adr_0101_header_glow_grain_is_one_tile_with_its_own_alpha`, `adr_0101_header_glow_grain_pattern_covers_its_area`. The grain is a source-over approximation (white and black pixels with the alpha of the noise). It is not the overlay blend of the website, and a visual check is open.
4. Done. Release build, separate target folder, synthetic 600 x 600 GIF, five runs, median: 20 frames 61.7 ms, 48 frames 143 to 146 ms, 49 frames 118 ms (first frame only). The 48-frame time is below 200 ms, so `MAX_FRAMES` stays 48. A GIF with more than 48 frames gives a glow from its first frame. Test: `adr_0101_header_glow_gif_over_the_frame_limit_gives_the_first_frame`. The earlier 20-frame time of 110 ms is now 62 ms, because the glow image no longer has the enlarge and grain steps. A real GIF has other content (inference).
5. Done. `HeaderGlowVm` has a `scope`. The artist page (key of the name) and the feed and label page (feed GUID) set it. While a new key prepares, the page shows the last glow of the same scope. A page with another scope does not get it. Tests: `adr_0101_header_glow_loads_hold_the_last_glow_of_a_page` and the scope assertions in the view model tests.
6. Done, with one reading (see "Choices of the delegate"). Each candidate read is counted. A glow with a failed read is marked incomplete, and `ImageCache` does not keep it in the LRU. The loads map has no entry for its key. A separate `partial` slot holds it, so the page shows it. The slot is cleared when the page asks for another key, so the next visit of the page prepares the glow again. When the own image does not read, the fallback candidates are read. For a feed with its own image the fallback is the mosaic covers. A track with its own image has the album cover as the fallback. Tests: `fetch_header_glow_does_not_keep_a_glow_with_a_failed_read` and `adr_0101_header_glow_loads_do_not_keep_an_incomplete_glow`.
7. Done. `CoverBackdrop::glow_style` takes an `Environment`. The background, `Label` and `SecondaryLabel` bytes come from `resolve_accessible_color` with the profile, the appearance and Increase Contrast. The entity color of the request already came from the live theme. `GlowStyle` has `profile` and `increase_contrast`, and both are in the fingerprint of the cache key. Test: `adr_0101_header_glow_style_follows_the_live_theme`.
8. Done. The test `adr_0101_header_glow_prepared_opacity_keeps_text_contrast` calculates the contrast of the prepared pixels (the worst block of each frame) and does not call the opacity search. The track tests cover the album cover (`adr_0101_header_glow_m4_track_page_uses_the_album_cover`). `library_album_header_glow` (`release_page.rs`) and `library_track_header_glow` (`track_page.rs`) give the Library facts, with tests, and `library/app_impl.rs` uses them. See "Choices of the delegate" for the reading of "the host uses `vm.header_glow`".
9. Done. `cover_color()`, its constant and its second test are deleted. The two tests that stay now test `main_color`. The module comment is corrected. The guard `adr_0083_cover_backdrop_uses_its_tokens` has a corrected comment, requires the new token calls (inset, grain), and reads the body of `glow_style` for each token and for `resolve_accessible_color`. Its failure message names ADR 0101 Decision 4.
10. Done. `PublisherPageVm::header_glow` uses `album_image_urls()` as the covers. Test: `adr_0101_header_glow_m4_feed_page_glow`.

### Choices of the delegate

- Item 6: "not in the loads map" and "no refetch on each render" conflict. I keep the incomplete glow in a separate `partial` slot, not in the map. The page shows it, and it prepares again at the next visit, which is a visit of another key and a return. A failure of the command itself (the runner) stays a `Failed` entry as before.
- Item 8: the Library host builds the glow before the shell renders, and the shell builds its own `track_page_vm` later. I did not move the glow load into the shell. The host calls the view model functions `library_album_header_glow` and `library_track_header_glow`. The Library track glow reads the stored `track_image_href` or `album_image_href`, and not the `api::Track` fields that `TrackPageVm.header_glow` reads. Both give the same cover for a Library track (inference, no test compares them).
- Item 3: the grain tile is not scaled with the UI scale. It has its own pixel size at each scale.

### Tests added or changed

`presentation::header_glow` (3 tests), `media::header_glow` (grain, GIF over the limit, prepared contrast), `ui::composites::cover_backdrop` (header place), `ui::tokens` (style follows the theme, grain pattern), `view_models` (fallback, scope, track album cover, Library album and track), `application::queries::images` (failed read), `media::cover_color` (2, adapted).

### Visual gates

All stay OPEN (V1 to V6). No agent walked them. The new glow box and the grain need a walk by a person or the main agent on Xvfb.

## Primary Agent Check - 2026-11-16

The primary agent read the correction diff and ran the gate: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (3215 library tests, 413 architecture tests) and `cargo build --bin hitit`. All pass.

The primary agent accepts these choices of the delegate:

- A glow that is not complete stays in a slot of its own. It does not replace a complete glow.
- The Library host builds the glow with the view model functions `library_album_header_glow` and `library_track_header_glow`.
- The grain tile is 240 px at each UI scale. The grain is a texture, not a dimension of the layout.
- The test of `application::queries::images` uses a literal `GlowStyle`.

Release binary times:

- A cover with no animation: approximately 20 ms.
- A GIF of 48 frames: 143 to 146 ms.
- A GIF of 49 frames: 118 ms. It uses only the first frame.
- The grain: 0.122 ms for each frame.

Open, not in this task: the facts column of the artist page wraps the labels in narrow lines. The defect was not examined for its start.

## Walk Result - 2026-11-16

The agent walked on an Xvfb display with the debug binary at M.

- V1: Pass in Dark and Light on "Fox Trot Tunes Volume 2". The glow covers the full column from the top of the page. It has no hard edge, and the grain is small.
- V1 colors: in Dark, the glow is the background color `#0b0b0d` at approximately 640 px. In Light, it is white at approximately 625 px. The agent did not compare it with musicindex.org.
- V2: Pass for the artist page "minimalizman" in Dark and for the track page "rose" (citrongrau) in Light. The label page was not walked.
- V3: Open. The test data has no animated cover.
- V4: Open. The agent did not find an item with no image.
- V5: Pass for the pages above. The test of M5 gives the contrast values for a white cover and a black cover.
- V6: Open.

The operator confirmation stays open.
