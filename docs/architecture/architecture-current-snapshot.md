# HitIt Architecture - Current Snapshot

## Status

Current - 2026-11-14, from commit `86d97e7`. This snapshot is advisory. It states no rule.
The layer rules are in ADRs 0023, 0024 and 0042, and `tests/architecture_tests.rs` enforces part of them.
The shell is ADR 0089, the list bar is ADR 0090 and the fit check is ADR 0097.

A subagent surveyed the code at `86d97e7`. The primary session examined the important claims against the code.
When the code and this snapshot disagree, the code is correct. Update this snapshot in the same change as a
change of layer, shell or async model.

`architecture-diagrams.md` is the view of 2026-04-30. Older ADRs cite its "ideal" section. Do not use it as a
description of the current code.

> **Conventions**
> - A solid arrow is a direct call or an import.
> - A dashed arrow is an asynchronous event or a snapshot that a bridge delivers.
> - A module name is its path under `src/`.

---

## 1. System Overview

```mermaid
graph TD
    subgraph External["External systems"]
        MI["MusicIndex API"]
        MIS["musicindex-search crate"]
        RSS["RSS feeds"]
        MB["MusicBrainz API"]
        FS["Filesystem<br/>audio files + tags"]
        MPV["mpv over IPC"]
        BC["Broadcast chain<br/>encoder · relay · publisher"]
    end

    subgraph Entry["Entry"]
        Main["main.rs<br/>user data move, then app or CLI"]
        Cli["cli.rs<br/>one-shot commands, no GPUI"]
        Boot["app/bootstrap.rs<br/>run_app · prepare_normal · mount_normal"]
    end

    subgraph Screens["Screens and shell"]
        Startup["app/startup.rs<br/>StartupScreen + recovery pages"]
        TopApp["app.rs + app/<br/>TopApp: sidebar, toolbar,<br/>search, Show, Settings"]
        Library["library.rs + library/<br/>LibraryApp + LibraryColumnView"]
    end

    subgraph DS["UI design system - ui/"]
        Shells["shells/<br/>workspace · release_page · track_page ·<br/>show · search_results_inspector · library/*"]
        Composites["composites/<br/>ListRow · TrackRow · SelectableTile ·<br/>SelectionActionBar · SkeletonInspector"]
        Primitives["primitives/<br/>Button · Label · Surface · Stack ·<br/>Skeleton · FitProbe"]
        Tokens["tokens.rs · style.rs · layouts.rs<br/>assets/tokens.json"]
    end

    subgraph Pres["Presentation adapters - presentation/"]
        Present["present_command"]
        Bridge["bridge_watch<br/>GpuiEventBridge"]
        Host["RuntimeHost<br/>tokio runtime + VmBus"]
        Maint["MaintenanceWorker<br/>StartupBackend runner"]
    end

    subgraph VMs["View models - view_models/, no GPUI"]
        Nav["sidebar · workspace/<br/>SidebarVm · FrameNavigationEntry ·<br/>WorkspaceLayout"]
        PageVm["release_page · track_page ·<br/>search_results/ · show/ · startup/"]
        Paged["paged_playlist_detail"]
    end

    subgraph AppL["Application - application/"]
        Runner["AsyncCommandRunner"]
        Bus["CommandBus<br/>commands/ · queries/"]
        Events["ApplicationEventBus<br/>events/"]
        Life["SessionLifecycle"]
        Ports["ports/download_manager<br/>ConversionRecovery"]
        PagedActor["PagedTrackListActor"]
    end

    subgraph RT["Runtime - runtime/"]
        VmBus["VmBus<br/>VmEvent"]
        PagedVm["PagedListVm"]
        Handles["playback_polling · tag_update ·<br/>musicbrainz_feed_saga · playlist_rss_check ·<br/>saved_search_counts · broadcast_*"]
    end

    subgraph Svc["Domain services, no GPUI"]
        Services["feed · library · playlist ·<br/>subscribe · metadata services"]
        Identity["track_identity · identity_ingest ·<br/>local_identity · track_compare"]
        Playback["playback · playback_owner"]
    end

    subgraph Infra["Infrastructure"]
        DB["db.rs + db/<br/>SQLite, migrations, maintenance"]
        Api["api.rs · http_client.rs"]
        RssM["rss/"]
        MbM["musicbrainz.rs"]
        Tags["audio_tags.rs · audio_format"]
        Driver["playback_driver/"]
        Media["media/ · remote_media.rs"]
        Broadcast["broadcast/"]
        Cfg["config.rs + config/"]
    end

    Main --> Boot & Cli
    Boot --> Maint
    Boot --> Startup --> TopApp
    TopApp --> Library
    TopApp & Library --> Shells --> Composites --> Primitives --> Tokens
    TopApp & Library --> VMs
    TopApp & Library --> Present --> Runner --> Bus
    Bus --> Services & Identity
    Cli --> Bus
    Runner -. events .-> Events -. drained in render .-> Bridge
    Runner -. invalidations .-> VmBus
    Host --> VmBus
    Handles -. watch snapshots .-> Bridge
    Paged --> PagedVm <-->|page requests + deliveries| PagedActor
    PagedActor --> DB
    Bus --> Ports --> Services
    Services --> DB & Api & RssM & Tags & MbM
    Playback --> Driver --> MPV
    Api --> MI
    VMs --> MIS
    RssM --> RSS
    MbM --> MB
    Tags --> FS
    Media --> FS
    Broadcast --> BC
```

