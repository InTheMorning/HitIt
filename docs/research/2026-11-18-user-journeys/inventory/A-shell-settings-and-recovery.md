<!-- Recorded output of ChatGPT inventory run A at commit 79c23c1. Kept unchanged as evidence. It is not edited to STE. See ../README.md. -->

Recorded output of ChatGPT inventory run A at commit 79c23c1a95a817a3d2ff2f4aba1cd9b6dd423391.

## 1. Reading this inventory

This inventory covers the shell, startup, Settings, setup tools, and recovery. It describes code paths, without running the app.

- Quoted text preserves code labels, capitalization, and punctuation.
- Icon controls can have a quoted accessibility label or tooltip instead of visible text.
- Templates identify text that depends on configuration, records, errors, or service responses.
- **INFERENCE** marks calculated step counts.
- **UNSURE** marks outcomes that source code alone cannot establish.
- Each click, key command, typed field, or list selection counts as one step.
- Opening a menu and selecting its item count as two steps.
- A native file selection would count as one step. The maintenance forms examined here use typed paths.
- Waits and automatic checks count as zero steps.
- Counts assume successful operations unless a branch states otherwise.
- Counts exclude scrolling and focus changes unless the named path requires them.

The narrow shell starts below 940 px. At exactly 940 px, the sidebar remains. Evidence: `src/ui/layouts.rs:73`, `src/app.rs:2225`.

Settings remembers its selected group and diagnostic page during the session. Counts state their starting page where this affects the path. Evidence: `src/view_models/settings.rs:301`.

## 2. Start the app and reach the normal shell

**Entry point:** Launch the executable without arguments. The launcher’s visible label is **UNSURE**. Evidence: `src/main.rs:15`.

There is no welcome questionnaire or first-run wizard in this startup path. Startup checks lead to the normal shell or recovery.

### Main path

| Step | User action | What appears next | Evidence (file:line) |
|---|---|---|---|
| 1 | Launch HitIt. | One window opens with startup checks. The working title is "Checking startup requirements". | `src/main.rs:16`, `src/app/bootstrap.rs:71`, `src/view_models/startup.rs:190` |
| 2 | Wait. | The app reads or creates configuration, checks music storage, and prepares SQLite. | `src/app/startup.rs:169`, `src/startup.rs:198`, `src/startup.rs:221` |
| 3 | Wait for successful preparation. | The normal shell replaces startup content in the existing window. | `src/app/bootstrap.rs:242`, `src/app/bootstrap.rs:289` |

**Count — INFERENCE:** 1 step from launch to the normal shell when initial preparation succeeds.

### First-run branches

| Condition | Code behavior and text | Evidence |
|---|---|---|
| Configuration does not exist. | Initial loading can create defaults. Startup retains the new document’s identity for first-run preparation. | `src/startup.rs:198`, `src/startup.rs:213` |
| The new default music directory does not exist. | Initial preparation creates it. A check without preparation instead reports "The new default music directory needs initial setup." | `src/startup/storage.rs:19` |
| An existing configuration names a missing music directory. | Startup reports "The configured location does not exist." It does not apply first-run creation to that existing configuration. | `src/startup.rs:70`, `src/startup/storage.rs:33` |
| The music path names a file. | "The music path points to a file, not a directory." | `src/startup/storage.rs:35` |
| Storage access fails. | Examples include "The operating system denied access." and "This path has the wrong file type." | `src/startup.rs:70` |
| Probe cleanup fails. | The report identifies the remaining probe and says "Remove this remaining probe file after correcting permissions, then choose Check again." | `src/startup/storage.rs:92` |
| A core check fails. | Recovery remains mounted with "App needs attention before it can open". | `src/view_models/startup.rs:190` |
| The previous application data cannot move. | The process prints template "Error: {error:#}" and exits before reading configuration. | `src/main.rs:9` |
| Font loading fails. | The app continues with system fonts. The terminal receives template "Figtree fonts are not available. The app continues with the system font. {error:#}". | `src/app/bootstrap.rs:60` |
| The window cannot open. | The terminal report includes "The window system rejected the app's initial window. App did not open another error window." | `src/app/bootstrap.rs:113` |

### Recovery actions

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Activate "Check again". | "Checking…" appears. Existing resources are checked without setup or migration. | `src/view_models/startup.rs:264`, `src/app/startup.rs:183` |
| 2 | Wait. | A report remains. Successful checks show "Ready to open the app". | `src/view_models/startup.rs:193` |
| 3 | Activate "Open app". | "Opening…" appears. The app revalidates configuration and prepares resources. | `src/view_models/startup.rs:273`, `src/app/startup.rs:184` |
| 4 | Wait. | The normal shell opens, or recovery remains with the new report. | `src/view_models/startup.rs:249`, `src/app/bootstrap.rs:242` |

**Count — INFERENCE:** 2 steps from a corrected recovery state through "Check again" and "Open app".

Other entries:

- "Copy startup report" copies the complete startup report. **INFERENCE:** 1 step. Evidence: `src/view_models/startup.rs:261`, `src/app/startup.rs:176`.
- "Quit" exits recovery. **INFERENCE:** 1 step. Evidence: `src/view_models/startup.rs:280`, `src/app/startup.rs:179`.
- The recovery page selector offers "Startup", "Configuration", and "Database". **INFERENCE:** 2 steps to another page. Evidence: `src/view_models/maintenance.rs:99`.

The page selector visibly shows its selected page. "Choose recovery page" is its accessibility label. Evidence: `src/ui/composites/maintenance_page.rs:69`.

**End state:** A normal app session, a retained recovery report, or process exit.

**Narrow layout:** Recovery uses its own maintenance layout. Its controls change from columns to a stacked arrangement below 760 px. Evidence: `src/view_models/maintenance.rs:58`.

## 3. Navigate the shell, toolbar, status bar, menus, and layers

### 3.1 Destination navigation

The sidebar entries are:

"Search", "Releases", "Artists", "My playlists", "Public playlists", "Live", "Show", "Inbox", and "Settings".

Evidence: `src/view_models/sidebar.rs:362`.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click a sidebar destination. | Its Music destination, Show surface, or Settings surface opens. The selected sidebar row changes. | `src/view_models/sidebar.rs:47`, `src/app/tab_bar.rs:26`, `src/app.rs:2208` |
| 1, narrow | Click "Search", "Releases", "Show", or "Inbox". | The same destination opens through the bottom tab bar. | `src/view_models/sidebar.rs:234`, `src/ui/composites/tab_bar.rs:75` |
| 1, narrow | Click "More". | A menu opens above the bottom bar. | `src/ui/composites/tab_bar.rs:109` |
| 2, narrow | Select "Artists", "My playlists", "Public playlists", "Live", or "Settings". | The selected destination opens. | `src/view_models/sidebar.rs:242`, `src/ui/composites/tab_bar.rs:86` |

**Counts — INFERENCE:**

- Wide sidebar destination: 1 step.
- Narrow direct tab: 1 step.
- Narrow destination through "More": 2 steps.

The narrow tab label includes a count when present, using template `"{} {count}"`. Evidence: `src/view_models/sidebar.rs:269`.

"More" also contains the feed-update command. It uses the same command owner as the sidebar feed area. Evidence: `src/app/tab_bar.rs:20`.

Its available label includes "Check all feeds", or a label made from the pending update state. Evidence: `src/view_models/library.rs:5400`, `src/library/app_impl.rs:5765`.

### 3.2 Toolbar navigation and search entry

| Entry point | Main path and next content | Count — INFERENCE | Evidence |
|---|---|---:|---|
| "Back" icon tooltip/accessibility label | Click it to move through the Music navigation history. | 1 | `src/view_models/app_toolbar.rs:107`, `src/app/toolbar.rs:131` |
| "Forward" icon tooltip/accessibility label | Click it to move forward through that history. | 1 | `src/view_models/app_toolbar.rs:108`, `src/app/toolbar.rs:143` |
| "Search Library and Index" field placeholder | Type a query. Submit with "Search" or the search icon. Results occupy the Music search surface. | 2 | `src/view_models/app_toolbar.rs:124`, `src/app/toolbar.rs:265` |
| "Search" button | Submit the current field value. | 1 | `src/view_models/app_toolbar.rs:126`, `src/app/toolbar.rs:265` |

The toolbar search button becomes an icon below 1120 px. Its accessibility label remains "Search Library and Index". Evidence: `src/ui/layouts.rs:19`, `src/app/toolbar.rs:253`.

