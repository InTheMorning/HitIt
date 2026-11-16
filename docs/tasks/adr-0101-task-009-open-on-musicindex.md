# ADR 0101 Task 009: Open On musicindex.org

Status: Implemented - operator visual checks open, 2026-11-16.

## Goal

Each release page, track page and artist page of HitIt has a link that opens the same item on musicindex.org. The link is an icon button in the header with the tooltip "Open on musicindex.org".

## Authority

- [ADR 0101](../adr/0101-the-design-language-follows-the-shared-rules.md) Decision 5, "Identity and provenance", and Decisions 11 and 12 own this packet.
- musicindex ADR 0023 R-LINK-1 (`../musicindex/docs/adr/0023-cross-app-words-links-and-messages.md` lines 54 to 70, at musicindex `785ae28`):
  1. Each release, track and artist page in HitIt has a link that opens the same item on musicindex.org.
  2. The addresses: a feed `https://musicindex.org/#feed=GUID`, a track `https://musicindex.org/#track=GUID&in=FEED_GUID`, a name `https://musicindex.org/#name=KEY`.
  3. The link is "Open on musicindex.org".
  5. When the index does not hold the item, the app shows "Not in the index yet" in place of the link.
- musicindex ADR 0028 R-PAGE-2 item 1 (`../musicindex/docs/adr/0028-cross-app-item-pages.md` lines 57 to 73): a link out is an icon button with an instant tooltip. Item 5: on Desktop, an icon button is 32 px with a 16 px icon.
- musicindex ADR 0026 R-TOOLTIP-1: an icon alone needs a tooltip and an accessibility label (ADR 0101 Decision 10).
- The `rust-dev` skill applies to each Rust change.

## Recorded Facts

Each fact is from HitIt `ad1fcd9`, read on 2026-11-15. A ChatGPT search found them, and the main session examined the items marked "examined".

- `IdentityActionKind` (`src/ui/composites/identity_action.rs:20`) has Website, Nostr and RSS. `identity_action_button` (`:63`) draws them. Examined.
- The track page gives its identity actions in `src/view_models/track_detail.rs:376`. The release page gives them in `src/ui/shells/entity.rs:147`. The shared track surface gets them through behavior slots (`src/ui/shells/track.rs:156`).
- `src/api.rs:11` gives only the API address, `https://api.musicindex.org`. No code builds a musicindex.org page address.
- The name level page view model has `key: ArtistNameKey` (`src/view_models/name_level_page.rs:322`). The publisher page view model has `feed_guid` (`src/view_models/publisher_page.rs:72`). Examined.
- `IconName` resolves `LucideIconName::Globe` for the public playlist (`src/ui/icons.rs:213`). Examined.
- Guards: `release_feed_identity_actions_use_shared_renderer` (`tests/architecture_tests.rs:8209`) and `track_identity_links_use_shared_renderer` (`:8238`). Test: `website_action_is_text_only_until_catalog_has_link_icon` (`src/ui/composites/identity_action.rs:99`).

Main session choice, 2026-11-15: an artist page that is a feed page (`publisher.rs`) uses the feed address. A name level page uses the name address with its `ArtistNameKey`. The packet confirms that `ArtistNameKey` gives the KEY of musicindex.org, and stops if it does not.

