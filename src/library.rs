#![warn(clippy::pedantic)]
#![expect(
    clippy::doc_markdown,
    clippy::manual_let_else,
    clippy::needless_pass_by_value,
    clippy::single_match_else,
    clippy::too_many_lines,
    clippy::unused_self,
    reason = "legacy screen module is being migrated incrementally under ADR 0023"
)]

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use gpui::{Entity, FocusHandle, Image, ScrollHandle};
use gpui_component::input::InputState;

use crate::api::{ArtistNameRow, Feed, LiveItem};
use crate::application::paged_track_list::{PagedTrackListMsg, PagedTrackListSnapshot};
use crate::application::{ApplicationServices, AsyncCommandRunner};
use crate::db::{self, TrackRow};
use crate::media::ImageCache;
use crate::metadata::{MusicBrainzLookupResult, PendingId3Edit, TagCompareResult, TrackContext};
use crate::presentation::collection_art::{
    CollectionArtHost, CollectionArtLoads, CollectionArtParts,
};
use crate::presentation::RuntimeHost;
use crate::runtime::actor::ActorHandle;
use crate::runtime::musicbrainz_feed_saga::MusicBrainzFeedSagaHandle;
use crate::ui::composites::music_column_scroll::{ListBarPin, MusicColumnList, ScrollReveal};
use crate::view_models::detail_selection::DetailSelection;
use crate::view_models::index_name_list::IndexNameListPage;
use crate::view_models::library::{
    description_line_count, AlbumNode, InspectorPanelKind, LibraryTrackInspectorDisplay,
    LibraryTrackInspectorState, LibraryViewModel, ReleasesRowId,
};
use crate::view_models::library_removal::{
    SelectionRemovalConfirmationState, SelectionRemovalReport,
};
use crate::view_models::list_selection::{ListSelection, SelectionMode};
use crate::view_models::public_playlists::PublicPlaylistPages;
use crate::view_models::recent_feeds::RecentFeedsPageVm;
use crate::view_models::releases_everything::ReleasesListCache;
use crate::view_models::selection_action_bar::ListBarCommand;
use crate::view_models::singles_group::SinglesGroupKey;
use crate::view_models::window_groups::WindowGroupKey;
use crate::view_models::workspace::{
    FrameNavigationEntry, NavigationPathStart, ReleaseKey, TrackKey,
};
use download_queue::DownloadQueue;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub(crate) enum LibraryDetail {
    None,
    Album(AlbumNode),
    Track(Box<InspectorFrame>),
    Playlist(PlaylistDetail),
    /// The page of one Library singles group (ADR 0084 Decision 6). The
    /// page reads the current tree at each render, so a download or a
    /// removal updates it in place.
    SinglesGroup(SinglesGroupKey),
    /// The page of one window group of New releases (ADR 0086 Decision 4).
    /// The page reads the cached layout at each render.
    WindowGroup(WindowGroupKey),
}

#[derive(Clone, Debug)]
pub(crate) struct PlaylistDetail {
    pub(crate) playlist: db::Playlist,
    pub(crate) tracks: Vec<TrackRow>,
}