The search field occupies half the content width, with a maximum width. Below 940 px, this calculation excludes no sidebar. Evidence: `src/app/toolbar.rs:108`.

The application mark has accessibility label "Application mark". It has a tooltip but no navigation handler. Evidence: `src/view_models/app_toolbar.rs:120`, `src/app/toolbar.rs:27`.

### 3.3 Status items

| Entry point | What appears next | Count — INFERENCE | Evidence |
|---|---|---:|---|
| Report item, template "{count} report" or "{count} reports" | Outside Settings, a capability notice opens below the toolbar. A second click closes it. | 1 | `src/view_models/status_bar.rs:208`, `src/app/status_bar.rs:115`, `src/app.rs:2239` |
| Same report item in Settings | Settings opens "Background tools" in its report view. | 1 | `src/app/status_bar.rs:116`, `src/app/capabilities.rs:101` |
| "Checking feeds" or template "{count} feed(s) failed" | The relevant playlist opens. | 1 | `src/view_models/status_bar.rs:164`, `src/app/status_bar.rs:101` |
| File-update status item | The tag-update popup opens. Its text comes from the tag-update projection. | 1 | `src/view_models/status_bar.rs:194`, `src/app/status_bar.rs:111` |
| Show status, including "Show active" or the current playback state | Show opens. | 1 | `src/view_models/status_bar.rs:229`, `src/app/status_bar.rs:123` |

Capability notices do not open automatically with every issue. The status report item controls their disclosure. Evidence: `src/app.rs:2212`.

### 3.4 Application menu and keyboard entries

Linux uses Ctrl where the binding source uses `cmd`. macOS uses Command. Evidence: `src/app/keyboard.rs:53`.

| Keys on Linux | Code label | Result | Evidence |
|---|---|---|---|
| Ctrl+Alt+P | "Show Play/Pause" | Start, pause, or resume the Show cue, according to state. | `src/app/keyboard.rs:110`, `src/app/playback_bar.rs:75` |
| Ctrl+Alt+Right | "Next Show cue entry" | Move to the next Show cue entry when available. | `src/app/keyboard.rs:116`, `src/app/playback_bar.rs:51` |
| Ctrl+Alt+Left | "Previous Show cue entry" | Move to the previous Show cue entry when available. | `src/app/keyboard.rs:122`, `src/app/playback_bar.rs:63` |
| Ctrl+Alt+F | "Focus Search" | Focus global search. | `src/app/keyboard.rs:128`, `src/app/keyboard.rs:395` |
| Ctrl+F | "Find" | Focus the same field. | `src/app/keyboard.rs:134` |
| Ctrl+N | "New Playlist" | Switch to Music and begin playlist creation. | `src/app/keyboard.rs:140`, `src/app/keyboard.rs:404` |
| Ctrl+1 | "Music" | Select the Music mount. | `src/app/keyboard.rs:146`, `src/app/keyboard.rs:415` |
| Ctrl+2 | "Show" | Select Show. | `src/app/keyboard.rs:152`, `src/app/keyboard.rs:433` |
| Ctrl+3 | "Settings" | Select Settings. | `src/app/keyboard.rs:158`, `src/app/keyboard.rs:424` |
| Ctrl+R | "Refresh Library" | Refresh while Music is selected. | `src/app/keyboard.rs:164`, `src/app/keyboard.rs:442` |
| Ctrl+, | "Preferences..." | Select Settings. | `src/app/menu.rs:81`, `src/app/menu.rs:199` |
| Alt+Left / Alt+Right | "Back" / "Forward" | Use the toolbar history routes. | `src/app/menu.rs:100`, `src/app/menu.rs:212` |
| Ctrl+Q | "Quit Application" | Quit the app. | `src/app/menu.rs:95`, `src/app/menu.rs:247` |

**Count — INFERENCE:** Each shortcut is one step.

The active-pane bindings also include:

| Keys | Exact code label | Evidence |
|---|---|---|
| Escape | "Back" | `src/app/keyboard.rs:170` |
| Up / Down | "Move Up" / "Move Down" | `src/app/keyboard.rs:176` |
| Enter | "Open" | `src/app/keyboard.rs:188` |
| Page Up / Page Down | "Move Up a Page" / "Move Down a Page" | `src/app/keyboard.rs:194` |
| Home / End | "Move to First Row" / "Move to Last Row" | `src/app/keyboard.rs:206` |
| Space | "Select Focused Row" | `src/app/keyboard.rs:218` |
| Ctrl+A | "Select All Rows" | `src/app/keyboard.rs:224` |
| Right / Left | "Open Group" / "Close Group" | `src/app/keyboard.rs:230` |
| Shift+F10 or Menu | "Open Row Menu" | `src/app/keyboard.rs:242` |
| Alt+Up / Alt+Down | "Move Row Up" / "Move Row Down" | `src/app/keyboard.rs:255` |

Input and button contexts exclude some active-pane bindings. These names describe commands, not persistent screen labels. Evidence: `src/app/keyboard.rs:12`.

The native application menu is installed only on macOS. Its exact entries are "Preferences...", "Services", "Hide Application", "Hide Others", "Show All", and "Quit Application". Evidence: `src/app/menu.rs:125`, `src/app/menu.rs:154`.

The native "Services" submenu contents are **UNSURE**. They come from the operating system.

### 3.5 Window layers

The shell renders sheet, dialog, and notification layers in that order. Evidence: `src/ui/shells/window_layers.rs:13`.

The initial window close handler requests app exit. Outstanding maintenance can finish before process exit. Evidence: `src/app/bootstrap.rs:78`, `src/view_models/startup.rs:202`.

**End state:** The selected destination, changed navigation history, disclosed report, or app exit.

## 4. Edit General settings and restore defaults

**Entry points:**

- "Settings" sidebar or "More" item: `src/view_models/sidebar.rs:372`.
- Ctrl+3, code label "Settings": `src/app/keyboard.rs:158`.
- Ctrl+,, code label "Preferences...": `src/app/menu.rs:81`.
- "General" group: `src/view_models/settings.rs:42`.

### Main path

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Open "Settings". | The retained Settings group opens. A new Settings model starts on "General". | `src/view_models/settings.rs:12`, `src/app/menu.rs:206` |
| 2, if needed | Select "General". | "Appearance" and "Defaults" sections appear. | `src/view_models/settings.rs:55` |
| 3 | Select a "UI scale" choice: "XS", "S", "M", "L", or "XL". | The interface scale changes immediately. | `src/view_models/settings.rs:501`, `src/app/settings.rs:75` |
| 4, alternative | Select a "Theme" choice. | The theme changes immediately. Choices are "System", "Dark", "Light", "High Contrast Dark", and "High Contrast Light". | `src/theme_profile.rs:49`, `src/app/settings.rs:76` |
| 5 | Click "Save". | General and Library values save together. Feedback appears below the form. | `src/view_models/settings.rs:487`, `src/app/settings.rs:224` |

**Counts — INFERENCE:**

- From anywhere, with General already selected: open Settings + one choice + Save = 3 steps.
- Add 1 step when selecting "General" is necessary.
- Change both scale and theme: add 1 step.

Success uses template `"Saved settings. Music files download under {}/artists"`. Evidence: `src/app.rs:1486`.

Failure uses template `"Error: {error:#}"`. A partial preparation failure uses `"App applied the saved settings but could not prepare {}/artists for downloads: {error}"`. Evidence: `src/app.rs:1491`.

### Use Defaults branch

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click "Use Defaults…". | Dialog title "Use the default settings?" appears. | `src/view_models/settings.rs:478`, `src/view_models/settings.rs:250` |
| 2 | Read the confirmation. | "This resets the General and Library settings to the defaults and saves them. You cannot undo this." | `src/view_models/settings.rs:251` |
| 3 | Click "Use Defaults". | The endpoint resets, the FLAC override clears, scale becomes Medium, theme becomes its default, and settings save. | `src/view_models/settings.rs:256`, `src/app/settings.rs:81` |

"Cancel" closes the dialog without resetting settings. It receives initial focus. Evidence: `src/view_models/settings.rs:253`, `src/ui/composites/confirmation_dialog.rs:97`.

Escape and unhandled Enter take the safe cancellation path. Evidence: `src/ui/composites/confirmation_dialog.rs:115`.

