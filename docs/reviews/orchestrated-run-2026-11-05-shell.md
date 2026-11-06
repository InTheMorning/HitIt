# Orchestrated Run Of 2026-11-05: The Shell

## Status

Open - 2026-11-06. The run implemented the [ADR 0089 phase plan](../plans/adr-0089-shell-phase-plan.md), packets 001 to 015, and each packet is committed. Packet 016 waits for the operator decision on ADR 0068. The final walk is open.

This record is advisory. It states no rule. Each rule stays in its ADR or its packet.

## Operator Decisions - 2026-11-05

- The operator asked for the phase plan and the packets, then for the orchestration, with a commit between packets.
- The run starts although sections 8 and 9 of [pending human checks](../pending-human-checks.md) are open. The orchestrator reads the request as this decision. The two sections stay open.

## Orchestrator Decisions

Each entry names the step, the decision and the reason.

- Plan: the orchestrator commits the uncommitted ADR 0089 documents with the plan, as one commit, because each packet builds on them.
- Plan: "the show playlist" is the source playlist of the active playback session. The app has no other value with this meaning. The Show cuelist is projected from that playlist.
- Plan: the row action Play moves from packet 014 to packet 016. Music has no Play action, and ADR 0068 owns Music playback.
- Plan: a removed release needs no state. A release that is not in the Library shows only in All.
- Plan: the failed state of a release is session state, from the outcomes of the download queue.
- Plan: open points of ADR 0089 keep their present behavior, as the phase plan lists.
- Plan: the toolbar and the title row of a list page each show the title, because R-TOOLBAR-1 and R-LIST-1 each ask for it. The final walk asks the operator about this.
- Packet 001, a page with no content: a Library page with no loaded data and no load shows its list root, and a load shows the skeleton. The stack entry stays. A reload with `ResetDetail` thus shows the list root under a page entry, and Back then stays on that list. The final walk checks this.
- Packet 001, focus: the visible frame takes the keyboard focus when it changes. The toolbar search field keeps the focus for a list root, because a live run changes the list root while the curator types. A page takes the focus from the field.
- Packet 001, Discover: "New releases" opens as a page, and Back shows the Discover views list. Packet 006 removes this result.
- Packet 002, Back at the root of a path: `go_back` with no earlier entry moves to the root of the path. This holds R-NAV-2 item 3. Back is thus enabled on each page over a root.
- Packet 002, scroll: the list scroll stays in long-lived view state (`ListState` or `ScrollHandle`), not in the navigation entry. The packet added handles for My playlists and the Discover views. The mechanical criterion "restored scroll" thus has no model test, and the final walk checks it.
- Packet 002, marks: a Library track, a name match and an artist by name in search results get no mark, because the row does not hold the data of its page entry. Needs attention rows get no mark, because packet 012 replaces them.
- Packet 002, Show and Settings: their frames lost Back and Forward, and the toolbar shows each control only when its history exists.
- Packet 003, frames: `SourceList` goes. `Detail` stays, because a removal changes `MusicColumns` and the shell widths. A later packet removes it when it is small.
- Packet 003, icons: Show uses Lucide `RadioTower`, and Settings uses Lucide `Settings`.
- Packet 003, keyboard: the Show and Settings rows take a click and the section shortcuts, as the old tabs did. A sidebar row has no Enter key action.
- Packet 004, My playlists: the page has no sort, and its rows stay in name sequence. Decision 5 says that the curator sets the sequence. A sequence needs stored positions and a move control, and ADR 0089 puts the moves of My playlists in Non-Goals. The final walk asks the operator which holds.
- Packet 004, view icons: no page has two views yet, so the icons are absent until packet 005.
- Packet 004, the In Library count: it counts each release that the scope holds, so a singles group of five counts five.
- Packet 005: the orchestrator divided it into Parts A to D, because the first agent stopped on its size before any edit.
- Packet 005, Artists: Artists has one sort, A to Z. The packet states the evidence.
- Packet 005 Part A: a first letter outside ASCII counts as "#". A later rule for sort keys can change this.
- Packet 005 Part B: a Library release older than the loaded recent page sits at the end of Newest in All, and a later page goes before it. The merge sorts by date at each page, so the sequence stays correct.
- Packet 005 Part B: the window group page of Releases rebuilds its layout at each render, as the page of a saved search does. Nobody measured it on a large list, and the final walk checks it.
- Packet 005 Part C: the tile grid of Releases is not a virtual list, as the Discover grid was not. Item 33 of the responsiveness list records this.
- Packet 005 Part D, tests: the first pass deleted the tests of ADR 0086 rules that stay. The orchestrator sent the work back, and a second agent ported them to the Releases path. The unit count went from 2682 to 2665. The rest belong to the deleted view.
- Packet 005 Part D, open finding: a window group page reads the rows that its key held when it opened. A fresh load of the list does not empty an open page (ADR 0086 packet 004 rule r17). No packet owns this. The phase plan records it.
- Packet 005 Part D: `set_window_length` had no production caller, so the agent deleted it. [ADR 0086 task 006](../tasks/adr-0086-task-006-window-length-setting.md) owns the setting.
- Packet 005 Part D: the Index always gives a key from the GUID or the entity ID, so the old rule on an empty GUID has no test. Tile focus has Up, Down, Home and End in the model. The renderer owns Left and Right.
- Packet 006, examples: the "How to Search" examples come from `QUERY_FIELDS`, because a guard of ADR 0087 rejects field text outside the parser.
- Packet 006, delete: the delete action stays on the page of a saved search. Packet 007 adds the item menu of a row with each of its four items, and it makes "New Saved Search" a button.
- Packet 007: the orchestrator divided it into Part A (migration 22, rename, duplicate, the menu and the button), Part B (the query field) and Part C (the seen state and the counts). The agent stopped early on the whole packet.
- Packet 007 Part A: the unique index on the query stays, so a duplicate holds the query "<query> copy". Part B opens the query field after a duplicate, so the curator changes it. A name is not unique, and a blank name clears the name.
- Packet 007 Part C: a marker row records the first run, so a first run with no results still counts later results. An actor pass that started before an open can briefly show an old count, and the next pass corrects it.
- Packet 008 Part A: "All" removes only `in:library`. A Library-only field, for example `is:ready`, stays in the text, and the present message of ADR 0087 explains it. The title row of the results page shows the query. The mockup shows the word "Results", and the final walk asks about this.
- Packet 008 Part B: All has no Select, because its groups need a new selection model. Its keyboard focus moves through its rows, across the headings. The results page has no Load more, so a partial count shows a plus sign. Artists and Labels divide on the link class of the feed, and a name row stays in Artists.
- Packet 008 Part B: `start_tab` has no reader now, and a saved search opens at All. The column stays until a later cleanup, because its removal needs a migration. A new saved search stores `releases` for All and `artists` for Labels.
- Packet 008 Part C: the results page has no tile view and no letter bar. The tile code is bound to `LibraryApp`, and the page holds one run with no name request. R-LIST-1 item 2 and Decision 5 are thus open for this page, and the final walk asks the operator. Track and artist rows have no date, so Newest keeps the sequence of the run for them.
- Packet 009: a Public playlists row has no keyboard focus and no Enter action in this run. The rel word of a remote item shows as the response gives it.
- Packet 010: the copy item is on a row and in the menu of the page. A tile has no menu, and the default view of the page is Tiles. A copy with no readable track still makes an empty playlist, and the report states the counts. The final walk checks each.
- Packet 010: the copy allows one request for each two seconds to one host, and HTTP 429 stops that host for the copy. Item 34 of the responsiveness list records the database lock.
- Packet 011: a Live row opens only an `http` or `https` link (`live_link_url`). A row with no feed GUID is dropped.
- Packet 012, Setup: the section holds the capabilities that the Show cards map to: Playback, Producer, LibraryPaths, Publisher and Encoder. The words "needs attention" stay in three status labels of the startup report, the live status and a Show card, because they are not the destination.
- Packet 012, source playlist: `LibraryApp` reads the source playlist of the session through its own connection. The view mode config field goes, and an old `config.toml` still loads.
- Packet 013: below 940 px, "Check All Feeds" and the feed state are in the More menu. The tag update button and the report text stay in the sidebar, so they are not reachable in a narrow window. The Inbox Updates section holds the tag update action there. A Show or Settings shortcut in a narrow window has no sidebar row to focus. The final walk checks each.
- Packet 015: the "…" menu of a Library track page has "Inspect Sources", and an Index track page has no such item. A request for a track that is not mounted opens the sheet one time, when the track mounts.
- Gate: `adr_0066_converter_preserves_flac_first_and_existing_ffmpeg_fallback` failed one time in a full run on 2026-11-06, and it passed three times alone and in the next full run. An agent saw `adr_0066_converter_timeout_reaps_child` fail one time under load. Both are in code that this run did not change. No packet owns the two timing results.

