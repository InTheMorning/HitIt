# musicindex ADR 0053 List Controls Phase Plan

Status: Open. Created 2026-11-07.

Packet 1 is implemented on 2026-11-08 (`b3b391d`). Packet 2 is implemented on 2026-11-08 (`90b6a8c`). Packet 4 is implemented on 2026-11-08 (`367e72c`). Packet 5 is implemented on 2026-11-08 (`2c5c7ea`). Packet 6 is implemented on 2026-11-08, with ADR 0090 task 004 (`2346d29`). Packet 7 is implemented on 2026-11-08.

Packet 7 note: `src/view_models/name_order.rs` compares names with `icu_collator` in the locale of the reader. It reads numbers as numbers. Each list that the app sorts on its own side uses it. A list that merges Library rows with node rows keeps the sequence of the node. Thus a name can move when the person changes the scope.

R-LIST-12 item 2 is open for one list. On a group page, a Library release with no title shows its item GUID, and A to Z uses that text.

Packet 6 note: a section offers Newest only when each of its rows has a date (R-LIST-9 item 1). A Library feed page row has the newest publish date of its tracks. An Index feed page row has no date, so its sections offer A to Z only. Packet 9 can give them Newest with `artist_key` on `/v1/feeds/recent`.

The public playlist page has no list bar, so its control is at the right end of a heading above its entries (R-LIST-11 item 3).

Packet 3 waits for the decision on the stored Curator Order. Packet 7 is committed as `7fc663f`. Packet 8 is committed as `bfb8674`. Packet 9 is committed as `70d8f23`.

Packet 3 is implemented on 2026-11-08 with migration 24. Each packet of this plan is implemented. musicindex ADR 0054 then replaces the reverse icon of these packets.

Packet 9 note: Artists offers Newest in the scope In Library from the newest Library release date of each artist. It offers Newest in the scope All when the node lists `artist_newest`. The stored contract is the live contract of Stophammer candidate v0.19.0-rc.5 (git `a0c0c5a`), with `newest_release_at`. The open checks are in section 12 of [pending human checks](../pending-human-checks.md).

Packet 2 note: a saved search page starts with Newest, also when its query has a word. Thus it keeps the window groups of ADR 0086 (answer 6). A new search from the toolbar starts with Relevance when its query has a word.

## Node Facts Of 2026-11-08

The Stophammer session reported these facts of candidate v0.19.0-rc.5 on the primary node (`git_revision` `a0c0c5a`, API.md at Stophammer `365d514`):

- `/v1/node/capabilities` lists `artist_newest` (Stophammer ADR 0089) and `feed_by_url` (Stophammer ADR 0088).
- `/v1/artist-names?sort=date` gives the artists by the row field `newest_release_at`. Rows with no date come last in each direction. `sort=date` with `from` gives the status 400.
- `/v1/feeds/recent?artist_key=<key>` gives the releases of one artist, newest first, and with `reverse=true` oldest first.
- The node puts names with symbols or accents after Z in its name sequence. Stophammer accepted a fix: a group "#" before A, with `from=#`. A subsequent candidate delivers it.

Packet 9 starts after packet 8.

Owner of the rules: [musicindex ADR 0053](../../../musicindex/docs/adr/0053-which-lists-have-a-sort-a-reverse-and-a-view.md) R-LIST-8 to R-LIST-12. musicindex ADR 0043 R-LIST-6 and R-LIST-7 give the shape of the control. This plan states no rule.

## Goal

Each v4vmm list shows the sorts, the reverse icon and the view icons of its R-LIST-9 kind. No other list shows a control.

## Answers Of 2026-11-07

The orchestrator session relayed these answers from the accepted musicindex ADR 0053. Each packet that uses an answer records it in its commit.

1. Results in the scope All have no reverse icon. ADR 0087 Operator Answers item 1 stays, because merged rows of two sources have no one order (musicindex ADR 0043 R-LIST-7 item 10, ADR 0053 R-LIST-8 item 3).
2. Artists show Newest in the scope In Library, from local dates. The scope All shows Newest when Stophammer gives `sort=date` on `/v1/artist-names`. A sort shows in a scope only when that scope can give it.
3. Artists have rows and tiles in both scopes. A name with no feed shows the shared logo placeholder. This is packet 8.
4. The My playlist page is an editor, because the curator drags its items. It has no sort control (R-LIST-8 item 1).
5. Below 8 rows, the list bar of a release page keeps Select and shows no sort control.

The orchestrator session relayed these operator answers on 2026-11-08:

6. A saved search opens with Newest and its window groups, also when its query has a word. A new search from the toolbar keeps Relevance.
7. Packet 3 uses a stored Curator Order: the column `playlists.position`, with a migration that keeps the name order of today. A new playlist goes last. The curator drags a row of My playlists in Curator Order to move it. Newest uses `created_at`. A change to a v4vmm ADR records the schema and the new edit.
8. A section of an item page offers Newest only when each of its rows has a date (R-LIST-9 item 1).
9. Packet 9 comes first, then packet 3.

## Packets

| Packet | Content | Lists |
|---|---|---|
| 1 | Reverse from the node | Releases All, Public playlists, Artists All |
| 2 | The search sorts | The results page and the saved search page: Relevance, A to Z, Newest, and reverse except on Relevance and in the scope All |
| 3 | A curated list | My playlists: Curator Order, A to Z, Newest, reverse, rows and tiles. The [ADR 0089 amendment of 2026-11-08](../adr/0089-the-shell-destinations-and-list-pages.md#amendment---2026-11-08-the-stored-curator-order) stores the Curator Order (schema 24) and adds the drag |
| 4 | Group pages | The singles group page and the window group page: Newest, A to Z, reverse, rows and tiles in the list bar |
| 5 | Track and item order | Release tracks: Feed Order and its reverse. Public playlist items: Playlist Order and its reverse. Each from 8 rows |
| 6 | Sections of an item page | The artist feed page and the name level page: Newest, A to Z, reverse and views from 8 releases, at the right end of the section heading. With ADR 0090 packet 004 |
| 7 | Name order | The local A to Z compares in the language of the reader and reads a number as a number (R-LIST-12) |
| 8 | Artists views | Rows and tiles in both scopes (answer 3) |
| 9 | Artists Newest in All | `/v1/artist-names` with `sort=date`, when the node lists the capability `artist_newest` (Stophammer ADR 0089). Each row then has `newest_release_at`. This waits until the orchestrator says that the deploy is live |

## Lists With No Control

Live, the Inbox, the Show cuelist, the list of saved searches, the download queue, Settings and the My playlist page have no sort control (R-LIST-8 items 1 and 2, R-LIST-9 item 3, answer 4).

## Checks

Mechanical: each packet has tests at the view model layer for the sorts, the default and the reverse of each list it changes, and for the request of a node list.

Visual: the main agent checks each packet on its own Xvfb display with openbox, in Light and Dark, before its commit. A person checks the list bar with Select and the sort control (ADR 0053 "How This Decision Is Checked", visual item 2).
