# ADR 0080 Tag Frames Phase Plan

## Status

Active - 2026-09-29. This plan is advisory. It states no rule.
[ADR 0080](../adr/0080-tag-frames-follow-their-owner.md) owns the rules.

## Packet Register

| Packet | Scope | Owners | Depends on | State |
|---|---|---|---|---|
| [001](../tasks/adr-0080-task-001-rss-frames-and-idempotent-writes.md) | RSS frames by owner, the album description frame, plain URLs, idempotent writes, and the compare and scan that use the writer resolution | ADR 0080 Decisions 1 to 8, for RSS values | ADR 0075 packet 022 | Implemented 2026-09-29. Operator V1 to V3 passed 2026-10-07. V4 moved to the overhaul plan |
| [002](../tasks/adr-0080-task-002-musicbrainz-url-relations-by-type.md) | MusicBrainz URL relations by relation type: release-group official homepage to `WOAR`, license to `WCOP` or `TXXX:LICENSE`, and no frame for each other type | ADR 0080 Decisions 6 and 8, for MusicBrainz values | 001 | Implemented 2026-09-29. Mechanical checks Green. Visual gate open and paused |
| [003](../tasks/archive/adr-0080-task-003-vorbis-date-shares-one-key.md) | The scan compares by the storage key of the format, and a write includes each edit that shares a key. A FLAC date settles after one write | ADR 0080 Decision 6 | 001 | Complete 2026-10-03. Operator V1 passed |
| [004](../tasks/archive/adr-0080-task-004-old-itunes-frames-and-safe-tag-writes.md) | Convert old iTunes v2.2 frames, write tags through a staged copy, and report a download tag-write failure | ADR 0080 Decision 6, ADR 0066 | 003 | Complete 2026-10-03. Operator V1 passed |
| [005](../tasks/archive/adr-0080-task-005-musicindex-image-frame.md) | The artwork URL in `TXXX:MusicIndex Image`: item image, else channel image, `http` or `https` only | ADR 0080 Decision 9 | 004 | Complete 2026-10-04. Operator V80-51 and V80-52 passed |
| [006](../tasks/adr-0080-task-006-contract-0-18-1-and-link-label.md) | The stored contract of Stophammer 0.18.1, and `label` on a source link | ADR 0075 section 6 | None | Implemented 2026-11-05. Mechanical checks Green |
| [007](../tasks/adr-0080-task-007-contributor-role-frames.md) | One role classifier, so that each credit role selects the correct frame | ADR 0080 Decisions 3 and 6 | None | Implemented 2026-11-05. Mechanical checks Green. Operator visual check open |
| [008](../tasks/adr-0080-task-008-rss-readers-agree.md) | The two RSS readers give one track number and one artwork URL | ADR 0075 artwork order, ADR 0080 Decision 9 | None | Implemented 2026-11-05. Mechanical checks Green. Operator visual check open |
| [009](../tasks/adr-0080-task-009-funding-link-frame.md) | The funding link in `WPAY` | ADR 0080 Decision 10 | 006 | Implemented 2026-11-05. Mechanical checks Green. Operator visual check open |
| [010](../tasks/adr-0080-task-010-license-frames.md) | The RSS license in `WCOP` or `TXXX:LICENSE`, and in `TXXX:MusicIndex License` | ADR 0080 Decision 11 | 006 | Implemented 2026-11-05. Mechanical checks Green. Operator visual check open |
| [011](../tasks/adr-0080-task-011-confirmed-label-frame.md) | A confirmed label in `TPUB` | ADR 0080 Decision 12 | None | Implemented 2026-11-05. Mechanical checks Green. Operator visual check open |
| [012](../tasks/adr-0080-task-012-language-frame.md) | The language in `TLAN` | ADR 0080 Decision 13 | None | Ready 2026-11-04 |
| [013](../tasks/adr-0080-task-013-mp4-date-shares-one-value.md) | An MP4 date settles after one write | ADR 0080 Decision 6 | None | Implemented 2026-11-05. Mechanical checks Green. Operator visual check open. |

