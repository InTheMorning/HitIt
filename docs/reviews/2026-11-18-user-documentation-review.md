# User Documentation Review - 2026-11-18

## Scope

This review compares the user documents of HitIt with the code at git `d9cc47e`.
The orchestrator requested it. It does not correct the documents. The operator decides the corrections.

Documents in scope:

- `README.md`
- `docs/architecture/app-overview.md`
- `docs/runbooks/workflows.md`
- `docs/runbooks/broadcast-operations.md`
- `docs/runbooks/arch-package.md`
- In-app text that tells the user what to do.

`docs/schema/storage-and-metadata.md` and `docs/README.md` are for a developer. This review examines them only where
`README.md` links to them.

## Method

- The main session read each document line by line.
- A ChatGPT run (`gpt-6.1-sol`, effort medium) made an inventory of the user surface with file and line evidence.
- The main session examined each claim of the inventory in the code before it used the claim.
- The main session ran the STE checker on each document.

Severity:

- **wrong**: the document states a fact that the code does not have, or has with a different name or location.
- **missing**: the app supports a user task, but no user document tells it.
- **unclear**: the text is for a different audience, a procedure has no steps or no screen result, or a term has no
  definition.

## Current User Surface

These facts are the reference for the findings. Each has code evidence.

- The top tabs are Music, Show and Settings (`src/app.rs:103`).
- The sidebar has Search, Releases, Artists, My playlists, Public playlists and Live. A line follows, then Show, Inbox
  and Settings (`src/view_models/sidebar.rs:143`, labels at `src/view_models/sidebar.rs:362`).
- Releases and Artists have the scope "All | In Library" (`src/view_models/list_page.rs:333`).
- "Check all feeds" and "Apply updates (n)" are at the bottom of the sidebar (`src/library/app_impl.rs:5799`,
  `src/view_models/library.rs:5404`).
- Settings has the tabs General, Library and Diagnostics (`src/view_models/settings.rs:40`).
  - General: Appearance (UI scale, Theme) and Defaults (Default values, "Use Defaults…").
  - Library: Music Index (Music Index endpoint), Files (Music directory, Audio converter) and Audio cache (Cached files).
  - Diagnostics: the version line with "Copy" as the first row, then the pages Database, Configuration, Background
    tools and App session (`src/view_models/settings.rs:50`, `src/view_models/settings.rs:93`,
    `src/view_models/settings.rs:314`).
- The Show page has the cards Source, Live Metadata and Stream, and the side panel Cuelist
  (`src/view_models/show.rs:1434`, `src/view_models/show.rs:1490`).
- The track actions are "Download Track", "Download Release", "Add to Playlist", "Copy Feed URL",
  "MusicBrainz Lookup", "Inspect Sources" and "Remove Track…" (`src/view_models/track_detail.rs:934`).
  No control has the label "Subscribe" or "Unsubscribe".

## Findings

### README.md