Operator choice, 2026-11-15: a Library page shows the link when it has the feed GUID. HitIt makes no request to know if the index holds the item. An Index page always shows the link. (open point 4 of the [phase plan](../plans/adr-0101-design-language-phase-plan.md#open-points)). A Library page with no feed GUID shows "Not in the index yet".

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Identity actions | extend | `src/ui/composites/identity_action.rs` | It draws each link out of a header. Add the kind `MusicIndex`. |
| Button tooltip | extend | `src/ui/primitives/button.rs` | Set the tooltip show delay for the instant link tooltip. |
| Page address | new | `src/view_models/musicindex_link.rs` | Searched `src/view_models/`, `src/api.rs` and `src/ui/composites/` in HitIt. No code builds a musicindex.org page address. Searched musicindex-lib 953e464 and Studio 44d1895 through the orchestrator on 2026-11-15: no link builder. Keep it in one file with no HitIt type, so that it can move to musicindex-lib. |
| Track page actions | extend | `src/view_models/track_detail.rs` | It gives the identity actions of a track. |
| Release page actions | extend | `src/ui/shells/entity.rs` | It gives the identity actions of a release. |
| Artist feed page header | extend | `src/ui/shells/publisher.rs` | It draws the header of an artist feed page. Give it the link. |
| Name level page header | extend | `src/ui/shells/name_level_page.rs` | It draws the header of a name level page. Give it the link. |

## Required Changes

1. **The address.** One pure function gives the address of a feed, a track and a name, as R-LINK-1 item 2 gives. It encodes each value for a URL fragment.
2. **The kind.** Add `IdentityActionKind::MusicIndex` with an icon, the tooltip and the accessibility label "Open on musicindex.org". It opens the address in the system browser.
3. **The pages.** The release page gives the feed address. The track page gives the track address with the feed GUID. The artist feed page gives the feed address. The name level page gives the name address.
4. **No GUID.** A page that cannot build the address shows "Not in the index yet" as muted text in place of the button.
5. **The order.** The link is the last link out of the header, after RSS.

## Do Not Change

- No change of the Website, Nostr and RSS actions.
- No request to the index to find if it holds the item.
- No link on a playlist page. R-LINK-1 names release, track and artist pages.
- No `git commit`, `git add` or `git stash`.

## Acceptance Criteria

Mechanical:

- M1: a test of the address function gives the three forms of R-LINK-1 item 2, with an encoded value.
- M2: a test of the track page view model proves the track address with `in=FEED_GUID`.
- M3: a test proves the feed address on a release page and an artist feed page, and the name address on a name level page.
- M4: a test proves "Not in the index yet" when a page has no GUID.
- M5: the identity guards pass, and they cover the new kind.
- M6: the fit tests of ADR 0097 pass.
- M7: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and `cargo build --bin hitit` pass.

Visual, for the operator, in Dark and Light at M:

- V1: a release page shows the musicindex.org button after RSS. The tooltip "Open on musicindex.org" shows at once.
- V2: the button opens the same release on musicindex.org in the browser.
- V3: a track page and an artist page open the same item on musicindex.org.

## Test Commands

- `cargo test --lib view_models::musicindex_link`
- `cargo test --lib view_models::track_detail`
- `cargo test --lib ui::composites::identity_action`
- `cargo test --test architecture_tests`
- `cargo test app::fit_check::`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Report

Give the files changed and the tests of M1 to M4 by name. Give the icon that you used. Give the address of one real release, track and name page.

A delegate does not commit. It leaves its change in the working tree and reports it. Write each comment and the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if `ArtistNameKey` is not the KEY that musicindex.org uses.
- Stop if the icon catalog of `gpui-kit-assets` has no icon for a link out.

## Progress - 2026-11-16

- `ArtistNameKey.key` is the `KEY` in musicindex.org `#name=KEY`. musicindex ADR 0067 item 100 uses that key for `/v1/feeds/by-name?artist_key=KEY`.
- `gpui-kit-assets` includes the Lucide `ExternalLink` icon.
- `page_url` encodes feed, track, feed GUID and name values in the URL fragment.
- Release, track, feed artist and name pages provide the required MusicIndex address. Pages without a GUID show `Not in the index yet`.
- The shared identity renderer uses the `ExternalLink` icon and the tooltip `Open on musicindex.org`. The new architecture guard covers the release and track renderers.
- M1 tests: `builds_encoded_feed_address`, `builds_track_address_with_feed`, `builds_name_address`.
- M2 test: `musicindex_track_link_includes_its_feed_guid`.
- M3 tests: `identity_actions_include_website_nostr_and_rss`, `musicindex_link_uses_the_publisher_feed_guid`, `musicindex_link_uses_the_name_key`.
- M4 tests: `release_without_index_guid_projects_not_in_index_yet`, `track_without_guid_projects_not_in_index_yet`.
- V1 to V3 remain open for the operator. The agent has not inspected them in HitIt on an Xvfb display.

## Operator Visual Check

Walk each step in Dark and Light at M. MusicIndex and a system browser must be available.

1. Compile HitIt with `cargo build --bin hitit`.
2. Start HitIt with `target/debug/hitit`.
3. Search for `I Don't Wanna Leave` and open that release page.
4. Examine the header buttons. The ExternalLink icon button must follow RSS. Hover over it. The tooltip must show immediately and read `Open on musicindex.org`.
5. Use Tab to select the button, then push Enter. The browser must open `https://musicindex.org/#feed=f46a2d4b-4f94-5b84-8808-5f095efbc4ca`.
6. Open the track `I Don't Wanna Leave`. Use Tab to select its MusicIndex button, then push Enter. The browser must open `https://musicindex.org/#track=96b17655-2548-4fd9-80ff-a346e49001f2&in=f46a2d4b-4f94-5b84-8808-5f095efbc4ca`.
7. Open the artist feed page `DJ Yo!`. Use Tab to select its MusicIndex button, then push Enter. The browser must open `https://musicindex.org/#feed=03d4ab60-da21-4453-8692-1c1e24ab2d49`.
8. Open the name page `$2 Holla`. Make sure its header has the same icon button and that it opens `https://musicindex.org/#name=%242%20holla`.

- Incorrect: a link opens an incorrect item, button sequence changes, the tooltip does not show immediately, or a page without a GUID shows a button.
- Close HitIt and each browser tab that these steps opened. These steps change no Library data.

## Primary Agent Check - 2026-11-16

The second delegate run stopped at its time limit of 600 s after it wrote the code. The primary agent read the diff and made these corrections:

- `publisher.rs` and `name_level_page.rs` had two copies of the button code, and four shells had copies of the "Not in the index yet" label. `musicindex_link` and `not_in_index_label` (`src/ui/composites/identity_action.rs`) own them. `musicindex_link` takes `MusicIndexLinkDisplay`, as the ADR 0038 guard requires.
- `entity_detail.rs` uses `MUSICINDEX_NOT_IN_INDEX_TEXT`, not a second literal.

Gate, run by the primary agent: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (3224 library tests, 413 architecture tests) and `cargo build --bin hitit` pass.

## Walk Result - 2026-11-16

The agent walked on an Xvfb display with the debug binary, in Light at M.

- V1: Pass. On "Fox Trot Tunes Volume 2", the ExternalLink button is after RSS. The tooltip "Open on musicindex.org" shows in less than 0.6 s.
- V3, part: the artist page "minimalizman" shows the button below the name. The track page "rose" shows the button in its row of links.
- V2 and V3: Open. The display has no browser, so the agent did not open an address.

The operator confirmation stays open.
