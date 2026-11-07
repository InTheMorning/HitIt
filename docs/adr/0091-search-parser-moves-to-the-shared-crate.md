# ADR 0091: The Search Parser Moves To The Shared Crate

## Status

Accepted - 2026-11-07. The operator accepted this record with the answers below. It changes no code. The [search move plan](../../../musicindex-lib/docs/plans/search-move-phase-plan.md) in `../musicindex-lib` gives the sequence, and this record is its Phase 0 item for v4vmm.

It amends [ADR 0087](0087-search-fields-and-live-results.md): the invariant "One parser reads the query text" stays, and the owner of that parser changes at step 1 of the adoption. Until step 1, the code and the guards stay as they are.

## Context

musicindex ADR 0049 states that musicindex.org, Studio and v4vmm read one query text. Its rule R-QUERY-4 puts one Rust crate for v4vmm and Studio in `../musicindex-lib`. Its Consequences state that ADR 0087 needs an amendment, because the parser moves from the view model layer.

These facts are true on 2026-11-07:

- v4vmm has the parser in `src/view_models/search_query.rs`, the Index plan in `src/view_models/search_results/search_run.rs` and the kind rule in `src/view_models/release_kind.rs`.
- `../musicindex-lib/search` holds a copy of this code as the crate `musicindex-search`, from commit `52c6f54` of that repository. v4vmm does not use it yet.
- v4vmm sends `kind=single` for `kind:single`, and each feed in the backup of 2026-11-05 states `unknown`. So `kind:single` finds no Index rows.
- v4vmm writes `\"` for a quote in a phrase. Its sanitizer changes each backslash to a space (ADR 0087 Known Difference 5).

## Decision

1. The parser, the Index plan and the kind rule move to the crate `musicindex-search`. All of `src/view_models/release_kind.rs` moves: `resolve`, `label`, `noun`, `is_single` and `from_query_kind`. musicindex ADR 0049 owns the syntax. A change to the syntax needs a record in `../musicindex`, not in this repository.
2. ADR 0087 Decisions 1 and 2 stop owning the grammar and the field table, and they cite musicindex ADR 0049 R-QUERY-1. ADR 0087 keeps Decision 3, the field list, and Decision 4, live results. Decision 5 keeps its rule that the Index receives filters and never syntax, and it cites R-QUERY-1 item 4.
3. These parts stay in v4vmm: `with_library_scope`, `library_scope_of`, the Library queries of `src/db/library_search.rs`, the run generations, the field list, the page rows and the HTTP request.
4. The action labels stay in v4vmm, in `src/view_models/album_page.rs` and `src/view_models/release_row_actions.rs`. They are not in `release_kind.rs`.
5. `src/view_models/search_query.rs` keeps the two scope functions, and gives each other name from the crate. `src/view_models/release_kind.rs` stays as a re-export. Other files thus do not change at the move.
6. v4vmm uses a Git dependency on `musicindex-search`, pinned with `rev`, as `gpui-base` is. A path override for local work stays out of each commit.
7. v4vmm builds no copy of what the crate owns: no `kind:` mapping (R-QUERY-6), no change to the text of a word or a phrase (R-QUERY-5), and no sort request code for search, because the crate will have sort and reverse.
8. The note for a query with only exclusions shows when v4vmm adopts the crate. The crate gives this cause its own skip value. The operator approved the note on 2026-11-07.

## The Adoption Has Two Steps

1. **Pin `3ee38db`.** This is the move, with no change of behavior. Each v4vmm search test passes with no change to the test.
2. **Pin the commit with the corrections.** This step changes the tests and closes ADR 0087 Known Differences 2 and 5.

Just before step 1, run the diff from v4vmm commit `1d6dbb9` on the three search files and `src/application/queries/search.rs`. Report each change. Do not adopt until the crate has it.

## The Guards That Change

All are in `tests/architecture_tests.rs`. They change in the commit of step 1, as the governance rule for a change of direction requires.

| Guard | Today | After the move |
|---|---|---|
| `ADR_0087_QUERY_PARSER_OWNER`, used by `adr_0087_query_parser_r87_1_30_one_parser_reads_field_syntax` | Names `src/view_models/search_query.rs` as the one file that holds field syntax | No file under `src/` holds a field token as a literal. Its message names the crate and musicindex ADR 0049 as the owner |
| `RUN_OWNER`, used by `typed_search_path_violations` | Names `src/view_models/search_results/search_run.rs` as the one file that builds an `IndexSearchRequest` | No file under `src/` builds that request. Its message names the crate |
| `adr_0084_release_kind_r7_07_no_hardcoded_download_or_remove_album_literal` | Excludes `src/view_models/release_kind.rs` from a search for release-action text | It loses this exclusion |
| `adr_0084_singles_group_r15_09_only_the_owner_groups_singles` | Lists `src/view_models/release_kind.rs` as an owner of `ReleaseKind::Single` | It loses this file as an owner |

## Alternatives Considered

- **Keep the parser in v4vmm, and let the other apps copy it.** Three copies then drift, and the site is JavaScript. musicindex ADR 0049 rejects this.
- **Correct `kind:` in v4vmm now.** Stophammer gives the requests today, so this is possible. It makes a second owner of a rule that R-QUERY-6 gives to one place, and the crate work then replaces it.

## Consequences

- ADR 0087 Known Differences 2 and 5 close at step 2 of the adoption. `kind:ep` still sends no Index request, because a count never gives EP.
- Until then, `kind:single` and `kind:album` show no Index rows in v4vmm. This is a known state, not a defect to correct here.
- A change to `src/view_models/search_query.rs`, `src/view_models/search_results/search_run.rs` or `src/view_models/release_kind.rs` before step 1 makes the copy in the crate differ. The operator asks to know of such a change first.
- The crate builds with Rust 1.93, and v4vmm has 1.97. The crate thus cannot use newer language features.

## Invariants

- One parser reads the query text. No code in v4vmm reads field syntax.
- No Index request carries field syntax in `q`.
- A Library value goes into SQL only as a bound parameter.

## Non-Goals

- A change to the field list, the live results or the search page layout.
- A shared Library search. The Library SQL stays in v4vmm.
- A shared HTTP client or shared response types.

## Verification

Mechanical, in the commit of step 1: each v4vmm search test passes with no change to the test, and the four guards above state the new owner. The diff from `1d6dbb9` shows no change that the crate lacks.

Visual, for the operator, after step 1: search, the field list and saved searches work as they did. The move changes no behavior, so a difference is a fault.
