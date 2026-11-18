<!-- Recorded output of ChatGPT inventory run B at commit c95aff3. Kept unchanged as evidence. It is not edited to STE. See ../README.md. -->

Recorded output of ChatGPT inventory run B at commit c95aff3a2dfd2910125146c5773ac7294b973798.

## 1. Reading this inventory

This inventory describes code paths. The app was not started. Files were not changed.

Quoted text preserves the code’s spelling, capitalization, and punctuation. Templates use braces for values supplied at runtime.

References identify source files and line numbers. “Index” means Music Index. “Library” means local Library membership.

Every count is **INFERENCE**. One click, key press, typed field, or list selection counts as one step. Automatic actions and waits count as zero. A native file selection would count as one step.

Counts assume successful requests and available controls. Additional navigation, pagination, and retries appear separately. Scrolling has no fixed count under the supplied method.

“Music open” does not identify the current destination, scope, sort, or view. Counts therefore state the required starting conditions.

**UNSURE** identifies behavior or text that the inspected code does not establish. External service results, browser behavior, and actual visual fit remain **UNSURE**.

## 2. Music navigation and narrow layout

Entry points include these destinations:

| Control label | What opens | Evidence |
|---|---|---|
| "Search" | Saved searches and search help | `src/view_models/sidebar.rs:364` |
| "Releases" | Combined Library and Index release list | `src/view_models/sidebar.rs:365` |
| "Artists" | Artist feeds and artist names | `src/view_models/sidebar.rs:366` |
| "My playlists" | Local playlists | `src/view_models/sidebar.rs:367` |
| "Public playlists" | Index playlists | `src/view_models/sidebar.rs:368` |
| "Live" | Current Index live items | `src/view_models/sidebar.rs:369` |
| "Inbox" | Library problems, updates, and setup issues | `src/view_models/sidebar.rs:370` |

The shortcut "Ctrl+1" selects Music on Linux. The registered command label is "Music". The code converts the registered `cmd-1` binding to `ctrl-1`. Evidence: `src/app/keyboard.rs:147`, `src/app/keyboard.rs:287`, `src/app/keyboard.rs:415`.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click a destination. | Its list or page replaces the current Music content. | `src/app/tab_bar.rs:26`, `src/library/app_impl.rs:3050` |
| 2 | Click an item’s title or body. | Its item page replaces the list frame when page content exists or loads. | `src/view_models/workspace/columns.rs:45`, `src/view_models/workspace/columns.rs:60` |
| 3 | Click "Back". | The preceding navigation entry returns. | `src/view_models/app_toolbar.rs:107`, `src/app/menu.rs:108` |
| 4 | Click "Forward". | The following navigation entry returns, when available. | `src/view_models/app_toolbar.rs:108`, `src/app/menu.rs:118` |

"Alt+Left" and "Alt+Right" provide Back and Forward on Linux. "Escape" also has an active-pane Back binding. Input fields, menus, and sheets can consume Escape first. Evidence: `src/app/menu.rs:103`, `src/app/menu.rs:113`, `src/app/keyboard.rs:171`, `src/app/keyboard.rs:356`.

**INFERENCE — counts:** destination opening is **1** step. Opening an item from its visible row is **1** step. Returning with "Back" adds **1**.

Below **940 px**, `SHELL_TAB_BAR_BREAKPOINT` replaces the sidebar with a bottom tab bar. At 940 px or above, the sidebar appears. Evidence: `src/ui/layouts.rs:71`.

The narrow bar directly contains "Search", "Releases", "Show", and "Inbox". "More" contains "Artists", "My playlists", "Public playlists", "Live", and "Settings". Evidence: `src/view_models/sidebar.rs:234`, `src/view_models/sidebar.rs:242`, `src/ui/composites/tab_bar.rs:109`.

**INFERENCE — narrow counts:** opening an item under "More" requires **2** steps instead of **1**. Release, Search, and Inbox destination counts stay unchanged.

The code selects one Music list frame or one item-page frame. This replacement also applies to the wide layout. Evidence: `src/view_models/workspace/columns.rs:47`.

## 3. Browse lists, change scope, sort, and view

Entry points are the destinations in section 2, their scope controls, their sort menu, and their view icons.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click "Releases" or "Artists". | The destination’s current scope, sort, and view appear. | `src/view_models/sidebar.rs:365`, `src/view_models/sidebar.rs:366` |
| 2 | Select "All". | The list includes Index results and Library facts. | `src/view_models/list_page.rs:1166`, `src/library/music_scopes.rs` |
| 3 | Select "In Library". | The list contains Library items. Its scope label can include the Library count. | `src/view_models/list_page.rs:1173` |
| 4 | Open the current sort control. | Available sort entries appear. | `src/view_models/list_page.rs:1245` |
| 5 | Select a sort entry. | The list changes order. Index requests depend on node capabilities. | `src/view_models/list_page.rs:148`, `src/view_models/list_page.rs:198` |
| 6 | Click a view icon. | The same list appears as rows or tiles. | `src/view_models/list_page.rs:109`, `src/view_models/list_page.rs:1111` |
| 7 | Click a row or tile. | Its item page opens. Selection mode changes this behavior. | `src/view_models/list_selection.rs:60` |

The view icons have no visible text label. Their accessibility labels use the template **"Show {noun} as {view}"**. The view values are exactly **"rows"** and **"tiles"**. Evidence: `src/view_models/list_page.rs:317`, `src/view_models/list_page.rs:1111`.

### 3.1 Available list controls

| List | Sort labels | Views and branches | Evidence |
|---|---|---|---|
| Releases | "Newest", "A to Z" | Rows and Tiles. Newest is the first/default sort. | `src/view_models/list_page.rs:171` |
| Artists | "A to Z", "Newest" | Rows and Tiles. Index Newest requires `artist_newest`. | `src/view_models/list_page.rs:181`, `src/view_models/list_page.rs:198` |
| My playlists | "Curator Order", "A to Z", "Newest" | Rows and Tiles | `src/view_models/list_page.rs:186`, `src/view_models/list_page.rs:109` |
| Public playlists | "Newest", "A to Z" | Rows and Tiles | `src/view_models/list_page.rs:173`, `src/view_models/list_page.rs:109` |
| Search results and saved search | "Relevance", "A to Z", "Newest" | Rows only. No view icons. | `src/view_models/list_page.rs:104`, `src/view_models/list_page.rs:178` |
| Singles group and window group | "Newest", "A to Z" | Rows and Tiles | `src/view_models/list_page.rs:175`, `src/view_models/list_page.rs:109` |
| Release section on a feed/name page | "Newest", "A to Z" | Controls appear from eight rows. Rows and Tiles. | `src/view_models/list_page.rs:177`, `src/view_models/list_page.rs:210` |
| Release tracks | Fixed "Feed Order" | Rows only. "Reverse" appears from eight rows. | `src/view_models/list_page.rs:182`, `src/view_models/list_page.rs:210`, `src/view_models/list_page.rs:889` |
| Public playlist entries | Fixed "Playlist Order" | Rows only. "Reverse" appears from eight entries. | `src/view_models/list_page.rs:183`, `src/view_models/list_page.rs:210` |
| Live | No sort menu | Rows only | `src/view_models/list_page.rs:118`, `src/view_models/list_page.rs:187` |

Reversed sort labels are exactly "Oldest", "Z to A", "Feed Order, Reversed", "Playlist Order, Reversed", and "Curator Order, Reversed". Evidence: `src/view_models/list_page.rs:267`.

Index reversal for Releases, Artists, and Public playlists requires the node’s `sort` capability. Evidence: `src/view_models/list_page.rs:148`.

The letter bar contains "A" through "Z", then "#". A letter changes the Index starting key or locates Library names. "#" supplies no Index `from` value. Evidence: `src/view_models/letter_bar.rs:48`, `src/view_models/letter_bar.rs:61`, `src/library/app_impl.rs:2978`.

**INFERENCE — counts from an open list:**

- Change scope: **1**.
- Change sort: **2**, including opening its menu.
- Change view: **1**.
- Select a letter: **1**.
- Toggle "Reverse": **1**.

No confirmation appears for these controls. List state changes are the end state.

### 3.2 Newest releases, grouping, and pagination

There is no "Discover" sidebar destination in the current destination labels. "Releases" with "Newest" supplies the newest-release path. Evidence: `src/view_models/sidebar.rs:362`, `src/view_models/list_page.rs:173`.

The newest Index request uses `GET /v1/feeds/recent`. Evidence: `src/view_models/index_name_list.rs:415`.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Open "Releases" with "All" and "Newest". | Recent Index releases and Library state appear. Singles can appear in window groups. | `src/view_models/library.rs:4313`, `src/view_models/library.rs:4368` |
| 2 | Click a release row or tile. | Its release page opens. | `src/library/app_impl.rs:3331`, `src/view_models/workspace/nav.rs:86` |
| 2a | Click a window-group row or tile. | The window-group page opens. | `src/app/search_selection.rs:215`, `src/view_models/workspace/nav.rs:101` |
| 3 | Click "All {count} singles" where supplied. | The complete singles-group page for that artist identity opens. | `src/view_models/singles_group.rs:866`, `src/library/app_impl.rs:718` |
| 4 | Click "Load more". | Another Index page loads. Existing results remain. | `src/view_models/index_name_list.rs:460` |
| 4a | Click "Try again" after failure. | The failed list request runs again. | `src/view_models/index_name_list.rs:363` |

