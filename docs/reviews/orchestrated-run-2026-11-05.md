# Orchestrated Run Of 2026-11-05

## Status

Open - 2026-11-05. The run implements [ADR 0080 packets 006 to 013](../plans/adr-0080-tag-frames-phase-plan.md). The final walk comes after the last packet.

This record is advisory. It states no rule. Each rule stays in its ADR or its packet.

## Operator Decisions - 2026-11-05

- The run starts now, although sections 8 and 9 of [pending human checks](../pending-human-checks.md) are open. The tag packets change the RSS readers, metadata and the tag writer. Those two sections cover the download queue, the singles group and the search field list. The two sections stay open.
- The run leaves out [ADR 0088](../adr/0088-a-feed-move-follows-the-rss-tag.md). It stays Proposed, and its packet waits.
- The orchestrator commits the planning documents in v4vmm and the open requests copy in Stophammer. Then it commits each reviewed, Green packet as HeyCitizen. It never pushes.
- An RSS funding link replaces each other `WPAY` value. ADR 0080 Decision 10 states this limit of Decision 6.
- The visual checks of the run come last, in one final walk.

## Order

The [phase plan](../plans/adr-0080-tag-frames-phase-plan.md#sequence) puts the corrections first. Each packet starts after the commit of the packet before it, because the packets change the same files.

1. [Packet 006](../tasks/adr-0080-task-006-contract-0-18-1-and-link-label.md): the contract of release 0.18.1 and the link label.
2. [Packet 007](../tasks/adr-0080-task-007-contributor-role-frames.md): a credit role selects the correct frame.
3. [Packet 008](../tasks/adr-0080-task-008-rss-readers-agree.md): the RSS readers agree.
4. [Packet 013](../tasks/adr-0080-task-013-mp4-date-shares-one-value.md): an MP4 date settles after one write.
5. [Packet 009](../tasks/adr-0080-task-009-funding-link-frame.md): the funding link in `WPAY`.
6. [Packet 010](../tasks/adr-0080-task-010-license-frames.md): the RSS license in a file.
7. [Packet 011](../tasks/adr-0080-task-011-confirmed-label-frame.md): a confirmed label in `TPUB`.
8. [Packet 012](../tasks/adr-0080-task-012-language-frame.md): the language in `TLAN`.

## Orchestrator Decisions

Each entry names the step, the decision and the reason.

- Packet 007, stale credit values: the orchestrator did not expand the packet. A file that the app wrote earlier can keep a `remixer` name in `TIPL` or an `original lyricist` name in `TEXT`. A correction needs a new removal path and a new scan rule, and Decision 6 does not name this value. The phase plan records the finding, and packet 007 V1 names it.
- Packet 007, classifier order: the agent reported two results outside the Studio list. `mixing` now gives `TIPL`, and `remix artist` now gives `TPE4`. The orchestrator accepted both, because the normalized role already read them so.
- Packet 013, the observation of 2026-10-16: before the change, the scan of an MP4 file gave `TYER` with no file value. The operator saw `TYER: 2025 (file: 2025-10-19)`, which has a file value. Thus the two files may not be MP4 files, and this packet may not correct them. The final walk asks for their format.
- Packet 010, row name: the RSS row is "License URL", not "License". A row with the name "License" would share its row ID with the MusicBrainz row, and it would change the value of that row.
- Packet 010, a MusicBrainz `WCOP` write: it now uses the same resolver as RSS. A MusicBrainz `WCOP` over a different file value moves the two URLs into `TXXX:LICENSE`. Before, it replaced the file value. Decision 6 requires this result.
- Packet 010, the guard `adr_0075_projection_views_combine_no_fact_and_column` forbids a fact read in `src/rss/check_apply.rs`. The agent removed its read and did not change the guard. The check thus writes the two license facts at each pass, with equal values.
- Packet 011, a foreign `TPUB` value: a write replaces it on MP3, FLAC and MP4. Decision 6 keeps a value from another tool, so this result conflicts with it. A keep rule needs new writer logic, and the app stores no MusicBrainz lookup result. The orchestrator did not decide this. The operator decides it after the run. The test `adr_0080_label_write_over_a_foreign_tpub_value_on_mp3_flac_and_mp4` records the present result.
- Packet 011, a second label: the stored title belongs to the first named feed only. A later confirmed label has a title only when the Library holds its feed, so a label that the Library does not hold gives no `TPUB` value. The packet forbids a new request.
- Packet 012, a foreign `TLAN` value: a write replaces it on MP3, FLAC and MP4. This is the same open point as `TPUB`. The test `adr_0080_language_write_over_a_foreign_tlan_value_on_mp3_flac_and_mp4` records the present result.

## Steps

Each entry names the packet, the result and the commit.

1. Packet 006: implemented and Green on 2026-11-05, with no stop. The contract is release 0.18.1, revision `61e8762`. The agent found no removal and no rename. `url_scheme_added` was in the 0.13.0 contract, so 0.18.1 adds only `label` to the source link schema. The agent renamed the contract version test to `adr_0084_scopes_r16_03_contract_fixture_is_0_18_1`. 2590 lib tests and 398 guards pass.
2. Packet 007: implemented and Green on 2026-11-05, with no stop. One classifier gives the frame and the normalized role. `remixer` gives `TPE4`, `original artist` gives `TOPE` and `original lyricist` gives `TOLY`. Of the 70 Studio roles, only `foley artist` changes: it now gives no frame. The orchestrator added the compare round-trip test. 2595 lib tests and 398 guards pass.
3. Packet 008: implemented and Green on 2026-11-05, with no stop. Each reader gives an artwork `podcast:image` first for a channel and for an item, and `podcast:episode` then `itunes:episode` for a track number. A value of `podcast:episode` that is not a number gives way to `itunes:episode`. 2603 lib tests and 398 guards pass.
4. Packet 013: implemented and Green on 2026-11-05, with no stop. The two new tests failed before the change, so the defect is confirmed on an MP4 file. A write stores one date atom, and the scan treats `TDRC` and `TYER` as one value. The agent added the fixture `docs/runbooks/fixtures/conversion.m4a`. 2605 lib tests and 398 guards pass.
5. Packet 009: implemented and Green on 2026-11-05, with no stop. Each reader and the RSS check store the funding rows of a channel and of an item. The file holds one resolved link in `WPAY`. The writer needed no change, because a test on a real file shows that `add_frame` replaces each earlier `WPAY`. The RSS check records no difference for a funding row, because a check row would show funding on a screen, and the packet forbids that. The next tag scan lists a changed `WPAY`. 2623 lib tests and 398 guards pass.
6. Packet 010: implemented and Green on 2026-11-05, after one guard failure that the agent corrected. The RSS license is stored as two local facts. Its URL goes into `WCOP`, or with a different URL into `TXXX:LICENSE`, and its text goes into `TXXX:MusicIndex License`. Real-file tests pass on MP3, FLAC and MP4. Schema version 21 stays. 2642 lib tests and 398 guards pass.
7. Packet 011: implemented and Green on 2026-11-05, with no stop. `TPUB` holds the titles of the confirmed labels, from `feed_publisher_relationships` for a Library context and from the `publisher` entries for an Index context. The owner name stays in `TXXX:V4V_PUBLISHER`. The agent corrected the Vorbis cleanup, because each FLAC write added a `TPUB` value. 2652 lib tests and 398 guards pass.
8. Packet 012: implemented and Green on 2026-11-05, with no stop. `TLAN` holds a three-letter code, from a table of 183 codes that the agent generated from `/usr/share/iso-codes/json/iso_639-2.json`. A Library track has no stored item language, so a Library write uses the channel code. The Library context reads the channel language from the stored value projection. 2664 lib tests and 398 guards pass.

## Final Walk

Not started.