Important points:

- `main.rs` first moves the user data of v4vmm (ADR 0096 Decision 6). With no argument it starts the app. With an
  argument it runs the CLI, which uses `CommandBus` and no GPUI.
- The app opens on `StartupScreen`. `TopApp` mounts only after the configuration, the music folder and the
  database pass their checks (section 7).
- `TopApp` is the one root entity. `LibraryApp` is a column model in it, not a different tab. There is no
  `SearchApp` and no Discover screen.
- A screen sends a command through `present_command`. The command runs on a blocking thread of the tokio
  runtime, and the result returns to the GPUI thread.

---

## 2. Layer Order And The Edges That Go Against It

The layer order runs from the top to the bottom of this diagram. The red edges go against the order. They are
in the code at `86d97e7`.

```mermaid
graph TD
    Screens["Screens and shell<br/>app/ · library/"]
    UI["UI design system<br/>ui/"]
    Pres["Presentation adapters<br/>presentation/"]
    VM["View models<br/>view_models/"]
    App["Application<br/>application/"]
    RT["Runtime<br/>runtime/"]
    Svc["Domain services"]
    Infra["Infrastructure<br/>db · api · rss · config · media"]

    Screens --> UI & Pres & VM & App & RT
    Pres --> App & RT & VM & UI
    VM --> App & RT
    App --> RT & Svc & Infra
    RT --> Infra
    Svc --> Infra

    Screens -->|"direct service and db calls"| Svc
    Library2["library/"] -->|"app::keyboard"| Screens
    RT -->|"tag_update · broadcast_service_watch"| App
    RT -->|"saved_search_counts"| VM
    App -->|"queries/search · capability_recovery"| VM
    VM -->|"mod · startup · library_search_facts"| Infra
    Infra -->|"db/maintenance/upgrade · config.rs"| VM
    Infra -->|"api.rs · rss/ · media/"| App

    linkStyle 16,17,18,19,20,21,22,23 stroke:#c0392b,stroke-width:2px
```

At `86d97e7`, `tests/architecture_tests.rs` enforces these rules:

| Test | Rule |
|---|---|
| `view_models_do_not_import_gpui_or_screen_layers` | No GPUI, `ui`, `app` or `library` import in `view_models/` |
| `application_layer_does_not_import_gpui_or_screen_layers` | No GPUI or screen import in `application/` |
| `runtime_layer_does_not_import_gpui_or_ui` | No GPUI, `ui` or `presentation` import in `runtime/` |
| `shared_ui_components_do_not_import_backend_or_screen_layers` | Primitives and composites do not import the application, the services, the database or a screen |
| `shared_top_level_ui_shells_do_not_import_screen_modules` | A shared shell does not import a screen |
| `cx_spawn_is_restricted_to_presentation_runtime_and_bootstrap` | Only these modules start GPUI tasks |
| `gpui_command_runner_is_retired`, `async_runtime_feature_flag_is_retired` | The `GpuiCommandRunner` and the `async-runtime` Cargo feature stay removed |