Group disclosure labels include "Show singles" and "Hide singles". Evidence: `src/view_models/singles_group.rs:53`.

Artists also has a separate "Show more" operation for a limited artist group. It exposes more group rows and moves focus toward the new rows. Evidence: `src/view_models/artist_groups.rs:236`, `src/library/app_impl.rs:4875`.

**INFERENCE — counts:** each "Load more", "Show more", "Try again", group opening, or disclosure click adds **1**.

List error templates include:

- **"MusicIndex did not answer the {singular} list request. The list shows Library {noun} only."**
- **"MusicIndex did not answer the request for more {noun}. The list keeps the loaded {noun}."**

Evidence: `src/view_models/index_name_list.rs:443`. The error row also includes request details. Recent-release failure details can include "Recent music unavailable". Evidence: `src/library/app_impl.rs:180`, `src/view_models/index_name_list.rs:481`.

The narrow layout changes destination access as described in section 2. The list operations use the same handlers.

## 4. Select items and perform row actions

Entry controls include "Select", "Done", row checkboxes, modifier clicks, and keyboard selection.

"Select" and "Done" come from the shared selection display. Evidence: `src/view_models/library.rs:2460`.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click "Select". | Row checkboxes appear. Plain row clicks select items. | `src/view_models/list_selection.rs:60` |
| 2 | Click a checkbox or row. | The selection count and available actions appear. | `src/view_models/selection_action_bar.rs:193` |
| 3 | Click "Select all", if shown. | The selectable rows in the current list become selected. | `src/view_models/selection_action_bar.rs:165` |
| 4 | Click an available selection action. | Downloads start, a playlist chooser opens, or a removal confirmation opens. | `src/view_models/selection_action_bar.rs:212`, `src/app/search_selection.rs:405` |
| 5 | Click "Clear". | The selection clears. | `src/view_models/selection_action_bar.rs:251` |
| 6 | Click "Done". | Selection mode ends and the selection clears. | `src/library/app_impl.rs:5393` |

With no selected item, the bar reads "Select items". Selection templates include:

- **"{count} {noun} selected"**
- **"Download {count} {noun}"**
- **"Add {count} {noun} to playlist"**
- **"Remove {count} {noun}…"**

Evidence: `src/view_models/selection_action_bar.rs:201`, `src/view_models/selection_action_bar.rs:212`, `src/view_models/selection_action_bar.rs:225`, `src/view_models/selection_action_bar.rs:238`.

The download count covers downloadable items. The removal count covers removable Library items. The counts can differ within one selection. Evidence: `src/view_models/selection_action_bar.rs:122`, `src/view_models/library.rs:4665`.

A group-row selection can select all its member singles. Evidence: `src/view_models/list_selection.rs:85`.

Selection remains after a search selection action. Evidence: `src/app/search_selection.rs:387`.

### 4.1 Row menus and download controls

| Row kind | Entry points and labels | Main result | Evidence |
|---|---|---|---|
| Release row in Releases, feed page, or name page | "More actions" menu or right-click. "Download {Kind}", "Add to Playlist", "Remove {Kind}…". | Download, playlist chooser, or removal confirmation | `src/view_models/release_row_actions.rs:27`, `src/view_models/release_row_actions.rs:234`, `src/library/app_impl.rs:3336` |
| Index-only release row | "More actions" → "Download {Kind}" | Queues that release | `src/view_models/release_row_actions.rs:234`, `src/library/app_impl.rs:3344` |
| Library track row on a release page | Icon tooltip "Download", or "More actions" → "Download" | Queues that track | `src/view_models/release_page.rs:65`, `src/view_models/release_page.rs:958` |
| Index track row on a release page | Icon tooltip "Download", or "More actions" → "Download" | Queues that track | `src/view_models/release_page.rs:1016` |
| Single row on a singles-group page | Track-style download slot and menu | Downloads the single. A plain row click opens its release page. | `src/view_models/release_page.rs:912`, `src/view_models/release_page.rs:1081` |
| Search result row | Plain click, or selection controls | Opens an item page or performs a selection action | `src/ui/shells/search_result_rows.rs:144`, `src/app/search_selection.rs:405` |
| Inbox track row | "More actions" → "Inspect Sources". Removed tracks can also have "Remove from Library…". | Opens the source sheet or removal confirmation | `src/view_models/library.rs:1820`, `src/view_models/library.rs:1844` |

`{Kind}` comes from the release kind, such as "Album", "EP", or "Single". Evidence: `src/view_models/album_page.rs:54`.

Track row menus disappear in selection mode. Unavailable menu actions are absent. Evidence: `src/view_models/release_page.rs:772`.

Release-list cloud icons show status. They are not inline download buttons. Evidence: `src/ui/composites/release_download_state.rs:41`.

Track download slots are buttons only in the downloadable state. They otherwise show status icons. Evidence: `src/ui/composites/track_row.rs:332`.

Download status labels include "Not downloaded", "Queued", "Downloading", "Partly downloaded", "Downloaded", and "Failed". Partial counts use **"{downloaded} of {total}"**. Evidence: `src/view_models/release_download_state.rs:56`.

### 4.2 Keyboard entry points

These active-pane keys have registered handlers:

| Key | Registered action label | Evidence |
|---|---|---|
| Up / Down | "Move Up" / "Move Down" | `src/app/keyboard.rs:177` |
| Enter | "Open" | `src/app/keyboard.rs:189` |
| Page Up / Page Down | "Move Up a Page" / "Move Down a Page" | `src/app/keyboard.rs:195` |
| Home / End | "Move to First Row" / "Move to Last Row" | `src/app/keyboard.rs:207` |
| Space | "Select Focused Row" | `src/app/keyboard.rs:219` |
| Ctrl+A on Linux | "Select All Rows" | `src/app/keyboard.rs:225`, `src/app/keyboard.rs:287` |
| Right / Left | "Open Group" / "Close Group" | `src/app/keyboard.rs:231` |
| Shift+F10 / Menu key | "Open Row Menu" | `src/app/keyboard.rs:244` |

Ctrl-click toggles a detail selection. Shift-click extends it. Evidence: `src/view_models/detail_selection.rs:123`.

These key paths require the relevant pane or row focus. Each required focus move or key press adds **1 INFERENCE** step. The registry alone does not establish a two-dimensional tile navigation path.

## 5. MH1 Curate music: find, take, drop

This section counts from a visible row. Finding the row adds the navigation or search steps in other sections.

### 5.1 Take music

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Open a release row’s "More actions" menu. | "Download {Kind}" appears when downloadable. | `src/view_models/release_row_actions.rs:234` |
| 2 | Select "Download {Kind}". | The release enters the download path. | `src/library/app_impl.rs:3344` |
| 3 | Wait. | Membership and download state update after completion. | `src/library/app_impl.rs:3745`, `src/view_models/release_download_state.rs:56` |

**INFERENCE — counts from each row kind:**

| Start point | Path | Count |
|---|---|---:|
| Release row in Releases | "More actions" → "Download {Kind}" | **2** |
| Release row on an artist/feed/name page | Same menu path | **2** |
| Release tile without a direct menu path | Open tile → "Download {Kind}" on its page | **2** |
| Search release row | Open row → "Download {Kind}" | **2** |
| Downloadable track row on a release page | Click the "Download" icon | **1** |
| Same track row through its menu | "More actions" → "Download" | **2** |
| Single row on a singles-group page | Click the "Download" icon | **1** |
| Search track row | Open row → "Download Release", or the known release-kind variant | **2** |
| Public playlist entry | Open entry → page download action | **2**, if the entry has a target |
| Live row | Open row → release-page download action | **2**, if the row resolves to a release |
| Inbox row | Open the track page first. Availability depends on its Library action state. | **UNSURE** as one universal acquisition count |

An Index track page downloads its release through its main action. A track row on an Index release page can download one track. Evidence: `src/view_models/track_detail.rs:620`, `src/view_models/release_page.rs:1016`.

No acquisition confirmation appears on these download paths.

Downloads can fail before work starts. Text includes:

- **"The track is no longer stored, so the app cannot download it."**
- **"The app could not read the track from the database. {error:#}"**
- **"the app has no feed GUID to read the track from Music Index."**
- **"the app could not read the release from Music Index. {error}"**
- **"Music Index gave the release {feed_guid} with no track {track_guid}."**

Evidence: `src/library/downloads.rs:305`, `src/library/downloads.rs:314`, `src/library/downloads.rs:350`, `src/library/downloads.rs:563`, `src/library/downloads.rs:603`.

The download progress item reports the job result. A mounted Library track page also receives **"Download error: {error:#}"**. Evidence: `src/library/app_impl.rs:3761`, `src/view_models/library.rs:3141`.

End state: successful downloads give Library membership and an audio file. A failed or partial release can retain a partial or failed state. Queue internals are outside this run.

### 5.2 Drop music from the Library

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Open "More actions". | The available removal item appears. | `src/view_models/release_row_actions.rs:248`, `src/view_models/release_page.rs:835` |
| 2 | Select "Remove {Kind}…", "Remove…", or "Remove from Library…". | A Library removal confirmation opens. | `src/library/app_impl.rs:3361`, `src/library/app_impl.rs:949` |
| 3 | Click "Remove". | Removal starts. | `src/view_models/library_removal.rs:67`, `src/view_models/library_removal.rs:77` |
| 3a | Click "Cancel". | The item stays in the Library. | `src/view_models/library_removal.rs:65` |