| ID | Line | Severity | Finding | Code evidence | Proposed correction |
| --- | --- | --- | --- | --- | --- |
| R-01 | 20-27, 114-117 | wrong, corrected in `966264c` | The README names a `Discover` tab, a `Library` tab and a `Cached` view. None exists. | `src/app.rs:103`, `src/view_models/sidebar.rs:143` | Describe the three tabs and the sidebar destinations. Describe the scope "All \| In Library". |
| R-02 | 141-150 | wrong, corrected in `966264c` | `Cmd/Ctrl+1` opens Music, not Library. `Cmd/Ctrl+2` opens Show, not Discover. `Esc` is "Back". The list omits Ctrl+Alt+P, Ctrl+Alt+Left, Ctrl+Alt+Right, Ctrl+N, Ctrl+A, Ctrl+, and Ctrl+Q. | `src/app/keyboard.rs:109-174`, `src/app/menu.rs:82-113` | Replace the list with the table of `APP_KEY_BINDING_SPECS` and the menu keys. |
| R-03 | 63-65 | wrong, corrected in `966264c` | No row "flac binary" exists. The row is Settings, Library, Files, "Audio converter". It opens Converter setup. | `src/view_models/settings.rs:182`, `src/view_models/settings.rs:455` | Name the row "Audio converter" and the tab path. |
| R-04 | 54-63 | wrong, corrected in `966264c` | The README says that without `flac` a WAV download stays WAV. The app tries `flac`, then `ffmpeg`. | `src/audio_format.rs:196-210`, `packaging/arch/PKGBUILD:38` | Say that `flac` or `ffmpeg` converts WAV. Say what occurs when neither is installed. |
| R-05 | 92-96 | missing | The Settings list gives three config keys. It omits UI scale, Theme, Use Defaults, Cached files and Diagnostics. It gives key names, not the row labels. | `src/view_models/settings.rs:50-80`, `src/view_models/settings.rs:176-187` | Give the tabs, the sections and the row labels. |
| R-06 | 104 | wrong, corrected in `966264c` | The thumbnail cache is in the config directory (`~/.config/hitit/thumbnail-cache`), not beside it. The README does not name the audio cache `~/.cache/hitit/audio`. | `src/app/bootstrap.rs:216-219`, `src/config.rs:768` | Give both cache paths. |
| R-07 | 21-24, 30 | wrong, corrected in `966264c` | The README uses subscribe and unsubscribe for actions that now have the labels "Download …" and "Remove …". A removal moves the audio file to the audio cache. | `src/view_models/track_detail.rs:934-946`, `src/view_models/selection_action_bar.rs:338`, `src/application/library_removal.rs:241-300` | Use the current labels. Tell what "Remove from Library…" does to the file. |
| R-08 | 7-16 | missing | The summary omits the show work: the Show page, the live metadata chain, the stream encoder and the playlists. | `src/app.rs:103`, `src/view_models/show.rs:1434`, `AGENTS.md:3-5` | Add show production and playlists to the summary. |
| R-09 | 46-77 | unclear | "Run" and "Build" are for a developer. `cargo run` makes a debug build, which is slow. No section tells a user how to install. The README does not link the Arch package runbook. | `packaging/arch/PKGBUILD:91-99` | Add an "Install" section first: the Arch package, then `cargo install --path .`. Put `cargo run --release` in a developer section. |
| R-10 | 79-90 | missing | The README does not tell that the app plays no audio until `[playback] driver = "mpv"` is set. | `src/config.rs:1166-1170` | Add the `[playback]` block and the result. |
| R-11 | 81 | missing | The first start moves the data of the previous name `v4vmm` to the `hitit` folders. A failed move stops the app. The README does not tell this. | `src/main.rs:9-14`, `src/config/user_data_move.rs:15-57` | Add one paragraph for a person who used the app before ADR 0096. |
| R-12 | 106-117 | wrong, corrected in `966264c` | The storage model says that `local_files` holds the cached files. The table `cached_files` holds them. This section is for a developer. | `src/application/library_removal.rs:300`, `docs/schema/storage-and-metadata.md:38-40` | Remove the section from the README. The schema document owns it. |
| R-13 | 152-158 | unclear | The document list links two developer documents and the ADR folder. It does not link the broadcast and package runbooks. | - | Link the user runbooks. Put developer links in a separate list. |

### docs/architecture/app-overview.md