## Questions For The Final Walk

The operator answers these after the walk. The orchestrator did not decide them.

1. A list page shows its title in the toolbar and in its title row (R-TOOLBAR-1 and R-LIST-1). Is this correct, or does one go?
2. My playlists stays in name sequence. Decision 5 says that the curator sets the sequence, and the Non-Goals put the moves of My playlists later. Which holds?
3. The results page has no tile view and no letter bar. The title row shows the query, and the mockup shows the word "Results". Is this correct?
4. A tile of a public playlist has no menu, and Tiles is the default view. Is the menu on the page enough for "Copy to My Playlists…"?
5. A copy with no readable track makes an empty playlist. Must it make nothing?
6. In a narrow window, the tag update button is only in the Inbox. Is this correct?
7. A reload that clears an open Library page shows its list root, with the page entry still on the stack. Is this correct?

## Result

Packets 001 to 015 are implemented and committed. Packet 016 waits for ADR 0068. The mechanical gate is Green at 2782 unit tests and 399 guards. No person has walked a visual check of this run.

## Walk Corrections - 2026-11-06

The operator asked for correction and recheck until the operator and the orchestrator agree. The screenshots of the first run are in `~/.claude/v4vmm-shots/20261106-092538`.

| Fault in the walk | Cause | Correction |
| --- | --- | --- |
| The sidebar and the page showed Search under the title "Releases" | The Library view model started on Search | The view model starts on Releases. A test compares it with the layout |
| The search list stayed open | Only a loss of focus closed it, and each focus opened it | ADR 0087 Decision 3 now states the rule. A click outside closes the list. Focus alone opens no list |
| A Releases row showed a 404 | The key held the row ID `index-feed:{guid}` | The key holds the GUID. A test holds this |
| The view icons were invisible | `layout-grid` and `layout-list` were not in the embedded icon list | The list holds them. A test loads each extra icon |
| A blue frame surrounded each page | The focused frame drew an outline | ADR 0089 Decision 9 removes it. The orchestrator decided this with no answer from the operator |
| The rows moved when a focus ring or a mark showed | The ring added width, and the mark bar took a column | The border always takes its space, and the bar sits over the padding |
| A sidebar tooltip repeated the row text | Each row had a tooltip | A row has a tooltip only when the tooltip adds text |
| The search field moved on each page | The title width moved it | The side areas have equal widths, and the center is at most 640 px wide |
| The letter bar used two lines | Each letter had the minimum hit width | A letter has no minimum width. The minimum height stays |
| Search, My playlists and the Inbox had no title row | They used no page head | Each has one, and it holds the page action |
| Album rows looked like children of a group | An empty column stayed for a chevron that the sort Newest does not show | In the sort Newest, a release row has no empty column |
| "1 file have tags", and centered buttons | Text and alignment | The text is singular, and the buttons align at the left |