Confirmation titles are "Remove Track from Library?" and "Remove Album from Library?". Evidence: `src/view_models/library_removal.rs:62`.

The explanation changes with playlist and Show cue references. One template is:

**"This track is in {playlist_reference_count} {playlist_label}. Removing it from the library will make it unavailable for playlist playback."**

Evidence: `src/view_models/library_removal.rs:105`.

Selection removal uses **"Remove {count} {noun} from the Library?"**, with "Cancel" and "Remove". Evidence: `src/view_models/library_removal.rs:258`.

**INFERENCE — counts:**

| Start point | Count |
|---|---:|
| Removable release row with its menu | **3** |
| Removable track row with its menu | **3** |
| Removed-from-feed Inbox row | **3** |
| Search Library result row | **4**: open page, open menu, select removal, confirm |
| Release tile through its page | **4** |
| Item page already open | **3** |
| Open list, select one item, remove it | **4**: "Select", checkbox, removal action, "Remove" |
| Open list, select `n` items, remove them | **n + 3** |

A pending download can make removal unavailable. Evidence: `src/view_models/release_page.rs:956`, `src/library/downloads.rs:221`.

Removal changes Library membership. Existing audio files move to the audio cache. Missing source files are skipped during that move. Evidence: `src/application/library_removal.rs:239`, `src/audio_cache.rs:39`.

Failure text includes **"Error removing track: {error:#}"** and selection-report template **"The app removed {removed} of … It could not remove: …."** Evidence: `src/view_models/library.rs:5447`, `src/view_models/library_removal.rs:363`.

The narrow layout adds only destination-access steps. Row and confirmation paths use the same commands.

## 6. Search, live results, fields, and failure notice

Entry points are:

- The toolbar field, placeholder "Search Library and Index". Evidence: `src/view_models/app_toolbar.rs:124`.
- The toolbar button "Search". Evidence: `src/view_models/app_toolbar.rs:126`.
- "Ctrl+F" and "Ctrl+Alt+F" on Linux. Their registered labels are "Find" and "Focus Search". Evidence: `src/app/keyboard.rs:129`, `src/app/keyboard.rs:135`, `src/app/keyboard.rs:287`.
- The "Search" destination. Evidence: `src/view_models/sidebar.rs:364`.
- "New Saved Search", which focuses the search field. Evidence: `src/view_models/search_results/saved_search.rs:589`.
- An example query under "Example searches". Evidence: `src/view_models/search_page.rs:147`.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click the search field or use its shortcut. | The field gets focus. The field list can open. | `src/app/keyboard.rs:395`, `src/app/search_field_list.rs:100` |
| 2 | Type one query. | After a 300 ms pause, Library and Index search runs. Focus stays in the field. | `src/app/search_live.rs:3`, `src/app/search_live.rs:154`, `src/ui/tokens.rs:1677` |
| 2a | Press Enter or click "Search". | Search runs without waiting for the typing pause. | `src/app/search_live.rs:6`, `src/view_models/app_toolbar.rs:126` |
| 3 | Select "All", "Releases", "Tracks", "Artists", or "Labels". | The selected result category appears. | `src/view_models/search_results/tabs.rs:50` |
| 4 | Click a result. | A release, track, feed/artist, name, or group page opens. | `src/app/search_selection.rs:198`, `src/app/search_dispatch.rs:739` |
| 5 | Click "Back". | The search location returns. | `src/view_models/app_toolbar.rs:107`, `src/app/search_live.rs:212` |

Tab text can use **"{tab label} {count}"**. A partial count uses **"{count}+"**. Evidence: `src/view_models/search_results/tabs.rs:97`.

Library and Index results with the same identity merge in the result list. Library rows retain their place. Evidence: `src/view_models/search_results/paged_tab.rs:31`.

Search has no Tiles control. Evidence: `src/view_models/list_page.rs:104`.

### 6.1 Field-list branch

Clicking a field, value, or example inserts query text. Generated field text uses templates such as **"{token}{quoted value}"**. Evidence: `src/view_models/search_field_list.rs:595`, `src/view_models/search_field_list.rs:611`.

Up, Down, Tab, Enter, and Escape can act on the field list before the input receives them. Evidence: `src/app/search_field_list.rs:445`.

Field-list messages include:

- "Not available with in:index"
- "Not available with a Library-only field or in:library"
- "Loading names from Music Index"
- "Names from Music Index did not load"
- "The Library names show. The next change of the text tries again."
- "This query asks for the Library only and for Music Index only. It shows no rows."

Evidence: `src/view_models/search_field_list.rs:35`.

**INFERENCE — counts:** field focus plus typing is **2** steps for live results. Explicit submission adds **1**. Each field-list selection adds **1**.

### 6.2 Scope, empty results, and failures

Selecting "All" or "In Library" changes the query text and starts another live run. Evidence: `src/app/search_live.rs:164`.

Empty results use **"No {noun}"** and **"No {noun} matched \"{query}\"."** Evidence: `src/view_models/search_results/empty_state.rs:37`.

The failure notice can appear as the empty state or beside surviving results. Its controls are "Show details", "Hide details", and "Copy report". Evidence: `src/ui/shells/search_results_inspector.rs:703`, `src/view_models/search_results/failure.rs:108`.

| Failure branch | Exact text or template | Evidence |
|---|---|---|
| Rejected query | "Music Index did not accept the query" | `src/view_models/search_results/failure.rs:33` |
| Rejected-query explanation | **"Music Index did not accept the query \"{query}\". Change the query, then search again."** | `src/view_models/search_results/failure.rs:54` |
| Runtime unavailable | "App could not start the Music Index search because its background runtime is unavailable. No search request was sent." | `src/view_models/search_results/failure.rs:62` |
| Endpoint/setup unavailable | "App could not start the Music Index search because its endpoint configuration or setup is unavailable. No search request was sent." | `src/view_models/search_results/failure.rs:66` |
| Required tool unavailable | "App could not start the Music Index search because a required tool is unavailable. No search request was sent." | `src/view_models/search_results/failure.rs:70` |
| Cancelled | "App cancelled the Music Index search before it completed." | `src/view_models/search_results/failure.rs:74` |
| Request failure | "App could not get search results from Music Index." | `src/view_models/search_results/failure.rs:78` |

The summary appends "Your local library remains available." and a branch-specific next-action explanation. Evidence: `src/view_models/search_results/failure.rs:82`.

The failure notice itself renders only the details and copy controls. Settings repair actions mentioned in its text use separate capability surfaces. Evidence: `src/ui/shells/search_results_inspector.rs:731`.

**INFERENCE — counts from a visible failure:** show details **1**, copy report **1**, or resubmit the unchanged field **1**. Correcting the query adds one typed-field step.

End state: results, an empty result, or a failure notice with surviving Library access.

The narrow layout uses the same toolbar search and result handlers.

## 7. Save, open, edit, rename, duplicate, and delete searches

### 7.1 Save and reopen

Entry controls are "Save search", "Search", and the saved-search name.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click "Save search" on current results. | The query is stored. Its action becomes "Saved". | `src/view_models/search_results/saved_search.rs:147`, `src/app/saved_search_dispatch.rs:32` |
| 2 | Click the "Search" destination. | Saved-search rows appear. | `src/view_models/search_page.rs:130` |
| 3 | Click a saved-search name. | The saved query runs on its saved-search page. | `src/app/saved_search_dispatch.rs:84` |

Saved-search names are stored text. New-result badges use text such as "3 new". Opening results marks loaded result identities as seen. Evidence: `src/view_models/search_page.rs:344`, `src/app/saved_search_dispatch.rs:99`.

**INFERENCE — counts:** save from current results **1**. Open a saved search from Music **2**. Search-and-save from an unfocused toolbar field **3**.

Save failure uses **"The app could not save the search \"{query}\". Database error: {error}"**. Evidence: `src/view_models/search_results/mod.rs:490`.

### 7.2 Saved-search row menu

The trigger tooltip is "More actions". Menu labels are "Edit Query", "Rename…", "Duplicate", and "Delete Saved Search…". Evidence: `src/view_models/search_page.rs:36`, `src/view_models/search_results/saved_search.rs:524`.

| Journey | Main path | What appears next or end state | Evidence |
|---|---|---|---|
| Edit query | "More actions" → "Edit Query" → type query → "Save Query" | The query runs live while edited, then becomes stored. | `src/view_models/search_results/saved_search_query.rs:4`, `src/view_models/search_results/saved_search_query.rs:146` |
| Edit from open page | Click the displayed query → type query → "Save Query" | The same saved-query editor operates. | `src/app/saved_search_query_edit.rs:292` |
| Cancel query edit | Click "Cancel" or press Escape | Stored query text returns. | `src/view_models/search_results/saved_search_query.rs:7`, `src/view_models/search_results/saved_search_query.rs:153` |
| Rename | "More actions" → "Rename…" → type "Saved search name" → "Save" | The saved name changes. | `src/view_models/search_results/saved_search.rs:532`, `src/view_models/search_results/saved_search.rs:574` |
| Duplicate | "More actions" → "Duplicate" | Another saved-search record is created. | `src/view_models/search_results/saved_search.rs:540`, `src/library/saved_searches.rs:191` |
| Delete | "More actions" → "Delete Saved Search…" → "Delete" | The record is deleted. | `src/view_models/search_results/saved_search.rs:548`, `src/view_models/search_results/saved_search.rs:193` |