#[derive(Clone, Debug)]
pub(crate) enum LibraryAppEvent {
    /// A click on the list bar above the rows of a group page (ADR 0090).
    /// The top app runs the command through its selection path.
    ListBar(ListBarCommand),
    PlayPlaylistAt {
        track_id: i64,
        playlist_id: i64,
        playlist_position: i64,
    },
    OpenIndexFeedDetail {
        feed_guid: String,
        label: String,
    },
    /// Opens the publisher page of a Library album or track (ADR 0077
    /// packet 004). `publisher_feed_guid` is the value that the album's
    /// stored owned relationship names, never name text.
    OpenPublisherPage {
        publisher_feed_guid: String,
    },
    /// Opens the feed page of a MusicIndex feed row of the Artists list,
    /// with the MusicIndex context, directly under its destination (ADR
    /// 0084 packet 016 decision D14).
    OpenIndexPublisherPage {
        publisher_feed_guid: String,
    },
    /// Opens the Library album page of a feed id, from the album name link
    /// of a track page (ADR 0083 Decision 5).
    OpenAlbumPage {
        feed_id: i64,
    },
    /// Opens a Library page in the one Music navigation stack (ADR 0084
    /// Decision 9). The Library keeps no navigation state: the top app
    /// changes the stack, then restores the page.
    OpenPage {
        entry: FrameNavigationEntry,
        start: NavigationPathStart,
    },
    /// A sidebar row was clicked (ADR 0089 Decision 1). The sidebar view
    /// lives in the Library, but Show and Settings are screen mounts, so
    /// the top app routes the row.
    SidebarRowSelected(crate::view_models::sidebar::SidebarDestination),
    /// An Index job of the download queue completed (ADR 0084 Decision
    /// 10, packet 022). The top app updates its open pages in place and
    /// starts the tag scan of a successful download.
    IndexDownloadFinished {
        job: DownloadJob,
        outcome: DownloadOutcome,
    },
    /// A request of the Search page (ADR 0089 Decision 4). The top app owns
    /// the toolbar field, so it applies the request.
    SearchPage(crate::view_models::search_page::SearchPageAction),
    /// Opens the Inspect Sources sheet of the open track page (ADR 0089
    /// Decision 8). The top app owns the window, so it opens the sheet.
    OpenInspectSourcesSheet,
    /// The repair action of a Setup item of the Inbox (ADR 0089 Decision
    /// 3). The top app runs it through its capability actions (ADR 0066).
    InboxSetupAction(crate::application::capability::CapabilityAction),
}

impl gpui::EventEmitter<LibraryAppEvent> for LibraryApp {}

#[derive(Clone, Debug, Default)]
pub(crate) enum LazyPanel<T> {
    #[default]
    Hidden,
    Loading,
    Empty(String),
    Loaded(T),
}

#[derive(Clone, Debug)]
pub(crate) struct InspectorFrame {
    pub(crate) entity_id: i64,
    pub(crate) title: String,
    pub(crate) track: TrackRow,
    pub(crate) source_context: Option<TrackContext>,
    pub(crate) image: Option<Arc<Image>>,
    pub(crate) expanded_id3_frame_groups: BTreeSet<String>,
    pub(crate) expanded_metadata_cells: BTreeSet<String>,
    pub(crate) pending_id3_edits: BTreeMap<String, PendingId3Edit>,
    pub(crate) suppressed_auto_id3_edits: BTreeSet<String>,
    pub(crate) applying_id3_edits: bool,
    pub(crate) id3_apply_error: Option<String>,
    pub(crate) local_subscription: bool,
    pub(crate) subscription_busy: bool,
    pub(crate) subscription_message: Option<String>,
    pub(crate) tag_compare: LazyPanel<TagCompareResult>,
    pub(crate) musicbrainz_lookup: LazyPanel<MusicBrainzLookupResult>,
    pub(crate) musicbrainz_selected: usize,
    pub(crate) inspector_state: LibraryTrackInspectorState,
}

impl InspectorFrame {
    pub(crate) fn inspector_display(
        &self,
        description: Option<&str>,
    ) -> LibraryTrackInspectorDisplay {
        let description = LibraryViewModel::display_description_text(description);
        let mut display = self
            .inspector_state
            .display(self.local_subscription && self.track.local_path.is_some());
        display.description_state = display
            .description_state
            .project_sticky(description_line_count(description));
        display
    }

    pub(crate) fn toggle_inspector_panel(&mut self, kind: InspectorPanelKind) -> bool {
        self.inspector_state.toggle_panel(kind);
        self.inspector_state.is_panel_expanded(kind)
    }

    pub(crate) fn toggle_description(&mut self) {
        self.inspector_state.toggle_description();
    }
}

#[derive(Clone)]
enum ThumbnailState {
    Loading,
    Loaded(Option<Arc<Image>>),
}

/// The main color of one cover (ADR 0083 task 005).
#[derive(Clone, Copy)]
enum CoverColorState {
    Loading,
    Loaded(Option<crate::media::cover_color::CoverColor>),
}