No test forbids the red edges. Inference: the edge from runtime to application and the edge from application to
runtime make a cycle (`application/async_command_runner.rs:36`, `runtime/tag_update.rs:20`). The screens in
`library/` and `app/search_dispatch.rs` call services and the database directly. Inference: this code is from
the time before the command bus.

---

## 3. The Shell

```mermaid
graph TD
    Root["gpui_component::Root"]
    Startup["StartupScreen<br/>app/startup.rs"]
    Top["TopApp<br/>app.rs:143"]

    Root --> Startup -->|"mount_normal"| Top

    Top --> Form{"shell_form(width)"}
    Form -->|"wide"| SidebarShell["render_sidebar_shell<br/>SidebarVm"]
    Form -->|"narrow"| TabBar["render_narrow_tab_bar<br/>app/tab_bar.rs"]

    subgraph Dest["SidebarDestination - view_models/sidebar.rs"]
        D1["Search · Releases · Artists ·<br/>My playlists · Public playlists · Live"]
        D2["Show"]
        D3["Inbox"]
        D4["Settings"]
    end

    SidebarShell --> Dest
    D1 & D3 -->|"MusicDestination<br/>AppTab::Music"| Music
    D2 -->|"AppTab::Show"| Show["build_show_screen<br/>ui/shells/show.rs · ShowPageVm<br/>queue and now playing"]
    D4 -->|"AppTab::Settings"| Settings["render_settings<br/>SettingsVm"]

    subgraph Music["Music workspace - render_workspace_content"]
        Toolbar["render_toolbar<br/>search field + field list"]
        List["List column<br/>LibraryColumnView or search results<br/>list bar: ListBarVm + SelectionActionBar"]
        Detail["Detail frame<br/>release · track · artist · publisher ·<br/>playlist · singles group · window group"]
    end

    List -->|"row click<br/>FrameNavigationEntry + ListRow"| Detail
    Top --> Layers["render_window_layers<br/>menus, popovers, confirmations"]
```

- A page opens over a list. `FrameNavigationEntry` (`view_models/workspace/nav.rs:70`) names each page:
  `Destination`, `SavedSearch`, `PlaylistDetail`, `PublicPlaylist`, `Release`, `Track`, `ArtistName`,
  `PublisherDetail`, `SinglesGroup`, `WindowGroup`, `Search`, `Settings` and `QueueNowPlaying`.
- `NavigationPathStart` tells if a page starts a new path (`NewPath`), stays in the path (`CurrentPath`) or
  opens from a row (`ListRow`).
- A release page has one of two sources. `ReleasePageSource::Library` reads the database.
  `ReleasePageSource::Index` reads MusicIndex. The two sources use one page shell, `ui/shells/release_page.rs`.
- The window is 390 px wide at minimum (`ui/layouts.rs`). `app/fit_check.rs` checks the shell and the list bar
  at 390 px and 940 px (ADR 0097).

---

## 4. The UI Design System

```mermaid
graph TD
    subgraph Tokens["Tokens - ui/tokens.rs · style.rs · assets/tokens.json"]
        T["SemanticColor · Spacing · Radius ·<br/>FontSize · Weight · Size · Motion · ScaleFactor"]
    end
    subgraph Prims["Primitives - ui/primitives/"]
        P["Button · Checkbox · ContextMenu · Divider · Image ·<br/>Label · MultilineText · Popover · SectionHeader ·<br/>Skeleton · Stack · Surface · Tooltip · PrimaryInput · FitProbe"]
    end
    subgraph Comps["Composites - ui/composites/"]
        C["ListRow · TrackRow · SelectableTile · Thumbnail ·<br/>DetailHeader · DisclosureGroup · SegmentedControl ·<br/>SelectionActionBar · SkeletonInspector · SplitPane ·<br/>ConfirmationDialog · SearchFieldList · LiveStatusStrip"]
    end
    subgraph Shells["Shells - ui/shells/"]
        S["workspace · release_page · track_page · publisher ·<br/>name_level_page · playlist · show · queue_now_playing ·<br/>search_results_inspector · window_layers · confirmations"]
        SL["library/*<br/>screen-owned shells for LibraryApp"]
    end
    Screens["Screens<br/>TopApp · LibraryApp · StartupScreen"]

    Tokens --> Prims --> Comps --> Shells --> Screens
    SL -. "may import library/" .-> Screens
```