**INFERENCE — counts from a visible saved-search row:** edit **4**, rename **4**, duplicate **2**, delete **3**.

**INFERENCE — counts from an open saved-search page:** query edit **3**. Page deletion through "Delete…" and confirmation is **2**.

The deletion title is "Delete saved search?". Its body uses:

**"Delete the saved search \"{query}\"? The Library and the downloads do not change."**

Buttons are "Cancel" and "Delete". Evidence: `src/view_models/search_results/saved_search.rs:193`.

Error texts include:

- "A saved search needs a query. Type a word, a phrase or a field."
- **"The app could not save the query: {error}"**
- "Another saved search has this name. Choose another name."
- **"The app could not rename the saved search: {error:#}"**
- **"The app could not duplicate the saved search: {error:#}"**
- **"The app could not delete the saved search \"{query}\". Database error: {error}"**

Evidence: `src/view_models/search_results/saved_search_query.rs:19`, `src/view_models/search_results/saved_search_query.rs:122`, `src/view_models/search_results/saved_search.rs:489`, `src/library/saved_searches.rs:154`, `src/library/saved_searches.rs:205`, `src/view_models/search_results/mod.rs:499`.

A deleted historical page can show "Saved search deleted" and "This saved search was deleted.". Evidence: `src/view_models/search_results/saved_search.rs:252`.

The narrow layout retains direct access to "Search". These paths add no "More" destination step.

## 8. Release item page

Entry points are release rows or tiles, search release results, release links on feed/name pages, and release-target public-playlist entries.

Their visible labels come from the item title. The page action labels use the release kind. Evidence: `src/view_models/release_page.rs:1126`, `src/view_models/album_page.rs:54`.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click the release title or tile. | One release page opens for the resolved Library or Index origin. | `src/view_models/release_page.rs:1126` |
| 2 | Read the header and tracks. | Facts can include "Release Kind", "Published", "First track published", "Tracks", "Duration", and "Language". | `src/view_models/entity_detail.rs:797` |
| 3 | Click "Download {Kind}" when offered. | The release enters the download path. | `src/view_models/album_page.rs:56`, `src/library/release_page.rs:99` |
| 3a | Click "Add to Playlist" when offered. | The playlist chooser opens. | `src/view_models/album_page.rs:57` |
| 4 | Click a track row. | Its track page opens. | `src/view_models/release_page.rs:893` |
| 5 | Click a linked artist or feed name. | Its artist/name/feed page opens. | `src/view_models/album_page.rs:203`, `src/app/search_dispatch.rs:1828` |

The release page shows all its track rows independently of a list filter. Evidence: `src/view_models/release_page.rs:162`.

The page menu’s accessibility label is "More actions for this album". It can contain:

- "Copy Feed URL"
- "MusicBrainz Lookup"
- "Remove {Kind}…"

Evidence: `src/view_models/album_page.rs:14`, `src/view_models/album_page.rs:58`.

Index-origin release pages offer download and feed-URL copying. Library membership and active work determine the other actions. Evidence: `src/view_models/album_page.rs:144`.

"Copy Feed URL" copies the URL and can change the menu trigger to "Copied". Evidence: `src/view_models/album_page.rs:33`.

"Description" can expand or collapse. Long descriptions can include "Show more". Evidence: `src/view_models/entity_detail.rs:860`, `src/view_models/library.rs:251`.

The "Identity" content can expose "Website", "Copy Nostr", "RSS", and "Open on musicindex.org". Evidence: `src/view_models/entity_detail.rs:696`, `src/view_models/entity_detail.rs:903`.

**INFERENCE — counts from a visible release row:**

- Open page: **1**.
- Open page and download: **2**.
- Open page and copy feed URL through its menu: **3**.
- Open page and start MusicBrainz lookup through its menu: **3**.
- Open page and remove it: **4**.

Index detail failure uses **"The app could not load this {kind} from Music Index. {message}"**. Loading uses **"Loading this {kind} from Music Index."** Evidence: `src/view_models/search_results/index_detail.rs:122`.

End states include an open release page, a download result, a copied URL, or removal. The narrow shell changes destination access, not these commands.

## 9. Track item page

Entry points include track rows on release pages, search track results, Inbox rows, and public-playlist track entries.

A single row on a singles-group page opens its release page instead. Its "Inspect Sources" menu opens the track page. Evidence: `src/view_models/release_page.rs:852`, `src/view_models/release_page.rs:917`.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click the track title. | The track page opens. | `src/view_models/release_page.rs:893`, `src/app/search_dispatch.rs:739` |
| 2 | Click "Download Track" on an undownloaded Library-backed track. | That track enters the download path. | `src/view_models/track_detail.rs:635`, `src/library/track_page.rs:31` |
| 2a | Click "Download Release" on an Index track. | Its containing release enters the download path. | `src/view_models/track_detail.rs:620`, `src/app/search_dispatch.rs:1652` |
| 3 | Click "Add to Playlist" when offered. | The playlist chooser opens. | `src/view_models/track_detail.rs:643`, `src/library/track_page.rs:49` |
| 4 | Open the page menu. | Available source, copy, MusicBrainz, and removal actions appear. | `src/view_models/track_page.rs:364` |
| 5 | Click a release or artist/feed name link. | Its corresponding page opens. | `src/view_models/track_detail.rs:1020`, `src/library/track_page.rs:58` |

The Index download label can name the release kind, such as "Download Single". Without that fact, it is "Download Release". Evidence: `src/view_models/track_detail.rs:935`.

Menu labels include "Copy Feed URL", "MusicBrainz Lookup", "Inspect Sources", and "Remove Track…". Evidence: `src/view_models/track_detail.rs:942`.

The Index track page offers no "Inspect Sources", MusicBrainz, playlist, or removal action. Evidence: `src/app/search_dispatch.rs:1682`.

Copying an Index track’s feed URL can require reading its album. The menu trigger can show "Copying…", "No feed URL", or "Copy failed". Evidence: `src/view_models/track_page.rs:192`.

Copy failure explanations are:

- "Music Index gave no feed URL for the album of this track."
- **"App could not read the album of this track from Music Index: {error}"**

Evidence: `src/view_models/track_page.rs:204`.

**INFERENCE — counts from a visible track row:** open **1**, open and download **2**, open and inspect sources **3**, open and start MusicBrainz lookup **3**, open and remove **4**.

Track-page facts include "Release", "Artist", "Track #", "Duration", "Release Date", "Feed publication date", and "Feed owner". Evidence: `src/view_models/track_detail.rs:57`.

End state is the open track page, with any command result. The narrow layout uses the same page commands.

## 10. Artist, name-level, publisher, and entity pages

### 10.1 Artist/feed page

Entry points include an "Artists" row or tile, a search artist/label/feed result, and linked feed names on item pages.

Visible entry labels use the supplied artist/feed title. Evidence: `src/view_models/publisher_page.rs:1308`, `src/view_models/track_detail.rs:486`.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click an artist/feed name. | Its feed page loads. | `src/app/publisher_dispatch.rs`, `src/view_models/publisher_page.rs:445` |
| 2 | Read its title, confirmation, and roles. | The header identifies the feed’s interpreted role. | `src/view_models/publisher_page.rs:635`, `src/view_models/publisher_page.rs:819` |
| 3 | Open a release section. | Available sections include "Albums and EPs", "Singles", and "Matched by name". | `src/view_models/publisher_page.rs:674` |
| 4 | Click a release or use its row menu. | The release page opens, or its download/removal command starts. | `src/view_models/publisher_page.rs:1191`, `src/library/app_impl.rs:3331` |
| 5 | Click a related feed. | Another feed page opens. | `src/ui/shells/publisher.rs:74` |

Other possible headings include:

- "Playlists"
- "Role not confirmed"
- **"As {role list}"**
- **"Listed by {feed title} only"**
- "Artists on these releases"
- "Labels and other feeds"
- "Shares releases with"

Evidence: `src/view_models/publisher_page.rs:649`, `src/view_models/publisher_page.rs:685`, `src/view_models/publisher_page.rs:997`, `src/view_models/publisher_page.rs:1127`.

This page supports release selection across its release sections. Evidence: `src/app/detail_selection.rs:126`.

Failure text includes:

- "The app could not load this feed page from Music Index."
- "The app could not load the other releases of this feed from Music Index. The releases in the Library stay on this page."

Evidence: `src/view_models/publisher_page.rs:418`, `src/view_models/publisher_page.rs:445`.

**INFERENCE — count from Music:** "Artists" → artist row is **2** wide or **3** narrow. A visible linked name opens the page in **1**.

### 10.2 Name-level page

A name link can resolve to a matched feed page or a name-level page. Evidence: `src/app/search_dispatch.rs:1824`.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click an artist name without a resolved feed identity. | The page title uses "Releases that name {name}". | `src/view_models/name_level_page.rs:390` |
| 2 | Read "Name only" and "About this page". | The page explains its name-based relation. | `src/view_models/name_level_page.rs:360` |
| 3 | Click a release. | Its release page opens. | `src/view_models/name_level_page.rs:277` |
| 4 | Click "Open the artist page", if a matched feed becomes available. | The matched artist feed page opens. | `src/view_models/name_level_page.rs:380`, `src/view_models/name_level_page.rs:566` |