**Count — INFERENCE:** 2 steps from General to confirmed reset. Cancellation also takes 2 steps.

**End state:** Current appearance changes immediately. Successful Save persists it. The reset handler does not change music or database paths.

**Narrow layout:** Controls wrap below their row names when space is insufficient. The text field layout is described in Section 5. Evidence: `src/ui/composites/settings_row.rs:59`.

## 5. Edit Library settings and delete cached files

**Entry points:** "Settings", then "Library". Evidence: `src/view_models/settings.rs:43`.

The sections are "Music Index", "Files", and "Audio cache". Evidence: `src/view_models/settings.rs:65`.

### Main paths

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Select "Library". | Rows include "Music Index endpoint", "Music directory", "Audio converter", and "Cached files". | `src/view_models/settings.rs:65`, `src/view_models/settings.rs:176` |
| 2 | Type the "Music Index endpoint". | The field holds the edited value. Help says "Use api.musicindex.org or a full http/https URL." | `src/view_models/settings.rs:195`, `src/app/settings.rs:251` |
| 3 | Click "Save". | The endpoint is normalized and applied to the app and Library. | `src/app.rs:1433`, `src/app.rs:1455` |
| 2, alternative | Click "Configuration repair" beside "Music directory". | Diagnostics opens its configuration page. The directory is changed through the repair editor. | `src/view_models/settings.rs:439`, `src/app/settings.rs:252` |
| 2, alternative | Click "Converter setup" beside "Audio converter". | Configuration repair opens with the converter field selected. | `src/view_models/settings.rs:450`, `src/app/settings.rs:56` |

**Counts — INFERENCE:**

- From anywhere: Settings + Library + endpoint field + Save = 4 steps.
- From Library: Configuration repair = 1 step to that page.
- From Library: Converter setup = 1 step to the selected converter editor.

The directory help explicitly describes ending the session before testing and saving a different existing folder. Evidence: `src/view_models/settings.rs:196`.

An unset converter path displays "Not set". Evidence: `src/app/settings.rs:258`.

### Cached-file path

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Open Library settings. | The cache section loads. Its title uses template "Cached files ({count})". | `src/app/settings.rs:185`, `src/view_models/cached_files.rs:128` |
| 2 | Click a track’s "Delete". | The app submits permanent deletion for that cached file. No confirmation dialog is called by this handler. | `src/view_models/cached_files.rs:78`, `src/app/settings.rs:349`, `src/app.rs:1529` |
| 2, alternative | Click "Delete All Cached". | The app submits deletion for the loaded cached paths. No confirmation dialog is called by this handler. | `src/view_models/cached_files.rs:101`, `src/app.rs:1533` |
| 3 | Wait. | The cache list reloads. Completion or error text appears below Settings content. | `src/app.rs:1561` |

**Count — INFERENCE:** 1 step from a loaded cache list to either deletion action. From anywhere through Settings and Library: 3 steps.

Cache-list texts include:

- "App is reading the cached-file list."
- "No cached files."
- "App could not read the cached-file list because background tools are unavailable. Choose Check again in Diagnostics, Background tools."
- "App could not read the cached-file list from the database. Reopen Settings to try again."
- A retained list adds " App is showing the last loaded list."

Evidence: `src/view_models/cached_files.rs:135`.

Deletion failure uses `"Error: {error:#}"`. Evidence: `src/app.rs:1565`.

**End state:** A saved endpoint, an opened repair editor, or a refreshed cache list.

**Narrow layout:** The endpoint field always sits below its row name. Other controls wrap as needed. Evidence: `src/ui/composites/settings_row.rs:48`.

## 6. Open Diagnostics, read reports, and copy information

**Entry points:**

- "Diagnostics": `src/view_models/settings.rs:44`.
- The status report item: `src/app/status_bar.rs:115`.
- "View tools in Settings": `src/view_models/startup/capabilities.rs:273`.
- Dependency-specific edit and review controls, listed in Section 10.

### Main path

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Open Settings and select "Diagnostics". | The version line appears first. The retained diagnostic page opens. Its initial page is "Database". | `src/app/settings.rs:174`, `src/view_models/settings.rs:95` |
| 2 | Click the diagnostic page selector. | Choices appear: "Database", "Configuration", "Background tools", and "App session". | `src/view_models/settings.rs:111`, `src/app/settings.rs:157` |
| 3 | Select a page. | That page replaces the prior diagnostic content. | `src/view_models/settings.rs:392` |
| 4 | Click "Show report". | The report replaces the instructions. | `src/view_models/maintenance.rs:15` |
| 5, alternative | Click "Show instructions". | Instructions replace the report. | `src/view_models/maintenance.rs:18` |

The selector visibly shows the selected page label. Its accessibility label is "Choose diagnostics page". Evidence: `src/view_models/settings.rs:310`, `src/ui/composites/maintenance_page.rs:69`.

**Counts — INFERENCE:**

- Anywhere to initial Diagnostics: 2 steps using a Settings shortcut and "Diagnostics".
- From Diagnostics to another diagnostic page: 2 steps.
- Instructions/report switch: 1 step.

### Copy paths

| Control | Copied object | Count — INFERENCE | Evidence |
|---|---|---:|---|
| "Copy" beside the version | The generated application version line | 1 | `src/view_models/settings.rs:324`, `src/app/settings.rs:64` |
| "Copy startup report" | Complete startup report | 1 | `src/app/startup.rs:176` |
| "Copy repair report" | Configuration repair report | 1 | `src/view_models/startup/correction.rs:214`, `src/presentation/configuration_editor.rs:127` |
| "Copy draft (redacted)" | Redacted configuration draft | 1 | `src/view_models/startup/correction.rs:213`, `src/presentation/configuration_editor.rs:130` |
| "Copy database report" | Database maintenance report | 1 | `src/view_models/startup/database.rs:240`, `src/presentation/database_tools.rs:112` |
| "Copy report" in Background tools | Complete background-tools report | 1 | `src/view_models/startup/capabilities.rs:334`, `src/app/capabilities.rs:179` |
| "Copy report" in session drain | Complete session-maintenance report | 1 | `src/view_models/startup/session.rs:94` |
| "Copy report" on a search failure | Complete search-failure report, including undisclosed details | 1 | `src/view_models/search_results/failure.rs:116`, `src/view_models/search_results/failure.rs:146` |

These copy handlers write clipboard content. A clipboard success notification is not shown in the examined handlers.

Reports use recorded UTC timestamps and generated resource details. Search reports redact endpoint details. Evidence: `src/view_models/startup.rs:318`, `src/view_models/search_results/failure.rs:94`.

Log frames provide "Go to latest". It resumes following the latest entries. Status text can be "Following latest entries", "Following paused", or "Earlier text is no longer available. Showing the oldest available entry." Evidence: `src/view_models/log_view.rs:69`.

**Count — INFERENCE:** 1 step from a paused log to "Go to latest".

**End state:** A diagnostic page, a disclosed report, or copied information.

**Narrow layout:** Maintenance actions stack below the layout threshold of 760 px. The labels and commands remain the same. Evidence: `src/view_models/maintenance.rs:58`.

## 7. Repair configuration, playback settings, and converter settings

There are no separate Settings groups named Playback, Broadcast, or Show setup. The current groups are General, Library, and Diagnostics. Evidence: `src/view_models/settings.rs:20`.

Playback and broadcast values are edited through "Configuration repair".

### Entry points

| Entry | Destination | Evidence |
|---|---|---|
| "Configuration repair" | Diagnostics, Configuration | `src/view_models/settings.rs:443` |
| "Configuration" diagnostic menu item | Same page | `src/view_models/settings.rs:114` |
| "Configuration" recovery menu item | Recovery configuration editor | `src/view_models/maintenance.rs:105` |
| "Edit endpoint" | Editor focused on endpoint configuration | `src/view_models/startup/capabilities.rs:645`, `src/app/capabilities.rs:164` |
| "Edit player settings" | Editor focused on player configuration | `src/view_models/startup/capabilities.rs:646` |
| "Edit relay sender settings" | Editor focused on host configuration | `src/view_models/startup/capabilities.rs:647` |
| "Edit publication settings" | Editor focused on publication configuration | `src/view_models/startup/capabilities.rs:648` |
| "Edit encoder settings" | Editor focused on encoder configuration | `src/view_models/startup/capabilities.rs:649` |
| "Edit converter setting" or "Converter setup" | Editor focused on `flac_path` | `src/view_models/startup/capabilities.rs:650`, `src/view_models/settings.rs:450` |
| "Edit display settings" | Editor focused on presentation configuration | `src/view_models/startup/capabilities.rs:651` |