The second screenshot run is in `~/.claude/v4vmm-shots/20261106-094507`. It found these faults:

| Fault in the walk | Cause | Correction |
| --- | --- | --- |
| A row of one track said "Album", and its page said "Single" | The Index row builder set no key and no kind, so the row kept the default kind. The Index states `release_kind: unknown` | The builder resolves the kind from the track count. A test holds this |
| A bright line ran under the status area, and a heavy line under the "Select" band | The frame chrome drew two borders in a text color | One border in the separator color, only under a visible header |
| Tiles and Rows looked equal | `Button` ignored its selected state | The selected button has a tint (R-LIST-1 item 3) |
| A long toolbar title was cut in the middle of a letter | The equal side areas of the toolbar leave less width | The title has an ellipsis and a tooltip (R-TOOLTIP-1 item 2) |
| The title moved by some pixels between pages | The title row took the height of its controls | The title row has one minimum height |

The frame outline, the tooltips on controls with text labels and the choices that have no rule wait for the operator. The orchestrator did not decide them again.

### Decisions From Evidence - 2026-11-06

On 2026-11-06 the operator told the orchestrator to make the best choices from evidence for the user experience and the long-term code quality. A reversal of each decision needs only its own change.

| Decision | Evidence | Cost |
| --- | --- | --- |
| The frame outline stays removed, and a row shows the keyboard focus when a list gets it | The outline read a stored flag, and screenshots 03 and 07 show it beside the real focus. R-FOCUS-1, WCAG 2.4.7 and the Apple HIG list guidance point to the item | An empty list has no row to mark |
| `Button` shows no tooltip with a text label. The sidebar rows have no tooltip | R-TOOLTIP-1 item 3 | The sentence "Inbox, 1 item needs a decision" no longer shows. The number and the mark stay |
| Sidebar rows and the list are tab stops | R-FOCUS-1 checks a page with Tab, and only buttons had tab stops | The Tab sequence is longer |
| The search field has a fill | R-FIELD-1 item 1 | None found |
| Playlist rows show the type dot, the word and "n tracks" | ADR 0089 Decision 6 | None found |
| A public playlist hides the name "Unknown Artist" | The live Index gives this placeholder as `release_artist`. AGENTS.md calls placeholder text a source problem. Stophammer can also correct it | No request to Stophammer exists yet |
| Focus alone opens no search list | ADR 0087 Decision 3 names a click, and the Apple HIG warns against a change with no action of the person | Ctrl+F shows no list until a key |
| "New Playlist" replaces "+" | The durable set forbids a glyph string in a renderer | A wider control |
| "New Saved Search" is a secondary button | The toolbar already has the primary "Search" button | Less emphasis |
| List rows keep a 2 px border in each state | A row must not move when its state changes | Rows are 2 px taller, and a marked row has a heavier outline |

