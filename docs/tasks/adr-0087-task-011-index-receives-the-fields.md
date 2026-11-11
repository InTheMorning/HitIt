# ADR 0087 Task 011: The Index Receives Each Field As Its Own Filter

Status: Implemented - 2026-11-06. The operator walked the visual checks on 2026-11-06, and they pass.

Depends on: [packet 002](adr-0087-task-002-library-fields-and-index-words.md). Stophammer release 0.15.0 of 2026-10-25 delivered the filters of [open request 10](../plans/hitit-open-requests.md#10-search-accepts-words-phrases-and-field-filters). This packet replaces packet 006 of the [phase plan](../plans/adr-0087-search-fields-phase-plan.md).

## Problem

A search with a field never asked the Index. `IndexSearchPlan::for_query` returned `Skip(Unsupported)` for each correct field other than `in:`. So `title:hey` listed only Library rows. The live Index gives 24 rows for `GET /v1/search?title=hey`. The packet sends the fields as filters, as ADR 0087 Decision 5 says.

## Authority

- [ADR 0087](../adr/0087-search-fields-and-live-results.md) Decision 5: "HitIt sends the words and the phrases in `q`, and each Index field as its own parameter." Its invariants "No Index request carries field syntax in `q`" and "Index rows show only when each Index field of the query is applied by Stophammer".
- ADR 0087 Decision 4: each run sends at most three Index requests.
- [ADR 0075](../adr/0075-metadata-ownership-and-completeness.md) section 6: the search sends no detail request for a row. A request uses its named profile.
- The stored contract `tests/fixtures/musicindex-openapi-0.18.1.json`, `GET /v1/search`: the parameters, the types that accept each one, and the `400` codes.
- [AGENTS.md](../../AGENTS.md): "Every fix gets a guard", "An acceptance criterion states how it is checked" and "Write reports for the operator".

## Recorded Facts - 2026-11-06

A read-only check of `https://api.musicindex.org/v1/search` gave these results on 2026-11-06. Each call was a `GET`.

| Check | Result |
|---|---|
| `title=hey&type=track` | 200. All 10 rows of the sample have `title`, `feed_title` and an image URL. Each title has a word that starts with "hey", for example "QT - hey_QT". |
| `title` and the release title of a track | `title=assorted&type=track` gives no row. `release=assorted&type=track` gives 7 rows of the feed "Various & Assorted". The `title` filter of a track matches the track title only. |
| `title` on a feed | `title=assorted&type=feed` gives the feed "Various & Assorted". |
| `kind=single` with `unknown` | `kind=single&type=feed` gives 0 rows, and `kind=single&type=track` gives 0 rows. `kind=unknown&type=feed` gives 20 rows, and the feed detail of a feed with one track states `release_kind` `unknown`. |
| `kind=album` and `kind=ep` | `kind=album&type=feed` and `kind=ep&type=feed` give 0 rows. |
| The date form of `after` and `before` | Only `YYYY-MM-DD` gives 200. `2026-10`, `2026-10-01T00:00:00Z` and `2026/10/01` give `400` with `invalid_filter`. A day is included from 00:00:00 to 23:59:59 UTC. `after=2026-10-31&before=2026-10-31` gives track rows whose `pub_date` lies on that day. |
| The unit of `min_duration` | Integer seconds. `min_duration=300&max_duration=400` gives 20 tracks, from 326 to 393 seconds. `min_duration=254&max_duration=254` gives the track of 254 seconds, so each limit is included. `min_duration=5m` gives `400` with `invalid_filter`. |
| `not_title` | `title=hey&type=track` with `not_title=listen` gives no track whose title holds "Listen". `not_q=listen` and `not_artist=matt` also remove the matching rows. |
| A filter with no `q` | `kind=single&type=feed` gives 200 with 0 rows. `after=2026-10-01&type=track` gives 200 with rows. `type=feed` alone gives `400` with `missing_positive_term`. `not_title=hey` alone gives `400` with `missing_positive_term`. |
| A type that does not accept a filter | `type=publisher&release=x`, `type=publisher&kind=single`, `type=publisher&after=2026-10-01`, `type=feed&min_duration=300` and `type=publisher&min_duration=300` each give `400` with `unsupported_filter`. |
| The `include` of a row | The rows of a filtered search carry the summary fields that the renderer needs. The packet keeps the current requests, which send no `include` for a feed or a track. |

## Mapping

| Query field | Index parameter | Types that accept it |
|---|---|---|
| words and phrases | `q` | all |
| `artist:` | `artist` | feed, track, publisher |
| `title:` | `title` | feed, track, publisher |
| `release:` | `release` | feed, track |
| `kind:` | `kind` (the kinds of one term are an OR) | feed, track |
| `after:` | `after` (`YYYY-MM-DD`) | feed, track |
| `before:` | `before` (`YYYY-MM-DD`) | feed, track |
| `duration:` | `min_duration` and `max_duration` (integer seconds) | track |
| `-artist:` | `not_artist` | feed, track, publisher |
| `-title:` | `not_title` | feed, track, publisher |
| `-release:` | `not_release` | feed, track |
| `-word` and `-"phrase"` | `not_q` | feed, track, publisher |
| `-kind:`, `-after:`, `-before:`, `-duration:` | none. The contract has no `not_` form. The part is `Skip(Unsupported)` with the note | none |
| `is:`, `playlist:`, `has:` | none. The part is `Skip(LibraryOnly)` | none |
| `in:` | none. It sets the scope | none |

A type that cannot accept a filter of a query gets no request. For example, `kind:single` sends a feed request and a track request, and no publisher request. A duration sends a track request only.

## Orchestrator Decisions

- **D1. The plan keeps one owner.** `IndexSearchPlan::for_query` builds the request, with `q` and the typed filters. `IndexSearchRequest::for_entity` gives the request of each entity type. No other code builds a request.
- **D2. The request has no field syntax.** `q` holds the words and the phrases. Each field is a filter value. A text filter holds words and phrases in the form of `q`, and a value with a space or a quote is quoted.
- **D3. Two values of one text field join with a space.** The contract allows one value for `artist`, `title` and `release`. Two words of the same field must both apply, so a space gives the same result as two terms.
- **D4. Two `kind:` terms keep the kinds that both have.** `kind:album,ep kind:ep` gives `ep`. Two terms with no shared kind give no Index request, and no request can match them.
- **D5. Two `after:` terms keep the later day, and two `before:` terms keep the earlier day.** Each `duration:` term tightens the range.
- **D6. A query with exclusions only sends no request.** Stophammer answers `400` with `missing_positive_term`. The part gives `Skip(NoIndexPart)`, and no note shows.
- **D7. The `kind` values are sent as they are.** The client does not send `unknown` or a track count. The stored rule of musicindex ADR 0020 R-WORD-1 needs two requests, and the run allows three. Open request 14 proposes one request for it. The operator decides when that request goes to Stophammer.
- **D8. The artist rows need a name term.** A name term is the `q` of the request. Without `q`, it is the words of the `artist:` filter. A request with neither gives no artist row. Without this rule, each artist of each result would become a row.
- **D9. The type split stays.** The run still sends feed, track and publisher requests, and it skips a type only when the type cannot accept a filter.
- **D10. The Index row path does not change.** The rows merge with the Library rows by the existing keys (ADR 0084 packet 010). Live results and saved searches use the same plan, so they get the same behavior. The new-result count of a saved search now counts the Index rows of a field query.

## Required Changes

1. `src/view_models/search_query.rs`: remove `index_filter_terms`, which no code uses now. Make `quote_phrase` `pub(crate)`.
2. `src/view_models/search_results/search_run.rs`: `IndexSearchRequest` holds `q: Option<String>` and `filters: Box<IndexFilters>`. Add `IndexFilters`, `IndexEntity` and `IndexSearchRequest::for_entity`. `UnsupportedIndexTerms` keeps only the excluded `kind:`, `after:`, `before:` and `duration:` tokens. Its note says "cannot exclude".
3. `src/api.rs` `Client::search`: take `query: Option<&str>` and the filter pairs. Send `q` only when it is `Some`. The pairs go through `build_url`, so the existing sanitizer applies.
4. `src/application/queries/search.rs`: `FetchIndexSearchResults` sends the request of each entity type. `index_filter_pairs` gives the pairs. `name_match_terms` gives the artist name terms. An entity type with no request gives an empty result, not an error.
5. `src/view_models/search_results/mod.rs`: re-export `IndexEntity` and `IndexFilters`.
6. Change the tests named in "Tests Changed" and add the tests of "Mechanical Acceptance Criteria".
7. The architecture guard `adr_0048_index_search_is_async_and_vm_owned` still requires the literal `type` values in `search.rs`. Keep them.

## Mechanical Acceptance Criteria

Each criterion is mechanical. A unit test proves it at the layer that owns it. Each test name starts with `adr_0087_index_fields_r87_11_`.

| Case | Owner | Required proof |
|---|---|---|
| R87-11-01 | `search_run.rs` | `artist:`, `title:`, `release:`, `kind:`, `after:`, `before:` and `duration:` give their `IndexFilters` values, and the words stay in `q` |
| R87-11-02 | `search_run.rs` | A query with fields only gives `q` `None`. A month gives its first day in `after` and its last day in `before` |
| R87-11-03 | `search_run.rs` | Each excluded word or phrase gives `not_q`. `-artist:`, `-title:` and `-release:` give `not_artist`, `not_title` and `not_release` |
| R87-11-04 | `search_run.rs` | The month of `before:` is the last day, and a leap year keeps 29 February. The duration forms give whole seconds, each limit included |
| R87-11-05 | `search_run.rs` | `in:` gives no filter. `is:` and `in:library` with a field give `Skip(LibraryOnly)` |
| R87-11-06 | `search_run.rs` | Repeated fields combine: the joined text, the later `after`, the earlier `before`, the common duration range, the shared kinds. No shared kind gives `Skip(NoIndexPart)` |
| R87-11-07 | `search_run.rs` | Exclusions only give `Skip(NoIndexPart)`. A word with an exclusion gives `Send` |
| R87-11-08 | `search_run.rs` | `IndexSearchRequest::for_entity` gives no publisher request for `release`, `kind`, a date or a duration. It gives no feed request for a duration. It gives each track request, and no request with no positive term |
| R87-11-09 | `search_run.rs` | An excluded `kind:`, `after:`, `before:` or `duration:` gives `Skip(Unsupported)` with its token. An excluded value that is not correct gives `Send` |
| R87-11-10 | `search.rs` | `title:hey` sends `title=hey` to each of the three types, and no request holds `q` |
| R87-11-11 | `search.rs` | `kind:single after:2026-10` sends the feed and track requests only, each with `kind` and `after`, and no `before` |
| R87-11-12 | `search.rs` | `duration:>5m -title:remix` sends one track request with `min_duration=301` and `not_title=remix`, and no `q` |
| R87-11-13 | `search.rs` | `haleen -remix` sends `q=haleen` and `not_q=remix` to each type |
| R87-11-14 | `search.rs` | A track row of a filtered search keeps its title and its track image, from the summary fields, with three requests |
| R87-11-15 | `search.rs` | A field query with no word and no `artist:` term gives no artist row |

The existing rule "No Index request carries field syntax in `q`" stays. Its test `adr_0087_library_fields_r87_2_23_index_request_holds_only_words` now reads the `q` field of the new struct.

## Tests Changed

- `adr_0087_library_fields_r87_2_22_excluded_terms_the_index_cannot_express` (`search_run.rs`) replaces `adr_0087_library_fields_r87_2_22_unsupported_terms_and_note_titles`. The old test stated that an included `after:` gives `Skip(Unsupported)`, and that a word with `-` gives "cannot exclude words". Packet 011 sends both to the Index. The new test states the excluded fields that stay unsupported.
- `adr_0087_library_fields_r87_2_23_send_holds_only_words` (`search_run.rs`) now builds the struct with `filters`. Its rule stays.
- `adr_0087_library_fields_r87_2_24_unsupported_skip_gives_the_note` (`view_models/search_results/tests.rs`) now uses `-after:` and `-kind:`, and the notes say "cannot exclude". The old text used an included field, which now gives a request.
- `adr_0087_query_parser_r87_1_19_index_filter_terms_give_fields_and_exclusions` (`search_query.rs`) is deleted with `index_filter_terms`. Its rule is in packet 011 R87-11-01 to R87-11-09 now.
- `adr_0087_live_results_r87_3_16_cancelled_run_sends_no_request` and `adr_0084_search_*` tests (`search.rs`) use `IndexSearchRequest::words` with the same requests and counts. Their rule does not change.

## Known Differences

The differences from the Library stay in ADR 0087 section "Known Differences From The Library". Packet 011 cannot close them, because each one needs a Stophammer change or a client rule that the run cannot send. Open request 14 asks for the Stophammer changes.

## Visual Acceptance Criteria

Each criterion is visual. Only a person can examine it in the running app. No test proves it.

- V1: type `title:hey` and wait for the pause. The Index rows of tracks show with their title and their art. The Library rows show as before.
- V2: type `kind:single` and wait for the pause. The page shows the Library rows and no error. No Index row shows, as the Index gives no single today (Known Differences 2).
- V3: type `haleen -kind:single` and wait for the pause. The page shows the note "MusicIndex cannot exclude kind: yet. The Library rows show." below the Library rows.
- V4: type `haleen after:2026-10` and wait for the pause. The result list changes in place. No row of a release date before 2026-10 shows in the Index rows of tracks.
- V5: type `duration:>5m` and wait for the pause. The Index shows track rows only, as Known Differences 4 says.

## Exclusions

- No change to the Stophammer repository. No new route.
- No client-side filter of a result. A row that the Index returns shows, and each row that a filter did not reduce does not show.
- No change to `docs/pending-human-checks.md` or the run record. The coordinator owns them.
- No change to the Library query builder, the field list, the keys or the saved search keys.
- No change to the stored contract. Its values already hold each parameter.

## Files To Inspect

- [ADR 0087](../adr/0087-search-fields-and-live-results.md) Decision 5, "Known Differences From The Library" and the invariants.
- `src/view_models/search_results/search_run.rs`, `src/view_models/search_query.rs`, `src/api.rs` `Client::search`, `src/application/queries/search.rs` `fetch_index_search_result_rows`.
- `src/runtime/saved_search_counts.rs`, `src/app/search_dispatch.rs` and `src/app/search_live.rs`, which send the plan. No change was needed there.
- `tests/fixtures/musicindex-openapi-0.18.1.json`, `GET /v1/search`.

## Checks

Run in this order in the worktree. Each must pass.

1. `cargo fmt`
2. `cargo clippy --all-targets -- -D warnings`
3. `cargo test`
4. `cargo test --test architecture_tests`
5. `cargo build --bin hitit`