### Main path

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click "Edit configuration". | The app loads the current file and its revision. Field buttons and the draft editor appear. | `src/view_models/startup/correction.rs:201`, `src/presentation/configuration_editor.rs:151` |
| 2 | Select a field button. | The editor shows that field’s current value. | `src/presentation/configuration_editor.rs:122`, `src/view_models/startup/correction.rs:319` |
| 3 | Type the replacement value. | The draft retains the edit without saving. | `src/view_models/startup/correction.rs:338` |
| 4, optional | Click "Test draft and paths". | The report opens with validation results. No configuration is saved. | `src/view_models/startup/correction.rs:209`, `src/presentation/configuration_editor.rs:160` |
| 5 | Click "Save correction". | The app validates again, preserves the original, replaces configuration, and opens the report. | `src/view_models/startup/correction.rs:211`, `src/view_models/startup/correction.rs:452` |
| 6 | Wait. | In a normal session, changed optional tools receive automatic checks. The original failed operation is not repeated. | `src/app/startup.rs:91`, `src/app/capabilities.rs:332` |

**Counts — INFERENCE:**

- From a closed configuration editor: Edit configuration + field selection + typed field + Save correction = 4 steps.
- From a dependency-specific edit button, with automatic field loading: edit button + typed field + Save correction = 3 steps.
- Explicit validation adds 1 step. Returning to the editor with "Show instructions" adds another step if further editing is required.

### Field controls

Except for `flac_path`, field buttons use the field name as their label. The `flac_path` button says "Converter setup". Evidence: `src/view_models/startup/correction.rs:205`.

The supported field list includes:

`"music_dir"`, `"db_path"`, `"musicindex_endpoint"`, `"ui_scale"`, `"theme_profile"`, `"playback.driver"`, `"playback.mpv_path"`, `"broadcast.hosts"`, `"broadcast.selected_host"`, `"broadcast.drop_directory"`, `"broadcast.drop_file_target"`, `"broadcast.encoder"`, `"workspace_layout"`, `"workspace.layout.content_pane_width"`, and `"workspace.layout.content_list_view_mode"`.

Containing-table controls also include `"playback"`, `"broadcast"`, `"workspace"`, and `"workspace.layout"`.

Evidence: `src/config/correction.rs:21`.

Text fields accept unquoted text. Structured fields require a TOML assignment named `value`. Blank structured input removes an optional field. Evidence: `src/view_models/startup/correction.rs:384`, `src/config/correction.rs:215`.

### Branches and errors

- "Close editor" hides the editor and retains the draft. "Reopen editor" restores it. Evidence: `src/presentation/configuration_editor.rs:114`.
- Escape selects "Close editor". Enter or Space then closes it. **INFERENCE:** 2 keys from the editor. Evidence: `src/view_models/startup/correction.rs:126`.
- "Reload file (discard draft)" replaces the draft with the current file. Evidence: `src/view_models/startup/correction.rs:202`.
- Core-path changes require "End app session" before validation or save becomes available. Evidence: `src/view_models/startup/correction.rs:192`, `src/view_models/startup/correction.rs:239`.
- Malformed TOML can expose document text instead of individual fields. Evidence: `src/view_models/startup/correction.rs:384`.
- After saving, another edit requires reloading. Evidence: `src/view_models/startup/correction.rs:224`.

A concurrent file change produces:

> "Configuration changed, disappeared, or its link changed after editing began. App kept your draft and did not replace the file. Copy the draft, then reload the editor; no automatic merge is available."

Evidence: `src/config/correction.rs:183`.

Invalid structured input produces:

> "App could not parse the proposed field. Enter a TOML assignment named value, or leave the field blank to remove it."

Evidence: `src/config/correction.rs:215`.

Repair operation failures use template `"App could not complete configuration {operation:?}. {message}"`. Evidence: `src/view_models/startup/correction.rs:460`.

### Converter branch

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Open "Converter setup". | The editor describes the configured FLAC path or PATH lookup. | `src/view_models/startup/converter.rs:9`, `src/view_models/startup/correction.rs:299` |
| 2, optional | Type a FLAC executable path, or clear it. | The draft changes. | `src/view_models/startup/converter.rs:10` |
| 3 | Click "Test converters". | The app checks the draft FLAC executable and PATH ffmpeg. The report opens. | `src/view_models/startup/correction.rs:210`, `src/presentation/configuration_editor.rs:160` |
| 4, optional | Click "Save correction". | The configuration saves with an original-file backup. | `src/view_models/startup/converter.rs:10` |

**Count — INFERENCE:** 1 step from the selected converter editor to testing. Editing and saving add 2 steps.

Reported outcomes include:

- "version check succeeded (exit 0)"
- Template "version check failed (exit {code})"
- "version check terminated by a signal"
- "executable was not found"
- "permission to execute was denied"
- "version check exceeded five seconds; app terminated and reaped the process"
- "version check reached the 16 KiB output limit; app terminated and reaped the process"

Evidence: `src/view_models/startup/converter.rs:33`.

Testing does not convert a track or install software. If neither converter passes, the report says WAV downloads may retain WAV files. Evidence: `src/view_models/startup/converter.rs:23`.

**End state:** A retained draft, a validation report, or saved configuration with automatic optional-tool checks.

**Narrow layout:** The same editor and commands use the maintenance layout described in Section 6.

## 8. Check, back up, preserve, restore, or repair a database

**Entry points:**

- Settings → "Diagnostics" → "Database": `src/view_models/settings.rs:113`.
- Recovery selector → "Database": `src/view_models/maintenance.rs:106`.

The task selector offers "Check", "Backup", "Preserve files", "Restore", and "Repair upgrade". Evidence: `src/view_models/startup/database.rs:80`.

Its visible trigger shows the selected task. Its accessibility label is "Choose database task". Evidence: `src/view_models/startup/database.rs:174`, `src/ui/composites/maintenance_page.rs:69`.

### Main paths

Each row below is a separate journey from the named database task.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| Check 1 | Click "Use configured database", or type "Existing database path". | The selected source path is available. This does not select another Library. | `src/view_models/startup/database.rs:176`, `src/view_models/startup/database.rs:212`, `src/view_models/startup/database.rs:473` |
| Check 2 | Click "Check database". | The report opens with database inspection results. | `src/view_models/startup/database.rs:216`, `src/view_models/startup/database.rs:356` |
| Backup 1 | Select or type the source. | The source path is retained. | `src/view_models/startup/database.rs:104` |
| Backup 2 | Type "New backup file". | A new destination filename is retained. | `src/view_models/startup/database.rs:140` |
| Backup 3 | Click "Back up database". | The app creates and verifies a database snapshot. The report opens. | `src/view_models/startup/database.rs:220`, `src/view_models/startup/database.rs:489` |
| Preserve 1 | Select or type the source. | The source path is retained. | `src/view_models/startup/database.rs:105` |
| Preserve 2 | Type "New preservation directory". | The destination directory is retained. | `src/view_models/startup/database.rs:142` |
| Preserve 3 | Click "End app session", when a session is active. | The session drains into recovery. | `src/view_models/startup/database.rs:224`, `src/app/startup.rs:123` |
| Preserve 4 | Click "Preserve database files". | The app copies database and journal files under exclusive access. | `src/view_models/startup/database.rs:228`, `src/presentation/database_tools.rs:95` |
| Restore 1 | Type "Chosen restore backup path". | The backup path is retained. | `src/view_models/startup/database.rs:177` |
| Restore 2 | Type "New preservation directory". | The preservation destination is retained. | `src/view_models/startup/database.rs:142` |
| Restore 3 | Click "Review restore". | The app validates a separate candidate and opens the report. | `src/view_models/startup/database.rs:233`, `src/view_models/startup/database.rs:442` |
| Restore 4 | Click "Restore database". | The active session ends if necessary. Preservation and installation begin. | `src/view_models/startup/database.rs:234`, `src/app/startup.rs:130` |
| Repair 1 | Click "Use configured database". | The configured source path loads. | `src/view_models/startup/database.rs:107` |
| Repair 2 | Click "Check database". | The report identifies whether the supported interrupted upgrade exists. | `src/view_models/startup/database.rs:482` |
| Repair 3 | Return with "Show instructions" and type a preservation directory. | The repair destination is retained. | `src/view_models/maintenance.rs:18`, `src/view_models/startup/database.rs:117` |
| Repair 4 | Click "End app session", when needed. | Maintenance access becomes available after drain. | `src/view_models/startup/database.rs:256` |
| Repair 5 | Click "Repair interrupted upgrade", when offered. | The app preserves the original and repairs migration 11 through a validated candidate. | `src/view_models/startup/database.rs:232`, `src/view_models/startup/database.rs:593` |