The shells in `ui/shells/library/` belong to the Library screen, and the module docs in `ui/shells/mod.rs`
permit their import of `library/`. A shared shell does not import a screen.

---

## 5. The Async Model

```mermaid
graph LR
    subgraph GPUI["GPUI thread"]
        Screen["TopApp / LibraryApp"]
        PC["present_command"]
        BW["bridge_watch"]
        GEB["GpuiEventBridge<br/>drained in render"]
    end

    subgraph Tokio["tokio runtime - RuntimeHost"]
        ACR["AsyncCommandRunner.dispatch<br/>admits through SessionLifecycle"]
        SB["spawn_blocking"]
        CB["CommandBus.execute"]
        AEB["ApplicationEventBus"]
        VB["VmBus<br/>TrackChanged · FeedChanged ·<br/>PlaylistChanged · InvalidateAll"]
        Actors["PagedTrackListActor ·<br/>runtime handles"]
    end

    Screen --> PC --> ACR --> SB --> CB
    CB -->|"oneshot result"| PC
    PC -->|"on success / on error"| Screen
    ACR -. "application events" .-> AEB -. subscribe .-> GEB --> Screen
    ACR -. invalidations .-> VB -. "VmEvent" .-> Actors
    Actors -. "watch snapshots" .-> BW --> Screen
```

- `RuntimeHost` (`presentation/runtime_host.rs`) owns a multi-thread tokio runtime and the `VmBus`. If its start fails, it is
  not there. Then the capability `BackgroundRuntime` fails, and `AsyncCommandRunner::unavailable` answers each
  command.
- No screen reads `VmEvent`. The actors and the runtime handles read it, and they send snapshots through
  `bridge_watch`.
- `SessionLifecycle` counts the admitted work. A session stops only after all of its work stops.
- The `async-runtime` Cargo feature is gone. tokio is a usual dependency.

---

## 6. Sequence - A Search From The Toolbar

```mermaid
sequenceDiagram
    autonumber
    participant U as Person
    participant T as TopApp
    participant Q as ApplicationQueryService
    participant DB as db/library_search
    participant R as AsyncCommandRunner
    participant CB as CommandBus
    participant API as MusicIndex API
    participant V as search results shell

    U->>T: type text or press Enter
    T->>T: submit_global_search -> SearchRun::new
    T->>T: show_toolbar_search_run
    T->>Q: search_local_library_tracks
    Q->>DB: search_library_tracks
    DB-->>T: Library rows
    T->>V: render the Library rows
    T->>R: dispatch_index_search via present_command
    R->>CB: FetchIndexSearchResults
    CB->>API: search feeds, tracks and publishers
    API-->>CB: Index rows
    CB-->>T: rows for this run generation
    T->>T: replace_index_results_for_run drops a stale run
    T->>V: render the Index rows
```

The `musicindex-search` crate parses the text and builds the `kind:` requests, `q` and the sort parameters
(ADR 0091). HitIt filters its Library rows with `host_keeps_row`.

---

## 7. Sequence - Open A Release Page From A List

```mermaid
sequenceDiagram
    autonumber
    participant U as Person
    participant L as LibraryApp
    participant T as TopApp
    participant W as WorkspaceLayout
    participant R as AsyncCommandRunner
    participant API as MusicIndex API
    participant S as release_page shell

    U->>L: click a release tile
    L->>L: click_release_row -> open_page(Release, ListRow)
    L-->>T: LibraryAppEvent::OpenPage
    T->>W: open_nav_list_row
    T->>T: resolve_release_page_source
    alt the release is in the Library
        T->>L: select_album(feed_id)
        L->>S: render from the database
    else the release is only in the Index
        T->>S: render SkeletonInspector
        T->>R: FetchIndexFeedDetail via present_command
        R->>API: fetch the feed detail
        API-->>T: IndexFeedDetailState::Loaded
        T->>S: render_index_release_page
    end
```

