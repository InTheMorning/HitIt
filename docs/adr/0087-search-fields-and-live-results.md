# ADR 0087: Search Has Fields, A Field List And Live Results

## Status

Accepted - 2026-10-23. The operator asked for this design, answered its first four questions and accepted it on 2026-10-23 (see "Operator Answers").
The [ADR 0087 phase plan](../plans/adr-0087-search-fields-phase-plan.md) holds its packets.

Amended 2026-11-07: [ADR 0091](0091-search-parser-moves-to-the-shared-crate.md) moves the parser, the Index plan and the kind rule to the `musicindex-search` crate, and musicindex ADR 0049 then owns the syntax. Until step 1 of its adoption, the code and the guards stay as they are.

It amends [ADR 0084](0084-music-column-browser.md) Decision 2. The toolbar search field shows a field list and reads fields in the query text. It shows results while the curator types. Search stays a toolbar command, never a section (ADR 0060).
It amends [ADR 0084](0084-music-column-browser.md) Decision 3: a saved search runs its fields against the Library and the Index.

Amended 2026-11-05 by [ADR 0089](0089-the-shell-destinations-and-list-pages.md): the saved searches move from Discover to the Search page. The toolbar field stays on each page.

Amended 2026-10-24: the operator decided that a live run retains no failed action. Decision 4 now states it. [ADR 0087 task 009](../tasks/adr-0087-task-009-one-error-for-the-latest-text.md) builds it.

Amended 2026-11-07: Decision 4 runs a query with only exclusions, so that the note of Decision 5 shows.

Amended 2026-11-06: Decision 5 sends each Index field as its own filter. [ADR 0087 task 011](../tasks/adr-0087-task-011-index-receives-the-fields.md) builds it. The known differences from the Library are in "Known Differences From The Library".

## Context

On 2026-10-23 the operator asked for a powerful search. The search field must show the field headers when the curator clicks it, as Discord does. The curator must be able to type fields in the query, as in the library search of Mixxx.

The orchestrator recorded these facts on 2026-10-23:

- The toolbar field is a `gpui_component` `InputState` that `TopApp` owns (`src/app.rs:147`). It reacts only to `InputEvent::PressEnter` (`on_global_search_event`, `src/app.rs:604-614`). Nothing runs while the curator types.
- No code parses a query. The Library search is one SQL `LIKE` match against the track title, the artist, the album title, the album artist and the feed title (`db::search_library_tracks`, `src/db.rs:561-601`). It gives at most 50 tracks (`DEFAULT_LOCAL_LIBRARY_SEARCH_LIMIT`).
- The Index search sends three `GET /v1/search` requests, for `type=feed`, `type=track` and `type=publisher` (`fetch_index_search_result_rows`, `src/application/queries/search.rs:226-275`).
- Stophammer gives `q` to FTS5 after it removes some operator characters (`sanitize_fts5_query`, `../stophammer/src/search.rs:224-242`). The FTS5 columns are `name`, `title`, `description` and `tags`. Thus `title:foo` works by chance, `artist:foo` gives `400`, and an apostrophe gives `400` (open request 10).
- The Stophammer contract of release 0.18.1 filters `GET /v1/search` by `artist`, `title`, `release` and `kind`, by date (`after` and `before`, each `YYYY-MM-DD`) and by track duration (`min_duration` and `max_duration`, in integer seconds). Each filter applies to some entity types only. A type that does not accept a filter gives `400`. The text filters and the exclusions are in open request 10. The values were read on 2026-11-06 with read-only GET calls.
- A saved search stores the raw query text only (`src/db/saved_searches.rs`, migration 21).
- The popover primitive keeps the keyboard focus inside the popover (`src/ui/primitives/popover.rs`). A field list must keep the focus in the search field.

## Decision

### 1. One Query Text With Words, Phrases And Fields

The query text holds terms. The terms of one query all apply together.

- A **word** matches as today, for example `haleen`.
- A **phrase** is text in double quotes, for example `"night drive"`. It matches as one string.
- A **field** is a field name, a colon and a value, for example `artist:jsmovie` or `release:"aged friends"`. Decision 2 lists the fields.
- A `-` before a word, a phrase or a field excludes the matches, for example `-title:remix`.
- A field name has no case. `Artist:` is the same as `artist:`.
- A text with a colon whose name is not a field stays one word. Thus `Re:Union` searches for "Re:Union".
- A field with a value that is not correct, for example `after:yesterday`, applies nothing. The field list shows the error of that field (Decision 3).

One parser in the view model layer reads the text. The parser gives a typed query: the words, the phrases, the fields and each error. The Library search and the Index search read the typed query, never the text.

### 2. The Fields