**Counts — INFERENCE:**

- From Check instructions: source choice + Check database = 2 steps.
- From Backup instructions: source choice + destination + Back up database = 3 steps.
- From Preserve instructions, with an active session: source + destination + End app session + Preserve database files = 4 steps.
- From Restore instructions: two fields + Review restore + Restore database = 4 steps.
- From Repair instructions, with an active session: source + check + Show instructions + destination + End app session + repair = 6 steps.
- Selecting another task adds 2 steps.
- Reaching initial Database from elsewhere through Settings and Diagnostics adds 2 steps.
- Preservation or repair without an active session removes the End app session step.

### Confirmations and branches

Restore review uses this template:

> "Ready for explicit Restore database.  
> Chosen backup: {}  
> Configured database to replace: {}  
> Preservation directory: {}  
> Validated candidate: {}  
> This replaces database records only. Keep the preservation directory to recover the previous database. Changes to the backup, destination, configuration or session require a new review."

Evidence: `src/view_models/startup/database.rs:389`.

This is a review report and explicit installation control. The handler does not open a separate confirmation dialog. Evidence: `src/presentation/database_tools.rs:95`.

- "Upgrade backup" prepares a candidate from an older valid backup. It does not replace the chosen backup. Evidence: `src/view_models/startup/database.rs:116`.
- Changed restore inputs invalidate the current review. Evidence: `src/view_models/startup/database.rs:376`.
- "Repair interrupted upgrade" is visible only for the recognized source condition. Evidence: `src/view_models/startup/database.rs:289`.
- "Cancel" requests cancellation and waits for the operation’s result and cleanup. Evidence: `src/view_models/startup/database.rs:371`, `src/view_models/startup/database.rs:404`.
- "Copy database report" copies the accumulated report. Evidence: `src/presentation/database_tools.rs:112`.
- A successful restore can reopen a fresh session after core checks. Evidence: `src/view_models/startup/database.rs:614`, `src/app/startup.rs:595`.
- Preservation alone leaves normal work stopped. Evidence: `src/view_models/startup/database.rs:576`.

### Error text

Database errors use template `"{operation}: {reason}"`. Remaining artifacts add a path-specific retention message. Evidence: `src/view_models/startup/database.rs:633`.

| Failure | Exact reason text or opening sentence | Evidence |
|---|---|---|
| Missing source or parent | "The file or its parent folder does not exist. Check the path and mounted storage. Then repeat the check." | `src/view_models/startup/database.rs:635` |
| Access | "App could not access the file. Check file and folder permissions. Then repeat the check." | `src/view_models/startup/database.rs:636` |
| Invalid file | "The selected path is not a regular database file. Select an existing SQLite database." | `src/view_models/startup/database.rs:637` |
| Busy | "SQLite reported a busy or locked database. Close other apps that use this database. Then repeat the check." | `src/view_models/startup/database.rs:638` |
| Corrupt | "SQLite could not read valid database contents. Retain the original files. Check a known backup before recovery." | `src/view_models/startup/database.rs:639` |
| I/O | "Storage could not complete the operation. Check free space, the mounted drive and permissions before retrying." | `src/view_models/startup/database.rs:640` |
| Cancelled | "The operator cancelled the operation before completion. Choose Check database or Back up database to start a new attempt." | `src/view_models/startup/database.rs:641` |
| Deadline | "The operation exceeded its 60-second limit. Check storage and competing database activity before retrying." | `src/view_models/startup/database.rs:642` |
| Destination conflict | "The destination is occupied, aliases the source or its journals, or the private candidate changed." | `src/view_models/startup/database.rs:643` |
| Candidate validation | "App did not accept the candidate after its integrity, foreign-key and schema checks." | `src/view_models/startup/database.rs:644` |
| SQL | "SQLite could not complete this check. Check the selected database and its schema before retrying." | `src/view_models/startup/database.rs:645` |
| Unsupported protocol | "App cannot establish the supported maintenance protocol for this source or session." | `src/view_models/startup/database.rs:646` |
| Unstable files | "A source file changed or its copied bytes did not match." | `src/view_models/startup/database.rs:647` |

Installation distinguishes stopped installation, verified rollback, unverified rollback, and failed verification. It does not claim rollback after committed installation fails verification. Evidence: `src/view_models/startup/database.rs:609`.

**End state:** Inspection report, verified backup, preservation copy, installed database, repaired candidate, or retained failure evidence.

**Narrow layout:** The same fields and actions remain. Destructive actions move first in the stacked action band. Evidence: `src/view_models/maintenance.rs:75`.

## 9. End the app session and return to recovery

**Entry points:**

- "End app session" on Diagnostics → "App session": `src/view_models/startup/session.rs:71`.
- Same label in core configuration repair: `src/view_models/startup/correction.rs:212`.
- Same label in database preservation and repair: `src/view_models/startup/database.rs:224`.

### Main path

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Click "End app session". | A session report appears. New commands stop entering the session. | `src/view_models/startup/session.rs:52` |
| 2 | Wait. | The app finishes tracked work, stops built-in playback, and releases database connections. | `src/view_models/startup/session.rs:121` |
| 3 | Use recovery tools. | Startup, Configuration, and Database pages are available. | `src/view_models/maintenance.rs:99` |
| 4 | Click "Check again", then "Open app", when appropriate. | Core resources are checked before a fresh session opens. | `src/app/startup.rs:183` |

**Count — INFERENCE:** 1 step from the session action to recovery when drain succeeds. Two further steps reopen through Check again and Open app.

The explanation states:

> "End this app session to open recovery and maintenance. App commands and built-in playback will stop. Unsaved Settings edits will be discarded. External relay sender and encoder services will keep running."

Evidence: `src/view_models/startup/session.rs:43`.

If drain fails, the report uses:

> "App could not finish ending this session. Maintenance is not available. Let the named work finish, then choose Retry drain."

It adds template `"Remaining: {}"`. Evidence: `src/view_models/startup/session.rs:116`.

"Retry drain", "Copy report", and "Quit" remain available according to state. Evidence: `src/view_models/startup/session.rs:86`.

**Count — INFERENCE:** Each retry, copy, or quit action adds 1 step.

**End state:** Recovery, a blocked drain report, or app exit. External service outcomes are **UNSURE**.

## 10. Recover optional tools and failed MusicIndex searches

### 10.1 Capability notices and Background tools

**Entry points:**

- Status report item, template "{count} reports": `src/view_models/status_bar.rs:208`.
- "View tools in Settings": `src/view_models/startup/capabilities.rs:273`.
- Diagnostics menu item "Background tools": `src/view_models/settings.rs:115`.
- Setup-item edit controls in Inbox: `src/view_models/startup/capabilities.rs:150`.
- Contextual "Repair playback", "Repair tool", "Repair action", and "Repair encoder": `src/view_models/queue_now_playing.rs:412`, `src/view_models/show.rs:1688`.

### Main path

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Open the report notice or Background tools. | Dependency issues and retained actions appear. | `src/app/capabilities.rs:74`, `src/view_models/startup/capabilities.rs:167` |
| 2 | Click the dependency’s edit control. | Settings opens Configuration repair with the relevant field selected. | `src/app/capabilities.rs:164` |
| 3 | Type the correction and click "Save correction". | The repair report opens. Changed optional tools receive automatic checks. | `src/app/startup.rs:91` |
| 4, when needed | Click a dependency check control. | The app performs a fresh setup check. The label becomes "Checking…" while working. | `src/app/capabilities.rs:182`, `src/view_models/startup/capabilities.rs:352` |
| 5 | Open the retained action’s controls in Background tools. | The original action remains available only under its current recovery conditions. | `src/view_models/startup/capabilities.rs:199` |
| 6 | Click its explicit retry or run control. | The original action runs. | `src/app/capabilities.rs:133` |

The persistent tool list includes:

