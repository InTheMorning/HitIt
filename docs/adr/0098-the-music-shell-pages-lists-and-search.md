# ADR 0098: The Music Shell, Its Pages, Its Lists And Its Search

## Status

Accepted - 2026-11-14.

- Accepted: 2026-11-14, by the consolidation rule that the operator accepted on 2026-11-14 (musicindex-research commit `df7b823`). This ADR makes no new decision. It restates each live rule of the six ADRs below, and the [item map](#item-map) gives the new place of each old item.
- Supersedes: [ADR 0084](archive/0084-music-column-browser.md), Music is a column browser.
- Supersedes: [ADR 0085](archive/0085-artists-by-confirmation.md), artists by confirmation.
- Supersedes: [ADR 0086](archive/0086-new-releases-singles-window.md), the window groups of singles.
- Supersedes: [ADR 0087](archive/0087-search-fields-and-live-results.md), the search fields and the live results.
- Supersedes: [ADR 0089](archive/0089-the-shell-destinations-and-list-pages.md), the shell, the destinations and the list pages.
- Supersedes: [ADR 0090](archive/0090-one-list-bar-above-the-rows.md), one list bar above the rows.
- Changed by: [ADR 0091](0091-search-parser-moves-to-the-shared-crate.md), 2026-11-07. The parser, the Index plan and the kind rule of Decisions 21 to 25 are in the `musicindex-search` crate. musicindex ADR 0049 owns the syntax. ADR 0091 records this change against ADR 0087.
- Revised: 2026-11-14: Decision 16 states the open point of ADR 0068 Answer 1. No rule changes.
- Revised: 2026-11-17: the known roles are the nine roles of musicindex ADR 0081 R-CURATOR-1. That record adds curator. When the node lists `playlist_links`, the pages and the lists also show the playlist links of musicindex ADR 0081 R-CURATOR-3 to R-CURATOR-6. No rule of this record changes.

The six ADRs above changed other ADRs. Their "Changes" lines and the "Changed by" lines in ADRs 0046, 0047, 0049, 0060, 0062, 0082, 0083 and 0091 stay as they are. Each of those "Changed by" lines names this ADR as the place of the restated rule.

A code comment, a test or a document that cites an item of the six ADRs finds its rule through the item map. This ADR does not change those citations.

The app has the name HitIt (ADR 0096). The six ADRs use the earlier name v4vmm. This ADR uses HitIt.

## Context

musicindex ADR 0080 owns the shell, the back control, sort, view and the toolbar of the three apps. musicindex ADR 0028 owns the sections and the header actions of an item page. This ADR cites them, and does not state their rules again. It states only what those records leave to HitIt (R-LIST-1 item 6), and the Music rules of HitIt.

Six ADRs held these rules from 2026-10-08 to 2026-11-06. ADR 0084 had five "Changed by" ADRs, more than the limit of three of the project-baseline skill. The archived ADRs keep the context, the operator answers and the rejected alternatives of each rule.

## Decision

### 1. One Sidebar Holds Each Destination

The shell is a stack of musicindex ADR 0080 R-SHELL-1: a list is a page at full width, and an item opens over it.

The sidebar is one list with no group labels. The toolbar has no section tabs.

| Destination | Page |
| --- | --- |
| Search | The saved searches (Decision 3) |
| Releases | The releases, and the start page of the app |
| Artists | The artists and labels |
| My playlists | The playlists of the curator, in the sequence that the curator sets |
| Public playlists | The public playlists of the Index (musicL feeds) |
| Live | What is live now on the Index |

A line follows these destinations. Below the line are Show, Inbox and Settings.

- Show stays a screen mount (ADR 0060). Settings keeps its pages.
- The feed state and "Check All Feeds" stay at the bottom of the sidebar until the status bar of [ADR 0101](0101-the-design-language-follows-the-shared-rules.md) Decision 7 takes them.
- "Copy to My Playlists…" is in the item menu of a public playlist.
- Discover, Tracks and Needs attention are not destinations. Releases with the sort Newest takes the "New releases" view of Discover. The Search page takes the saved searches. A track stays reachable from a release page, a playlist and the Tracks tab of the results. The Inbox takes the tracks of Needs attention (Decision 2).

In a narrow window, the tab bar of R-SHELL-1 item 7 has Search, Releases, Show, Inbox and More. More opens a menu with each other destination.

ADR 0060 records the result of this decision for its own invariants.

### 2. The Inbox Holds What Needs A Decision

The Inbox holds the tracks that a show cannot use as they are, the updates and the setup that a show needs. The sidebar shows its count, in words for the accessibility label, so the count does not rely on color alone. An empty Inbox shows an empty state of musicindex ADR 0026 R-EMPTY-1.

The Inbox lists each track that a show cannot use as it is. The tracks of the show playlist come first.

### 3. The Search Page Lists The Saved Searches

One search field stays in the toolbar on each page (R-TOOLBAR-1). Search stays a toolbar command, never a section (ADR 0060). A click on the Search destination opens the Search page and puts the focus in that field.

- The page lists each saved search, with its name and the count of its new results. The query is the subtitle when it differs from the name. "New Saved Search" makes one.
- The count of the new results of a saved search needs stored state. Its packet adds a migration through the ADR 0016 registry.
- The subtitle of a saved search with no name of its own names what its query reads: "Library and Index", "Library" or "Index". A query with a scope conflict reads nothing, so its subtitle is "No results".
- "How to Search" gives four steps and example queries. The steps show when no search is saved.
- A saved search opens over the list, with the same results as the results page. "Edit Query", or a click on its query, puts the saved query in the toolbar search field and gives that field the focus (R-TOOLBAR-1 item 2). The results change while the curator types. The title row shows "Editing: " and the name of the saved search, with "Save Query" and "Cancel".
- "Save Query" or Enter keeps the change. "Cancel" or Escape restores the stored query. Leaving the page, or a new search, ends the edit with no save.
- The item menu of a saved search has Edit Query, Rename…, Duplicate and Delete Saved Search….
- "Duplicate" makes a new row with the same query. Its name is the name of the original followed by " copy". A later copy takes " copy 2" and so on. The copy opens in edit mode with its query in the toolbar field.
- "Rename…" refuses a name that another saved search holds, and nothing changes. A blank name becomes the query text, or its first free form.

A saved search keeps its text:

- A saved search stores the query text. Each run parses the text again, and runs its fields against the Library and the Index.
- A saved search is keyed by its name. Its row id stays the primary key and the navigation key. Migration 23 drops the unique index of the query that migration 21 added. Migration 23 gives each saved search a required name, unique without case. A name that another saved search holds is taken, and the new name gets " 2", " 3" and so on.
- Two saved searches may hold one query. The save action does not save a second copy of a query that a saved search holds. A query edit that gives the text of another saved search is accepted.
- The name of a saved search follows the query only when the name equals the old query text, without case. A name that the operator gave does not change.

The results page has two rows below its title:

1. "All | In Library" is part of the query. A click writes `in:library` or removes it, so this control shows no counts.
2. The type tabs are All, Releases, Tracks, Artists and Labels. They select a group of one run and do not change the query, so each tab shows its count.

A selection goes into the query text in three conditions:

- It must stay with a saved search.
- It needs a new run or the Index.
- A field gives the same selection.

In each other condition, it can be a view of one run.

### 4. Each List Page Has Its Defaults

R-LIST-1 gives the form of sort and view. musicindex ADR 0053 R-LIST-9 gives the sorts of each kind of list, and its [phase plan](../plans/musicindex-adr-0053-list-controls-phase-plan.md) applies them here. This table gives the defaults (R-LIST-1 item 6, R-LIST-10 item 3). Each page keeps its own sort, view and scope.

| Page | Sort at the start | View at the start | Scope |
| --- | --- | --- | --- |
| Releases | Newest | Rows | All |
| Artists | A to Z | Rows | All |
| My playlists | Curator Order (Decision 5) | Rows | None |
| Public playlists | Newest | Tiles | None |
| The results | Relevance when the query has a word, else Newest | Rows | In the query (Decision 3) |
| A saved search | Newest | Rows | In the query (Decision 3) |

The scope of Releases and Artists is the segmented control "All | In Library", below the title at the left. All is the default. No control shows Index items only.

- All holds the Index items and the Library items. Each Library item keeps its state.
- In Library holds what is downloaded, downloading or failed.
- A release that the curator removed shows only in All.
- In Library shows a count. All shows no count, because the Index gives no total. [Open request 13](../plans/hitit-open-requests.md) asks for a total.
- With the sort Newest, Releases groups the singles of one artist by the window of Decision 9. With the sort A to Z, it uses the group of Decision 8.

### 5. The Curator Sets The Order Of My Playlists

musicindex ADR 0053 R-LIST-9 gives a playlist that a person curates the sorts Curator Order, A to Z and Newest. Curator Order is the default. The curator sets this sequence.

1. **The sequence is stored.** The table `playlists` has the column `position`. The migration of schema 24 fills it from the name sequence of 2026-11-08, so no playlist moves. The migration registry of ADR 0016 owns the change.
2. **A new playlist goes last.** It gets the next free position.
3. **The curator drags a row to move it.** On My playlists with the sort Curator Order, the curator drags a playlist row to a new position. The app writes the new positions in one transaction. With A to Z or Newest, a row has no drag. In Curator Order, the row menu also has "Move up" and "Move down", so the keyboard can move a playlist.
4. **Newest uses the time that the curator made the playlist** (`created_at`), newest first. An edit does not make a playlist new.
5. **Each sort has a reversed entry.** The sort menu lists "Curator Order, Reversed" after "Curator Order", "Z to A" after "A to Z", and "Oldest" after "Newest" (musicindex ADR 0054 R-LIST-13).
6. **The "Add to playlist" menus follow the Curator Order.** The curator puts the playlists that they use most at the top.

### 6. One Page For Each Item, And The Library Is A State

Each release, track, artist and playlist has one page, whatever its origin. The origin selects the data source of the page, not its route or its layout.

| Item | Page identity |
|---|---|
| Release | Its feed GUID |
| Track | Its item GUID and its feed GUID |
| Artist | Decision 10 |
| Playlist | Its local id |

A page shows its Library state with a word, for example "In Library", "3 of 7 tracks" or "Not downloaded". Its actions follow that state ([ADR 0101](0101-the-design-language-follows-the-shared-rules.md) Decision 5).

A click on a Library row and a click on an Index row open the same page. A list is a page, and an item opens over it. The words "list column" and "detail column" in an earlier record mean the list page and the item page.

### 7. A Release Has A Kind

The curator sees "Release" for a feed of music, in place of "Album" and "Feed". Many releases are singles. Internal names do not change ([ADR 0060](0060-workflow-surface-structure.md) Vocabulary).

Each release shows its kind: Album, EP or Single. Labels that name a release use its kind, for example "Download single".

The kind comes from the value that the feed states, when it names album, EP or single. Without a stated value, a release with one track is a single, and any other release is an album. EP shows only when the feed states it.

### 8. Singles Of One Artist Show As A Group

In the Releases list with the sort A to Z, two or more singles of one artist in the current scope show as one group row. The row shows the artist, the count and the Library state, for example "Haleen · 7 singles · 5 in Library".

- The row opens in place to show each single. Its checkbox selects each single of the group.
- The group page lists the singles, newest first. Its main action is "Download n singles" when some are not in the Library, else "Add to playlist".
- A single page links to its group, for example "All 7 singles".
- On a feed page, "Albums and EPs" come before "Singles" (Decision 11.3).

The group key is the artist identity of Decision 10. Two artists with the same name never share a group.

A name that matches exactly one artist feed has no group of its own. Its singles join the group of that feed, and the group page shows them under "Matched by name", marked as inferred, as the feed page does (Decision 10). A name that matches two or more feeds keeps its own group.

### 9. Releases With The Sort Newest Groups Singles By A Window

#### 9.1 A Window Group Holds The Singles Of One Artist In One Window

A window group is a set of singles with one `SinglesGroupKey`. The singles of the group have their release date in one window. The release date of a MusicIndex single is its `newest_item_at`. The release date of a Library single is the latest `pub_date` of its tracks, as in the Decision 8 groups. The window length is the setting of Decision 9.7, 14 days by default.

The app makes the groups of one key from newest to oldest:

1. The newest single that is in no group starts a group. Its release date is the anchor of the group.
2. The group takes each single of the key whose release date is at most one window length before the anchor.
3. The app repeats step 1 until each single of the key is in a group.

Example: jSmovie has singles on Oct 20, Oct 15, Oct 8 and Oct 1. With a window of 14 days, group 1 holds Oct 20, Oct 15 and Oct 8. The single of Oct 1 starts group 2.

- A group with one single is not a group. It shows as the single, with the "Single" kind.
- Albums and EPs never join a group. They do not start, stop or break a window.
- A single with no release date joins no group.
- An artist thus shows at most one group tile for each window. An album of the same artist can show near it.

#### 9.2 A Group Tile Takes The Place Of Its Newest Single

The group tile shows at the position of its anchor single. The other singles of the group show no tile of their own.

#### 9.3 The Group Tile Shows The Count, The State And The Window

The group tile is a collection. Its art is the art of the anchor single in the logo frame of Decision 17.

- **Title.** The artist name of the key.
- **Second line.** The date span of the window, from the oldest single to the anchor, for example "Oct 8 to Oct 20".
- **Kind badge.** "n Singles", for example "9 Singles", in place of "Single".
- **State word.** "New" when one or more singles of the group are not in the Library. Then a count follows, for example "5 in library". When the Library holds each single, the state word is "In library".

The Rows view shows the same groups with the same facts in each row.

#### 9.4 The Group Opens A Window Group Page

A click on a group tile opens a window group page. That page is one page of Decision 6, and it goes into the navigation stack of Decision 12.

- The page lists the singles of the window, newest first.
- Its main action is "Download n singles" when the Library does not hold each single. Else its main action is "Add to playlist".
- It links to the Decision 8 group page of the key, for example "All 23 singles".

In selection mode, the checkbox of a group tile selects each single of the window (Decision 13).

#### 9.5 The Pager Loads Until Each Window Closes, Up To A Cap

A window is closed when the app has a row whose release date is older than the start of the window. The recency order then shows that no other single of the window exists. The recency order of [ADR 0062](0062-music-content-surface.md) stays.

After a page arrives, the app loads one more page while a window of a loaded group is open. It loads at most 5 extra pages, 100 feeds, for one page request. At most 4 feed detail reads of a page run at the same time. The pager keeps its loading state during the extra loads, and the tiles show when the loads stop.

When the cap stops the loads, each open group shows a "+" after its count, for example "13+ Singles". Its count grows in place when the next page arrives, and the "+" goes when its window closes. A single tile becomes a group tile in place when a later page adds a single to its window. The mounted view updates in place.

#### 9.6 Saved Searches Group By The Same Window

A saved search (Decision 3) makes window groups with the same key and window. The app makes the windows from the release date of Decision 9.1, not from the relevance order.

A relevance order never closes a window. A saved search thus groups only its loaded rows. It loads no extra page for a window. Each group of a saved search that a later page can change shows the "+" of Decision 9.5.

The group tile takes the position of its anchor single in the relevance order. The relevance order of each other tile does not change.

#### 9.7 The Window Length Is A Setting

Settings has a value for the window length in days. The value is 1 to 60. The default is 14. No value turns the grouping off.

- The value is in `config.toml` with `#[serde(default)]`, so an older `config.toml` keeps loading.
- A change of the value makes the groups again in the mounted view. The curator does not move away and back.
- The Settings row uses the shared guarded editor of [ADR 0069](0069-grouped-settings-and-selective-presets.md) task 002.
- **Configuration exception.** ADR 0066 holds back each configuration-format change until its task 004 passes. The operator made one exception on 2026-10-23: this setting adds one key with `#[serde(default)]`. The exception releases no other key, preset or format change.

### 10. One Artist Page With Two Levels

An artist page has one layout and one of two identities. The page says which identity it has.

- **Feed level.** The page of a feed that albums name, keyed by its feed GUID ([ADR 0077](0077-publisher-feed-artist-binding.md) Decision 1). It shows the feed title, its confirmation level and the releases of Decisions 11.2 to 11.4.
- **Name level.** A labeled grouping of the releases that name an artist text and name no publisher feed, for example "Releases that name Ledbetter". Its level word is "Name only". The page says that a name is not an identity. A name level page never merges with a feed page. The "Matched by name" rule below lists such releases on a feed page, marked as inferred.
  The key of a name is its sort key, as Stophammer ADR 0070 gives it: white space collapsed and lowercase. Two names that differ only in case or white space are one name.
- **Matched by name.** A name can match exactly one artist feed. Then the feed page also lists the releases of that name, in a section "Matched by name". The section says that the relationship is inferred from the name.
  A name matches a feed when its key is the key of the feed title. It also matches when its key is the key of an artist text that a release of that feed states. That name then has no name level page and no row of its own. When two or more feeds match a name, the name keeps its name level page.

The artist name on a release page opens the feed level of the artist feed of Decision 11.6. When the release names no feed, it opens the name level. Decisions 11.5 and 11.6 give the role rows and the "Also named" rows of the other named feeds.

### 11. Artists By Confirmation, And No Publisher Word

#### 11.1 The Interface Does Not Name A Feed "Publisher"

musicindex ADR 0033 R-WORD-3 and R-WORD-4 own the words for a publisher feed in all apps. This decision keeps only the parts that apply to HitIt:

- The frame title "Publisher" of Decision 11.7 is the test for a feed that is definitely not an artist. ADR 0033 R-WORD-3 item 3 cites it. Its frame title constant is also the label of the Publisher badge.
- The Inspect sources view shows each RSS value with its element name, for example `podcast:remoteItem medium="publisher"`. ADR 0033 R-WORD-4 item 2 permits it.
- Code identifiers, database columns, API field names and the CLI JSON output keep the technical term. ADR 0033 R-WORD-4 item 3 permits it.

#### 11.2 Artists Have Three Confirmation Levels

Each feed and each credit name in Artists has one level. The level comes from the link facts of Stophammer 0.7.0 only.

| Level | Rule |
|---|---|
| Artist | One link or more of the feed has `role_agreement` `both`, and its role set contains `artist` |
| Unverified artist | A confirmed link (`two_way_validated` true) makes the feed the primary named feed of one album or more. No link of the feed has `role_agreement` `both` |
| Name only | An artist text that names no feed. This is the name level of Decision 10 |

- A feed with agreed roles, none of which is `artist`, is not in Artists. Decision 11.4 shows it.
- A one-side `artist` role does not confirm an artist. The feed stays an unverified artist, and its page shows the stated role with its side.
- A feed that albums name only after their primary named feed, with no agreed role, is not in Artists. The release page shows it (Decision 11.6).
- A feed that no album names is not in Artists. Its page opens from "Shares releases with" and from search.
- The Artists list shows no level word. Only the artist page shows the level (Decision 11.3).
- "Unverified" tells that nothing confirms the role. The app uses "artist" as its default word for a named feed, not as a derived fact.

#### 11.3 Each Page Section Names One Relation

The page of a feed has no type. It shows the level of Decision 11.2 in its header, in words. Its releases go in these sections, in this sequence:

| Section | Releases |
|---|---|
| "Albums and EPs", then "Singles" (Decision 8) | The release names the feed, and the link has an agreed role set that contains `artist`, or no stated role. This is the discography of the feed |
| "As <roles>", for example "As label" or "As label and producer" | The release names the feed, and the link has an agreed role set with a known role and without `artist`. One section for each different set of known roles |
| "Role not confirmed" | The release names the feed, and the link has `role_agreement` `one_side` or `conflict` |
| "Listed by <feed title> only" | The feed lists the release, and the release does not name the feed (`album_names_as` is null) |

- No section heading names the relation "artist". On an artist page, the heading is the release kind.
- An "As <roles>" heading gives only the known roles, in the sequence of musicindex ADR 0036 R-ROLE-1. It joins the last two words with "and" (musicindex ADR 0066 R-ROLE-6).
  A link with an agreed role set that has no known role has no role (musicindex ADR 0066 R-ROLE-4). Its release goes in "Albums and EPs" or "Singles". The agreement of the two sides compares the words of the feeds, so "label" and "recordLabel" on the two sides gives the agreement `conflict`.
- Each row in "Role not confirmed" tells who stated which role, by feed title. A known word stays a role word. The unknown words of one side show as RSS text in one `rel="…"` (musicindex ADR 0066 R-ROLE-5 items 3 and 4). Examples: "Sir Libre Records: label · the album: rel="recordLabel"" and "The album: label · Sir Libre Records: no role".
- A row in another section shows no role line. Its section already tells the relation.
- A release appears one time. A section with no release is absent.

#### 11.4 Other Roles Appear When A Feed States Them

- An "As <roles>" section of Decision 11.3 shows only when a feed states a known role other than `artist`.
- The header lists the agreed roles, as ADR 0082 Decision 4 gives.
- "Shares releases with" stays as ADR 0082 Decision 5 gives. Each row shows the feed title, its roles and the count of shared releases, with no source text. The count holds releases of each kind, so the words are "release" and "releases" (musicindex ADR 0085 R-SCOPE-1 item 2 and R-WORD-9 item 5). The `releases give rel="…"` text of "Shares releases with" is RSS text and keeps each word.
- A Labels destination is a subsequent decision. The operator makes it when many feeds have a label role.

#### 11.5 The Release Page Names Its Artist, And A Role Row Only For Another Role

- The artist name on a release page opens the artist page of the artist feed of Decision 11.6, as Decision 10 gives.
- The release page has no "Publisher" row.
- When the agreed role set of the link does not contain `artist`, the release page shows one role row with the source text.
  An example is "Label: Sir Libre Records". The row opens the page of that feed.

#### 11.6 An Album Can Name More Than One Feed

The named feeds of an album keep the sequence of the RSS: the `position` of each item with `source` `podcast_publisher`. The first one is the primary named feed.

- The artist feed of the album is the first named feed with an agreed role set that contains `artist`. Else it is the primary named feed with a confirmed link. The release page then says that the artist is unverified.
- Sometimes no named feed has an agreed `artist` role, and the primary named feed has an agreed role set without `artist`. Then the album has no artist feed. The artist name opens the name level, and the primary named feed shows as a role row.
- Each other named feed with a stated role shows as a role row of Decision 11.5, for example "Label: Sir Libre Records".
- Each other named feed with no stated role shows as an "Also named" row with its feed title. The row opens the page of that feed.
- Sometimes no named feed has an agreed `artist` role, and the link of the primary named feed is not confirmed. Then the album has no artist feed. The artist name opens the name level. The primary named feed shows as an "Also named" row.
- An observation time never selects a named feed. `owned_publisher_feed_guid` uses the artist feed of this decision.

#### 11.7 The Frame Title Of A Feed Page Names Its Relation

The frame title of a feed page is the type label of musicindex ADR 0033 R-WORD-3. The title and the header badge show the same word (ADR 0033 R-WORD-3 item 6). A known role counts when a side states it (musicindex ADR 0066 R-ROLE-3).

| Page | Frame title |
|---|---|
| A side states one or more roles | The label of musicindex ADR 0037 R-WORD-7. When the level is Artist or Unverified artist (Decision 11.2) and no side states `artist`, "Artist" comes first |
| No side states a role, and the level is Artist or Unverified artist | "Artist" |
| No role, no side states `artist`, and each named link has a known rank that is not 1 | "Publisher" |
| Each other page | "Feed" |

- "Definitely not an artist" is the test of this decision: no side states `artist`, and each named link has a known rank that is not 1. ADR 0033 R-WORD-3 item 4 cites it.
- An unknown rank never gives "Publisher". Before the page facts load, the title is "Feed".
- The R-WORD-7 label lists each stated role. Two or more roles do not give the label "Publisher".
- A type label and its dot use only the known roles of musicindex ADR 0081 R-CURATOR-1 items 1 and 2. They ignore each other role word, and the compare ignores capital letters (musicindex ADR 0066 R-ROLE-3). A feed with only unknown role words has no role (R-ROLE-4).
- The badge has one dot. The dot has the color of the first role in the label, in the R-ROLE-1 sequence. It is filled when the two sides give that role (musicindex ADR 0066 R-WORD-8). A feed with no role has a hollow dot, with the word "Publisher" or "Feed" of this decision.
- Each interface text that names the roles of a feed uses only the known roles, in the R-ROLE-1 sequence. The "Roles" fact joins them with ", ", and the "As <roles>" heading of Decision 11.3 joins them with "and". A text that only lists the roles does not show when no known role stays (musicindex ADR 0066 R-ROLE-6).
- A text that tells what one source states, such as the line of a "Role not confirmed" row, shows an unknown word as RSS text in the form `rel="…"` (R-ROLE-5 items 3 and 4). Inspect sources shows each role word as RSS text (R-ROLE-5).
- The header, the sections and the rows keep Decision 11.1.

#### 11.8 Each Artist List Has Three Groups

musicindex ADR 0057 R-ARTIST-1 gives each artist list three groups: the stated artists, the inferred artists, and the artist names with no feed. These lists are the Artists list in each scope, its tiles and the Artists tab of the search results. Each group has a heading that gives its type. The rows show no level word.

In the scope All, each group loads its own pages of 200 rows, and the stated group sends `stated_rel=artist`. The inferred group and the names group show 20 rows and a "Show more" button, as musicindex ADR 0057 R-SITE-15 gives.

One part of R-SITE-15 is not applied. The heading of a capped group gives its count only after each row of the group is loaded.

### 12. One Navigation Stack

The Music content area has one navigation stack. Each page change goes into it: a sidebar click, a row, a link and a search. musicindex ADR 0080 R-NAV-2 gives Back and Forward.

- A page shows the back icons and its title (R-TOOLBAR-1). No page shows a breadcrumb.
- Back and Forward show a page at the scroll position that it had in that history entry. A new open of a page, by a sidebar click, a row, a link or a search, shows the page at the top.
- No screen keeps a second navigation stack. This completes [ADR 0046](0046-workspace-frame-architecture.md) Invariant 2 for Music.

### 13. Selection Is One Shared Behavior In One List Bar

Each list of releases and tracks has the same selection model. It has two modes.

- **Normal mode.** No row shows a checkbox, and no row keeps space for one. A plain click opens the page of the row.
- **Selection mode.** "Select" starts it. Each row shows its checkbox. A plain click changes the selection of the row, and opens no page.
- Ctrl+click adds a row, Shift+click adds a range, Ctrl+A selects the list, and Space changes the row with keyboard focus. Each of them also starts selection mode.
- "Done" and Escape end selection mode and clear the selection.
- The bar shows the count, "Select all" while a row is not selected, the actions for the whole selection and "Clear". An example is "Download 7 tracks". With no selected row, it shows "Select items" and "Select all". A singles group page and a window group page say "singles" in the bar.
- "Select" and "Done" give the keyboard focus to the rows, so Ctrl+A and Space act on them.
- The keyboard focus shows in both modes.

No step needs a drag or a scroll wheel. The selection and the availability of each bar action are view-model state.

The controls of the selection are in one list bar:

1. A list of selectable rows on a page has one list bar, directly above its rows.
2. In normal mode, the left end has "Select". The right end has the sort menu, the rows icon and the tiles icon (musicindex ADR 0054 R-LIST-13). A list with one fixed sequence has the toggle "Reverse" there (R-LIST-14). A control that the list does not have is absent.
3. In selection mode, the left end has "Done", the count, "Select all", the actions for the selection and "Clear". The right end does not change, so a curator can sort while selecting. Selection is by item, so a new order keeps it.
4. The bar has the selection background in selection mode, and each row shows its checkbox. These two facts show the mode change.
5. The bar stays in view while the rows scroll.
6. In a narrow window, the bar wraps. The two ends stay on separate lines, so a destructive action does not move next to the sort menu.
7. A destructive action keeps its role (musicindex ADR 0042 R-ACT-4) and its confirmation.
8. The page has no selection band and no selection bar at its bottom.

Each page moves to the list bar in a packet of the [ADR 0090 phase plan](../plans/adr-0090-list-bar-phase-plan.md). Until its packet, a page keeps its earlier position: "Select" in the title row of the list page, and the action bar at the bottom of the list.

A list page has its title row above its rows. A later packet decides if that row is the list bar, and checks musicindex ADR 0080 R-LIST-1 item 4 first. A change to that rule needs a record in `../musicindex`.

### 14. One Download Queue

Each download goes through one queue, from the list bar or from a page.

- At most three downloads run at the same time. Two downloads of one release never run at the same time.
- A Library release and the Index copy of that release have the same queue key.
- The queue starts the jobs in their order. A job of a busy release waits, and the jobs after it can start.
- One failure does not stop the queue. A removal of a release waits until no download of that release is queued or running.
- One progress item shows the batch on each page, for example "Downloading 3 of 9". It names the item and the failure count. A finished batch shows its recorded finish time until the operator dismisses it.

These host rules apply to the RSS request of a download only. They do not apply to the MusicIndex reads or the enclosure downloads. ADR 0058 leaves this host policy unowned, and this decision owns it.

- The queue sends one RSS request at a time to each host. The minimum interval between two RSS requests to one host is 2 seconds.
- Each download has 3 tries. After HTTP 429 with a `Retry-After` value of 60 seconds or less, the queue waits for that value. After HTTP 429 with no `Retry-After` value, the queue waits 60 seconds.
- After HTTP 429 with a `Retry-After` value of more than 60 seconds, the queue stops the host for the batch. Each queued download of that host fails with a text that names the host and the requested wait. The downloads of other hosts continue.
- A waiting download keeps its slot. "Stop queued" removes each waiting download.

### 15. A Row Keeps One Main Action

- Each list row shows its type dot and type word (musicindex ADR 0038 R-DOT-4). A numbered track row on a release page does not, because its section heading gives the type.
- The library icon and the time each have one column, so they align in a list.
- The library icon is muted. A warning or a danger keeps its status color.
- The main action of an album or a track in the Library is Play.
- Adding to a playlist is in the item menu only.

### 16. Each Track Row Keeps A Place For Audition

Each track row and each release page keep a position for an audition control. The audition player of [ADR 0068](0068-show-cue-and-audition-isolation.md) fills it. Until then, no control shows in that position: a control that cannot act is absent.

Open point: [ADR 0068](0068-show-cue-and-audition-isolation.md) Answer 1, B, Accepted on 2026-11-11, puts the Audition part in the now-playing panel (Decision 18). Thus the row position can be not necessary. The operator decides this point before the ADR 0068 packets start, and until then this rule stays.

### 17. The Logo Frames A Collection

As on `../musicindex/search.html`, the MusicIndex logo shape frames the art of a collection. A collection is a singles group, an artist page or a playlist.

- The frame holds a mosaic of up to four covers. Two or three covers repeat to fill four cells, as `search.html` does.
- With no cover, the frame shows the entity color.
- A collection with one cover shows that cover as a square. A release always shows its cover as a square.
- An artist feed can give its own image in its RSS channel. That image replaces the mosaic and shows as a square, as one cover does. The mosaic is the fallback for each collection that gives no image. A playlist and the name level give no image.
- When the given image does not load, the frame shows the mosaic, or the entity color.

The logo shape is a named token. The technique is a packet's choice, for example a mosaic image that the image cache composes and masks off the UI thread. The view model carries no renderer type.

### 18. The Now-Playing Panel Is On The Right

The right side holds the panel of R-SHELL-1 item 5, only while something plays or a show is live. With neither, the panel is absent.

- The panel holds the current track, the transport, a seek control and Up Next. The transport and Up Next move there from the Show page.
- Play stays on the current page and opens the panel. It does not move to the Show page.
- The curator can expand and collapse the panel.

ADR 0068 decides the audio route of Play. Its [revision of 2026-11-11](0068-show-cue-and-audition-isolation.md#revision-for-the-adr-0089-shell---2026-11-11) gives the panel a Show part and an Audition part.

### 19. Inspect Sources Is A Sheet

"Inspect Sources" opens as a sheet (R-SHELL-1 item 6). ADR 0101 Decision 8 keeps its content. The app has no general inspector.

### 20. Shared Rules That Stay

- musicindex ADR 0068 R-SCALE-1 item 2 stays: HitIt uses the macOS values.
- The letter bar is one tab stop. Left, Right, Home and End move a cursor in it, and Enter or Space runs the letter. A move sends no request. This follows the WAI-ARIA toolbar pattern.
- One page shows at a time, so a frame draws no border and no focus outline. The focused control or row shows the focus (musicindex ADR 0026 R-FOCUS-1). When a list with rows gets the keyboard focus, a row shows it.

### 21. One Query Text With Words, Phrases And Fields

The toolbar search field shows a field list, reads fields in the query text and shows results while the curator types.

musicindex ADR 0049 R-QUERY-1 owns the grammar of this decision and the field table of Decision 22 (ADR 0091 Decision 2). These two decisions restate them for HitIt. A change to the syntax needs a record in `../musicindex`, not in this repository.

The query text holds terms. The terms of one query all apply together.

- A **word** matches as a plain search, for example `haleen`.
- A **phrase** is text in double quotes, for example `"night drive"`. It matches as one string.
- A **field** is a field name, a colon and a value, for example `artist:jsmovie` or `release:"aged friends"`. Decision 22 lists the fields.
- A `-` before a word, a phrase or a field excludes the matches, for example `-title:remix`.
- A field name has no case. `Artist:` is the same as `artist:`.
- A text with a colon whose name is not a field stays one word. Thus `Re:Union` searches for "Re:Union".
- A field with a value that is not correct, for example `after:yesterday`, applies nothing. The field list shows the error of that field (Decision 23).

One parser reads the text. ADR 0091 puts it in the `musicindex-search` crate, with the Index plan and the kind rule of Decision 7. The parser gives a typed query: the words, the phrases, the fields and each error. The Library search and the Index search read the typed query, never the text.

### 22. The Fields

| Field | Values | Library | Index |
|---|---|---|---|
| `artist:` | text | The track artist or the album artist | Stophammer filter (Decision 25) |
| `title:` | text | The track title or the release title | Stophammer filter |
| `release:` | text | The release title | Stophammer filter |
| `kind:` | `album`, `ep`, `single`, with commas for more than one | The release kind of Decision 7 | Stophammer filter. It sends the kind requests of musicindex ADR 0049 R-QUERY-6 (Known Differences 2) |
| `in:` | `library`, `index` | Sets the scope | Sets the scope |
| `after:`, `before:` | a date, `2026-10-01`, or a month, `2026-10` | The release date of Decision 9.1, and the `pub_date` of a track | Stophammer filter. A feed row uses its newest item date (Known Differences 3) |
| `duration:` | `>5m`, `<3:30`, `3m..5m` | The duration of a track | Stophammer filter for track rows only (Known Differences 4) |
| `is:` | `downloaded`, `partial`, `ready`, `removed` | The Library state of Decision 6, the show readiness of ADR 0059, and the "Removed from feed" mark of ADR 0076 | None |
| `playlist:` | a playlist name | The tracks of that playlist | None |
| `has:` | `split`, `image` | A value block with payment routes, or an image of its own | None |

- `title:` matches the track title and the release title. `release:` matches the release title only.
- The Library search has one query builder for each field. It gives at most 50 tracks.
- `after:` and `before:` include their date. A month includes each day of the month.
- `duration:` applies to tracks. A release row shows when one or more of its tracks match.
- `is:`, `playlist:` and `has:` apply to the Library only. A query with one of them shows no Index rows. The field list marks these fields "Library only".
- `in:index` shows no Library rows. A query with `in:index` and a Library-only field shows no rows, and the field list shows the conflict.
- An excluded `kind:`, `after:`, `before:` or `duration:` has no Index filter. The Index part shows the note of Decision 25.

### 23. The Field List

A click in the empty search field opens the field list under the field. Each row gives one field, its values and one example, for example `kind:  album, ep, single  ·  kind:single`.

- While the curator types a field name, the list shows the fields that start with that text.
- After a field name and its colon, the list shows the values of the field:
  - `artist:` shows the matching artist names of the Library and of `GET /v1/artist-names?from=`. The field list sends one such request after each pause.
  - `release:` shows the matching release titles of the Library.
  - `playlist:` shows the matching playlist names.
  - `kind:`, `in:`, `is:` and `has:` show their fixed values.
- The keyboard focus stays in the search field. Up and Down move the selected row. Tab and Enter put the selected row into the text. Escape closes the list. A click on a row puts it into the text.
- A click that is not on the field or the list closes the list, and the field keeps the keyboard focus. The focus alone does not open the list. A click in the field or a typed key opens it.
- Enter with no selected row runs the search.
- The list shows the error of a field, for example "after: needs a date, for example 2026-10-01".
- The view model gives each row, its text, its accessibility label and its typed action state. The renderer only composes.
- A shared composite under `src/ui/composites` draws the list. It does not use the popover primitive, because that primitive takes the focus.

### 24. Results Show While The Curator Types

After the curator stops typing for a pause, the search runs against the Library and the Index. Enter runs it at once. A named token holds the pause, 300 ms at first.

- Each run has a generation. The app drops a result of an older generation.
- A text with no word, phrase or correct field runs nothing. A term with `-` counts, so `-remix` runs. The Library part lists each row except the excluded rows, and the Index part sends no request (Decision 25). `-!!!`, `-artist:` and `in:library` alone run nothing.
- The mounted search page updates in place. A run does not add a new page to the navigation stack. Enter, or a click on a result, keeps the page in the stack (Decision 12).
- Each run sends at most three Index requests.
- A live run retains no failed action of ADR 0066. Enter retains it.

### 25. The Index Receives Filters, Never Syntax

HitIt sends the words and the phrases in `q`, and each Index field as its own parameter. Stophammer never parses the field syntax (musicindex ADR 0049 R-QUERY-1 item 4). [Open request 10](../plans/hitit-open-requests.md#10-search-accepts-words-phrases-and-field-filters) asked for these parameters. Stophammer deployed them in release 0.15.0.

- `q` holds the words and the phrases with no `-`. A query with fields and no word sends no `q`.
- `artist:`, `title:` and `release:` give the parameters `artist`, `title` and `release`. Two values of one field join with a space, so each value applies.
- A word or a phrase with `-` gives `not_q`. `-artist:`, `-title:` and `-release:` give `not_artist`, `not_title` and `not_release`. Each value is one exclusion.
- `kind:` gives the parameters of musicindex ADR 0049 R-QUERY-6, which the crate `musicindex-search` builds. `single` gives `kind=unknown` and `tracks=1`. `album` gives `kind=unknown` and `min_tracks=2`. Two `kind:` terms keep the kinds that both have. No shared kind sends no Index request. `ep` sends no Index request, because a count never gives EP.
- `after:` gives `after`, and `before:` gives `before`. Each value is `YYYY-MM-DD`. A day gives that day in both. A month gives its first day in `after:`, and its last day in `before:`.
- `duration:` gives `min_duration` and `max_duration`, in integer seconds. Each limit is included, as `duration:` is.
- Each entity type takes only some filters. A publisher row has no release, kind, date or duration, so `type=publisher` gets no request when the query has one of them. A feed row has no duration, so `type=feed` gets no request for a duration. Stophammer gives `400` for a filter that a type does not take. HitIt sends no such filter. Each type gets its own request, so the run still sends at most three requests.
- A query with exclusions and no positive term sends no request. Stophammer gives `400` with the code `missing_positive_term` for that request. The page shows the note "Music Index needs a word or a field to search for. The Library rows show." The note title has no period at its end: "MusicIndex needs a word or a field to search for".
- An excluded `kind:`, `after:`, `before:` or `duration:` has no parameter. The Index part shows the note "Music Index cannot exclude kind: yet. The Library rows show." The crate `musicindex-search` gives the first sentence. The contract has no `not_` form for these fields.
- The note of the Index part is above the rows, outside their scroll container, so a long list does not hide it and each row keeps its index.

The page never shows Index rows that a filter did not reduce.

These rules of sort apply to the results:

- Results in the scope All have no reverse icon.
- Each selected sort makes one `GET /v1/node/capabilities`, with no cache.
- Each sort can change the rows two times. The page first sorts its rows, and the Index rows then replace them.

## Known Differences From The Library

Each difference comes from the Index, not from the parser. The values were read on 2026-11-06 with read-only GET calls.

2. **`kind:`.** `kind:ep` sends no Index request, because a count never gives EP (R-QUERY-6). The Library gives `single` to one track, `album` to each other release and no `ep`. Open request 14, part 2, was declined, and its answer gives the filters that an app sends.
3. **`after:` and `before:` on a feed.** The Index compares the newest item date of the feed. The Library compares the release date (Decision 9.1). A track row uses its `pub_date` in both.
4. **`duration:`.** The Index filters track rows only. A query with `duration:` shows no feed rows from the Index. The Library shows a release when one of its tracks matches.
6. **Artist rows.** A query with no word gives artist rows only for the name of its `artist:` term. A query with no word and no `artist:` term gives no artist row.
7. **The first page.** The Index part reads the first page of each type, 20 rows. The Library reads at most 50 tracks.

Differences 1 and 5 closed on 2026-11-07, and the numbers stay so that a citation finds its difference.

## Open Points

The research request of ADR 0089 leaves these open, and this ADR does not decide them:

- the verbs "Download" or "Get" and "Drop"
- upcoming items in Live
- the other actions of a public playlist
- `in:index` on the results page, which only typed text can reach
- the main row action, shown on each row or only on hover and focus

## Invariants

- No view model exposes the text "Select an item".
- Each destination opens on content or an empty state.
- The now-playing panel renders only while something plays or a show is live.
- Each list page has its own sort, view and scope state.
- The view model gives the list bar its controls, labels and availability. The renderer decides none of them.
- Each control of the list bar keeps its accessibility label and its keyboard reach. "Select" and "Done" give the keyboard focus to the rows (Decision 13).
- A key that moves the focus to a button in a Music column scrolls the column to show that button, with its focus ring. A wheel scroll does not move back to the focused button. A guard in `src/ui/composites/music_column_scroll.rs` holds this.
- A menu trigger and a link pill of a page header take a tab stop, and Enter or Space opens a menu. The menu of a list row stays out of the Tab sequence. A guard in `src/ui/primitives/button.rs` holds this.
- A single is in at most one window group.
- Each single of a window group has the same `SinglesGroupKey`.
- No two singles of one window group are more than one window length apart.
- Albums and EPs are never in a window group.
- Releases with the sort A to Z keeps the Decision 8 groups, with no window.
- The word rules of musicindex ADR 0033 R-WORD-3 and R-WORD-4 hold. The frame title of Decision 11.7 is the one HitIt exception.
- A level comes from `role_agreement`, `role`, `album_names_as`, `two_way_validated` and the rank of the named feed only. A count, a name match or a credit text never selects it.
- Each level shows in words. Color is never the only difference.
- A release appears one time on one page.
- A stated role that one side gives stays visible, with its side.
- The named feeds of an album keep their RSS sequence.
- The start of a fixed value, such as `has:spl`, is an incomplete value and not an error.
- One parser reads the query text. No other code reads field syntax.
- A Library value goes into SQL only as a bound parameter, with its `LIKE` characters escaped.
- A Library text term matches at the start of a word, as the Index does. "hey" finds "HeyCitizen" and not "They".
- No Index request carries field syntax in `q`.
- Index rows of an entity type show only when Stophammer applies each Index field of the query to that type.
- No Index request holds a filter that its entity type does not accept.
- A result of an older run never replaces the result of a newer run.

## Non-Goals

- The folders of My playlists, a move into a folder, and the renames. The operator plans them for later.
- The audition player and the cue of ADR 0068.
- A Labels destination.
- A new MusicIndex request for the artist levels.
- A grouped query from Stophammer, and a group of albums or EPs.
- A change to the recency order of ADR 0062.
- A change to the sort, view or reverse rules of musicindex ADR 0043.
- Colored chips for the fields in the text. The text stays plain.
- Search operators `OR`, parentheses and fuzzy matches.
- Fields for genre, BPM and key. The Library stores them only as file tags.

## Consequences

- One record owns the Music shell, the item pages, the lists, the selection and the search of HitIt. ADR 0084 no longer has more "Changed by" ADRs than the limit.
- The rationale, the operator answers and the rejected alternatives stay in the six archived ADRs, out of the reading path.
- A guard name or a code comment that cites ADR 0084, 0085, 0086, 0087, 0089 or 0090 stays as it is. The item map resolves it. A guard that this ADR does not name changes only when its rule changes.
- A change to a rule here changes this ADR, with a "Changed by" line. A subsequent consolidation follows the same rule when this ADR reaches the limit.

## Verification

Mechanical criteria, each at the layer that owns it:

- The sidebar view model exposes the destinations of Decision 1 in that sequence, and Releases is the default.
- The tab bar view model exposes Decision 1 in a narrow window, and More gives each other destination.
- No view model exposes "Select an item". A situational guard blocks it.
- The list page view model of each page gives the defaults of Decision 4.
- The scope model puts a downloaded, downloading or failed release in In Library, and a removed release only in All.
- A click on the results scope writes or removes `in:library`, and a type tab does not change the query.
- The panel view model is absent while nothing plays and no show is live.
- The navigation model gives R-NAV-2 items 2 to 4, has one stack for the Music content area, and no view model exposes a breadcrumb. A guard blocks a second navigation stack in a Music screen.
- The Inbox view model lists each track that a show cannot use, with the tracks of the show playlist first.
- A release opened from a Library row and from an Index row builds the same page view model, keyed by its feed GUID.
- The kind of a release follows Decision 7.
- The Releases list view model gives window groups with the sort Newest, and Decision 8 groups with the sort A to Z. It gives one Decision 8 group row for two or more singles of one artist identity, and no group for one single.
- The artist page view model gives the feed level only for a feed GUID, and labels the name level.
- The selection model supports add, range, all and clear, and gives the bar actions with typed availability. The view model exposes the controls of each mode of the list bar, and a test holds them. A guard holds that the first page of the list bar has no frame header action.
- The view model makes the windows by the chain of Decision 9.1. Singles on Oct 20, Oct 15, Oct 8 and Oct 1 with a window of 14 days give one group of three and one single.
- The view model never puts an album or an EP in a window group, and never puts a single with no release date in a group.
- The view model puts a group row at the position of its anchor single, in Releases and in a saved search.
- The group row exposes the artist name, the date span, the count, the state word and the Library count of Decision 9.3.
- The pager view model requests one more page while a window is open, and stops after 5 extra pages. After the cap, the group row exposes the open state, and the count changes when the next page arrives.
- The query of a Releases page runs at most 4 feed detail reads at the same time. It keeps the row order of the list response.
- A Library single takes the latest `pub_date` of its tracks as its release date.
- A saved search view model loads no extra page for a window, and an open group exposes the open state.
- The window group page view model lists the singles newest first. It gives the main action of Decision 9.4 and the link to the Decision 8 group.
- A config with no window value loads with 14 days. The Settings editor accepts 1 to 60, and rejects 0 and 61. A change of the value makes the groups again in the mounted page view model.
- The guard `adr_0085_feed_page_guard_no_interface_text_names_a_feed_publisher` finds no interface string in `src/view_models`, `src/ui`, `src/app` or the views that names a feed "publisher" (ADR 0033 R-WORD-5). The frame title constant of Decision 11.7 is the exception.
- The feed page view model gives the frame title of Decision 11.7 for each row of its table. A feed with an unknown rank gets "Feed", not "Publisher".
- The artists view model gives the level of Decision 11.2 for each `role_agreement` and `album_names_as` combination. Its list rows expose no level text.
- The artist page view model gives the sections of Decision 11.3 in their sequence, with each release one time. A test uses the Sir Libre Records facts of 2026-10-18: 3 agreed `label` links, 3 conflicts of `label` and `recordLabel`, and 1 listed release.
- The release page view model gives a role row only when the agreed role set does not contain `artist`.
- The release view model gives the named feeds in their position sequence, and selects the artist feed by the rule of Decision 11.6. A test with two named feeds and a newer observation of the second feed still selects the first feed.
- The tests that name ADR 0057 in `src/view_models/artist_groups.rs` and `src/application/queries/name_lists.rs` check the group sequence and the page of each group of Decision 11.8.
- The parser gives the words, the phrases, the fields, the exclusions and the errors of a query text. `artist:jsmovie kind:single after:2026-10-01 -title:remix "night drive"` gives five terms.
- The parser keeps `Re:Union` as one word, and gives an error for `after:yesterday`.
- The Library query builder binds each value as a parameter and escapes `%`, `_` and `\`.
- A Library query with each field of Decision 22 gives the expected tracks from a recorded database.
- A query with a Library-only field gives no Index request.
- A query with an excluded `kind:`, `after:`, `before:` or `duration:` gives no Index request and the note of Decision 25.
- Each field of a query reaches its parameter, and a request holds no filter that its entity type does not accept. Tests in `src/application/queries/search.rs` and `src/view_models/search_results/search_run.rs` hold this.
- No Index request holds a field name in `q`.
- The field list view model gives the fields for an empty text, the matching fields for `ar`, and the values for `kind:`.
- The field list view model moves the selection with Up and Down, and Tab puts the selected row into the text.
- A result of an older generation does not change the search page view model.
- A saved search with field text gives the same typed query after a restart.

Visual criteria, for the operator, in Light and Dark, at normal and narrow width. [Pending human checks](../pending-human-checks.md) holds each open check.

- The app starts on Releases, and an item opens over its list.
- Back returns to the list at the same scroll position, with the row marked.
- The sidebar and the tab bar reach each destination.
- The Search page, the results page and a saved search show their rows in the sequence of Decision 3.
- The panel opens on Play, and the page does not change.
- Inspect Sources opens as a sheet.
- The singles group, the two artist levels and the logo frame.
- Releases with the sort Newest in Tiles and in Rows, with an artist that has many singles, such as jSmovie: the group tile, its logo frame, title, date span, "n Singles" badge and state word, the "+" of an open group and its change when the next page loads, and the loading state while the pager loads extra pages.
- The window group page and the link to the full group.
- The Settings row of the window length, and the change of Releases after a change of the value.
- HeyCitizen shows "Artist" and its five roles. DETOX shows "Unverified artist". No screen shows the word "publisher", apart from the frame title of Decision 11.7.
- The frame title of the Sir Libre Records page is "Label". Sir Libre Records shows "As label", "Role not confirmed" and "Listed by Sir Libre Records only". Its release shows the "Label:" row.
- The field list opens under the field on a click. The focus stays in the field while the curator moves through the list with the keys.
- The values of `artist:` and `playlist:` show and update while the curator types.
- The results change in place while the curator types, with no flash of old results.
- The error text of a field that is not correct, and the note of Decision 25 for an excluded field that the Index cannot apply.
- The list bar is directly above the rows, and the page has no band above its title. Select, act and Done need almost no mouse travel. The mode change is clear. The bar stays in view during a scroll. A narrow window wraps the bar with its two ends on separate lines.

## Item Map

Each row gives an item of a superseded ADR and its place in this ADR. "Not restated" means that no live rule stays. The Context and Alternatives Considered sections of each ADR are not restated. They stay in the archived ADR as the reason for each rule.

### ADR 0084

| Old item | New item |
| --- | --- |
| Decision 1, three columns, the collapsed sidebar and the column scrollbars | Not restated. musicindex ADR 0069 R-SHELL-1, now musicindex ADR 0080 R-SHELL-1, replaced it on 2026-10-25. Decision 1 cites R-SHELL-1. The scroll to a focused button is an Invariant |
| Decision 2, the destinations | Not restated. ADR 0089 Decision 1 replaced it. See Decision 1 |
| Decision 3, Music opens on Discover | Not restated. ADR 0089 replaced it. See Decisions 1 and 3 |
| Decision 4, one page for each item | Decision 6 |
| Decision 5, release kind | Decision 7 |
| Decision 6, singles groups and "Matched by name" | Decision 8 |
| Decision 7, two artist levels and "Matched by name" | Decision 10 |
| Decision 8, two scopes | Not restated. ADR 0089 Decision 5 replaced it. See Decision 4 |
| Decision 9, one navigation stack and the scroll position | Decision 12. The location breadcrumb is not restated |
| Decision 10, selection modes and "Select all" | Decision 13 |
| Decision 10, the download queue and the host rules | Decision 14 |
| Decision 11, audition place | Decision 16 |
| Decision 12, logo frame and artist image | Decision 17 |
| Verification | Verification |

### ADR 0085

| Old item | New item |
| --- | --- |
| Decision 1, no publisher word | Decision 11.1 |
| Decision 2, three levels | Decision 11.2 |
| Decision 3, sections | Decision 11.3 |
| Decision 4, other roles | Decision 11.4 |
| Decision 5, release page role row | Decision 11.5 |
| Decision 6, more than one named feed | Decision 11.6 |
| Decision 7, frame title, and the Status notes on the badge, R-WORD-7, R-ROLE-3 to R-ROLE-6 and R-WORD-8 | Decision 11.7 |
| Status note on musicindex ADR 0057 R-ARTIST-1 and R-SITE-15 | Decision 11.8 |
| Status note on the name level and "Name only" | Decision 10 |
| Invariants | Invariants |
| Non-Goals | Non-Goals. The storage of the position of each named feed is done (schema 19) |
| Consequences, `owned_publisher_feed_guid` uses the artist feed | Decision 11.6 |
| Consequences, each other item | Not restated. They record the change of 2026-10-18 |
| Verification | Verification |

### ADR 0086

| Old item | New item |
| --- | --- |
| Decision 1, the window chain | Decision 9.1 |
| Decision 2, the tile place | Decision 9.2 |
| Decision 3, the tile facts | Decision 9.3 |
| Decision 4, the window group page | Decision 9.4. Its breadcrumb example is not restated, because no page shows a breadcrumb |
| Decision 5, the pager cap | Decision 9.5 |
| Decision 6, saved searches | Decision 9.6 |
| Decision 7, the setting and the configuration exception | Decision 9.7 |
| Operator Answers 2026-10-23, items 1 to 5 | Decisions 9.1, 9.5, 9.6 and 9.7 |
| Invariants | Invariants |
| Non-Goals | Non-Goals |
| Verification | Verification |

### ADR 0087

| Old item | New item |
| --- | --- |
| Decision 1, query text | Decision 21 |
| Decision 2, the fields | Decision 22 |
| Decision 3, the field list | Decision 23 |
| Decision 4, live results | Decision 24 |
| Decision 5, the Index filters | Decision 25 |
| Decision 6, saved search text | Decision 3 |
| Operator Answers 2026-10-23, items 1 and 2 | Decisions 23 and 22 |
| Operator Answers 2026-10-23, item 3, the packet sequence | Not restated. It is a plan sequence |
| Operator Answers 2026-11-07, items 1 to 3, 5 and 9 | Decision 25 |
| Operator Answers 2026-11-07, item 4 | Decision 24 |
| Operator Answers 2026-11-07, item 7 | Decision 3 |
| Operator Answers 2026-11-07, items 6 and 8, passed visual checks | Not restated. They are check results |
| Known Differences 2, 3, 4, 6 and 7 | Known Differences, with the same numbers |
| Known Differences 1 and 5 | Not restated. They closed on 2026-11-07 |
| Consequences, one query builder for each field and the 50-track limit | Decision 22 |
| Consequences, each other item | Not restated |
| Invariants | Invariants |
| Non-Goals | Non-Goals |
| Verification | Verification |

### ADR 0089

| Old item | New item |
| --- | --- |
| Decision 1, one sidebar | Decision 1 |
| Decision 2, three destinations go | Decision 1 |
| Decision 3, the Inbox | Decision 2 |
| Decision 4, the Search page and the saved searches | Decision 3 |
| Decision 5, the list defaults and the scope | Decision 4 |
| Decision 6, one main action | Decision 15. The "Select" position is in Decision 13 |
| Decision 7, the now-playing panel | Decision 18 |
| Decision 8, Inspect Sources | Decision 19 |
| Decision 9, items 1 and 2, what stays of ADR 0084 and the navigation stack | Decisions 6 and 12 |
| Decision 9, items 3 to 6, ADR 0087, R-SCALE-1, the letter bar and the focus | Decisions 3, 21 to 25 and Decision 20 |
| Amended Rules, the ADR 0060 invariants | Not restated. ADR 0060 Invariants state the result |
| The Contract | Not restated. It records the stored contract 0.18.1 as context |
| Operator Answers 2026-11-05, items 1, 3, 4 and 5 | Decisions 12, 2, 4 and 4 |
| Operator Answers 2026-11-05, item 2, ADR 0068 first | Not restated. ADR 0068 is Accepted on 2026-11-11 |
| Amendment 2026-11-08, items 1 to 6, the Curator Order | Decision 5 |
| Open Points | Open Points |
| Invariants | Invariants |
| Non-Goals | Non-Goals |
| Consequences, the stored count of new results | Decision 3 |
| Consequences, each other item | Not restated. They record the change of 2026-11-05 |
| Follow-Up Work, the packets come before the new tokens | Not restated. It is a plan sequence, and the [overhaul plan](../plans/design-and-cleanup-overhaul-plan.md) holds it |
| Verification | Verification |

### ADR 0090

| Old item | New item |
| --- | --- |
| Decisions 1 to 8, the list bar | Decision 13, items 1 to 8 |
| Status, the move of each page in a packet | Decision 13 |
| Consequences, the title row of a list page | Decision 13 |
| Invariants | Invariants |
| Non-Goals | Non-Goals, and Decision 4 for the sorts of each list |
| Verification | Verification |
