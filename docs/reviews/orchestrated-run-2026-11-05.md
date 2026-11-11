# Orchestrated Run Of 2026-11-05

## Status

Open - 2026-11-05. The run implemented [ADR 0080 packets 006 to 013](../plans/adr-0080-tag-frames-phase-plan.md) and [ADR 0076 packet 011](../tasks/adr-0076-task-011-new-reader-reads-each-feed-once.md). Each packet is committed. The final walk is in progress, and four points wait for the operator.

This record is advisory. It states no rule. Each rule stays in its ADR or its packet.

## Operator Decisions - 2026-11-05

- The run starts now, although sections 8 and 9 of [pending human checks](../pending-human-checks.md) are open. The tag packets change the RSS readers, metadata and the tag writer. Those two sections cover the download queue, the singles group and the search field list. The two sections stay open.
- The run leaves out [ADR 0088](../adr/0088-a-feed-move-follows-musicindex.md). It stays Proposed, and its packet waits.
- The orchestrator commits the planning documents in v4vmm and the open requests copy in Stophammer. Then it commits each reviewed, Green packet as HeyCitizen. It never pushes.
- An RSS funding link replaces each other `WPAY` value. ADR 0080 Decision 10 states this limit of Decision 6.
- The visual checks of the run come last, in one final walk.

## Order

