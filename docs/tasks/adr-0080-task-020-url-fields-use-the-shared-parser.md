# ADR 0080 Task 020: URL Fields Use The Shared Parser

## Status

Done - 2026-11-22. The main session ran the gate.

- Orchestrator choice, 2026-11-22 (the operator lets the orchestrator make
  choices for suite efficiency): HitIt uses `stophammer-parser` at rev
  7bd898a36eea2aeabfa17b2a8a2e9f192af5472c for `web_url_or_none` and
  `web_link_for_read`. The sort of 46a48e1 stays, because it selects one of
  two rows and the parser does not own that choice.
- Agent choice, 2026-11-22: `website_from_links` skips a row if its URL gives
  no read value and takes the subsequent row, as `funding_url_from_links`
  does.
- Agent choice, 2026-11-22: HitIt trims each value before
  `web_url_or_none`, as the replaced checks did.
- Agent choice, 2026-11-22: the image fetch and the transcript fetch keep
  their own scheme checks. These checks select a network read or a local
  file read. They do not write or show a URL field.
- Agent choice, 2026-11-22: when the read value of a file value on a website
  row is `None`, the compare uses the raw value. A file with a bad URL then
  shows a difference and not an empty cell.

## Goal

Each URL field that HitIt writes or shows gets its rule from the shared
parser. A file that HitIt wrote shows "=" on each website row at the next
compare.

## Authority

- Musicindex ADR 0086 R-FEED-1 items 2 and 3: a Git dependency at a fixed
  rev, and no local copy of a parser function.
- Stophammer ADR 0054 §4: a URL field gives its value only when the scheme
  is `http` or `https`.
- Stophammer ADR 0054 §4a: a website link, a web page link and a live item
  link with no scheme get `https://` when the value is a host name.
- [ADR 0080](../adr/0080-tag-frames-follow-their-owner.md) Decision 3: a
  file that the app wrote shows no difference at the next compare.
  Decisions 8 to 11: each URL frame holds a plain `http` or `https` URL.

## Change

1. `Cargo.toml`: add `stophammer-parser` at the fixed rev, with
   `default-features = false`.
2. `src/metadata.rs`:
   - `web_link_read_value` calls `web_link_for_read`. `web_url_value` trims
     the value and calls `web_url_or_none`.
   - `website_from_links` gives the read value of the row.
   - `format_id3_website_url` (`WOAR`, `WOAF`) gives the read value.
   - `funding_url_from_links`, `valid_license_url` and
     `valid_musicindex_image_url` use `web_url_value`. The image URL keeps
     its limit of 2,048 characters.
   - A new `format_id3_web_url` applies `web_url_value` to `WXXX` and to
     each other `W` frame. `APIC` keeps `format_id3_url`, because its value
     can be a local path.
   - The compare of the rows "Website" and "RSS feed website" uses the read
     value on the two sides.
3. `src/view_models/live_page.rs`: `live_link_url` gives the read value.
   The local function `is_web_link` is deleted.
4. `src/view_models/track_metadata_grid.rs`: `artwork_url` uses
   `web_url_value`.
5. `src/rss/subscribe.rs`: the stored funding URL uses `web_url_or_none`.
6. `src/db/publisher_relationships.rs`: the stored channel image uses
   `web_url_or_none`. The length limit and the control character check
   stay.
7. `src/feed_service.rs`: the sort comment no longer says that the parser
   replaces the sort.

## Open

- From the orchestrator review of 09bfff1 (task 019), low: `tmcl_value`
  splits the value of a performer row on " / " before it calls
  `involved_people_value`. A name that contains " / ", for example
  "AC / DC", becomes the two names "AC" and "DC" with the same instrument.
  The crate cannot report `ContainsPairSeparator` for this path. This is
  the same " / " risk as `TXXX:Podcast Contributors`, because
  `merge_joined_values` joins the row names with " / ". The orchestrator
  decides the owner of the two subsequently, and this task does not change
  the code.

## Do Not Change

- The `rss_link`-first sort of `hydrate_feed_identity`.
- Endpoint parsing: `src/rss/host.rs`, `src/broadcast/live_value.rs`,
  `src/config.rs` and `src/remote_media.rs`.
- `src/assets/tokens.json` and the musicindex-lib crates.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Web URL rule | reuse | ~/.cargo/git/checkouts | `stophammer_parser::web_url_or_none` at rev 7bd898a, file `src/web_link.rs`. |
| Website link read rule | reuse | ~/.cargo/git/checkouts | `stophammer_parser::web_link_for_read` at rev 7bd898a, file `src/web_link.rs`. |
| Website row selection | reuse | src/feed_service.rs | The sort of 46a48e1 in `hydrate_feed_identity`. |
| URL compare value | extend | src/metadata.rs | `normalized_field_compare_value` reads the website rows before `normalized_url_compare_value`. |

## Acceptance Criteria

Mechanical:

1. A channel website "www.strangeloveofmusic.wixsite.com/strangelove" with
   no scheme gives the `WOAR` value
   "https://www.strangeloveofmusic.wixsite.com/strangelove". Test
   `adr_0080_task_018_website_with_no_scheme_gets_https_woar`.
2. The `rss_link` row comes before the `rss` row. Test
   `adr_0080_task_018_website_takes_the_musicindex_row`.
3. After `id3_edits_for_track_context` writes a file with two website links
   with no scheme, `aligned_compare_rows` gives the rows "Website" and
   "RSS feed website" the status `ComparisonStatus::Match`. Test
   `adr_0080_task_020_written_website_matches_its_row`.
4. A live item link with no scheme gives the `https://` read value. A
   different scheme gives `None`. Test `adr_0089_live_link_accepts_only_http_and_https`.
5. Each existing test passes. A changed assertion is listed in the report
   with its cause.

Visual: none.

## Checks

- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo build --bin hitit`