The note is "A name is not an identity. These releases state this artist text and name no artist feed.". Evidence: `src/view_models/name_level_page.rs:365`.

A row’s inferred relation uses **"names \"{artist_text}\", no feed"**. Evidence: `src/view_models/name_level_page.rs:180`.

Loading text is "Loading the releases of this name from Music Index.". Failure text is:

"The app could not load the releases of this name from Music Index. The releases in the Library stay on this page."

Evidence: `src/view_models/name_level_page.rs:372`.

The page can report "Music Index has more releases for this name. This page shows the first 1,000.". Evidence: `src/view_models/name_level_page.rs:378`.

**INFERENCE — counts:** open from a visible name **1**. Follow its matched-artist notice **1** additional step.

### 10.3 Publisher and generic entity coverage

`PublisherDetail` is the feed-page navigation entry. Its UI can identify an artist, label, another role, "Publisher", or "Feed". Evidence: `src/view_models/workspace/nav.rs:95`, `src/ui/shells/publisher.rs:116`, `src/view_models/publisher_page.rs:697`.

The inspected navigation enum contains no separate generic “entity page” destination. Release, track, name, and feed pages are the identifiable paths. Evidence: `src/view_models/workspace/nav.rs:70`.

Contributor rows show supplied names and optional website/Nostr actions. Their row body has no navigation handler in the contributor renderer. Evidence: `src/ui/shells/entity.rs:260`.

**UNSURE:** a separate user journey called “entity page” could not be identified beyond these concrete page types.

## 11. Inspect Sources and MusicBrainz lookup

### 11.1 Inspect Sources

Entry points are:

- A Library track page menu item "Inspect Sources". Evidence: `src/view_models/track_detail.rs:944`.
- A track row’s "More actions" → "Inspect Sources". Evidence: `src/view_models/release_page.rs:807`.
- An Inbox row’s "More actions" → "Inspect Sources". Evidence: `src/view_models/library.rs:1844`.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Select "Inspect Sources". | The track page opens if needed. The "Inspect Sources" sheet opens. | `src/library/app_impl.rs:4105`, `src/library/app_impl.rs:4121` |
| 2 | Click "Compare ID3". | Embedded tags load. The control can read "Reading ID3...". | `src/ui/composites/inspect_sources_sheet.rs:122`, `src/library/app_impl.rs:3962` |
| 3 | Read or expand metadata groups and cells. | RSS, file-tag, and available MusicBrainz values appear separately. | `src/view_models/track_metadata_grid.rs:529`, `src/library/app_impl.rs:3807` |
| 4 | Click "Apply tags ({count})", when offered. | Pending ID3 edits are written. | `src/view_models/entity_detail.rs:435`, `src/library/app_impl.rs:3827` |
| 4a | Click "Discard staged". | Explicit pending edits clear. | `src/view_models/entity_detail.rs:447`, `src/library/app_impl.rs:3898` |
| 5 | Close the sheet. | Its Sources-expanded state closes. | `src/ui/composites/inspect_sources_sheet.rs:49`, `src/library/app_impl.rs:4143` |

The app automatically derives pending edits for the apply operation. It blocks duplicate effective targets. Evidence: `src/library/app_impl.rs:3840`.

Messages include:

- "Reading embedded metadata..."
- **"{count} staged tag edit{s}"**
- "Applying tags…"
- **"Resolve duplicate ID3 target{s}: {conflicts}"**
- **"Error applying ID3 edits: {error}"**

Evidence: `src/view_models/entity_detail.rs:288`, `src/view_models/entity_detail.rs:429`, `src/view_models/entity_detail.rs:307`.

**INFERENCE — counts from a visible Library track row:** menu → "Inspect Sources" is **2**. Compare adds **1**. Apply adds **1**, for **4** total when edits are available.

The file panel contains "Re-read" and "Re-download". Both call the same comparison reload function. "Re-download" does not directly queue an audio download in that handler. Evidence: `src/view_models/entity_detail.rs:299`, `src/library/app_impl.rs:3992`.

The sheet requests a fixed `INSPECT_SOURCES_SHEET_WIDTH`. The inspected opening code has no separate narrow branch. Actual narrow-window fit is **UNSURE**. Evidence: `src/ui/composites/inspect_sources_sheet.rs:44`.

### 11.2 Track MusicBrainz lookup

Entry point: Library track page menu → "MusicBrainz Lookup". Evidence: `src/view_models/track_detail.rs:943`.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Open the track page menu. | "MusicBrainz Lookup" appears with its availability. | `src/view_models/track_detail.rs:648` |
| 2 | Select "MusicBrainz Lookup". | The source sheet opens and the lookup starts or its retained panel appears. | `src/library/track_page.rs:37`, `src/library/app_impl.rs:4220` |
| 3 | Open the "MusicBrainz: {trigger_value}" picker. | Candidate releases appear. | `src/view_models/musicbrainz_panel.rs:50`, `src/ui/composites/musicbrainz_panel.rs:128` |
| 4 | Select a candidate. | That candidate’s metadata appears. | `src/library/app_impl.rs:4239` |
| 5 | Use the tag-compare/apply path if edits are available. | File tags change through the explicit apply action. | `src/library/app_impl.rs:3827` |

Loading text is "Searching MusicBrainz...". Empty text includes "No MusicBrainz release" and "No MusicBrainz recording match found". Evidence: `src/view_models/entity_detail.rs:293`, `src/view_models/musicbrainz_panel.rs:35`, `src/ui/composites/musicbrainz_panel.rs:109`.

Lookup requires a local subscription and recorded local path. Evidence: `src/library/app_impl.rs:4045`.

**INFERENCE — counts from an open track page:** start lookup **2**. Open the picker and select another candidate adds **2**.

**UNSURE:** actual MusicBrainz matches and remote error content depend on the service.

### 11.3 Release MusicBrainz lookup

Entry point: Library release page menu → "MusicBrainz Lookup". Evidence: `src/view_models/album_page.rs:59`.

The app processes downloaded tracks. It first attempts an album lookup, then can fall back to per-track lookup. It retains staged results for track pages. Evidence: `src/library/app_impl.rs:4290`, `src/library/app_impl.rs:4326`, `src/library/app_impl.rs:3792`.

Messages include:

- "No downloaded tracks to process"
- **"MusicBrainz: album lookup for {count} tracks..."**
- **"Album lookup failed ({error:#}), falling back to per-track..."**
- "Album lookup: no results, falling back to per-track..."
- **"MusicBrainz: staged {total_edits} edit{s} across {processed} tracks"**

Evidence: `src/view_models/library.rs:5738`.

**INFERENCE — count from an open release page:** **2** to start lookup. Writing resulting track edits requires the separate source/compare/apply path.

## 12. Inbox, feed checks, updates, and tag writes

### 12.1 Open Inbox

Entry point: "Inbox" in the sidebar or narrow tab bar. Evidence: `src/view_models/sidebar.rs:370`, `src/view_models/sidebar.rs:234`.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click "Inbox". | The app requests a fresh Library readiness report. | `src/library/app_impl.rs:1027` |
| 2 | Read the sections. | "Tracks", "Updates", and "Setup" appear when supplied. | `src/view_models/inbox.rs:30` |
| 3 | Click a track’s fix, update action, setup action, or row. | The corresponding command or item page opens. | `src/ui/shells/library/inbox.rs:77`, `src/library/app_impl.rs:837` |

The track section places tracks from the show playlist first. The Inbox count includes track problems, updates, and setup items. Evidence: `src/view_models/inbox.rs:182`, `src/view_models/library.rs:5331`.

**INFERENCE — count from Music:** **1** to open Inbox in either layout.

### 12.2 Check all feeds

Entry point: "Check all feeds" at the bottom of the wide sidebar. In narrow layout, the same command appears under "More". Evidence: `src/view_models/library.rs:5418`, `src/library/app_impl.rs:5762`.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click "Check all feeds". | The control changes to "Checking...". Status uses "Checking {feed_count} feeds...". | `src/view_models/library.rs:5415`, `src/view_models/library.rs:5864` |
| 2 | Wait. | The app checks feeds, applies stale Index updates, then repairs missing payment-route tags. | `src/application/commands/feed.rs:427`, `src/application/commands/feed.rs:440`, `src/application/commands/feed.rs:452` |
| 3 | Read the result. | The sidebar result reports checked feeds, changes, and route-repair outcomes. | `src/view_models/library.rs:6001`, `src/library/app_impl.rs:5950` |

The combined command clears the pending stale list after completion. It does not leave its own stale results waiting for "Apply updates". Evidence: `src/view_models/library.rs:5871`.

Result text starts with **"Checked {count} feed{s}"**. It appends change, update-issue, and route-repair clauses. Evidence: `src/view_models/library.rs:6001`.

Failure uses **"Feed check error: {message:#}"**. Evidence: `src/view_models/library.rs:5858`.

Some individual staleness-check errors are skipped by the batch loop. That loop does not return each ordinary check failure as an error. Evidence: `src/application/commands/feed.rs:275`.

**INFERENCE — count from Music:** **1** wide, **2** narrow.

### 12.3 Pending updates and "Apply updates"