Answers to the seven questions:

1. Keep both titles. R-TOOLBAR-1 item 1 gives the toolbar title, and R-LIST-1 item 4 gives the title row. A change needs a musicindex record.
2. My playlists stays in name sequence, because the Non-Goals of ADR 0089 put moves later.
3. Correct. Decision 5 gives the results rows, and a query names the results better than one fixed word.
4. Correct for now. The page menu has the copy and Decision 1 holds. A tile has no menu on any page, so a menu needs a shared tile owner.
5. No. A copy with no readable track now makes nothing and gives a report.
6. Correct. The Inbox holds the same action, and the tab bar shows the Inbox count.
7. An open finding. Back can then appear to do nothing. No new packet owns it yet.

The "Select" band of an item page stays. No shared rule places it, and each item page type needs a change. This needs its own packet.

### Keyboard Focus Report - 2026-11-06

The operator reported that Tab moved the focus and no outline showed. The sidebar ring and the row mark read `Window::last_input_was_keyboard`, and GPUI clears that fact at each mouse move. This is a probable cause, and it is not proved. A new owner, `src/ui/focus_ring.rs`, holds the state: a key press shows the ring, a mouse button hides it, and a mouse move changes nothing. "Select" and the view icons now have a tab stop, the ring and keyboard activation.

The operator then found that the stop with no mark comes after the search field and before the main button of the page. That stop was the list area, which the orchestrator had made a tab stop on each page. Only a list with row focus and with rows is now a tab stop. The script scenario `keyboard` presses Tab and takes a screenshot at each stop.

The third screenshot run is in `~/.claude/v4vmm-shots/20261106-103223`. It holds the scenario `shell` only. It showed these faults, now corrected: a marked group row had the fill of a selected row, and the download icon of a track row used the accent color (ADR 0089 Decision 6). The orchestrator also found that its tooltip rule removed the tooltip of icon buttons with an empty label, and corrected this.

The keyboard run is in `~/.claude/v4vmm-shots/20261106-104940`. Tabs 1 to 7 gave identical images, so no ring painted. After Down, the sidebar row showed its ring, so the ring paints. GPUI gives a key with a binding to its action and returns before the element key listeners, so the listener never saw Tab. The state now uses a keystroke interceptor, and its test has a real Tab binding. The earlier mouse-move explanation stays unproved, because this run does not test it.

The same image showed that a click on a sidebar row left the focus on the row, so the arrow keys never reached the list. A mouse click now moves the focus to the list. A keyboard user stays on the row.

The next keyboard run is in `~/.claude/v4vmm-shots/20261106-110306`. Each Tab stop shows its ring, the ring stays after a mouse move, Shift+Tab goes back and Down moves through the list. It showed these faults, now corrected: a list row kept its ring after the focus left the list, the list came before the page controls in the Tab sequence, and a button with a plain click handler was not a tab stop. A mouse click on a Releases title-row control now gives the focus back to the list, so the arrow keys keep working. ADR 0069 keeps the focus transfer of a button click, so the shared button keeps it. The sort menu and the letter bar stay out of the Tab sequence.

The keyboard run in `~/.claude/v4vmm-shots/20261106-115450` gave a Tab sequence in screen order: the sidebar rows, the two sidebar buttons, the search field, "Search", "Open Show", the view icons, "Select", the scope and the list. The list stop showed no mark, and the cause in the focus listener was not found. The listener is gone. The list now marks its row in the draw that sees the keyboard focus.

The run in `~/.claude/v4vmm-shots/20261106-120337` shows the first row marked at the list stop, and the mark goes at the next stop. The operator then asked to reach the letters as a unit. The letter bar is now one tab stop with a cursor (ADR 0089 Decision 9).

The run in `~/.claude/v4vmm-shots/20261106-122447` shows the letter bar as one stop on Artists. The cursor moves with no request, Enter runs the letter, and Tab leaves and returns. The sort menu and the row menus still need the mouse, and the "Select" band of an item page has no owner. They wait for a follow-up packet.

The run in `~/.claude/v4vmm-shots/20261106-123939` covered scroll, Load more, Back, the sort, the scope, the pages, the results, the narrow form and Light. It showed these faults, now corrected: the mark did not show in Light, the "More actions" cell of a public playlist page showed only on hover, and the rows of the results page had no type dot. A saved search row and a feed row keep their trailing `TagBadge`, which already has the dot and the word.