The [phase plan](../plans/adr-0080-tag-frames-phase-plan.md#sequence) puts the corrections first. Each packet starts after the commit of the packet before it, because the packets change the same files.

1. [Packet 006](../tasks/archive/adr-0080-task-006-contract-0-18-1-and-link-label.md): the contract of release 0.18.1 and the link label.
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
- Operator checks, the tag reader: a test on 2026-11-05 showed that `ffprobe` does not show `WPAY`, `WCOP` or `TLAN` of an MP3 file. The orchestrator changed the steps of packets 007, 009, 010, 011 and 012 to `exiftool -a -G1 -s FILE`, which shows each frame on MP3, and each key on FLAC and MP4.
- Packet 013, the operator command: the packet named a column that does not exist. The command now reads `local_files.path`.
- Packet 013, the observation of 2026-10-16: the Library holds no MP4 file, so the MP4 defect did not cause it. The operator confirmed that both named files are FLAC files. The test `adr_0080_shared_key_full_date_only_flac_settles_after_one_write` proves the cause. A FLAC file with only `DATE=2025-10-19` matches `TDRC` and not `TYER`, so the scan gives the text that the operator saw. One write then stores both values, and the file settles. Thus this is not a defect. The other file with `TYER: 2026 (file: 2026-03-09)` has no confirmed format.
- Operator checks, the scope of a write: the orchestrator first told the operator to select a small playlist for a write. That was incorrect, because a write includes each listed file. The orchestrator corrected the walk on 2026-11-05.
- Packet 006: the orchestrator moved it to the archive, because it has no visual check.

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
9. ADR 0076 packet 011: implemented and Green on 2026-11-05, with no stop. The operator asked for this packet after the popup results. A feed with a lower or missing RSS reader revision gets one request with no validators. The value is an `rss` fact of the feed, so the schema stays at version 21. 2670 lib tests and 398 guards pass.
10. ADR 0080 packet 015: implemented and Green on 2026-11-05, with no stop. Commit `402300a` kept a `TPUB` value from another source. The operator changed this rule, and the next commit removes that code. A write replaces `TPUB` and `TLAN`.
11. ADR 0080 packet 016: implemented and Green on 2026-11-05, with no stop. The operator decided "overwrite all". Decision 6 changes: a write replaces each other value of a frame that it supplies, and the scan lists such a value. The next scan can list more files.

## Open Points For The Operator

The operator answered points 1, 3 and 4 on 2026-11-05, and each recommendation stands. Point 1: the operator first accepted the recommendation, and then changed the rule on 2026-11-05: a write replaces `TPUB` and `TLAN`, so that a frame does not grow. ADR 0080 is amended, and [packet 015](../tasks/archive/adr-0080-task-015-tpub-replaces-other-values.md) states the rule in tests. Point 3: no code change, and the next full write waits for more playlist checks. Point 4: no change.

1. A write replaces a `TPUB` or `TLAN` value from a different source. ADR 0080 Decision 6 keeps such a value. Recommendation: amend Decision 13, so that `TLAN` holds one language and replaces the value. Keep the `TPUB` value, because Decision 6 names a MusicBrainz label. This needs one new packet for the writer. On 2026-11-05 the operator ran `exiftool` on the 231 Library files: 0 files hold a publisher, label or language value. Thus a write replaces no such value in the Library of that date. The rule stays open for a file from a different tool.
2. A file that the app wrote earlier can keep a `remixer` name in `TIPL`, or an `original lyricist` name in `TEXT`. The [phase plan](../plans/adr-0080-tag-frames-phase-plan.md#open-finding---2026-11-05) records it. On 2026-11-05 the command in Setup step 4 gave 0 rows, so no credit in the Library has this role. No packet is necessary for the Library of that date. Thus no Library track can walk packet 007 V1 with no test credit in Studio.
3. Check RSS reads only a feed with a track in a playlist. The operator has two playlists and 91 feeds. "Check all feeds" reads MusicIndex, not RSS. Thus the license reaches a track only when a playlist holds a track of its feed, or at a new subscribe. ADR 0076 packet 011 corrected the HTTP `304` gap of this path. It does not widen the path. The operator decides if a full rewrite waits for more playlist checks.
4. A later confirmed label has a title only when the Library holds its feed. Recommendation: no change, because a stored title of each named feed needs a new request or a schema change.

## Final Walk

Not started. The "Operator Visual Check" section of each packet gives the steps and the cleanup. Section 10 of [pending human checks](../pending-human-checks.md) lists each check.

### Setup

The run adds no migration and no configuration key, so no backup is necessary. The "Update n files" button is in the Music sidebar, next to "Check all feeds". It lists each Library file that differs, and "Write Tags" writes each listed file. It is not limited to a playlist, and it has no selection for each file. Thus do not click "Write Tags" until more playlist checks are done (open point 3). The scan runs by itself at start, after an RSS check and after a write.

1. Open a terminal in a desktop session.
2. Build both binaries: `cd ~/build/v4vmm && cargo build --release && cargo build --bin v4vmm`.
3. Find the format of the two files of the observation of 2026-10-16. Run this command, and then run `exiftool -FileType FILE` on each path:
   `sqlite3 -readonly ~/.local/share/v4vmm/v4vmm.sqlite "SELECT t.track_title, lf.path FROM local_files lf JOIN tracks t ON t.id = lf.track_id WHERE t.track_title LIKE '%Ethereal Realm%' OR t.track_title LIKE '%Whispers%'"`
4. Count the credits of open point 2:
   `sqlite3 -readonly ~/.local/share/v4vmm/v4vmm.sqlite "SELECT lower(role), count(*) FROM entity_contributors WHERE lower(role) LIKE '%remix%' OR lower(role) LIKE '%original lyric%' GROUP BY 1"`
5. Start the app with `./target/release/v4vmm`. Run the RSS check of each playlist that you use, because the funding, license and label values reach the Library at that check.

### Results Of The Popup - 2026-11-05

The operator ran the scenario `tagpopup` of `~/.claude/scripts/v4vmm-shots.sh` on a list of 144 files. These results cover only the popup. No person has clicked "Write Tags".

- Packet 012: each line that the orchestrator read shows `TLAN: eng (file: no value)`.
- Packet 009: `WPAY` lines hold plain URLs with no label text.
- Packet 011: `TPUB: HeyCitizen` shows on two albums. `feed_publisher_relationships` holds two rows for the HeyCitizen publisher feed, feeds 2 and 3. Each stored row has `label` in `role` and `both` in `role_agreement`, and its `observed_at` is 2026-10-03.
- Packet 011, correction: the orchestrator first called this value incorrect. On 2026-11-05 the operator stated that HeyCitizen is the artist, the label, the publisher and the hosting. Thus `TPUB: HeyCitizen` is correct. The live feeds state only `rel="artist"`, so the stored rows and the feeds do not agree.
- The track page reads the live response. Thus "The Platform" shows an empty "Label" row, and this is correct. The scan reads the stored rows.
- At 2026-11-05 16:45:49 UTC the app report shows a tag update of 143 files, and 1 file not written. The button then shows "Update 3 files". Thus the files of the two albums hold this value.
- The next scan listed "To be in Nashville" and "The Wait Is Over" with `file: no value` for frames that `exiftool` shows in the files. [Packet 014](../tasks/adr-0080-task-014-reader-keeps-each-frame.md) corrects this reader fault. A write with the fault can remove frames. No source holds a frame that it removed, so the count of such frames is not known.
- `changed_frames` reads only the expected frames. When the stored rows give no label, the scan shows no difference for `TPUB`, and a write does not remove the value.
- Packet 011 V2 passes on "Stay Awhile": the "Label" row shows no RSS value. V1 stays open.
- `TYER: 2025 (file: 2025-10-19)` shows on "Ethereal Realm ( 528hz )". "Whispers" has no `TYER` line.
- "The Heycitizen Experience" tracks show a Nostr key in the file that differs from the feed key. The operator confirmed that `npub109pc…5aaj` is the correct key, and the live feed holds only this key. Thus this replacement is correct.
- The live feed `Lofi-Experience/lofi.xml` holds the incorrect key `npub12um9…f5ck` 7 times. The operator's correction is not in this feed.

After ADR 0076 packet 011, restart the app, so that it uses the new binary.

### Results Of ADR 0076 Packet 011 - 2026-11-05

- V1, feed 49, "Chatti und Claude". The report of 14:43:49 UTC shows `Not modified: 1`.
- The database holds `rss_reader_revision` with the value 1 for this feed. Thus an earlier check applied its document.
- The orchestrator did not see the report of the first check.
- V1, "Stay Awhile" in "packet 025 test". The report of 14:45:22 UTC shows HTTP 200 and `Documents received: 1`. The report of 14:45:37 UTC shows HTTP 304 and `Not modified: 1`.
- V1 passes on the two feeds. The first report of "Chatti und Claude" stays unseen.
- V2 is open. The popup of 09:46 local time shows only `TLAN` for the tracks of the two checked feeds, and no license line. It is not known if these feeds have `podcast:license`.
- The screenshots of that popup have gaps, so they do not prove that no file in the list has a license line.

### Results After The Write And Packet 014 - 2026-11-05

The operator started the release binary with `id3` 1.17.1, and opened "Update n files".

- Packet 014 V1 passes. The popup lists 1 file, "Tiddies In My Face (Single)". It does not list "To be in Nashville" or "The Wait Is Over". The earlier list had 3 files.
- That file shows "In use by the show" and `TLAN: eng (file: no value)`. Thus the app correctly did not write it.
- Packet 012 V1 passes. `exiftool` shows `eng` in the 143 written files.
- Packet 009 V1 passes, because the popup listed `WPAY` with plain URLs. The second part of V2 passes: the scan shows no difference for the written files. The `exiftool` part of V2 is open.
- This scan does not prove `TPUB`, because the file and the stored rows give the same value.
- Packet 014 has a known limit: a write with `id3` 1.16.4 possibly removed frames that no source supplies. The scan cannot find them.
- Observation: the popup title is "Update 1 file?", its text says 0 files, and "Write Tags" stays available. The orchestrator did not read what a click does in this condition. No packet owns this.
- Observation: `push_track_metadata_row` at `src/metadata.rs:1397` gives the full date to the "Release year" row of the track page. The compare row at `src/metadata.rs:1651` uses `release_year_from_value`. The popup line `TYER: 2025` holds a year, so the file value is not affected. This is a display defect, and no packet owns it.

### Results Of The File, License And Album Page Checks - 2026-11-05

- Packet 009 V2 passes on MP3 files. `exiftool` shows `PaymentURL` with a plain URL in 33 files: 15 with `https://phafe.com/mattfinlay`, and 18 with the PayPal URL. No FLAC file shows a payment tag. The orchestrator did not read if a FLAC track has a feed with a funding link.
- Packet 010 V1 and V2 and ADR 0076 packet 011 V2 stay open. The Library holds 0 license facts, and the two checked feeds hold 0 `podcast:license` lines. These results show no defect. These checks need a feed with `podcast:license`.
- Album page test: the operator opened the album page. The rows of feeds 2 and 3 stay at `artist, author, host, label, producer`, with `observed_at` on 2026-10-03.
- Cause, from the code. `hydrate_album_identity_on_view` in `src/library/app_impl.rs:2465` sends no request when an album has a website, a Nostr key, a description and metadata facts. Thus an open of such a page does not refresh its rows, and a restart does not change this. The orchestrator did not read the facts of feed 2, so this cause is not proved on that row.
- `apply_feed_updates` in `src/feed_service.rs:482` requests `publisher`, and writes no relationship row. It runs only for a feed that MusicIndex reports as changed. The once-per-session pass reads only a feed with no stored row.
- On 2026-11-05 the live API gives `role: artist` with the same relationship key for the two feeds. Thus one new write corrects each stored row.
- The orchestrator's earlier statement, that an album page open refreshes the rows, did not give this condition.
- Role sequence. musicindex ADR 0036 R-ROLE-1 gives the sequence artist, label, hosting, producer, host, author, network, sponsor. ADR 0033 R-WORD-3 item 1 gives the label "Artist" to a feed with the role artist, also when it has other roles. ADR 0011 item 4 permits more than one role on a feed.
- These rules own the word and the dot. No document in the sister repositories names `TPUB`. v4vmm ADR 0080 Decision 12 and `role_contains_label` accept `label` at each position in the set. Thus a feed with the set artist and label gives a `TPUB` value. The operator decides if `TPUB` follows the sequence.
- Operator decision, 2026-11-05: HeyCitizen is the artist, the label, the publisher and the hosting. Thus Decision 12 stays, and no file removal is necessary. The publisher feed proves the artist role: its 4 music items and the two album links hold `rel="artist"`.
- Consequence: RSS is the source, and the live feeds do not state `label`. A new write of the stored rows gives `artist` only, and the "Label" row of the track page stays empty. Before a packet refreshes the rows, the feeds must state each role, on the two sides of each link. The operator owns this change in Studio.
- Packet 011 V1: the file holds `TPUB` from a confirmed label row, and the operator confirmed the value. The orchestrator counts V1 as passed on HeyCitizen, not on Sir Libre Records.

### Scope Of The Scan - 2026-11-05

- `local_files` holds 231 files. 208 have `is_in_library` 1, and 23 have `is_in_library` 0. The sidebar count and the scan read only the 208 files (`db::library_tracks`).
- A sample of the 23 files is under `artists/`, as the Library files are. No "Update n files" write reaches these files. The operator decides if this needs a change.
- After packet 016, the button shows "Update 1 file". The operator did not yet confirm the start time of the running binary.

### Order

1. Packet 013 V1. On 2026-11-05 the operator reported that the Library holds 156 `.mp3` files, 75 `.flac` files and no MP4 file. No real file can walk this check, so it stays open.
2. Packet 008 V1. It needs a feed that you host.
3. Packet 012 V1.
4. Packet 009 V1 and V2.
5. Packet 010 V1 and V2. It needs a change to a test feed in Studio.
6. Packet 011 V1 and V2. Also read the "Label" row of the Library track page before the scan, and report an empty row for an album with a confirmed label.
7. Packet 007 V1.
8. ADR 0076 packet 011 V1 and V2.