"Background runtime", "Thumbnail maintenance", "Music Index", "Built-in playback", "Relay sender host", "Drop-file publication", "Stream encoder", "Audio converter", and "Presentation settings".

Evidence: `src/view_models/startup/capabilities.rs:85`, `src/view_models/startup/capabilities.rs:449`.

### Check controls

| Exact label | What the check does | Evidence |
|---|---|---|
| "Check endpoint" | Validates the saved URL without sending a search. | `src/view_models/startup/capabilities.rs:677`, `src/view_models/startup/capabilities.rs:690` |
| "Check player" | Prepares saved playback setup without playing a track. | `src/view_models/startup/capabilities.rs:678`, `src/view_models/startup/capabilities.rs:693` |
| "Check relay sender" | Reads service state without starting or stopping a service. | `src/view_models/startup/capabilities.rs:679`, `src/view_models/startup/capabilities.rs:696` |
| "Check publication folder" | Tests directory access without publishing metadata. | `src/view_models/startup/capabilities.rs:680`, `src/view_models/startup/capabilities.rs:699` |
| "Check encoder" | Reads encoder status without connecting or disconnecting. | `src/view_models/startup/capabilities.rs:681`, `src/view_models/startup/capabilities.rs:702` |
| "Check converter setting" | Tests saved converter executables without converting a file. | `src/view_models/startup/capabilities.rs:682`, `src/view_models/startup/capabilities.rs:705` |
| "Check display settings" | Validates and applies saved presentation settings. | `src/view_models/startup/capabilities.rs:683`, `src/view_models/startup/capabilities.rs:708` |
| "Check again" | Handles other dependencies, including runtime recovery. | `src/view_models/startup/capabilities.rs:684`, `src/view_models/startup/capabilities.rs:711` |

**Count — INFERENCE:** Each check is 1 step from its displayed control. Automatic checks after Save correction are zero steps.

Retained run labels include:

"Run search again", "Start selected Show cue entry", "Stop Show", "Pause original track", "Resume original track", "Play next track", "Play previous track", "Connect stream", and "Disconnect stream".

Evidence: `src/view_models/startup/capabilities.rs:756`.

Conversion recovery labels include "Redownload original track", "Retry original conversion", and "Retry original track". Evidence: `src/view_models/startup/capabilities.rs:745`.

"Dismiss" removes the retained action. For conversion entries, its handler also submits conversion discard. Evidence: `src/app/capabilities.rs:134`.

### Missing mpv or ssh

| Failure | Text and available continuation | Evidence |
|---|---|---|
| Missing mpv | "mpv is not installed. Install mpv, then choose Check player in Settings." | `src/view_models/startup/capabilities.rs:491` |
| Missing ssh | State "ssh not installed", with "ssh is not installed on this computer. Install OpenSSH, then check again." | `src/view_models/show.rs:501`, `src/view_models/show.rs:514` |
| Missing unit | "Not installed", with "Unit file not found." | `src/view_models/show.rs:500`, `src/view_models/show.rs:513` |
| Unreachable host | "Not reachable", with "Host cannot be reached." | `src/view_models/show.rs:502`, `src/view_models/show.rs:518` |

Package installation happens outside these app controls. Its steps and success are **UNSURE**.

A player check can fail with:

> "App kept its loaded player unchanged. End the app session before changing playback settings."

Evidence: `src/application/capability_recovery/setup.rs:65`.

An encoder check can fail with:

> "No stream encoder is configured. Configure broadcast.encoder before retrying a stream action."

Evidence: `src/application/capability_recovery/setup.rs:104`.

### 10.2 Failed MusicIndex search

**Entry points:** A submitted toolbar search or a retained "Run search again" action. Evidence: `src/app/toolbar.rs:265`, `src/view_models/startup/capabilities.rs:756`.

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Submit a search. | A failed Index request produces a search-failure summary. Local Library availability remains stated separately. | `src/view_models/search_results/failure.rs:59` |
| 2 | Click "Show details". | The timestamped technical report appears. | `src/view_models/search_results/failure.rs:113` |
| 3, optional | Click "Copy report". | The complete report is copied, including details that were not expanded. | `src/view_models/search_results/failure.rs:116`, `src/view_models/search_results/failure.rs:146` |
| 4 | Correct the query, connection, endpoint, or dependency. | The relevant draft or external condition changes. | `src/view_models/search_results/failure.rs:60` |
| 5 | Submit again, or use "Run search again" after recovery. | A new request runs. | `src/view_models/startup/capabilities.rs:756` |

Failure summaries include:

- Endpoint unavailable: "App could not start the Music Index search because its endpoint configuration or setup is unavailable. No search request was sent."
- Runtime unavailable: "App could not start the Music Index search because its background runtime is unavailable. No search request was sent."
- Other request failure: "App could not get search results from Music Index."
- Cancellation: "App cancelled the Music Index search before it completed."
- Rejected query template: `"Music Index did not accept the query \"{query}\". Change the query, then search again."`

Evidence: `src/view_models/search_results/failure.rs:53`, `src/view_models/search_results/failure.rs:60`.

"Hide details" reverses the disclosure. Evidence: `src/view_models/search_results/failure.rs:110`.

**Counts — INFERENCE:**

- From a displayed failure to details: 1 step.
- From a displayed failure to copied report: 1 step.
- From a rejected query to another search: one typed query + submit = 2 steps.
- Endpoint repair and replay depend on the retained-action route described above.

**End state:** A changed dependency, new operation result, retained failure report, or dismissed recovery entry.

**Narrow layout:** Notice buttons wrap. Settings entry through "More" adds one click compared with the sidebar. Direct repair controls retain their routes.

## 11. Job 1: first launch to hear real audio

**Start:** Fresh install.

**Finding:** Initial launch does not produce real audio. The default playback driver is `"null"`, and the generated configuration leaves mpv setup commented. Evidence: `src/config.rs:113`, `src/config.rs:1167`.

A longer path exists through player configuration and a playable Show cue. Audible output remains **UNSURE** without running mpv and checking the audio system.

### Current path

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Launch HitIt. | Startup automatically prepares the fresh configuration and core resources. | `src/main.rs:15`, `src/startup.rs:198` |
| 2 | Press Ctrl+3, code label "Settings". | Settings opens. | `src/app/keyboard.rs:158` |
| 3 | Select "Diagnostics". | Diagnostic tools appear. | `src/view_models/settings.rs:44` |
| 4 | Open the diagnostic selector. | Diagnostic page choices appear. | `src/app/settings.rs:157` |
| 5 | Select "Configuration". | Configuration repair appears. | `src/view_models/settings.rs:114` |
| 6 | Click "Edit configuration". | The editor loads. | `src/view_models/startup/correction.rs:201` |
| 7 | Select "playback.driver". | The player driver field appears. | `src/config/correction.rs:27` |
| 8 | Type `mpv`. | The draft driver changes. | `src/config/correction.rs:56`, `src/view_models/startup/correction.rs:338` |
| 9 | Click "Save correction". | Configuration saves. The changed playback setup receives an automatic check. | `src/app/startup.rs:91` |
| 10 | Obtain a playlist containing a downloaded, available track. | A playlist exists with playable local audio. This is the Music acquisition part outside Run A’s detailed scope. | `src/view_models/queue_now_playing.rs:192`, `src/playback_owner.rs:384` |
| 11 | Open that playlist’s row menu and select "Load into Show Cue". | A stored cue loads. An RSS check also starts. | `src/view_models/library.rs:778`, `src/library/app_impl.rs:2242` |
| 12 | Open "Show". | Show exposes the loaded cue and transport. | `src/app/keyboard.rs:433`, `src/app/show.rs:134` |
| 13 | Activate "Start", accessibility label "Start Show", or Ctrl+Alt+P. | The app submits StartShow for the selected cue entry. | `src/view_models/queue_now_playing.rs:119`, `src/app/playback_bar.rs:89` |

**Count — INFERENCE:** 14 + M steps for this path.

Here, M is the Music acquisition work in table step 10. Opening the playlist menu and selecting its item count as two steps.

**UNSURE:** A single fresh-install total cannot be established within this scope. It depends on music acquisition, installed mpv, available files, and external audio setup.

Branches:

