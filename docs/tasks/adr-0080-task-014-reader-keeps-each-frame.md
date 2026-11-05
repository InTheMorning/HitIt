# ADR 0080 Task 014: The MP3 Reader Keeps Each Frame

Status: Implemented 2026-11-05. Mechanical checks Green. Operator visual check open.

## Goal

The MP3 reader gives each frame of a tag. A write keeps each frame that it does not replace.

## Authority

- [ADR 0080](../adr/0080-tag-frames-follow-their-owner.md) Decision 6: a write keeps each value that no source supplied.
- [AGENTS.md](../../AGENTS.md): "Every fix gets a guard".

## Recorded Facts - 2026-11-05

- After a write of 143 files, "To be in Nashville" and "The Wait Is Over" showed `file: no value` for frames that `exiftool` shows in the files.
- In the two files, each frame with no value comes after one position in the tag. Each earlier frame shows its value.
- `id3` 1.16.4 calls `read` for a frame header, and takes a short read as the end of the tag. A frame header across the 8 KiB read buffer gives a short read. The crate gives no error.
- `write_mp3_edits` reads the tag with the same function. A write on such a read removes each later frame.
- `id3` 1.17.1 calls `read_exact`. The FLAC and MP4 paths use `lofty`, and this fault does not apply to them.

## Required Changes

1. Require `id3` 1.17, and use 1.17.1 in `Cargo.lock`.
2. Add a test with a frame header across the read buffer. It reads the tag and writes one edit.

## Do Not Change

- No change to the frames that the app writes.
- No change to the scan.

## Acceptance Criteria

Mechanical:

- `adr_0080_frame_header_across_read_buffer_keeps_each_later_frame` failed on `id3` 1.16.4, and it passes on 1.17.1.
- The commit checks of AGENTS.md pass.

Visual, listed apart, for the operator to walk:

- V1: "Update n files" does not list "To be in Nashville" or "The Wait Is Over" with `file: no value`.

## Operator Visual Check

1. Close v4vmm. Start `~/build/v4vmm/target/release/v4vmm`.
2. Wait for the button in the left sidebar. Click **Update n files**.
3. Wrong: "To be in Nashville" or "The Wait Is Over" shows a frame with `file: no value` that `exiftool -a -G1 -s FILE` shows in the file.
4. Click **Cancel**.

Cleanup: none.

## Known Limit

A write with an earlier version can have removed frames from a file. These frames are not known if no source supplies them. The test cannot find them, and the scan cannot find them.

## Test Commands

- `cargo test --lib adr_0080_frame_header_across`
- `cargo test`
- `cargo fmt -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --bin v4vmm`