## Sequence

Packet 001 goes first. It owns the writer, the removal of app values, and the compare. Packet 002 then changes only the MusicBrainz source of URL values.

Until packet 002 is complete, packet 001 keeps each MusicBrainz URL value that the present code writes. Decision 6 does not permit the removal of a MusicBrainz value.

Packets 006 to 008 and 013 are corrections, so they go first. Packets 009 to 012 add frames. Packets 009 to 013 each add a difference to "Update n files" or remove one, so run them before the next rewrite of the real Library. Then one rewrite settles each file.

[ADR 0088](../adr/0088-a-feed-move-follows-the-rss-tag.md) owns the feed move, and its [task 001](../tasks/adr-0088-task-001-rss-check-follows-a-move.md) waits for the operator. [Open request 12](v4vmm-open-requests.md) asks Stophammer for `season` and for the license. No packet waits for that request.

## Recorded Facts - 2026-11-04

The tag review of 2026-11-04 compared Studio at `79fff94`, Stophammer at `61e8762` and v4vmm at `74aab71`. The live API gave release 0.18.1.

| Gap | Owner |
|---|---|
| `podcast:funding` reaches no file | Packet 009 |
| `podcast:license` reaches no file | Packet 010 |
| `podcast:season` reaches no file | ADR 0080 Decision 14 defers it |
| `TPUB` has no RSS label, and the compare shows the owner name | Packet 011 |
| `itunes:new-feed-url` has no reader | ADR 0088 |
| The readers give different track numbers and different artwork | Packet 008 |
| Credit roles select incorrect frames | Packet 007 |
| The stored contract is release 0.13.0 | Packet 006 |
| The MP4 date difference has no owner | Packet 013 |
| Language has no row | Packet 012 |
| Explicit, `<copyright>`, `itunes:keywords` and the credit attributes reach no file | No packet. ADR 0080 "Consequences" |

## Recorded Facts - 2026-09-29

- `release_url_values` in `src/musicbrainz.rs` gives each release URL relation as one text value, `<relation type> (<target type>, <direction>): <url>`.
  `musicbrainz_value_for_field` in `src/metadata.rs` puts all of them in the "Website" row, and thus in `WOAR`.
- The lookup requests `url-rels` of a release only. On 2026-09-29 a release lookup gave no official homepage. The release group holds it, and `release-group-level-rels` in the same lookup gives it. Packet 002 records the evidence.
- `changed_frames` in `src/application/queries/tag_update.rs` counts a frame as equal when any file value matches the expected value. A file with extra stale values in the same frame shows no difference.

## Open Defect - 2026-10-03

- An MP4 file has the same defect as packet 003 corrected for Vorbis. The writer stores `TDRC` and `TYER` in two freeform atoms, and the reader gives both values as `TDRC`.
  Thus the scan always reports `TYER` on an MP4 file with a date. The reader label does not name the atom, so a key comparison cannot correct it. Packet 013 owns this defect.
- Operator observation of 2026-10-16: the "Update 2 files" list showed two files, "Ethereal Realm ( 528hz )" and "Whispers". Each showed one frame: `TYER: 2025 (file: 2025-10-19)` and `TYER: 2025 (file: 2025-10-03)`. The operator wrote no tag.
  It is not confirmed that the two files are MP4 files, or that a write changes the stored date.

## Open Finding - 2026-11-05

- Packet 007 moves a credit to its correct frame. A file that the app wrote earlier can hold the same name in the frame of the earlier mapping: `remixer` in `TIPL`, and `original lyricist` in `TEXT`.
  A write replaces a frame only when its edits name that frame. So the earlier value stays when the track has no other credit for that frame. The scan shows no difference for it. ADR 0080 Decision 6 does not list this value as an earlier form, and no packet owns it.

## Session Rules

Each implementation session owns one packet. It completes the packet, checks it, and stops.
A packet that changes user-visible behavior leaves its visual gate open in its `Status:` line and in
[pending human checks](../pending-human-checks.md).
