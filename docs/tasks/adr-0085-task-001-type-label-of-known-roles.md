# ADR 0085 Task 001: The Type Label Lists Each Known Role

Status: Implemented - 2026-11-14. Mechanical checks are Green. The visual checks V1 to V3 are open in section 16 of [pending human checks](../pending-human-checks.md).

## Goal

The type label of a feed page lists each known role in the R-ROLE-1 sequence, with " · " between two words. Each label that names a role ignores a role word that is not known. Sir Libre Records shows "Label".

## Authority

- [ADR 0085](../adr/0085-artists-by-confirmation.md) Decision 7 and its amendments of 2026-11-05 and 2026-11-14.
- musicindex ADR 0037 R-WORD-7, as the orchestrator relayed it on 2026-11-14:
  1. The type label of a publisher feed with roles is the word of each role, in the sequence of ADR 0036 R-ROLE-1. One " · " is between two words, for example "Artist · Label · Producer · Host · Author".
  2. The dot has the color of the first role in that sequence.
  3. A row that adds a count to the label keeps the count after the roles, for example "Artist · Label · 4 albums".
  4. "Publisher" and "Feed" (ADR 0033 R-WORD-3 items 4 and 5) stay the labels of a feed with no role.
- musicindex ADR 0066 (commit `df500dc`):
  - R-ROLE-3: the known roles are the eight roles of ADR 0036 R-ROLE-1 item 1. A type label and a dot ignore each other role word. The compare ignores capital letters.
  - R-ROLE-4: a feed with only unknown role words has no role. It gets "Publisher" or "Feed" with a hollow dot.
  - R-ROLE-5: a technical view, such as Inspect sources, shows each role word as RSS text, also an ignored word, with no added capital letter.
  - R-WORD-8: the one dot has the color and the fill of the first role. It is filled when the two sides give the first role. It is hollow when one side gives it or the app infers it. The other roles do not change the dot.
- The R-ROLE-1 sequence: artist, label, hosting, producer, host, author, network, sponsor.
- The separator is a space, the middle dot U+00B7 and a space.

## Recorded Facts

- 2026-11-14: `PublisherPageVm::header_badge` (`src/view_models/publisher_page.rs:759`) gives "Artist" alone when a side states `artist`. It gives "Publisher" for two or more other roles. It compares `"artist"` with capital letters.
- 2026-11-14: `header_role_for_word` (`src/view_models/publisher_page.rs:492`) maps each word with no color to `HeaderRole::Publisher`.
- 2026-11-14: `role_words` (`src/view_models/role_words.rs:29`) keeps the case of the feed and sorts by the lowercase word. It keeps unknown words.
- 2026-11-14: these callers also make a label from role words:
  - `confirmed_feed_roles` in `src/view_models/library.rs:2657`, the roles of a feed in the Artists list
  - `named_feed_row` in `src/view_models/entity_detail.rs:967`, the role row of a release page ("Label: Sir Libre Records")
  - `from_link_facts` in `src/view_models/artists_everything.rs:168`, the level of a feed in the scope All
  - `header_facts` and the code near `src/view_models/publisher_page.rs:923`
- 2026-11-14: the test `adr_0033_r_word_3_item_1_two_roles_rows` (`src/view_models/publisher_page.rs:1433`) can assert the earlier "Publisher" label. This is an inference. Examine it first.
- 2026-11-14: Sir Libre Records gives `label` on one side and `recordLabel` on the other side.

## Base components

| Component | Status | Location | Reason |
|---|---|---|---|
| Role word list | extend | src/view_models/role_words.rs | Add one function that gives the known roles of a role set in the R-ROLE-1 sequence. Keep `role_words` for Inspect sources. |
| Header badge | extend | src/view_models/publisher_page.rs | `header_badge` gives the new label, color and dot. |
| Entity color of a role | reuse | src/view_models/publisher_page.rs | `HeaderRole` has one variant for each known role. |

## Required Changes

