# ADR 0101 Task 009: Open On musicindex.org

Status: Waits - 2026-11-15. This packet starts after [task 001](adr-0101-task-001-shared-tokens.md) passes its checks.

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

Main session choice, 2026-11-15: a Library page shows the link when it has the feed GUID. HitIt makes no request to know if the index holds the item. An Index page always shows the link. The operator can change this at the walk (open point 4 of the [phase plan](../plans/adr-0101-design-language-phase-plan.md#open-points)). A Library page with no feed GUID shows "Not in the index yet".

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Identity actions | extend | `src/ui/composites/identity_action.rs` | It draws each link out of a header. Add the kind `MusicIndex`. |
| Page address | new | `src/view_models/musicindex_link.rs` | Searched `src/view_models/`, `src/api.rs` and `src/ui/composites/` in HitIt. No code builds a musicindex.org page address. |
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
