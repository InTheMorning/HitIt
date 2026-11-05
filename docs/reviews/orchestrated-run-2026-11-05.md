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

## Steps

Each entry names the packet, the result and the commit.

1. Packet 006: implemented and Green on 2026-11-05, with no stop. The contract is release 0.18.1, revision `61e8762`. The agent found no removal and no rename. `url_scheme_added` was in the 0.13.0 contract, so 0.18.1 adds only `label` to the source link schema. The agent renamed the contract version test to `adr_0084_scopes_r16_03_contract_fixture_is_0_18_1`. 2590 lib tests and 398 guards pass.
2. Packet 007: implemented and Green on 2026-11-05, with no stop. One classifier gives the frame and the normalized role. `remixer` gives `TPE4`, `original artist` gives `TOPE` and `original lyricist` gives `TOLY`. Of the 70 Studio roles, only `foley artist` changes: it now gives no frame. The orchestrator added the compare round-trip test. 2595 lib tests and 398 guards pass.
3. Packet 008: implemented and Green on 2026-11-05, with no stop. Each reader gives an artwork `podcast:image` first for a channel and for an item, and `podcast:episode` then `itunes:episode` for a track number. A value of `podcast:episode` that is not a number gives way to `itunes:episode`. 2603 lib tests and 398 guards pass.
4. Packet 013: implemented and Green on 2026-11-05, with no stop. The two new tests failed before the change, so the defect is confirmed on an MP4 file. A write stores one date atom, and the scan treats `TDRC` and `TYER` as one value. The agent added the fixture `docs/runbooks/fixtures/conversion.m4a`. 2605 lib tests and 398 guards pass.

## Final Walk

Not started.
