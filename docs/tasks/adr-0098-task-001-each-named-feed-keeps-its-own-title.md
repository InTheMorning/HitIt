# ADR 0098 Task 001: Each Named Feed Keeps Its Own Title

Status: Implemented - operator visual checks V1 and V2 open, 2026-11-18. The orchestrator asked for this packet on 2026-11-18, with the approval of the operator.

## Goal

After a download, the release page and the publisher page of a Library album show the title of each named feed. A role row shows "Label: Sir Libre Records", not a GUID. The page of Sir Libre Records shows the title "Sir Libre Records", not the title of a different feed.

## Authority

- [ADR 0098](../adr/0098-the-music-shell-pages-lists-and-search.md) Decision 11.6: each other named feed with a stated role shows as a role row, for example "Label: Sir Libre Records". An "Also named" row shows its feed title.
- [ADR 0077](../adr/0077-publisher-feed-artist-binding.md), Artist page title: the title is the `<title>` of the publisher feed.

## Defect

The [third agent walk of 2026-11-18](../reviews/agent-walk-2026-11-18-third.md) defect 1, and observation O1 of the [journey record of 2026-11-18](../research/2026-11-18-user-journeys/observations.md):

- Before the download, the release page of "Motive to Kill" shows "Label: Sir Libre Records".
- After the download, it shows "Label: 34bf795f-f551-4db1-8877-ad19aa5f89c8".
- After the download, the page of Sir Libre Records shows the title "Motive to Kill".

Screenshots: `docs/reviews/screenshots/agent-walk-2026-11-18-third/label-guid-after-download.png` and `publisher-page-after-download.png`.

## Cause

The main session read the code and the live Music Index API on 2026-11-18.

API facts. `GET /v1/feeds/0a8495bd-9b8f-5e6b-b522-f5b986d5faac` with the include list `FEED_FULL` gives:

- `publisher_feed_title` "Motive to Kill" and `publisher_feed_guid` `010c031b-667f-4f74-bf57-fea7297f07c0`. That is the artist feed. Its title is also "Motive to Kill".
- Two `publisher` entries with `direction` `music_to_publisher`. Each has its own `remote_feed_title`: "Motive to Kill" for `010c031b-…` (role `artist`), and "Sir Libre Records" for `34bf795f-…` (role `label`).

Code facts:

- The hydration (`src/application/queries/library.rs`, the call of `upsert_feed_publisher_relationships`) stores one `publisher_feed_title` for the album. It writes the same value on each `feed_publisher_relationships` row of the album. The `remote_feed_title` of each entry is not stored.
- `db::publisher_relationships::named_feeds` gives the stored `publisher_feed_title` to the first named feed only. Each other named feed gets the title of a Library feed with its GUID, or no title. Sir Libre Records is not a Library feed, so its title is `None`.
- `ReleaseDetailVm::named_feed_row` (`src/view_models/entity_detail.rs`) shows the GUID when the title is `None`.
- `fetch_library_publisher_page` (`src/application/queries/library.rs`) takes the page title from the stored `publisher_feed_title` of the first row that names the page feed. For Sir Libre Records, that row holds "Motive to Kill", the title of the artist feed.
- Before the download, the Index path (`views::named_feeds_from_api`) reads `remote_feed_title` of each entry. Thus the Index page is correct.

## Change

- Store the title of each named feed on its own row: the `remote_feed_title` of the entry. Add a column with a migration, or use an existing column if its meaning agrees. The implementer selects and records the choice in the report.
- `named_feeds` gives each named feed the title of its own row. Use the Library feed title only when the row holds no title.
- `fetch_library_publisher_page` takes the page title from the title of the page feed on its rows, not from `publisher_feed_title`.
- Keep `publisher_feed_title` for each reader that wants the title of the first named feed. Read each reader of it before a change, and list them in the report.
- A Library album that the app hydrated before this change gets the titles at its next hydration. If the stored rows have no title, the rows keep the current result until then. Write this in the report. Do not add a new request.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Publisher relationship store | extend | src/db/publisher_relationships.rs | It stores the rows and reads the named feeds. |
| Library hydration | extend | src/application/queries/library.rs | It writes the rows and reads the Library publisher page. |
| Index named feeds | reuse | src/views.rs | `named_feeds_from_api` already reads `remote_feed_title` of each entry. |
| Release role rows | reuse | src/view_models/entity_detail.rs | `named_feed_row` shows the title. No change. |
| Database migrations | extend | src/db.rs | Only if a new column is necessary. |

## Mechanical Criteria

- A test of `named_feeds` stores two `music_to_publisher` entries with different `remote_feed_title` values, for example "Motive to Kill" (artist) and "Sir Libre Records" (label). Each named feed gets its own title. Before the change, the second title is `None`. The implementer records that failure.
- A test of `fetch_library_publisher_page` for the label GUID gives the title "Sir Libre Records". Before the change, it gives "Motive to Kill".
- A test proves that a row with no stored title still uses the Library feed title.
- The commit checks of AGENTS.md pass.

## Visual Criteria

- V1: download "Motive to Kill". The release page shows "Label: Sir Libre Records".
- V2: open Sir Libre Records from that row. The page title is "Sir Libre Records".
- Incorrect: a role row shows a GUID, or a feed page shows the title of a different feed.

The walk needs `cdn.kolomona.com` and `sirlibre.com` in the allowed hosts.

## Stop And Ask

- Stop if the change needs a new request to Music Index.
- Stop if a reader of `publisher_feed_title` needs the title of a feed other than the first named feed.

A delegate does not commit. It leaves the change in the working tree and reports it. Apply the STE skill at `/home/citizen/.agents/skills/asd-ste100/SKILL.md` to each comment and report.

## Report

Storage choice: no new column. Migration 28 only deletes old rows. The column `publisher_feed_title` of `feed_publisher_relationships` already means "the title of the publisher feed that this row names". A `music_to_publisher` row now stores the `remote_feed_title` of its own entry in that column. When the entry has no title, the row takes the album-level title only if the response has exactly one `music_to_publisher` entry. Otherwise the row stores null. A row of another direction stores the album-level title, as before.

Readers examined: `named_feeds` and `local_albums_for_publisher` read the stored column. `fetch_library_publisher_page` takes the page title from `local_albums_for_publisher`, so it needs no change. `CuratorLink::of` in `playlist_curators.rs` and the hydration call read the API field `Feed.publisher_feed_title`, not the stored column. No reader needs the title of a feed other than its own row.

Failing output before the fix: `adr_0098_named_feeds_each_keep_their_own_title` gave `left: None, right: Some("Sir Libre Records")`. `adr_0098_publisher_page_title_is_the_title_of_the_page_feed` gave `left: Some("Motive to Kill"), right: Some("Sir Libre Records")`. The test for the Library title fallback passed before and after.

Changed test: `adr_0077_publisher_relationship_hydration_stores_each_field` asserted the same title on two entries. Its fixture entries now carry `remote_feed_title`, and the second row expects its own title.

An album hydrated before this change: migration 28 (`named_feed_titles`) deletes the stored rows of each album with two or more named feeds, because each row held the album-level title. The identity backfill then hydrates the album again, and the hydration writes the title of each entry. An album with one named feed keeps its row. An album with a stored `publisher` hold keeps its rows, because the hydration does not write a held `music_to_publisher` row. The hydration writes each other column of a deleted row again from the response.
