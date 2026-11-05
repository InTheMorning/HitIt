# ADR 0080 Task 007: A Credit Role Selects The Correct Frame

Status: Implemented 2026-11-05. Mechanical checks Green. Operator visual check open.

## Goal

Each credit role selects the frame that its name states. One function classifies a role, so the frame and the normalized role cannot disagree.

## Authority

- [ADR 0080](../adr/0080-tag-frames-follow-their-owner.md) Decisions 3 and 6: the compare uses the resolution of the writer, and a write is idempotent.
- `id3v24_musicbrainz_rss_mapping.html` is the advisory frame map.
- Studio ADR 0112 owns the Studio role list. This packet does not change it.
- [AGENTS.md](../../AGENTS.md): "Every fix gets a guard".

## Recorded Facts - 2026-11-04

An agent read the code to find these facts. No test confirms them on 2026-11-04.

- `contributor_id3_rows` in `src/metadata.rs` calls `contributor_role_target` with the lowercase role of the feed.
- `contributor_role_target` tests the roles in this sequence: composer, lyricist, album artist, then `TIPL` words, then musician, conductor, remixer, original artist, original lyricist.
- Thus:
  - `remixer` contains `mixer`, and goes to `TIPL`, not `TPE4`.
  - `original artist` contains `artist`, and goes to `TPE1`, not `TOPE`.
  - `original lyricist` contains `lyric`, and goes to `TEXT`, not `TOLY`.
  - `Foley Artist` of the Studio role list contains `artist`, and goes to `TPE1`.
- `normalized_contributor_role` tests original lyricist and original artist first, and `remix` before `mix`. It reads the same strings differently from `contributor_role_target`.
- `Band` of the Studio list goes to `TPE2`. `Writer` goes to `TEXT`. Studio writes lowercase roles (Studio ADR 0112 item 5).
- A role with no target goes into `TXXX:MusicIndex Contributors` only. This is correct.

## Required Changes

1. Write one role classifier in `src/metadata.rs`. `contributor_role_target`, `normalized_contributor_role` and `involved_people_role` read its result.
2. Test the specific forms first: original lyricist, original artist, remixer, then the forms that exist today.
3. A role that holds the word `artist` selects `TPE1` only when the role is one of these: `artist`, `lead artist`, `main artist`, `featured artist`, `recording artist`, `solo artist`. Each other role that holds `artist`, for example `foley artist` or `cover artist`, selects no frame. It stays in `TXXX:MusicIndex Contributors`.
4. Keep each other present result. `musician`, roles with `lead` and roles with `performer` keep `TPE1`.
5. Do not change the instrument path of `instrument_role`.

## Do Not Change

- No change to `TXXX:MusicIndex Contributors`.
- No change to the channel credit fallback in `track_with_feed_defaults`.
- No new attribute of a credit in a file.

## Acceptance Criteria

Mechanical:

- A table test in `src/metadata.rs` gives the frame of each role: `remixer` `TPE4`, `original artist` `TOPE`, `original lyricist` `TOLY`, `foley artist` none, `artist` `TPE1`, `mix engineer` `TIPL`, `mastering engineer` `TIPL`.
- A test reads the 70 roles of the Studio list as text in the test, with their expected frames. Each role keeps its frame or has a stated new result.
- A test: `normalized_contributor_role` and the frame agree for each role of the table.
- The ADR 0080 round-trip test passes for a track with each role of the table.
- The commit checks of AGENTS.md pass.

Visual, listed apart, for the operator to walk:

- V1: a track with a `remixer` credit shows a difference for `TPE4` in "Update n files". After the update, an external tag reader shows the name in `TPE4` and not in `TIPL`.

## Operator Visual Check

1. Run `cargo run --bin v4vmm` in a desktop session.
2. Open a Library track with a `remixer` credit in its feed. If no track has one, add the credit to a test feed in Studio and run the RSS check of that playlist.
3. Open "Update n files", and read the list of frames.
4. Wrong: the list does not show `TPE4`, or it shows the name for `TIPL`. A file that this app wrote earlier can still hold the earlier `TIPL` value after the update. The [phase plan](../plans/adr-0080-tag-frames-phase-plan.md#open-finding---2026-11-05) records this, so it is not a failure of this check.
5. Confirm the update. Run `ffprobe -hide_banner FILE`, with FILE as the path of the track.
6. Wrong: the output does not show the name in the remixer tag.
7. Run the scan again. Wrong: the file shows a difference.

Cleanup: remove the test credit in Studio if you added it. The tag rewrite stays, because it is correct.

## Test Commands

- `cargo test --lib metadata::`
- `cargo test --lib adr_0080_`
- `cargo test`
- `cargo test --test architecture_tests`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo check --all-targets`
- `cargo build --bin v4vmm`

## Report

Give the files changed, the tests run, and each role in the Studio list that changes its frame.

## Stop And Ask

Stop if an existing test requires no frame for a role in the list of Required Change 3.

Stop if an existing test requires `TPE1` for a different role.
