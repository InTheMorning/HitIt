# musicindex ADR 0081 Task 002: The Playlist Links

Status: Done. The agent did the live check in four themes. The visual checks of the operator are open.

## Goal

HitIt builds musicindex ADR 0081 R-CURATOR-3 to R-CURATOR-6 from the fields of Stophammer ADR 0102:

- R-CURATOR-3: a publisher page has a "Playlists" section after its release sections.
- R-CURATOR-4: a playlist page has a "Curator" row.
- R-CURATOR-5: the Public playlists list has one group for each curator feed, then "Other playlists".
- R-CURATOR-6: the app reads the fields of Stophammer ADR 0102.

Each new part shows only when `GET /v1/node/capabilities` names `playlist_links` (Stophammer ADR 0102 §7). A node without the capability shows each page as before.

The orchestrator sent this task on 2026-11-17. It answers HitIt requests 16 and 17. Stophammer ADR 0102 is Accepted (stophammer `7ca65ca`), and §5a is Accepted (stophammer `c3e028b`). The code is on stophammer `main` (tasks 001 to 003 in `b231ec1`, task 004 in `b4f4f1d`). No Stophammer release holds it.

The record of the Public playlists section is ADR 0098 Decisions 1 and 4. ADR 0081 names HitIt ADR 0089, but ADR 0098 supersedes ADR 0089.

## Facts

- Stophammer gives `confirmed_publisher` and `unconfirmed_publisher` on each `musicL` row of `GET /v1/feeds/recent` and `GET /v1/feeds/by-name`, with no include. `confirmed_publisher` has `feed_guid`, `title`, `role` and `role_agreement`. `role_agreement` can be null. `unconfirmed_publisher` has `feed_guid` and `title`. At most one of the two is not null.
- `role` is the sorted role set joined with ", " (`normalize_rel` in stophammer `src/query.rs`).
- `GET /v1/feeds/playlist-publishers` gives the feed rows of `/v1/feeds/recent` with the link facts (`two_way_link_count`, `two_way_playlist_link_count`, `agreed_roles`) and `newest_playlist_at`. It takes `limit` (200 or less) and `cursor`.
- On a publisher feed, `include=publisher` gives one row with the direction `publisher_to_playlist` for each `musicL` item.
- A confirmed playlist link is a row with `two_way_validated` true and a `role_agreement` that is not `conflict` (Stophammer ADR 0102 §2). `one_side` and null count as confirmed.
- `agreed_roles` holds the roles of the release links only (Stophammer ADR 0102 §3).
- HitIt reads the capabilities one time in `FetchNodeSorts` (`src/application/queries/node_sorts.rs`) into `NodeSorts` (`src/view_models/list_page.rs`).

## Orchestrator Choices

Each choice is from 2026-11-17.

1. Publisher page sections. ADR 0098 Decision 11 is the HitIt form of R-CURATOR-3 item 6.

   A playlist link with `role_agreement` `conflict` goes into "Role not confirmed". A one-way playlist link goes into "Listed by <feed title> only". Each of these two sections shows the playlists with the releases. A confirmed playlist goes into a new "Playlists" section after the release sections (R-CURATOR-3 item 4). The type label shows "Curator" only when a confirmed link agrees on curator (item 5).
2. The Curator row of a playlist page shows only when the node names `playlist_links`. Without the capability, the page has no row, as before. With the capability, a confirmed link opens the feed, with its title and the dot of its first role. In each other condition, the row shows the author text, with "(unverified link)" for a one-way link (R-CURATOR-4 item 2).
3. The groups of Public playlists apply in the Rows view and in the Tiles view. Each group has its heading, then its rows or its tiles. Each row and each tile keeps its look (R-CURATOR-5 item 5).
4. No new HitIt ADR. ADR 0098 gets the reference to musicindex ADR 0081 in its "Revised:" line of 2026-11-17. That line already names ADR 0081 for the role curator, so the record keeps three change lines.

Orchestrator choice of 2026-11-17: `tests/fixtures/musicindex-openapi-0.21.0.json` holds the contract of Stophammer main at `b4f4f1d` (ADR 0102 tasks 001 to 004). It is version 0.21.0 from a local debug build, with no release tag. The guard of ADR 0075 section 6 needs the ADR 0102 fields, and no release holds them.

