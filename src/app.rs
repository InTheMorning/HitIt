#![warn(clippy::pedantic)]

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use gpui::{
    div, prelude::*, App, Context, Div, Entity, Focusable as _, Image, Render, Styled, Window,
};
use gpui_component::input::{InputEvent, InputState};
use rusqlite::Connection;

use crate::application::commands::download::RemoveCachedFiles;
use crate::application::queries::library::LoadCachedTracksTree;
use crate::application::{
    ApplicationEventSubscriber, ApplicationServices, AsyncCommandRunner, CommandContext,
};
use crate::config;
use crate::library::{
    DownloadStatusView, LibraryApp, LibraryAppEvent, LibraryColumn, LibraryColumnView,
};
use crate::media::ImageCache;
use crate::playback_driver::ConfiguredPlaybackDriver;
use crate::playback_owner::PlaybackOwner;
use crate::presentation::collection_art::{
    collection_art_image, CollectionArtHost, CollectionArtLoads, CollectionArtParts,
};
use crate::presentation::{bridge_watch, present_command, GpuiEventBridge};
use crate::runtime::playback_polling::{PlaybackPollingHandle, PlaybackTickOutcome};
use crate::runtime::{
    BroadcastReadinessSnapshot, BroadcastReadinessWatchHandle, BroadcastServiceWatchHandle,
    BroadcastServiceWatchPace, BroadcastServiceWatchSnapshot,
};
use crate::theme_profile::ThemeProfile;
use crate::ui::shells::search_results_inspector::{
    render_search_results_inspector, SearchResultsInspectorSlots,
};
use crate::ui::shells::window_layers::render_window_layers;
use crate::ui::shells::workspace::{render_workspace, WorkspaceSlots};
use crate::ui::tokens::{color, SemanticColor};
use crate::view_models::app_toolbar::AppToolbarVm;
use crate::view_models::cached_files::CachedFilesVm;
use crate::view_models::library::DescriptionState;
use crate::view_models::publisher_page::PublisherPageContext;
use crate::view_models::search_page::SearchPageAction;
use crate::view_models::search_results::{SearchResultsInspectorPageVm, SearchResultsTab};
use crate::view_models::settings::{SettingsAction, SettingsVm};
use crate::view_models::show::{EventSectionInput, ShowCommandState, ShowPageVm};
use crate::view_models::sidebar::{ShellForm, SidebarDestination};
use crate::view_models::workspace::{
    music_focus_moves, music_focus_target, music_page_content, FrameNavigationEntry,
    FrameNavigationState, IndexTrackTarget, MusicColumns, MusicDestination, MusicFocusTarget,
    MusicPageContent, NavigationPathStart, PendingToolbarSearchText, ReleasePageSource,
    TrackPageSource, WorkspaceFrameId, WorkspaceFrameKind, WorkspaceFrameState, WorkspaceLayout,
    WorkspaceLayoutConfig,
};

mod capabilities;
mod session;
mod settings;
mod startup;

mod bootstrap;
mod detail_selection;
mod events;
pub(crate) mod keyboard;
mod menu;
mod name_level_dispatch;
mod page_title;
mod playback_bar;
mod publisher_dispatch;
mod queue_now_playing;
mod resize;
mod saved_search_dispatch;
mod saved_search_query_edit;
mod search_dispatch;
mod search_field_list;
mod search_live;
mod search_selection;
mod show;
mod tab_bar;
mod toolbar;

pub use bootstrap::run_app;

use search_dispatch::{RemoteCoverColorState, RemoteDetailThumbnailState};
use search_field_list::{FieldValueRequest, SearchFieldListView};
use search_selection::{sort_handler, SearchPageSlot, SearchRowClick};
use settings::render_settings;
use show::{build_live_status_strip, build_show_screen};
use toolbar::{render_app_mark, render_toolbar};

const WORKSPACE_CONTENT_FRAME_ID: WorkspaceFrameId = WorkspaceFrameId::new(2);

// ---------------------------------------------------------------------------
// AppTab
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppTab {
    Music,
    Show,
    Settings,
}

/// ADR 0060 app-section mount boundary.
///
/// Music and Settings render through the workspace shell. Show is a
/// whole-screen mount because it does not use frame history or a page title.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorkspaceScreenMount {
    Music,
    Show,
    Settings,
}

impl From<AppTab> for WorkspaceScreenMount {
    fn from(tab: AppTab) -> Self {
        match tab {
            AppTab::Music => Self::Music,
            AppTab::Show => Self::Show,
            AppTab::Settings => Self::Settings,
        }
    }
}

// ---------------------------------------------------------------------------
// TopApp
// ---------------------------------------------------------------------------

impl CollectionArtHost for TopApp {
    fn collection_art_parts(&mut self) -> CollectionArtParts<'_> {
        CollectionArtParts {
            loads: &mut self.collection_art,
            cache: &self.image_cache,
            runner: &self.command_runner,
        }
    }
}

pub struct TopApp {
    tab: AppTab,
    library: Entity<LibraryApp>,
    /// The Music sidebar column view (ADR 0084 Decision 1). Its content is
    /// unchanged by this packet.
    library_sidebar_view: Entity<LibraryColumnView>,
    /// The Music list column view. It always shows the content list.
    library_list_view: Entity<LibraryColumnView>,
    /// The Music detail column view. It shows the open Library page, or the
    /// empty state, when the open page is not an Index, publisher, name
    /// match, or search page.
    library_detail_view: Entity<LibraryColumnView>,
    /// The download progress item (ADR 0084 Decision 10, packet 022). It
    /// shows on each section, beside the live status strip.
    download_status_view: Entity<DownloadStatusView>,
    global_search_input: Entity<InputState>,
    /// The field list under the toolbar search field (ADR 0087 Decision 3).
    search_field_list: Entity<SearchFieldListView>,
    /// The query that a restore of a search page writes into
    /// `global_search_input` at the next render (ADR 0084 task 030, D5).
    pending_toolbar_search: PendingToolbarSearchText,
    endpoint_input: Entity<InputState>,
    music_dir_input: Entity<InputState>,
    flac_path_input: Entity<InputState>,
    workspace_layout: WorkspaceLayout,
    last_music_content_nav: Option<FrameNavigationState>,
    search_results_detail: Option<SearchResultsInspectorPageVm>,
    /// The pause, the run generation and the Index request of the toolbar
    /// search page (ADR 0087 Decision 4, packet 003).
    toolbar_search_runs: search_live::SearchSlotRuns,
    /// The run generation and the Index request of the saved-search page
    /// (ADR 0087 Decision 4, packet 003). This slot arms no pause.
    saved_search_runs: search_live::SearchSlotRuns,
    /// The keyboard focus of the search list in the list column (ADR 0084
    /// packet 010). The key handlers act on the search selection only while
    /// the search list carries the focus.
    search_list_focus: gpui::FocusHandle,
    /// The keyboard target that the window focus last moved to, in the
    /// Music surface (ADR 0089 Invariants). `None` outside Music. The app
    /// moves the focus only when the target changes.
    music_focus_applied: Option<MusicFocusTarget>,
    /// The scroll of the search list in the list column (packet 028 D2). A
    /// keyboard focus move scrolls the focused row into view.
    search_list_scroll: gpui::ScrollHandle,
    /// The mounted saved-search page of the detail column (ADR 0084
    /// Decision 3, packet 012). It is separate from the toolbar search, so
    /// a toolbar search does not change it. Back and Forward to the same
    /// saved search keep it.
    saved_search_page: Option<crate::view_models::search_results::SavedSearchPage>,
    /// The keyboard focus of the saved-search page (packet 012).
    saved_search_focus: gpui::FocusHandle,
    /// The scroll of the saved-search page (packet 028 D2).
    saved_search_scroll: gpui::ScrollHandle,
    queue_text_filter: Option<String>,
    show_page: ShowPageVm,
    show_commands: ShowCommandState,
    broadcast: config::BroadcastCapabilities,
    music_dir: PathBuf,
    musicindex_endpoint: config::MusicIndexEndpoint,
    content_pane_width: gpui::Pixels,
    is_content_pane_resizing: bool,
    ui_scale: crate::config::UiScale,
    theme_profile: ThemeProfile,
    cfg_path: PathBuf,
    settings_status: String,
    settings: SettingsVm,
    settings_scroll: crate::ui::composites::settings::SettingsScrollHandles,
    _global_search_sub: gpui::Subscription,
    /// The query field of the saved-search page (ADR 0089 Decision 4). It
    /// is not the toolbar field.
    saved_search_query_input: Entity<InputState>,
    _saved_search_query_sub: gpui::Subscription,
    _library_sub: gpui::Subscription,
    _field_value_sub: gpui::Subscription,
    _appearance_sub: gpui::Subscription,
    playback_owner: Option<Arc<Mutex<PlaybackOwner<ConfiguredPlaybackDriver>>>>,
    playback_polling: Option<PlaybackPollingHandle>,
    broadcast_readiness_watch: Option<BroadcastReadinessWatchHandle>,
    broadcast_readiness_snapshot: Option<BroadcastReadinessSnapshot>,
    publisher_service_watch: Option<BroadcastServiceWatchHandle>,
    publisher_watch_generation: u64,
    publisher_service_snapshot: Option<BroadcastServiceWatchSnapshot>,
    /// The pace last sent to the service watch, so a repeat is never sent (ADR 0059).
    broadcast_service_watch_pace: BroadcastServiceWatchPace,
    show_log_resize: Option<(f32, f32)>,
    event_section_input: Option<EventSectionInput>,
    event_session: show::EventSession,
    conn: Arc<Mutex<Connection>>,
    image_cache: Arc<ImageCache>,
    remote_detail_thumbnails: BTreeMap<String, RemoteDetailThumbnailState>,
    remote_cover_colors: BTreeMap<String, RemoteCoverColorState>,
    /// The load state of each collection art (ADR 0084 Decision 12).
    collection_art: CollectionArtLoads,
    cached_files: CachedFilesVm,
    application_services: Arc<ApplicationServices>,
    command_runner: AsyncCommandRunner,
    application_event_bridge: Arc<GpuiEventBridge>,
    runtime_host: Option<Arc<crate::presentation::RuntimeHost>>,
    capability_vm: crate::view_models::startup::capabilities::CapabilityReportVm,
    capability_observations: crate::application::capability::CapabilityObservations,
    maintenance_worker: Option<crate::presentation::maintenance_executor::MaintenanceClient>,
    session_callback: Option<crate::ui::composites::maintenance_forms::SessionCallback>,
    previous_session_report: String,
    log_frames: crate::ui::composites::log_frame::LogFrames,
    configuration_editor_subscription: Option<gpui::Subscription>,
    database_tools: Option<Entity<crate::presentation::database_tools::DatabaseTools>>,
    database_tools_subscription: Option<gpui::Subscription>,
    configuration_editor:
        Option<Entity<crate::presentation::configuration_editor::ConfigurationEditor>>,
    /// The mounted publisher page (ADR 0077 packet 004). `None` when the
    /// current frame is not a publisher page.
    publisher_page: Option<publisher_dispatch::PublisherPageState>,
    /// The route (Library or Index) each visited publisher feed GUID last
    /// opened with (ADR 0077 packet 004). A history restore
    /// of an older `PublisherDetail` entry reads this to fetch with the
    /// same query it originally used.
    publisher_page_routes: BTreeMap<String, PublisherPageContext>,
    /// The mounted name-match track page (ADR 0077 packet 006). `None` when
    /// the current frame is not a name-match page.
    name_level_page: Option<name_level_dispatch::NameLevelPageState>,
    /// The Index feed detail-on-open fetch, reached from an active search
    /// (ADR 0075 packet 047). `None` when no such fetch is tracked.
    index_feed_detail_state: Option<search_dispatch::IndexFeedDetailState>,
    /// The Index track detail-on-open fetch (ADR 0075 packet 047). `None`
    /// when no such fetch is tracked: the operator has not opened a track
    /// row, or opened one whose full detail already came from the
    /// name-match page's own fetch.
    index_track_detail_state: Option<search_dispatch::IndexTrackDetailState>,
    /// The feed read of the open Index track page, keyed by the feed GUID
    /// (ADR 0084 packet 023). The track page shows the feed identity
    /// section and the "Feed publication date" row from it. `None` when
    /// no read is tracked.
    index_track_feed_state: Option<search_dispatch::IndexFeedDetailState>,
    /// The data source of the open release page (ADR 0084 Decision 4).
    /// `None` when the current page is not a release. The top app resolves
    /// and stores this when a release opens or restores, and after a
    /// download; the render path reads it and makes no lookup of its own.
    release_page_source: Option<ReleasePageSource>,
    /// The label an opening row gave a release, by feed GUID (ADR 0084
    /// Decision 4). The toolbar title and the Index loading page show it until
    /// the release's own title loads.
    release_page_labels: BTreeMap<String, String>,
    /// The data source of the open track page (ADR 0084 Decision 4). `None`
    /// when the current page is not a track. The top app resolves and
    /// stores this when a track opens or restores, and after a download;
    /// the render path reads it and makes no lookup of its own.
    track_page_source: Option<TrackPageSource>,
    /// The label an opening row gave a track, by its Index activation
    /// target (ADR 0084 Decision 4). The toolbar title and the Index loading
    /// page show it until the track's own title loads.
    track_page_labels: BTreeMap<IndexTrackTarget, String>,
    /// The disclosure state of an Index release's description, by feed
    /// GUID (ADR 0084 Decision 4, ADR 0047 item 8 amended 2026-10-16). The
    /// state lives for the session only; it is not stored.
    index_release_description_states: BTreeMap<String, DescriptionState>,
    /// The disclosure state of an Index track's description, by its Index
    /// activation target (ADR 0084 Decision 4, ADR 0047 item 8 amended
    /// 2026-10-16). Same rule as the release state above.
    index_track_description_states: BTreeMap<IndexTrackTarget, DescriptionState>,
}