Opening a Library release can automatically check that feed. This separate path can add a pending stale entry. Evidence: `src/library/app_impl.rs:2713`, `src/library/app_impl.rs:3072`, `src/view_models/library.rs:5822`.

Entry labels are:

- Sidebar/menu: **"Apply updates ({count})"**.
- Inbox: "Apply updates".
- State text: "Update available".

Evidence: `src/view_models/library.rs:5408`, `src/view_models/inbox.rs:182`, `src/view_models/library.rs:127`.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click the available apply action. | Status uses "Applying updates to {count} feed(s)...". | `src/view_models/library.rs:5881` |
| 2 | Wait. | Stored Index metadata updates. A tag-difference scan follows. | `src/library/app_impl.rs:3151` |
| 3 | Read the result. | The pending list clears and the result message appears. | `src/view_models/library.rs:5894` |

This apply path changes stored metadata without writing the general audio tags. Its result templates include:

- "Stored no Music Index track record. No audio file changed"
- **"Stored the Music Index record of {tracks_refreshed} track{s}. No audio file changed"**
- **"Feed errors ({count}): {errors}"**
- **"Feed update error: {error:#}"**

Evidence: `src/application/commands/feed.rs:762`, `src/view_models/library.rs:5900`.

**INFERENCE — counts:** apply from a visible wide-sidebar action **1**. Narrow "More" → apply **2**. Music → "Inbox" → "Apply updates" **2**.

### 12.4 Write changed file tags

Entry controls use **"Update {files}"**, where `{files}` is "1 file" or "{count} files". They appear in the sidebar, Inbox Updates, or status bar. Evidence: `src/view_models/tag_update.rs:71`, `src/view_models/inbox.rs:196`, `src/view_models/status_bar.rs:190`.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click "Update {files}". | A confirmation titled "Update {files}?" lists files and proposed frames. | `src/view_models/tag_update.rs:134`, `src/library/app_impl.rs:1472` |
| 2 | Click "Write Tags". | The app writes the listed eligible files. | `src/view_models/tag_update.rs:150`, `src/library/app_impl.rs:1486` |
| 2a | Click "Cancel". | No listed file is written. | `src/view_models/tag_update.rs:148` |
| 3 | Wait. | Counts and readiness refresh. A report records written and unwritten files. | `src/library/app_impl.rs:1433`, `src/view_models/tag_update.rs:304` |

The action is absent when the scan finds no differing file. It becomes **"Updating {files}"** during writing. Evidence: `src/view_models/tag_update.rs:57`.

Files can carry "In use by the show" or "Cannot read". Unreadable details use **"The app cannot read the tags of this file: {error}"**. Evidence: `src/view_models/tag_update.rs:24`, `src/view_models/tag_update.rs:167`.

Report and error templates include:

- **"Tag update at {at}: the app wrote {files}."**
- **"Wrote {frames} to {title}."**
- **"Did not write {title}. The file is in use by the show."**
- **"Did not write {title}. The app cannot read its tags."**
- **"Could not write {title}. {error}"**
- "The tag update wrote no file. The background runtime did not accept the request."
- **"The app could not compare the file tags with the stored metadata. {error}"**

Evidence: `src/view_models/tag_update.rs:247`, `src/view_models/tag_update.rs:267`, `src/view_models/tag_update.rs:304`.

**INFERENCE — counts:** visible update control → "Write Tags" is **2**. Music → "Inbox" → update → "Write Tags" is **3**.

The status-bar update control stays available in the narrow shell. Its handler opens the same popup. Evidence: `src/app/status_bar.rs:111`.

### 12.5 Playlist RSS-check branch

Entry point: "Check RSS" on a playlist page. Evidence: `src/view_models/playlist_rss_check.rs:74`.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click "Check RSS". | The playlist RSS check runs. | `src/library/app_impl.rs:1530` |
| 2 | Wait. | An "RSS check" report appears. Stored RSS changes can be applied automatically. | `src/view_models/playlist_rss_check.rs:218`, `src/view_models/playlist_rss_check.rs:423` |
| 3 | Click "Copy feed URL" or "Open podping.me" for a reported stale feed. | The URL is copied or podping.me opens externally. | `src/view_models/playlist_rss_check.rs:475`, `src/view_models/playlist_rss_check.rs:497` |
| 4 | Use "Update {files}" → "Write Tags" if tag differences remain. | Eligible file tags update. | `src/library/app_impl.rs:1402` |

The RSS report states that it writes stored values and no audio tag. Evidence: `src/view_models/playlist_rss_check.rs:423`.

HTTP 429 reporting includes **"The check stopped requests to {host} after HTTP 429 (Too Many Requests)."** Retry-After branches add the requested wait. Evidence: `src/view_models/playlist_rss_check.rs:401`.

If the actor is unavailable, Library status uses **"Error starting the RSS check: {error:#}"**. Evidence: `src/library/app_impl.rs:1541`, `src/view_models/library.rs:5461`.

**INFERENCE — count from Music:** "My playlists" → playlist → "Check RSS" is **3** wide or **4** narrow. Tag confirmation adds **2**.

**UNSURE:** podping.me submission and the Index’s subsequent response are external behavior.

## 13. Baseline Job 2: take a new release from Discover

Start: **Music open**.

The current destination is "Releases". No "Discover" destination appears in the current sidebar labels. Evidence: `src/view_models/sidebar.rs:362`.

Assumptions: Releases retains "All" and "Newest". The chosen item is a standalone downloadable release.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click "Releases". | The newest release list appears. | `src/view_models/sidebar.rs:365`, `src/view_models/list_page.rs:173` |
| 2 | Open the chosen row’s "More actions". | "Download {Kind}" appears. | `src/view_models/release_row_actions.rs:27`, `src/view_models/release_row_actions.rs:234` |
| 3 | Select "Download {Kind}". | The release download starts. | `src/library/app_impl.rs:3344` |
| — | Wait. | Successful completion gives Library membership. | `src/library/app_impl.rs:3745` |

**INFERENCE — count: 3**, wide or narrow.

Alternative: open the release row or tile, then click its "Download {Kind}" action. **INFERENCE — count: 3** from Music.

Branches:

- Selecting "All" adds **1 INFERENCE** step.
- Changing to "Newest" adds **2 INFERENCE** steps.
- Each "Load more" or retry adds **1 INFERENCE** step.
- A window group opens its group page. Individual or missing-single downloads then follow that page’s controls.
- Download errors and end states are in section 5.

The literal historical "Discover" path is unavailable as a sidebar path. The newest-release job remains possible through "Releases".

## 14. Baseline Job 3: search an artist, take three chosen singles

Start: **Music open**.

Assumptions: the artist result exists, its page lists the three chosen singles, and those singles are downloadable.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Focus "Search Library and Index". | The toolbar field has focus. | `src/view_models/app_toolbar.rs:124` |
| 2 | Type the artist query. | Live search results appear after the pause. | `src/app/search_live.rs:154` |
| 3 | Select "Artists". | Artist results appear. | `src/view_models/search_results/tabs.rs:53` |
| 4 | Click the artist’s name. | Its artist/feed page opens. | `src/view_models/publisher_page.rs:1308` |
| 5 | Click "Select". | Release checkboxes appear. | `src/app/detail_selection.rs:126`, `src/view_models/library.rs:2460` |
| 6 | Select the first chosen single under "Singles". | It joins the selection. | `src/view_models/publisher_page.rs:676`, `src/view_models/list_selection.rs:60` |
| 7 | Select the second chosen single. | It joins the selection. | `src/view_models/list_selection.rs:60` |
| 8 | Select the third chosen single. | The action bar counts the selected releases. | `src/view_models/selection_action_bar.rs:257` |
| 9 | Click "Download 3 releases". | The three releases enter the download path. | `src/view_models/selection_action_bar.rs:212`, `src/app/detail_selection.rs:178` |

**INFERENCE — count: 9**, wide or narrow. The search path does not require the narrow "More" menu.

An alternative uses each single’s row menu: "More actions" → "Download Single". **INFERENCE — count: 10** from Music.

Branches:

- Explicit Enter or "Search" submission adds **1 INFERENCE** step.
- Name-only results can open the name-level page, whose release rows use the same selection path.
- Pagination, disclosures, and retries add their actual control steps.
- If fewer than three matching downloadable singles exist, this exact job cannot finish with three downloads.
- Search failures follow section 6. Download failures follow section 5.

End state: all three selected releases are in the Library after successful completion.

## 15. Baseline Job 4: take all missing singles of a known artist

Start: **Music open**.

Let `n` be the number of missing singles listed on the chosen artist page.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click "Artists". | The artist list appears. | `src/view_models/sidebar.rs:366` |
| 2 | Click the known artist. | Its feed page or name-level page opens. | `src/view_models/publisher_page.rs:1308`, `src/view_models/name_level_page.rs:390` |
| 3 | Click "Select". | Release checkboxes appear. | `src/app/detail_selection.rs:126` |
| 4… | Select each missing single under "Singles". | The selection covers those releases. | `src/view_models/publisher_page.rs:676`, `src/view_models/list_selection.rs:60` |
| Last | Click "Download {n} releases". | The missing selected releases enter the download path. | `src/view_models/selection_action_bar.rs:212` |

**INFERENCE — count: n + 4 wide, n + 5 narrow.**

The narrow path begins "More" → "Artists". Loading additional visible pages or groups adds steps.

