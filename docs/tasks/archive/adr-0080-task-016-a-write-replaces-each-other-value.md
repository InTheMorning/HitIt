# ADR 0080 Task 016: A Write Replaces Each Other Value Of A Frame

Status: Complete 2026-11-05. Mechanical checks Green. It has no visual check.

## Goal

A frame that a write supplies holds only the values of that write. A frame does not grow.

## Authority

- [ADR 0080](../../adr/0080-tag-frames-follow-their-owner.md) Decisions 6 and 11, amended 2026-11-05.
- The operator decided on 2026-11-05: "overwrite all".

## Required Changes

1. `remove_stale_id3_website_frames` and `remove_stale_lofty_website_items` remove each value of a `WOAR` or `WOAF` frame that the write supplies.
2. `resolve_license_edits` reads only the edits. One URL gives `WCOP`. More than one URL gives `TXXX:LICENSE`. The writer removes each file value of the two frames first.
3. `changed_frames` lists a file with another value in `WOAR`, `WOAF`, `WPAY`, `TPUB`, `TLAN`, or in the license frames.

## Do Not Change

- A write does not change a frame that it does not supply.
- RSS and MusicBrainz values of one write stay side by side.
- No change to the compare rows.

## Acceptance Criteria

Mechanical:

- `adr_0080_write_replaces_each_other_woar_value`: only the value of the write stays in `WOAR`.
- `adr_0080_website_write_replaces_another_value_on_mp3_flac_and_mp4`: the scan lists the file. After one write it holds one URL, and the scan lists nothing.
- `adr_0080_scan_reports_earlier_labeled_woar_and_another_value`: the scan lists a file with the current value beside another value.
- `adr_0080_license_resolution_gives_one_frame`, `adr_0080_license_*_write_replaces_a_different_wcop_url` and `adr_0080_license_*_list_frame_is_replaced`: the file holds only the RSS URL in `WCOP`.
- `adr_0080_mb_write_keeps_channel_and_musicbrainz_homepage_in_woar`: two values of one write stay.
- The commit checks of AGENTS.md pass.

Visual: none. The packet changes no screen, and the tests read the written files. The next "Update n files" can list more files. The final walk of the run reads that list.

## Known Limit

A value from another tool or from MusicBrainz is lost at the next write of its frame. The app keeps no copy.

## Test Commands

- `cargo test --lib adr_0080_`
- `cargo test`
- `cargo test --test architecture_tests`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --release`
- `cargo build --bin v4vmm`