impl TopApp {
    #[expect(
        clippy::too_many_arguments,
        reason = "top-level app bootstrap still wires shared state explicitly"
    )]
    #[expect(
        clippy::needless_pass_by_value,
        reason = "bootstrap takes ownership of shared app resources before distributing them to child entities"
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "top-level app bootstrap still owns one-time child entity and service wiring"
    )]
    fn new(
        conn: Arc<Mutex<Connection>>,
        image_cache: Arc<ImageCache>,
        cfg_path: PathBuf,
        musicindex_endpoint: config::MusicIndexEndpoint,
        music_dir: PathBuf,
        flac_path: config::ConfigField<Option<PathBuf>>,
        broadcast: config::BroadcastCapabilities,
        workspace_layout_config: Option<WorkspaceLayoutConfig>,
        workspace_layout_prefs: Option<&config::WorkspaceLayoutPrefs>,
        ui_scale: crate::config::UiScale,
        theme_profile: ThemeProfile,
        playback_owner: Option<Arc<Mutex<PlaybackOwner<ConfiguredPlaybackDriver>>>>,
        runtime_host: Option<Arc<crate::presentation::RuntimeHost>>,
        session: crate::application::session_lifecycle::SessionLifecycle,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        crate::ui::theme_bridge::install_theme_for_window(
            theme_profile,
            ui_scale.into(),
            window,
            cx,
        );
        let library_cache = Arc::clone(&image_cache);
        let application_services = Arc::new(
            ApplicationServices::local_with_service_adapters()
                .expect("application services are fully wired"),
        );
        let application_event_bridge = Arc::new(GpuiEventBridge::new());
        let application_event_subscriber: Arc<dyn ApplicationEventSubscriber> =
            application_event_bridge.clone();
        application_services
            .event_bus()
            .subscribe(application_event_subscriber);
        let command_runner = match runtime_host.as_ref() {
            Some(host) => AsyncCommandRunner::with_vm_bus_on_handle(
                application_services.command_bus(),
                application_services.event_bus(),
                host.bus().clone(),
                host.handle().clone(),
            ),
            None => AsyncCommandRunner::unavailable(
                application_services.command_bus(),
                application_services.event_bus(),
                crate::application::capability::ExecutionUnavailable::RUNTIME,
            ),
        }
        .with_session(session.clone());
        let library_services = Arc::clone(&application_services);
        let library_runtime_host = runtime_host.clone();
        let global_search_display = AppToolbarVm::new().display().global_search;
        let global_search_input = cx.new(|cx: &mut Context<InputState>| {
            InputState::new(window, cx).placeholder(global_search_display.placeholder)
        });
        let global_search_sub =
            cx.subscribe_in(&global_search_input, window, Self::on_global_search_event);
        let saved_search_query_input = cx.new(|cx: &mut Context<InputState>| {
            InputState::new(window, cx)
                .placeholder(crate::view_models::search_results::SAVED_SEARCH_QUERY_PLACEHOLDER)
        });
        let saved_search_query_sub = cx.subscribe_in(
            &saved_search_query_input,
            window,
            Self::on_saved_search_query_event,
        );
        let library = cx.new(|cx| {
            let mut library = LibraryApp::new(
                conn.clone(),
                library_cache,
                musicindex_endpoint.clone(),
                music_dir.clone(),
                library_services,
                library_runtime_host.clone(),
                window,
                cx,
            );
            library.bind_session(session.clone());
            library.playback_availability =
                crate::application::capability::FeatureAvailability::from_resources(
                    &musicindex_endpoint,
                    &broadcast,
                    playback_owner.is_some(),
                )
                .with_runtime(command_runner.availability())
                .require(crate::application::capability::Dependency::Playback);
            library
        });
        let search_field_list = cx.new(|cx| {
            SearchFieldListView::new(global_search_input.clone(), library.clone(), window, cx)
        });
        // ADR 0087 Decision 3, packet 005: the field list asks for the
        // MusicIndex names of an `artist:` value, and the top app sends the
        // request through the command runner.
        let field_value_sub = cx.subscribe(
            &search_field_list,
            |this: &mut Self, _, request: &FieldValueRequest, cx| {
                this.dispatch_field_value_request(request, cx);
            },
        );
        // ADR 0084 Decision 1: one view for each Music column, over the one
        // Library entity. Each observes the Library entity, so a Library
        // notify renders its column again.
        let library_sidebar_view =
            cx.new(|cx| LibraryColumnView::new(library.clone(), LibraryColumn::Sidebar, cx));
        let library_list_view =
            cx.new(|cx| LibraryColumnView::new(library.clone(), LibraryColumn::List, cx));
        let library_detail_view =
            cx.new(|cx| LibraryColumnView::new(library.clone(), LibraryColumn::Detail, cx));
        let download_status_view = cx.new(|cx| DownloadStatusView::new(library.clone(), cx));
        let library_sub = cx.subscribe_in(
            &library,
            window,
            move |this: &mut Self, library, event: &LibraryAppEvent, window, cx| match event {
                LibraryAppEvent::PlayPlaylistAt {
                    track_id,
                    playlist_id,
                    playlist_position,
                } => {
                    this.play_playlist_at(*playlist_id, *playlist_position, *track_id, cx);
                    // ADR 0076 Decision 2 and its phase plan trigger mapping:
                    // playback from a playlist also starts the RSS check of
                    // that playlist. Playback does not wait for the check.
                    library.update(cx, |library, cx| {
                        library.check_playlist_rss(
                            *playlist_id,
                            crate::runtime::RssCheckTrigger::PlaybackStart,
                            cx,
                        );
                    });
                }
                LibraryAppEvent::OpenIndexFeedDetail { feed_guid, label } => {
                    this.open_index_feed_detail_from_music(feed_guid, label.clone(), cx);
                }
                LibraryAppEvent::OpenPublisherPage {
                    publisher_feed_guid,
                } => {
                    this.open_publisher_page(
                        publisher_feed_guid.clone(),
                        PublisherPageContext::Library,
                        cx,
                    );
                }
                LibraryAppEvent::OpenIndexPublisherPage {
                    publisher_feed_guid,
                } => {
                    this.open_index_publisher_page_from_list_row(publisher_feed_guid.clone(), cx);
                }
                LibraryAppEvent::OpenAlbumPage { feed_id } => {
                    this.open_library_album_page(*feed_id, cx);
                }
                LibraryAppEvent::OpenPage { entry, start } => {
                    this.open_music_page(entry, *start, cx);
                }
                LibraryAppEvent::SearchPage(action) => {
                    this.handle_search_page_action(action, window, cx);
                }
                LibraryAppEvent::SidebarRowSelected(row) => {
                    this.open_sidebar_row(*row, window, cx);
                }
                // ADR 0084 Decision 10: an Index job of the download queue
                // completed. The open pages change in place.
                LibraryAppEvent::IndexDownloadFinished { job, outcome } => {
                    this.finish_index_download(job, outcome, cx);
                }
                LibraryAppEvent::InboxSetupAction(action) => {
                    this.capability_action(*action, window, cx);
                }
            },
        );
        let appearance_sub = cx.observe_window_appearance(window, |this, window, cx| {
            if this.theme_profile == ThemeProfile::System {
                crate::ui::theme_bridge::install_theme_for_window(
                    ThemeProfile::System,
                    this.ui_scale.into(),
                    window,
                    cx,
                );
                cx.notify();
            }
        });
        let endpoint_default = musicindex_endpoint.require().unwrap_or_default().to_owned();
        let endpoint_input = cx.new(|cx: &mut Context<InputState>| {
            InputState::new(window, cx)
                .placeholder("https://api.musicindex.org")
                .default_value(endpoint_default)
        });
        let music_dir_default = music_dir.display().to_string();
        let music_dir_input = cx.new(|cx: &mut Context<InputState>| {
            InputState::new(window, cx)
                .placeholder("~/V4Vmusic")
                .default_value(music_dir_default)
        });
        let flac_path_default = flac_path
            .as_ref()
            .ok()
            .and_then(Option::as_ref)
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let flac_path_input = cx.new(|cx: &mut Context<InputState>| {
            InputState::new(window, cx)
                .placeholder("flac (from $PATH)")
                .default_value(flac_path_default)
        });
        let show_page =
            ShowPageVm::idle().with_execution_availability(command_runner.availability());

        Self {
            tab: AppTab::Music,
            library,
            library_sidebar_view,
            library_list_view,
            library_detail_view,
            download_status_view,
            global_search_input,
            search_field_list,
            pending_toolbar_search: PendingToolbarSearchText::default(),
            endpoint_input,
            music_dir_input,
            flac_path_input,
            workspace_layout: Self::initial_workspace_layout(
                workspace_layout_config.as_ref(),
                WorkspaceScreenMount::Music,
            ),
            last_music_content_nav: None,
            search_results_detail: None,
            toolbar_search_runs: search_live::SearchSlotRuns::default(),
            saved_search_runs: search_live::SearchSlotRuns::default(),
            search_list_focus: cx.focus_handle(),
            music_focus_applied: None,
            search_list_scroll: gpui::ScrollHandle::new(),
            saved_search_page: None,
            saved_search_focus: cx.focus_handle(),
            saved_search_scroll: gpui::ScrollHandle::new(),
            queue_text_filter: None,
            show_page,
            show_commands: ShowCommandState::default(),
            broadcast,
            music_dir,
            musicindex_endpoint,
            content_pane_width: Self::initial_content_pane_width(workspace_layout_prefs),
            is_content_pane_resizing: false,
            ui_scale,
            theme_profile,
            cfg_path,
            settings_status: String::new(),
            settings: SettingsVm::default(),
            settings_scroll: crate::ui::composites::settings::SettingsScrollHandles::default(),
            _global_search_sub: global_search_sub,
            saved_search_query_input,
            _saved_search_query_sub: saved_search_query_sub,
            _library_sub: library_sub,
            _field_value_sub: field_value_sub,
            _appearance_sub: appearance_sub,
            playback_owner,
            playback_polling: None,
            broadcast_readiness_watch: None,
            broadcast_readiness_snapshot: None,
            publisher_service_watch: None,
            publisher_watch_generation: 0,
            publisher_service_snapshot: None,
            broadcast_service_watch_pace: BroadcastServiceWatchPace::Background,
            show_log_resize: None,
            event_section_input: None,
            event_session: show::EventSession::default(),
            conn,
            image_cache,
            remote_detail_thumbnails: BTreeMap::new(),
            remote_cover_colors: BTreeMap::new(),
            collection_art: CollectionArtLoads::default(),
            cached_files: CachedFilesVm::default(),
            application_services,
            command_runner,
            application_event_bridge,
            runtime_host,
            capability_vm: crate::view_models::startup::capabilities::CapabilityReportVm::new(
                BTreeMap::new(),
                false,
            ),
            capability_observations:
                crate::application::capability::CapabilityObservations::default(),
            maintenance_worker: None,
            session_callback: None,
            previous_session_report: String::new(),
            log_frames: crate::ui::composites::log_frame::LogFrames::default(),
            configuration_editor: None,
            configuration_editor_subscription: None,
            database_tools: None,
            database_tools_subscription: None,
            publisher_page: None,
            publisher_page_routes: BTreeMap::new(),
            name_level_page: None,
            index_feed_detail_state: None,
            index_track_detail_state: None,
            index_track_feed_state: None,
            release_page_source: None,
            release_page_labels: BTreeMap::new(),
            track_page_source: None,
            track_page_labels: BTreeMap::new(),
            index_release_description_states: BTreeMap::new(),
            index_track_description_states: BTreeMap::new(),
        }
    }

    fn maybe_start_playback_polling(&mut self, cx: &mut Context<Self>) {
        if self.playback_polling.is_some() {
            return;
        }
        let Some(host) = self.runtime_host.clone() else {
            return;
        };
        let Some(owner) = self.playback_owner.clone() else {
            return;
        };
        if !owner
            .lock()
            .expect("lock playback owner")
            .driver()
            .is_live_driver()
        {
            return;
        }
        let _enter = host.handle().enter();
        let handle = crate::runtime::playback_polling::spawn(
            owner,
            Arc::clone(&self.conn),
            self.command_runner.session(),
        );
        bridge_watch(
            handle.subscribe(),
            |this: &mut Self, snapshot, cx| {
                this.apply_playback_tick(snapshot.outcome, cx);
            },
            cx,
        );
        self.playback_polling = Some(handle);
    }

    fn on_global_search_event(
        &mut self,
        _entity: &Entity<InputState>,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // ADR 0087 Decision 4: a change of the text arms the pause, and
        // Enter runs the search at once.
        match event {
            InputEvent::Change => self.start_search_pause(cx),
            InputEvent::PressEnter { .. } => self.submit_global_search(window, cx),
            InputEvent::Focus | InputEvent::Blur => {}
        }
    }

    /// Gives the keyboard focus to the sidebar row of Show or Settings (ADR
    /// 0089 Decision 1), so a section shortcut reaches its row. A Music
    /// shortcut keeps the focus rule of `apply_music_focus`: the list page
    /// takes the focus when the frame changes, so this does nothing for
    /// Music.
    pub(super) fn focus_active_tab(&self, window: &mut Window, cx: &mut Context<Self>) {
        let row = match self.tab {
            AppTab::Music => return,
            AppTab::Show => SidebarDestination::Show,
            AppTab::Settings => SidebarDestination::Settings,
        };
        self.library
            .update(cx, |library, cx| library.focus_sidebar_row(row, window, cx));
    }

    /// Renders the sidebar column of the sidebar form (ADR 0089 Decision 1).
    /// It holds the application mark and the sidebar rows.
    fn render_sidebar_shell(&self, cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .w(crate::ui::layouts::MUSIC_SIDEBAR_WIDTH)
            .h_full()
            .min_h_0()
            .overflow_hidden()
            .bg(crate::ui::style::color::bg_sidebar())
            .child(render_app_mark(cx))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .child(self.library_sidebar_view.clone()),
            )
    }

    /// Routes a click on a sidebar row (ADR 0089 Decision 1). Show and
    /// Settings select their mount. A Music row selects the Music mount
    /// first, when another mount shows, and then opens its destination.
    fn open_sidebar_row(
        &mut self,
        row: SidebarDestination,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match row {
            SidebarDestination::Show => self.select_tab(AppTab::Show, window, cx),
            SidebarDestination::Settings => self.select_tab(AppTab::Settings, window, cx),
            music_row => {
                if self.tab != AppTab::Music {
                    self.select_tab(AppTab::Music, window, cx);
                }
                if let Some(destination) = music_row.music_destination() {
                    self.library.update(cx, |library, cx| {
                        library.open_page(
                            FrameNavigationEntry::Destination(destination),
                            NavigationPathStart::NewPath,
                            cx,
                        );
                    });
                    // ADR 0089 Decision 4: a click on Search puts the focus in the
                    // toolbar field.
                    if destination == MusicDestination::Search {
                        self.focus_global_search(window, cx);
                    }
                }
            }
        }
    }

    /// Applies a request of the Search page to the toolbar field (ADR 0089
    /// Decision 4). The field keeps the focus after each request.
    fn handle_search_page_action(
        &mut self,
        action: &SearchPageAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match action {
            SearchPageAction::NewSavedSearch => {
                self.global_search_input
                    .update(cx, |input, cx| input.set_value("", window, cx));
                self.focus_global_search(window, cx);
            }
            SearchPageAction::RunQuery(query) => {
                let text = query.clone();
                self.global_search_input
                    .update(cx, |input, cx| input.set_value(text, window, cx));
                self.open_search_results_in_content_list(query, window, cx);
            }
            SearchPageAction::DeleteSavedSearch(search) => {
                Self::request_saved_search_delete_for(search, window, cx);
            }
            SearchPageAction::EditSavedSearchQuery(search) => {
                self.open_saved_search_query_edit(search, window, cx);
            }
            SearchPageAction::DuplicateSavedSearch(id) => {
                let copy = self.library.update(cx, |library, cx| {
                    library.duplicate_saved_search_row(*id, cx)
                });
                let search = copy.and_then(|copy| {
                    self.library
                        .read(cx)
                        .saved_searches()
                        .iter()
                        .find(|search| search.id == copy)
                        .cloned()
                });
                if let Some(search) = search {
                    self.open_saved_search_query_edit(&search, window, cx);
                }
            }
        }
    }

    /// The sidebar row of the current screen (ADR 0089 Decision 1).
    fn sidebar_row_for_mount(&self, mount: WorkspaceScreenMount, cx: &App) -> SidebarDestination {
        match mount {
            WorkspaceScreenMount::Music => {
                SidebarDestination::for_music(self.library.read(cx).active_destination())
            }
            WorkspaceScreenMount::Show => SidebarDestination::Show,
            WorkspaceScreenMount::Settings => SidebarDestination::Settings,
        }
    }

    fn select_tab(&mut self, tab: AppTab, window: &mut Window, cx: &mut Context<Self>) {
        self.tab = tab;
        self.focus_active_tab(window, cx);
        self.sync_broadcast_service_watch_pace();

        if matches!(tab, AppTab::Show) {
            self.queue_text_filter = None;
            self.invalidate_broadcast_readiness_snapshot();
            self.refresh_show_page(cx);
            cx.notify();
            return;
        }

        let Some(content_list_id) = self.content_list_frame_id() else {
            cx.notify();
            return;
        };

        match tab {
            AppTab::Settings => {
                self.settings.dispatch(SettingsAction::Open);
                self.cached_files.enter();
                self.start_cached_load(cx);
                if let Some(nav) = self.workspace_layout.frame_nav(content_list_id).cloned() {
                    if !matches!(nav.current(), FrameNavigationEntry::Settings) {
                        self.last_music_content_nav = Some(nav);
                    }
                }
                if let Err(error) = self
                    .workspace_layout
                    .reset_nav(content_list_id, FrameNavigationEntry::Settings)
                {
                    self.settings_status = format!("Error switching tabs: {error}");
                }
                self.sync_search_results_detail_with_nav(content_list_id, cx);
            }
            AppTab::Music => {
                let restoring_from_settings = self
                    .workspace_layout
                    .frame_nav(content_list_id)
                    .is_some_and(|nav| matches!(nav.current(), FrameNavigationEntry::Settings));
                if restoring_from_settings {
                    let nav = self.last_music_content_nav.clone().unwrap_or_else(|| {
                        FrameNavigationState::new(FrameNavigationEntry::Destination(
                            MusicDestination::Search,
                        ))
                    });
                    if let Err(error) = self.workspace_layout.replace_nav(content_list_id, nav) {
                        self.settings_status = format!("Error switching tabs: {error}");
                    }
                }

                self.sync_search_results_detail_with_nav(content_list_id, cx);
                let entry = self.workspace_layout.frame_nav(content_list_id).map_or(
                    FrameNavigationEntry::Destination(MusicDestination::Search),
                    |nav| nav.current().clone(),
                );
                self.library.update(cx, |library, cx| {
                    if !matches!(
                        entry,
                        FrameNavigationEntry::Search(_)
                            | FrameNavigationEntry::Destination(_)
                            | FrameNavigationEntry::Settings
                    ) {
                        library.hydrate_detail_from_nav(&entry, cx);
                    }
                    library.refresh(cx);
                });
                // ADR 0084 Decision 3, packet 012: a saved-search page keeps
                // its rows, or shows the empty state of a deleted search.
                self.restore_saved_search_page_for_nav(&entry, cx);
            }
            AppTab::Show => {}
        }

        cx.notify();
    }

    /// The Music list column's "Select" or "Done" button (ADR 0084
    /// Decision 10 Amendment, 2026-10-19).
    fn toggle_list_column_selection_mode(&mut self, cx: &mut Context<Self>) {
        self.library.update(cx, |library, cx| {
            library.toggle_list_column_selection_mode(cx);
        });
        cx.notify();
    }

    fn set_search_results_tab(
        &mut self,
        _frame_id: WorkspaceFrameId,
        tab: SearchResultsTab,
        cx: &mut Context<Self>,
    ) {
        if let Some(detail) = &mut self.search_results_detail {
            detail.set_tab(tab);
            self.scroll_search_page_to_top(SearchPageSlot::ToolbarSearch);
            cx.notify();
        }
    }

    /// Sets the sort of the search page `slot` (ADR 0089 Decision 4). The
    /// rows follow the sort, and the query does not change.
    fn set_search_list_sort(
        &mut self,
        slot: SearchPageSlot,
        sort: crate::view_models::list_page::ListSort,
        cx: &mut Context<Self>,
    ) {
        if let Some(page) = self.search_page_mut(slot) {
            page.set_list_sort(sort);
            cx.notify();
        }
    }

    fn content_list_frame_id(&self) -> Option<WorkspaceFrameId> {
        self.workspace_layout
            .frames()
            .iter()
            .find(|frame| matches!(frame.kind(), WorkspaceFrameKind::ContentList))
            .map(WorkspaceFrameState::id)
    }

    fn handle_content_list_back_select(&mut self, cx: &mut Context<Self>) {
        let Some(content_list_id) = self.content_list_frame_id() else {
            return;
        };

        if let Some(entry) = self.workspace_layout.pop_nav(content_list_id) {
            // ADR 0084 task 030, decision D5 (review of 2026-10-21): only a
            // history move shows the query of a search page in the toolbar
            // field.
            self.pending_toolbar_search.record(&entry);
            self.restore_content_list_nav_entry(content_list_id, &entry, cx);
        }
    }

    /// Handles a Forward press on the content-list frame (ADR 0046 Task 015).
    ///
    /// Forward reuses the Back restore path: it moves the frame's navigation
    /// state ahead by one entry, then restores that entry's page through the
    /// same functions Back uses, so both directions keep one page restore
    /// path and the same stale-result checks.
    fn handle_content_list_forward_select(&mut self, cx: &mut Context<Self>) {
        let Some(content_list_id) = self.content_list_frame_id() else {
            return;
        };

        if let Some(entry) = self.workspace_layout.pop_nav_forward(content_list_id) {
            // ADR 0084 task 030, decision D5 (review of 2026-10-21): only a
            // history move shows the query of a search page in the toolbar
            // field.
            self.pending_toolbar_search.record(&entry);
            self.restore_content_list_nav_entry(content_list_id, &entry, cx);
        }
    }

    /// Opens a Music page in the one navigation stack (ADR 0084 Decision 9).
    /// A new path starts at `entry` itself, a destination
    /// (ADR 0084 Decision 2). A destination with a default child page
    /// (ADR 0089 Decision 4: the Search page lists the saved searches) opens that
    /// page under it, in the same path start. The page restores through
    /// the function that Back and Forward use.
    fn open_music_page(
        &mut self,
        entry: &FrameNavigationEntry,
        start: NavigationPathStart,
        cx: &mut Context<Self>,
    ) {
        let Some(content_list_id) = self.content_list_frame_id() else {
            return;
        };
        let changed = match start {
            NavigationPathStart::NewPath => {
                self.workspace_layout
                    .start_nav_path(content_list_id, entry.clone(), None)
            }
            NavigationPathStart::CurrentPath => self
                .workspace_layout
                .push_nav(content_list_id, entry.clone()),
            NavigationPathStart::ListRow => self
                .workspace_layout
                .open_nav_list_row(content_list_id, entry.clone()),
        };
        if let Err(error) = changed {
            self.settings_status = format!("Error opening page: {error}");
            cx.notify();
            return;
        }
        let opened_entry = entry.clone();
        // ADR 0084 Decision 3, packet 012: each open of a saved search runs
        // its query again. Back and Forward keep the mounted page.
        if matches!(opened_entry, FrameNavigationEntry::SavedSearch { .. }) {
            self.saved_search_page = None;
        }
        self.restore_content_list_nav_entry(content_list_id, &opened_entry, cx);
    }

    /// Restores the page for a navigation entry that Back, Forward or
    /// `open_music_page` selected on the
    /// content-list frame (ADR 0046 Task 015). A `Search` entry takes a new
    /// run and starts its Index part (ADR 0087 packet 003 decision D4).
    ///
    /// This function does not touch the toolbar field. ADR 0084 task 030,
    /// decision D5, corrected in the review of 2026-10-21: each caller
    /// that is a history move records the toolbar text itself, so a new
    /// submit does not overwrite a query the operator is still typing.
    fn restore_content_list_nav_entry(
        &mut self,
        content_list_id: WorkspaceFrameId,
        entry: &FrameNavigationEntry,
        cx: &mut Context<Self>,
    ) {
        self.restore_content_list_page(content_list_id, entry, cx);
        if let FrameNavigationEntry::Search(query) = entry {
            let run = self.next_search_run(SearchPageSlot::ToolbarSearch, query);
            self.start_index_search_for_run(SearchPageSlot::ToolbarSearch, &run, cx);
        }
    }

    /// Restores the page for `entry` on the content-list frame, and starts
    /// no Index part. A toolbar search run calls it, and then starts the
    /// Index part with its own run (ADR 0087 packet 007 decision D3).
    pub(super) fn restore_content_list_page(
        &mut self,
        content_list_id: WorkspaceFrameId,
        entry: &FrameNavigationEntry,
        cx: &mut Context<Self>,
    ) {
        // ADR 0087 packet 003 decision D10: each page change stops the
        // pending pause of the toolbar field.
        self.toolbar_search_runs.gate.cancel_pause();
        self.sync_search_results_detail_with_nav(content_list_id, cx);
        self.library.update(cx, |library, cx| {
            library.hydrate_detail_from_nav(entry, cx);
        });
        self.restore_publisher_page_for_nav(entry, cx);
        self.restore_name_level_page_for_nav(entry, cx);
        self.restore_saved_search_page_for_nav(entry, cx);
        self.restore_release_page_source_for_nav(entry, cx);
        self.restore_track_page_source_for_nav(entry, cx);
        cx.notify();
    }

    /// Writes the recorded toolbar query into the search field. The write
    /// does not move the focus and does not submit a search.
    ///
    /// ADR 0084 task 030, decision D5, corrected in the review of
    /// 2026-10-21: the caller records the text only on a history move
    /// (Back, Forward), never on a new submit or on
    /// `open_music_page`. The comparison trims the field value first. A
    /// typed query with a trailing space stays as typed, because the
    /// recorded text is already trimmed and the two then compare equal.
    fn apply_pending_toolbar_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(text) = self.pending_toolbar_search.take() else {
            return;
        };
        self.global_search_input.update(cx, |input, cx| {
            if input.value().trim() != text.as_str() {
                input.set_value(text, window, cx);
            }
        });
    }

    pub(super) fn focus_global_search(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.global_search_input.update(cx, |input, cx| {
            input.focus(window, cx);
        });
    }

    fn apply_playback_tick(&mut self, outcome: PlaybackTickOutcome, cx: &mut Context<Self>) {
        match outcome {
            PlaybackTickOutcome::Idle => {
                if self.show_page.is_active() {
                    self.refresh_show_page(cx);
                }
            }
            PlaybackTickOutcome::Advanced => {
                self.settings_status.clear();
                self.refresh_show_page(cx);
            }
            PlaybackTickOutcome::SessionStopped => {
                // ADR 0076 packet 005: the show released its files. The
                // tag update scan runs again, so the marks "In use by the
                // show" are current.
                self.refresh_show_page(cx);
                self.library
                    .update(cx, |library, _cx| library.scan_tag_updates());
            }
            PlaybackTickOutcome::Error(error) => {
                self.settings_status = format!("Playback error: {error}");
            }
        }
    }

    fn set_ui_scale(
        &mut self,
        scale: crate::config::UiScale,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.ui_scale == scale {
            return;
        }
        self.ui_scale = scale;
        crate::ui::theme_bridge::install_theme_for_window(
            self.theme_profile,
            scale.into(),
            window,
            cx,
        );
        cx.notify();
    }

    fn set_theme_profile(
        &mut self,
        profile: ThemeProfile,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.theme_profile == profile {
            return;
        }
        self.theme_profile = profile;
        crate::ui::theme_bridge::install_theme_for_window(
            profile,
            self.ui_scale.into(),
            window,
            cx,
        );
        cx.notify();
    }

    fn persist_workspace_layout(&self) -> Result<()> {
        config::save_workspace_layout(&self.cfg_path, &self.workspace_layout.to_config())
    }

    fn initial_workspace_layout(
        config: Option<&WorkspaceLayoutConfig>,
        mount: WorkspaceScreenMount,
    ) -> WorkspaceLayout {
        let layout = WorkspaceLayout::from_config(config);
        Self::visible_workspace_layout(&layout, mount)
    }

    fn visible_workspace_layout(
        layout: &WorkspaceLayout,
        mount: WorkspaceScreenMount,
    ) -> WorkspaceLayout {
        // ADR 0084 Decision 1: Music keeps the sidebar, the list and the
        // detail columns. Settings and Show keep the content-list frame
        // only, as before.
        if matches!(mount, WorkspaceScreenMount::Music) {
            return Self::visible_music_layout(layout);
        }

        let mut content_seen = false;
        let mut frames: Vec<_> = layout
            .frames()
            .iter()
            .filter_map(|frame| match frame.kind() {
                WorkspaceFrameKind::ContentList => {
                    if content_seen {
                        None
                    } else {
                        content_seen = true;
                        let title = Self::content_list_frame_title(layout, frame.id(), mount);
                        Some(WorkspaceFrameState::new(frame.id(), frame.kind(), title))
                    }
                }
                // ADR 0060 task 002: Queue lives in Show, not curation.
                WorkspaceFrameKind::QueueNowPlaying | WorkspaceFrameKind::Detail => None,
            })
            .collect();

        if !frames
            .iter()
            .any(|frame| matches!(frame.kind(), WorkspaceFrameKind::ContentList))
        {
            frames.insert(
                0,
                WorkspaceFrameState::new(
                    Self::unused_workspace_frame_id(&frames, WORKSPACE_CONTENT_FRAME_ID),
                    WorkspaceFrameKind::ContentList,
                    Self::root_content_frame_title(),
                ),
            );
        }

        let focused_frame_id = layout
            .focused_frame_id()
            .filter(|id| frames.iter().any(|frame| frame.id() == *id));
        let focused_frame_id = focused_frame_id.or_else(|| {
            frames
                .iter()
                .find(|frame| matches!(frame.kind(), WorkspaceFrameKind::ContentList))
                .map(WorkspaceFrameState::id)
        });

        let mut visible = WorkspaceLayout::new(frames, focused_frame_id)
            .expect("visible workspace layout preserves unique frame ids");
        let visible_ids: Vec<_> = visible
            .frames()
            .iter()
            .map(WorkspaceFrameState::id)
            .collect();
        for id in visible_ids {
            if let Some(nav) = layout.frame_nav(id).cloned() {
                visible
                    .replace_nav(id, nav)
                    .expect("visible workspace layout contains copied frame id");
            }
        }
        visible
    }

    /// Gives the Music visible layout: the sidebar, the list and the
    /// detail columns, in that order (ADR 0084 Decision 1). The columns
    /// carry no title: the toolbar shows the page title (ADR 0089 Decision
    /// 9). The sidebar shows no header.
    fn visible_music_layout(layout: &WorkspaceLayout) -> WorkspaceLayout {
        // ADR 0084 Decision 1: Music always shows its three columns, whatever
        // frames a saved `config.toml` layout holds. The saved content-list
        // frame keeps its id, because the one navigation stack lives on it.
        // A missing sidebar or detail frame gets a free id.
        let saved_id = |kind: WorkspaceFrameKind| {
            layout
                .frames()
                .iter()
                .find(|frame| frame.kind() == kind)
                .map(WorkspaceFrameState::id)
        };
        let content_list_id =
            saved_id(WorkspaceFrameKind::ContentList).unwrap_or(WORKSPACE_CONTENT_FRAME_ID);
        // ADR 0089 Decision 9: the page title is in the toolbar, so the
        // Music frames show no title.
        let mut frames = vec![WorkspaceFrameState::new(
            content_list_id,
            WorkspaceFrameKind::ContentList,
            String::new(),
        )];
        let detail_id = saved_id(WorkspaceFrameKind::Detail)
            .unwrap_or_else(|| Self::unused_workspace_frame_id(&frames, WorkspaceFrameId::new(3)));
        frames.push(WorkspaceFrameState::new(
            detail_id,
            WorkspaceFrameKind::Detail,
            String::new(),
        ));

        let focused_frame_id = layout
            .focused_frame_id()
            .filter(|id| frames.iter().any(|frame| frame.id() == *id));
        let focused_frame_id = focused_frame_id.or_else(|| {
            frames
                .iter()
                .find(|frame| matches!(frame.kind(), WorkspaceFrameKind::ContentList))
                .map(WorkspaceFrameState::id)
        });

        let mut visible = WorkspaceLayout::new(frames, focused_frame_id)
            .expect("visible music layout preserves unique frame ids");
        let visible_ids: Vec<_> = visible
            .frames()
            .iter()
            .map(WorkspaceFrameState::id)
            .collect();
        for id in visible_ids {
            if let Some(nav) = layout.frame_nav(id).cloned() {
                visible
                    .replace_nav(id, nav)
                    .expect("visible music layout contains copied frame id");
            }
        }
        visible
    }

    fn content_list_frame_title(
        layout: &WorkspaceLayout,
        frame_id: WorkspaceFrameId,
        _mount: WorkspaceScreenMount,
    ) -> String {
        let Some(nav) = layout.frame_nav(frame_id) else {
            return Self::root_content_frame_title();
        };
        match nav.current() {
            FrameNavigationEntry::Destination(_) | FrameNavigationEntry::Settings => {
                Self::root_content_frame_title()
            }
            entry => Self::page_type_title(entry),
        }
    }

    /// The title that names the type of a page, before its own facts give
    /// a name (ADR 0084 Decisions 4 and 5). `page_title` replaces it with
    /// the page's name where a page has one.
    fn page_type_title(entry: &FrameNavigationEntry) -> String {
        match entry {
            FrameNavigationEntry::Destination(_) | FrameNavigationEntry::Settings => {
                Self::root_content_frame_title()
            }
            FrameNavigationEntry::SavedSearch { .. } => {
                crate::view_models::search_results::SavedSearch::FRAME_TITLE.to_string()
            }
            FrameNavigationEntry::Search(_) => "Search Results".to_string(),
            // ADR 0089 packet 009: a public playlist shows its loaded title,
            // and this type title shows before it loads.
            FrameNavigationEntry::PlaylistDetail(_) | FrameNavigationEntry::PublicPlaylist(_) => {
                "Playlist".to_string()
            }
            // ADR 0084 Decision 4/5: the curator sees "Track" for a track
            // and "Release" for a feed of music, whatever its origin.
            FrameNavigationEntry::Track(_) => "Track".to_string(),
            FrameNavigationEntry::Release(_) => "Release".to_string(),
            // ADR 0084 packet 017 decision D5: the frame title of the name
            // level page.
            FrameNavigationEntry::ArtistName(_) => {
                crate::view_models::name_level_page::NameLevelPageVm::FRAME_TITLE.to_string()
            }
            // ADR 0085 Decision 7: the default title, before the page's
            // own facts load.
            FrameNavigationEntry::PublisherDetail(_) => "Feed".to_string(),
            FrameNavigationEntry::SinglesGroup(_) | FrameNavigationEntry::WindowGroup(_) => {
                "Singles".to_string()
            }
            FrameNavigationEntry::QueueNowPlaying => "Queue".to_string(),
        }
    }

    /// Gives the Music list column's title: the destination name, or
    /// "Search Results" while the location starts from a search (ADR 0084
    /// Decision 2). The list column shows the root of the location, not
    /// its current page.
    fn music_list_frame_title(layout: &WorkspaceLayout, frame_id: WorkspaceFrameId) -> String {
        let Some(nav) = layout.frame_nav(frame_id) else {
            return Self::root_content_frame_title();
        };
        match nav.root() {
            FrameNavigationEntry::Destination(destination) => destination.label().to_string(),
            FrameNavigationEntry::Search(_) => "Search Results".to_string(),
            _ => Self::root_content_frame_title(),
        }
    }

    fn root_content_frame_title() -> String {
        String::new()
    }

    fn unused_workspace_frame_id(
        frames: &[WorkspaceFrameState],
        preferred: WorkspaceFrameId,
    ) -> WorkspaceFrameId {
        if !frames.iter().any(|frame| frame.id() == preferred) {
            return preferred;
        }
        let next = frames
            .iter()
            .map(|frame| frame.id().value())
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        WorkspaceFrameId::new(next)
    }

    fn save_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let endpoint = self.endpoint_input.read(cx).value().to_string();
        let music_dir = self.music_dir_input.read(cx).value().to_string();
        let flac_path = self.flac_path_input.read(cx).value().to_string();
        let ui_scale = self.ui_scale;
        let theme_profile = self.theme_profile;
        match config::save_app_settings(
            &self.cfg_path,
            &endpoint,
            &music_dir,
            &flac_path,
            ui_scale,
            theme_profile,
        ) {
            Ok((
                normalized_endpoint,
                normalized_music_dir,
                normalized_flac_path,
                saved_scale,
                saved_profile,
            )) => {
                self.capability_vm.pending.saved();
                if let Err(error) = self.persist_workspace_layout() {
                    self.settings_status = format!("Error: {error:#}");
                    cx.notify();
                    return;
                }
                let download_preparation = config::prepare_artists_directory(&normalized_music_dir);
                self.musicindex_endpoint = normalized_endpoint.clone().into();
                self.library.update(cx, |library, cx| {
                    library.set_musicindex_endpoint(normalized_endpoint.clone().into(), cx);
                    library.set_music_dir(normalized_music_dir.clone(), cx);
                });
                self.endpoint_input.update(cx, |input, cx| {
                    input.set_value(normalized_endpoint.clone(), window, cx);
                });
                self.music_dir_input.update(cx, |input, cx| {
                    input.set_value(normalized_music_dir.display().to_string(), window, cx);
                });
                self.music_dir.clone_from(&normalized_music_dir);
                if let Some(owner) = &self.playback_owner {
                    if let Ok(mut playback_owner) = owner.lock() {
                        playback_owner.set_music_dir(normalized_music_dir);
                    }
                }
                let flac_display = normalized_flac_path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                self.flac_path_input.update(cx, |input, cx| {
                    input.set_value(flac_display, window, cx);
                });
                // Apply scale/profile change immediately so the UI reflects it.
                crate::ui::theme_bridge::install_theme_for_window(
                    saved_profile,
                    saved_scale.into(),
                    window,
                    cx,
                );
                self.settings_status = match download_preparation {
                    Ok(()) => format!(
                        "Saved settings. Music files download under {}/artists",
                        self.music_dir.display()
                    ),
                    Err(error) => format!(
                        "App applied the saved settings but could not prepare {}/artists for downloads: {error}",
                        self.music_dir.display()
                    ),
                };
            }
            Err(error) => {
                self.settings_status = format!("Error: {error:#}");
            }
        }
        cx.notify();
    }

    fn reload_cached(&mut self, cx: &mut Context<Self>) {
        self.cached_files.invalidate();
        self.start_cached_load(cx);
    }

    fn start_cached_load(&mut self, cx: &mut Context<Self>) {
        if !self.cached_files.begin_load() {
            return;
        }
        present_command(
            &self.command_runner,
            LoadCachedTracksTree::new(Arc::clone(&self.conn)),
            CommandContext::next(),
            cx,
            |this, result, cx| {
                this.cached_files.complete(Ok((result.count, result.tree)));
                this.start_cached_load(cx);
            },
            |this, error, cx| {
                this.cached_files.complete(Err(error));
                this.start_cached_load(cx);
            },
        );
    }

    fn delete_cached_file(&mut self, path: String, cx: &mut Context<Self>) {
        self.delete_cached_files(vec![path], cx);
    }

    fn delete_all_cached(&mut self, cx: &mut Context<Self>) {
        let paths: Vec<String> = self
            .cached_files
            .tree()
            .artists
            .iter()
            .flat_map(|a| &a.albums)
            .flat_map(|a| &a.tracks)
            .filter_map(|t| {
                t.local_path
                    .as_ref()
                    .map(|path| path.resolve(&self.music_dir).display().to_string())
            })
            .collect();
        self.delete_cached_files(paths, cx);
    }

    fn delete_cached_files(&mut self, paths: Vec<String>, cx: &mut Context<Self>) {
        if paths.is_empty() {
            return;
        }
        let command = RemoveCachedFiles::new(Arc::clone(&self.conn), self.music_dir.clone(), paths);
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            |this, result, cx| {
                this.settings_status = result.message().to_string();
                this.reload_cached(cx);
            },
            |this, error, _cx| {
                this.settings_status = format!("Error: {error:#}");
            },
        );
    }

    fn active_workspace_screen_mount(&self) -> WorkspaceScreenMount {
        self.tab.into()
    }

    fn resolve_search_result_thumbnails(
        &mut self,
        thumbnail_hrefs: Vec<String>,
        cx: &mut Context<Self>,
    ) -> BTreeMap<String, Option<Arc<Image>>> {
        let mut thumbnails = BTreeMap::new();
        for href in thumbnail_hrefs {
            if thumbnails.contains_key(&href) {
                continue;
            }
            let thumbnail = self.index_remote_detail_hero_image(&href, cx);
            thumbnails.insert(href, thumbnail);
        }
        thumbnails
    }

    /// The loaded logo frame art of each search row with art, by row id
    /// (packet 028 D3), and of each group row of a saved search, by its
    /// element id (ADR 0086 packet 005). It starts each load that is not
    /// known, through the same path as the Artists list.
    fn resolve_search_result_arts(
        &mut self,
        items: &[crate::view_models::search_results::SearchResultItemVm],
        cx: &mut Context<Self>,
    ) -> BTreeMap<String, crate::ui::composites::CollectionArtImage> {
        use crate::view_models::search_results::SearchResultItemVm;
        items
            .iter()
            .filter_map(|item| match item {
                SearchResultItemVm::Row(row) => Some((row.id.clone(), &row.art.as_ref()?.art)),
                SearchResultItemVm::WindowGroup(tile) => Some((tile.element_id.clone(), &tile.art)),
                SearchResultItemVm::Heading(_) => None,
            })
            .map(|(id, art)| {
                let image =
                    collection_art_image(self, art, crate::ui::composites::ThumbnailSize::Sm, cx);
                (id, image)
            })
            .collect()
    }

    fn render_workspace_screen_mount(
        &mut self,
        mount: WorkspaceScreenMount,
        show_window_width: f32,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        match mount {
            // ADR 0084 Decision 1: Music mounts as three column views, not as
            // one whole screen. No Music path calls this function. The arm
            // gives the list column so that the match stays total.
            WorkspaceScreenMount::Music => self.library_list_view.clone().into_any_element(),
            WorkspaceScreenMount::Show => {
                build_show_screen(self, show_window_width, cx).into_any_element()
            }
            WorkspaceScreenMount::Settings => render_settings(self, cx),
        }
    }

    /// Gives the slots for a page that the Library's own detail renders
    /// (ADR 0084 Decision 1). In Music, the page fills the detail slot,
    /// through `library_detail_view`. Settings and Show never reach a
    /// Library page, so they get empty slots.
    fn library_or_screen_slot(&self, fills_detail: bool) -> WorkspaceSlots {
        if fills_detail {
            WorkspaceSlots::new().detail_slot(self.library_detail_view.clone())
        } else {
            WorkspaceSlots::new()
        }
    }

    /// Fills the list column with the search results (ADR 0084 Decision
    /// 2, packet 010). A result row opens its page in the detail column.
    /// The Releases tab and the Tracks tab have the "Select" button, the
    /// keyboard focus and the action bar of packet 018.
    fn with_search_results_list(
        &mut self,
        slots: WorkspaceSlots,
        content_frame_id: WorkspaceFrameId,
        cx: &mut Context<Self>,
    ) -> WorkspaceSlots {
        let entity = cx.entity();
        let (items, header_action, bar, thumbnail_hrefs, save_action) = {
            let library = self.library.read(cx);
            let lookup = library.search_lookup();
            let search_results = self.search_results_detail.as_ref().unwrap();
            (
                search_results.visible_items(&lookup),
                search_results.header_action(&lookup),
                search_results
                    .action_bar(&lookup)
                    .zip(search_results.selection_rows(&lookup)),
                search_results.thumbnail_hrefs(&lookup),
                search_results.save_action(library.saved_searches()),
            )
        };
        let thumbnails = self.resolve_search_result_thumbnails(thumbnail_hrefs, cx);
        let arts = self.resolve_search_result_arts(&items, cx);
        let tab_entity = entity.clone();
        let select_entity = entity.clone();
        let toggle_entity = entity.clone();
        let failure_entity = entity.clone();
        let save_entity = entity.clone();
        let scope_entity = entity.clone();
        let sort_entity = entity.clone();
        let inspector_slots = SearchResultsInspectorSlots::new()
            .on_failure_action(move |action, _window, cx| {
                failure_entity.update(cx, |this, cx| {
                    this.handle_search_failure_action(SearchPageSlot::ToolbarSearch, action, cx);
                });
            })
            .on_tab_select(move |tab, _window, cx| {
                tab_entity.update(cx, |this, cx| {
                    this.set_search_results_tab(content_frame_id, tab, cx);
                });
            })
            .on_scope_select(move |scope, window, cx| {
                scope_entity.update(cx, |this, cx| {
                    this.set_toolbar_library_scope(scope, window, cx);
                });
            })
            .on_sort_select(sort_handler(sort_entity, SearchPageSlot::ToolbarSearch))
            .on_result_select(move |tab, result_id, key, modifiers, window, cx| {
                select_entity.update(cx, |this, cx| {
                    this.click_search_result_row(
                        SearchPageSlot::ToolbarSearch,
                        SearchRowClick {
                            tab,
                            result_id,
                            key,
                            modifiers,
                        },
                        window,
                        cx,
                    );
                });
            })
            .on_row_toggle(move |key, window, cx| {
                toggle_entity.update(cx, |this, cx| {
                    this.toggle_search_result_row(SearchPageSlot::ToolbarSearch, &key, window, cx);
                });
            })
            .header_action(save_action, move |_window, cx| {
                save_entity.update(cx, |this, cx| {
                    this.save_toolbar_search(cx);
                });
            })
            .with_thumbnails(thumbnails)
            .with_arts(arts)
            .with_items(items)
            .with_scroll(self.search_page_scroll(SearchPageSlot::ToolbarSearch))
            .with_marked_entry(
                self.workspace_layout
                    .frame_nav(content_frame_id)
                    .and_then(|nav| nav.marked_entry().cloned()),
            );
        let inspector_content = render_search_results_inspector(
            self.search_results_detail.as_ref().unwrap(),
            &inspector_slots,
            cx,
        );
        let bar = bar.map(|(bar, rows)| {
            self.render_search_action_bar(SearchPageSlot::ToolbarSearch, bar, &rows, cx)
        });
        let content =
            self.render_search_list_frame(SearchPageSlot::ToolbarSearch, inspector_content, bar);
        let mut slots = slots.content_list(content);
        if let Some(header_action) = header_action {
            let header_action_entity = entity;
            slots = slots
                .content_list_header_action(header_action)
                .on_content_list_header_action(move |_window, cx| {
                    header_action_entity.update(cx, |this, cx| {
                        this.toggle_search_selection_mode(SearchPageSlot::ToolbarSearch, cx);
                    });
                });
        }
        slots
    }

    /// Gives the page content state of a Music page entry (ADR 0089
    /// Decision 5). An Index page renders its own state. A Library page
    /// is loaded when the Library detail holds it, and loading while the
    /// Library tree reloads.
    fn music_page_content_state(&self, entry: &FrameNavigationEntry, cx: &App) -> MusicPageContent {
        let own_content = match entry {
            FrameNavigationEntry::Release(_) => matches!(
                self.release_page_source,
                Some(ReleasePageSource::Index { .. })
            ),
            FrameNavigationEntry::Track(_) => {
                matches!(self.track_page_source, Some(TrackPageSource::Index { .. }))
            }
            FrameNavigationEntry::PlaylistDetail(_)
            | FrameNavigationEntry::SinglesGroup(_)
            | FrameNavigationEntry::WindowGroup(_) => false,
            _ => true,
        };
        let library = self.library.read(cx);
        music_page_content(
            own_content,
            library.detail_shows_page(entry),
            library.is_library_loading(),
        )
    }

    /// Gives the visible Music frame and the page content state of the
    /// current entry (ADR 0089 Decisions 1 and 5). Other mounts show no
    /// Music frame.
    fn music_frame_for(
        &self,
        mount: WorkspaceScreenMount,
        entry: Option<&FrameNavigationEntry>,
        cx: &App,
    ) -> (MusicColumns, MusicPageContent) {
        match (mount, entry) {
            (WorkspaceScreenMount::Music, Some(entry)) => {
                let content = self.music_page_content_state(entry, cx);
                (MusicColumns::for_entry(entry, content), content)
            }
            _ => (MusicColumns::ListRoot, MusicPageContent::Absent),
        }
    }

    /// Gives the keyboard target of the visible Music frame (ADR 0089
    /// Invariants). Gives `None` outside the Music mount.
    fn music_focus_target_now(&self, cx: &App) -> Option<MusicFocusTarget> {
        let layout =
            Self::visible_workspace_layout(&self.workspace_layout, WorkspaceScreenMount::Music);
        let content_frame_id = layout
            .frames()
            .iter()
            .find(|frame| matches!(frame.kind(), WorkspaceFrameKind::ContentList))
            .map_or(WORKSPACE_CONTENT_FRAME_ID, WorkspaceFrameState::id);
        let nav = self.workspace_layout.frame_nav(content_frame_id)?;
        let entry = nav.current().clone();
        let (columns, _) = self.music_frame_for(WorkspaceScreenMount::Music, Some(&entry), cx);
        let search_results_open =
            nav.active_search_query().is_some() && self.search_results_detail.is_some();
        Some(music_focus_target(columns, &entry, search_results_open))
    }

    /// Moves the window focus to the Music frame that shows, when the frame
    /// changes (ADR 0089 Invariants). A row click, Enter, Back and Forward
    /// all change the frame, so the keys reach the frame that shows. The
    /// focus does not move while the frame stays the same.
    fn apply_music_focus(
        &mut self,
        mount: WorkspaceScreenMount,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let target = match mount {
            WorkspaceScreenMount::Music => self.music_focus_target_now(cx),
            WorkspaceScreenMount::Show | WorkspaceScreenMount::Settings => None,
        };
        if target == self.music_focus_applied {
            return;
        }
        self.music_focus_applied = target;
        let Some(target) = target else {
            return;
        };
        let search_field_focused = self
            .global_search_input
            .read(cx)
            .focus_handle(cx)
            .contains_focused(window, cx);
        if !music_focus_moves(target, search_field_focused) {
            return;
        }
        let handle = match target {
            MusicFocusTarget::ListColumn => self.library.read(cx).list_column_focus().clone(),
            MusicFocusTarget::DetailColumn => self.library.read(cx).detail_column_focus().clone(),
            MusicFocusTarget::SearchResults => self
                .search_page_focus(SearchPageSlot::ToolbarSearch)
                .clone(),
            MusicFocusTarget::SearchField => self.global_search_input.read(cx).focus_handle(cx),
            MusicFocusTarget::SavedSearchPage => {
                self.search_page_focus(SearchPageSlot::SavedSearch).clone()
            }
        };
        window.focus(&handle, cx);
    }

    #[expect(
        clippy::too_many_lines,
        reason = "ADR 0050 keeps the workspace dispatcher centralized while handler clusters move out"
    )]
    fn render_workspace_content(
        &mut self,
        mount: WorkspaceScreenMount,
        show_window_width: f32,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        if matches!(mount, WorkspaceScreenMount::Show) {
            return self.render_workspace_screen_mount(mount, show_window_width, cx);
        }

        let layout = Self::visible_workspace_layout(&self.workspace_layout, mount);
        let content_frame_id = layout
            .frames()
            .iter()
            .find(|frame| matches!(frame.kind(), WorkspaceFrameKind::ContentList))
            .map_or(WORKSPACE_CONTENT_FRAME_ID, WorkspaceFrameState::id);
        let entity = cx.entity();
        // ADR 0084 Decision 1: in Music, a top-app page fills the detail
        // slot, and the list column always shows the Library list. In
        // Settings, the page still fills the content-list slot, as before.
        let fills_detail = matches!(mount, WorkspaceScreenMount::Music);

        // Stage 2: Body switch is exhaustive on nav top
        let current_nav = self
            .workspace_layout
            .frame_nav(content_frame_id)
            .map(|nav| nav.current().clone());
        // ADR 0089 Decision 4: a Back or a destination change ends the query
        // field of the saved-search page.
        self.discard_saved_search_query_edit_when_left(current_nav.as_ref());

        // ADR 0084 packet 018: the detail selection belongs to the open
        // page. A move to another page clears it and sets `Normal`.
        // ADR 0084 Decision 9, amended on 2026-10-21. A new open shows the
        // page at the top. Back and Forward show the stored offset of the
        // history entry. The entry that closed keeps its offset.
        if let (true, Some(entry), Some(target)) = (
            fills_detail,
            &current_nav,
            self.workspace_layout
                .frame_nav(content_frame_id)
                .map(FrameNavigationState::detail_scroll_target),
        ) {
            let closed = self.library.update(cx, |library, _cx| {
                library.bind_detail_selection(entry, target)
            });
            if let (Some((location, offset)), Some(nav)) = (
                closed,
                self.workspace_layout.frame_nav_mut(content_frame_id),
            ) {
                nav.record_detail_scroll(location, offset);
            }
        }
        // ADR 0089 Decisions 1 and 5: the frame that shows, and the page
        // content state, come from one fact. The shell and the keyboard
        // gates read the same fact.
        let (music_columns, page_content) = self.music_frame_for(mount, current_nav.as_ref(), cx);
        self.library.update(cx, |library, _cx| {
            library.set_music_page_open(music_columns.shows_detail());
        });
        // ADR 0084 Decision 3, packet 012: the Search page marks the
        // saved search of the current location as its open row.
        let open_saved_search = self
            .workspace_layout
            .frame_nav(content_frame_id)
            .and_then(FrameNavigationState::active_saved_search);
        self.library.update(cx, |library, _cx| {
            library.set_open_saved_search(open_saved_search);
        });
        // ADR 0089 Decision 9, R-NAV-2 item 2: the row of the page that Back
        // left open is marked in the Music list.
        let marked_entry = self
            .workspace_layout
            .frame_nav(content_frame_id)
            .and_then(|nav| nav.marked_entry().cloned());
        self.library.update(cx, |library, _cx| {
            library.set_marked_entry(marked_entry);
        });

        let mut workspace_slots = match &current_nav {
            // ADR 0084 Decision 2: search results fill the list column, and
            // a result opens in the detail column. With no result open, the
            // Music detail column shows its empty state.
            Some(FrameNavigationEntry::Search(_)) if self.search_results_detail.is_some() => {
                if fills_detail {
                    WorkspaceSlots::new().detail_slot(self.library_detail_view.clone())
                } else {
                    self.with_search_results_list(WorkspaceSlots::new(), content_frame_id, cx)
                }
            }
            // ADR 0084 Decision 3, packet 012: a saved search shows its
            // results in the detail column, as "New releases" does.
            Some(FrameNavigationEntry::SavedSearch { .. }) => {
                let detail_content = self.render_saved_search_page_content(cx);
                if fills_detail {
                    WorkspaceSlots::new().detail_slot(detail_content)
                } else {
                    WorkspaceSlots::new().content_list(detail_content)
                }
            }
            // ADR 0084 Decision 7, packet 017: the name level page reads the
            // Library tree at each render and its own MusicIndex request.
            Some(FrameNavigationEntry::ArtistName(_)) => {
                let detail_content = self.render_name_level_page_content(cx);
                if fills_detail {
                    WorkspaceSlots::new().detail_slot(detail_content)
                } else {
                    WorkspaceSlots::new().content_list(detail_content)
                }
            }
            // ADR 0084 Decision 4: the body switch renders a release from
            // its stored, already-resolved data source. It makes no
            // database lookup here.
            Some(FrameNavigationEntry::Release(_)) => match self.release_page_source.clone() {
                Some(ReleasePageSource::Index { feed_guid }) => {
                    // While the operator's own open-time fetch is still
                    // loading, the cached content-list row shows at once,
                    // when the content list already holds it.
                    let detail = self.index_release_page_detail(&feed_guid, cx);
                    let detail_content = self.render_index_feed_or_fallback_detail(&detail, cx);
                    if fills_detail {
                        WorkspaceSlots::new().detail_slot(detail_content)
                    } else {
                        WorkspaceSlots::new().content_list(detail_content)
                    }
                }
                Some(ReleasePageSource::Library { .. }) | None => {
                    self.library_or_screen_slot(fills_detail)
                }
            },
            // ADR 0084 Decision 4: the body switch renders a track from
            // its stored, already-resolved data source. It makes no
            // database lookup here.
            Some(FrameNavigationEntry::Track(_)) => match self.track_page_source.clone() {
                Some(TrackPageSource::Index { target }) => {
                    let label = self.track_opening_label(&target);
                    let detail = self.index_track_detail_display(&target, &label);
                    let detail_content = self.render_index_feed_or_fallback_detail(&detail, cx);
                    if fills_detail {
                        WorkspaceSlots::new().detail_slot(detail_content)
                    } else {
                        WorkspaceSlots::new().content_list(detail_content)
                    }
                }
                Some(TrackPageSource::Library { .. }) | None => {
                    self.library_or_screen_slot(fills_detail)
                }
            },
            Some(FrameNavigationEntry::Settings) => {
                let settings_screen = self.render_workspace_screen_mount(
                    WorkspaceScreenMount::Settings,
                    show_window_width,
                    cx,
                );
                WorkspaceSlots::new().content_list(settings_screen)
            }
            // ADR 0077 packet 004: the publisher page loads through its own
            // fetch, independent of the Library tree and the Index results.
            Some(FrameNavigationEntry::PublisherDetail(_)) => {
                let detail_content = self.render_publisher_page_content(cx);
                if fills_detail {
                    WorkspaceSlots::new().detail_slot(detail_content)
                } else {
                    WorkspaceSlots::new().content_list(detail_content)
                }
            }
            // ADR 0089 packet 009: the item page of a public playlist reads
            // its own state in the Library.
            Some(FrameNavigationEntry::PublicPlaylist(feed_guid)) => {
                let feed_guid = feed_guid.clone();
                let detail_content = self.library.update(cx, |library, cx| {
                    library.render_public_playlist_page(&feed_guid, window, cx)
                });
                if fills_detail {
                    WorkspaceSlots::new().detail_slot(detail_content)
                } else {
                    WorkspaceSlots::new().content_list(detail_content)
                }
            }
            // Entity details, no selection, and every other nav variant
            // (for example a search with no loaded results): render the
            // Library-backed Music surface.
            _ => self.library_or_screen_slot(fills_detail),
        };

        // ADR 0084 Decisions 1 and 2: the Music list column shows the search
        // results while the location starts from a search, else the Library
        // list. The open page does not change it. ADR 0089 Decision 1: the
        // sidebar is not a frame, so `render` places it beside the content.
        let search_location_is_open = self
            .workspace_layout
            .frame_nav(content_frame_id)
            .and_then(FrameNavigationState::active_search_query)
            .is_some()
            && self.search_results_detail.is_some();
        if matches!(mount, WorkspaceScreenMount::Music) && search_location_is_open {
            workspace_slots = self.with_search_results_list(workspace_slots, content_frame_id, cx);
        } else if matches!(mount, WorkspaceScreenMount::Music) {
            workspace_slots = workspace_slots.content_list(self.library_list_view.clone());
            // ADR 0084 Decision 10 Amendment, 2026-10-19: the list
            // column's "Select" and "Done" button, for Tracks and Needs
            // attention. Releases and Artists show it in their title row
            // (ADR 0089 Decision 6). The view model gives `None` for every
            // other destination and for an empty list.
            if let Some(header_action) = self.library.read(cx).header_action_for_list_column() {
                let header_action_entity = entity.clone();
                workspace_slots = workspace_slots
                    .content_list_header_action(header_action)
                    .on_content_list_header_action(move |_window, cx| {
                        header_action_entity.update(cx, |this, cx| {
                            this.toggle_list_column_selection_mode(cx);
                        });
                    });
            }
        }

        // ADR 0089 Decision 5: a page whose data is loading shows the shared
        // loading state in the detail column. The shell gets the frame fact.
        if matches!(mount, WorkspaceScreenMount::Music) {
            workspace_slots = workspace_slots.music_columns(music_columns);
            if page_content == MusicPageContent::Loading {
                workspace_slots =
                    workspace_slots.detail_slot(crate::ui::composites::SkeletonInspector::new());
            }
        }

        // ADR 0084 packet 018: the detail column has its own keyboard
        // focus, its "Select" and "Done" button and its action bar.
        if fills_detail {
            if let Some(content) = workspace_slots.take_detail_slot() {
                let content = self.render_detail_column_frame(content, cx);
                workspace_slots = workspace_slots.detail_slot(content);
            }
            if let Some(header_action) = self.detail_column_header_action(cx) {
                let header_action_entity = entity.clone();
                workspace_slots = workspace_slots
                    .detail_header_action(header_action)
                    .on_detail_header_action(move |_window, cx| {
                        header_action_entity.update(cx, |this, cx| {
                            this.toggle_detail_selection_mode(cx);
                        });
                    });
            }
        }

        // Stage 6: Wire content pane width and resize handler
        workspace_slots = workspace_slots
            .content_pane_width(self.content_pane_width)
            .on_content_pane_resize_start({
                let resize_entity = entity.clone();
                move |_window, cx| {
                    resize_entity.update(cx, |this, cx| {
                        this.begin_content_pane_resize(cx);
                    });
                }
            })
            .on_content_pane_resize_move({
                let resize_entity = entity.clone();
                move |x, _window, cx| {
                    resize_entity.update(cx, |this, cx| {
                        this.resize_content_pane(x, cx);
                    });
                }
            })
            .on_content_pane_resize_end({
                let resize_entity = entity.clone();
                move |_width, _window, cx| {
                    resize_entity.update(cx, |this, cx| {
                        this.end_content_pane_resize(cx);
                    });
                }
            });

        render_workspace(&layout, workspace_slots, cx).into_any_element()
    }
}