A singles-group page offers a shorter acquisition action: **"Download {missing_count} Single{s}"**. It counts singles absent from the Library. Evidence: `src/view_models/singles_group.rs:1473`.

**INFERENCE — count from an already-open singles-group page: 1.** If all its singles are present, its action becomes "Add to Playlist". Evidence: `src/view_models/singles_group.rs:1487`.

The singles-group Index load can be limited to the artist’s 1,000 newest releases. Its notice states that limit. Evidence: `src/view_models/singles_group.rs:81`.

End state: the selected missing singles are in the Library after successful completion.

**UNSURE:** “all missing singles” beyond the loaded results or the stated 1,000-release limit cannot be established from this page.

## 16. Baseline Job 8: check feeds, apply, write tags

Start: **Music open**.

The current "Check all feeds" command includes stale-update application. A separate apply click is not normally part of that command’s completion.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click "Check all feeds". | Feed checks start. Stale updates and missing-route repairs run automatically. | `src/library/app_impl.rs:3126`, `src/application/commands/feed.rs:427` |
| — | Wait. | The command completes and the tag scan runs. | `src/library/app_impl.rs:3137` |
| 2 | Click "Update {files}". | The proposed file writes appear. | `src/view_models/tag_update.rs:145` |
| 3 | Click "Write Tags". | Eligible file tags are written. | `src/view_models/tag_update.rs:151` |

**INFERENCE — count: 3 wide, 4 narrow**, using "More" → "Check all feeds" and the status-bar tag control.

If no tag difference exists, the update control is absent. **INFERENCE — count: 1 wide, 2 narrow** for the completed check.

If pending updates already exist, the sidebar action is "Apply updates ({count})". Applying them, opening the tag popup, and confirming takes **3 INFERENCE** steps wide.

The equivalent narrow pending-update path takes **4 INFERENCE** steps.

A separate pending apply can also follow an automatic check triggered by opening a Library release. Evidence: `src/library/app_impl.rs:2713`, `src/view_models/library.rs:5822`.

Confirmations, failures, skipped files, and report locations are in section 12.

End state: stored feed metadata is current to the successful requests. Eligible tags are written. Failed, unreadable, or show-used files remain outstanding.

## 17. Baseline Job 9: fix a “Needs attention” item

Start: **Music open**.

The current destination label is "Inbox". Its track problem labels are "Missing routes", "No upstream routes", "Missing file", "Not downloaded", "Route out of date", and "Removed from feed". Evidence: `src/application/queries/broadcast.rs:122`.

### 17.1 Main paths and counts by problem type

| Problem type | User action path | What appears next or end state | Count and evidence |
|---|---|---|---|
| "Missing routes" | "Inbox" → "Fix routes" | The row shows "Repairing…"/"Repairing routes". Readiness refreshes after completion. | **INFERENCE: 2**, both layouts. `src/view_models/library.rs:133`, `src/view_models/library.rs:1788`, `src/library/app_impl.rs:3180` |
| "No upstream routes" | "Inbox" → inspect the item | The row says "The feed owner must add payment routes." No direct repair action exists. | **INFERENCE: 1** to reach the problem. Completion count **UNSURE**, because the feed owner must change the source. `src/view_models/library.rs:136`, `src/view_models/library.rs:1798` |
| "Route out of date" | "Inbox" → "Update {files}" → "Write Tags" | The stored route is written if the file is eligible. | **INFERENCE: 3**, both layouts. `src/view_models/library.rs:146`, `src/view_models/inbox.rs:196`, `src/view_models/tag_update.rs:151` |
| "Removed from feed", keep usable | "Inbox" → "Confirm" | The removed-track mark is confirmed. The row returns to its underlying route/readiness state. | **INFERENCE: 2**, both layouts. `src/view_models/library.rs:148`, `src/library/app_impl.rs:963` |
| "Removed from feed", remove | "Inbox" → "More actions" → "Remove from Library…" → "Remove" | The track leaves the Library. | **INFERENCE: 4**, both layouts. `src/view_models/library.rs:1844`, `src/library/app_impl.rs:949` |
| "Missing file" | "Inbox" → track row | The track page opens. The Inbox has no direct file-repair action. | **INFERENCE: 2** to reach the track page. Direct completion is unavailable. `src/view_models/library.rs:1801`, `src/view_models/library.rs:1844` |
| "Not downloaded" | "Inbox" → track row | The track page opens. The Inbox menu itself has no Download action. | **INFERENCE: 2** to reach the track page. The next action depends on the track page’s subscription state. `src/view_models/library.rs:1844`, `src/ui/shells/library/track_detail.rs:46` |

The "Missing routes" result can be:

- **"Repaired payment routes for {title}"**
- **"The feed owner must add payment routes for {title}"**
- **"Payment routes already ready for {title}"**
- **"Route repair failed for {title}: {reason}"**

Evidence: `src/view_models/library.rs:6059`.

"Confirm" runs directly. It does not open another confirmation dialog. Command failure goes to Library error status. Evidence: `src/library/app_impl.rs:968`.

A successful "Confirm" does not prove that the track’s routes or file are ready. The refreshed report supplies that next state.

### 17.2 Missing-file and undownloaded branches

The readiness report distinguishes a missing recorded file from a Library track without a downloaded file. Evidence: `src/application/queries/broadcast.rs:360`.

The Inbox renderer supplies no direct download or file-repair action for either state. Evidence: `src/view_models/library.rs:1798`, `src/view_models/library.rs:1844`.

A track page offers "Download Track" when its action state is not downloaded. Its downloaded state comes from `frame.local_subscription`. Evidence: `src/view_models/track_detail.rs:635`, `src/ui/shells/library/track_detail.rs:46`.

**UNSURE:** a universal one-click recovery from "Not downloaded" is not established by the readiness state alone.

For a track page that offers "Remove Track…", removal followed by reacquisition is a code path:

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click "Inbox". | The problem list appears. | `src/library/app_impl.rs:1031` |
| 2 | Click the track. | Its track page opens. | `src/library/app_impl.rs:3210` |
| 3 | Open its page menu. | "Remove Track…" appears when available. | `src/view_models/track_detail.rs:653` |
| 4 | Select "Remove Track…". | Removal confirmation opens. | `src/library/track_page.rs:42` |
| 5 | Click "Remove". | Library membership is removed. Missing source files are skipped by the cache move. | `src/view_models/library_removal.rs:68`, `src/audio_cache.rs:50` |
| 6 | Click "Download Track" if the retained page now offers it. | Reacquisition starts. | `src/view_models/track_detail.rs:635`, `src/library/track_page.rs:31` |

**INFERENCE — conditional count: 6**, both layouts.

**UNSURE:** the retained-page transition and successful reacquisition were not observed. This is a conditional recovery path, not a fixed direct Inbox fix.

"Re-download" inside the comparison panel is not a separate audio-download handler. It repeats comparison. Evidence: `src/library/app_impl.rs:3992`.

### 17.3 Setup problems

Inbox "Setup" entries carry dependency-specific text and configuration actions. Only dependencies associated with show cards enter this section. Evidence: `src/view_models/startup/capabilities.rs:150`.

**INFERENCE — count from Music:** "Inbox" plus the setup action is **2** steps to its configuration surface.

Completion counts are **UNSURE** within this scope. They depend on Settings, installed tools, service state, and the reported dependency.

## 18. MH3 Get show-ready: Library updates and problems

Start: **Music open**. This section excludes the pre-show check.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click "Check all feeds". | Feed checks, stale metadata application, and missing-route repair run. | `src/application/commands/feed.rs:427` |
| 2 | Click "Inbox". | A fresh Library readiness report appears. | `src/library/app_impl.rs:1027` |
| 3 | Use "Update {files}" → "Write Tags" if offered. | Eligible file tags update and readiness refreshes. | `src/library/app_impl.rs:1433` |
| 4 | Resolve each surviving problem. | Its row changes, disappears, or retains its failure state. | `src/library/app_impl.rs:1036`, `src/library/app_impl.rs:3180` |

**INFERENCE — count with no problems or tag changes: 2 wide, 3 narrow.**

**INFERENCE — count with one tag-write batch and no other remaining problems: 4 wide, 5 narrow.**

From an already-open Inbox, add:

- **1 INFERENCE** step per "Fix routes".
- **1 INFERENCE** step per removed-track "Confirm".
- **3 INFERENCE** steps per removed-track Library removal.
- **2 INFERENCE** steps per additional tag-write batch.
- Actual navigation and repair steps for missing files or setup issues.

No fixed completed count exists when a feed owner must add routes or a required service remains unavailable.

End state: no remaining relevant Library problems in the refreshed report, or explicit outstanding failures. This does not establish playback or broadcast readiness outside this scope.

## 19. Other list-page journeys in scope

### 19.1 Public playlists

Entry point: "Public playlists", or narrow "More" → "Public playlists". Evidence: `src/view_models/sidebar.rs:368`, `src/view_models/sidebar.rs:242`.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Open "Public playlists". | Index playlists appear as rows or tiles. | `src/view_models/list_page.rs:109` |
| 2 | Click a playlist. | Its ordered entries appear. | `src/ui/shells/library/public_playlists.rs:245` |
| 3 | Click an entry with a target. | Its track or release page opens. | `src/ui/shells/library/public_playlists.rs:259` |
| 4 | Use that item page’s download action. | Music enters the acquisition path. | `src/view_models/track_detail.rs:620`, `src/view_models/album_page.rs:56` |