| ID | Line | Severity | Finding | Code evidence | Proposed correction |
| --- | --- | --- | --- | --- | --- |
| A-01 | 15-29 | wrong, corrected in `966264c` | The document describes a "Library" tab with a "Cached" sub-view. | `src/app.rs:103`, `src/view_models/sidebar.rs:143`, `src/view_models/settings.rs:65-79` | Describe Music with its sidebar destinations. Put Cached files in Settings, Library, Audio cache. |
| A-02 | 43-44 | wrong, corrected in `966264c` | The document says that the former Discover modules stay in the tree. No Discover module exists. `grep 'mod discover' src` finds nothing. | `src/view_models/sidebar.rs:362-374` | Remove the sentence and the link to the note. |
| A-03 | 31-35, 10-13, 122-125 | unclear | The text gives ADR numbers and the history of a change ("is no longer a separate tab", "ADR 0047 and ADR 0048 moved"). | - | Describe the present only. Keep ADR references in a developer section. |
| A-04 | 46-53 | wrong, corrected in `966264c` | The Settings section lists two values. Settings has three tabs and nine rows. | `src/view_models/settings.rs:40-80` | Give the same Settings description as R-05. |
| A-05 | 55-77 | missing | The CLI list omits `--version`, `help`, `library repair-paths`, `player ping`, `playback next`, `previous`, `pause`, `resume`, and the `broadcast` and `liveitem` commands. "Phase 2" is history. | `src/cli.rs:670-700`, `src/main.rs:5` | Give the full list from `help_text`, or tell the user to run `hitit help`. |
| A-06 | 88-95 | missing | First Run omits the `v4vmm` data move and the startup check. It does not tell what the screen shows when a check fails ("App needs attention before it can open", "Check again", "Open app"). | `src/main.rs:9-14`, `src/view_models/startup.rs:190`, `src/view_models/startup.rs:267` | Add the move and the recovery screen to the steps. |
| A-07 | 120-128 | missing | The Broadcast section links only an architecture document. It does not describe the Show page, its three cards or the Cuelist. | `src/view_models/show.rs:1434`, `src/view_models/show.rs:1490` | Add a Show section and link `broadcast-operations.md`. |
| A-08 | 137 | unclear | "Stream encoding" is a non-goal, but the Stream card controls the `butt` encoder with "Connect" and "Disconnect". | `src/view_models/show.rs:2873-2882` | Say that the app controls the encoder and does not encode audio. |

### docs/runbooks/workflows.md

| ID | Line | Severity | Finding | Code evidence | Proposed correction |
| --- | --- | --- | --- | --- | --- |
| W-01 | 3-46 | wrong, corrected in `966264c` | The section "Discover Workflow" tells the user to use a `Discover` tab and Subscribe actions. Neither exists. | `src/view_models/sidebar.rs:143`, `src/view_models/track_detail.rs:934-946` | Rewrite as "Find Music" with Search, Releases, Artists and the "Download …" actions. |
| W-02 | 42-46 | wrong, corrected in `966264c` | "Unsubscribe does not automatically delete files from disk." A removal moves the audio file from the music folder to the audio cache. | `src/application/library_removal.rs:241-300` | Tell what "Remove from Library…" does, and where the file goes. |
| W-03 | 48-56 | wrong, corrected in `966264c` | The document describes a `Library` tab with groups by artist. | `src/view_models/sidebar.rs:143`, `src/view_models/list_page.rs:333` | Describe the scope "In Library" of Releases and Artists. |
| W-04 | 87-97 | wrong, corrected in `966264c` | The `Cached` sub-tab does not exist. The list is in Settings, Library, Audio cache. A delete removes the `cached_files` row, not a `local_files` row. | `src/view_models/settings.rs:65-79`, `src/view_models/cached_files.rs:77`, `src/view_models/cached_files.rs:100`, `src/library_service.rs:111` | Give the path, the controls "Delete" and "Delete All Cached", and the result. |
| W-05 | 58-85 | unclear | Compare, staging and MusicBrainz have no steps and no control names. The controls are "Inspect Sources", "Compare ID3", "Apply tags (n)", "Discard staged" and "MusicBrainz Lookup". | `src/view_models/entity_detail.rs:329`, `src/view_models/entity_detail.rs:435`, `src/view_models/track_detail.rs:943-944` | Write numbered steps. Give what the screen shows after each step. |
| W-06 | 103-127 | wrong, corrected in `966264c` | The document does not tell where "Check all feeds" is. It says "Apply updates" appears after a single-feed check. The label is "Apply updates (n)", at the bottom of the sidebar, and the Inbox Updates section also has "Apply updates". | `src/library/app_impl.rs:5799-5803`, `src/view_models/library.rs:5404-5420`, `src/view_models/inbox.rs:170-185` | Give the location and the label. Add the Inbox. |
| W-07 | 157-268 | unclear | The playback section is for a developer: `PlaybackSession`, the mpv IPC socket, `Drop` and "a future playback daemon". | - | Move the section to an architecture document. Keep only the `[playback]` setup for the user. |
| W-08 | 233-239 | unclear | The text says that playlist row play actions in the desktop library route through the owner. AGENTS.md says that only typed Show commands start program playback, and no page of Music starts it. The main session found no "Play" label in the Music view models. This is an inference. A walk must confirm it. | `AGENTS.md` (Known open defects), `src/view_models/show.rs:1490` | After the walk, describe how the user plays a show from the Cuelist. |
| W-09 | 314-321 | wrong, corrected in `966264c` | "Three related states" introduces a list of four states. | - | Say "four", or remove the playback state from the list. |
| W-10 | all | missing | No user document tells these tasks: make and order a playlist (Ctrl+N, "Add to Playlist", Curator Order) and save a search. Also missing: use the Inbox, Live and Public playlists, and back up and restore the database. | `src/app/keyboard.rs:139`, `src/view_models/inbox.rs:28`, `src/view_models/live_page.rs:24`, `src/view_models/startup/database.rs:80-118` | Add one procedure for each task. |