impl Drop for TopApp {
    fn drop(&mut self) {
        if !self
            .command_runner
            .session()
            .accepts(self.command_runner.session().generation())
        {
            return;
        }
        if let Some(owner) = &self.playback_owner {
            match owner.lock() {
                Ok(mut playback_owner) => {
                    if let Some(warning) = playback_owner.broadcast_drop_file_shutdown_warning() {
                        eprintln!("{warning}");
                    }
                    if let Err(error) = playback_owner.clear_broadcast_drop_file() {
                        eprintln!(
                        "v4vmm::broadcast: failed to remove mpv now-playing drop file on shutdown: {error:#}"
                    );
                    }
                }
                Err(_) => {
                    eprintln!("v4vmm::broadcast: failed to lock playback owner on shutdown");
                }
            }
        }
        if let Err(error) = self.persist_workspace_layout() {
            eprintln!("v4vmm::workspace: failed to save workspace layout on shutdown: {error:#}");
        }
    }
}

impl Render for TopApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.defer_application_event_drain(window, cx);
        self.apply_pending_toolbar_search(window, cx);
        let mount = self.active_workspace_screen_mount();
        self.apply_music_focus(mount, window, cx);
        let show_window_width = f32::from(window.bounds().size.width);
        let sidebar_row = self.sidebar_row_for_mount(mount, cx);
        self.library.update(cx, |library, cx| {
            library.set_sidebar_selected(Some(sidebar_row), cx);
        });
        let live_status_strip = build_live_status_strip(self, mount, cx);
        let capability_notice = if mount == WorkspaceScreenMount::Settings {
            None
        } else {
            self.render_capabilities(false, cx)
        };
        let bg_canvas = color(cx, SemanticColor::SystemBackground);
        let text_primary = color(cx, SemanticColor::Label);
        // ADR 0089 Decision 1: a narrow window shows the tab bar at the bottom
        // of the window, and no sidebar. Otherwise the sidebar sits beside the
        // content on each mount. The toolbar belongs to the content area
        // (R-TOOLBAR-1).
        let shell_form = crate::view_models::sidebar::shell_form(
            show_window_width,
            f32::from(crate::ui::layouts::SHELL_TAB_BAR_BREAKPOINT),
        );
        let sidebar_column =
            (shell_form == ShellForm::Sidebar).then(|| self.render_sidebar_shell(cx));
        let tab_bar = (shell_form == ShellForm::TabBar).then(|| self.render_narrow_tab_bar(cx));
        let content_column = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .overflow_hidden()
            .child(render_toolbar(self, mount, window, cx))
            .when_some(capability_notice, gpui::ParentElement::child)
            .child(
                div()
                    .key_context(keyboard::ACTIVE_PANE_KEY_CONTEXT)
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .overflow_hidden()
                    .when_some(live_status_strip, gpui::ParentElement::child)
                    // ADR 0084 Decision 10: the download progress item
                    // shows on each section, beside the live status strip.
                    .child(self.download_status_view.clone())
                    // ADR 0060: workspace render delegates to the active app-section mount.
                    .child(self.render_workspace_content(mount, show_window_width, window, cx)),
            );
        let body = div()
            .flex()
            .flex_row()
            .flex_1()
            .min_h_0()
            .overflow_hidden()
            .when_some(sidebar_column, gpui::ParentElement::child)
            .child(content_column);
        div()
            .size_full()
            .bg(bg_canvas)
            .text_color(text_primary)
            .text_sm()
            .flex()
            .flex_col()
            .overflow_hidden()
            .on_action(cx.listener(TopApp::handle_toggle_playback))
            .on_action(cx.listener(TopApp::handle_skip_playback_next))
            .on_action(cx.listener(TopApp::handle_skip_playback_previous))
            .on_action(cx.listener(TopApp::handle_open_preferences))
            .on_action(cx.listener(TopApp::handle_navigate_back))
            .on_action(cx.listener(TopApp::handle_navigate_forward))
            .on_action(cx.listener(TopApp::handle_focus_search))
            .on_action(cx.listener(TopApp::handle_new_playlist))
            .on_action(cx.listener(TopApp::handle_select_music_tab))
            .on_action(cx.listener(TopApp::handle_select_show_tab))
            .on_action(cx.listener(TopApp::handle_select_settings_tab))
            .on_action(cx.listener(TopApp::handle_refresh_library))
            .on_action(cx.listener(TopApp::handle_cancel_active_pane))
            .on_action(cx.listener(TopApp::handle_move_selection_up))
            .on_action(cx.listener(TopApp::handle_move_selection_down))
            .on_action(cx.listener(TopApp::handle_confirm_selection))
            .on_action(cx.listener(TopApp::handle_move_selection_page_up))
            .on_action(cx.listener(TopApp::handle_move_selection_page_down))
            .on_action(cx.listener(TopApp::handle_move_selection_home))
            .on_action(cx.listener(TopApp::handle_open_focused_group))
            .on_action(cx.listener(TopApp::handle_close_focused_group))
            .on_action(cx.listener(TopApp::handle_move_selection_end))
            .on_action(cx.listener(TopApp::handle_toggle_focused_selection))
            .on_action(cx.listener(TopApp::handle_select_all_visible))
            .child(body)
            .when_some(tab_bar, gpui::ParentElement::child)
            .children(render_window_layers(window, cx))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        TopApp, WorkspaceFrameId, WorkspaceFrameKind, WorkspaceFrameState, WorkspaceLayout,
        WorkspaceScreenMount,
    };

    /// ADR 0089 Decision 1: the Music visible layout has the content-list
    /// frame and the detail frame, in that order. The sidebar is not a frame.
    #[test]
    fn adr_0089_music_visible_layout_has_list_and_detail_frames_in_order() {
        let layout = WorkspaceLayout::from_config(None);
        let visible = TopApp::visible_workspace_layout(&layout, WorkspaceScreenMount::Music);
        let kinds: Vec<_> = visible
            .frames()
            .iter()
            .map(WorkspaceFrameState::kind)
            .collect();
        assert_eq!(
            kinds,
            vec![WorkspaceFrameKind::ContentList, WorkspaceFrameKind::Detail]
        );
    }

    /// ADR 0089 Decision 1: a saved layout with only the content-list frame
    /// still gives Music its list and detail frames, and keeps the
    /// content-list frame id.
    #[test]
    fn adr_0089_saved_single_frame_layout_still_has_list_and_detail_frames() {
        let content_list_id = WorkspaceFrameId::new(2);
        let layout = WorkspaceLayout::new(
            vec![WorkspaceFrameState::with_default_title(
                content_list_id,
                WorkspaceFrameKind::ContentList,
            )],
            Some(content_list_id),
        )
        .expect("a one-frame layout is valid");
        let visible = TopApp::visible_workspace_layout(&layout, WorkspaceScreenMount::Music);
        let frames: Vec<_> = visible
            .frames()
            .iter()
            .map(|frame| (frame.kind(), frame.id()))
            .collect();
        assert_eq!(
            frames.iter().map(|(kind, _)| *kind).collect::<Vec<_>>(),
            vec![WorkspaceFrameKind::ContentList, WorkspaceFrameKind::Detail]
        );
        assert_eq!(
            frames[0].1, content_list_id,
            "the content-list frame must keep its saved id"
        );
        assert!(
            frames[1].1 != content_list_id,
            "each frame id must be unique"
        );
    }

    /// R12-05 (ADR 0084 Decision 3, packet 012): the frame title of a
    /// saved-search page is "Saved search".
    #[test]
    fn adr_0084_saved_search_r12_05_frame_title_is_saved_search() {
        use crate::view_models::search_results::SavedSearchId;
        use crate::view_models::workspace::FrameNavigationEntry;

        let mut layout = WorkspaceLayout::from_config(None);
        let content_list_id = layout
            .frames()
            .iter()
            .find(|frame| matches!(frame.kind(), WorkspaceFrameKind::ContentList))
            .map(WorkspaceFrameState::id)
            .expect("the default layout has a content-list frame");
        layout
            .open_nav_list_row(
                content_list_id,
                FrameNavigationEntry::SavedSearch {
                    id: SavedSearchId(3),
                    query: "bluegrass".to_owned(),
                },
            )
            .expect("the content-list frame opens the row");
        assert_eq!(
            TopApp::content_list_frame_title(&layout, content_list_id, WorkspaceScreenMount::Music),
            "Saved search"
        );
    }

    /// C4-01: the Settings visible layout has the content-list frame only.
    #[test]
    fn adr_0084_columns_c4_01_settings_visible_layout_has_content_list_only() {
        let layout = WorkspaceLayout::from_config(None);
        let visible = TopApp::visible_workspace_layout(&layout, WorkspaceScreenMount::Settings);
        let kinds: Vec<_> = visible
            .frames()
            .iter()
            .map(WorkspaceFrameState::kind)
            .collect();
        assert_eq!(kinds, vec![WorkspaceFrameKind::ContentList]);
    }
}