**INFERENCE — count from Music to acquisition:** **4 wide, 5 narrow**, when a target exists.

An entry without a target shows text and has no click handler. Evidence: `src/ui/shells/library/public_playlists.rs:245`.

The row/page menu offers "Copy to My Playlists…". It opens "Copy to My Playlists", with "Name of the new playlist", "Copy", and "Cancel". Evidence: `src/library/public_playlists.rs:491`, `src/view_models/public_playlists.rs:30`.

The copy downloads no file. It reports **"Copied {copied} of {total} {noun} to {name}. {count} could not be read."** Evidence: `src/view_models/public_playlists.rs:255`, `src/view_models/public_playlists.rs:307`.

**INFERENCE — count from a visible public-playlist row:** menu, copy item, typed name, "Copy" = **4**. A usable prefilled name removes the typed-field step.

Name errors include "Enter a name for the playlist." and **"A playlist named \"{name}\" already exists. Enter another name."** Evidence: `src/view_models/public_playlists.rs:40`, `src/view_models/public_playlists.rs:295`.

### 19.2 Live

Entry point: "Live", or narrow "More" → "Live". Evidence: `src/view_models/sidebar.rs:369`.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Open "Live". | Current live items appear, or "Nothing is live now.". | `src/view_models/live_page.rs:24` |
| 2 | Click a live row. | Its feed release page opens. | `src/library/live_page.rs`, `src/view_models/live_page.rs:156` |
| 2a | Click "Open link". | An HTTP or HTTPS content link opens externally. | `src/view_models/live_page.rs:26`, `src/view_models/live_page.rs:198` |

**INFERENCE — count from Music:** open Live **1 wide, 2 narrow**. Open a row or external link adds **1**.

**UNSURE:** browser behavior and the live content supplied by the external host.

## 20. Controls without handlers and handlers that do nothing

| Place | Observed code behavior | Evidence |
|---|---|---|
| Release-list download-state icon | Has tooltip and accessibility text, but no download click handler | `src/ui/composites/release_download_state.rs:41` |
| Search-result download-state icon | Uses the same noninteractive release-state renderer | `src/ui/shells/search_result_rows.rs:123` |
| Public-playlist entry without a target | Text row has no click handler | `src/ui/shells/library/public_playlists.rs:245` |
| Index track row without a track GUID | Receives no key and no row click | `src/view_models/release_page.rs:989` |
| Contributor row body | Shows contributor text and optional action controls. The row body has no navigation handler. | `src/ui/shells/entity.rs:285` |
| "Saved" action | Unavailable because the query is already saved | `src/view_models/search_results/saved_search.rs:147` |
| Library track page menu-open notification | `MenuOpened` does nothing. The menu’s action handlers still operate. | `src/library/track_page.rs:56` |
| Public-playlist menu’s playlist-selection callbacks | Explicit empty callbacks. That menu has no playlist-selection level. | `src/library/public_playlists.rs:523` |
| My-playlist row menu’s playlist-selection callbacks | Empty callbacks with an empty playlist-choice collection | `src/library/app_impl.rs:2134` |
| Inbox page-title action callback | Empty callback, paired with no header action | `src/library/app_impl.rs:6028` |
| Index release commands for removal, source inspection, and MusicBrainz | Empty match branches. Those actions are not offered on that origin. | `src/app/search_dispatch.rs:1581` |
| Index track commands for track download, playlist addition, source inspection, MusicBrainz, and removal | Empty match branches. Those actions are not offered on that page. | `src/app/search_dispatch.rs:1682` |

These entries distinguish unavailable or absent controls from a visible control whose action fails.

## 21. Labels that differ for one action

| Action | Different labels or forms | Evidence |
|---|---|---|
| Find recent music | Historical/code references to Discover/New releases. Current destination "Releases", sort "Newest". | `src/view_models/sidebar.rs:365`, `src/view_models/list_page.rs:173`, `src/library/app_impl.rs:500` |
| Open Library problems | Code/comments retain “Needs attention”. Current destination is "Inbox". | `src/view_models/library.rs:1816`, `src/view_models/sidebar.rs:370` |
| Search focus | Field "Search Library and Index", button "Search", shortcut labels "Find" and "Focus Search" | `src/view_models/app_toolbar.rs:124`, `src/app/keyboard.rs:129` |
| Download release | "Download Album", "Download EP", "Download Single", "Download Release", and selection template "Download {count} releases" | `src/view_models/album_page.rs:56`, `src/view_models/track_detail.rs:937`, `src/view_models/selection_action_bar.rs:212` |
| Download track | Row menu/tooltip "Download". Track page "Download Track". | `src/view_models/release_page.rs:43`, `src/view_models/track_detail.rs:940` |
| Remove a Library track | "Remove…", "Remove Track…", "Remove from Library…", selection "Remove {count} tracks…" | `src/view_models/release_page.rs:47`, `src/view_models/track_detail.rs:945`, `src/view_models/release_page.rs:53`, `src/view_models/selection_action_bar.rs:238` |
| Remove release | Kind-specific "Remove {Kind}…". Confirmation still says "Remove Album from Library?". | `src/view_models/album_page.rs:60`, `src/view_models/library_removal.rs:72` |
| Add to playlist | "Add to Playlist", "Add to playlist", and counted selection labels | `src/view_models/album_page.rs:57`, `src/view_models/library.rs:3155`, `src/view_models/selection_action_bar.rs:225` |
| Copy feed address | "Copy Feed URL" on item pages. "Copy feed URL" in the RSS report. | `src/view_models/album_page.rs:58`, `src/view_models/playlist_rss_check.rs:475` |
| Apply pending updates | "Apply updates ({count})" in the sidebar/menu. "Apply updates" in Inbox. | `src/view_models/library.rs:5408`, `src/view_models/inbox.rs:182` |
| Save a search | "Save search" becomes unavailable "Saved". | `src/view_models/search_results/saved_search.rs:147` |
| Delete saved search | Row menu "Delete Saved Search…", page action "Delete…", confirmation "Delete" | `src/view_models/search_results/saved_search.rs:548`, `src/view_models/search_results/saved_search.rs:169`, `src/view_models/search_results/saved_search.rs:201` |
| Repeat file comparison | "Re-read" and "Re-download" call the same reload operation | `src/view_models/entity_detail.rs:299`, `src/library/app_impl.rs:3992` |
| Library membership | "In library", "In Library", and "Downloaded" appear on different surfaces | `src/view_models/library.rs:132`, `src/view_models/library.rs:6643`, `src/view_models/release_download_state.rs:60` |

## 22. Results and errors shown away from their initiating control

| Initiating control or event | Result/error location and text | Evidence |
|---|---|---|
| Opening a Library release | Automatic feed-check status appears in the sidebar feed area, outside the release page | `src/library/app_impl.rs:2713`, `src/view_models/library.rs:5811`, `src/library/app_impl.rs:5950` |
| Inbox "Apply updates" | Updates the shared feed-update status. Wide rendering places that result in the sidebar. | `src/library/app_impl.rs:3151`, `src/view_models/library.rs:5894`, `src/library/app_impl.rs:5950` |
| Narrow "More" feed command | Result text becomes menu-item description from shared feed status | `src/library/app_impl.rs:5765`, `src/library/app_impl.rs:5794` |
| Release or track row download | Job result appears in the shared download progress item | `src/library/app_impl.rs:3721`, `src/library/downloads.rs:234` |
| Failed Library track download | Also updates the mounted track page’s subscription message, if that track is open | `src/library/app_impl.rs:3761` |
| "Write Tags" | Failed-write words enter the app event list and status-bar event display | `src/library/app_impl.rs:1439`, `src/view_models/status_bar.rs:108` |
| Inbox "Fix routes" | Completion text enters shared Library status. The Inbox report refreshes independently. | `src/view_models/library.rs:5916`, `src/library/app_impl.rs:3197`, `src/library/app_impl.rs:5910` |
| Release "MusicBrainz Lookup" | Album progress and fallback messages enter shared Library status | `src/view_models/library.rs:5738`, `src/library/app_impl.rs:4326` |
| Search result navigation failure | Some paths assign `settings_status`, including **"Failed to navigate to release: {error}"** and **"Failed to navigate to track: {error}"** | `src/app/search_dispatch.rs:873`, `src/app/search_dispatch.rs:1012` |
| Search result identity failure | Some paths assign `settings_status`, including **"Track {track_id} not found"** | `src/app/search_dispatch.rs:828` |
| Playlist RSS check | The report belongs to the playlist page. The status-bar feed item can navigate back to that playlist. | `src/view_models/playlist_rss_check.rs:218`, `src/app/status_bar.rs:101` |
| Search request failure | The result list or its notice shows the failure, away from the toolbar field | `src/ui/shells/search_results_inspector.rs:703` |
| Index track feed-URL copy | The page-menu trigger shows "Copying…", "No feed URL", or "Copy failed" | `src/view_models/track_page.rs:192` |
| External image load failure | The inspected thumbnail request has an empty failure callback. No user-facing error is produced there. | `src/app/search_dispatch.rs:1818` |

**UNSURE:** immediate visibility of messages stored in `settings_status` while Music remains open. The assignment paths establish the storage target, not a visible Music notice.