### docs/runbooks/broadcast-operations.md

| ID | Line | Severity | Finding | Code evidence | Proposed correction |
| --- | --- | --- | --- | --- | --- |
| B-01 | 21-25 | wrong, corrected in `966264c` | "Keeps the old entry for Forget." The Show page has no Forget control. Only the CLI forgets an event. | `src/view_models/show.rs:2510-2565`, `src/cli.rs:52-56` | Tell the user to run `hitit broadcast events forget EVENT_ID`. |
| B-02 | 21-25 | unclear | The procedure is a paragraph. It has no steps and no screen result. It does not give the path Show, Live Metadata. | `src/view_models/show.rs:1434`, `src/view_models/show.rs:2517-2530` | Write numbered steps with the result of each. |
| B-03 | 44-46 | unclear | "An event that you reserved from a terminal" has no definition. `hitit broadcast events create` writes the same registry as the app, so "Copy feed tag" is available for it. The control is off only when the stored endpoint gives no URI. | `src/cli.rs:278-279`, `src/view_models/show.rs:2537-2546`, `src/view_models/show.rs:2592` | Say which event has no tag: an event that another tool reserved. |
| B-04 | 61-75 | missing | The document does not give the default token folder `~/.config/hitit/broadcast/tokens`. It does not say that a database backup excludes the token files. | `src/broadcast/tokens.rs:27-33`, `src/view_models/startup/database.rs:175` | Add the folder and the backup scope. |
| B-05 | 77-93 | missing | Attach is given only as a CLI command. The Live Metadata card has "Attach" and "Detach". | `src/view_models/show.rs:2549`, `src/view_models/show.rs:2559` | Give the app procedure first, then the CLI. |
| B-06 | 95-107, 128-130 | unclear | "Use the Show surface" does not name the controls. The controls are "Start", "Stop", "Reset" and "Logs", for the services "Producer" and "Relay sender". | `src/view_models/show.rs:2913-2949` | Name the card, the service and the control. |
| B-07 | 109-112 | missing | The document does not give the keys of a remote host. It does not tell that Settings, Diagnostics, Configuration repair can edit them. | `src/config.rs:140-160`, `src/config/correction.rs:27-36` | Add a `[[broadcast.hosts]]` example with `transport = "ssh"`, `destination` and `instance_name`, and `broadcast.selected_host`. |
| B-08 | 150-165 | unclear | The recovery steps use terminal words. The app has "Replace" for a dead event, and Replace does step 1. Step 5 has no condition the user can see. | `src/view_models/show.rs:2523` | Start with Replace. Give the screen result of each step. |
| B-09 | 167-177 | unclear | The references and the section "Verify The Event Recovery Interface" are for a developer or an agent. | - | Move them to a developer section, or remove them. |
| B-10 | 17 | missing | The Stream card has no procedure. The document does not tell the `[broadcast.encoder]` keys or the controls "Connect" and "Disconnect". | `src/view_models/show.rs:2729-2882`, `src/config.rs:183-192` | Add a "Stream" procedure. |

### docs/runbooks/arch-package.md