impl CollectionArtHost for LibraryApp {
    fn collection_art_parts(&mut self) -> CollectionArtParts<'_> {
        CollectionArtParts {
            loads: &mut self.collection_art,
            cache: &self.cache,
            runner: &self.command_runner,
        }
    }
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "each flag is one independent state of the Library surface"
)]
pub struct LibraryApp {
    conn: Arc<Mutex<Connection>>,
    application_services: Arc<ApplicationServices>,
    command_runner: AsyncCommandRunner,
    cache: Arc<ImageCache>,
    musicindex_endpoint: crate::config::MusicIndexEndpoint,
    music_dir: PathBuf,
    pub(crate) playback_availability:
        Result<(), crate::application::capability::ExecutionUnavailable>,
    /// Stateful screen view-model. Owns all pure UI state and loaded
    /// snapshots — tree, selection, expansion sets, sort orders,
    /// picker toggles, status, search query, playlists, MusicBrainz
    /// lookup state, feed-update workflow state. The fields kept on
    /// `LibraryApp` itself are GPUI-bound (Entity / Subscription),
    /// service handles, screen-only inspector state, or maps that
    /// still hold `Arc<gpui::Image>`. See ADR 0023.
    vm: LibraryViewModel,
    detail: LibraryDetail,
    thumbnails: BTreeMap<(String, bool), ThumbnailState>,
    cover_colors: BTreeMap<String, CoverColorState>,
    /// The load state of each collection art (ADR 0084 Decision 12).
    collection_art: CollectionArtLoads,
    new_playlist_input: Entity<InputState>,
    rename_playlist_input: Entity<InputState>,
    _rename_playlist_sub: gpui::Subscription,
    /// `true` while the list has the keyboard focus and the ring shows.
    list_row_focus_shows: bool,
    /// The one tab stop of the letter bar. One bar shows at a time.
    letter_bar_focus: FocusHandle,
    /// The keyboard cursor of the letter bar.
    letter_cursor: Option<crate::view_models::letter_bar::LetterKey>,
    /// `true` while the letter bar has the keyboard focus and the ring shows.
    letter_cursor_shows: bool,
    /// Optional async runtime host (ADR 0040). When present, screens
    /// can spawn paged-track-list actors that publish snapshots back
    /// via `presentation::bridge_watch`.
    pub(crate) runtime_host: Option<Arc<RuntimeHost>>,
    /// Currently spawned paged playlist actor for the active
    /// `LibraryDetail::Playlist`. Replaced on each `select_playlist`;
    /// dropping the previous handle closes the actor's inbox so it
    /// exits gracefully.
    pub(crate) playlist_actor: Option<PlaylistActorState>,
    /// Feed-level MusicBrainz lookup saga actor. Dropping the handle
    /// closes the inbox and lets the runtime task exit.
    musicbrainz_feed_saga: Option<MusicBrainzFeedSagaHandle>,
    /// Playlist RSS check actor (ADR 0076 Decision 2). Dropping the handle
    /// closes the inbox and lets the runtime task exit.
    playlist_rss_check: Option<crate::runtime::PlaylistRssCheckHandle>,
    /// Latest RSS check snapshot. `None` while no check actor runs.
    playlist_rss_snapshot: Option<crate::runtime::PlaylistRssCheckSnapshot>,
    /// Tag update actor (ADR 0076 Decision 8). Dropping the handle closes
    /// the inbox and lets the runtime task exit.
    tag_update: Option<crate::runtime::TagUpdateHandle>,
    /// Latest tag update snapshot. `None` while no tag update actor runs.
    tag_update_snapshot: Option<crate::runtime::TagUpdateSnapshot>,
    /// `true` when the actor did not accept the latest confirm.
    tag_update_request_failed: bool,
    /// Existing Recent Feeds pager consumed by the default Music content region.
    recent_music_page: RecentFeedsPageVm,
    /// Whether the app asked the node for its sort capability (musicindex ADR
    /// 0053 R-LIST-8 item 4). The app asks once per session, until a read
    /// answers.
    index_sorts_requested: bool,
    /// The scroll of the Inbox list (ADR 0089 Decision 3). The Library holds
    /// it, so the offset survives a page opened over the list.
    inbox_scroll: ScrollHandle,
    /// The scroll of the Search page list (ADR 0089 Decision 4). The Library
    /// holds it, so the offset survives a page opened over the list.
    search_page_scroll: ScrollHandle,
    /// The scroll of the Playlists list (ADR 0089 Decision 1). The Library
    /// holds it, so the offset survives a page opened over the list.
    playlists_scroll: ScrollHandle,
    /// The keyboard cursor of the My playlists list, by playlist id (ADR 0092
    /// Decision 1). The id keeps the cursor on a playlist that moves. A change
    /// of destination clears it.
    playlists_selection: ListSelection<i64>,
    /// The request to keep the focused playlist row in view (ADR 0092
    /// Decision 1).
    playlists_reveal: ScrollReveal,
    /// The loaded MusicIndex artist name pages of the Artists scope
    /// "Everything" (packet 016 decision D9). They stay until the app
    /// stops.
    artist_name_page: IndexNameListPage<ArtistNameRow>,
    /// The loaded MusicIndex release name pages of the Releases scope
    /// "Everything" (packet 016 decision D9).
    release_name_page: IndexNameListPage<Feed>,
    /// The singles groups whose Releases list rows show their singles
    /// (ADR 0084 Decision 6). This is view state. It is not stored.
    expanded_singles_groups: BTreeSet<SinglesGroupKey>,
    /// The one download queue of the Library and the Index (ADR 0084
    /// Decision 10, packet 022). At most three downloads run at the same
    /// time, and never two of one release.
    download_queue: DownloadQueue,
    /// The Releases list selection (ADR 0084 Decision 10). A change of
    /// destination clears it. Required correction item 2, 2026-10-18:
    /// the focus and the anchor hold a [`ReleasesRowId`], a release row
    /// or a group row, distinct from the [`ReleaseKey`] members that the
    /// selected set holds.
    releases_selection: ListSelection<ReleaseKey, ReleasesRowId>,
    /// The Releases list's mode (ADR 0084 Decision 10 Amendment,
    /// 2026-10-19): `Normal` or `Selecting`, beside `releases_selection`.
    /// A change of destination resets it to `Normal`.
    releases_selection_mode: SelectionMode,
    /// The Tracks and Inbox lists' shared selection (ADR 0084 Decision 10).
    /// A change of destination clears it.
    track_selection: ListSelection<TrackKey>,
    /// The Tracks and Inbox lists' shared mode (ADR 0084 Decision 10
    /// Amendment, 2026-10-19), beside `track_selection`. A
    /// change of destination resets it to `Normal`.
    track_selection_mode: SelectionMode,
    /// The Releases list's held virtual list state (ADR 0084 Decision 10
    /// and task 037), so a keyboard focus move keeps the focused row in
    /// view.
    releases_list_state: MusicColumnList,
    /// The scroll of the Tiles view of the Releases list (ADR 0089 packet
    /// 005 Part C). The Library holds it, so the offset survives a change
    /// of view.
    releases_tiles_scroll: ScrollHandle,
    /// The reveal of the Tiles view of the Releases list (ADR 0084 Decision
    /// 10, ADR 0089 packet 005 Part D). It scrolls the Tiles grid, not the
    /// rows list.
    releases_tile_reveal: ScrollReveal,
    /// The element id of the item that the next reveal shows.
    releases_tile_target: Option<String>,
    /// The last build of the Releases list and its key (ADR 0084 task
    /// 037). `TopApp::render` reads the header action through `&self`, so
    /// the cache needs a `RefCell`.
    releases_list_cache: RefCell<ReleasesListCache>,
    /// The Artists list's virtual list state (ADR 0084 task 037, operator
    /// answer Q1 option B). The Artists list keeps no cache of its merge.
    artists_list_state: MusicColumnList,
    /// The scroll of the Tiles view of the Artists list (musicindex ADR 0053
    /// R-LIST-10). The Library holds it, so the offset survives a change of
    /// view.
    artists_tiles_scroll: ScrollHandle,
    /// The reveal of the Tiles view of the Artists list. It scrolls the
    /// Tiles grid to the first tile of a letter.
    artists_tile_reveal: ScrollReveal,
    /// The element id of the tile that the next reveal shows.
    artists_tile_target: Option<String>,
    /// The Public playlists list's virtual list state (ADR 0089 packet 009).
    public_playlists_list_state: MusicColumnList,
    /// The scroll of the Tiles view of the Public playlists list (ADR 0089
    /// packet 009). The Library holds it, so the offset survives a change
    /// of view.
    public_playlists_tiles_scroll: ScrollHandle,
    /// The pages of the Public playlists list, one for each sort (ADR 0089
    /// packet 009). The Index gives its rows in order, so the list keeps
    /// the request state of its current sort.
    public_playlist_list: IndexNameListPage<Feed>,
    /// The item page of the open public playlist (ADR 0089 packet 009).
    public_playlist_pages: PublicPlaylistPages,
    /// The Live list's virtual list state (ADR 0089 packet 011).
    live_list_state: MusicColumnList,
    /// The pages of the Live list (ADR 0089 packet 011). A new read keeps the
    /// loaded rows on view until its answer comes.
    live_list: IndexNameListPage<LiveItem>,
    /// The Music list column's own focus handle (ADR 0084 Decision 10).
    /// The selection keyboard actions act only while it, or a row inside
    /// it, carries the keyboard focus.
    list_column_focus: FocusHandle,
    /// The pending whole-selection removal confirmation (ADR 0084
    /// Decision 10).
    selection_removal: SelectionRemovalConfirmationState,
    /// The result of the whole-selection removal that runs, when one runs
    /// (ADR 0084 Decision 10).
    selection_removal_report: Option<SelectionRemovalReport>,
    /// The detail column selection, its mode and its page entry (ADR 0084
    /// Decision 10, packet 018). A page change clears it.
    detail_selection: DetailSelection,
    /// The Music detail column's own focus handle (packet 018). The
    /// detail selection keys act only while it carries the focus.
    detail_column_focus: FocusHandle,
    /// The focus handle of each sidebar row (ADR 0089 Decision 1). A
    /// section shortcut moves the focus to the row of its section.
    sidebar_focus: Vec<(crate::view_models::sidebar::SidebarDestination, FocusHandle)>,
    /// The sidebar row of the current screen (ADR 0089 Decision 1). The
    /// top app sets it at each render.
    sidebar_selected: Option<crate::view_models::sidebar::SidebarDestination>,
    /// The Library track whose track page opens its Inspect Sources sheet
    /// when it mounts (ADR 0083 task 006, decision D4; ADR 0089 Decision 8).
    pending_inspect_sources: Option<i64>,
    /// The spring-load timer of the sidebar row "My playlists" (ADR 0083
    /// task 008, decision D1).
    spring_load: crate::view_models::library::SpringLoadVm,
    /// The held scroll handle of each Music detail page (packet 018), so
    /// a key can scroll the page or keep the focused row in view.
    detail_column_scroll: ScrollHandle,
    /// The binding of `detail_column_scroll` to the open history entry
    /// (ADR 0084 Decision 9, amended on 2026-10-21).
    detail_scroll_binding: detail_selection::DetailScrollBinding,
    /// The request to keep the focused detail row in view (packet 018).
    detail_scroll_reveal: ScrollReveal,
    /// The pinned copy of the list bar of a group page (ADR 0090 Decision
    /// 5). It keeps the bar position that the last paint recorded.
    list_bar_pin: ListBarPin,
    /// The saved searches, in `id` order (ADR 0084 Decision 3, packet
    /// 012). The app reads them at start, and again after each save and
    /// each delete.
    saved_searches: Vec<crate::view_models::search_results::SavedSearch>,
    /// The saved search of the current location, for the open row of the
    /// Search page. The top app sets it at each render.
    open_saved_search: Option<crate::view_models::search_results::SavedSearchId>,
    /// The state of "How to Search" over a list with saved searches (ADR 0089
    /// Decision 4). The page changes it on each click.
    how_to_search: crate::view_models::search_page::HowToSearchDisclosure,
    /// The inline rename input of a saved search row (ADR 0089 Decision 4).
    saved_search_rename_input: Entity<InputState>,
    _saved_search_rename_sub: gpui::Subscription,
    /// The name input of the copy dialog of a public playlist (ADR 0089
    /// packet 010).
    public_playlist_copy_input: Entity<InputState>,
    _public_playlist_copy_input_sub: gpui::Subscription,
    /// The open copy dialog of a public playlist, or `None`.
    public_playlist_copy: Option<crate::library::public_playlists::PublicPlaylistCopyDialog>,
    /// The public playlist whose copy runs on the runtime (ADR 0089 packet
    /// 010). One copy runs at a time.
    copying_public_playlist: Option<String>,
    /// The report of the last copy to My playlists. It shows on the page of
    /// the copied playlist, until that page closes or the next copy starts.
    public_playlist_copy_report:
        Option<crate::view_models::public_playlists::PublicPlaylistCopyReport>,
    /// The saved search whose row shows the rename input.
    saved_search_renaming: Option<crate::view_models::search_results::SavedSearchId>,
    /// The message of the last saved search request, for example a missing
    /// saved search or a copy that already exists.
    saved_search_message: Option<String>,
    /// The count of new results of each saved search (ADR 0089 Decision 4,
    /// packet 007 Part C). The actor publishes it, and an open sets zero.
    saved_search_counts: std::collections::BTreeMap<
        crate::view_models::search_results::SavedSearchId,
        crate::view_models::search_results::NewResultCount,
    >,
    /// The actor that counts the new results of the saved searches. It is
    /// started at the first request of the Search page.
    saved_search_counts_handle:
        Option<crate::runtime::saved_search_counts::SavedSearchCountsHandle>,
    /// The start of the last request to the actor, for the interval of
    /// ADR 0089 Decision 4.
    saved_search_counts_requested_at: Option<std::time::Instant>,
    /// The page that Back left open over the current list root, for the
    /// marked row of the Music list (R-NAV-2 item 2). The top app sets it at
    /// each render.
    marked_entry: Option<FrameNavigationEntry>,
    /// `true` when the current Music location shows a page, so the detail
    /// column shows and the list column does not (ADR 0089 Decision 1).
    /// The top app sets it at each render, and the keyboard gates read it.
    music_page_open: bool,
    /// `true` after the once-per-session identity backfill starts (ADR
    /// 0084 task 034, decision D2). It starts after the first successful
    /// Library tree load of the session, and it never starts twice.
    album_identity_backfill_started: bool,
    /// The Library releases that still wait for the backfill, newest
    /// feed id first. Each completion pops and starts the next entry, so
    /// at most `ALBUM_IDENTITY_BACKFILL_CONCURRENCY` commands run at a
    /// time.
    album_identity_backfill_queue: VecDeque<(i64, String)>,
    /// One readiness load at a time (ADR 0084 Decision 10 as amended
    /// 2026-10-24, task 039, step 3).
    readiness_flight: single_flight::SingleFlight,
    /// `true` while an Inbox open waits for the next readiness result.
    readiness_open_pending: bool,
    /// The show playlist track ids of the last readiness load (task 039,
    /// step 2). The Inbox uses them without a database read on the main
    /// thread.
    inbox_source_track_ids: Vec<i64>,
    /// One Library tree load at a time (task 039, step 4).
    tree_flight: single_flight::SingleFlight,
    /// One playlist sidebar load at a time (task 039, step 4).
    playlist_flight: single_flight::SingleFlight,
    /// The steps that read the playlist list. Each one runs after the next
    /// load that ends with no waiting run.
    playlist_after: Vec<PlaylistContinuation>,
}

/// A step that reads the playlist list after a reload.
type PlaylistContinuation = Box<dyn FnOnce(&mut LibraryApp, &mut gpui::Context<LibraryApp>)>;

pub(crate) struct PlaylistActorState {
    pub(crate) playlist_id: i64,
    pub(crate) snapshot: PagedTrackListSnapshot,
    pub(crate) handle: ActorHandle<PagedTrackListMsg, PagedTrackListSnapshot>,
}

mod app_impl;
mod column_view;
mod detail_selection;
mod download_queue;
mod downloads;
mod live_page;
mod music_scopes;
mod public_playlists;
mod release_page;
mod saved_searches;
mod single_flight;
mod track_page;

pub(crate) use app_impl::playlist_options;
pub(crate) use column_view::{DownloadStatusView, LibraryColumn, LibraryColumnView};
pub(crate) use download_queue::{DownloadJob, DownloadOutcome};
pub(crate) use downloads::{
    api_track_from_view, index_release_job, index_release_job_by_guid, index_track_job,
    index_track_job_by_key,
};