## Open Items

- Replace `tests/fixtures/musicindex-openapi-0.21.0.json` with the contract of the first Stophammer release candidate that holds ADR 0102. Then run the guard again. The orchestrator sends the candidate hash.
- A playlist row of a feed page opens with a click only. The keyboard selection of a feed page holds release keys only (`feed_page_release_rows` in `src/view_models/detail_selection.rs`). The arrow keys skip a playlist row, and Enter does not open it. A subsequent task adds a playlist row kind to that selection, with no release command.

Main session note of 2026-11-17: the Playlists section of a feed page and the "N playlists" text do not examine the capability. They read the fields of Stophammer ADR 0102. Only a node with ADR 0102 sends `publisher_to_playlist` rows and `two_way_playlist_link_count`. A node without the capability thus shows each page as before.

## Main Session Choices

These follow the website task `../musicindex/docs/tasks/site-curator-task-001-playlist-links.md`.

1. The group order with Newest follows the sequence of the rows. The group with the newest playlist thus comes first. With A to Z, the groups follow the feed titles. "Other playlists" is always last. A reversed sort reverses the groups and keeps "Other playlists" last.
2. The heading label of a group comes from two sources. They are `agreed_roles` of the row of `/v1/feeds/playlist-publishers`, and the agreed roles of the confirmed links in the group. The app reads at most 5 pages of 200 rows of that route.
3. The groups come from the pages that the list has read. After "Load more", a group can get more rows, and a heading can get one more role.
4. A search, Artists or Labels row can have `two_way_link_count` 0 and `two_way_playlist_link_count` 1 or more. Such a row shows "N playlists" in place of the release count. This is gap 3 of the website task.
5. The dot of the Curator row comes from the role of the confirmed link. It also reads `agreed_roles` of the row of `/v1/feeds/playlist-publishers` when the app holds it.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Capability read | extend | src/application/queries/node_sorts.rs | `FetchNodeSorts` reads the capabilities. `NodeSorts` gets the flag `playlist_links`. |
| Capability flags | extend | src/view_models/list_page.rs | `NodeSorts::from_capabilities` sets each flag. |
| Feed row decode | extend | src/api.rs | `Feed` gets `confirmed_publisher`, `unconfirmed_publisher` and `two_way_playlist_link_count`. |
| Publisher relation decode | reuse | src/api.rs | `PublisherRelationship` decodes each direction as text. |
| Request profiles | extend | src/application/request_profiles.rs | A new path shape and profile for `/v1/feeds/playlist-publishers`. The playlist detail adds `publisher`. |
| Cursor paging | reuse | src/view_models/index_name_list.rs | `IndexNameListBatch` holds one page of rows. |
| Public playlist list | extend | src/view_models/public_playlists.rs | The list items get group headings. |
| Group heading | extend | src/view_models/artist_groups.rs | `ArtistGroupHeadingVm` is the heading pattern of the Artists list. |
| Group heading shell | reuse | src/ui/shells/library/artist_groups.rs | `render_artist_group_heading` draws a heading. |
| Publisher page sections | extend | src/view_models/publisher_page.rs | `PublisherPageVm::sections` gets the Playlists section and the playlist rows of the two other sections. |
| Publisher page facts | extend | src/application/queries/feed.rs | `publisher_page_facts_from_feed` also reads `publisher_to_playlist`. |
| Named feed row | reuse | src/ui/shells/entity.rs | `render_named_feed_rows` draws the Curator row of a playlist page. |
| Search row text | extend | src/application/queries/search.rs | `index_feed_page_result_display` gives the playlist count. |
| Fixed API server | reuse | src/application/queries/name_lists.rs | `serve_routes` gives fixed JSON for each path in a test. |

## Phase A: The Data

Files to change: `src/api.rs`, `src/application/request_profiles.rs`, `src/application/queries/node_sorts.rs`, `src/view_models/list_page.rs`, `src/application/queries/public_playlists.rs`.