1. Add a typed known role to `src/view_models/role_words.rs`. Give it the eight roles in the R-ROLE-1 sequence. Add one function that returns the known roles of a role set, with no duplicate, in that sequence. The compare ignores capital letters.
2. Add one function that joins known roles into a type label with " · ". Each word has a capital first letter.
3. Change `header_badge`:
   - The roles are the known roles that a side states. When the level is Artist or Unverified artist and no side states `artist`, "Artist" comes first (ADR 0085 Decision 7).
   - The word is the joined label. The role is the `HeaderRole` of the first role.
   - `confirmed` is true when the two sides give the first role (R-WORD-8). For "Artist" from the level, `confirmed` is true when the level is Artist.
   - With no known role, keep the "Publisher" and "Feed" rules of Decision 7, with a hollow dot (R-ROLE-4).
4. Make each caller in Recorded Facts that shows a role name use the known roles. A row that adds a count puts the count after the roles (R-WORD-7 item 3).
5. Keep the role row of the release page with its "and" join. Remove only the unknown words from it. This is an open point of this task: R-ROLE-3 names a type label, and the role row is a label of the same type. The operator can change this selection at the visual check.
6. Inspect sources keeps each role word as RSS text (R-ROLE-5). Do not change it.
7. Remove `HeaderRole::Publisher` from `header_role_for_word`, or remove the function, if no caller can give it a word that is not known.
8. Change the tests that assert the earlier label. Each changed test names ADR 0085 Decision 7 and musicindex ADR 0037 R-WORD-7.

## Do Not Change

- No change to the confirmation levels of Decision 2.
- No change to the colors. `HeaderRole` keeps one color for each role.
- No change to Inspect sources.
- No change to the tag frames. `role_contains_label` stays the test of ADR 0080 Decision 3.

## Acceptance Criteria

Mechanical:

- A test gives "Label" with a filled dot for the sides `label` and `label,recordLabel`.
- A test gives "Label · Producer" with a filled label dot, when the two sides give `label` and only the album side gives `producer`.
- A test gives "Artist · Label" with the artist color, when a side states `artist` and `label`.
- A test gives "Publisher" or "Feed" with a hollow dot for a feed with only unknown words.
- A test proves that `LABEL`, `Label` and `label` give the same role.
- A test proves that the Artists list gives the roles in the R-ROLE-1 sequence, and a count after the roles where a count shows.
- The commit checks of AGENTS.md pass.

Visual, listed apart, for the operator to walk:

- V1: the page of Sir Libre Records shows the badge and the title "Label". Its shared releases heading is "Artists on these releases".
- V2: a feed page with two or more known roles shows each role with " · ", and the dot has the color of the first role.
- V3: Inspect sources of Sir Libre Records still shows `recordLabel`.

## Operator Visual Check

1. Run `cargo run --bin hitit` in a desktop session.
2. Open the feed page of Sir Libre Records. Do V1.
3. Open a feed page with two or more roles. Do V2.
4. Open Inspect sources on the Sir Libre Records page. Do V3.
5. Wrong: a page shows "Publisher" for a feed with a known role, or Inspect sources hides a role word.

## Test Commands

- `cargo test --lib view_models::role_words`
- `cargo test --lib view_models::publisher_page`
- `cargo test --lib view_models::library`
- `cargo test --test architecture_tests`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin hitit`

## Implementation Notes - 2026-11-14

- `role_words.rs` has `KnownRole`, `known_roles` and `type_label`. `role_words` stays for Inspect sources.
- `header_role_for` replaces `header_role_for_word`. `HeaderRole::Publisher` is now only the color of a feed with no known role that is definitely not an artist.
- The Artists list (`confirmed_feed_roles`) and the role row of the release page use the known roles. A role row with no known role is an "Also named" row (Decision 6).
- A second commit changes `FeedLinkClass::OtherRoles` to known roles. A feed with only unknown role words has no role (R-ROLE-4). The badge of a search result row is the type label, for example "Label · Producer".
- Open, asked of the orchestrator on 2026-11-14: the "Roles" fact of `header_facts` and the "As <roles>" section heading show all role words. R-ROLE-3 names only a type label and a dot.

## Report

Give the files changed and the tests changed. For each caller in Recorded Facts, tell if it changed and why.

A delegate does not commit. It leaves its change in the working tree and reports it. Write the report in Simplified Technical English (`/home/citizen/.agents/skills/asd-ste100/SKILL.md`).

## Stop And Ask

- Stop if a caller shows role words that is neither a type label nor a technical view.
- Stop if a feed page with the level Artist and no stated `artist` gives a different first role in the current tests.
