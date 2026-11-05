# ADR 0080 Task 011: A Confirmed Label In `TPUB`

Status: Ready - 2026-11-04. The operator visual check stays open until a person walks it.

## Goal

`TPUB` holds a label that both sides of a link confirm. The `itunes:owner` name stops being the RSS value of the "Label" row.

## Authority

- [ADR 0080](../adr/0080-tag-frames-follow-their-owner.md) Decisions 3, 4 and 12.
- [ADR 0082](../adr/0082-publisher-roles-belong-to-each-album-link.md): a role belongs to each album link.
- [ADR 0085](../adr/0085-artists-by-confirmation.md): the confirmation levels. musicindex ADR 0033 R-WORD-3 and R-WORD-4 own the words.
- [ADR 0075](../adr/0075-metadata-ownership-and-completeness.md) Decision I: a MusicBrainz label stays apart.

## Recorded Facts - 2026-11-04

- `source_value_for_metadata_field` in `src/metadata.rs` gives `publisher_text` for "Publisher" and for "Label".
- `id3_frame_hint` maps "Label" to `TPUB`, and "Publisher" to `TXXX:V4V_PUBLISHER`.
- The "Label" row comes from the MusicBrainz list (`push("Label", "label-info", ...)`). No RSS row writes `TPUB`, but the compare shows the owner name as its RSS value.
- `feed_publisher_relationships` holds `role`, `role_agreement`, `publisher_feed_title` and `album_names_as` for each link (`src/db/publisher_relationships.rs`). `named_feeds` reads it.
- The live API gives `agreed_roles` `["label"]` for Sir Libre Records, on a list row. A `publisher` entry gives `role` and `role_agreement`.
- On 2026-10-18, 6 albums named a second feed with the role `label`.

## Required Changes

1. Give `TrackContext` the confirmed label titles of the feed of the track. Read them from `feed_publisher_relationships` in the same place as the other local facts. For an Index context, read them from the `publisher` entries of the feed response.
2. A confirmed label has a `role` that holds `label`, and `role_agreement` `both`. Keep the RSS sequence of the named feeds.
3. Remove `"Label"` from the `publisher_text` branch of `source_value_for_metadata_field`. "Label" gives the confirmed titles, joined as `join_values` joins them.
4. Add an RSS "Label" row with its frame `TPUB`. Keep the MusicBrainz value on the same row, as "Website" does.
5. The compare and the writer use the same value (Decision 3).
6. Keep `TXXX:V4V_PUBLISHER` as it is.

## Do Not Change

- No new request for each track or for each album (ADR 0075 packet 047). If a context has no stored relationship, it gives no `TPUB` value.
- No change to the confirmation rule of ADR 0085.
- No new interface word. Use the words that the screens use today.

## Acceptance Criteria

Mechanical:

- The tag edits for a track of an album with a confirmed `label` link hold its title in `TPUB`.
- A link with `role_agreement` `conflict`, or with no agreement, gives no `TPUB` value.
- A link with only the `artist` role gives no `TPUB` value.
- An `itunes:owner` name gives `TXXX:V4V_PUBLISHER` only. The RSS value of the "Label" row is empty for that track.
- Two confirmed labels give two values in their RSS sequence, and a second write gives equal frames.
- The compare of freshly built edits reports no difference.
- A MusicBrainz label stays apart from the RSS value in the row.
- The commit checks of AGENTS.md pass.

Visual, listed apart, for the operator to walk:

- V1: a track of an album with Sir Libre Records as its confirmed label shows `TPUB` in "Update n files".
- V2: the "Label" row of a track with only an owner name shows no RSS value.

## Operator Visual Check

1. Run `cargo run --bin v4vmm` in a desktop session.
2. Open a Library track of "Motive to Kill", or of another album that names Sir Libre Records. Download the album first if the Library does not hold it.
3. Open "Update n files".
4. Wrong: the list does not show `TPUB: Sir Libre Records`.
5. Open a track of an album with no label link, and open its metadata compare.
6. Wrong: the "Label" row shows the owner name as its RSS value.
7. Confirm the update of the first track. Run `ffprobe -hide_banner FILE`, with FILE as the path of the track.
8. Wrong: the publisher tag does not show the label. Then run the scan again. Wrong: the file shows a difference.

Cleanup: remove the album from the Library if you downloaded it for this check.

## Test Commands

- `cargo test --lib adr_0080_`
- `cargo test --lib metadata::`
- `cargo test`
- `cargo test --test architecture_tests`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo check --all-targets`
- `cargo build --bin v4vmm`

## Report

Give the files changed and the tests run. Name the path that gives the titles for a Library context and for an Index context.

## Stop And Ask

Stop if a track context cannot read the relationship without a new request.