| Field | Values | Library | Index |
|---|---|---|---|
| `artist:` | text | The track artist or the album artist | Stophammer filter (Decision 5) |
| `title:` | text | The track title or the release title | Stophammer filter |
| `release:` | text | The release title | Stophammer filter |
| `kind:` | `album`, `ep`, `single`, with commas for more than one | The release kind of ADR 0084 Decision 5 | Stophammer filter. It sends the kind requests of musicindex ADR 0049 R-QUERY-6 (Known Differences 2) |
| `in:` | `library`, `index` | Sets the scope | Sets the scope |
| `after:`, `before:` | a date, `2026-10-01`, or a month, `2026-10` | The release date of ADR 0086 Decision 1, and the `pub_date` of a track | Stophammer filter. A feed row uses its newest item date (Known Differences 3) |
| `duration:` | `>5m`, `<3:30`, `3m..5m` | The duration of a track | Stophammer filter for track rows only (Known Differences 4) |
| `is:` | `downloaded`, `partial`, `ready`, `removed` | The Library state of ADR 0084 Decision 8, the show readiness of ADR 0059, and the "Removed from feed" mark of ADR 0076 | None |
| `playlist:` | a playlist name | The tracks of that playlist | None |
| `has:` | `split`, `image` | A value block with payment routes, or an image of its own | None |

- `after:` and `before:` include their date. A month includes each day of the month.
- `duration:` applies to tracks. A release row shows when one or more of its tracks match.
- `is:`, `playlist:` and `has:` apply to the Library only. A query with one of them shows no Index rows. The field list marks these fields "Library only".
- `in:index` shows no Library rows. A query with `in:index` and a Library-only field shows no rows, and the field list shows the conflict.
- An excluded `kind:`, `after:`, `before:` or `duration:` has no Index filter. The Index part shows the note of Decision 5.

### 3. The Field List

A click in the empty search field opens the field list under the field. Each row gives one field, its values and one example, for example `kind:  album, ep, single  ·  kind:single`.

- While the curator types a field name, the list shows the fields that start with that text.
- After a field name and its colon, the list shows the values of the field:
  - `artist:` shows the matching artist names of the Library and of `GET /v1/artist-names?from=`.
  - `release:` shows the matching release titles of the Library.
  - `playlist:` shows the matching playlist names.
  - `kind:`, `in:`, `is:` and `has:` show their fixed values.
- The keyboard focus stays in the search field. Up and Down move the selected row. Tab and Enter put the selected row into the text. Escape closes the list. A click on a row puts it into the text.
- A click that is not on the field or the list closes the list, and the field keeps the keyboard focus. The focus alone does not open the list. A click in the field or a typed key opens it.
- Enter with no selected row runs the search.
- The list shows the error of a field, for example "after: needs a date, for example 2026-10-01".
- The view model gives each row, its text, its accessibility label and its typed action state. The renderer only composes.
- A shared composite under `src/ui/composites` draws the list. It does not use the popover primitive, because that primitive takes the focus.

### 4. Results Show While The Curator Types

After the curator stops typing for a pause, the search runs against the Library and the Index. Enter runs it at once. A named token holds the pause, 300 ms at first.

- Each run has a generation. The app drops a result of an older generation.
- A text with no word, phrase or correct field runs nothing. A term with `-` counts, so `-remix` runs. The Library part lists each row except the excluded rows, and the Index part sends no request (Decision 5). `-!!!`, `-artist:` and `in:library` alone run nothing.
- The mounted search page updates in place. A run does not add a new page to the navigation stack. Enter, or a click on a result, keeps the page in the stack (ADR 0084 Decision 9).
- Each run sends at most three Index requests, the same as one search today.
- A live run retains no failed action of ADR 0066. Enter retains it.

### 5. The Index Receives Filters, Never Syntax

