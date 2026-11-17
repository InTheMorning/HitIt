# musicindex ADR 0081 Task 003: The Not Indexed Playlist Page

Status: Done. The agent did the live check in the dark theme. Visual criterion V2 is open.

## Goal

A publisher page can show a playlist row that the index does not hold. An example is a row of "Listed by ChadF only". A click on the row opens the playlist page. Before this task, the page showed the raw reply "GET failed with HTTP 404 Not Found: {"error":"feed not found"}".

After this task, the page shows a "Not indexed" state when Music Index replies with HTTP `404`. The page does not show the raw HTTP text. The website does the same.

The orchestrator sent this task on 2026-11-17, after its check of `1014014`. The owner of the playlist page is ADR 0098 and musicindex ADR 0081 R-CURATOR-3.

## Orchestrator Choices

Each choice is from 2026-11-17.

1. A feed that the index does not hold shows a "Not indexed" state on the page. The page does not show the raw HTTP text.
2. If a simpler solution exists, the row does not open a page that fails. The report names the solution that the task used.

## Main Session Choices

1. The task uses the "Not indexed" state. HitIt cannot know before the read if the index holds a feed of a one-way row, because the row comes from the publisher feed. Thus a row that does not open needs a read for each row. The state on the page needs no extra read.
2. Only HTTP `404` gives the state. A connection error, a timeout or a `5xx` reply keeps the error state with "Try again".
3. The state has no "Try again" button. Another read gives the same reply.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| HTTP status error | extend | src/api.rs | `HttpStatusError` gets `is_not_found`, as `is_bad_request` for the search. |
| Playlist read command | extend | src/application/queries/public_playlists.rs | `FetchPublicPlaylist` gives no feed for a `404` reply. |
| Playlist page load state | extend | src/view_models/public_playlists.rs | `PublicPlaylistLoadState` gets `NotIndexed`. `PublicPlaylistPages::finish` takes an optional feed. |
| Playlist page render | extend | src/ui/shells/library/public_playlists.rs | A new render function for the state, with the loading and error states. |
| Test route server | extend | src/application/queries/name_lists.rs | `serve_routes` can send a `404` reply. |

## Changes

1. `src/api.rs`: `HttpStatusError::is_not_found` tells if the status is `404`.
2. `src/application/queries/public_playlists.rs`: `PublicPlaylistLoad.feed` is `Option<Feed>`. A `404` reply gives `None`. Each other error stays a `CommandError::Query`.
3. `src/view_models/public_playlists.rs`: `PublicPlaylistPages::finish` sets `Loaded` for a feed and `NotIndexed` for `None`.
4. `src/library/public_playlists.rs` and `src/ui/shells/library/public_playlists.rs`: the page renders `NotIndexed` as the text "Not indexed" and one line of help text.

## Mechanical Criteria

Each criterion is proved by a test.

- M1. The view model: `finish` with no feed gives the state `NotIndexed`. The state holds no error text, and `loaded_title` gives `None`.
- M2. The command: a `404` reply with `{"error":"feed not found"}` gives a load with no feed, not a `CommandError`.
- M3. The command: a `500` reply gives a `CommandError`, so the page keeps the error state.

## Visual Criteria

A person judges each of these criteria.

- V1. On the ChadF page, a click on the row "No title" (`0a28983e-c60c-467d-8d76-6ec7663fbfec`) opens a page that shows "Not indexed". The page shows no HTTP text and no "Try again" button.
- V2. The state looks correct in the light theme and in the high contrast theme.

## Test Commands

```bash
cargo fmt -- --check
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo test --test architecture_tests
cargo build --bin hitit
```

## Do Not Commit

A delegate does not commit. The main session reads the diff, runs the checks and makes the commit.