- Installing mpv adds external steps whose count is **UNSURE**.
- A custom `"playback.mpv_path"` adds a field selection and typed field. Evidence: `src/config/correction.rs:28`.
- Existing playback can prevent replacing the loaded driver. Ending the session then becomes necessary. Evidence: `src/application/capability_recovery/setup.rs:65`.
- Cue entries can show "Not in Library", "Not downloaded", or "File missing". Evidence: `src/view_models/queue_now_playing.rs:192`.
- An empty playlist reports "The playlist has no tracks. The Show cue did not change." Evidence: `src/view_models/library.rs:3967`.
- A failed cue change uses `"The Show cue did not change: {error:#}"`. Evidence: `src/view_models/library.rs:3963`.
- An existing cue changes the menu label to "Replace Show Cue". The handler replaces it directly. Evidence: `src/view_models/library.rs:792`, `src/library/app_impl.rs:2187`.

**Narrow layout:** Ctrl+3 avoids the extra "More" step. Reaching "My playlists" through navigation requires "More" and its menu item.

**End state:** The app requests playback through mpv. Actual heard audio is **UNSURE**.

## 12. Job 16: configure a stream encoder

**Start:** Anywhere in the normal shell.

**Entry points:**

- Settings shortcuts and navigation from Section 3.
- "Edit encoder settings" in Background tools: `src/view_models/startup/capabilities.rs:649`.
- "Repair encoder" when a stream action projects a recoverable capability problem: `src/view_models/show.rs:1732`.

### Current path

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Press Ctrl+3. | Settings opens. | `src/app/keyboard.rs:158` |
| 2 | Select "Diagnostics". | Diagnostic tools appear. | `src/view_models/settings.rs:44` |
| 3 | Open the diagnostic selector. | Page choices appear. | `src/app/settings.rs:157` |
| 4 | Select "Configuration". | Configuration repair appears. | `src/view_models/settings.rs:114` |
| 5 | Click "Edit configuration". | The current configuration loads. | `src/view_models/startup/correction.rs:201` |
| 6 | Select "broadcast.encoder". | The structured encoder field appears. | `src/config/correction.rs:33` |
| 7 | Type a TOML `value` assignment for the encoder. | The draft holds the encoder configuration. | `src/config/correction.rs:215` |
| 8 | Click "Save correction". | The app validates and saves it. The changed encoder receives an automatic setup check. | `src/app/startup.rs:91`, `src/application/capability_recovery/setup.rs:104` |

**Count — INFERENCE:** 8 steps for the stated path with an unloaded editor.

The generated configuration documents `binary_path`, `default_server_name`, `address`, and `port`. It gives `"butt"` as the example binary. Evidence: `src/config.rs:1185`.

**Alternate count — INFERENCE:** From a displayed "Edit encoder settings" control: edit control + typed field + Save correction = 3 steps.

### Branches and verification

| Action or condition | Result | Evidence |
|---|---|---|
| Click "Test draft and paths". | The draft is validated without saving. | `src/view_models/startup/correction.rs:209` |
| Click "Check encoder". | The app reads encoder status. It does not connect or disconnect. | `src/view_models/startup/capabilities.rs:702` |
| Encoder absent from configuration | "No stream encoder is configured. Configure broadcast.encoder before retrying a stream action." | `src/application/capability_recovery/setup.rs:106` |
| Encoder status request fails | "App could not read the configured encoder's status. Check its executable and connection, then check again." | `src/application/capability_recovery/setup.rs:108` |
| Open Show and select "Stream" | The detail shows server, connection, signal, recording, listeners, and reported song where available. | `src/view_models/show.rs:2742` |
| Click "Connect" | The app sends the stream connection command. | `src/view_models/show.rs:2883`, `src/app/show.rs:194` |
| Click "Disconnect" | The app sends the disconnection command. | `src/view_models/show.rs:2893`, `src/app/show.rs:199` |

Connection states are "Connected", "Connecting", "Disconnected", "Working", "Not installed", "Not reachable", and "Unknown". Evidence: `src/view_models/show.rs:1225`.

Signal states include "Audio present", "No audio signal", and "Signal unknown". Evidence: `src/view_models/show.rs:1298`.

The connection controls are absent when no encoder is configured or its state is NotInstalled. Evidence: `src/view_models/show.rs:2875`.

Stream command failures use template `"Stream command error: {error:#}"`. Evidence: `src/app/show.rs:1113`.

**End state:** HitIt stores encoder control configuration and checks its status. Creating the encoder’s server profile and supplying real audio are **UNSURE** external work.

**Narrow layout:** Keyboard entry keeps the 8-step count. Sidebar entry through "More" adds one step.

## 13. MH6 Get set up: first run plus show setup

**Start:** Fresh install.

**Requested end state:** Audio works and services are green.

The code has separate startup, player, publication, relay sender, and encoder paths. It has no single setup journey with this combined completion state.

### Current path

| Step | User action | What appears next | Evidence |
|---|---|---|---|
| 1 | Launch HitIt. | Core startup checks run. | `src/app/startup.rs:169` |
| 2 | Configure playback and load a playable cue through Job 1. | Show can submit a real-player command. | `src/config.rs:1167`, `src/app/playback_bar.rs:89` |
| 3 | Configure `"broadcast.hosts"` and `"broadcast.selected_host"` where required. | The relay sender target changes after saved setup checks. | `src/config/correction.rs:29`, `src/application/capability_recovery/setup.rs:76` |
| 4 | Configure `"broadcast.drop_directory"` and `"broadcast.drop_file_target"` where required. | Built-in playback metadata publication gains its configured destination. | `src/config/correction.rs:31`, `src/application/capability_recovery/setup.rs:93` |
| 5 | Configure `"broadcast.encoder"` through Job 16. | Encoder status becomes available if its external setup answers. | `src/application/capability_recovery/setup.rs:104` |
| 6 | Open Background tools and run required checks. | Reports distinguish local setup checks from external service responses. | `src/view_models/startup/capabilities.rs:688` |
| 7 | Open Show and use the applicable service "Start" and stream "Connect" controls. | Service commands run and status observations update. | `src/view_models/show.rs:2917`, `src/app/show.rs:179`, `src/app/show.rs:194` |

Inbox also lists setup issues that map to Show dependencies. Its edit control opens the corresponding configuration field. Evidence: `src/view_models/startup/capabilities.rs:146`.

**Count — INFERENCE:** No fixed total. The path combines Job 1, encoder setup, selected configuration fields, and external service installation.

For a loaded configuration editor, each additional field costs one selection and one typed field. One Save correction can save several edited fields. Evidence: `src/view_models/startup/correction.rs:385`.

**UNSURE:** The requested “audio works and services are green” completion cannot be proved by setup checks alone.

- "Check endpoint" does not test a search response.
- "Check player" does not play audio.
- "Check publication folder" does not publish metadata.
- "Check encoder" does not connect the stream.

Evidence: `src/view_models/startup/capabilities.rs:688`.

**End state:** Separate observed capability and service states. There is no combined setup-complete screen in the traced path.

**Narrow layout:** "Show" and "Inbox" remain direct tabs. "Settings" is under "More". Direct issue edit controls keep their routes.

## 14. MH7 Recover when something breaks

**Start:** Where the failure appears.

**Requested end state:** The problem is fixed without searching through Settings.

The code provides direct repair controls for some failures. Other failures provide report text and instructions for external work.

### Current paths and counts

| Failure location | User action | What appears next | Count — INFERENCE | Evidence |
|---|---|---|---:|---|
| Capability notice with an editable failure | Click its edit control, type a correction, click "Save correction". | The repair report opens and changed optional tools receive checks. | 3 to saved correction | `src/app/capabilities.rs:164`, `src/app/startup.rs:91` |
| Same retained-action failure | After correction, open the diagnostic page selector, select "Background tools", and run the retained action. | The original action runs. | 6 total with the preceding 3-step correction | `src/view_models/settings.rs:115`, `src/app/capabilities.rs:133` |
| Status report item before notice disclosure | Click the report item first. | The notice opens. | Add 1 | `src/app/status_bar.rs:115` |
| Tool needs only a fresh check | Click its check control. | Setup is checked without retrying the original action. | 1 | `src/app/capabilities.rs:182` |
| Rejected search query | Type a changed query and submit. | Another search runs. | 2 | `src/view_models/search_results/failure.rs:53` |
| Search request or connection failure | Read "Show details" or "Copy report", correct the external condition, then resubmit. | A new result or failure appears. | 1 for disclosure or copy, then an external count marked UNSURE, then 1 submit | `src/view_models/search_results/failure.rs:77` |
| Core storage failure in recovery | Correct storage externally, click "Check again", then "Open app". | Core checks permit or refuse a fresh session. | 2 app steps plus external steps marked UNSURE | `src/startup.rs:87`, `src/app/startup.rs:183` |
| Supported interrupted database upgrade | Use the repair path in Section 8. | A preserved, validated repair is attempted. | 6 from active-session Repair instructions | `src/view_models/startup/database.rs:117` |
| Missing mpv or ssh | Install the named software, then check again. | Tool state is checked. | External steps marked UNSURE, plus 1 check from its control | `src/view_models/startup/capabilities.rs:491`, `src/view_models/show.rs:514` |