v4vmm sends the words and the phrases in `q`, and each Index field as its own parameter. Stophammer never parses the field syntax. [Open request 10](../plans/v4vmm-open-requests.md#10-search-accepts-words-phrases-and-field-filters) asks for these parameters. Stophammer deployed them in release 0.15.0.

- `q` holds the words and the phrases with no `-`. A query with fields and no word sends no `q`.
- `artist:`, `title:` and `release:` give the parameters `artist`, `title` and `release`. Two values of one field join with a space, so each value applies.
- A word or a phrase with `-` gives `not_q`. `-artist:`, `-title:` and `-release:` give `not_artist`, `not_title` and `not_release`. Each value is one exclusion.
- `kind:` gives the parameters of musicindex ADR 0049 R-QUERY-6, which the crate `musicindex-search` builds. `single` gives `kind=unknown` and `tracks=1`. `album` gives `kind=unknown` and `min_tracks=2`. Two `kind:` terms keep the kinds that both have. No shared kind sends no Index request. `ep` sends no Index request, because a count never gives EP.
- `after:` gives `after`, and `before:` gives `before`. Each value is `YYYY-MM-DD`. A day gives that day in both. A month gives its first day in `after:`, and its last day in `before:`.
- `duration:` gives `min_duration` and `max_duration`, in integer seconds. Each limit is included, as `duration:` is.
- Each entity type takes only some filters. A publisher row has no release, kind, date or duration, so `type=publisher` gets no request when the query has one of them. A feed row has no duration, so `type=feed` gets no request for a duration. Stophammer gives `400` for a filter that a type does not take. v4vmm sends no such filter. Each type gets its own request, so the run still sends at most three requests.
- A query with exclusions and no positive term sends no request. Stophammer gives `400` with the code `missing_positive_term` for that request. The page shows the note "Music Index needs a word or a field to search for. The Library rows show."
- An excluded `kind:`, `after:`, `before:` or `duration:` has no parameter. The Index part shows the note "Music Index cannot exclude kind: yet. The Library rows show." The crate `musicindex-search` gives the first sentence. The contract has no `not_` form for these fields.

The page never shows Index rows that a filter did not reduce.

### 6. A Saved Search Keeps Its Text

A saved search stores the query text, as today. Each run parses the text again.

Migration 21 added a unique index of the query. Migration 23 drops that index. The operator decided on 2026-11-06 that a saved search is keyed by its name. The row id stays the primary key and the navigation key. Two saved searches may hold one query. The save action does not save a second copy of a query that a saved search holds. A query edit that gives the text of another saved search is accepted. The name of a saved search follows the query only when the name equals the old query text, without case. A name that the operator gave does not change. ADR 0089 Decision 4 owns the rules of the name.

## Invariants

- The start of a fixed value, such as `has:spl`, is an incomplete value and not an error. The operator asked for this on 2026-11-06.
- One parser reads the query text. No other code reads field syntax.
- A Library value goes into SQL only as a bound parameter, with its `LIKE` characters escaped.
- A Library text term matches at the start of a word, as the Index does. "hey" finds "HeyCitizen" and not "They". The operator decided this on 2026-11-06.
- No Index request carries field syntax in `q`.
- Index rows of an entity type show only when Stophammer applies each Index field of the query to that type.
- No Index request holds a filter that its entity type does not accept.
- A result of an older run never replaces the result of a newer run.

## Non-Goals

- Colored chips for the fields in the text. The text stays plain in this ADR.
- Search operators `OR`, parentheses and fuzzy matches.
- Fields for genre, BPM and key. The Library stores them only as file tags.
- A change to the result tabs or to the search page layout.

## Known Differences From The Library

These differences stay after packet 011 of [ADR 0087 task 011](../tasks/adr-0087-task-011-index-receives-the-fields.md). Each one comes from the Index, not from the parser. The values were read on 2026-11-06 with read-only GET calls.

1. Closed on 2026-11-07. Stophammer release 0.19.0 (its ADR 0085) makes the `title` filter of a track match the track title or the title of its feed, as the Library does. On 2026-11-07, `title=assorted&type=track` returned rows. No code in v4vmm changed.
2. **`kind:`.** Closed on 2026-11-07. The crate `musicindex-search` builds the kind requests of musicindex ADR 0049 R-QUERY-6, and v4vmm sends them. `kind:single` sends `kind=unknown&tracks=1`, and `kind:album` sends `kind=unknown&min_tracks=2`. On 2026-11-07, `kind=unknown&tracks=1` and `kind=unknown&min_tracks=2` each gave rows. `kind:ep` still sends no Index request, because a count never gives EP (R-QUERY-6). The Library gives `single` to one track, `album` to each other release and no `ep`. Open request 14, part 2, was declined, and its answer gives the filters that an app sends.
3. **`after:` and `before:` on a feed.** The Index compares the newest item date of the feed. The Library compares the release date (ADR 0086 Decision 1). A track row uses its `pub_date` in both.
4. **`duration:`.** The Index filters track rows only. A query with `duration:` shows no feed rows from the Index. The Library shows a release when one of its tracks matches.
5. **Words in a phrase with a quote.** Closed on 2026-11-07. The crate `musicindex-search` writes a space for each quote and each backslash (musicindex ADR 0049 R-QUERY-5), and v4vmm sends that text. So `q` and each text filter no longer hold `\"` or `\\`. The FTS5 tokenizer reads each space as a separator, so a phrase matches as before.
6. **Artist rows.** A query with no word gives artist rows only for the name of its `artist:` term. A query with no word and no `artist:` term gives no artist row.
7. **The first page.** The Index part reads the first page of each type, 20 rows. The Library reads at most 50 tracks.

## Consequences

- A curator can find "the singles of jSmovie after October 1" with one query.
- Live results send Index requests while the curator types. The pause and the generation limit the cost.
- `is:ready` reads the show readiness of each matched track. On a large Library this can be slow.
- The app adds one shared composite, the field list, and one parser module.
- The Library search gets one query builder for each field. The 50-track limit stays.
- The Index fields send filters to Stophammer. The known differences of this ADR stay until their open requests are answered.

## Alternatives Considered

- **Filter the loaded Index rows in v4vmm.** Rejected by the operator on 2026-10-23: only the first page of each type gets filtered, so a filter gives a wrong Index list.
- **The fields apply to the Library only.** Rejected by the operator on 2026-10-23.
- **The Library live and the Index on Enter.** Rejected by the operator on 2026-10-23: each result updates as the curator types.
- **The field headers only, with no values.** Rejected by the operator on 2026-10-23.
- **Stophammer parses the syntax.** Rejected: v4vmm then cannot tell the curator which field is not correct. Also, the two clients of Stophammer must then agree on one syntax.
- **The popover primitive for the field list.** Rejected: it takes the keyboard focus from the search field.

## Operator Answers - 2026-10-23

1. **Value sources for `artist:`.** The field list sends one `GET /v1/artist-names?from=` request after each pause, with the Library names.
2. **`title:` scope.** `title:` matches the track title and the release title. `release:` matches the release title only.
3. **Order with the remaining work.** The packets of this ADR come after ADR 0086 packet 005 and before ADR 0069 task 002.

## Operator Answers - 2026-11-07

A different session relayed these answers on 2026-11-07. They accept the behavior of commit `c433582`.

1. Results in the scope All have no reverse icon.
2. The note title has no period at its end: "MusicIndex needs a word or a field to search for".
3. Each selected sort makes one `GET /v1/node/capabilities`, with no cache.

The operator visual check of these search corrections stays open in [pending human checks](../pending-human-checks.md).

Later answers of 2026-11-07, relayed by the orchestrator session:

4. `-remix` alone follows Decision 5. The Library part runs, the page shows the note, and the Index part sends no request.
5. Each sort can change the rows two times. The page first sorts its rows, and the Index rows then replace them. The operator accepts this.
6. The operator accepts the look of the sort rows, of `in:library` with the reverse icon, and of the scope All, from the agent screenshots of 2026-11-07.
7. The subtitle of a saved search with no name of its own names what its query reads: "Library and Index", "Library" or "Index". The orchestrator session directed this correction on 2026-11-07. A query with a scope conflict reads nothing, so its subtitle is "No results".
8. The operator passed the look of the `-remix` note (`shots2/t1-remix-pause.png`, `shots2/m11.png`), the Dark theme of the search pages (`shots2/o1-dark-note.png`) and the Phase 3 check of the search move plan, `haleen kind:single` (`shots/s6-kind-single.png`, `shots2/b2-kind-tracks.png`), from the agent screenshots in `/tmp/claude-1000/-home-citizen-build-v4vmm/d4edb497-6260-481e-8270-1c6baa78e1dc/scratchpad/xv`, through the orchestrator session on 2026-11-07. A later real use can reopen each item.
9. The note of the Index part is above the rows, outside their scroll container, so a long list does not hide it and each row keeps its index. The operator decided this on 2026-11-07, through the orchestrator session.

## Verification

### Mechanical Criteria

A unit test in the owner layer proves each criterion.

- The parser gives the words, the phrases, the fields, the exclusions and the errors of a query text. `artist:jsmovie kind:single after:2026-10-01 -title:remix "night drive"` gives five terms.
- The parser keeps `Re:Union` as one word, and gives an error for `after:yesterday`.
- The Library query builder binds each value as a parameter and escapes `%`, `_` and `\`.
- A Library query with each field of Decision 2 gives the expected tracks from a recorded database.
- A query with a Library-only field gives no Index request.
- A query with an excluded `kind:`, `after:`, `before:` or `duration:` gives no Index request and the note of Decision 5.
- Each field of a query reaches its parameter, and a request holds no filter that its entity type does not accept. Packet 011 tests these in `src/application/queries/search.rs` and `src/view_models/search_results/search_run.rs`.
- No Index request holds a field name in `q`.
- The field list view model gives the fields for an empty text, the matching fields for `ar`, and the values for `kind:`.
- The field list view model moves the selection with Up and Down, and Tab puts the selected row into the text.
- A result of an older generation does not change the search page view model.
- A saved search with field text gives the same typed query after a restart.

### Visual Criteria

A person inspects each criterion in the running app.

- The field list opens under the field on a click, in Light and Dark, at normal and narrow width.
- The focus stays in the field while the curator moves through the list with the keys.
- The values of `artist:` and `playlist:` show and update while the curator types.
- The results change in place while the curator types, with no flash of old results.
- The error text of a field that is not correct.
- The note of Decision 5 for an excluded field that the Index cannot apply.
