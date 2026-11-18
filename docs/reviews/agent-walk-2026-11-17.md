# Agent Walk Of The Open Checks - 2026-11-17

Status: record of an agent walk. This walk closes no gate. Each visual gate stays open until a person walks it.
[Pending human checks](../pending-human-checks.md) owns each check, and this record gives the agent results.

## Method

- Binary: the release build of HEAD `ab0a9cc`.
- Four walker agents used four Xvfb displays (:78 to :81) with openbox. Each walker had its own scratch
  `HOME`, `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_CACHE_HOME` and music folder. No walker touched the
  Library of the operator.
- The app calls had `allowed_domains: ["api.musicindex.org"]`. Other hosts were not declared.
- The walker reports and all screenshots were in the scratch folder of the session. This folder keeps
  only the screenshots of the defects, in [screenshots/agent-walk-2026-11-17](screenshots/agent-walk-2026-11-17/).
- The main agent read each report and examined the screenshots of each Fail and each new defect.

## Download Fault In The Sandbox

In the app, each new feed fetch from `wavlake.com` failed with "HTTP status client error (403 Forbidden)".
Two walkers saw it, for the feeds `d08fd0e0-ec86-4d75-bc1d-39dff737fdf9` and
`c4c5ebf1-4bdd-4ace-b155-56dcef474dfc`. One earlier Wavlake download ("Waltz Tunes Volume 2") passed.

In the same sandbox, `curl` and a small program with `reqwest` 0.13 and the User-Agent `hitit/0.1.0` got
HTTP 200 for both URLs. The test also got 200 in a background call with the same `allowed_domains`.
`fetch_feed_document` in `src/rss/subscribe.rs` sends the same request. The cause is not known.
Inference: a host-side block of repeated requests from this network. This is not proven.

Each check that needs a download stays open for the operator desktop. If a download also fails there with
HTTP 403, it is an app defect.

## Defects Found

1. **ADR 0089 task 015 V3 fails.** A long value in the Inspect Sources sheet does not wrap. "Album description"
   ends in an ellipsis on one line. Screenshot: `adr-0089-015-v3-sheet-ellipsis.png`. The main agent
   confirmed it.
2. **At XL and 390 px the page title wraps one letter on each line.** The labels of the tab bar overlap.
   At 440 px the page fits. ADR 0097 sets 390 px as the narrowest width. Screenshot:
   `adr-0101-001-v4-xl-390.png`. The main agent confirmed it. The operator decides if XL at 390 px must fit.
3. **At 480 px the toolbar title shows "..".** Screenshot: `toolbar-title-480.png`.
4. **Below 940 px the More menu has no "Check All Feeds" and no feed state.** The
   [shell run record](orchestrated-run-2026-11-05-shell.md) says that the More menu holds them. Screenshot:
   `adr-0089-013-more-menu.png`.
5. **The packet text is out of date in three places.** ADR 0087 packet 010 step 7 says 11 Down presses for
   `has:`, but 12 are necessary, and test R87-10-02 uses 12. ADR 0087 packet 004 says the field list has
   11 rows, but it has 12. ADR 0087 packet 005 says the mark is "MusicIndex", but the screen shows "Music Index".
6. **Small findings for the operator to judge:**
   - The toolbar field keeps the last query on other pages and after Back.
   - The delete dialog of a saved search names the query, not the row name.
   - "In Library" shows no count, although ADR 0089 packet 004 step 9 says that it does.
   - Show and Settings have no toolbar title.
   - Settings has no Back or Forward button.
   - At 900 px the letter bar uses two lines.
   - The Settings "Check" menu is on a different line from the title at XL and 440 px (section 25 V5).
   - A gpui-component field keeps a corner of about 4 px. This is the known open point of ADR 0101 task 001.

## Results By Section

Pass means that the agent saw the expected screen. Superseded means that a later decision removed the screen of
the check.

| Section | Pass | Superseded | Open for the agent | Open for a person |
| --- | --- | --- | --- | --- |
| 6 | 0086-002 V4, V6, V8; 0086-003 V4 to V7 (List); 0086-004 V4, V9, V10; 0086-005 V3; 0087-002 V6, V8; 0087-003 V2 (rows), V8; 0087-004 V9; 0087-005 V5 | 0086-004 V2; 0087-002 V2 to V4, V9; 0087-003 V5 (ADR 0098, archived ADR 0087 task 011) | Light and narrow widths of most checks; 0087-004 V7, V8 | Downloads: 0086-003 V9, 0086-004 V6, 0086-005 V1, V2, V4 to V7, 0087-002 V5, 0087-005 V1. Timing and flicker: 0086-001 V1, V2, V4, 0086-003 V1, V8, 0087-003 V2. Network cut: 0086-003 V10. Bad endpoint: 0087-005 V6 |
| 7 | 0059-019 V1 to V3 (local fixture); 0087-007 V4 through 0087-009 V1 | - | - | Downloads: 0084-035 V2, V3. A real show feed: 0059-019 step 8 and V4 |
| 8 | - | - | - | All. Downloads, and 038 V1 needs feeds 109 to 128 of the operator database |
| 9 | 0087-009 V1, V2; 0087-010 V1, V2, V4, V5 | - | - | 0087-010 V3 and the Library part of 0087-009 V1 (Library names) |
| 11 | 001 V1, V2; 002 V1, V3; 003 V1, V2; 004 V1, V2; 005 V1 to V4; 006 V1, V3, V4; 007 V1; 008 V1 to V4; 009 V1, V2; 010 V1; 011 V1; 013 V1 to V3; 014 V1, V4; 015 V1, V4. **Fail: 015 V3** | - | 002 V2; 003 V3; 004 V3; 006 V2 (click); 007 V2, V4; 009 V3; 014 V2; the search field list, the Tab order, the letter bar Tab stop, the pinned list bar | 004 V4 and 014 V3 (a failed download); 005 V5 (smooth scroll); 007 V3 and 012 V1 to V4 (Inbox items); 010 V2, V3 (feed hosts); 011 V2, V3 (a live item, a browser); 015 V2 (a hardware keyboard) |
| 22 | - | - | - | V2: a Library with many releases |
| 25 | V1 (row, 1440), V2, V3 (HitIt radii), V4 (1440 and 440) in Light | - | V1 tile and dialog button at 390 and 940 in Light | V3 field corner; V4 XL at 390 px (defect 2); V5 layout |
| 26 | V2 in Light | - | - | - |
| 28 | V3 | - | - | V1 the day in the time zone of the operator |
| 29 | - | - | - | V1 (a local list has no loading rows); V3 (several releases); V4 (desktop settings) |
| 30 | - | - | V3 (an animated cover), V4 (an item with no image) | V1 compare with musicindex.org; V6 timing |
| 32 | V1, V2 in both high-contrast profiles; V3 sidebar in both, row in High Contrast Dark | - | V3 row in High Contrast Light | - |
| 33 | - | - | V2 (the screenshot does not show the row of the moved file) | - |

## Questions Of The Shell Run Record

The walker gave evidence for the questions of the [shell run record](orchestrated-run-2026-11-05-shell.md#questions-for-the-final-walk).
The operator answers them.

- Q1: a list page shows two titles, one in the toolbar and one in the title row.
- Q3: the results page shows the query as its title. A saved search also shows a gray bar with the query.
- Q4: a public playlist tile has no menu. A row has "⋯". The page menu holds the copy.
- Q5: the copy showed "The app could not read a track of this playlist, so it made no playlist."
  The sandbox blocked the feed hosts, so this is probably the cause.
- Q6: see defect 4. The Inbox was empty, so the tag update action was not seen.
- Q7: not walked.