The main session found no wrong statement. The package name, the dependencies, `options=('!lto')`, `check()` and the
installed files agree with `packaging/arch/PKGBUILD`.

| ID | Line | Severity | Finding | Code evidence | Proposed correction |
| --- | --- | --- | --- | --- | --- |
| P-01 | 19-31 | missing | The runbook stops after `makepkg -si`. It does not tell how to start the app (the menu entry or `hitit`), what the first start shows, how to update the package, or how to remove it. | `packaging/arch/PKGBUILD:91-99`, `packaging/arch/hitit.desktop:5` | Add "Start", "Update" (run `makepkg -si` again) and "Remove" (`sudo pacman -R hitit-git`). |
| P-02 | 43 | missing | The runbook names `mpv` as the playback driver. It does not tell that the driver stays off until `[playback] driver = "mpv"` is set. | `src/config.rs:1166-1170` | Add the config block. |
| P-03 | 46-62 | unclear | "Package Options" and "Package Checks" are for a package maintainer. | - | Put them under a maintainer heading at the end. |

### In-App Text

| ID | Location | Severity | Finding | Proposed correction |
| --- | --- | --- | --- | --- |
| I-01 | `src/config.rs:1159-1163` | wrong, corrected in `966264c` | The comment in the generated `config.toml` says that without `flac` a WAV download stays WAV. The app also tries `ffmpeg`. The comment uses "e.g." and a macOS command. | Say "`flac` or `ffmpeg`". Remove the Latin abbreviation. |
| I-02 | `src/view_models/show.rs:1769` | unclear | "Use Check again in Background tools." does not give the path. Background tools is in Settings, Diagnostics. `src/view_models/cached_files.rs:138` gives the path "Diagnostics, Background tools". | Give the full path "Settings, Diagnostics, Background tools". |
| I-03 | `src/view_models/cached_files.rs:141` | unclear | "Reopen Settings to try again." The list is on the Library tab of Settings. "Reopen" does not tell which action reads the list again. | Name the action that reads the list again. |
| I-04 | `src/view_models/settings.rs:311-312` | unclear | The Save scope note has a semicolon and 22 words in one sentence. | Use two sentences. |

## STE Checker Results

Command: `python3 ~/.agents/skills/asd-ste100/scripts/ste_lint.py --check --no-heuristics FILE`.

| Document | Findings | Rule 1.6 (word) | Other rules |
| --- | --- | --- | --- |
| `README.md` | 24 | 24 | 0 |
| `docs/architecture/app-overview.md` | 19 | 19 | 0 |
| `docs/runbooks/workflows.md` | 84 | 78 | 6.3 x3, 5.1 x1, 3.4 x1, 9.3 x1 |
| `docs/runbooks/broadcast-operations.md` | 51 | 48 | 5.1 x1, 6.3 x1, 8.1 x1 |
| `docs/runbooks/arch-package.md` | 8 | 8 | 0 |
| `docs/schema/storage-and-metadata.md` | 48 | 44 | 6.3 x2, 3.4 x1, 9.3 x1 |

A rewrite of the wrong sections removes most of these results. The review does not list each word.

## Counts

| Severity | Count |
| --- | --- |
| wrong | 18 |
| missing | 14 |
| unclear | 16 |
| Total | 48 |

Commit `966264c` corrected the 18 wrong findings. The 14 missing findings and the 16 unclear findings stay open.

## Most Important Findings

1. R-01, A-01, W-01, W-03 and W-04: the user documents describe a Discover tab, a Library tab and a Cached view. The app
   has Music, Show and Settings, with a sidebar, and the cache list is in Settings, Library.
2. R-02: the keyboard table sends the user to the wrong tabs. Ctrl+2 opens Show, not Discover.
3. W-02 and R-07: the documents say that a removal keeps the file. The app moves the file to the audio cache, and the
   action labels are "Download …" and "Remove …", not Subscribe.

## Evidence Sources

- The main session verified each finding in the code at `d9cc47e`.
- The ChatGPT inventory supplied candidate facts. The main session did not use a candidate fact that it did not find
  in the code.
- W-08 is an inference. A walk must confirm it.
