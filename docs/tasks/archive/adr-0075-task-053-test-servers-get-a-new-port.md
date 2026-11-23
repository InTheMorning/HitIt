# ADR 0075 Task 053: Test Servers Get A New Port

## Status

Done - 2026-11-22. The main session ran the gate.

- Operator choice, 2026-11-22: correct the CI failure of
  `adr_0075_request_profile_library_track_detail_falls_back_to_unscoped_then_feed`.
- Agent choice, 2026-11-22: all test servers use the new helper, not only
  the servers that reach the shared request store. One rule for all test
  servers is easier to keep than a list of the servers that need it.

## Goal

A test does not get a stored response of an earlier test. Each test server
of one test process has its own endpoint.

## Authority

ADR 0075 packet 018 P18-6: the request store is one value for the full
process. The key of a stored response holds the endpoint. A track response
stays for 30 minutes and a feed response stays for 15 minutes.

## Recorded Facts

The CI run of 2026-11-22 failed one test of 3460. The server of the test
received two requests and not three:

- `/v1/feeds/f1/tracks/t1?include=...`
- `/v1/tracks/t1?include=...`

The third request, `/v1/feeds/f1?include=...` with the include list of
`LIBRARY_TRACK_DETAIL_FEED`, did not reach the server.

Cause:

1. Each test server binds `127.0.0.1:0`. When a test stops its server, the
   system can give the same port to a subsequent test.
2. Tests in `src/application/queries/library.rs` and `src/feed_service.rs`
   send requests for the feed "f1" and the track "t1" with the same include
   lists.
3. A test of `library.rs` lets the two track requests fail and the feed
   request pass. The store keeps the feed response and no failure.
4. The failed test then got the same port. Its two track requests reached
   its server. The store gave the feed response of the earlier test, so
   the feed request did not reach the server.

The test passed 15 times alone before this change. The fixture comment said
that a fresh port makes each key different. That is not true when the
system gives the same port again.

## Change

1. New `src/test_ports.rs` (test builds only):
   `fresh_local_listener()` binds `127.0.0.1:0` again until the port is
   one that it did not give before in this process.
2. Each `TcpListener::bind("127.0.0.1:0")` of a test uses
   `crate::test_ports::fresh_local_listener()`.
3. The fixture comment in `src/feed_service.rs` names the helper.

## Do Not Change

- The request store and its windows. The production code does not change.

## Base components

| Component | Status | Location | Reason |
| --- | --- | --- | --- |
| Local test listener | new | src/test_ports.rs | Searched src for `TcpListener::bind` and the `test_support` modules: no helper exists, and each test made its own listener. Not searched: ~/build/musicindex-lib, because the agent does not read other suite repositories. The helper keeps the state of one HitIt test process, so it is local to HitIt. |
| Request store | reuse | src/application/request_reuse.rs | No change. |

## Acceptance Criteria

Mechanical:

1. `fresh_local_listener` gives 200 different ports when each listener is
   closed before the next call. Test
   `test_ports::tests::fresh_local_listener_never_gives_a_port_again`.
2. No test calls `TcpListener::bind("127.0.0.1:0")` directly. Manual check
   with `grep`, because the failure occurred one time only. The helper
   is the only call site.
3. Each existing test passes.

Visual: none.

## Checks

- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo build --bin hitit`