---

## 8. Sequence - Take A Release Into The Library

```mermaid
sequenceDiagram
    autonumber
    participant U as Person
    participant T as TopApp
    participant L as LibraryApp download queue
    participant R as AsyncCommandRunner
    participant DM as ServiceDownloadManager
    participant SS as subscribe_service
    participant RSS as RSS feed
    participant API as MusicIndex API
    participant FS as Filesystem
    participant DB as SQLite

    U->>T: click Download on a release page
    T->>L: download_index_feed -> queue_downloads
    L->>L: pump_downloads, at most 3 jobs, wait for the host turn
    L->>R: SubscribeFeed via present_command
    R->>DM: subscribe_feed
    DM->>SS: ConversionRecovery -> subscribe_feed_retaining
    SS->>RSS: fetch_feed_document
    SS->>DB: subscribe the feed, persist the MusicIndex context
    loop each track
        SS->>API: fetch_track
        SS->>FS: download_track, convert, write ID3v2.4 tags, promote
        SS->>DB: mark_track_downloaded
    end
    SS-->>T: IndexDownloadFinished
    T->>T: scan_tag_updates, show the page from the Library source in place
    R-->>R: ApplicationEventBus events + VmEvent::InvalidateAll
```

If no track downloads, the feed returns to "not subscribed" and the page shows an error.

---

## 9. Sequence - App Start, Database Check And Recovery

```mermaid
sequenceDiagram
    autonumber
    participant M as main
    participant B as bootstrap
    participant S as StartupScreen
    participant W as MaintenanceWorker
    participant SB as StartupBackend
    participant DB as SQLite
    participant T as TopApp

    M->>M: move_previous_user_data
    M->>B: run_app
    B->>W: start the hitit-maintenance thread
    B->>S: open the window with StartupScreen
    S->>W: request CheckAgain, intent Initial
    W->>SB: execute
    SB->>SB: load the configuration, check the fields
    SB->>SB: storage::check_music probes the music folder
    SB->>DB: db::startup::check_database
    alt Ready or NeedsPreparation
        SB->>DB: prepare_database: backup, then migrate
        SB-->>B: CoreResult::Prepared
        B->>B: prepare_normal: paths, playback, RuntimeHost, image cache
        B->>T: mount_normal -> TopApp::new
    else a check fails
        SB-->>S: CoreResult::Checked
        S->>S: show a RecoveryPage and the StartupReportVm
        Note over S: Check Again, Open App, Copy Report or Quit.<br/>ConfigurationEditor and DatabaseTools correct the fault.
    end
```

`StartupStage` names the step with the fault. "Check" does not prepare the database. "Open" prepares it again only
if the configuration bytes did not change.

---

## File Map

The [source map](source-map.md) gives the owner file of each area. This table gives the folders of each layer.

| Layer | Files |
|---|---|
| Entry | `src/main.rs`, `src/cli.rs`, `src/app/bootstrap.rs` |
| Startup and recovery | `src/startup.rs`, `src/startup/`, `src/app/startup.rs`, `src/presentation/{startup_presenter,maintenance_executor,configuration_editor,database_tools}.rs`, `src/db/startup.rs`, `src/db/maintenance/` |
| Screens and shell | `src/app.rs`, `src/app/`, `src/library.rs`, `src/library/` |
| UI design system | `src/ui/{tokens,style,layouts}.rs`, `src/ui/{primitives,composites,shells}/`, `src/assets/tokens.json` |
| Presentation adapters | `src/presentation/` |
| View models | `src/view_models/` |
| Application | `src/application/`, with `commands/`, `queries/`, `events/`, `errors/` and `ports/` |
| Runtime | `src/runtime/` |
| Domain services | `src/{feed,library,playlist,metadata}_service.rs`, `src/subscribe_service.rs`, `src/subscribe_service/`, `src/{track_compare,track_identity,identity_ingest,local_identity,local_metadata,playback,playback_owner}.rs` |
| Infrastructure | `src/db.rs`, `src/db/`, `src/{api,http_client,remote_media,musicbrainz,audio_tags,config}.rs`, `src/{rss,audio_format,playback_driver,media,broadcast,provider_observation,config}/` |