1. Add `NodeSorts::playlist_links`, set from the capability name `playlist_links`. Update the doc comments: the struct now holds each capability flag that the app reads.
2. Add to `Feed`: `confirmed_publisher: Option<ConfirmedPublisher>`, `unconfirmed_publisher: Option<UnconfirmedPublisher>`, `two_way_playlist_link_count: Option<i64>`, `two_way_link_count: Option<i64>` (if not present), `agreed_roles: Option<Vec<String>>` (if not present) and `newest_playlist_at: Option<i64>`. Each new field has `#[serde(skip_serializing_if = "Option::is_none")]` and a doc comment that names Stophammer ADR 0102 and its section.
3. Add `ConfirmedPublisher { feed_guid: String, title: Option<String>, role: Option<String>, role_agreement: Option<String> }` and `UnconfirmedPublisher { feed_guid: String, title: Option<String> }`.
4. Add `two_way_playlist_link_count: Option<i64>` to `SearchResult` and `ArtistNameRow`.
5. Add `RequestPathShape::PlaylistPublisherList` and the profile `INDEX_PLAYLIST_PUBLISHERS` with no include. Add `Client::fetch_playlist_publishers(limit, cursor) -> Result<PlaylistPublishersResponse>` with `data: Vec<Feed>` and `pagination: Pagination`.
6. Add the constant `PUBLISHER_TO_PLAYLIST_DIRECTION = "publisher_to_playlist"` with the other direction texts, if they are constants.
7. Change `INDEX_PUBLIC_PLAYLIST_DETAIL` to the include `remote_items,publisher`.
8. Add `FetchPlaylistPublishers` in `src/application/queries/public_playlists.rs`. It reads at most 5 pages of 200 rows and stops when `has_more` is false. It gives `Vec<Feed>`.

Mechanical criteria of Phase A:

- A decode test reads a recorded `musicL` row with `confirmed_publisher` and one with `unconfirmed_publisher`, and a row of an older node with no field.
- A decode test reads a recorded row of `/v1/feeds/playlist-publishers`.
- A test proves that `NodeSorts::from_capabilities` sets `playlist_links`.
- A test with `serve_routes` proves that `FetchPlaylistPublishers` follows the cursor and stops after 5 pages.

## Phase B: The Pages

The main session writes Phase B after it reads the Phase A diff.

1. Publisher page: the facts read the `publisher_to_playlist` rows. `sections` puts each confirmed playlist in "Playlists" after the release sections. A conflict goes into "Role not confirmed", and a one-way link goes into "Listed by <feed title> only". The type label counts the roles of the confirmed playlist links only with `role_agreement` `both`. The release counts and the artist counts do not change.
2. Playlist page: the Curator row of choice 2.
3. Public playlists: the groups of R-CURATOR-5, in both views. A row with `unconfirmed_publisher` in "Other playlists" ends with "(unverified link)".
4. Search, Artists and Labels rows: main session choice 4.

Mechanical criteria of Phase B:

- Without `playlist_links`, each view model gives the same output as before.
- A feed with only playlist links: the publisher page view model has the label "Curator" and a "Playlists" section. It has no release section and no release count.
- The publisher page view model of a label feed with one release and one playlist has "Label · Curator" only when the playlist link agrees on curator. The release sections come before "Playlists".
- The Curator row view model gives the feed link, the author text, or the author text with "(unverified link)".
- The Public playlists view model gives the groups in the sequence of choice 1, "Other playlists" last, and the note only on the rows with `unconfirmed_publisher`.

## Live Check

The node of the Stophammer ADR 0102 test set must start in the same command as the check:

```sh
NO_PROXY=127.0.0.1,localhost ../musicindex/scratch/curator/real-node.sh COMMAND
```

The node listens on `127.0.0.1:18102`. Expected results:

- ChadF has the label "Curator" and a Playlists section with IAM and ITDV.
- Sir Libre Records has the label "Label · Curator". Its Releases section comes before its Playlists section.
- The Curator rows of IAM and ITDV open ChadF. The Curator row of the one-way playlist ends with "(unverified link)".
- Public playlists has the groups of ChadF and Sir Libre Records, then "Other playlists" with the one-way playlist and its note.

The screenshots are in the dark, light and high contrast themes.

## Visual Criteria

The operator judges these on a node with `playlist_links`:

- The curator dot and each group heading are clear in each theme.
- The groups in the Tiles view are clear.

## Test Commands

```bash
cargo test --lib curator
cargo test --lib public_playlist
cargo test --lib publisher_page
```

## Do Not Commit

A delegate does not commit. The main session reads the diff, runs the gate and commits.
