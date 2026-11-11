# ADR 0084 Task 038: A Downloaded Single Joins The Group Of Its Feed

Status: Implemented - 2026-10-25. The visual checks V1 to V5 stay open in [pending human checks](../pending-human-checks.md#8-packets-038-and-039---implemented-2026-10-25). The operator answered Q1 on 2026-10-24.

Depends on: [packet 035](adr-0084-task-035-download-stores-feed-links.md). This packet fixes the defect of packet 035 V1. The walk of this packet walks packet 035 V1 to V3 again.

## Goal

The Library tree gives one release for each Library feed. Two feeds with the same artist text and the same title stay two releases. A downloaded `MusicIndex` single then shows in the Library state of its group row, and the group of the other feed keeps its singles.

## Authority

- [ADR 0084](../adr/0084-music-column-browser.md) Decision 4: the identity of a release is its feed GUID. Decision 5: the kind of a release. Decision 6: the singles groups.
- [Packet 035](adr-0084-task-035-download-stores-feed-links.md) V1, which failed on 2026-10-24.
- [Packet 026](adr-0084-task-026-index-singles-groups.md) decision D8: the Library decides a Library release.
- [Archived packet 036](archive/adr-0084-task-036-group-row-at-artist-name.md): the place of a group row.
- [AGENTS.md](../../AGENTS.md): "Every fix gets a guard" and "Current-view state must update in place".

## Recorded Facts - 2026-10-24

### The defect

The operator found it on 2026-10-24, in the walk of packet 035 V1.

- In Releases, scope "Everything", two "Haleen" groups show at the level "Artist feed":
  - The group of the artist feed `64569161-a8f7-52e5-8aae-9e0385043149` holds Library singles. It showed "56 singles" and the "in Library" icon.
  - The group of the artist feed `fe80a515-2826-4a46-aa37-b27f38d3c30f` holds `MusicIndex` singles that are not in the Library. It showed "38 singles".
- The operator downloaded the group `fe80a515`. 20 downloads completed. 18 downloads failed with HTTP 429. [Packet 039, the host limit](adr-0084-task-039-download-queue-host-limit.md), owns the HTTP 429 failures.
- The Library now holds the feeds with id 109 to 128. Titles include "Sense Of Awe", "Gentle Breeze", "Breathing", "Shanti", "The Sacred Temple", "Silent Upgrade" and "Adi Mantra".
- Each of these feeds has a `feed_publisher_relationships` row with the direction `music_to_publisher`, the `remote_feed_guid` `fe80a515…` and the `publisher_feed_title` "Haleen".
- The Library held each of these titles before, in a different feed with a different feed GUID. These older feeds link to `64569161`.
- After the download, the group `fe80a515` still shows "38 singles". Each of its rows keeps the "not downloaded" icon, in the list and on the group page.
- The group `64569161` changed from 56 to 36 singles.
- A search for "haleen d" shows, for each title, one row "Album · Haleen · 1 track" and one row "Single · Haleen · 1 track". The operator recorded the "Album" rows as the new feeds.

### Code facts: the Library tree joins two feeds into one release

The code facts come from the working tree of 2026-10-24, at commit `78afacc`. Find each call site again before you change it.

1. `src/db.rs:536-548` `library_tracks` reads each Library track in the order `f.title COLLATE NOCASE, t.track_number, t.track_title COLLATE NOCASE`. `LoadLibraryTracksTree` (`src/application/queries/library.rs:83-86`) gives these rows to `build_tree`.
2. `src/application/queries/library.rs:372-383` `build_tree` puts each track in a map with two keys: `display_artist` and `display_album`. `display_album` is the `album_title` of the track, else its `feed_title` (`src/view_models/library.rs:5682-5688`). The Library feed id is not part of the key.
3. Thus two Library feeds give one `AlbumNode` when their tracks have the same artist text and the same title text. The node holds the tracks of the two feeds.
4. `src/application/queries/library.rs:399-401` sorts the tracks of the node by `track_number`. The node takes `feed_id` and `feed_guid` from the first track only. `named_feeds` (`:443-447`), `metadata_facts` and `stored_values` come from that one feed id.
5. `AlbumNode::release_kind` (`src/view_models/library.rs:6155-6160`) calls `ReleaseKind::resolve` with `self.tracks.len()`. A node with the tracks of two singles has two tracks. Without a stated kind, `ReleaseKind::resolve` (`src/view_models/release_kind.rs:32-41`) gives `Album` for two tracks.
6. `AlbumNode::release_key` (`src/view_models/library.rs:6137-6142`) gives `ReleaseKey::FeedGuid` of the first track. The feed GUID of the other feed is the key of no node.

### How the joined node gives each observation

These facts follow from the code facts. The order of the two tracks in a node is not proved (see "Facts that this packet does not prove").

- **56 to 36.** `library_singles_grouped` (`src/view_models/singles_group.rs:378-415`) skips each node that is not a single (line 384). Each joined node is an Album. Thus the 20 older singles leave the group `64569161`. `SinglesGroup::count` (`:202-204`) is the sum of the Library singles, the `MusicIndex` singles and the matched singles, so the count falls by 20.
- **The 20 older singles do not come back as `MusicIndex` members.** `everything_groups` (`src/view_models/singles_group.rs:528-533`) collects the release key of each tree node, also of an Album node. The joined node has the key of the older feed. Line 546 thus keeps the `MusicIndex` row of the older single out of the group.
- **38 stays 38.** No node has the key `ReleaseKey::FeedGuid` of a new feed. Line 546 thus keeps each of the 38 `MusicIndex` rows of the group `fe80a515` as a member that is not in the Library. `in_library_count` (`:242-249`) counts no new single, so each row keeps the "not downloaded" icon.
- **The group page shows the same.** `SinglesGroupPageVm::from_tree` reads the same `everything_groups`, so the page and the list agree.
- **"Album" and "Single" in search.** `LocalFeedResult` (`src/view_models/search_results/results.rs:605-648`) makes one row for each Library feed, with the key of that feed and one track. `release_row_vm` (`src/view_models/search_results/rows.rs:222-223`) takes the kind of the tree node when `album_for_release_key` (`src/view_models/library.rs:3969-3976`) finds a node with that key. The feed whose key the joined node has shows "Album". The other feed finds no node and shows "Single", from `ReleaseKind::resolve(None, 1)`.
- **The feed links are correct.** Packet 035 works. The rows of `feed_publisher_relationships` for the feeds 109 to 128 exist. But a node reads `named_feeds` of its first feed only, so the link of the other feed has no effect.

### Facts that this packet does not prove

- **The order of the two tracks in a joined node.** The code gives this result: when the feeds 109 to 128 store the `MusicIndex` feed GUID, the joined node has the key of the older feed. With the key of the new feed, line 546 removes that row from the group `fe80a515`. That group then shows 18, not 38.
- **Evidence for the stored GUID.** `hydrate_album_identity_by_feed_guid` (`src/library/app_impl.rs:2443-2461`) finds the new feed id by the `MusicIndex` feed GUID (`db::find_feed_id_by_guid`). The rows of the new feeds exist. Thus the feeds 109 to 128 very probably store the `MusicIndex` GUID. The RSS check (`src/rss/check_apply.rs`) can also write such a row, so this is not a proof.
- **The search observation.** With the key of the older feed, the "Album" row is the older feed and the "Single" row is the new feed. That is the opposite of the operator record. A search row does not show its feed id, so the record can be wrong. Step 2 of the operator check tells the two cases apart.
- If a new feed stores no feed GUID or a different GUID, the fix of this packet is not sufficient. The group `fe80a515` would then count the single two times: one time as a Library single and one time as a `MusicIndex` row. Step 2 of the operator check finds this case. Then the orchestrator writes a separate packet.

## Design

1. In `build_tree` (`src/application/queries/library.rs:372-383`), the album key of each artist is the pair of `display_album` and the Library feed id of the track. Two feeds never share a node. The map keeps the order of the album text, and the feed id gives the order of two feeds with the same text.
2. The key is the album text and the feed id (Q1, answer A). One feed with two album texts still gives two nodes, as before this packet.
3. No other production code changes. With one node for each feed, the code facts above give the correct result:
   - Each new feed is its own node with one track, so it is a single.
   - Its `named_feeds` name `fe80a515`, so it joins the group `fe80a515` as a Library single.
   - Its key is its feed GUID, so line 546 removes its `MusicIndex` row from the members.
   - Each older feed is its own single again, in the group `64569161`.
4. Change the doc comment of `build_tree`. It states that a node holds the tracks of one Library feed (ADR 0084 Decision 4) and names this packet.

### Consequences that follow from ADR 0084 Decision 6

These are correct results of the present rules. They are not defects of this packet.

- The Library then holds two artist feeds with the title "Haleen". `tree_name_feed_matches` (`src/view_models/library.rs:2371-2398`) thus matches the name "haleen" to two feeds. A Library single with the name level "Haleen" then leaves "Matched by name" of the group `64569161`. It shows in a group "By name", or as its own release row when it is alone.
- The scope "Library" shows each title of the two feeds two times: one time in each group.
- The Artists list shows two feed rows "Haleen" (ADR 0085).

## Decisions

- **D1.** A Library tree node holds the tracks of one Library feed. Reason: ADR 0084 Decision 4 gives each release the identity of its feed. A node with two feeds has two identities and one key.
- **D2.** The order of the nodes of an artist stays the order of the album text. Reason: the Library list and the existing tests read that order.
- **D3.** The view models, the singles groups and the search rows do not change. Reason: they give the correct result from a correct tree (code facts above).

## Questions

- **Q1. Does one feed with two album texts give one node or two nodes?**

  Facts: today, one feed whose tracks state two `album_title` values gives two nodes with the same feed id and the same release key. The defect of this packet does not need a change of this case. No observation of this case is recorded.

  - Option A: the key is the album text and the feed id. Two feeds never share a node. One feed with two album texts gives two nodes, as today.
  - Option B: the key is the feed id only. One feed always gives one node. Its name is the album text of its first track. This also changes a feed that holds the tracks of more than one album.

  **Recommendation: Option A.** It fixes the defect and changes no other case. Option B agrees better with ADR 0084 Decision 4. But it changes a case that no observation and no test describes. A later packet can do it after an observation.

## Operator Answers - 2026-10-24

- Q1: A. Key each album by the album text plus the feed id. Two feeds never share a node. One feed with two album texts still gives two nodes.

## Required Changes

1. `src/application/queries/library.rs`: Design steps 1 and 4.
2. Tests: add each test of "Mechanical Acceptance Criteria". Keep each existing test. If an existing test fails, record its name and the reason in "Implementation Notes".
3. Documents: the orchestrator adds V1 to V5 to [pending human checks](../pending-human-checks.md) and changes the row of packet 035 in the [phase plan](../plans/adr-0084-music-column-browser-phase-plan.md). This packet does not change those files.

## Mechanical Acceptance Criteria

Each criterion here is mechanical. A unit test beside its owning code has the prefix `adr_0084_task_038_`. No test sends a request to `api.musicindex.org`. Each test uses an in-memory database with `db::init_schema`, as the `build_tree` tests in `src/library/app_impl.rs` do.

| Case | Required proof |
|---|---|
| R38-01 | Mechanical, unit test in `src/application/queries/library.rs`, fixture 1. Regression test: `adr_0084_task_038_two_feeds_with_one_title_stay_two_releases`. `build_tree` gives two album nodes, each with one track and one feed id. The release keys are `FeedGuid("g1")` and `FeedGuid("g2")`. `release_kind()` of each node is `ReleaseKind::Single` |
| R38-02 | Mechanical, unit test, same file, fixture 2. The guard of the class: `adr_0084_task_038_each_tree_node_holds_one_feed`. Each track of a node has the `feed_id` of that node. In one artist node, no two nodes have the same `feed_id` and the same name |
| R38-03 | Mechanical, unit test, same file, fixture 3. The defect of packet 035 V1: `adr_0084_task_038_downloaded_single_joins_its_feed_group`. `singles_groups_everything` gives the group `P2` with the Library single `g2` and the `MusicIndex` single `g4`. Its `count()` is 2 and its `in_library_count()` is 1. The group `P1` has the Library singles `g1` and `g3` |
| R38-04 | Mechanical, unit test, same file. With the tree of R38-03, the group `P2` holds no `MusicIndex` member with the key `FeedGuid("g2")` (packet 026 decision D8) |
| R38-05 | Mechanical. Each existing test passes: the `build_tree` tests, and the tests of packets 015, 026, 032, 035 and 036. `cargo test --test architecture_tests` passes |

The fixtures:

1. Fixture 1: two feeds, with the GUIDs `g1` and `g2`, have the title "Sense Of Awe". Each feed has one Library track with the artist text "Haleen".
2. Fixture 2: three shapes in one database. Two feeds have one title and one artist. Two other feeds have one title and two artists. One more feed has two tracks.
3. Fixture 3: feed 1 (`g1`) names the artist feed `P1`. Feed 2 (`g2`) has the title of feed 1 and names `P2`. Feed 3 (`g3`) names `P1`.
   - Each feed has a confirmed `music_to_publisher` row and one Library track with the artist text "Haleen".
   - `IndexFeedSingles` holds the `MusicIndex` singles `g2` and `g4` for `P2`.

The failure message of R38-01 and R38-02 has this form:

```text
ADR 0084 Decision 4, task 038: a Library tree node must hold the tracks of one
Library feed. Two feeds with one title gave one node, so a downloaded single
left its group. Key the album map of build_tree by the album text and the feed id.
```

R38-01 to R38-04 are situational. They cite ADR 0084 Decision 4 and this packet. Delete them when that decision is superseded.

A check with the code of today must make R38-01, R38-02 and R38-03 fail. Record that check in "Implementation Notes".

## Visual Acceptance Criteria

Each criterion here is visual. Only a person can examine it. No test proves it.

- V1: Releases, "Everything". The group row `fe80a515` shows its downloaded singles in its Library state, for example "20 of 38". The group row `64569161` shows about 56 singles again.
- V2: packet 035 V1 again. Select the group row `fe80a515` and click "Download". After each download, its row changes in place. The group row stays at the same position. Each new single stays in the group.
- V3: packet 035 V2 again. On the group page `fe80a515`, each downloaded single shows under the singles of the feed, not under "Matched by name".
- V4: packet 035 V3 again. Download one `MusicIndex` single from its Index release page. Open the group page of its artist feed with no other navigation. The single shows under the singles of the feed.
- V5: search for "haleen d". Each release row of the feeds 109 to 128 and of the older feeds shows "Single · Haleen · 1 track" and the "in Library" icon.

## Exclusions

- No change to the HTTP 429 failures. [Packet 039, the host limit](adr-0084-task-039-download-queue-host-limit.md), owns them.
- No change to `named_feeds`, the hydration or the feed links. Packet 035 owns them.
- No change to the artist key of `build_tree`. One feed whose tracks state two artist texts still gives one node for each artist text.
- No change to the case of one feed with two album texts. It still gives two nodes (Q1, answer A).
- No change to the singles group view models, the Releases list or the search rows (decision D3).
- No schema change and no migration.

## Files To Inspect

- [AGENTS.md](../../AGENTS.md) and the [source map](../architecture/source-map.md).
- `src/application/queries/library.rs` (`build_tree`, `LoadLibraryTracksTree`, `LoadCachedTracksTree`).
- `src/db.rs` (`library_tracks`, `cached_tracks`).
- `src/view_models/library.rs` (`AlbumNode`, `release_key`, `release_kind`, `display_album`, `tree_name_feed_matches`, `album_for_release_key`).
- `src/view_models/singles_group.rs` (`library_singles_grouped`, `everything_groups`, `SinglesGroup`).
- `src/view_models/release_kind.rs` (`ReleaseKind::resolve`).
- `src/library/app_impl.rs` (the `build_tree` tests and their fixture).

## Checks

```bash
cargo test
cargo test --test architecture_tests
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo check --all-targets
touch src/main.rs && cargo build --bin hitit
```

## Rollback

Revert the commit. This packet adds no migration, no stored data and no configuration key.

## Operator Visual Check

This check needs a Linux desktop session and a connection to `https://api.musicindex.org`. It needs the Library feeds 109 to 128 and the older Library feeds with the same titles. HTTP 429 failures can occur until the host limit packet is complete. A failure with HTTP 429 is not a defect of this packet.

1. Build and start the debug binary:

   ```bash
   cd ~/build/v4vmm
   cargo build --bin hitit
   nm -C target/debug/hitit | grep -c TestDispatcher
   ./target/debug/hitit
   ```

   The `nm` command must give `0`.

2. Fact check. In a second terminal, list the stored GUID of each new feed:

   ```bash
   sqlite3 -readonly ~/.local/share/hitit/hitit.sqlite "SELECT f.id, f.feed_guid, f.title, r.remote_feed_guid FROM feeds f LEFT JOIN feed_publisher_relationships r ON r.feed_id = f.id AND r.direction = 'music_to_publisher' WHERE f.id BETWEEN 109 AND 128 ORDER BY f.id;"
   ```

   **Wrong if:** a `feed_guid` is empty. Stop the check and record the row. The orchestrator then writes a separate packet.

3. List the two feeds of one title, in the order that the Library reads them:

   ```bash
   sqlite3 -readonly ~/.local/share/hitit/hitit.sqlite "SELECT f.id, f.feed_guid, t.id, t.track_number, t.album_title, t.album_artist_name, t.artist_name FROM tracks t JOIN feeds f ON f.id = t.feed_id WHERE t.is_in_library = 1 AND f.title = 'Sense Of Awe' ORDER BY f.title COLLATE NOCASE, t.track_number, t.track_title COLLATE NOCASE;"
   ```

   Record the rows. The query must give two feed ids.

4. V1: in Music, open Releases with the scope "Everything". Find the two "Haleen" group rows.
   **Wrong if:** the group `fe80a515` shows no download count, or the group `64569161` shows about 36 singles.

5. V2: click "Select". Select the group row `fe80a515` and click "Download".
   **Wrong if:** a row of a downloaded single does not change in place, the group row moves, or a downloaded single leaves the group.

6. V3: open the group page `fe80a515`.
   **Wrong if:** a downloaded single shows under "Matched by name", or shows the "not downloaded" icon.

7. V4: open the Index release page of one more Haleen single that is not in the Library. Click "Download". When it is complete, open the group page of its artist feed with no other navigation.
   **Wrong if:** the single shows under "Matched by name", or does not show.

8. V5: type "haleen d" in the toolbar search field.
   **Wrong if:** a release row of one track shows "Album", or a Library release shows the "not downloaded" icon.

A "By name" group "Haleen" can show after the fix. That is correct. The name "Haleen" then matches two artist feeds (ADR 0084 Decision 6).

Cleanup: steps 5 and 7 add singles to the Library. If the Library must stay as before the check, select those singles in Releases with the scope "Library" and remove them. The queries are read-only and change nothing.

## Implementation Notes - 2026-10-25

- Change: `build_tree` in `src/application/queries/library.rs` keys each album by the album text and the Library feed id (Design steps 1 and 4). Its doc comment states the rule and names this packet.
- Tests in `src/application/queries/library.rs`: `adr_0084_task_038_two_feeds_with_one_title_stay_two_releases` (R38-01), `adr_0084_task_038_each_tree_node_holds_one_feed` (R38-02), and `adr_0084_task_038_downloaded_single_joins_its_feed_group` (R38-03 and R38-04).
- Old-key run: with the album text as the only key, the three tests fail (`cargo test --lib adr_0084_task_038`: 3 of 3 tests did not pass). The new key is restored after that run.
- All existing tests are Green.
- Gate on 2026-10-25: `cargo fmt -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` are Green. The suite ran 2576 library tests and 397 architecture tests.
- Visual checks V1 to V5 stay open in [pending human checks](../pending-human-checks.md#8-packets-038-and-039---implemented-2026-10-25).