The 6-step retained-action path assumes one editable field and automatic setup checks that succeed. It excludes external installation and further errors.

### Branches and limitations

- Saving a correction does not repeat the failed action. Evidence: `src/view_models/startup/correction.rs:452`.
- Compact notice controls can open the original action’s Settings controls without running it. Evidence: `src/view_models/startup/capabilities.rs:199`.
- Missing software requires external installation. Evidence: `src/view_models/startup/converter.rs:11`, `src/view_models/show.rs:514`.
- Database recovery can remain open after failed installation or unverified rollback. Evidence: `src/view_models/startup/database.rs:617`.
- A successful check can validate configuration without proving external reachability. Evidence: `src/view_models/startup/capabilities.rs:498`.

**End state:** A retried operation, a checked tool, a new session, or a retained report. A fixed external problem is **UNSURE** until its operation succeeds.

**Narrow layout:** Direct repair controls retain the same route. Manual Settings navigation through "More" adds one step.

## 15. Inactive controls, different labels, and displaced feedback

### 15.1 Controls and handlers that perform no action in some states

| Control or command | Condition and behavior | Evidence |
|---|---|---|
| "Application mark" | Tooltip-only mark. No activation handler is attached. | `src/app/toolbar.rs:27` |
| "Refresh Library", Ctrl+R | Does nothing outside Music. | `src/app/keyboard.rs:448` |
| Active-pane "Back", Escape | The top-level handler does nothing on Show and Settings. Editor and dialog handlers can intercept Escape separately. | `src/app/keyboard.rs:453`, `src/ui/composites/maintenance_forms.rs:55` |
| Active-pane "Open", Enter | The top-level handler does nothing on Show and Settings. Focused controls have their own activation behavior. | `src/app/keyboard.rs:664` |
| "Select Focused Row" and "Select All Rows" | Top-level handlers do nothing on Show and Settings. | `src/app/keyboard.rs:629`, `src/app/keyboard.rs:647` |
| "Move Row Up" and "Move Row Down" | Handlers act only while Music is selected. | `src/app/keyboard.rs:610`, `src/app/keyboard.rs:622` |
| "Back" and "Forward" history commands | No history means no navigation. | `src/app/menu.rs:209`, `src/app/menu.rs:221` |
| "Show Play/Pause" | No cue intent means the handler returns without submitting playback. | `src/app/playback_bar.rs:75` |
| "Next Show cue entry" / "Previous Show cue entry" | Unavailable movement returns without submitting a command. | `src/app/playback_bar.rs:51`, `src/app/playback_bar.rs:63` |
| Cached deletion | An empty path set returns without a command. | `src/app.rs:1550` |

The examined Settings and maintenance action builders attach handlers to their action buttons. Disabled availability can prevent activation. Evidence: `src/ui/composites/maintenance_forms.rs:196`, `src/ui/composites/maintenance_forms.rs:354`.

The empty `SettingsEffect::Navigate` handler is not an inactive navigation control. The Settings view model changes selection before returning that effect. Evidence: `src/view_models/settings.rs:373`, `src/app/settings.rs:53`.

### 15.2 Different labels associated with the same action

| Labels | Relationship | Evidence |
|---|---|---|
| "Settings" / "Preferences..." | Both select Settings. | `src/app/keyboard.rs:424`, `src/app/menu.rs:199` |
| "Find" / "Focus Search" | Both focus global search. | `src/app/keyboard.rs:128`, `src/app/keyboard.rs:395` |
| "Search" / "Search Library and Index" | Visible submit label versus icon accessibility label. | `src/view_models/app_toolbar.rs:126` |
| "Use Defaults…" / "Use Defaults" | Entry control versus confirmation control. | `src/view_models/settings.rs:256`, `src/view_models/settings.rs:478` |
| "flac_path" / "Converter setup" / "Edit converter setting" | Stored field name, editor button label, and capability edit entry. | `src/config/correction.rs:24`, `src/view_models/startup/correction.rs:205`, `src/view_models/startup/capabilities.rs:650` |
| "Check again" / "Checking…" | Idle versus working startup check. | `src/view_models/startup.rs:264` |
| "Open app" / "Opening…" | Idle versus working opening request. | `src/view_models/startup.rs:273` |
| "Show details" / "Hide details" | Opposite states of search-report disclosure. | `src/view_models/search_results/failure.rs:110` |
| "Show report" / "Show instructions" | Opposite states of maintenance content disclosure. | `src/view_models/maintenance.rs:15` |
| "Start", "Pause", "Resume", "Retry" / "Repair playback" | Show transport labels change with playback and capability state. | `src/view_models/queue_now_playing.rs:48`, `src/view_models/queue_now_playing.rs:412` |
| "Connect" / "Connect stream" | Show control versus retained encoder recovery action. | `src/view_models/show.rs:2886`, `src/view_models/startup/capabilities.rs:794` |
| "Disconnect" / "Disconnect stream" | Show control versus retained encoder recovery action. | `src/view_models/show.rs:2895`, `src/view_models/startup/capabilities.rs:795` |
| "Load into Show Cue" / "Replace Show Cue" | The same menu-action variant loads or replaces according to cue state. | `src/view_models/library.rs:787` |
| "Copy", "Copy report", "Copy startup report", "Copy repair report", "Copy database report" | Related clipboard controls identify different copied objects. | `src/view_models/settings.rs:328`, `src/view_models/startup.rs:262`, `src/view_models/startup/correction.rs:214`, `src/view_models/startup/database.rs:240` |

### 15.3 Results and errors shown away from their originating controls

| Originating action | Result or error location | Evidence |
|---|---|---|
| Save General or Library values | Shared Settings message below the whole form. | `src/app/settings.rs:224`, `src/app.rs:1486` |
| Delete one cached file or all cached files | Shared Settings message, followed by cache-list reload. | `src/app.rs:1561` |
| Configuration validation, converter testing, or Save correction | The page switches from instructions/editor content to its report. | `src/presentation/configuration_editor.rs:160` |
| Save correction for an optional tool | Checks run through Background tools while the configuration report remains the editor’s report surface. | `src/app/startup.rs:91`, `src/app/capabilities.rs:332` |
| Database check, backup, restore review, preservation, or repair | The database page switches to its report. | `src/view_models/startup/database.rs:356` |
| "Use configured database" | Source field updates, but explanatory text accumulates in the database report without the same automatic view switch. | `src/view_models/startup/database.rs:356`, `src/view_models/startup/database.rs:462` |
| Capability issue outside Settings | A status report count appears. The notice requires disclosure through that count. | `src/app.rs:2212`, `src/app/status_bar.rs:115` |
| Show playback commands | Their text enters shared `settings_status`. Show also receives that status message. | `src/app/playback_bar.rs:41`, `src/app/show.rs:475` |
| Stream command | Shared status carries template "Stream command error: {error:#}". Show displays the shared message. | `src/app/show.rs:1113`, `src/app/show.rs:475` |
| Some navigation and search-opening failures | They enter `settings_status`, which General and Library render below their forms. | `src/app.rs:1022`, `src/app/search_live.rs:247`, `src/app/settings.rs:224` |
| Optional startup notices | They enter shared Settings status after normal mounting. | `src/app/bootstrap.rs:296` |
| Failed initial window creation | A startup report goes to stderr because no error window opens. | `src/app/bootstrap.rs:113` |
| Font or system-accessibility fallback | A terminal message reports the fallback. | `src/app/bootstrap.rs:60`, `src/ui/system_settings.rs:55` |
| Some startup audio-cache repairs | Errors go to stderr while startup continues. | `src/startup.rs:449` |
| Successful database restore | A fresh session can open. The retained report directs the user to Settings → Diagnostics → Database. | `src/view_models/startup/database.rs:614` |