use std::collections::{BTreeMap, BTreeSet, VecDeque};

use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use super::download_queue::{DownloadJob, DownloadOutcome, JobIdentity};
use super::single_flight::{Finish, Run, SingleFlight};
use super::PlaylistActorState;
use super::{InspectorFrame, LazyPanel, LibraryApp, LibraryDetail, PlaylistDetail, ThumbnailState};
use crate::app::keyboard::primary_modifier_held;
use crate::application::commands::download::{SubscribeThenAppendToPlaylist, SubscribeTrack};
use crate::application::commands::feed::{
    ApplyFeedUpdates, ApplyFeedUpdatesResult, CheckFeedStaleness, CheckFeedStalenessResult,
    CheckFeedsAndRepairRoutes, CheckFeedsAndRepairRoutesResult, SubscribeFeed,
};
use crate::application::commands::library_removal::RemoveFromLibrary;
use crate::application::commands::metadata::{ApplyTrackId3Edits, LookupMusicBrainzTrack};
use crate::application::commands::payment_routes::{
    PaymentRouteRepairStatus, PaymentRouteRepairTrackResult, RepairPaymentRoutesForTrack,
};
use crate::application::commands::playlist::{
    ConfirmRemovedTrack, CreatePlaylist, DeletePlaylist, MovePlaylistEntries,
    RemovePlaylistEntries, RemovePlaylistTrackAt, RemoveTrackFromAllPlaylists, RenamePlaylist,
    ReorderPlaylistTrack,
};
use crate::application::errors::command::CommandError;
use crate::application::library_removal::{
    removal_referring_playlist_names, LibraryRemovalIntent, LibraryRemovalTarget,
};
use crate::application::queries::broadcast::{
    BroadcastReadinessLoad, BroadcastReadinessReport, LoadBroadcastReadiness,
};
use crate::application::queries::feed::FetchRecentFeedsPage;
use crate::application::queries::images::{FetchCoverColor, FetchThumbnail};
use crate::application::queries::library::{
    AlbumHydrationTrigger, AlbumIdentityHydration, CompareLibraryTrack, FetchLibraryTrackContext,
    HydrateAlbumIdentity, LibraryTrackCompare, LibraryTracksTree, LoadLibraryTracksTree,
};
use crate::application::queries::playlist::{LoadPlaylistSidebar, PlaylistSidebarLoad};
use crate::application::{ApplicationServices, AsyncCommandRunner, CommandContext};
use crate::db::{self, TrackRow};
use crate::feed_service::track_row_to_track_context;
use crate::library_path::LibraryRelativePath;
use crate::media::ImageCache;
use crate::metadata::{
    auto_populated_pending_id3_edits, pending_id3_conflict_descriptions,
    pending_id3_edits_for_apply, MusicBrainzLookupResult,
};
use crate::presentation::collection_art::{collection_art_image, CollectionArtLoads};
use crate::presentation::delayed_presenter::present_after;
use crate::presentation::{bridge_watch, present_command};
use crate::runtime::musicbrainz_feed_saga::{MusicBrainzFeedSagaState, StartFeedLookup};
use crate::subscribe_service::{self, SubscribeFeedRequest, SubscribeTrackRequest};
use crate::ui::composites::inspect_sources_sheet::InspectSourcesSheet;
use crate::ui::composites::letter_bar::LetterBarKeys;
use crate::ui::composites::letter_bar::LetterHandler;
use crate::ui::composites::list_page_head::{
    list_page_head, ListHeadButtonHandler, ListPageHeadHandlers, ListReverseHandler,
    ListScopeHandler, ListSortHandler, ListViewHandler,
};
use crate::ui::composites::list_row::RowMenu;
use crate::ui::composites::music_column_scroll::{
    music_column_scrollbar, ListBarPin, MusicColumnList, ScrollReveal,
};
use crate::ui::composites::page_scroll_content::page_scroll_content;
use crate::ui::composites::playlist_popover::{CreateHandler, SelectHandler};
use crate::ui::composites::release_row_menu::ReleaseRowMenuContext;
use crate::ui::composites::track_row_menu::{MenuActionHandler, TrackRowMenuHandlers};
use crate::ui::composites::{
    CollectionArtImage, PlaylistOption, PlaylistOptionDisplay, StatusRole, ThumbnailSize,
};
use crate::ui::control_styles::ControlStyle;
use crate::ui::primitives::{Button as UiButton, ContextMenuItem, ContextMenuItemDisplay};
use crate::ui::shells::library::content_list::{
    render_library_content_list, ContentListMenuContext,
};
use crate::ui::shells::library::destination_lists::{
    render_artists_list_item, render_library_destination_rows, render_library_playlists_list,
    render_library_releases_list, render_releases_list_item, LeadingDisclosure, RowArts,
};
use crate::ui::shells::library::detail::render_library_detail;
use crate::ui::shells::library::inbox::render_inbox_sections;
use crate::ui::shells::library::index_list_status::IndexListActions;
use crate::ui::shells::library::releases_tiles::{
    render_releases_list_tile, render_releases_tile_grid,
};
use crate::ui::shells::library::search_page_list::{render_search_page_list, SearchPageListRename};
use crate::ui::shells::library::sidebar::{render_sidebar_row, render_sidebar_separator};
use crate::ui::shells::library::track_detail_metadata::{
    render_library_inspect_sources_sheet_body, track_metadata_rows_for_frame,
};
use crate::ui::shells::library_removal_confirmation::{
    open_library_removal_confirmation_dialog, open_selection_removal_confirmation_dialog,
};
use crate::ui::shells::playlist_delete_confirmation::open_playlist_delete_dialog;
use crate::ui::shells::tag_update_confirmation::{
    open_tag_update_confirmation_dialog, render_tag_update_button, render_tag_update_report,
};
use crate::ui::style::color;
use crate::ui::style::spacing;
use crate::ui::tokens::Motion;
use crate::view_models::artist_identity::{ArtistNameDestination, ArtistNameKey, NameFeedMatches};
use crate::view_models::detail_selection::DetailSelection;
use crate::view_models::entity_detail::TrackMetadataActionState;
use crate::view_models::inbox::InboxSection;
use crate::view_models::index_name_list::IndexNameListPage;
use crate::view_models::library::{
    description_line_count, list_column_header_action, releases_disclosure_step, AlbumNode,
    BroadcastRouteRepairCompletion, BroadcastRouteRepairCompletionStatus, ContentListRowActionKind,
    ContentListSelectionTarget, DestinationListRowTarget, FeedCheckRouteRepairOutcome,
    FeedUpdateActionDisplay, FeedUpdateActionKind, FeedUpdateDisplay, FeedUpdatePhase,
    InspectorPanelKind, LibrarySearchLookup, LibraryTrackActionVm, LibraryTrackInspectorState,
    LibraryTrackRowVm, LibraryTree, LibraryViewModel, MbTrackStatus, PlaylistAppendIntent,
    PlaylistAppendOutcome, PlaylistDetailActionsDisplay, ReleasesDisclosureStep,
    ReleasesListItemVm, ReleasesRowId, RemovalConfirmation, TrackSubscribeOutcome,
};
use crate::view_models::library::{
    name_level_art, spring_load_navigation, take_inspect_sources_request, ArtistsListItemVm,
    SpringLoadGeneration,
};
use crate::view_models::library::{PlaylistDeleteConfirmation, PlaylistSidebarVm};
use crate::view_models::library_removal::{
    SelectionRemovalConfirmationState, SelectionRemovalPendingItem, SelectionRemovalReport,
};
use crate::view_models::library_search_facts::LibrarySearchFacts;
use crate::view_models::list_page::{ListPageHeadVm, ListPageKey, ListScope, ListSort, ListView};
use crate::view_models::list_selection::{keyboard_entry_step, list_is_tab_stop};
use crate::view_models::list_selection::{FocusStep, ListSelection, SelectionMode, SelectionRow};
use crate::view_models::name_level_page::NameLevelLibraryRelease;
use crate::view_models::playlist_option_displays;
use crate::view_models::public_playlists::PublicPlaylistPages;
use crate::view_models::recent_feeds::{
    RecentFeedsLoadIntent, RecentFeedsPageStep, RecentFeedsPageVm,
};
use crate::view_models::release_kind::ReleaseKind;
use crate::view_models::release_page::TrackRowMenuAction;
use crate::view_models::release_row_actions::{
    ReleaseRowCommand, ReleaseRowMenuVm, ReleaseRowTarget,
};
use crate::view_models::releases_everything::{ReleasesDownloadPlan, ReleasesListRows};
use crate::view_models::search_field_values::LibraryFieldValues;
use crate::view_models::search_page::SearchPageAction;
use crate::view_models::search_page::{HowToSearchDisclosure, SearchPageVm};
use crate::view_models::search_results::ArtistSearchFacts;
use crate::view_models::search_results::{SearchPageActionDisplay, SearchPageActionStyle};
use crate::view_models::selection_action_bar::{SelectionActionBarVm, SelectionActionKind};
use crate::view_models::singles_group::{
    SinglesGroupKey, SinglesGroupPageEntryVm, SinglesGroupPageVm,
};
use crate::view_models::track_detail::TrackNameLinkTarget;
use crate::view_models::window_groups::WindowGroupKey;
use crate::view_models::workspace::{
    FrameHeaderActionDisplay, FrameNavigationEntry, MusicDestination, NavigationPathStart,
    ReleaseKey, TrackKey,
};
use crate::views::{EntityIdentityLinks, LocalIdentityFacts};
use gpui::{
    div, prelude::*, AnyElement, App, Context, Entity, FontWeight, Image, InteractiveElement,
    IntoElement, Modifiers, SharedString, Styled, Window,
};
use gpui_component::input::{InputEvent, InputState};
use rusqlite::Connection;

/// The message of a failed Recent music page (ADR 0086 Decision 5). The
/// Releases list shows it with the detail of the failure.
const RECENT_MUSIC_FAILED: &str = "Recent music unavailable";

impl InspectorFrame {
    fn for_track(track: TrackRow, image: Option<Arc<Image>>) -> Self {
        let title = LibraryTrackRowVm::new(&track, None).display_title();
        let local_subscription = track.is_in_library && track.local_path.is_some();
        Self {
            entity_id: track.id,
            title,
            local_subscription,
            track,
            source_context: None,
            image,
            expanded_id3_frame_groups: BTreeSet::new(),
            expanded_metadata_cells: BTreeSet::new(),
            pending_id3_edits: BTreeMap::new(),
            suppressed_auto_id3_edits: BTreeSet::new(),
            applying_id3_edits: false,
            id3_apply_error: None,
            subscription_busy: false,
            subscription_message: None,
            tag_compare: LazyPanel::Hidden,
            musicbrainz_lookup: LazyPanel::Hidden,
            musicbrainz_selected: 0,
            inspector_state: LibraryTrackInspectorState::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LibraryReloadMode {
    ResetDetail,
    PreserveDetail,
}

#[derive(Clone, Debug)]
enum TrackSubscriptionAction {
    Download(Box<TrackRow>),
    Remove(i64),
}

pub(super) fn command_error_detail(error: CommandError) -> String {
    match error {
        CommandError::Unavailable(reason) => reason.to_string(),
        CommandError::ObservationWriteFailure(failure) => failure.to_string(),
        CommandError::ObservedQueryFailure(failure) => failure.message().to_owned(),
        CommandError::ObservedCommandFailure(failure) => {
            command_error_detail(failure.cause().clone())
        }
        error @ (CommandError::SessionDraining(_) | CommandError::HostThrottled { .. }) => {
            error.to_string()
        }
        CommandError::Playlist(message)
        | CommandError::Feed(message)
        | CommandError::Download(message)
        | CommandError::Metadata(message)
        | CommandError::Playback(message)
        | CommandError::Query(message)
        | CommandError::QueryRejected(message)
        | CommandError::Other(message) => message,
        CommandError::Cancelled => "command cancelled".to_string(),
    }
}

/// ADR 0075 consumes query evidence before any selected-frame check.
fn retain_library_query_failure(vm: &mut LibraryViewModel, error: &CommandError) {
    if matches!(error, CommandError::ObservationWriteFailure(_)) {
        vm.retain_observation_failure(error);
    } else {
        vm.retain_query_evidence(error);
    }
}

fn apply_library_comparison_result(
    vm: &mut LibraryViewModel,
    detail: &mut LibraryDetail,
    entity_id: i64,
    result: LibraryTrackCompare,
) {
    vm.retain_observation_receipts(&result.track_context.observation_receipts);
    if let LibraryDetail::Track(frame) = detail {
        if frame.entity_id == entity_id {
            frame.source_context = Some(result.track_context);
            frame.tag_compare = LazyPanel::Loaded(result.tag_compare);
        }
    }
}

fn apply_library_comparison_failure(
    vm: &mut LibraryViewModel,
    detail: &mut LibraryDetail,
    entity_id: i64,
    error: CommandError,
) {
    retain_library_query_failure(vm, &error);
    if let LibraryDetail::Track(frame) = detail {
        if frame.entity_id == entity_id {
            frame.tag_compare =
                LazyPanel::Empty(LibraryViewModel::deferred_panel_error_message(error));
        }
    }
}

fn apply_library_hydration_result(
    vm: &mut LibraryViewModel,
    detail: &mut LibraryDetail,
    feed_id: i64,
    hydration: AlbumIdentityHydration,
) {
    vm.retain_observation_receipts(&hydration.observation_receipts);
    vm.update_album_identity_facts(feed_id, &hydration.identity_facts);
    vm.update_album_metadata_facts(feed_id, &hydration.metadata_facts);
    vm.update_album_stored_values(feed_id, &hydration.stored_values);
    vm.update_album_description(feed_id, hydration.description.as_deref());
    // ADR 0084 task 034: the singles join rule and the Artists list
    // image read `AlbumNode::named_feeds`, so the tree needs this field
    // too, not only the identity facts.
    vm.update_album_named_feeds(feed_id, &hydration.named_feeds);
    if let LibraryDetail::Album(album) = detail {
        if album.feed_id == Some(feed_id) {
            album.identity_facts = hydration.identity_facts;
            *album.metadata_facts = hydration.metadata_facts;
            album.stored_values = Some(Box::new(hydration.stored_values));
            album.description = hydration.description;
            album.named_feeds = hydration.named_feeds;
        }
    }
}

/// The Library releases that the identity backfill pass can hydrate,
/// newest feed id first (ADR 0084 task 034, decision D2). Only an album
/// with both a feed id and a feed GUID is a candidate: the pass has
/// nothing to read or hydrate otherwise. A higher `feed_id` is a more
/// recently subscribed Library row, so sorting by it descending gives
/// the stated "newest first" order with no extra read.
fn album_identity_backfill_order(tree: &LibraryTree) -> Vec<(i64, String)> {
    let mut candidates: Vec<(i64, String)> = tree
        .artists
        .iter()
        .flat_map(|artist| artist.albums.iter())
        .filter_map(|album| Some((album.feed_id?, album.feed_guid.clone()?)))
        .collect();
    candidates.sort_by_key(|&(feed_id, _)| std::cmp::Reverse(feed_id));
    candidates
}

/// ADR 0075 retains feed-view evidence before the stale entry reduces into state.
fn apply_feed_view_check_result(vm: &mut LibraryViewModel, result: CheckFeedStalenessResult) {
    vm.retain_observation_receipts(result.observation_receipts());
    let checked_feed_id = result.feed_id();
    vm.finish_feed_view_check(checked_feed_id, Ok(result.into_stale()));
}

/// ADR 0075 retains combined-check evidence before any result reduction.
fn retain_feed_check_evidence(
    vm: &mut LibraryViewModel,
    outcome: &CheckFeedsAndRepairRoutesResult,
) -> FeedCheckRouteRepairOutcome {
    vm.retain_observation_receipts(outcome.observation_receipts());
    feed_check_route_repair_outcome(outcome)
}

/// ADR 0075 retains update evidence before the completion message reduces.
fn apply_feed_updates_result(vm: &mut LibraryViewModel, result: &ApplyFeedUpdatesResult) {
    vm.retain_observation_receipts(result.observation_receipts());
    vm.finish_apply_feed_updates(result.message().to_string());
}

fn feed_check_route_repair_outcome(
    outcome: &CheckFeedsAndRepairRoutesResult,
) -> FeedCheckRouteRepairOutcome {
    let summary = &outcome.route_repairs().summary;
    let feed_update_failures = outcome
        .feed_updates()
        .map_or(0, |updates| updates.feed_errors().len());
    FeedCheckRouteRepairOutcome::new(
        outcome.feeds_checked(),
        outcome.stale_feed_count(),
        feed_update_failures,
        summary.repaired,
        summary.no_routes_upstream,
        summary.failed,
    )
}

fn broadcast_route_repair_completion(
    result: &PaymentRouteRepairTrackResult,
) -> BroadcastRouteRepairCompletion {
    let status = match &result.status {
        PaymentRouteRepairStatus::Repaired => BroadcastRouteRepairCompletionStatus::Repaired,
        PaymentRouteRepairStatus::NoRoutesUpstream => {
            BroadcastRouteRepairCompletionStatus::NoRoutesUpstream
        }
        PaymentRouteRepairStatus::Failed => BroadcastRouteRepairCompletionStatus::Failed {
            reason: result
                .reason
                .clone()
                .unwrap_or_else(|| "unknown error".into()),
        },
        PaymentRouteRepairStatus::Skipped => BroadcastRouteRepairCompletionStatus::Skipped,
    };
    BroadcastRouteRepairCompletion::new(result.track_id, result.title.clone(), status)
}

fn apply_library_removal_to_album_detail(detail: &mut LibraryDetail, target: LibraryRemovalTarget) {
    if let LibraryDetail::Album(album) = detail {
        match target {
            LibraryRemovalTarget::Track(track_id) => {
                if let Some(track) = album.tracks.iter_mut().find(|track| track.id == track_id) {
                    track.is_in_library = false;
                    track.local_path = None;
                }
            }
            LibraryRemovalTarget::Feed(feed_id) if album.feed_id == Some(feed_id) => {
                for track in &mut album.tracks {
                    track.is_in_library = false;
                    track.local_path = None;
                }
            }
            LibraryRemovalTarget::Feed(_) => {}
        }
    }
}

fn apply_track_subscription_to_album_detail(
    detail: &mut LibraryDetail,
    track_id: i64,
    path: Option<LibraryRelativePath>,
    marked_downloaded: bool,
) {
    if !marked_downloaded {
        return;
    }

    let LibraryDetail::Album(album) = detail else {
        return;
    };

    if let Some(track) = album.tracks.iter_mut().find(|track| track.id == track_id) {
        track.is_in_library = true;
        track.local_path = path;
    }
}

fn api_feed_from_album(album: &AlbumNode) -> crate::api::Feed {
    crate::api::Feed {
        feed_guid: album.feed_guid.clone(),
        feed_url: album.feed_url.clone(),
        title: Some(album.name.clone()),
        description: album.description.clone(),
        image_url: album.image_href.clone(),
        tracks: Some(
            album
                .tracks
                .iter()
                .map(subscribe_service::track_row_to_api_track)
                .collect(),
        ),
        ..crate::api::Feed::default()
    }
}

fn album_thumbnail_urls(album: &AlbumNode) -> Vec<String> {
    album
        .image_href
        .iter()
        .chain(
            album
                .stored_values
                .iter()
                .flat_map(|values| values.credits.iter())
                .filter_map(|contributor| contributor.image_url.as_ref()),
        )
        .chain(album.tracks.iter().filter_map(|track| {
            track
                .track_image_href
                .as_ref()
                .or(track.album_image_href.as_ref())
        }))
        .cloned()
        .collect()
}

fn apply_library_removal_to_inspector_frame(
    frame: &mut InspectorFrame,
    target: LibraryRemovalTarget,
) {
    let message = match target {
        LibraryRemovalTarget::Track(track_id) if frame.entity_id == track_id => {
            Some("Removed track")
        }
        LibraryRemovalTarget::Feed(feed_id) if frame.track.feed_id == feed_id => {
            Some("Removed feed")
        }
        LibraryRemovalTarget::Track(_) | LibraryRemovalTarget::Feed(_) => None,
    };

    if let Some(message) = message {
        reset_removed_inspector_frame(frame, message);
    }
}

fn reset_removed_inspector_frame(frame: &mut InspectorFrame, message: &'static str) {
    frame.subscription_busy = false;
    frame.local_subscription = false;
    frame.track.is_in_library = false;
    frame.track.local_path = None;
    frame.source_context = None;
    frame.tag_compare = LazyPanel::Hidden;
    frame.pending_id3_edits.clear();
    frame.suppressed_auto_id3_edits.clear();
    frame.id3_apply_error = None;
    frame.subscription_message = Some(message.into());
}

// ---------------------------------------------------------------------------
// LibraryApp
// ---------------------------------------------------------------------------

impl LibraryApp {
    /// Starts one page request of New releases: the fresh load, one "Load
    /// more" click or one scroll probe (ADR 0086 Decision 5, packet 003
    /// decision D2).
    pub(crate) fn start_recent_music_load(&mut self, append: bool, cx: &mut Context<Self>) {
        let Some(intent) = self.recent_music_page.begin_load(append) else {
            return;
        };
        let reverse = self.vm.list_index_reversed(ListPageKey::Releases);

        cx.notify();
        self.dispatch_recent_music_page(intent, reverse, cx);
    }

    /// Sends one page of the current page request. After the page arrives,
    /// the pager decides if an extra page loads (ADR 0086 Decision 5). The
    /// result reaches the content list only when the chain stops.
    fn dispatch_recent_music_page(
        &mut self,
        intent: RecentFeedsLoadIntent,
        reverse: bool,
        cx: &mut Context<Self>,
    ) {
        let generation = intent.generation();
        let resume_after = intent.resume_after();
        let command = FetchRecentFeedsPage::new(
            self.musicindex_endpoint.clone(),
            intent.into_cursor(),
            resume_after,
            reverse,
        );
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, batch, cx| {
                if !this.recent_music_page.receive_page(generation, batch) {
                    return;
                }
                let windows_open = this.vm.recent_music_windows_open(&this.recent_music_page);
                match this.recent_music_page.next_step(windows_open) {
                    RecentFeedsPageStep::LoadExtra(intent) => {
                        this.dispatch_recent_music_page(intent, reverse, cx);
                    }
                    RecentFeedsPageStep::Finished => cx.notify(),
                }
            },
            move |this, error, cx| {
                let detail = command_error_detail(error);
                if this
                    .recent_music_page
                    .fail_page(generation, RECENT_MUSIC_FAILED, detail)
                {
                    cx.notify();
                }
            },
        );
    }

    pub(crate) fn open_content_list_row(&mut self, row_id: &str, cx: &mut Context<Self>) {
        if let Some(selection) = self.vm.content_list_index_feed_selection(row_id) {
            cx.emit(super::LibraryAppEvent::OpenIndexFeedDetail {
                feed_guid: selection.feed_guid,
                label: selection.label,
            });
            return;
        }
        // ADR 0084 Decision 4: a Library track row in the content list
        // opens its track page in the current path.
        if let Some(key) = self.vm.content_list_track_row_key(row_id) {
            cx.emit(super::LibraryAppEvent::OpenPage {
                entry: FrameNavigationEntry::Track(key),
                start: NavigationPathStart::ListRow,
            });
        }
    }

    /// Opens the page a Releases or Artists destination-list row names
    /// (ADR 0084 Decision 2), directly under its destination (Decision 9).
    pub(crate) fn open_destination_row(
        &mut self,
        target: &DestinationListRowTarget,
        cx: &mut Context<Self>,
    ) {
        let entry = match target {
            DestinationListRowTarget::Release(key) => FrameNavigationEntry::Release(key.clone()),
            // ADR 0084 Decision 7: a name row opens the name level page.
            // A row whose name matches one feed has no row.
            DestinationListRowTarget::ArtistName(key) => {
                FrameNavigationEntry::ArtistName(key.clone())
            }
            // ADR 0085 Decisions 2 and 6: a feed row opens the feed page.
            // With no recorded context for this GUID, the restore falls
            // back to the Library query (ADR 0077 packet 004).
            DestinationListRowTarget::Publisher(feed_guid) => {
                FrameNavigationEntry::PublisherDetail(feed_guid.clone())
            }
            // ADR 0084 packet 016 decision D14: a MusicIndex feed row opens
            // the feed page with the MusicIndex context. The top app
            // records the context before it restores the page.
            DestinationListRowTarget::IndexPublisher(feed_guid) => {
                cx.emit(super::LibraryAppEvent::OpenIndexPublisherPage {
                    publisher_feed_guid: feed_guid.clone(),
                });
                return;
            }
            // ADR 0089 packet 009: a public playlist row opens its page.
            DestinationListRowTarget::PublicPlaylist(feed_guid) => {
                FrameNavigationEntry::PublicPlaylist(feed_guid.clone())
            }
        };
        self.open_page(entry, NavigationPathStart::ListRow, cx);
    }

    /// The destination whose list the list column currently shows (ADR
    /// 0084 Decision 2), for the top app's own content-list chrome.
    pub(crate) fn active_destination(&self) -> MusicDestination {
        self.vm.active_destination()
    }

    /// The hovered Releases-row thumbnail url, when the pointer is over
    /// one (ADR 0084 Decision 2). `render_list_column` reads this to
    /// animate the matching thumbnail, the same way the Library sidebar
    /// tree once did.
    pub(crate) fn hovered_thumb_url(&self) -> Option<&str> {
        self.vm.hovered_thumb_url()
    }

    /// Records the hovered Releases-row thumbnail url (ADR 0084 Decision
    /// 2).
    pub(crate) fn set_hovered_thumb(&mut self, url: Option<String>, cx: &mut Context<Self>) {
        if self.vm.set_hovered_thumb_url(url) {
            cx.notify();
        }
    }

    /// Opens the publisher page of an album or a track's album feed (ADR
    /// 0077 packet 004). The top app owns the publisher page fetch and its
    /// navigation entry.
    /// Opens the page of a name link under a Library track title (ADR 0083
    /// Decision 5).
    pub(crate) fn open_track_name_link(
        &mut self,
        target: &TrackNameLinkTarget,
        cx: &mut Context<Self>,
    ) {
        match target {
            TrackNameLinkTarget::LibraryAlbum(feed_id) => {
                cx.emit(super::LibraryAppEvent::OpenAlbumPage { feed_id: *feed_id });
            }
            TrackNameLinkTarget::IndexAlbum { feed_guid } => {
                cx.emit(super::LibraryAppEvent::OpenIndexFeedDetail {
                    feed_guid: feed_guid.clone(),
                    label: feed_guid.clone(),
                });
            }
            TrackNameLinkTarget::Publisher(publisher_feed_guid) => {
                self.open_publisher_page(publisher_feed_guid, cx);
            }
        }
    }

    /// Opens the page of a name link under a Library album title.
    pub(crate) fn open_album_name_link(
        &mut self,
        target: &crate::view_models::album_page::AlbumNameLinkTarget,
        cx: &mut Context<Self>,
    ) {
        match target {
            // ADR 0084 Decision 7: the artist name opens the feed page of
            // the one feed that the name matches, else the name level page,
            // in the current path.
            crate::view_models::album_page::AlbumNameLinkTarget::Artist(name) => {
                match self
                    .vm
                    .name_feed_matches()
                    .destination(ArtistNameKey::from_text(name))
                {
                    ArtistNameDestination::FeedPage(feed_guid) => {
                        self.open_publisher_page(&feed_guid, cx);
                    }
                    ArtistNameDestination::NameLevel(key) => self.open_page(
                        FrameNavigationEntry::ArtistName(key),
                        NavigationPathStart::CurrentPath,
                        cx,
                    ),
                }
            }
            crate::view_models::album_page::AlbumNameLinkTarget::Publisher(publisher_feed_guid) => {
                self.open_publisher_page(publisher_feed_guid, cx);
            }
            // ADR 0084 Decision 6: the link of a single page opens its
            // group page in the current path.
            crate::view_models::album_page::AlbumNameLinkTarget::SinglesGroup(key) => {
                self.open_page(
                    FrameNavigationEntry::SinglesGroup(key.clone()),
                    NavigationPathStart::CurrentPath,
                    cx,
                );
            }
        }
    }

    /// Opens the window group page of `key` in the current path (ADR 0086
    /// Decision 4). A click on a group tile of New releases calls it.
    pub(crate) fn open_window_group(&mut self, key: &WindowGroupKey, cx: &mut Context<Self>) {
        self.open_page(
            FrameNavigationEntry::WindowGroup(key.clone()),
            NavigationPathStart::CurrentPath,
            cx,
        );
    }

    /// Opens the page of a singles group from its row in the Releases list
    /// (ADR 0084 Decisions 6 and 9). A list row starts the path at its
    /// destination, the same as a release row. The "All n singles" link of
    /// a single page keeps the current path.
    pub(crate) fn open_singles_group(&mut self, key: &SinglesGroupKey, cx: &mut Context<Self>) {
        self.open_page(
            FrameNavigationEntry::SinglesGroup(key.clone()),
            NavigationPathStart::ListRow,
            cx,
        );
    }

    /// Opens or closes a singles group row of the Releases list in place
    /// (ADR 0084 Decision 6). It does not navigate.
    pub(crate) fn toggle_singles_group(&mut self, key: &SinglesGroupKey, cx: &mut Context<Self>) {
        if !self.expanded_singles_groups.remove(key) {
            self.expanded_singles_groups.insert(key.clone());
        }
        cx.notify();
    }

    /// The name links of a release page with the link to the singles group
    /// of a single, when the release `key` is in one (ADR 0084 Decision 6,
    /// packet 026 decision D14).
    pub(crate) fn release_page_name_links(
        &self,
        base: Vec<crate::view_models::album_page::AlbumNameLinkVm>,
        key: Option<&ReleaseKey>,
    ) -> Vec<crate::view_models::album_page::AlbumNameLinkVm> {
        self.vm.release_page_name_links(base, key)
    }

    /// The page of a singles group, from the current tree and the loaded
    /// `MusicIndex` singles (ADR 0084 Decision 6, packet 026 decision D12).
    pub(crate) fn singles_group_page(&self, key: &SinglesGroupKey) -> SinglesGroupPageVm {
        self.vm.singles_group_page(key)
    }

    /// The match of each name key to the artist feeds of the Library (ADR
    /// 0084 Decision 7), from the current tree.
    pub(crate) fn name_feed_matches(&self) -> NameFeedMatches {
        self.vm.name_feed_matches()
    }

    /// The Library releases of the name `key` (ADR 0084 Decision 7), from
    /// the current tree.
    pub(crate) fn name_level_members(&self, key: &str) -> Vec<NameLevelLibraryRelease> {
        self.vm.name_level_members(key)
    }

    /// The name keys that match exactly the feed `feed_guid` (ADR 0084
    /// Decision 7, "Matched by name"), from the current tree.
    pub(crate) fn matched_name_keys(&self, feed_guid: &str) -> Vec<ArtistNameKey> {
        self.vm.matched_name_keys(feed_guid)
    }

    /// The feed GUID of each Library release whose identity is not one of
    /// the names `keys`, from the current tree.
    pub(crate) fn other_identity_feed_guids(&self, keys: &[ArtistNameKey]) -> BTreeSet<String> {
        self.vm.other_identity_feed_guids(keys)
    }

    /// The Library facts of the search rows for one render (ADR 0084
    /// packet 010), from the current tree.
    pub(crate) fn search_lookup(&self) -> LibrarySearchLookup<'_> {
        self.vm.search_lookup()
    }

    /// The Library facts of the search fields for one run (ADR 0087 packet
    /// 002): the release kind and the dates of each tree track, and the
    /// ready set of the last readiness report.
    pub(crate) fn library_search_facts(&self) -> LibrarySearchFacts {
        self.vm.library_search_facts()
    }

    /// The Library facts that the artist rows of a search read (ADR 0084
    /// Decision 7, packet 017 decisions D6 and D7). `feed_images` carries
    /// the channel image of each artist feed (ADR 0084 Decision 12), so a
    /// feed row of a search shows the same image as the Artists list.
    pub(crate) fn artist_search_facts(&self) -> ArtistSearchFacts {
        let feed_images = self
            .vm
            .tree()
            .feed_images()
            .entries()
            .into_iter()
            .map(|(guid, href)| (guid.to_owned(), href.to_owned()))
            .collect();
        ArtistSearchFacts {
            matches: self.vm.name_feed_matches(),
            name_level_art: self
                .vm
                .name_level_members_by_key()
                .into_iter()
                .map(|(key, members)| (key, name_level_art(&members)))
                .collect(),
            feed_images,
        }
    }

    /// The values of `artist:`, `release:` and `playlist:` for the field
    /// list of the toolbar search field (ADR 0087 Decision 3, packet 005).
    #[must_use]
    pub(crate) fn search_field_values(&self) -> Arc<LibraryFieldValues> {
        self.vm.field_values()
    }

    pub(crate) fn open_publisher_page(
        &mut self,
        publisher_feed_guid: &str,
        cx: &mut Context<Self>,
    ) {
        cx.emit(super::LibraryAppEvent::OpenPublisherPage {
            publisher_feed_guid: publisher_feed_guid.to_owned(),
        });
    }

    pub(crate) fn run_content_list_row_action(
        &mut self,
        action: ContentListRowActionKind,
        cx: &mut Context<Self>,
    ) {
        match action {
            ContentListRowActionKind::RepairBroadcastRoutes { track_id } => {
                self.repair_broadcast_routes_for_track(track_id, cx);
            }
            ContentListRowActionKind::ConfirmRemovedFromFeed { track_id } => {
                self.confirm_removed_track(track_id, cx);
            }
        }
    }

    /// Runs one item of the row menu of a Tracks or Inbox row
    /// (ADR 0083 task 007, decisions D9 and D10). `row_id` is the id of the
    /// row, which is the id of its Library track. "Add to playlist" opens
    /// a level of the menu, and the playlist items are items of a playlist
    /// row only, so this function ignores them.
    pub(crate) fn run_content_list_menu_action(
        &mut self,
        row_id: &str,
        action: TrackRowMenuAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Ok(track_id) = row_id.parse::<i64>() else {
            return;
        };
        self.run_track_row_menu_action(track_id, action, window, cx);
    }

    /// "Add to playlist" of the row menu of a Tracks or Inbox row
    /// (ADR 0083 task 007, decision D9).
    pub(crate) fn add_content_row_to_playlist(
        &mut self,
        row_id: &str,
        playlist_id: i64,
        cx: &mut Context<Self>,
    ) {
        if let Ok(track_id) = row_id.parse::<i64>() {
            self.add_track_to_playlist(track_id, playlist_id, cx);
        }
    }

    /// "New Playlist" of the row menu of a Tracks or Inbox row
    /// (ADR 0083 task 007, decision D9).
    pub(crate) fn create_playlist_for_content_row(
        &mut self,
        row_id: &str,
        name: &str,
        cx: &mut Context<Self>,
    ) {
        if let Ok(track_id) = row_id.parse::<i64>() {
            self.create_playlist_and_add_track(name, track_id, cx);
        }
    }

    /// Runs one item of the row menu of a playlist row (ADR 0083 task 007,
    /// decision D5). "Move up", "Move down" and "Remove from playlist" act
    /// on the entry at `position` of the playlist. Each other item acts on
    /// the Library track.
    pub(crate) fn run_playlist_row_menu_action(
        &mut self,
        entry: (i64, i64),
        track_id: i64,
        action: TrackRowMenuAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (playlist_id, position) = entry;
        match action {
            TrackRowMenuAction::MoveUp => {
                self.move_playlist_track(playlist_id, position, position - 1, cx);
            }
            TrackRowMenuAction::MoveDown => {
                self.move_playlist_track(playlist_id, position, position + 1, cx);
            }
            // ADR 0083 task 007, decision D6: the track stays in the
            // Library, so no confirmation opens.
            TrackRowMenuAction::RemoveFromPlaylist => {
                self.remove_playlist_track_at(playlist_id, position, cx);
            }
            TrackRowMenuAction::AddToPlaylist
            | TrackRowMenuAction::Download
            | TrackRowMenuAction::InspectSources
            | TrackRowMenuAction::Remove
            | TrackRowMenuAction::CopyToMyPlaylists
            | TrackRowMenuAction::RenamePlaylist
            | TrackRowMenuAction::DeletePlaylist => {
                self.run_track_row_menu_action(track_id, action, window, cx);
            }
        }
    }

    /// Runs one Library item of a track row menu of a list row (ADR 0083
    /// task 007, "The commands").
    pub(crate) fn run_track_row_menu_action(
        &mut self,
        track_id: i64,
        action: TrackRowMenuAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match action {
            TrackRowMenuAction::Download => self.download_library_tracks(&[track_id], cx),
            // ADR 0083 task 006, decision D4, and ADR 0089 Decision 8: open
            // the track page, then its Inspect Sources sheet.
            TrackRowMenuAction::InspectSources => {
                if let Some(key) = self.vm.library_track_key(track_id) {
                    self.open_inspect_sources(track_id, FrameNavigationEntry::Track(key), cx);
                }
            }
            // ADR 0083 Decision 5: "Remove from Library…" always asks for
            // confirmation.
            TrackRowMenuAction::Remove => {
                self.remove_track_after_confirmation(track_id, window, cx);
            }
            TrackRowMenuAction::AddToPlaylist
            | TrackRowMenuAction::MoveUp
            | TrackRowMenuAction::MoveDown
            | TrackRowMenuAction::RemoveFromPlaylist
            | TrackRowMenuAction::CopyToMyPlaylists
            | TrackRowMenuAction::RenamePlaylist
            | TrackRowMenuAction::DeletePlaylist => return,
        }
        cx.notify();
    }

    /// ADR 0076 Decision 7: the operator confirms that the show plays a
    /// removed track. The mounted readiness list then shows its route state.
    /// The check actor reads the marks of each loaded playlist that holds
    /// the track again, so the mounted playlist page updates in place
    /// (packet 005).
    fn confirm_removed_track(&mut self, track_id: i64, cx: &mut Context<Self>) {
        let command = ConfirmRemovedTrack::new(Arc::clone(&self.conn), track_id);
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, _changed, cx| {
                this.refresh_current_broadcast_readiness_report(cx);
                this.reload_playlist_rss_track_marks(track_id);
            },
            |this, error, cx| {
                this.vm.set_error_status(error);
                this.refresh_current_broadcast_readiness_report(cx);
            },
        );
    }

    /// ADR 0076 packet 005: the check actor reads the "removed from feed"
    /// marks again for each loaded playlist with a mark for this track.
    fn reload_playlist_rss_track_marks(&self, track_id: i64) {
        if let Some(checker) = &self.playlist_rss_check {
            let _ = checker.reload_track_marks(track_id);
        }
    }

    /// ADR 0076 packet 005: the check actor reads the "removed from feed"
    /// marks of this playlist again.
    fn reload_playlist_rss_marks(&self, playlist_id: i64) {
        if let Some(checker) = &self.playlist_rss_check {
            let _ = checker.reload_playlist_marks(playlist_id);
        }
    }

    /// Stores the Setup items of the Inbox (ADR 0089 Decision 3) and redraws
    /// the sidebar count.
    pub(crate) fn set_inbox_setup(
        &mut self,
        setup: Vec<crate::view_models::inbox::InboxSetupItem>,
        cx: &mut Context<Self>,
    ) {
        self.vm.set_inbox_setup(setup);
        cx.notify();
    }

    /// Opens the Inbox destination from an already-fetched report (ADR 0084
    /// Decision 2, ADR 0089 Decision 3). Show's "Open" calls this with its
    /// own cached report, so this makes no readiness query. It uses the
    /// show playlist ids of the last readiness load (task 039, step 2).
    pub(crate) fn show_broadcast_readiness_report(
        &mut self,
        report: &BroadcastReadinessReport,
        cx: &mut Context<Self>,
    ) {
        self.vm
            .show_inbox_report(report, &self.inbox_source_track_ids);
        cx.notify();
    }

    /// Opens the Inbox destination from a fresh load (ADR 0084 Decision 2: a
    /// sidebar click). The render path makes no query: this runs only from
    /// an explicit destination-select event. The Inbox keeps its last state
    /// until the load ends (task 039, step 2).
    fn open_inbox_destination(&mut self, cx: &mut Context<Self>) {
        self.readiness_open_pending = true;
        self.request_broadcast_readiness(cx);
    }

    /// Keeps the Inbox count current (ADR 0084 Decision 2): on load, after
    /// a feed check or a repair, and after a download or a removal. It
    /// replaces the active list's rows only when the Inbox is already open.
    fn refresh_current_broadcast_readiness_report(&mut self, cx: &mut Context<Self>) {
        self.request_broadcast_readiness(cx);
    }

    /// Starts the readiness load, or sets the run-again flag when one runs
    /// (task 039, step 3). The file tags are read off the main thread.
    fn request_broadcast_readiness(&mut self, cx: &mut Context<Self>) {
        let runner = &self.command_runner;
        let generation = runner.session().generation();
        let Some(run) = self
            .readiness_flight
            .request(generation, |live| runner.session().accepts(live))
        else {
            return;
        };
        let command = LoadBroadcastReadiness::new(Arc::clone(&self.conn), self.music_dir.clone());
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, load, cx| this.finish_broadcast_readiness(run, Ok(load), cx),
            move |this, error, cx| this.finish_broadcast_readiness(run, Err(error), cx),
        );
    }

    /// Applies the result of one readiness run. A result of a run that is not
    /// current is dropped.
    fn finish_broadcast_readiness(
        &mut self,
        run: Run,
        result: Result<BroadcastReadinessLoad, CommandError>,
        cx: &mut Context<Self>,
    ) {
        let again = match self.readiness_flight.finish(run) {
            Finish::Stale => return,
            Finish::Done => false,
            Finish::RunAgain => true,
        };
        match result {
            Ok(load) => {
                self.inbox_source_track_ids = load.source_track_ids;
                if std::mem::take(&mut self.readiness_open_pending) {
                    self.vm
                        .show_inbox_report(&load.report, &self.inbox_source_track_ids);
                } else {
                    self.vm
                        .refresh_inbox_report(&load.report, &self.inbox_source_track_ids);
                }
            }
            Err(error) => self.vm.set_error_status(error),
        }
        if again {
            self.request_broadcast_readiness(cx);
        }
    }

    pub(crate) fn recent_music_index_feed_detail(
        &self,
        activation_id: &str,
        fallback_id: &str,
        fallback_label: &str,
    ) -> Option<crate::view_models::search_results::IndexDetailDisplay> {
        self.vm
            .content_list_index_feed_detail(activation_id, fallback_id, fallback_label)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "top-level app bootstrap passes the shared services into the library surface"
    )]
    pub(crate) fn new(
        conn: Arc<Mutex<Connection>>,
        cache: Arc<ImageCache>,
        musicindex_endpoint: crate::config::MusicIndexEndpoint,
        music_dir: PathBuf,
        application_services: Arc<ApplicationServices>,
        runtime_host: Option<Arc<crate::presentation::RuntimeHost>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let chrome = LibraryViewModel::chrome_display();
        // The controls of the page come first in the Tab sequence, as they do
        // on the screen, so the list has the higher tab index.
        let list_column_focus = cx.focus_handle().tab_index(1);
        let new_playlist_input = cx.new(|cx: &mut Context<InputState>| {
            InputState::new(window, cx).placeholder(chrome.new_playlist_placeholder)
        });
        let rename_playlist_input = cx.new(|cx: &mut Context<InputState>| {
            InputState::new(window, cx)
                .placeholder(PlaylistDetailActionsDisplay::RENAME_INPUT_PLACEHOLDER)
        });
        let rename_playlist_sub =
            cx.subscribe(&rename_playlist_input, Self::on_rename_playlist_event);
        let saved_search_rename_input = cx.new(|cx: &mut Context<InputState>| {
            InputState::new(window, cx).placeholder("Saved search name")
        });
        let saved_search_rename_sub = cx.subscribe(
            &saved_search_rename_input,
            Self::on_saved_search_rename_event,
        );
        let public_playlist_copy_input = cx.new(|cx: &mut Context<InputState>| {
            InputState::new(window, cx)
                .placeholder(crate::view_models::public_playlists::COPY_NAME_PLACEHOLDER)
        });
        let public_playlist_copy_sub = cx.subscribe_in(
            &public_playlist_copy_input,
            window,
            Self::on_public_playlist_copy_input_event,
        );
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
        };
        let vm = LibraryViewModel::new();
        let detail_column_scroll = gpui::ScrollHandle::new();
        let inbox_scroll = gpui::ScrollHandle::new();
        let releases_tiles_scroll = gpui::ScrollHandle::new();
        let mut app = Self {
            playback_availability: Err(
                crate::application::capability::ExecutionUnavailable::configured(
                    crate::application::capability::Dependency::Playback,
                ),
            ),
            conn,
            application_services,
            command_runner,
            cache,
            musicindex_endpoint,
            music_dir,
            vm,
            detail: LibraryDetail::None,
            thumbnails: BTreeMap::new(),
            cover_colors: BTreeMap::new(),
            collection_art: CollectionArtLoads::default(),
            new_playlist_input,
            rename_playlist_input,
            _rename_playlist_sub: rename_playlist_sub,
            saved_search_rename_input,
            _saved_search_rename_sub: saved_search_rename_sub,
            public_playlist_copy_input,
            _public_playlist_copy_input_sub: public_playlist_copy_sub,
            public_playlist_copy: None,
            copying_public_playlist: None,
            public_playlist_copy_report: None,
            saved_search_renaming: None,
            saved_search_message: None,
            saved_search_counts: std::collections::BTreeMap::new(),
            saved_search_counts_handle: None,
            saved_search_counts_requested_at: None,
            runtime_host,
            playlist_actor: None,
            musicbrainz_feed_saga: None,
            playlist_rss_check: None,
            playlist_rss_snapshot: None,
            tag_update: None,
            tag_update_snapshot: None,
            tag_update_request_failed: false,
            recent_music_page: RecentFeedsPageVm::loading(),
            index_sorts_requested: false,
            artist_name_page: IndexNameListPage::default(),
            release_name_page: IndexNameListPage::default(),
            inbox_scroll,
            search_page_scroll: gpui::ScrollHandle::new(),
            playlists_scroll: gpui::ScrollHandle::new(),
            expanded_singles_groups: BTreeSet::new(),
            download_queue: super::DownloadQueue::default(),
            releases_selection: ListSelection::new(),
            releases_selection_mode: SelectionMode::Normal,
            track_selection: ListSelection::new(),
            track_selection_mode: SelectionMode::Normal,
            releases_list_state: MusicColumnList::new(cx),
            releases_tile_reveal: ScrollReveal::new(releases_tiles_scroll.clone()),
            releases_tile_target: None,
            releases_tiles_scroll,
            releases_list_cache: std::cell::RefCell::default(),
            artists_list_state: MusicColumnList::new(cx),
            public_playlists_list_state: MusicColumnList::new(cx),
            public_playlists_tiles_scroll: gpui::ScrollHandle::new(),
            public_playlist_list: IndexNameListPage::default(),
            public_playlist_pages: PublicPlaylistPages::default(),
            live_list_state: MusicColumnList::new(cx),
            live_list: IndexNameListPage::default(),
            list_column_focus,
            list_row_focus_shows: false,
            letter_bar_focus: cx.focus_handle().tab_index(0).tab_stop(true),
            letter_cursor: None,
            letter_cursor_shows: false,
            selection_removal: SelectionRemovalConfirmationState::new(),
            selection_removal_report: None,
            detail_selection: DetailSelection::new(),
            detail_column_focus: cx.focus_handle(),
            sidebar_focus: crate::view_models::sidebar::SidebarDestination::ALL
                .into_iter()
                .map(|row| (row, cx.focus_handle().tab_index(0).tab_stop(true)))
                .collect(),
            sidebar_selected: None,
            pending_inspect_sources: None,
            spring_load: crate::view_models::library::SpringLoadVm::default(),
            detail_scroll_reveal: ScrollReveal::new(detail_column_scroll.clone()),
            list_bar_pin: ListBarPin::default(),
            detail_column_scroll,
            detail_scroll_binding: super::detail_selection::DetailScrollBinding::default(),
            saved_searches: Vec::new(),
            open_saved_search: None,
            how_to_search: HowToSearchDisclosure::Hidden,
            marked_entry: None,
            music_page_open: false,
            album_identity_backfill_started: false,
            album_identity_backfill_queue: VecDeque::new(),
            readiness_flight: SingleFlight::new(),
            readiness_open_pending: false,
            inbox_source_track_ids: Vec::new(),
            tree_flight: SingleFlight::new(),
            playlist_flight: SingleFlight::new(),
            playlist_after: Vec::new(),
        };
        // ADR 0084 Decision 3, packet 012: Discover shows the saved
        // searches from the first render.
        app.reload_saved_searches();
        app.maybe_start_musicbrainz_feed_saga(cx);
        app.maybe_start_playlist_rss_check(cx);
        app.maybe_start_tag_update(cx);
        app.start_async_reload(cx);
        app.start_recent_music_load(false, cx);
        app
    }

    /// Install the repaired runtime into the existing child, without new subscriptions.
    pub(crate) fn install_runtime(
        &mut self,
        host: Arc<crate::presentation::RuntimeHost>,
        cx: &mut Context<Self>,
    ) {
        if self.runtime_host.is_some() {
            return;
        }
        self.command_runner = AsyncCommandRunner::with_vm_bus_on_handle(
            self.application_services.command_bus(),
            self.application_services.event_bus(),
            host.bus().clone(),
            host.handle().clone(),
        );
        self.runtime_host = Some(host);
        self.maybe_start_musicbrainz_feed_saga(cx);
        self.maybe_start_playlist_rss_check(cx);
        self.maybe_start_tag_update(cx);
        self.start_async_reload_preserving_detail(cx);
        self.start_recent_music_load(false, cx);
    }

    pub(crate) fn release_session_actors(&mut self) {
        self.playlist_actor.take();
        self.musicbrainz_feed_saga.take();
        self.playlist_rss_check.take();
        self.playlist_rss_snapshot = None;
        self.tag_update.take();
        self.tag_update_snapshot = None;
        self.saved_search_counts_handle.take();
        self.runtime_host.take();
    }

    pub(crate) fn bind_session(
        &mut self,
        session: crate::application::session_lifecycle::SessionLifecycle,
    ) {
        self.command_runner = self.command_runner.clone().with_session(session);
    }

    fn maybe_start_musicbrainz_feed_saga(&mut self, cx: &mut Context<Self>) {
        if self.musicbrainz_feed_saga.is_some() {
            return;
        }
        let Some(host) = self.runtime_host.as_ref() else {
            return;
        };
        let _enter = host.handle().enter();
        let handle = crate::runtime::musicbrainz_feed_saga::spawn(
            self.application_services.command_bus(),
            host.bus().session(),
        );
        bridge_watch(
            handle.subscribe(),
            |this: &mut Self, state, cx| {
                this.apply_musicbrainz_feed_saga_state(state, cx);
            },
            cx,
        );
        self.musicbrainz_feed_saga = Some(handle);
    }

    /// Start the playlist RSS check actor (ADR 0076 Decision 2, ADR 0040).
    fn maybe_start_playlist_rss_check(&mut self, cx: &mut Context<Self>) {
        if self.playlist_rss_check.is_some() {
            return;
        }
        let Some(host) = self.runtime_host.as_ref() else {
            return;
        };
        let _enter = host.handle().enter();
        let handle =
            crate::runtime::playlist_rss_check::spawn(Arc::clone(&self.conn), host.bus().session());
        self.playlist_rss_snapshot = Some(handle.latest());
        bridge_watch(
            handle.subscribe(),
            |this: &mut Self, snapshot, cx| {
                // ADR 0076 packet 002: a check that applied RSS values
                // changes stored feed and track values. The mounted view
                // reloads them in place.
                let applied = this.playlist_rss_snapshot.as_ref().is_some_and(|previous| {
                    previous.applied_revision() != snapshot.applied_revision()
                });
                // ADR 0076 Decision 8: a check that completes starts the
                // tag update scan.
                let completed = this.playlist_rss_snapshot.as_ref().is_some_and(|previous| {
                    previous
                        .running_playlists()
                        .any(|playlist_id| !snapshot.is_running(playlist_id))
                });
                this.playlist_rss_snapshot = Some(snapshot);
                if applied {
                    this.refresh_origin_playlist_actor();
                    this.start_async_reload_preserving_detail(cx);
                }
                if completed {
                    this.scan_tag_updates();
                }
            },
            cx,
        );
        if let LibraryDetail::Playlist(detail) = &self.detail {
            let _ = handle.load(detail.playlist.id);
        }
        self.playlist_rss_check = Some(handle);
    }

    /// Start the tag update actor (ADR 0076 Decision 8, ADR 0040) and its
    /// first scan.
    fn maybe_start_tag_update(&mut self, cx: &mut Context<Self>) {
        if self.tag_update.is_some() {
            return;
        }
        let Some(host) = self.runtime_host.as_ref() else {
            return;
        };
        let _enter = host.handle().enter();
        let handle = crate::runtime::tag_update::spawn(
            Arc::clone(&self.conn),
            host.bus().clone(),
            host.bus().session(),
        );
        self.tag_update_snapshot = Some(handle.latest());
        bridge_watch(
            handle.subscribe(),
            |this: &mut Self, snapshot, cx| {
                // A confirm changed file tags. The readiness list reads the
                // route frames again.
                let wrote = this
                    .tag_update_snapshot
                    .as_ref()
                    .is_some_and(|previous| previous.last_write != snapshot.last_write);
                this.tag_update_snapshot = Some(snapshot);
                if wrote {
                    this.refresh_current_broadcast_readiness_report(cx);
                }
            },
            cx,
        );
        let _ = handle.scan(self.music_dir.clone());
        self.tag_update = Some(handle);
    }

    /// Request a tag update scan (ADR 0076 Decision 8). The scan runs in
    /// the runtime actor. The app requests it after a completed RSS check,
    /// after a download, after a feed update and after a tag apply.
    pub(crate) fn scan_tag_updates(&mut self) {
        if let Some(handle) = &self.tag_update {
            let _ = handle.scan(self.music_dir.clone());
        }
    }

    /// Open the "Update n file(s)" popup. The confirm writes the listed
    /// files that the popup shows.
    pub(crate) fn open_tag_update_popup(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let snapshot = self.tag_update_snapshot.as_ref();
        let Some(display) = crate::view_models::tag_update::popup(snapshot) else {
            return;
        };
        let files = snapshot
            .and_then(|snapshot| snapshot.scan.as_ref())
            .map(|scan| scan.files.clone())
            .unwrap_or_default();
        open_tag_update_confirmation_dialog(window, cx, display, move |this, cx| {
            this.confirm_tag_update(files.clone(), cx);
        });
    }

    fn confirm_tag_update(
        &mut self,
        files: Vec<crate::application::queries::tag_update::TagUpdateFile>,
        cx: &mut Context<Self>,
    ) {
        self.tag_update_request_failed = !self
            .tag_update
            .as_ref()
            .is_some_and(|handle| handle.confirm(self.music_dir.clone(), files));
        cx.notify();
    }

    /// Send `CheckPlaylistRss` for the playlist (ADR 0076 Decision 2).
    ///
    /// The "Check RSS" action and the playback start of `src/app.rs` call
    /// this method. The command returns at once. The check runs in the
    /// runtime actor, and its snapshot updates the mounted playlist page.
    pub(crate) fn check_playlist_rss(
        &mut self,
        playlist_id: i64,
        trigger: crate::runtime::RssCheckTrigger,
        cx: &mut Context<Self>,
    ) {
        let Some(checker) = self.playlist_rss_check.clone() else {
            if trigger == crate::runtime::RssCheckTrigger::Button {
                self.vm
                    .fail_playlist_rss_check("The background runtime is not available.");
                cx.notify();
            }
            return;
        };
        let command = crate::application::commands::playlist::CheckPlaylistRss::new(
            checker,
            playlist_id,
            trigger,
        );
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            |_this, (), _cx| {},
            |this, err, _cx| this.vm.fail_playlist_rss_check(err),
        );
    }

    fn on_rename_playlist_event(
        &mut self,
        _entity: Entity<InputState>,
        event: &InputEvent,
        cx: &mut Context<Self>,
    ) {
        if let InputEvent::PressEnter { .. } = event {
            self.submit_playlist_rename(cx);
        }
    }

    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.start_async_reload_preserving_detail(cx);
    }

    pub fn begin_new_playlist(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // ADR 0084 Decision 2: the Playlists list, with its inline input,
        // lives in the list column now. `cmd-n` opens that destination so
        // the input is in view.
        self.open_page(
            FrameNavigationEntry::Destination(MusicDestination::Playlists),
            NavigationPathStart::NewPath,
            cx,
        );
        if !self.vm.playlist_sidebar().creating_playlist {
            self.vm.toggle_creating_playlist();
        }
        self.vm.cancel_playlist_rename();
        self.new_playlist_input.update(cx, |input, cx| {
            input.focus(window, cx);
        });
        cx.notify();
    }

    pub fn pop_inspector(&mut self, cx: &mut Context<Self>) {
        if !matches!(self.detail, LibraryDetail::None) {
            self.detail = LibraryDetail::None;
            cx.notify();
        }
    }

    pub fn set_musicindex_endpoint(
        &mut self,
        endpoint: crate::config::MusicIndexEndpoint,
        cx: &mut Context<Self>,
    ) {
        self.musicindex_endpoint = endpoint;
        cx.notify();
    }

    pub fn set_music_dir(&mut self, music_dir: PathBuf, cx: &mut Context<Self>) {
        self.music_dir = music_dir;
        self.scan_tag_updates();
        cx.notify();
    }

    /// Kick off a non-blocking library reload.
    ///
    /// The render thread returns immediately; the heavy
    /// `library_tracks` + `build_tree` work runs on the background
    /// executor. While the reload is in flight the view-model is
    /// flagged so the sidebar paints skeleton placeholders instead of
    /// blocking the cold-open paint.
    pub(crate) fn start_async_reload(&mut self, cx: &mut Context<Self>) {
        self.start_async_reload_with_mode(LibraryReloadMode::ResetDetail, cx);
    }

    fn start_async_reload_preserving_detail(&mut self, cx: &mut Context<Self>) {
        self.start_async_reload_with_mode(LibraryReloadMode::PreserveDetail, cx);
    }

    fn start_async_reload_with_mode(&mut self, mode: LibraryReloadMode, cx: &mut Context<Self>) {
        self.vm.begin_library_reload();
        self.vm.clear_mb_status();
        if mode == LibraryReloadMode::ResetDetail {
            self.vm.clear_library_selection();
            self.detail = LibraryDetail::None;
        }
        cx.notify();

        self.reload_playlists(cx);
        if mode == LibraryReloadMode::PreserveDetail {
            self.refresh_selected_detail(cx);
        }

        self.request_library_tree_load(cx);
    }

    /// Starts the Library tree load, or sets the run-again flag when one
    /// runs (task 039, step 4). Each completion of a run starts the waiting
    /// run, so the last request always gets a result.
    fn request_library_tree_load(&mut self, cx: &mut Context<Self>) {
        let runner = &self.command_runner;
        let generation = runner.session().generation();
        let Some(run) = self
            .tree_flight
            .request(generation, |live| runner.session().accepts(live))
        else {
            return;
        };
        let command = LoadLibraryTracksTree::new(Arc::clone(&self.conn));
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, result, cx| this.finish_library_tree_load(run, Ok(result), cx),
            move |this, err, cx| this.finish_library_tree_load(run, Err(err), cx),
        );
    }

    fn finish_library_tree_load(
        &mut self,
        run: Run,
        result: Result<LibraryTracksTree, CommandError>,
        cx: &mut Context<Self>,
    ) {
        let again = match self.tree_flight.finish(run) {
            Finish::Stale => return,
            Finish::Done => false,
            Finish::RunAgain => true,
        };
        match result {
            Ok(result) => self.apply_library_tree(result, cx),
            Err(err) => self.vm.set_error_status(err),
        }
        if again {
            self.request_library_tree_load(cx);
        }
    }

    /// Applies one loaded Library tree to the view model, and the selections
    /// and the backfill that follow a load.
    fn apply_library_tree(&mut self, result: LibraryTracksTree, cx: &mut Context<Self>) {
        self.vm.replace_tree(result.tree);
        self.vm.finish_library_reload(result.count);
        // ADR 0089 Decision 5: the scope "All" of the active list
        // sends its first name request after a Library load, once.
        self.request_default_name_list(cx);
        // ADR 0084 Decision 2: the Inbox count stays
        // current on each Library load.
        self.refresh_current_broadcast_readiness_report(cx);
        // ADR 0084 Decision 10: a removal drops the removed
        // items from both selections through `retain_visible`.
        // Every reload re-syncs them, so a stale selected key
        // never outlives the row it named.
        let releases_visible = self.releases_visible_members();
        self.releases_selection.retain_visible(&releases_visible);
        let tracks_visible = self.content_list_visible_track_keys();
        self.track_selection.retain_visible(&tracks_visible);
        // ADR 0084 packet 018: the same for the detail selection
        // of a Library page. The feed page and the Index page
        // keep each row after a removal.
        if let Some(rows) = self.library_detail_page_rows() {
            self.detail_selection.retain_visible(&rows.members());
        }
        // ADR 0084 task 034, decision D2: the first successful
        // Library tree load of the session starts the
        // once-per-session identity backfill pass.
        if !self.album_identity_backfill_started {
            self.start_album_identity_backfill(cx);
        }
    }

    /// Reads the playlist list and the cover of each playlist in the
    /// background (task 039, step 4). The sidebar keeps its last list until
    /// the result arrives.
    pub(crate) fn reload_playlists(&mut self, cx: &mut Context<Self>) {
        self.request_playlist_load(cx);
    }

    /// Reloads the playlists, then runs `after` with the new list. A caller
    /// that selects or adds to a playlist uses this, because it reads the
    /// list (task 039, step 4).
    pub(crate) fn reload_playlists_then(
        &mut self,
        cx: &mut Context<Self>,
        after: impl FnOnce(&mut Self, &mut Context<Self>) + 'static,
    ) {
        self.playlist_after.push(Box::new(after));
        self.request_playlist_load(cx);
    }

    fn request_playlist_load(&mut self, cx: &mut Context<Self>) {
        let runner = &self.command_runner;
        let generation = runner.session().generation();
        let Some(run) = self
            .playlist_flight
            .request(generation, |live| runner.session().accepts(live))
        else {
            return;
        };
        let command = LoadPlaylistSidebar::new(Arc::clone(&self.conn));
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, load, cx| this.finish_playlist_load(run, Ok(load), cx),
            move |this, error, cx| this.finish_playlist_load(run, Err(error), cx),
        );
    }

    fn finish_playlist_load(
        &mut self,
        run: Run,
        result: Result<PlaylistSidebarLoad, CommandError>,
        cx: &mut Context<Self>,
    ) {
        let again = match self.playlist_flight.finish(run) {
            Finish::Stale => return,
            Finish::Done => false,
            Finish::RunAgain => true,
        };
        match result {
            Ok(load) => {
                self.vm.replace_playlists(load.playlists);
                self.vm.replace_playlist_covers(load.covers);
            }
            Err(error) => self.vm.fail_playlist_load(error),
        }
        if again {
            self.request_playlist_load(cx);
            return;
        }
        // The waiting continuations read the list that the last run loaded.
        for after in std::mem::take(&mut self.playlist_after) {
            after(self, cx);
        }
    }

    pub(crate) fn playlists(&self) -> &[db::Playlist] {
        self.vm.playlists()
    }

    fn refresh_selected_detail(&mut self, cx: &mut Context<Self>) {
        let playlist_id = match &self.detail {
            LibraryDetail::Playlist(detail) => Some(detail.playlist.id),
            LibraryDetail::None
            | LibraryDetail::Album(_)
            | LibraryDetail::Track(_)
            | LibraryDetail::SinglesGroup(_)
            | LibraryDetail::WindowGroup(_) => None,
        };
        if let Some(playlist_id) = playlist_id {
            self.select_playlist(playlist_id, cx);
            return;
        }

        if let LibraryDetail::Album(album) = &self.detail {
            if let Some(feed_id) = album.feed_id {
                if let Some(album) = self.album_for_detail_by_feed_id(feed_id) {
                    // Packet 026 review: this reload keeps the page open, so
                    // it must not resend a failed request for the artist
                    // feed's MusicIndex singles.
                    self.refresh_album_detail(&album, cx);
                }
                return;
            }
        }

        if let LibraryDetail::Track(frame) = &self.detail {
            self.refresh_selected_track(frame.entity_id, cx);
        }
    }

    fn refresh_selected_track(&mut self, track_id: i64, cx: &mut Context<Self>) {
        // Preserve-detail reloads use one synchronous PK lookup, matching
        // the sidebar playlist refresh path until a shared async detail actor
        // is introduced.
        let track = {
            let conn = self.conn.lock().expect("lock db");
            db::track_row_by_id(&conn, track_id).unwrap_or_default()
        };
        let Some(track) = track else {
            return;
        };
        let Some(frame) = self.selected_track_frame_mut() else {
            return;
        };
        if frame.entity_id != track_id {
            return;
        }
        frame.title = LibraryTrackRowVm::new(&track, None).display_title();
        frame.local_subscription = track.is_in_library && track.local_path.is_some();
        frame.track = track.clone();
        frame.source_context = None;
        frame.tag_compare = LazyPanel::Hidden;
        frame.pending_id3_edits.clear();
        frame.suppressed_auto_id3_edits.clear();
        frame.id3_apply_error = None;
        self.load_track_source_context(track, cx);
    }

    /// Asks the top app to open a page in the one Music navigation stack
    /// (ADR 0084 Decision 9). The top app restores the page through
    /// [`Self::hydrate_detail_from_nav`], or, for a release, through its own
    /// resolved data source (ADR 0084 Decision 4).
    pub(crate) fn open_page(
        &mut self,
        entry: FrameNavigationEntry,
        start: NavigationPathStart,
        cx: &mut Context<Self>,
    ) {
        cx.emit(super::LibraryAppEvent::OpenPage { entry, start });
    }

    /// Mounts the playlist page. It changes no navigation state.
    pub(crate) fn select_playlist(&mut self, id: i64, cx: &mut Context<Self>) {
        self.vm.select_playlist(id);
        let conn = self.conn.lock().expect("lock db");
        let playlist = self.vm.playlist_by_id(id);
        let tracks = self
            .application_services
            .query_service()
            .playlist_tracks(&conn, id)
            .unwrap_or_default();
        drop(conn);
        if let Some(playlist) = playlist {
            self.detail = LibraryDetail::Playlist(PlaylistDetail {
                playlist,
                tracks: tracks.clone(),
            });
            self.spawn_playlist_actor(id, &tracks, cx);
            if let Some(checker) = &self.playlist_rss_check {
                let _ = checker.load(id);
            }
            self.vm.replace_playlist_tracks(tracks);
            // ADR 0084 packet 025: a reload drops each selected entry that
            // the playlist no longer holds at its position. A move to
            // another page resets the selection before this call.
            if self.detail_selection.entry() == Some(&FrameNavigationEntry::PlaylistDetail(id)) {
                if let Some(rows) = self.library_detail_page_rows() {
                    self.detail_selection.retain_visible(&rows.members());
                }
            }
        }
        cx.notify();
    }

    /// Spawn (or replace) the paged playlist actor for `playlist_id`.
    ///
    /// Dropping the previous handle closes its inbox so the actor task
    /// exits gracefully. The new handle is bridged via
    /// [`crate::presentation::bridge_watch`] so snapshot publishes
    /// trigger a re-render automatically.
    fn spawn_playlist_actor(
        &mut self,
        playlist_id: i64,
        initial_rows: &[TrackRow],
        cx: &mut Context<Self>,
    ) {
        use crate::application::paged_track_list::{PagedTrackListActor, PagedTrackListMsg};
        use crate::db::TrackListing;

        let Some(host) = self.runtime_host.clone() else {
            return;
        };
        if let Some(state) = &self.playlist_actor {
            if state.playlist_id == playlist_id {
                let _ = state
                    .handle
                    .try_send(PagedTrackListMsg::PrimeRows(initial_rows.to_vec()));
                let _ = state.handle.try_send(PagedTrackListMsg::Refresh);
                return;
            }
        }
        self.playlist_actor = None;
        // Open a dedicated connection for the actor: rusqlite Connections
        // are not Sync, and the actor consumes its connection by value.
        let db_path = self
            .conn
            .lock()
            .ok()
            .and_then(|conn| conn.path().map(std::path::PathBuf::from));
        let Some(db_path) = db_path else {
            return;
        };
        let conn = match crate::db::startup::open_existing(&db_path) {
            Ok(conn) => conn,
            Err(error) => {
                self.vm.set_error_status(format!(
                    "App could not open the playlist database: {}",
                    error.reason
                ));
                return;
            }
        };
        let mut actor = match PagedTrackListActor::new(conn, TrackListing::Playlist { playlist_id })
        {
            Ok(actor) => actor,
            Err(err) => {
                eprintln!("v4vmm::library: PagedTrackListActor::new failed: {err}");
                return;
            }
        };
        actor.prime_initial_rows(initial_rows.iter().cloned());
        let bus = host.bus().clone();
        let _enter = host.handle().enter();
        let handle = actor.spawn(bus);
        let snapshot = handle.borrow().clone();
        let rx = handle.subscribe();
        self.playlist_actor = Some(PlaylistActorState {
            playlist_id,
            snapshot,
            handle,
        });
        bridge_watch(
            rx,
            move |this: &mut Self, snap, _cx| {
                if let Some(state) = &mut this.playlist_actor {
                    if state.playlist_id == playlist_id {
                        state.snapshot = snap;
                    }
                }
            },
            cx,
        );
    }

    /// Refreshes the rows of the mounted playlist actor after a change on a
    /// track page. A later Back to that playlist then shows the change.
    /// The playlist page itself reloads through `refresh_selected_detail`.
    fn refresh_origin_playlist_actor(&mut self) {
        use crate::application::paged_track_list::PagedTrackListMsg;

        if !matches!(self.detail, LibraryDetail::Track(_)) {
            return;
        }
        let Some(state) = &self.playlist_actor else {
            return;
        };
        let playlist_id = state.playlist_id;
        let tracks = {
            let conn = self.conn.lock().expect("lock db");
            self.application_services
                .query_service()
                .playlist_tracks(&conn, playlist_id)
                .unwrap_or_default()
        };
        let _ = state.handle.try_send(PagedTrackListMsg::PrimeRows(tracks));
        let _ = state.handle.try_send(PagedTrackListMsg::Refresh);
    }

    pub(crate) fn create_playlist(&mut self, cx: &mut Context<Self>) {
        let name = self.new_playlist_input.read(cx).value().to_string();
        let name = name.trim();
        if name.is_empty() {
            return;
        }
        let command = CreatePlaylist::new(Arc::clone(&self.conn), name.to_string());
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            |this, result, cx| {
                this.vm.close_creating_playlist();
                let playlist_id = result.playlist_id();
                // ADR 0084 Decision 2: "+" opens the playlist from the
                // Playlists destination, in the current path. The page
                // reads the list, so it opens after the reload.
                this.reload_playlists_then(cx, move |this, cx| {
                    this.open_page(
                        FrameNavigationEntry::PlaylistDetail(playlist_id),
                        NavigationPathStart::CurrentPath,
                        cx,
                    );
                });
            },
            |this, err, _cx| this.vm.fail_playlist_create(err),
        );
    }

    fn rename_playlist(&mut self, id: i64, new_name: String, cx: &mut Context<Self>) {
        let trimmed = new_name.trim();
        if trimmed.is_empty() {
            return;
        }
        let command = RenamePlaylist::new(Arc::clone(&self.conn), id, trimmed.to_string());
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, (), cx| {
                this.reload_playlists_then(cx, move |this, cx| {
                    if this.vm.is_playlist_selected(id) {
                        this.select_playlist(id, cx);
                    }
                });
            },
            |this, err, _cx| this.vm.fail_playlist_rename(err),
        );
    }

    pub(crate) fn begin_playlist_rename(
        &mut self,
        playlist_id: i64,
        current_name: String,
        placeholder: &'static str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.vm.begin_playlist_rename(playlist_id);
        self.rename_playlist_input.update(cx, |input, cx| {
            input.set_placeholder(placeholder, window, cx);
            input.set_value(current_name, window, cx);
            input.focus(window, cx);
        });
        cx.notify();
    }

    pub(crate) fn cancel_playlist_rename(&mut self, cx: &mut Context<Self>) {
        self.vm.cancel_playlist_rename();
        cx.notify();
    }

    /// Toggles the inline new-playlist input of the Playlists destination
    /// list (ADR 0084 Decision 2).
    pub(crate) fn toggle_creating_playlist(&mut self, cx: &mut Context<Self>) {
        self.vm.toggle_creating_playlist();
        cx.notify();
    }

    pub(crate) fn submit_playlist_rename(&mut self, cx: &mut Context<Self>) {
        let Some(playlist_id) = self.vm.renaming_playlist_id() else {
            return;
        };
        let name = self.rename_playlist_input.read(cx).value().to_string();
        if name.trim().is_empty() {
            return;
        }
        self.vm.cancel_playlist_rename();
        self.rename_playlist(playlist_id, name, cx);
        cx.notify();
    }

    /// The item menu of each playlist row of the My playlists list, by
    /// playlist id (ADR 0089 Decision 6). Each menu runs the page actions of
    /// its row.
    pub(crate) fn playlist_row_menus(
        &self,
        sidebar: &PlaylistSidebarVm,
        cx: &mut Context<Self>,
    ) -> BTreeMap<i64, RowMenu> {
        sidebar
            .rows
            .iter()
            .map(|row| {
                let entity = cx.weak_entity();
                let id = row.id;
                let name = row.name.clone();
                let confirmation = row.delete_confirmation.clone();
                let action: MenuActionHandler = Rc::new(
                    move |action: TrackRowMenuAction, window: &mut Window, cx: &mut App| {
                        let _ = entity.update(cx, |this, cx| {
                            this.run_playlist_row_menu_command(
                                id,
                                &name,
                                &confirmation,
                                action,
                                window,
                                cx,
                            );
                        });
                    },
                );
                let select_playlist: SelectHandler =
                    Rc::new(|_: &i64, _: &mut Window, _: &mut App| {});
                let create_playlist: CreateHandler =
                    Rc::new(|_: &String, _: &mut Window, _: &mut App| {});
                let menu = RowMenu {
                    menu: row.menu.clone(),
                    playlists: Rc::new(Vec::new()),
                    handlers: TrackRowMenuHandlers {
                        action,
                        select_playlist,
                        create_playlist,
                    },
                    return_focus: self.list_column_focus.clone(),
                };
                (row.id, menu)
            })
            .collect()
    }

    /// Runs one item of the menu of a playlist row. "Rename" opens the
    /// playlist page with its rename input. "Delete" opens the confirmation
    /// of the playlist page, and only its confirm deletes the playlist.
    fn run_playlist_row_menu_command(
        &mut self,
        playlist_id: i64,
        name: &str,
        confirmation: &PlaylistDeleteConfirmation,
        action: TrackRowMenuAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match action {
            TrackRowMenuAction::RenamePlaylist => {
                self.open_page(
                    FrameNavigationEntry::PlaylistDetail(playlist_id),
                    NavigationPathStart::CurrentPath,
                    cx,
                );
                self.begin_playlist_rename(
                    playlist_id,
                    name.to_owned(),
                    PlaylistDetailActionsDisplay::RENAME_INPUT_PLACEHOLDER,
                    window,
                    cx,
                );
            }
            TrackRowMenuAction::DeletePlaylist => {
                open_playlist_delete_dialog(window, cx, confirmation.clone(), move |this, cx| {
                    this.delete_playlist(playlist_id, cx);
                });
            }
            _ => return,
        }
        cx.notify();
    }

    pub(crate) fn delete_playlist(&mut self, id: i64, cx: &mut Context<Self>) {
        let command = DeletePlaylist::new(Arc::clone(&self.conn), id);
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, (), cx| {
                if this.vm.clear_playlist_selection_if(id) {
                    this.detail = LibraryDetail::None;
                }
                this.reload_playlists(cx);
            },
            |this, err, _cx| this.vm.fail_playlist_delete(err),
        );
    }

    pub(crate) fn remove_playlist_track_at(
        &mut self,
        playlist_id: i64,
        position: i64,
        cx: &mut Context<Self>,
    ) {
        let command = RemovePlaylistTrackAt::new(Arc::clone(&self.conn), playlist_id, position);
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, (), cx| {
                this.reload_playlist_rss_marks(playlist_id);
                this.reload_playlists_then(cx, move |this, cx| {
                    if this.vm.is_playlist_selected(playlist_id) {
                        this.select_playlist(playlist_id, cx);
                    }
                });
            },
            |this, err, _cx| this.vm.fail_playlist_track_remove(err),
        );
    }

    /// ADR 0076 Decision 7: "Remove from all playlists" of a removed track.
    /// The confirmation names each playlist that holds the track. The
    /// command runs only after the operator confirms.
    pub(crate) fn request_remove_track_from_all_playlists(
        &mut self,
        track_id: i64,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let loaded = {
            let Ok(conn) = self.conn.lock() else {
                self.vm.set_error_status("database lock poisoned");
                cx.notify();
                return;
            };
            crate::playlist_service::playlists_holding_track(&conn, track_id).and_then(
                |playlists| {
                    let title = db::track_row_by_id(&conn, track_id)?
                        .and_then(|track| track.track_title)
                        .unwrap_or_else(|| "this track".to_owned());
                    Ok((title, playlists))
                },
            )
        };
        let (title, playlists) = match loaded {
            Ok(loaded) => loaded,
            Err(error) => {
                self.vm.set_error_status(format!("{error:#}"));
                cx.notify();
                return;
            }
        };
        if playlists.is_empty() {
            return;
        }
        let display =
            crate::view_models::playlist_rss_check::remove_from_all_playlists_confirmation(
                &title, &playlists,
            );
        crate::ui::shells::playlist_removal_confirmation::open_remove_from_all_playlists_dialog(
            window,
            cx,
            display,
            move |this, cx| this.remove_track_from_all_playlists(track_id, cx),
        );
    }

    fn remove_track_from_all_playlists(&mut self, track_id: i64, cx: &mut Context<Self>) {
        let command = RemoveTrackFromAllPlaylists::new(Arc::clone(&self.conn), track_id);
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, changed, cx| {
                let selected = changed
                    .iter()
                    .copied()
                    .find(|playlist_id| this.vm.is_playlist_selected(*playlist_id));
                this.reload_playlists_then(cx, move |this, cx| {
                    if let Some(playlist_id) = selected {
                        this.select_playlist(playlist_id, cx);
                    }
                });
                this.reload_playlist_rss_track_marks(track_id);
                this.refresh_current_broadcast_readiness_report(cx);
            },
            |this, err, _cx| this.vm.fail_playlist_track_remove(err),
        );
    }

    pub(crate) fn move_playlist_track(
        &mut self,
        playlist_id: i64,
        from: i64,
        to: i64,
        cx: &mut Context<Self>,
    ) {
        if from == to {
            return;
        }
        let command = ReorderPlaylistTrack::new(Arc::clone(&self.conn), playlist_id, from, to);
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, (), cx| {
                if this.vm.is_playlist_selected(playlist_id) {
                    this.select_playlist(playlist_id, cx);
                }
            },
            |this, err, _cx| this.vm.fail_playlist_track_reorder(err),
        );
    }

    /// "Move to top" and "Move to bottom" of the playlist page bar (ADR
    /// 0084 Decision 10, packet 025). One transaction moves the selected
    /// entries. The selection and the focus then follow the entries.
    pub(crate) fn move_selected_playlist_entries(
        &mut self,
        edge: db::PlaylistEdge,
        cx: &mut Context<Self>,
    ) {
        let LibraryDetail::Playlist(detail) = &self.detail else {
            return;
        };
        let playlist_id = detail.playlist.id;
        let entries = self.detail_selection.selected_playlist_entries();
        if entries.is_empty() {
            return;
        }
        let command = MovePlaylistEntries::new(Arc::clone(&self.conn), playlist_id, entries, edge);
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, total, cx| {
                this.detail_selection.rekey_after_move(edge, total);
                if this.vm.is_playlist_selected(playlist_id) {
                    this.select_playlist(playlist_id, cx);
                }
            },
            |this, err, _cx| this.vm.fail_playlist_track_reorder(err),
        );
    }

    /// "Remove n from playlist" of the playlist page bar (ADR 0084
    /// Decision 10, packet 025). One transaction removes the selected
    /// entries. The tracks stay in the Library. The rows clear, and the
    /// mode stays `Selecting`.
    pub(crate) fn remove_selected_playlist_entries(&mut self, cx: &mut Context<Self>) {
        let LibraryDetail::Playlist(detail) = &self.detail else {
            return;
        };
        let playlist_id = detail.playlist.id;
        let entries = self.detail_selection.selected_playlist_entries();
        if entries.is_empty() {
            return;
        }
        let command = RemovePlaylistEntries::new(Arc::clone(&self.conn), playlist_id, entries);
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, (), cx| {
                this.reload_playlist_rss_marks(playlist_id);
                this.refresh_current_broadcast_readiness_report(cx);
                this.detail_selection.clear_rows();
                this.reload_playlists_then(cx, move |this, cx| {
                    if this.vm.is_playlist_selected(playlist_id) {
                        this.select_playlist(playlist_id, cx);
                    }
                });
            },
            |this, err, _cx| this.vm.fail_playlist_track_remove(err),
        );
    }

    pub(crate) fn add_track_to_playlist(
        &mut self,
        track_id: i64,
        playlist_id: i64,
        cx: &mut Context<Self>,
    ) {
        if let Some(intent) = self.vm.begin_playlist_append(playlist_id, vec![track_id]) {
            self.spawn_subscribe_then_append(intent, cx);
        }
    }

    pub(crate) fn create_playlist_and_add_track(
        &mut self,
        name: &str,
        track_id: i64,
        cx: &mut Context<Self>,
    ) {
        let command = CreatePlaylist::new(Arc::clone(&self.conn), name.to_string());
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, result, cx| {
                let playlist_id = result.playlist_id();
                this.reload_playlists_then(cx, move |this, cx| {
                    this.add_track_to_playlist(track_id, playlist_id, cx);
                });
            },
            |this, err, _cx| this.vm.fail_playlist_create(err),
        );
    }

    pub(crate) fn add_album_to_playlist(
        &mut self,
        feed_id: i64,
        playlist_id: i64,
        cx: &mut Context<Self>,
    ) {
        self.add_albums_to_playlist(&[feed_id], playlist_id, cx);
    }

    /// Adds the tracks of each album to a playlist, in the order of
    /// `feed_ids`, through one append (ADR 0084 Decision 6). A singles
    /// group page uses this path to add its singles newest first.
    pub(crate) fn add_albums_to_playlist(
        &mut self,
        feed_ids: &[i64],
        playlist_id: i64,
        cx: &mut Context<Self>,
    ) {
        let conn = self.conn.lock().expect("lock db");
        let mut tracks = Vec::new();
        for feed_id in feed_ids {
            match db::feed_tracks(&conn, *feed_id) {
                Ok(feed_tracks) => tracks.extend(feed_tracks),
                Err(err) => {
                    drop(conn);
                    self.vm.fail_album_tracks_load(err);
                    cx.notify();
                    return;
                }
            }
        }
        drop(conn);
        let track_ids: Vec<i64> = tracks.iter().map(|t| t.id).collect();
        if track_ids.is_empty() {
            self.vm.set_album_has_no_tracks();
            cx.notify();
            return;
        }
        if let Some(intent) = self.vm.begin_playlist_append(playlist_id, track_ids) {
            self.spawn_subscribe_then_append(intent, cx);
        }
    }

    pub(crate) fn create_playlist_and_add_album(
        &mut self,
        name: &str,
        feed_id: i64,
        cx: &mut Context<Self>,
    ) {
        self.create_playlist_and_add_albums(name, vec![feed_id], cx);
    }

    /// Creates a playlist, then adds the tracks of each album to it in the
    /// order of `feed_ids` (ADR 0084 Decision 6).
    pub(crate) fn create_playlist_and_add_albums(
        &mut self,
        name: &str,
        feed_ids: Vec<i64>,
        cx: &mut Context<Self>,
    ) {
        let command = CreatePlaylist::new(Arc::clone(&self.conn), name.to_string());
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, result, cx| {
                let playlist_id = result.playlist_id();
                this.reload_playlists_then(cx, move |this, cx| {
                    this.add_albums_to_playlist(&feed_ids, playlist_id, cx);
                });
            },
            |this, err, _cx| this.vm.fail_playlist_create(err),
        );
    }

    fn spawn_subscribe_then_append(
        &mut self,
        intent: PlaylistAppendIntent,
        cx: &mut Context<Self>,
    ) {
        let playlist_id = intent.playlist_id();
        let track_ids = intent.track_ids().to_vec();
        cx.notify();

        let command = SubscribeThenAppendToPlaylist::new(
            Arc::clone(&self.conn),
            self.application_services.download_manager(),
            playlist_id,
            track_ids,
        );
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, outcome, cx| {
                this.vm.finish_playlist_append(
                    &intent,
                    PlaylistAppendOutcome::new(
                        outcome.appended(),
                        outcome.downloaded(),
                        outcome.failed().len(),
                    ),
                );
                this.reload_playlists(cx);
            },
            |this, err, cx| {
                this.vm.fail_playlist_append(err);
                this.reload_playlists(cx);
            },
        );
    }

    /// The main color of the cover at `url` for a page header backdrop
    /// (ADR 0083 task 005). The first call starts a background fetch and
    /// gives `None`. The view updates when the color arrives.
    fn cover_color_for_url(
        &mut self,
        url: Option<&str>,
        cx: &mut Context<Self>,
    ) -> Option<crate::media::cover_color::CoverColor> {
        let url = url?.trim();
        if url.is_empty() {
            return None;
        }
        if let Some(color) = self.cache.peek_cover_color(url) {
            return Some(color);
        }
        self.command_runner.availability().ok()?;
        match self.cover_colors.get(url) {
            Some(super::CoverColorState::Loaded(color)) => return *color,
            Some(super::CoverColorState::Loading) => return None,
            None => {}
        }
        let key = url.to_string();
        self.cover_colors
            .insert(key.clone(), super::CoverColorState::Loading);
        let command = FetchCoverColor::new(Arc::clone(&self.cache), key.clone());
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, color, cx| {
                this.cover_colors
                    .insert(key, super::CoverColorState::Loaded(color));
                cx.notify();
            },
            |_, _, _| {},
        );
        None
    }

    fn thumbnail_for_url(
        &mut self,
        url: Option<&str>,
        animated: bool,
        cx: &mut Context<Self>,
    ) -> Option<Arc<Image>> {
        let url = url?.trim();
        if url.is_empty() {
            return None;
        }
        let animated = animated && crate::ui::tokens::Environment::current(cx).allows_motion();
        let cached = if animated {
            self.cache.peek(url)
        } else {
            self.cache.peek_static(url)
        };
        if let Some(img) = cached {
            return Some(img);
        }
        // Hot-cache reads stay available; deferred fetches retry after repair.
        self.command_runner.availability().ok()?;
        let key = (url.to_string(), animated);
        match self.thumbnails.get(&key) {
            Some(ThumbnailState::Loaded(image)) => return image.clone(),
            Some(ThumbnailState::Loading) => return None,
            None => {}
        }
        self.thumbnails.insert(key.clone(), ThumbnailState::Loading);
        let command = FetchThumbnail::new(Arc::clone(&self.cache), key.0.clone(), animated);
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, image, cx| {
                this.thumbnails.insert(key, ThumbnailState::Loaded(image));
                cx.notify();
            },
            |_, _, _| {},
        );
        None
    }

    pub(crate) fn select_album(&mut self, album: &AlbumNode, cx: &mut Context<Self>) {
        self.mount_album(album, true, cx);
    }

    /// Mounts `album` as the Library detail again, for a reload that
    /// keeps the page open. This covers a Library refresh
    /// ([`Self::refresh_selected_detail`]) and the end of a release
    /// download. It sends no request for the `MusicIndex` singles of the
    /// artist feed. [`Self::select_album`] already sent that request
    /// when the page opened. A failed request must not run again on
    /// each reload with no new page open (packet 026 review).
    pub(crate) fn refresh_album_detail(&mut self, album: &AlbumNode, cx: &mut Context<Self>) {
        self.mount_album(album, false, cx);
    }

    /// Shared body of [`Self::select_album`] and
    /// [`Self::refresh_album_detail`]. `request_singles` sends the request
    /// for the `MusicIndex` singles of the artist feed of a Library single,
    /// for "All n singles" (packet 026 decision D14), only on a genuine
    /// page open (packet 026 review).
    fn mount_album(&mut self, album: &AlbumNode, request_singles: bool, cx: &mut Context<Self>) {
        if let Some(feed_id) = album.feed_id {
            self.vm.select_library_item(feed_id);
        } else {
            self.vm.clear_library_selection();
        }
        let mut album = album.clone();
        if let Some(feed_id) = album.feed_id {
            if let Ok(tracks) = self
                .conn
                .lock()
                .map_err(|_| anyhow::anyhow!("database lock poisoned"))
                .and_then(|conn| db::feed_tracks(&conn, feed_id))
            {
                if !tracks.is_empty() {
                    album.tracks = tracks;
                }
            }
        }
        self.detail = LibraryDetail::Album(album.clone());
        self.hydrate_album_identity_on_view(&album, cx);
        if request_singles {
            self.request_library_single_feed(&album, cx);
        }
        if let Some(feed_id) = album.feed_id {
            self.check_feed_on_view(feed_id, cx);
        }
    }

    pub(crate) fn find_album_by_feed_id(&self, feed_id: i64) -> Option<&AlbumNode> {
        for artist_node in &self.vm.tree().artists {
            for album in &artist_node.albums {
                if album.feed_id == Some(feed_id) {
                    return Some(album);
                }
            }
        }
        None
    }

    pub(crate) fn album_for_detail_by_feed_id(&self, feed_id: i64) -> Option<AlbumNode> {
        if let Some(album) = self.find_album_by_feed_id(feed_id) {
            return Some(album.clone());
        }

        let conn = self.conn.lock().ok()?;
        let tracks = db::feed_tracks(&conn, feed_id).ok()?;
        let first = tracks.first()?;
        let language = db::feed_language_by_id(&conn, feed_id).ok().flatten();
        let named_feeds =
            db::publisher_relationships::named_feeds(&conn, feed_id).unwrap_or_default();
        Some(AlbumNode {
            name: first
                .feed_title
                .clone()
                .or_else(|| first.album_title.clone())
                .unwrap_or_else(|| "Untitled Feed".to_string()),
            feed_id: Some(feed_id),
            feed_guid: first.feed_guid.clone(),
            feed_url: db::feed_url_by_id(&conn, feed_id).ok().flatten(),
            language,
            description: None,
            image_href: first
                .album_image_href
                .clone()
                .or_else(|| first.track_image_href.clone()),
            named_feeds,
            identity_facts: crate::local_identity::feed_facts(&conn, feed_id).unwrap_or_default(),
            metadata_facts: Box::new(
                crate::local_metadata::feed_facts(&conn, feed_id).unwrap_or_default(),
            ),
            stored_values: crate::application::queries::stored_values::feed_values(&conn, feed_id)
                .ok()
                .map(Box::new),
            tracks,
        })
    }

    /// Sends `HydrateAlbumIdentity` for one feed through the ADR 0040
    /// runtime, then applies the result in place (ADR 0084 task 034).
    /// The album page view hook, the post-download hooks, and the
    /// once-per-session backfill pass each call this one function, so
    /// they all apply a hydration result the same way.
    ///
    /// `trigger` names the caller and sets the request intent (ADR 0084
    /// task 035). A download and the backfill pass send a new request, so
    /// a kept Index response cannot skip the write of the feed links.
    /// For `AlbumHydrationTrigger::Backfill`, the completion, success or
    /// failure, then starts the next queued release, so the pass runs
    /// at most `ALBUM_IDENTITY_BACKFILL_CONCURRENCY` commands at a time.
    /// Neither closure changes a `DownloadOutcome`: a hydration failure
    /// never marks a download as failed (decision D3).
    fn hydrate_album_identity(
        &mut self,
        feed_id: i64,
        feed_guid: String,
        trigger: AlbumHydrationTrigger,
        cx: &mut Context<Self>,
    ) {
        let command = HydrateAlbumIdentity::new(
            Arc::clone(&self.conn),
            self.musicindex_endpoint.clone(),
            feed_id,
            feed_guid,
            trigger,
        );
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, hydration, cx| {
                apply_library_hydration_result(&mut this.vm, &mut this.detail, feed_id, hydration);
                if trigger == AlbumHydrationTrigger::Backfill {
                    this.pump_album_identity_backfill(cx);
                }
                cx.notify();
            },
            move |this, error, cx| {
                retain_library_query_failure(&mut this.vm, &error);
                if trigger == AlbumHydrationTrigger::Backfill {
                    this.pump_album_identity_backfill(cx);
                }
            },
        );
    }

    /// Finds the Library feed row that a `MusicIndex` download just
    /// wrote, by its feed GUID, then hydrates it (ADR 0084 task 034,
    /// decision D1). Sends no command when `feed_guid` is absent or the
    /// Library has no feed row of that GUID yet: the once-per-session
    /// backfill pass picks up a release this misses.
    pub(super) fn hydrate_album_identity_by_feed_guid(
        &mut self,
        feed_guid: Option<String>,
        cx: &mut Context<Self>,
    ) {
        let Some(feed_guid) = feed_guid else {
            return;
        };
        let feed_id = {
            let Ok(conn) = self.conn.lock() else {
                return;
            };
            db::find_feed_id_by_guid(&conn, &feed_guid).ok().flatten()
        };
        let Some(feed_id) = feed_id else {
            return;
        };
        self.hydrate_album_identity(feed_id, feed_guid, AlbumHydrationTrigger::AfterDownload, cx);
    }

    fn hydrate_album_identity_on_view(&mut self, album: &AlbumNode, cx: &mut Context<Self>) {
        if album_has_feed_identity_actions(&album.identity_facts)
            && album.description.is_some()
            && !album.metadata_facts.is_empty()
        {
            return;
        }
        let (Some(feed_id), Some(feed_guid)) = (album.feed_id, album.feed_guid.clone()) else {
            return;
        };
        self.hydrate_album_identity(feed_id, feed_guid, AlbumHydrationTrigger::PageView, cx);
    }

    /// The number of identity backfill commands that
    /// `start_album_identity_backfill` starts before any reply arrives
    /// (ADR 0084 task 034, decision D2).
    const ALBUM_IDENTITY_BACKFILL_CONCURRENCY: usize = 4;

    /// Starts the once-per-session identity backfill pass (ADR 0084 task
    /// 034, decision D2). Sets `album_identity_backfill_started` first,
    /// so a later Library tree reload never starts a second pass. Builds
    /// the queue of releases that have a feed GUID, no stored row of
    /// `feed_publisher_relationships`, and no prior hydration observation
    /// (Review Corrections 2026-10-22, below), newest feed id first, then
    /// pumps it up to `ALBUM_IDENTITY_BACKFILL_CONCURRENCY` times.
    ///
    /// A feed whose MusicIndex record names no publisher never gains a
    /// stored relationship row, so a candidate read that only checks that
    /// table repeats the feed's request at every app start, forever. The
    /// second read below excludes a feed whose hydration-shaped request
    /// already observed a `success` or a `partial` result, from the
    /// retained evidence of ADR 0075. A read failure reports through the
    /// Library status path and leaves the queue empty for this session;
    /// it never leaves the failure unreported.
    fn start_album_identity_backfill(&mut self, cx: &mut Context<Self>) {
        self.album_identity_backfill_started = true;
        let candidates = album_identity_backfill_order(self.vm.tree());
        if candidates.is_empty() {
            return;
        }
        let feed_ids: Vec<i64> = candidates.iter().map(|(feed_id, _)| *feed_id).collect();
        let feed_guids: Vec<String> = candidates
            .iter()
            .map(|(_, feed_guid)| feed_guid.clone())
            .collect();
        let provider_identity = self
            .musicindex_endpoint
            .require()
            .map(str::to_owned)
            .unwrap_or_default();
        let include = crate::application::request_profiles::LIBRARY_ALBUM_HYDRATION_FEED
            .include()
            .unwrap_or_default();
        let (needing, observed) = {
            let conn = match self.conn.lock() {
                Ok(conn) => conn,
                Err(_) => {
                    self.vm.set_error_status("database lock poisoned");
                    return;
                }
            };
            let needing = match db::publisher_relationships::feed_ids_with_no_stored_relationship(
                &conn, &feed_ids,
            ) {
                Ok(ids) => ids,
                Err(error) => {
                    self.vm.set_error_status(error);
                    return;
                }
            };
            let observed = match db::provider_observations::feed_guids_with_hydration_observation(
                &conn,
                &provider_identity,
                include,
                &feed_guids,
            ) {
                Ok(guids) => guids,
                Err(error) => {
                    self.vm.set_error_status(error);
                    return;
                }
            };
            (needing, observed)
        };
        let needing: BTreeSet<i64> = needing.into_iter().collect();
        self.album_identity_backfill_queue = candidates
            .into_iter()
            .filter(|(feed_id, feed_guid)| {
                needing.contains(feed_id) && !observed.contains(feed_guid)
            })
            .collect();
        for _ in 0..Self::ALBUM_IDENTITY_BACKFILL_CONCURRENCY {
            self.pump_album_identity_backfill(cx);
        }
    }

    /// Starts the next queued identity backfill command, when one is
    /// queued (ADR 0084 task 034, decision D2). `hydrate_album_identity`
    /// calls this again from its own completion, so the pass keeps at
    /// most `ALBUM_IDENTITY_BACKFILL_CONCURRENCY` commands running.
    fn pump_album_identity_backfill(&mut self, cx: &mut Context<Self>) {
        let Some((feed_id, feed_guid)) = self.album_identity_backfill_queue.pop_front() else {
            return;
        };
        self.hydrate_album_identity(feed_id, feed_guid, AlbumHydrationTrigger::Backfill, cx);
    }

    /// Synchronize LibraryApp detail state to match a navigation entry.
    ///
    /// Maps the nav entry to the appropriate Library detail state:
    /// - `PlaylistDetail(id)` → call `select_playlist` (ADR 0084 Decision 9)
    /// - `Search(_)` / others → clear detail to root state
    ///
    /// A `Release` entry and a `Track` entry mount through the top app's
    /// resolved data source (ADR 0084 Decision 4), not through this
    /// generic hydrate path.
    pub(crate) fn hydrate_detail_from_nav(
        &mut self,
        entry: &FrameNavigationEntry,
        cx: &mut Context<Self>,
    ) {
        // ADR 0084 packet 018: a page change clears the detail selection
        // and sets `Normal`.
        self.reset_detail_selection(entry);
        match entry {
            FrameNavigationEntry::PlaylistDetail(playlist_id) => {
                self.select_playlist(*playlist_id, cx);
            }
            // ADR 0084 Decision 2: a destination becoming the current
            // location fills the list column with its items and clears
            // the detail column.
            FrameNavigationEntry::Destination(destination) => {
                self.show_destination_list(*destination, cx);
                // ADR 0089 packet 011: the Live list reads again each time
                // its page shows, so a Back to it starts a new read.
                if *destination == MusicDestination::Live {
                    self.reload_live_list(cx);
                }
                // ADR 0089 Decision 5: the scope "All" sends its first name
                // request when the destination shows, once.
                self.request_default_name_list(cx);
                // ADR 0089 Decision 4, packet 007 Part C: opening the Search
                // page starts the new-results count, at most once per interval.
                if *destination == MusicDestination::Search {
                    self.request_saved_search_counts(cx);
                }
                self.clear_detail();
            }
            // ADR 0084 Decision 3, packet 012: a saved search is a page
            // under Discover. The top app renders it in the detail column.
            FrameNavigationEntry::SavedSearch { .. } => {
                self.vm.set_active_destination(MusicDestination::Search);
                self.clear_detail();
            }
            // ADR 0084 Decision 6: the group page reads the tree at each
            // render. Packet 026 Part B: each open of the page reads the
            // MusicIndex singles of its feed when the app has no result.
            FrameNavigationEntry::SinglesGroup(key) => {
                self.vm.clear_library_selection();
                self.detail = LibraryDetail::SinglesGroup(key.clone());
                self.request_singles_group_feed(key, cx);
            }
            // ADR 0086 Decision 4: the window group page reads its rows
            // from its key. Each open reads the MusicIndex singles of the
            // feed, so the link to the group page can show.
            FrameNavigationEntry::WindowGroup(key) => {
                self.vm.clear_library_selection();
                self.detail = LibraryDetail::WindowGroup(key.clone());
                self.request_singles_group_feed(&key.group, cx);
            }
            // ADR 0089 packet 009: the item page of a public playlist reads
            // the playlist with its remote items, unless the page holds it.
            FrameNavigationEntry::PublicPlaylist(feed_guid) => {
                self.vm.clear_library_selection();
                self.clear_detail();
                self.request_public_playlist(feed_guid, cx);
            }
            // ADR 0084 Decision 7: the top app renders the name level page,
            // as it renders the feed page.
            FrameNavigationEntry::Search(_)
            | FrameNavigationEntry::ArtistName(_)
            | FrameNavigationEntry::Settings => {
                // Reset detail to default/unset state for root navigation
                self.clear_detail();
            }
            // `Release` and `Track` mount through the top app. `QueueNowPlaying`
            // doesn't drive library detail.
            _ => {}
        }
    }

    /// Fills the list column with the items of `destination` (ADR 0084
    /// Decision 2). Each branch reuses data this entity already holds or
    /// already loads: no new query runs on the render path.
    fn show_destination_list(&mut self, destination: MusicDestination, cx: &mut Context<Self>) {
        self.vm.set_active_destination(destination);
        // ADR 0084 Decision 10: a change of destination clears both
        // selections, including the keyboard focus: the new
        // destination's rows are a different list. Amendment,
        // 2026-10-19: it also ends selection mode for both lists.
        self.releases_selection = ListSelection::new();
        self.track_selection = ListSelection::new();
        self.releases_selection_mode = SelectionMode::Normal;
        self.track_selection_mode = SelectionMode::Normal;
        match destination {
            // ADR 0084 Decision 3, ADR 0089 Decision 2: Discover's list
            // column shows its saved searches, not the content list.
            MusicDestination::Search
            | MusicDestination::Releases
            | MusicDestination::Artists
            | MusicDestination::Playlists
            | MusicDestination::PublicPlaylists
            | MusicDestination::Live => {
                // The Discover, Releases, Artists, and Playlists lists read
                // straight from already-loaded data; they hold no
                // `ContentListPageVm` state of their own.
            }
            MusicDestination::Inbox => {
                self.open_inbox_destination(cx);
            }
        }
    }

    /// Clear the detail state, resetting to library root.
    pub(crate) fn clear_detail(&mut self) {
        self.detail = LibraryDetail::None;
        self.vm.clear_library_selection();
    }

    fn check_feed_on_view(&mut self, feed_id: i64, cx: &mut Context<Self>) {
        if !self.vm.begin_feed_view_check(feed_id) {
            return;
        }
        let command = CheckFeedStaleness::new(
            Arc::clone(&self.conn),
            self.musicindex_endpoint.clone(),
            feed_id,
        );
        cx.notify();
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, outcome, _cx| {
                apply_feed_view_check_result(&mut this.vm, outcome);
            },
            move |this, error, _cx| {
                retain_library_query_failure(&mut this.vm, &error);
                this.vm.finish_feed_view_check_error(feed_id, error);
            },
        );
    }

    pub(crate) fn check_all_feeds(&mut self, cx: &mut Context<Self>) {
        if self.vm.feed_update_state().phase != FeedUpdatePhase::Idle {
            return;
        }
        let feeds = {
            let conn = match self.conn.lock() {
                Ok(conn) => conn,
                Err(_) => {
                    self.vm.set_feed_check_error("database lock poisoned");
                    cx.notify();
                    return;
                }
            };
            match self
                .application_services
                .query_service()
                .subscribed_feeds_for_stale_check(&conn)
            {
                Ok(rows) => rows,
                Err(err) => {
                    self.vm.set_feed_check_error(err);
                    cx.notify();
                    return;
                }
            }
        };
        self.vm.begin_all_feed_check(feeds.len());
        cx.notify();

        let command = CheckFeedsAndRepairRoutes::new(
            Arc::clone(&self.conn),
            self.musicindex_endpoint.clone(),
            self.music_dir.clone(),
            feeds,
        );
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            |this, outcome, cx| {
                let repair = retain_feed_check_evidence(&mut this.vm, &outcome);
                this.vm.finish_all_feed_check_with_route_repair(repair);
                this.refresh_current_broadcast_readiness_report(cx);
                this.scan_tag_updates();
            },
            |this, error, cx| {
                retain_library_query_failure(&mut this.vm, &error);
                this.vm.set_feed_check_error(error);
                this.refresh_current_broadcast_readiness_report(cx);
            },
        );
    }

    pub(crate) fn apply_all_feed_updates(&mut self, cx: &mut Context<Self>) {
        let Some(stale) = self.vm.begin_apply_feed_updates() else {
            return;
        };
        cx.notify();

        let command = ApplyFeedUpdates::new(
            Arc::clone(&self.conn),
            self.musicindex_endpoint.clone(),
            stale,
        );
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            |this, outcome, _cx| {
                apply_feed_updates_result(&mut this.vm, &outcome);
                // ADR 0076 Decision 8: the feed update changes the database
                // only. The scan then offers the file write.
                this.scan_tag_updates();
            },
            |this, error, _cx| {
                retain_library_query_failure(&mut this.vm, &error);
                this.vm.finish_apply_feed_updates_error(error);
            },
        );
    }

    fn repair_broadcast_routes_for_track(&mut self, track_id: i64, cx: &mut Context<Self>) {
        if !self.vm.begin_broadcast_route_repair(track_id) {
            return;
        }
        cx.notify();

        let command = RepairPaymentRoutesForTrack::new(
            Arc::clone(&self.conn),
            self.musicindex_endpoint.clone(),
            self.music_dir.clone(),
            track_id,
        );
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            |this, outcome, cx| {
                let completion = broadcast_route_repair_completion(&outcome);
                this.vm.finish_broadcast_route_repair(&completion);
                this.refresh_current_broadcast_readiness_report(cx);
            },
            move |this, error, cx| {
                this.vm.fail_broadcast_route_repair(track_id, error);
                this.refresh_current_broadcast_readiness_report(cx);
            },
        );
    }

    /// Mounts the track page. It changes no navigation state.
    pub(crate) fn select_track(&mut self, track: &TrackRow, cx: &mut Context<Self>) {
        self.select_track_detail(track, cx);
    }

    fn select_track_detail(&mut self, track: &TrackRow, cx: &mut Context<Self>) {
        self.vm.select_library_item(track.id);
        let image = track
            .track_image_href
            .as_deref()
            .or(track.album_image_href.as_deref())
            .and_then(|url| self.thumbnail_for_url(Some(url), true, cx));
        let mut frame = InspectorFrame::for_track(track.clone(), image);
        frame.inspector_state.description_state = self.vm.track_description_state(track.id, None);
        // ADR 0083 task 006, decision D4: "Inspect sources" of a track row
        // opens this page with its "Inspect sources" disclosure open.
        if take_inspect_sources_request(&mut self.pending_inspect_sources, track.id) {
            frame
                .inspector_state
                .expand_panel(InspectorPanelKind::Sources);
            cx.emit(super::LibraryAppEvent::OpenInspectSourcesSheet);
        }
        if let Some(lookup) = self.vm.staged_musicbrainz(track.id).cloned() {
            frame.musicbrainz_lookup = LazyPanel::Loaded(lookup);
            frame.musicbrainz_selected = 0;
        }
        self.detail = LibraryDetail::Track(Box::new(frame));
        cx.notify();
        self.load_track_source_context(track.clone(), cx);
    }

    fn load_track_source_context(&mut self, track: TrackRow, cx: &mut Context<Self>) {
        let entity_id = track.id;
        let command = FetchLibraryTrackContext::new(
            Arc::clone(&self.conn),
            track,
            self.musicindex_endpoint.clone(),
        );
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, context, _cx| {
                if let Some(frame) = this.selected_track_frame_mut() {
                    retain_track_source_context(frame, entity_id, context);
                }
            },
            |this, error, _| this.vm.retain_observation_failure(&error),
        );
    }

    /// The named feeds of the mounted track's album feed (ADR 0085
    /// Decisions 5 and 6, carries ADR 0077 packet 004 R4-03). The
    /// already-loaded Library tree carries this value; the screen sends no
    /// request for it.
    pub(crate) fn mounted_track_named_feeds(
        &self,
    ) -> Vec<crate::view_models::named_feeds::NamedFeed> {
        let LibraryDetail::Track(frame) = &self.detail else {
            return Vec::new();
        };
        self.find_album_by_feed_id(frame.track.feed_id)
            .map(|album| album.named_feeds.clone())
            .unwrap_or_default()
    }

    /// The title of a Library page (ADR 0084 Decision 9). The
    /// top app owns the page title. The Library gives only the names it has.
    pub(crate) fn navigation_label(&self, entry: &FrameNavigationEntry) -> String {
        match entry {
            FrameNavigationEntry::PlaylistDetail(playlist_id) => self
                .vm
                .playlist_by_id(*playlist_id)
                .map_or_else(|| "Playlist".to_string(), |playlist| playlist.name),
            FrameNavigationEntry::Release(key) => self
                .release_label(key)
                .unwrap_or_else(|| "Release".to_string()),
            // ADR 0084 Decision 6: the page title is the label.
            FrameNavigationEntry::SinglesGroup(key) => {
                let page = self.singles_group_page(key);
                page.title
            }
            // ADR 0086 Decision 4: the page title is the label.
            FrameNavigationEntry::WindowGroup(key) => self.vm.window_group_page(key).title,
            _ => entry.display_label(),
        }
    }

    /// The Library album name of a release key, when the Library holds that
    /// release (ADR 0084 Decision 4). The label of each release
    /// segment by its own key.
    pub(crate) fn release_label(&self, key: &ReleaseKey) -> Option<String> {
        match key {
            ReleaseKey::LibraryFeed(feed_id) => self
                .album_for_detail_by_feed_id(*feed_id)
                .map(|album| album.name),
            ReleaseKey::FeedGuid(feed_guid) => self
                .vm
                .tree()
                .artists
                .iter()
                .flat_map(|artist| &artist.albums)
                .find(|album| album.feed_guid.as_deref() == Some(feed_guid.as_str()))
                .map(|album| album.name.clone()),
        }
    }

    /// The Library track title of a track key, when the Library holds that
    /// track (ADR 0084 Decision 4). The label of each track
    /// segment by its own key.
    pub(crate) fn track_label(&self, key: &TrackKey) -> Option<String> {
        self.vm
            .tree()
            .artists
            .iter()
            .flat_map(|artist| &artist.albums)
            .flat_map(|album| &album.tracks)
            .find(|track| track.track_key() == *key)
            .map(|track| LibraryTrackRowVm::new(track, None).display_title())
    }

    /// Runs one command of a release row menu (ADR 0084 task 031). The
    /// Releases list, the feed page and the name page use this one host.
    /// Each command uses the path of the matching action of the selection
    /// action bar, for one release. "Remove <kind>…" uses the path of the
    /// release page menu.
    pub(crate) fn run_release_row_command(
        &mut self,
        target: &ReleaseRowTarget,
        command: ReleaseRowCommand,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match (command, target.feed_id) {
            (ReleaseRowCommand::Download, _) => {
                let plan = ReleasesDownloadPlan {
                    library_feed_ids: target.feed_id.into_iter().collect(),
                    index_releases: target
                        .index_feed_guid()
                        .map(|feed_guid| (feed_guid.to_owned(), Some(target.title.clone())))
                        .into_iter()
                        .collect(),
                };
                self.download_release_plan(plan, cx);
            }
            (ReleaseRowCommand::AddToPlaylist(playlist_id), Some(feed_id)) => {
                self.add_albums_to_playlist(&[feed_id], playlist_id, cx);
            }
            (ReleaseRowCommand::CreatePlaylist(name), Some(feed_id)) => {
                self.create_playlist_and_add_albums(&name, vec![feed_id], cx);
            }
            (ReleaseRowCommand::Remove, Some(feed_id)) => {
                self.remove_album_after_confirmation(feed_id, window, cx);
            }
            // An Index release has no "Add to playlist" and no "Remove"
            // item (task 031, decision D5).
            (
                ReleaseRowCommand::AddToPlaylist(_)
                | ReleaseRowCommand::CreatePlaylist(_)
                | ReleaseRowCommand::Remove,
                None,
            ) => {}
        }
    }

    /// The context of the release row menus of one list (ADR 0084 task
    /// 031): the playlists, the focus that the right-click menu gives
    /// back, and the handler that runs each command in this entity.
    pub(crate) fn release_row_menu_context(
        &self,
        entity: gpui::WeakEntity<Self>,
        return_focus: gpui::FocusHandle,
    ) -> ReleaseRowMenuContext {
        ReleaseRowMenuContext {
            playlists: std::rc::Rc::new(playlist_options(self.vm.playlists())),
            return_focus,
            on_command: std::rc::Rc::new(move |target, command, window, cx| {
                if let Some(entity) = entity.upgrade() {
                    entity.update(cx, |this, cx| {
                        this.run_release_row_command(target, command, window, cx);
                    });
                }
            }),
        }
    }

    /// The menu of a release row of a feed page or a name page (ADR 0084
    /// task 031). The Releases list rows get the same menu from the view
    /// model.
    #[must_use]
    pub(crate) fn release_row_menu(
        &self,
        row_id: &str,
        key: &ReleaseKey,
        title: &str,
        kind: ReleaseKind,
    ) -> Option<ReleaseRowMenuVm> {
        let selecting = self.detail_selection().mode().is_selecting();
        self.vm
            .release_row_menu(row_id, key, title, kind, selecting)
    }

    /// "Remove album…" of an album page: a destructive menu item, so it
    /// asks for each removal (ADR 0083 Decision 5).
    pub(crate) fn remove_album_after_confirmation(
        &mut self,
        feed_id: i64,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.request_library_removal(
            LibraryRemovalIntent::FeedId(feed_id),
            RemovalConfirmation::Always,
            window,
            cx,
        );
    }

    /// Runs one Library release job of the download queue (ADR 0084
    /// Decision 10). Each completion reports to the queue, and the queue
    /// starts the next job.
    pub(super) fn run_library_feed_job(
        &mut self,
        identity: JobIdentity,
        feed_id: i64,
        cx: &mut Context<Self>,
    ) {
        let Some(feed) = self.api_feed_for_album(feed_id) else {
            let outcome = DownloadOutcome::Failed {
                error: "the release has no tracks.".to_owned(),
            };
            self.finish_download(&identity, outcome, cx);
            return;
        };
        // ADR 0084 Decision 10 as amended 2026-10-24, task 039: the RSS
        // request waits for its host.
        let resume_identity = identity.clone();
        if !self.await_host_turn(
            &identity,
            feed.feed_url.as_deref(),
            move |this, cx| this.run_library_feed_job(resume_identity, feed_id, cx),
            cx,
        ) {
            return;
        }
        // ADR 0084 task 034, decision D1: hydrate this feed's identity
        // after the download succeeds, so it joins its feed group with
        // no page view.
        let hydrate_feed_guid = feed.feed_guid.clone();
        let failed_identity = identity.clone();
        let command = SubscribeFeed::new(
            Arc::clone(&self.conn),
            self.application_services.download_manager(),
            SubscribeFeedRequest {
                feed,
                musicindex_endpoint: self.musicindex_endpoint.clone(),
            },
        );
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, _result, cx| {
                if let Some(feed_guid) = hydrate_feed_guid {
                    this.hydrate_album_identity(
                        feed_id,
                        feed_guid,
                        AlbumHydrationTrigger::AfterDownload,
                        cx,
                    );
                }
                this.refresh_origin_playlist_actor();
                this.start_async_reload_preserving_detail(cx);
                this.finish_download(&identity, DownloadOutcome::Succeeded, cx);
            },
            move |this, error, cx| {
                if let CommandError::HostThrottled { host, retry_after } = &error {
                    let job = DownloadJob::LibraryFeed { feed_id };
                    this.finish_host_throttled(&failed_identity, job, host, *retry_after, cx);
                    return;
                }
                let outcome = DownloadOutcome::Failed {
                    error: error.to_string(),
                };
                this.finish_download(&failed_identity, outcome, cx);
            },
        );
    }

    /// "Remove track…" of the track page: the removal always asks for
    /// confirmation (ADR 0083 Decision 5).
    pub(crate) fn remove_track_after_confirmation(
        &mut self,
        track_id: i64,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.request_library_removal(
            LibraryRemovalIntent::TrackId(track_id),
            RemovalConfirmation::Always,
            window,
            cx,
        );
    }

    fn request_library_removal(
        &mut self,
        intent: LibraryRemovalIntent,
        confirmation: RemovalConfirmation,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let plan_result = {
            let conn = self.conn.lock().expect("lock db");
            self.application_services
                .query_service()
                .library_removal_plan(&conn, &intent)
        };
        let plan = match plan_result {
            Ok(plan) => plan,
            Err(err) => {
                self.vm.set_error_status(err);
                cx.notify();
                return;
            }
        };
        if !self.vm.confirm_library_removal(plan, confirmation) {
            let Some(display) = self.vm.pending_library_removal_confirmation() else {
                cx.notify();
                return;
            };
            open_library_removal_confirmation_dialog(
                window,
                cx,
                display,
                |this, cx| {
                    this.vm.cancel_pending_library_removal();
                    cx.notify();
                },
                |this, cx| {
                    this.execute_pending_library_removal(cx);
                },
            );
            cx.notify();
            return;
        }

        self.execute_library_removal_target(plan.target(), cx);
    }

    fn execute_library_removal_target(
        &mut self,
        target: LibraryRemovalTarget,
        cx: &mut Context<Self>,
    ) {
        if let LibraryRemovalTarget::Feed(feed_id) = target {
            // ADR 0084 Decision 10: a removal of a release waits until no
            // download of that release is queued or running.
            if let Some(refusal) = self.release_removal_refusal(feed_id) {
                if !self.record_selection_removal_result(target, Some(refusal.clone())) {
                    self.vm.set_status(refusal);
                }
                cx.notify();
                return;
            }
            self.vm.begin_feed_removal(feed_id);
            cx.notify();
        }
        let command = RemoveFromLibrary::new(Arc::clone(&self.conn), target);
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, result, cx| {
                this.finish_feed_removal(target);
                this.record_selection_removal_result(target, None);
                this.apply_library_removal_result_to_selected_detail(result.target());
                // A mounted readiness list drops the removed track in place.
                this.refresh_current_broadcast_readiness_report(cx);
                this.refresh_origin_playlist_actor();
                this.start_async_reload_preserving_detail(cx);
            },
            move |this, err, _cx| {
                this.finish_feed_removal(target);
                if !this.record_selection_removal_result(target, Some(format!("{err:#}"))) {
                    this.vm.set_error_status(err);
                }
            },
        );
    }

    fn finish_feed_removal(&mut self, target: LibraryRemovalTarget) {
        if let LibraryRemovalTarget::Feed(feed_id) = target {
            self.vm.finish_feed_removal(feed_id);
        }
    }

    fn apply_library_removal_result_to_selected_detail(&mut self, target: LibraryRemovalTarget) {
        apply_library_removal_to_album_detail(&mut self.detail, target);

        if let Some(frame) = self.selected_track_frame_mut() {
            apply_library_removal_to_inspector_frame(frame, target);
        }
    }

    fn execute_pending_library_removal(&mut self, cx: &mut Context<Self>) {
        let Some(target) = self.vm.take_pending_library_removal() else {
            cx.notify();
            return;
        };
        self.execute_library_removal_target(target, cx);
    }

    fn api_feed_for_album(&self, feed_id: i64) -> Option<crate::api::Feed> {
        let album = self.album_for_detail_by_feed_id(feed_id)?;
        Some(api_feed_from_album(&album))
    }

    /// Runs one Library track job of the download queue (ADR 0084
    /// Decision 10). The completion updates the open release page and the
    /// open track page of the track in place.
    pub(super) fn run_library_track_job(
        &mut self,
        identity: JobIdentity,
        track_id: i64,
        cx: &mut Context<Self>,
    ) {
        let track = match self.conn.lock() {
            Ok(conn) => db::track_row_by_id(&conn, track_id),
            Err(_) => Err(anyhow::anyhow!("The database connection is not available.")),
        };
        let track = match track {
            Ok(Some(track)) => track,
            Ok(None) => {
                let error = "the track is no longer stored.".to_owned();
                self.fail_library_track_job(&identity, track_id, error, cx);
                return;
            }
            Err(error) => {
                let error =
                    format!("the app could not read the track from the database. {error:#}");
                self.fail_library_track_job(&identity, track_id, error, cx);
                return;
            }
        };
        // ADR 0084 Decision 10 as amended 2026-10-24, task 039: the RSS
        // request waits for its host.
        let feed_url = match self.conn.lock() {
            Ok(conn) => db::feed_url_by_id(&conn, track.feed_id).ok().flatten(),
            Err(_) => None,
        };
        let resume_identity = identity.clone();
        if !self.await_host_turn(
            &identity,
            feed_url.as_deref(),
            move |this, cx| this.run_library_track_job(resume_identity, track_id, cx),
            cx,
        ) {
            return;
        }
        // ADR 0084 task 034, decision D1: hydrate this track's feed after
        // the download succeeds, the same reasoning as the Library feed
        // job. This task's own reading of D1 extends the hook to the
        // Library track job too (see the packet's "Orchestrator
        // Decisions", D4): a Library track can be the only track of a
        // single-track feed that was never hydrated.
        let hydrate_feed = (track.feed_id, track.feed_guid.clone());
        let failed_identity = identity.clone();
        let command = SubscribeTrack::new(
            Arc::clone(&self.conn),
            self.application_services.download_manager(),
            SubscribeTrackRequest::LibraryTrack {
                track: Box::new(track),
            },
            LibraryTrackActionVm::track_subscribe_success_message(),
        );
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, result, cx| {
                if let (feed_id, Some(feed_guid)) = hydrate_feed {
                    this.hydrate_album_identity(
                        feed_id,
                        feed_guid,
                        AlbumHydrationTrigger::AfterDownload,
                        cx,
                    );
                }
                apply_track_subscription_to_album_detail(
                    &mut this.detail,
                    track_id,
                    result.relative_path().cloned(),
                    result.marked_downloaded(),
                );
                // ADR 0084 Decision 10: the progress item reports each
                // job. Only a format warning also goes to the status line.
                if let Some(warning) = result.format_warning() {
                    this.vm.finish_track_subscribe(TrackSubscribeOutcome::new(
                        result.path().to_string(),
                        Some(warning.to_string()),
                    ));
                }
                if let Some(frame) = this.selected_track_frame_mut() {
                    if frame.entity_id == track_id {
                        frame.subscription_busy = false;
                        frame.local_subscription = result.marked_downloaded();
                        frame.track.is_in_library = result.marked_downloaded();
                        if result.marked_downloaded() {
                            frame.track.local_path = result.relative_path().cloned();
                        }
                        frame.source_context = None;
                        frame.tag_compare = LazyPanel::Hidden;
                        frame.pending_id3_edits.clear();
                        frame.suppressed_auto_id3_edits.clear();
                        frame.id3_apply_error = None;
                        frame.subscription_message = Some(result.message().into());
                    }
                }
                this.refresh_origin_playlist_actor();
                this.start_async_reload_preserving_detail(cx);
                this.scan_tag_updates();
                this.finish_download(&identity, DownloadOutcome::Succeeded, cx);
            },
            move |this, error, cx| {
                if let CommandError::HostThrottled { host, retry_after } = &error {
                    let job = DownloadJob::LibraryTrack { track_id };
                    this.finish_host_throttled(&failed_identity, job, host, *retry_after, cx);
                    return;
                }
                this.fail_library_track_job(&failed_identity, track_id, error.to_string(), cx);
            },
        );
    }

    /// Records a failed Library track job, and shows the error on the open
    /// track page of the track.
    fn fail_library_track_job(
        &mut self,
        identity: &JobIdentity,
        track_id: i64,
        error: String,
        cx: &mut Context<Self>,
    ) {
        if let Some(frame) = self.selected_track_frame_mut() {
            if frame.entity_id == track_id {
                frame.subscription_busy = false;
                frame.subscription_message = Some(
                    LibraryTrackActionVm::subscription_error_message(true, &error),
                );
            }
        }
        self.finish_download(identity, DownloadOutcome::Failed { error }, cx);
    }

    fn selected_track_frame_mut(&mut self) -> Option<&mut InspectorFrame> {
        match &mut self.detail {
            LibraryDetail::Track(frame) => Some(frame),
            LibraryDetail::None
            | LibraryDetail::Album(_)
            | LibraryDetail::Playlist(_)
            | LibraryDetail::SinglesGroup(_)
            | LibraryDetail::WindowGroup(_) => None,
        }
    }

    fn stage_musicbrainz_lookup_for_track(
        &mut self,
        track_id: i64,
        lookup: MusicBrainzLookupResult,
    ) {
        self.vm.stage_musicbrainz(track_id, lookup.clone());
        if let Some(frame) = self.selected_track_frame_mut() {
            if frame.entity_id == track_id {
                frame.musicbrainz_lookup = LazyPanel::Loaded(lookup);
                frame.musicbrainz_selected = 0;
                frame.id3_apply_error = None;
            }
        }
    }

    pub(crate) fn toggle_id3_frame_group(&mut self, group_key: String, cx: &mut Context<Self>) {
        let Some(frame) = self.selected_track_frame_mut() else {
            return;
        };
        if !frame.expanded_id3_frame_groups.remove(&group_key) {
            frame.expanded_id3_frame_groups.insert(group_key);
        }
        cx.notify();
    }

    pub(crate) fn toggle_metadata_cell(&mut self, cell_key: String, cx: &mut Context<Self>) {
        let Some(frame) = self.selected_track_frame_mut() else {
            return;
        };
        if !frame.expanded_metadata_cells.remove(&cell_key) {
            frame.expanded_metadata_cells.insert(cell_key);
        }
        cx.notify();
    }

    pub(crate) fn apply_pending_id3_edits(&mut self, cx: &mut Context<Self>) {
        let Some(frame) = self.selected_track_frame_mut() else {
            return;
        };
        if frame.applying_id3_edits {
            return;
        }
        let LazyPanel::Loaded(result) = &frame.tag_compare else {
            return;
        };
        let fallback_context = track_row_to_track_context(&frame.track);
        let track_context = frame.source_context.clone().unwrap_or(fallback_context);
        let rows = track_metadata_rows_for_frame(frame, &track_context, Some(result));
        let pending_id3_edits = auto_populated_pending_id3_edits(
            &rows,
            &frame.pending_id3_edits,
            &frame.suppressed_auto_id3_edits,
            result.format,
        );
        if pending_id3_edits.is_empty() {
            return;
        }
        let conflicts = pending_id3_conflict_descriptions(&pending_id3_edits);
        if !conflicts.is_empty() {
            frame.id3_apply_error = Some(TrackMetadataActionState::duplicate_id3_target_message(
                &conflicts,
            ));
            cx.notify();
            return;
        }

        let entity_id = frame.entity_id;
        let track_id = frame.track.id;
        let path = PathBuf::from(result.path.clone());
        let edits = pending_id3_edits_for_apply(&pending_id3_edits);
        frame.applying_id3_edits = true;
        frame.id3_apply_error = None;
        cx.notify();

        let command = ApplyTrackId3Edits::new(path, edits, track_context)
            .with_stored_route(Arc::clone(&self.conn), track_id);
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, result, _cx| {
                if let Some(frame) = this.selected_track_frame_mut() {
                    if frame.entity_id == entity_id {
                        frame.applying_id3_edits = false;
                        frame.tag_compare = LazyPanel::Loaded(result);
                        frame.pending_id3_edits.clear();
                        frame.suppressed_auto_id3_edits.clear();
                        frame.id3_apply_error = None;
                    }
                }
                // ADR 0076 Decision 8: the button count follows the file.
                this.scan_tag_updates();
            },
            move |this, error, _cx| {
                if let Some(frame) = this.selected_track_frame_mut() {
                    if frame.entity_id == entity_id {
                        frame.applying_id3_edits = false;
                        frame.id3_apply_error =
                            Some(TrackMetadataActionState::id3_apply_error_message(error));
                    }
                }
            },
        );
    }

    pub(crate) fn clear_pending_id3_edits(&mut self, cx: &mut Context<Self>) {
        let Some(frame) = self.selected_track_frame_mut() else {
            return;
        };
        if frame.applying_id3_edits {
            return;
        }
        frame.pending_id3_edits.clear();
        frame.suppressed_auto_id3_edits.clear();
        frame.id3_apply_error = None;
        cx.notify();
    }

    pub(crate) fn toggle_local_subscription(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(action) = (match self.selected_track_frame_mut() {
            Some(frame) if frame.subscription_busy => return,
            Some(frame) if frame.local_subscription => {
                Some(TrackSubscriptionAction::Remove(frame.entity_id))
            }
            Some(frame) => {
                frame.subscription_busy = true;
                frame.subscription_message =
                    Some(LibraryTrackActionVm::subscription_busy_message(true).into());
                Some(TrackSubscriptionAction::Download(Box::new(
                    frame.track.clone(),
                )))
            }
            None => None,
        }) else {
            return;
        };
        let track = match action {
            TrackSubscriptionAction::Remove(track_id) => {
                self.request_library_removal(
                    LibraryRemovalIntent::TrackId(track_id),
                    RemovalConfirmation::WhenReferenced,
                    window,
                    cx,
                );
                return;
            }
            TrackSubscriptionAction::Download(track) => track,
        };
        // ADR 0084 Decision 10: the track page queues one job. The page
        // stays busy only while a job of this track is queued or in
        // flight, because the queue drops the track when a download of its
        // release is queued or running.
        let track_id = track.id;
        self.download_library_tracks(&[track_id], cx);
        if !self.library_track_download_active(track_id) {
            if let Some(frame) = self.selected_track_frame_mut() {
                if frame.entity_id == track_id {
                    frame.subscription_busy = false;
                    frame.subscription_message = None;
                }
            }
        }
        cx.notify();
    }

    pub(crate) fn toggle_tag_compare(&mut self, cx: &mut Context<Self>) {
        let Some(frame) = self.selected_track_frame_mut() else {
            return;
        };
        if !frame.local_subscription || frame.track.local_path.is_none() {
            return;
        }
        let expanded = frame.toggle_inspector_panel(InspectorPanelKind::CompareId3);
        if !expanded {
            cx.notify();
            return;
        }
        match frame.tag_compare {
            LazyPanel::Loaded(_) => {
                cx.notify();
                return;
            }
            LazyPanel::Loading => return,
            LazyPanel::Empty(_) | LazyPanel::Hidden => {
                frame.tag_compare = LazyPanel::Loading;
            }
        }

        let entity_id = frame.entity_id;
        let track = frame.track.clone();
        cx.notify();

        self.start_compare_library_track(entity_id, track, cx);
    }

    pub(crate) fn redownload_tag_compare(&mut self, cx: &mut Context<Self>) {
        self.reload_tag_compare(cx);
    }

    pub(crate) fn reread_tag_compare(&mut self, cx: &mut Context<Self>) {
        self.reload_tag_compare(cx);
    }

    fn reload_tag_compare(&mut self, cx: &mut Context<Self>) {
        let Some(frame) = self.selected_track_frame_mut() else {
            return;
        };
        if !matches!(frame.tag_compare, LazyPanel::Loaded(_)) {
            return;
        }
        frame.tag_compare = LazyPanel::Loading;
        let entity_id = frame.entity_id;
        let track = frame.track.clone();
        cx.notify();

        self.start_compare_library_track(entity_id, track, cx);
    }

    fn start_compare_library_track(
        &mut self,
        entity_id: i64,
        track: TrackRow,
        cx: &mut Context<Self>,
    ) {
        let command = CompareLibraryTrack::new(
            Arc::clone(&self.conn),
            track,
            self.musicindex_endpoint.clone(),
            self.music_dir.clone(),
        );
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, result, _cx| {
                apply_library_comparison_result(&mut this.vm, &mut this.detail, entity_id, result);
            },
            move |this, error, _cx| {
                apply_library_comparison_failure(&mut this.vm, &mut this.detail, entity_id, error);
            },
        );
    }

    pub(crate) fn toggle_musicbrainz_lookup(&mut self, cx: &mut Context<Self>) {
        let Some(frame) = self.selected_track_frame_mut() else {
            return;
        };
        if !frame.local_subscription || frame.track.local_path.is_none() {
            return;
        }
        let expanded = frame.toggle_inspector_panel(InspectorPanelKind::MusicBrainz);
        if !expanded {
            cx.notify();
            return;
        }
        match frame.musicbrainz_lookup {
            LazyPanel::Loaded(_) => {
                cx.notify();
                return;
            }
            LazyPanel::Loading => return,
            LazyPanel::Empty(_) | LazyPanel::Hidden => {
                frame.musicbrainz_lookup = LazyPanel::Loading;
            }
        }

        let entity_id = frame.entity_id;
        let track = frame.track.clone();
        cx.notify();

        present_command(
            &self.command_runner,
            LookupMusicBrainzTrack::new(track),
            CommandContext::next(),
            cx,
            move |this, result, _cx| {
                if let Some(frame) = this.selected_track_frame_mut() {
                    if frame.entity_id == entity_id {
                        frame.musicbrainz_selected = 0;
                        frame.musicbrainz_lookup = LazyPanel::Loaded(result);
                    }
                }
            },
            move |this, error, _cx| {
                if let Some(frame) = this.selected_track_frame_mut() {
                    if frame.entity_id == entity_id {
                        frame.musicbrainz_lookup =
                            LazyPanel::Empty(LibraryViewModel::deferred_panel_error_message(error));
                    }
                }
            },
        );
    }

    /// Opens the track page `entry` of `track_id` with its Inspect Sources
    /// sheet (ADR 0083 task 006, decision D4; ADR 0089 Decision 8). The
    /// pending request marks the track when it mounts. The event opens the
    /// sheet after the page mounts, because the top app owns the window.
    pub(crate) fn open_inspect_sources(
        &mut self,
        track_id: i64,
        entry: FrameNavigationEntry,
        cx: &mut Context<Self>,
    ) {
        // A track that is not mounted yet takes the request when it mounts,
        // and the mount emits the sheet event. A mounted track gets the event
        // here, so the sheet opens once in either case.
        let mounted = matches!(
            &self.detail,
            LibraryDetail::Track(frame) if frame.entity_id == track_id
        );
        if !mounted {
            self.pending_inspect_sources = Some(track_id);
        }
        self.open_page(entry, NavigationPathStart::CurrentPath, cx);
        if mounted {
            cx.emit(super::LibraryAppEvent::OpenInspectSourcesSheet);
        }
    }

    /// Opens the Inspect Sources sheet of the open track page (ADR 0089
    /// Decision 8). The sheet shows the Sources state of the page, so the
    /// state of the page and the open sheet stay equal.
    pub(crate) fn open_inspect_sources_sheet(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(frame) = self.selected_track_frame_mut() else {
            return;
        };
        frame
            .inspector_state
            .expand_panel(InspectorPanelKind::Sources);
        cx.notify();
        InspectSourcesSheet {
            source: cx.entity(),
            body: render_library_inspect_sources_sheet_body,
            on_close: Self::collapse_inspect_sources,
        }
        .open(window, cx);
    }

    /// Closes the Inspect Sources sheet state of the open track page. The
    /// sheet calls it when it closes (ADR 0089 Decision 8).
    pub(crate) fn collapse_inspect_sources(&mut self, cx: &mut Context<Self>) {
        if let Some(frame) = self.selected_track_frame_mut() {
            frame
                .inspector_state
                .collapse_panel(InspectorPanelKind::Sources);
        }
        cx.notify();
    }

    /// The track frame whose Inspect Sources sheet is open. `None` when no
    /// track page shows the sheet.
    pub(crate) fn inspect_sources_frame(&self) -> Option<&InspectorFrame> {
        match &self.detail {
            LibraryDetail::Track(frame) if frame.inspector_display(None).inspect_sources_open() => {
                Some(frame)
            }
            LibraryDetail::None
            | LibraryDetail::Track(_)
            | LibraryDetail::Album(_)
            | LibraryDetail::Playlist(_)
            | LibraryDetail::SinglesGroup(_)
            | LibraryDetail::WindowGroup(_) => None,
        }
    }

    /// The pointer of a track drag moves on or off the sidebar row of
    /// `destination` (ADR 0083 task 008, decision D1). On the row of a
    /// spring-loaded destination, this starts one timer of
    /// `Motion::SpringLoadDelay`. Off the row, the timer stops.
    pub(crate) fn spring_load_pointer(
        &mut self,
        destination: MusicDestination,
        on_row: bool,
        cx: &mut Context<Self>,
    ) {
        if !on_row {
            self.spring_load.leave();
            return;
        }
        let active = self.vm.active_destination();
        let Some(generation) = self.spring_load.enter(destination, active, Instant::now()) else {
            return;
        };
        present_after(Motion::SpringLoadDelay.duration(), cx, move |this, cx| {
            this.fire_spring_load(generation, cx);
        });
    }

    /// The spring-load timer of `generation` ends. When the drag continues
    /// and the pointer stayed on the row, the list column opens the
    /// destination. The navigation is the same as a click on the sidebar
    /// row.
    fn fire_spring_load(&mut self, generation: SpringLoadGeneration, cx: &mut Context<Self>) {
        if !cx.has_active_drag() {
            self.spring_load.end_drag();
            return;
        }
        let delay = Motion::SpringLoadDelay.duration();
        if let Some(destination) = self.spring_load.fire(generation, Instant::now(), delay) {
            let (entry, start) = spring_load_navigation(destination);
            self.open_page(entry, start, cx);
        }
    }

    /// A track drag ends with a drop (ADR 0083 task 008). The spring-load
    /// timer stops.
    pub(crate) fn end_track_drag(&mut self) {
        self.spring_load.end_drag();
    }

    /// The downloads and the removals in progress (ADR 0084 Decision 10).
    /// The Index release page reads the phase of its track rows from it
    /// (ADR 0083 task 006, decision D8).
    pub(crate) fn busy_items(&self) -> &crate::view_models::library::BusyItems {
        self.vm.busy_items()
    }

    /// "MusicBrainz lookup" of the track page menu: opens the Inspect Sources
    /// state and starts the lookup, or shows the panel when it is already open.
    pub(crate) fn show_musicbrainz_lookup(&mut self, cx: &mut Context<Self>) {
        let Some(frame) = self.selected_track_frame_mut() else {
            return;
        };
        frame
            .inspector_state
            .expand_panel(InspectorPanelKind::Sources);
        if frame
            .inspector_state
            .is_panel_expanded(InspectorPanelKind::MusicBrainz)
        {
            cx.notify();
            return;
        }
        self.toggle_musicbrainz_lookup(cx);
    }

    pub(crate) fn select_musicbrainz_candidate(&mut self, idx: usize, cx: &mut Context<Self>) {
        let Some(frame) = self.selected_track_frame_mut() else {
            return;
        };
        if let LazyPanel::Loaded(result) = &frame.musicbrainz_lookup {
            if idx < result.lookup.candidates.len() {
                frame.musicbrainz_selected = idx;
                cx.notify();
            }
        }
    }

    pub(crate) fn toggle_track_description(&mut self, cx: &mut Context<Self>) {
        let Some(frame) = self.selected_track_frame_mut() else {
            return;
        };
        let line_count =
            description_line_count(frame.source_context.as_ref().and_then(|context| {
                LibraryViewModel::display_description_text(context.track.description.as_deref())
            }));
        frame.inspector_state.project_description(line_count);
        frame.toggle_description();
        let track_id = frame.entity_id;
        let state = frame.inspector_state.description_state;
        self.vm.set_track_description_state(track_id, state);
        // Review of 2026-10-21: this click can grow the page. The pending
        // restored offset must not jump the page once it grows.
        self.clear_pending_detail_scroll();
        cx.notify();
    }

    pub(crate) fn toggle_album_description(&mut self, feed_id: i64, cx: &mut Context<Self>) {
        let Some(description) = self.detail_album_description(feed_id) else {
            return;
        };
        self.vm
            .toggle_album_description(feed_id, Some(description.as_str()));
        // Review of 2026-10-21: this click can grow the page. The pending
        // restored offset must not jump the page once it grows.
        self.clear_pending_detail_scroll();
        cx.notify();
    }

    fn detail_album_description(&self, feed_id: i64) -> Option<String> {
        let LibraryDetail::Album(album) = &self.detail else {
            return None;
        };
        (album.feed_id == Some(feed_id))
            .then(|| {
                LibraryViewModel::display_description_text(album.description.as_deref())
                    .map(str::to_owned)
            })
            .flatten()
    }

    pub(crate) fn musicbrainz_feed(&mut self, album: AlbumNode, cx: &mut Context<Self>) {
        let downloadable: Vec<TrackRow> = album
            .tracks
            .into_iter()
            .filter(|t| t.local_path.is_some())
            .collect();
        if !self
            .vm
            .begin_musicbrainz_album_lookup(downloadable.iter().map(|track| track.id))
        {
            cx.notify();
            return;
        }
        cx.notify();

        let feed_id = album.feed_id.unwrap_or(0);
        let feed_title = Some(album.name.clone());
        let request = StartFeedLookup::new(feed_id, feed_title, downloadable);
        let Some(saga) = self.musicbrainz_feed_saga.as_ref() else {
            self.vm.fail_musicbrainz_album_lookup_with_fallback(
                crate::application::capability::ExecutionUnavailable::RUNTIME.to_string(),
            );
            cx.notify();
            return;
        };
        if !saga.try_start(request) {
            self.vm
                .fail_musicbrainz_album_lookup_with_fallback("lookup actor unavailable");
            cx.notify();
        }
    }

    fn apply_musicbrainz_feed_saga_state(
        &mut self,
        state: MusicBrainzFeedSagaState,
        _cx: &mut Context<Self>,
    ) {
        match state {
            MusicBrainzFeedSagaState::Idle
            | MusicBrainzFeedSagaState::AlbumSearchInFlight { .. } => {}
            MusicBrainzFeedSagaState::AlbumSearchFailed { error, .. } => {
                self.vm.fail_musicbrainz_album_lookup_with_fallback(error);
            }
            MusicBrainzFeedSagaState::AlbumSearchEmpty { .. } => {
                self.vm.fallback_empty_musicbrainz_album_lookup();
            }
            MusicBrainzFeedSagaState::PerTrackInFlight {
                track_id,
                progress,
                total,
                ..
            } => {
                self.vm
                    .begin_musicbrainz_album_track_stage(track_id, progress, total);
            }
            MusicBrainzFeedSagaState::TrackDone {
                track_id,
                progress,
                total,
                edit_count,
                lookup,
                ..
            } => {
                self.vm
                    .begin_musicbrainz_album_track_stage(track_id, progress, total);
                self.stage_musicbrainz_lookup_for_track(track_id, lookup);
                self.vm.finish_musicbrainz_album_track_stage(
                    track_id,
                    MbTrackStatus::Done(edit_count),
                );
            }
            MusicBrainzFeedSagaState::TrackSkipped {
                track_id,
                progress,
                total,
                ..
            } => {
                self.vm
                    .begin_musicbrainz_album_track_stage(track_id, progress, total);
                self.vm
                    .finish_musicbrainz_album_track_stage(track_id, MbTrackStatus::Skipped);
            }
            MusicBrainzFeedSagaState::Completed {
                total_edits,
                processed,
                ..
            } => {
                self.vm
                    .finish_musicbrainz_album_lookup(total_edits, processed);
            }
        }
    }
}

fn album_has_feed_identity_actions(facts: &LocalIdentityFacts) -> bool {
    let identity = EntityIdentityLinks::from_source_facts(
        None,
        facts.source_links.clone(),
        facts.source_ids.clone(),
    );
    identity.website_url.is_some() && identity.nostr_npub.is_some()
}

// ---------------------------------------------------------------------------
// ADR 0084 Decision 10: the shared selection model, the checkbox, the
// clicks, the keys and the action bar of the Releases and Tracks lists.
// ---------------------------------------------------------------------------

impl LibraryApp {
    /// Every release key the Releases list currently shows, flattened
    /// from the selection rows of [`Self::releases_rows`], for
    /// [`ListSelection::retain_visible`]. The selection rows come in their
    /// render sequence (Required correction item 1, 2026-10-18): a release
    /// row, or a singles group row, and, for an open group, one row for
    /// each of its singles, after the group row.
    fn releases_visible_members(&self) -> Vec<ReleaseKey> {
        self.releases_rows()
            .selection_rows()
            .iter()
            .flat_map(|row| row.members.iter().cloned())
            .collect()
    }

    /// The Tracks or Inbox list's visible rows, in their
    /// render sequence. Each row has one member: its own track key
    /// (Required correction item 2: "The Tracks lists keep one member
    /// for each row").
    fn content_list_visible_rows(&self) -> Vec<SelectionRow<TrackKey, TrackKey>> {
        self.content_list_visible_track_keys()
            .into_iter()
            .map(SelectionRow::of_one)
            .collect()
    }

    /// The Tracks or Inbox list's visible track keys, in their
    /// render sequence.
    fn content_list_visible_track_keys(&self) -> Vec<TrackKey> {
        self.vm
            .content_list_page()
            .visible_rows()
            .into_iter()
            .filter_map(|row| self.vm.content_list_track_row_key(&row.id))
            .collect()
    }

    /// A click on a Releases destination-list row. Ctrl+click and
    /// Shift+click start selection mode and act on the row, as before;
    /// neither opens a page. In `Selecting` mode, a plain click also
    /// toggles the row and opens no page (ADR 0084 Decision 10
    /// Amendment, 2026-10-19). In `Normal` mode, a plain click opens the
    /// row's page. An Artist or a Publisher row has no selection model,
    /// so it always opens its page.
    pub(crate) fn click_release_row(
        &mut self,
        target: &DestinationListRowTarget,
        modifiers: Modifiers,
        cx: &mut Context<Self>,
    ) {
        let DestinationListRowTarget::Release(key) = target else {
            self.open_destination_row(target, cx);
            return;
        };
        let row = SelectionRow::new(ReleasesRowId::Release(key.clone()), vec![key.clone()]);
        if primary_modifier_held(modifiers) {
            self.releases_selection_mode = SelectionMode::Selecting;
            self.releases_selection.toggle(&row);
            cx.notify();
            return;
        }
        if modifiers.shift {
            self.releases_selection_mode = SelectionMode::Selecting;
            let rows = self.releases_rows();
            self.releases_selection
                .extend_to(&row, rows.selection_rows());
            cx.notify();
            return;
        }
        if self.releases_selection_mode.is_selecting() {
            self.releases_selection.toggle(&row);
            cx.notify();
            return;
        }
        self.releases_selection.click(row.id);
        self.open_destination_row(target, cx);
    }

    /// A click on a release row's own checkbox (ADR 0084 Decision 10):
    /// a Releases row, or a single row of an open group (Required
    /// correction item 4). It never opens a page.
    pub(crate) fn toggle_release_row_selection(&mut self, key: ReleaseKey, cx: &mut Context<Self>) {
        let row = SelectionRow::new(ReleasesRowId::Release(key.clone()), vec![key]);
        self.releases_selection_mode = SelectionMode::Selecting;
        self.releases_selection.toggle(&row);
        cx.notify();
    }

    /// A click on a singles group row's body (ADR 0084 Decisions 6 and
    /// 10). Ctrl+click starts selection mode and adds or removes every
    /// single of the group, the same as its checkbox; Shift+click starts
    /// selection mode and extends the selection to the group row, in the
    /// Releases list's visible sequence. Neither opens a page. In
    /// `Selecting` mode, a plain click on the row body also toggles the
    /// group (Amendment, 2026-10-19). In `Normal` mode, a plain click
    /// opens the group page.
    pub(crate) fn click_singles_group_row(
        &mut self,
        key: &SinglesGroupKey,
        single_keys: &[ReleaseKey],
        modifiers: Modifiers,
        cx: &mut Context<Self>,
    ) {
        let row = SelectionRow::new(ReleasesRowId::Group(key.clone()), single_keys.to_vec());
        if primary_modifier_held(modifiers) {
            self.releases_selection_mode = SelectionMode::Selecting;
            self.releases_selection.toggle(&row);
            cx.notify();
            return;
        }
        if modifiers.shift {
            self.releases_selection_mode = SelectionMode::Selecting;
            let rows = self.releases_rows();
            self.releases_selection
                .extend_to(&row, rows.selection_rows());
            cx.notify();
            return;
        }
        if self.releases_selection_mode.is_selecting() {
            self.releases_selection.toggle(&row);
            cx.notify();
            return;
        }
        self.releases_selection.click(row.id);
        self.open_singles_group(key, cx);
    }

    /// A click on a singles group row's checkbox (ADR 0084 Decision 6):
    /// it adds or removes every single of the group (Required
    /// correction item 3).
    pub(crate) fn toggle_release_group_selection(
        &mut self,
        key: &SinglesGroupKey,
        keys: &[ReleaseKey],
        cx: &mut Context<Self>,
    ) {
        let row = SelectionRow::new(ReleasesRowId::Group(key.clone()), keys.to_vec());
        self.releases_selection_mode = SelectionMode::Selecting;
        self.releases_selection.toggle(&row);
        cx.notify();
    }

    /// A click on a window group row or tile of the Releases list (ADR
    /// 0086 Decision 3, ADR 0089 packet 005 Part C). It follows the rule
    /// of [`Self::click_singles_group_row`]: Ctrl+click and a click in
    /// selection mode toggle each member, and Shift+click extends the
    /// selection. A plain click in `Normal` mode opens the group page.
    pub(crate) fn click_releases_window_group(
        &mut self,
        row_id: &str,
        members: &[ReleaseKey],
        page_key: &WindowGroupKey,
        modifiers: Modifiers,
        cx: &mut Context<Self>,
    ) {
        let row = SelectionRow::new(ReleasesRowId::Window(row_id.to_owned()), members.to_vec());
        if primary_modifier_held(modifiers) {
            self.releases_selection_mode = SelectionMode::Selecting;
            self.releases_selection.toggle(&row);
            cx.notify();
            return;
        }
        if modifiers.shift {
            self.releases_selection_mode = SelectionMode::Selecting;
            let rows = self.releases_rows();
            self.releases_selection
                .extend_to(&row, rows.selection_rows());
            cx.notify();
            return;
        }
        if self.releases_selection_mode.is_selecting() {
            self.releases_selection.toggle(&row);
            cx.notify();
            return;
        }
        self.releases_selection.click(row.id);
        self.open_window_group(page_key, cx);
    }

    /// A click on the checkbox of a window group row or tile of the
    /// Releases list (ADR 0086 Decision 3). It adds or removes each member
    /// of the group, and never opens the page.
    pub(crate) fn toggle_releases_window_group(
        &mut self,
        row_id: &str,
        members: &[ReleaseKey],
        cx: &mut Context<Self>,
    ) {
        let row = SelectionRow::new(ReleasesRowId::Window(row_id.to_owned()), members.to_vec());
        self.releases_selection_mode = SelectionMode::Selecting;
        self.releases_selection.toggle(&row);
        cx.notify();
    }

    /// A click on a Tracks or Inbox row. Ctrl+click and
    /// Shift+click start selection mode and act on the row, as before;
    /// neither opens a page. In `Selecting` mode, a plain click also
    /// toggles the row and opens no page (Amendment, 2026-10-19). In
    /// `Normal` mode, a plain click opens the row's page. A row with no
    /// track key, for example an Index row, has no selection model, so
    /// it always opens its page.
    pub(crate) fn click_content_list_row(
        &mut self,
        row_id: &str,
        modifiers: Modifiers,
        cx: &mut Context<Self>,
    ) {
        let Some(key) = self.vm.content_list_track_row_key(row_id) else {
            self.open_content_list_row(row_id, cx);
            return;
        };
        let row = SelectionRow::of_one(key.clone());
        if primary_modifier_held(modifiers) {
            self.track_selection_mode = SelectionMode::Selecting;
            self.track_selection.toggle(&row);
            cx.notify();
            return;
        }
        if modifiers.shift {
            self.track_selection_mode = SelectionMode::Selecting;
            let visible = self.content_list_visible_rows();
            self.track_selection.extend_to(&row, &visible);
            cx.notify();
            return;
        }
        if self.track_selection_mode.is_selecting() {
            self.track_selection.toggle(&row);
            cx.notify();
            return;
        }
        self.track_selection.click(key);
        self.open_content_list_row(row_id, cx);
    }

    /// A click on a Tracks or Inbox row's own checkbox (ADR
    /// 0084 Decision 10). It never opens a page.
    pub(crate) fn toggle_content_list_row_selection(
        &mut self,
        row_id: &str,
        cx: &mut Context<Self>,
    ) {
        if let Some(key) = self.vm.content_list_track_row_key(row_id) {
            self.track_selection_mode = SelectionMode::Selecting;
            self.track_selection.toggle(&SelectionRow::of_one(key));
            cx.notify();
        }
    }

    /// Up, Down, Page Up, Page Down, Home and End (ADR 0084 Decision
    /// 10). They act only while the Music list column carries the
    /// keyboard focus, and keep the newly focused row in view. Required
    /// correction item 6: the Releases list gives the group row and
    /// each single row of an open group its own focus stop, the same as
    /// a release row.
    /// Asks the Tiles grid of the Releases list to show the item `element`
    /// at the next layout (ADR 0089 packet 005 Part D). The item carries the
    /// reveal probe, and the probe scrolls the grid by the least distance.
    pub(crate) fn reveal_releases_tile(&mut self, element: Option<&str>) {
        self.releases_tile_target = element.map(str::to_owned);
        if self.releases_tile_target.is_some() {
            self.releases_tile_reveal.request();
        }
    }

    /// Marks a row when the list has the keyboard focus and no row shows it
    /// (ADR 0089 Decision 9, musicindex ADR 0026 R-FOCUS-1). The list runs this
    /// in the draw that sees the focus, so the row shows in that same draw. A
    /// row that Back marked keeps its place.
    fn mark_row_for_keyboard_entry(&mut self) {
        let has_focused_row = match self.vm.active_destination() {
            MusicDestination::Releases => self.releases_selection.focused().is_some(),
            MusicDestination::Inbox => self.track_selection.focused().is_some(),
            _ => return,
        };
        let Some(step) = keyboard_entry_step(
            self.list_row_focus_shows,
            has_focused_row,
            self.marked_entry.is_some(),
        ) else {
            return;
        };
        match self.vm.active_destination() {
            MusicDestination::Releases => {
                let rows = self.releases_rows();
                let visible = rows.selection_rows();
                if let Some(focused) = self.releases_selection.move_focus(step, visible) {
                    if let Some(index) = visible.iter().position(|row| row.id == focused) {
                        self.releases_list_state
                            .scroll_to_reveal_item(index + rows.leading());
                    }
                }
            }
            MusicDestination::Inbox => {
                let visible = self.content_list_visible_rows();
                if self.track_selection.move_focus(step, &visible).is_some() {
                    self.inbox_scroll.scroll_to_item(0);
                }
            }
            _ => {}
        }
    }

    pub(crate) fn move_selection_focus(
        &mut self,
        step: FocusStep,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        if !self.list_column_has_focus(window, cx) {
            return;
        }
        match self.vm.active_destination() {
            MusicDestination::Releases => {
                let rows = self.releases_rows();
                let visible = rows.selection_rows();
                // A status row at the list start is the first item of the
                // list (ADR 0084 packet 016 decision D8, task 037).
                let leading = rows.leading();
                if let Some(focused) = self.releases_selection.move_focus(step, visible) {
                    if matches!(self.vm.list_view(ListPageKey::Releases), ListView::Tiles) {
                        self.reveal_releases_tile(rows.item_element_of_selection(&focused));
                    } else if let Some(index) = visible.iter().position(|row| row.id == focused) {
                        self.releases_list_state
                            .scroll_to_reveal_item(index + leading);
                    }
                }
                cx.notify();
            }
            MusicDestination::Inbox => {
                let visible = self.content_list_visible_rows();
                if let Some(focused) = self.track_selection.move_focus(step, &visible) {
                    if let Some(index) = visible.iter().position(|row| row.id == focused) {
                        self.inbox_scroll.scroll_to_item(index);
                    }
                }
                cx.notify();
            }
            MusicDestination::Search
            | MusicDestination::Artists
            | MusicDestination::Playlists
            | MusicDestination::PublicPlaylists
            | MusicDestination::Live => {}
        }
    }

    /// Right (`open` true) or Left on the focused Releases row (ADR 0084
    /// Decisions 6 and 10): opens or closes a singles group, or moves the
    /// focus from a single to its group row.
    pub(crate) fn disclose_focused_group(
        &mut self,
        open: bool,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        if !self.list_column_has_focus(window, cx)
            || self.vm.active_destination() != MusicDestination::Releases
        {
            return;
        }
        let rows = self.releases_rows();
        match releases_disclosure_step(self.releases_selection.focused(), open, rows.items()) {
            ReleasesDisclosureStep::Open(key) => {
                self.expanded_singles_groups.insert(key);
            }
            ReleasesDisclosureStep::Close(key) => {
                self.expanded_singles_groups.remove(&key);
            }
            ReleasesDisclosureStep::FocusGroup(key) => {
                self.releases_selection.focus_row(ReleasesRowId::Group(key));
            }
            ReleasesDisclosureStep::None => return,
        }
        cx.notify();
    }

    /// Space (ADR 0084 Decision 10): starts selection mode and toggles
    /// the row with the keyboard focus (Amendment, 2026-10-19). Required
    /// correction item 3: on a group row, this adds or removes every
    /// single of the group.
    pub(crate) fn toggle_focused_selection(&mut self, window: &Window, cx: &mut Context<Self>) {
        if !self.list_column_has_focus(window, cx) {
            return;
        }
        match self.vm.active_destination() {
            MusicDestination::Releases => {
                self.releases_selection_mode = SelectionMode::Selecting;
                let rows = self.releases_rows();
                self.releases_selection
                    .toggle_focused(rows.selection_rows());
                cx.notify();
            }
            MusicDestination::Inbox => {
                self.track_selection_mode = SelectionMode::Selecting;
                let visible = self.content_list_visible_rows();
                self.track_selection.toggle_focused(&visible);
                cx.notify();
            }
            MusicDestination::Search
            | MusicDestination::Artists
            | MusicDestination::Playlists
            | MusicDestination::PublicPlaylists
            | MusicDestination::Live => {}
        }
    }

    /// Ctrl+A (ADR 0084 Decision 10): starts selection mode and selects
    /// every member of every visible row (Amendment, 2026-10-19).
    pub(crate) fn select_all_visible(&mut self, window: &Window, cx: &mut Context<Self>) {
        if self.list_column_has_focus(window, cx) {
            self.select_each_list_row(cx);
        }
    }

    /// "Select all" of the list column bar, and the end of Ctrl+A.
    fn select_each_list_row(&mut self, cx: &mut Context<Self>) {
        match self.vm.active_destination() {
            MusicDestination::Releases => {
                self.releases_selection_mode = SelectionMode::Selecting;
                let rows = self.releases_rows();
                self.releases_selection.select_all(rows.selection_rows());
                cx.notify();
            }
            MusicDestination::Inbox => {
                self.track_selection_mode = SelectionMode::Selecting;
                let visible = self.content_list_visible_rows();
                self.track_selection.select_all(&visible);
                cx.notify();
            }
            MusicDestination::Search
            | MusicDestination::Artists
            | MusicDestination::Playlists
            | MusicDestination::PublicPlaylists
            | MusicDestination::Live => {}
        }
    }

    /// Enter (ADR 0084 Decision 10): opens the page of the row with the
    /// keyboard focus. Gives `false` with no focused row, so the caller
    /// keeps the present Enter behavior for every other case. Required
    /// correction item 1: a focused group row opens the group page, not
    /// the release page of one of its singles.
    pub(crate) fn open_focused_selection_row(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.list_column_has_focus(window, cx) {
            return false;
        }
        match self.vm.active_destination() {
            MusicDestination::Releases => match self.releases_selection.focused().cloned() {
                Some(ReleasesRowId::Release(key)) => {
                    self.open_page(
                        FrameNavigationEntry::Release(key),
                        NavigationPathStart::ListRow,
                        cx,
                    );
                    true
                }
                Some(ReleasesRowId::Group(key)) => {
                    self.open_singles_group(&key, cx);
                    true
                }
                Some(ReleasesRowId::Window(row_id)) => {
                    let page_key =
                        self.releases_rows()
                            .items()
                            .iter()
                            .find_map(|item| match item {
                                ReleasesListItemVm::WindowGroup(tile)
                                    if tile.open_row_id == row_id =>
                                {
                                    Some(tile.page_key.clone())
                                }
                                _ => None,
                            });
                    let Some(page_key) = page_key else {
                        return false;
                    };
                    self.open_window_group(&page_key, cx);
                    true
                }
                None => false,
            },
            MusicDestination::Inbox => {
                let Some(focused) = self.track_selection.focused().cloned() else {
                    return false;
                };
                self.open_page(
                    FrameNavigationEntry::Track(focused),
                    NavigationPathStart::ListRow,
                    cx,
                );
                true
            }
            MusicDestination::Search
            | MusicDestination::Artists
            | MusicDestination::Playlists
            | MusicDestination::PublicPlaylists
            | MusicDestination::Live => false,
        }
    }

    /// Escape (ADR 0084 Decision 10 Amendment, 2026-10-19): ends
    /// selection mode and clears the selection, when the list is in
    /// `Selecting` mode. Gives `false` outside that mode, so the caller
    /// keeps the present Escape behavior.
    pub(crate) fn clear_focused_selection(
        &mut self,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.list_column_has_focus(window, cx) {
            return false;
        }
        match self.vm.active_destination() {
            MusicDestination::Releases if self.releases_selection_mode.is_selecting() => {
                self.releases_selection_mode = SelectionMode::Normal;
                self.releases_selection.clear();
                cx.notify();
                true
            }
            MusicDestination::Inbox if self.track_selection_mode.is_selecting() => {
                self.track_selection_mode = SelectionMode::Normal;
                self.track_selection.clear();
                cx.notify();
                true
            }
            _ => false,
        }
    }

    /// Sets whether the current Music location shows a page (ADR 0089
    /// Decision 1). The top app calls it at each render.
    pub(crate) fn set_music_page_open(&mut self, open: bool) {
        self.music_page_open = open;
    }

    /// Returns whether the current Music location shows a page (ADR 0089
    /// Decision 1).
    pub(crate) const fn music_page_open(&self) -> bool {
        self.music_page_open
    }

    /// Returns whether the Library tree reloads (ADR 0089 Decision 5).
    pub(crate) fn is_library_loading(&self) -> bool {
        self.vm.is_library_loading()
    }

    /// The focus handle of the Music list column (ADR 0089 Invariants).
    pub(crate) const fn list_column_focus(&self) -> &gpui::FocusHandle {
        &self.list_column_focus
    }

    /// Sets the sidebar row of the current screen (ADR 0089 Decision 1).
    /// The top app calls it at each render, and the sidebar renders again
    /// only when the row changes.
    pub(crate) fn set_sidebar_selected(
        &mut self,
        selected: Option<crate::view_models::sidebar::SidebarDestination>,
        cx: &mut Context<Self>,
    ) {
        if self.sidebar_selected != selected {
            self.sidebar_selected = selected;
            cx.notify();
        }
    }

    /// Moves the keyboard focus to the sidebar row of `row` (ADR 0089
    /// Decision 1 and Invariants).
    pub(crate) fn focus_sidebar_row(
        &self,
        row: crate::view_models::sidebar::SidebarDestination,
        window: &mut Window,
        cx: &mut App,
    ) {
        if let Some((_, handle)) = self.sidebar_focus.iter().find(|(key, _)| *key == row) {
            window.focus(handle, cx);
        }
    }

    /// The focus handle of a sidebar row (ADR 0089 Decision 1).
    fn sidebar_row_focus(
        &self,
        row: crate::view_models::sidebar::SidebarDestination,
    ) -> Option<&gpui::FocusHandle> {
        self.sidebar_focus
            .iter()
            .find(|(key, _)| *key == row)
            .map(|(_, handle)| handle)
    }

    /// Returns whether the Library detail holds the page of a Library-backed
    /// Music entry (ADR 0089 Decision 5). A Library release or track is
    /// checked here, so an Index release or track is checked by its caller.
    pub(crate) fn detail_shows_page(&self, entry: &FrameNavigationEntry) -> bool {
        match (entry, &self.detail) {
            (FrameNavigationEntry::PlaylistDetail(id), LibraryDetail::Playlist(detail)) => {
                detail.playlist.id == *id
            }
            (FrameNavigationEntry::Release(_), LibraryDetail::Album(_))
            | (FrameNavigationEntry::Track(_), LibraryDetail::Track(_)) => true,
            (FrameNavigationEntry::SinglesGroup(key), LibraryDetail::SinglesGroup(shown)) => {
                key == shown
            }
            (FrameNavigationEntry::WindowGroup(key), LibraryDetail::WindowGroup(shown)) => {
                key == shown
            }
            _ => false,
        }
    }

    /// Returns whether the list column carries the keyboard focus and
    /// shows. A hidden list column ignores the keys (ADR 0089 Invariants).
    pub(crate) fn list_column_has_focus(&self, window: &Window, cx: &App) -> bool {
        !self.music_page_open && self.list_column_focus.contains_focused(window, cx)
    }

    /// Returns whether the detail column carries the keyboard focus and
    /// shows. A hidden detail column ignores the keys (ADR 0089 Invariants).
    pub(crate) fn detail_column_has_focus(&self, window: &Window, cx: &App) -> bool {
        self.music_page_open && self.detail_column_focus.contains_focused(window, cx)
    }

    /// The Music list column's header action (ADR 0084 Decision 10
    /// Amendment, 2026-10-19): "Select" in `Normal`, "Done" in
    /// `Selecting`, for the Tracks and Inbox list. Releases and
    /// Artists show their Select in the title row (ADR 0089 Decision 6).
    /// Absent for every other destination and for an empty list.
    pub(crate) fn header_action_for_list_column(&self) -> Option<FrameHeaderActionDisplay> {
        if self.vm.active_destination() != MusicDestination::Inbox {
            return None;
        }
        let list_has_rows = !self.content_list_visible_rows().is_empty();
        list_column_header_action(
            MusicDestination::Inbox,
            self.track_selection_mode,
            list_has_rows,
        )
    }

    /// The title row of the Releases list (ADR 0089 Decision 6). Its count
    /// is the number of Library releases, and the view model shows it only
    /// in the scope "In Library". Its Select action is the Releases
    /// selection's "Select" or "Done".
    pub(crate) fn releases_page_head(&self) -> ListPageHeadVm {
        let rows = self.releases_rows();
        let count = Some(self.vm.library_release_count());
        let select = list_column_header_action(
            MusicDestination::Releases,
            self.releases_selection_mode,
            !rows.selection_rows().is_empty(),
        );
        self.vm.list_page_head(ListPageKey::Releases, count, select)
    }

    /// The title row of the Artists list (ADR 0089 Decision 6). Its count
    /// shows only in the scope "In Library".
    pub(crate) fn artists_page_head(&self, library_count: usize) -> ListPageHeadVm {
        self.vm
            .list_page_head(ListPageKey::Artists, Some(library_count), None)
    }

    /// The header action's click (ADR 0084 Decision 10 Amendment,
    /// 2026-10-19): "Select" starts selection mode, with no change to
    /// the selection. "Done" ends it and clears the selection.
    pub(crate) fn toggle_list_column_selection_mode(&mut self, cx: &mut Context<Self>) {
        match self.vm.active_destination() {
            MusicDestination::Releases => {
                self.releases_selection_mode = match self.releases_selection_mode {
                    SelectionMode::Normal => SelectionMode::Selecting,
                    SelectionMode::Selecting => {
                        self.releases_selection.clear();
                        SelectionMode::Normal
                    }
                };
            }
            MusicDestination::Inbox => {
                self.track_selection_mode = match self.track_selection_mode {
                    SelectionMode::Normal => SelectionMode::Selecting,
                    SelectionMode::Selecting => {
                        self.track_selection.clear();
                        SelectionMode::Normal
                    }
                };
            }
            MusicDestination::Search
            | MusicDestination::Artists
            | MusicDestination::Playlists
            | MusicDestination::PublicPlaylists
            | MusicDestination::Live => {}
        }
        cx.notify();
    }

    /// The Releases list's action bar (ADR 0084 Decision 10), or `None`
    /// outside `Selecting` mode (Amendment, 2026-10-19). The bar reads the
    /// row states from the cached rows (ADR 0084 task 037).
    pub(crate) fn releases_selection_action_bar(&self) -> Option<SelectionActionBarVm> {
        self.vm.releases_selection_action_bar(
            &self.releases_selection,
            self.releases_selection_mode,
            &self.releases_rows(),
        )
    }

    /// The Tracks or Inbox list's action bar (ADR 0084
    /// Decision 10), or `None` outside `Selecting` mode (Amendment,
    /// 2026-10-19).
    pub(crate) fn content_list_selection_action_bar(&self) -> Option<SelectionActionBarVm> {
        self.vm.content_list_selection_action_bar(
            &self.track_selection,
            self.track_selection_mode,
            self.content_list_visible_rows().len(),
        )
    }

    /// Runs a Download, Remove or Clear action of the Releases list's
    /// action bar (ADR 0084 Decision 10). "Add to playlist" opens its
    /// own popover, through [`Self::add_selected_releases_to_playlist`].
    pub(crate) fn run_releases_selection_action(
        &mut self,
        kind: SelectionActionKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match kind {
            // ADR 0084 packet 016 Required Change 15: the Library jobs of
            // the Library releases, and one Index release job with no feed
            // for each MusicIndex release. Each job reads its feed in its
            // own queue slot.
            SelectionActionKind::Download => {
                let plan = self.vm.releases_download_plan(
                    &self.releases_selection,
                    &self.release_name_page,
                    self.vm.index_feed_singles(),
                );
                self.download_release_plan(plan, cx);
            }
            SelectionActionKind::Remove => self.request_releases_selection_removal(window, cx),
            SelectionActionKind::SelectAll => self.select_each_list_row(cx),
            SelectionActionKind::Clear => {
                self.releases_selection.clear();
                cx.notify();
            }
            // Only the playlist page bar of packet 025 gives the playlist
            // actions.
            SelectionActionKind::AddToPlaylist
            | SelectionActionKind::RemoveFromPlaylist
            | SelectionActionKind::MoveToTop
            | SelectionActionKind::MoveToBottom => {}
        }
    }

    /// "Add n releases to playlist" of the action bar: adds every
    /// selected release, whatever its download state.
    pub(crate) fn add_selected_releases_to_playlist(
        &mut self,
        playlist_id: i64,
        cx: &mut Context<Self>,
    ) {
        let feed_ids = self.vm.releases_selected_feed_ids(&self.releases_selection);
        self.add_albums_to_playlist(&feed_ids, playlist_id, cx);
    }

    /// The "New Playlist" create flow of the Releases action bar's "Add
    /// to playlist" popover.
    pub(crate) fn create_playlist_and_add_selected_releases(
        &mut self,
        name: &str,
        cx: &mut Context<Self>,
    ) {
        let feed_ids = self.vm.releases_selected_feed_ids(&self.releases_selection);
        self.create_playlist_and_add_albums(name, feed_ids, cx);
    }

    /// Runs a Remove or Clear action of the Tracks or Inbox
    /// list's action bar (ADR 0084 Decision 10). "Download" never shows
    /// there, since every selectable row is already in the Library.
    /// "Add to playlist" opens its own popover, through
    /// [`Self::add_selected_tracks_to_playlist`].
    pub(crate) fn run_content_list_selection_action(
        &mut self,
        kind: SelectionActionKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match kind {
            // ADR 0084 Decision 10 Design: "Download" never shows here,
            // since every selectable row is already in the Library.
            // "Add to playlist" opens its own popover and dispatches no
            // bar action.
            // Only the playlist page bar of packet 025 gives the playlist
            // actions.
            SelectionActionKind::Download
            | SelectionActionKind::AddToPlaylist
            | SelectionActionKind::RemoveFromPlaylist
            | SelectionActionKind::MoveToTop
            | SelectionActionKind::MoveToBottom => {}
            SelectionActionKind::Remove => self.request_content_list_selection_removal(window, cx),
            SelectionActionKind::SelectAll => self.select_each_list_row(cx),
            SelectionActionKind::Clear => {
                self.track_selection.clear();
                cx.notify();
            }
        }
    }

    /// "Add n tracks to playlist" of the action bar.
    pub(crate) fn add_selected_tracks_to_playlist(
        &mut self,
        playlist_id: i64,
        cx: &mut Context<Self>,
    ) {
        let track_ids = self.content_list_selected_track_ids();
        self.add_tracks_to_playlist(track_ids, playlist_id, cx);
    }

    /// The "New Playlist" create flow of the Tracks action bar's "Add to
    /// playlist" popover.
    pub(crate) fn create_playlist_and_add_selected_tracks(
        &mut self,
        name: &str,
        cx: &mut Context<Self>,
    ) {
        let track_ids = self.content_list_selected_track_ids();
        self.create_playlist_and_add_tracks(name, track_ids, cx);
    }

    fn content_list_selected_track_ids(&self) -> Vec<i64> {
        self.vm
            .content_list_selected_tracks(&self.track_selection)
            .into_iter()
            .map(|(track_id, _title)| track_id)
            .collect()
    }

    /// Adds each track of `track_ids` to a playlist, through one append
    /// (ADR 0084 Decision 10). An empty `track_ids` adds nothing.
    pub(crate) fn add_tracks_to_playlist(
        &mut self,
        track_ids: Vec<i64>,
        playlist_id: i64,
        cx: &mut Context<Self>,
    ) {
        if track_ids.is_empty() {
            return;
        }
        if let Some(intent) = self.vm.begin_playlist_append(playlist_id, track_ids) {
            self.spawn_subscribe_then_append(intent, cx);
        }
    }

    /// Creates a playlist, then adds each track of `track_ids` to it
    /// (ADR 0084 Decision 10).
    pub(crate) fn create_playlist_and_add_tracks(
        &mut self,
        name: &str,
        track_ids: Vec<i64>,
        cx: &mut Context<Self>,
    ) {
        if track_ids.is_empty() {
            return;
        }
        let command = CreatePlaylist::new(Arc::clone(&self.conn), name.to_string());
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, result, cx| {
                let playlist_id = result.playlist_id();
                this.reload_playlists_then(cx, move |this, cx| {
                    this.add_tracks_to_playlist(track_ids.clone(), playlist_id, cx);
                });
            },
            |this, err, _cx| this.vm.fail_playlist_create(err),
        );
    }

    /// "Remove n releases…" of the Releases action bar (ADR 0084
    /// Decision 10): opens one confirmation that lists each selected
    /// release with one or more downloaded tracks.
    fn request_releases_selection_removal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let feed_ids = self
            .vm
            .releases_removable_feed_ids(&self.releases_selection);
        let items = feed_ids
            .into_iter()
            .map(|(feed_id, title)| (LibraryRemovalIntent::FeedId(feed_id), title))
            .collect();
        self.request_selection_removal(items, "release", window, cx);
    }

    /// "Remove n tracks…" of the Tracks action bar (ADR 0084 Decision
    /// 10): opens one confirmation that lists each selected track.
    fn request_content_list_selection_removal(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let tracks = self.vm.content_list_removable_tracks(&self.track_selection);
        let items = tracks
            .into_iter()
            .map(|(track_id, title)| (LibraryRemovalIntent::TrackId(track_id), title))
            .collect();
        self.request_selection_removal(items, "track", window, cx);
    }

    /// Opens one confirmation that lists each item of `items` (ADR 0084
    /// Decision 10). The list column bars and the detail column bar
    /// (packet 018) use it. `item_noun` is "track" or "release".
    pub(crate) fn request_selection_removal(
        &mut self,
        items: Vec<(LibraryRemovalIntent, String)>,
        item_noun: &'static str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let items = self.plan_selection_removal(items.into_iter());
        if items.is_empty() {
            return;
        }
        self.selection_removal.open(item_noun, items);
        self.open_selection_removal_dialog(window, cx);
    }

    /// Plans one removal for each `(intent, title)` pair (ADR 0084
    /// Decision 10 Design: "The removal confirmation" reuses the present
    /// `LibraryRemovalPlan` for each item). A planning error drops that
    /// one item and reports it; the rest continue.
    fn plan_selection_removal(
        &mut self,
        items: impl Iterator<Item = (LibraryRemovalIntent, String)>,
    ) -> Vec<SelectionRemovalPendingItem> {
        let conn = self.conn.lock().expect("lock db");
        let mut pending = Vec::new();
        for (intent, title) in items {
            let plan = match self
                .application_services
                .query_service()
                .library_removal_plan(&conn, &intent)
            {
                Ok(plan) => plan,
                Err(err) => {
                    self.vm.set_error_status(err);
                    continue;
                }
            };
            let referring_playlists =
                removal_referring_playlist_names(&conn, plan.target()).unwrap_or_default();
            pending.push(SelectionRemovalPendingItem::new(
                plan,
                title,
                referring_playlists,
            ));
        }
        pending
    }

    /// Opens the pending whole-selection removal confirmation (ADR 0084
    /// Decision 10).
    fn open_selection_removal_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(display) = self.selection_removal.pending_display() else {
            return;
        };
        open_selection_removal_confirmation_dialog(
            window,
            cx,
            display,
            |this, cx| {
                this.selection_removal.cancel();
                cx.notify();
            },
            |this, cx| {
                this.execute_selection_removal(cx);
            },
        );
        cx.notify();
    }

    /// "Remove" of the selection removal confirmation. When one removal
    /// fails, the others continue (ADR 0084 Decision 10 Design: "The
    /// removal confirmation").
    fn execute_selection_removal(&mut self, cx: &mut Context<Self>) {
        let (items, item_noun) = self.selection_removal.take_pending_items();
        let targets: Vec<LibraryRemovalTarget> = items.iter().map(|(target, _)| *target).collect();
        self.selection_removal_report = Some(SelectionRemovalReport::start(items, item_noun));
        for target in targets {
            self.execute_library_removal_target(target, cx);
        }
    }

    /// Records one removal result in the open selection report, when one
    /// waits for `target`. Returns `true` when the report owns the result,
    /// so the caller sets no status of its own (ADR 0084 Decision 10).
    fn record_selection_removal_result(
        &mut self,
        target: LibraryRemovalTarget,
        error: Option<String>,
    ) -> bool {
        let Some(report) = self.selection_removal_report.as_mut() else {
            return false;
        };
        if !report.waits_for(target) {
            return false;
        }
        match error {
            Some(error) => report.record_failure(target, error),
            None => report.record_done(target),
        }
        if report.finished() {
            if let Some(text) = report.failure_text() {
                self.vm.set_error_status(text);
            }
            self.selection_removal_report = None;
        }
        true
    }
}

// ---------------------------------------------------------------------------
// Render
// ---------------------------------------------------------------------------

impl LibraryApp {
    /// The sidebar rows for the current selection (ADR 0089 Decision 1). The
    /// narrow tab bar reads the same rows.
    pub(crate) fn sidebar_vm(&self) -> crate::view_models::sidebar::SidebarVm {
        self.vm
            .sidebar(self.sidebar_selected, self.tag_update_snapshot.as_ref())
    }

    /// The feed-update action as the menu items of the narrow tab bar (ADR
    /// 0089 Decision 1). The item keeps the label, the disabled state and the
    /// status text of the sidebar feed area. The item runs the same command.
    pub(crate) fn feed_update_menu_items(&self, cx: &mut Context<Self>) -> Vec<ContextMenuItem> {
        let FeedUpdateDisplay {
            state_label,
            status_message,
            action,
        } = self.vm.feed_update_display();
        let description: Vec<String> = state_label
            .map(str::to_owned)
            .into_iter()
            .chain(status_message)
            .collect();
        let kind = action.kind;
        let library = cx.entity();
        let item = ContextMenuItem::new(ContextMenuItemDisplay {
            id: action.button_id.into(),
            label: action.label.clone().into(),
            a11y_label: action.label.into(),
            destructive: false,
            disabled: action.disabled,
        })
        .on_select(move |_, cx| {
            library.update(cx, |library, cx| match kind {
                FeedUpdateActionKind::ApplyUpdates => library.apply_all_feed_updates(cx),
                FeedUpdateActionKind::CheckAllFeeds => library.check_all_feeds(cx),
            });
        });
        let item = if description.is_empty() {
            item
        } else {
            item.description(description.join(". "))
        };
        vec![item]
    }

    /// Renders the sidebar column (ADR 0089 Decision 1): the rows of
    /// `SidebarVm`, a line, then the feed area. `TopApp::render` places it
    /// beside the toolbar and the content on each mount. The feed-update
    /// area and "Check All Feeds" stay at its bottom.
    pub(crate) fn render_sidebar_column(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let sidebar = self
            .vm
            .sidebar(self.sidebar_selected, self.tag_update_snapshot.as_ref());
        let mut items: Vec<AnyElement> = Vec::new();
        for entry in sidebar.entries() {
            match entry {
                crate::view_models::sidebar::SidebarEntryVm::Row(row) => {
                    let focus = self
                        .sidebar_row_focus(row.destination)
                        .cloned()
                        .unwrap_or_else(|| cx.focus_handle());
                    items.push(render_sidebar_row(row, &focus, window, cx));
                }
                crate::view_models::sidebar::SidebarEntryVm::Separator => {
                    items.push(render_sidebar_separator());
                }
            }
        }

        let tag_update_button = crate::view_models::tag_update::button(
            self.tag_update_snapshot.as_ref(),
        )
        .map(|button| {
            render_tag_update_button(
                button,
                cx.listener(|this, _, window, cx| {
                    this.open_tag_update_popup(window, cx);
                }),
            )
        });
        let tag_update_report = crate::view_models::tag_update::report(
            self.tag_update_snapshot.as_ref(),
            self.tag_update_request_failed,
        )
        .map(|report| render_tag_update_report(report, cx));
        let status = self.vm.status_snapshot();
        let status_color = if status.is_error {
            StatusRole::Danger.color(cx)
        } else {
            color::text_muted()
        };
        let status_text = status.text;
        // ADR 0083 Decision 1: the sidebar surface, distinct from the
        // content canvas behind `trailing_pane`.
        let destinations_scroll = {
            let container = div()
                .id("music-destinations-scroll")
                .flex_1()
                .min_h_0()
                .p(spacing::SM)
                .child(page_scroll_content(cx).gap(spacing::XXS).children(items));
            music_column_scrollbar(container, "music-destinations-scroll-scrollbar")
        };
        div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .overflow_hidden()
            .bg(color::bg_sidebar())
            .child(destinations_scroll)
            .child({
                let FeedUpdateDisplay {
                    state_label: feed_state_label,
                    status_message: feed_status,
                    action,
                } = self.vm.feed_update_display();
                let FeedUpdateActionDisplay {
                    kind,
                    button_id,
                    label,
                    disabled,
                } = action;
                div()
                    .id("feed-update-section")
                    .flex()
                    .flex_col()
                    .w_full()
                    .min_w_0()
                    // ADR 0084 Decision 1: the sidebar is narrow, so the
                    // status text and the buttons stack, and the buttons
                    // wrap. A row of both clipped the last button.
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_start()
                            .gap(spacing::XS)
                            .px(spacing::MD)
                            .py(spacing::XS)
                            .border_t_1()
                            .border_color(color::border_subtle())
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(spacing::XXS)
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(status_color)
                                            .child(SharedString::from(status_text)),
                                    )
                                    .when_some(feed_state_label, |el, label| {
                                        el.child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(color::accent())
                                                .child(SharedString::from(label)),
                                        )
                                    }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .flex_wrap()
                                    .items_center()
                                    .gap(spacing::SM)
                                    // ADR 0076 Decision 8: the button is
                                    // absent when no file differs.
                                    .children(tag_update_button)
                                    .child(if kind == FeedUpdateActionKind::ApplyUpdates {
                                        UiButton::styled(button_id, ControlStyle::Primary)
                                            .label(label)
                                            .disabled(disabled)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.apply_all_feed_updates(cx);
                                            }))
                                    } else {
                                        UiButton::styled(button_id, ControlStyle::Secondary)
                                            .label(label)
                                            .disabled(disabled)
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.check_all_feeds(cx);
                                            }))
                                    }),
                            ),
                    )
                    .children(tag_update_report)
                    .when_some(feed_status, |el, msg| {
                        el.child(
                            div()
                                .id("feed-update-result")
                                .w_full()
                                .min_w_0()
                                .px(spacing::MD)
                                .py(spacing::XS)
                                .text_xs()
                                .text_color(color::text_muted())
                                .child(SharedString::from(msg)),
                        )
                    })
            })
            .into_any_element()
    }

    /// Renders the Music list column (ADR 0084 Decision 1). It always
    /// shows the content list. It no longer gives its place to the detail
    /// page: the detail column renders separately, in
    /// `render_detail_column`.
    pub(crate) fn render_list_column(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        // ADR 0084 Decision 2: the list column shows the items of the
        // current root destination. ADR 0084 Decision 3: Discover shows
        // its views, not the content list.
        // R-FOCUS-1: a row shows the focus only while its list has the keyboard
        // focus, so one place at a time shows a ring.
        // `is_focused` does not read the tree of the last frame, so it is true in
        // the first draw after a Tab.
        self.letter_cursor_shows =
            self.letter_bar_focus.is_focused(window) && crate::ui::focus_ring::focus_ring_shows(cx);
        self.list_row_focus_shows = (self.list_column_focus.is_focused(window)
            || self.list_column_focus.contains_focused(window, cx))
            && crate::ui::focus_ring::focus_ring_shows(cx);
        self.mark_row_for_keyboard_entry();
        let mut action_bar: Option<AnyElement> = None;
        let page_head: Option<AnyElement>;
        let content_list_pane = match self.vm.active_destination() {
            MusicDestination::Search => {
                let vm = SearchPageVm::new(&self.saved_searches, self.open_saved_search)
                    .with_marked(self.marked_entry.as_ref())
                    .with_message(self.saved_search_message.clone())
                    .with_new_counts(&self.saved_search_counts);
                let entity = cx.entity();
                page_head = Some(Self::render_titled_page_head(
                    MusicDestination::Search.label(),
                    Some(vm.new_action().clone()),
                    move |_window, cx| {
                        entity.update(cx, |_this, cx| {
                            cx.emit(super::LibraryAppEvent::SearchPage(
                                SearchPageAction::NewSavedSearch,
                            ));
                        });
                    },
                    cx,
                ));
                let rename = SearchPageListRename {
                    renaming: self.saved_search_renaming,
                    input: &self.saved_search_rename_input,
                };
                render_search_page_list(
                    &vm,
                    self.how_to_search,
                    &rename,
                    &self.search_page_scroll,
                    cx,
                )
            }
            MusicDestination::Inbox => {
                page_head = Some(Self::render_titled_page_head(
                    MusicDestination::Inbox.label(),
                    None,
                    |_window, _cx| {},
                    cx,
                ));
                let mut content_row_thumbs: BTreeMap<String, Option<Arc<Image>>> = BTreeMap::new();
                for (row_id, url) in self.vm.content_list_thumbnail_sources() {
                    let img = self.thumbnail_for_url(Some(&url), false, cx);
                    content_row_thumbs.insert(row_id, img);
                }
                // ADR 0084 Decision 10: the rows carry the Tracks or
                // Inbox selection's checkbox facts.
                let rows = self.vm.content_list_rows_with_selection(
                    &self.track_selection,
                    self.track_selection_mode,
                    self.list_row_focus_shows,
                );
                let rows = self
                    .vm
                    .content_list_rows_with_marked(rows, self.marked_entry.as_ref());
                // ADR 0083 task 007, decision D15: the right-click menu of
                // a row gives the focus back to the list column.
                let menu = ContentListMenuContext {
                    playlists: std::rc::Rc::new(playlist_options(self.vm.playlists())),
                    return_focus: self.list_column_focus.clone(),
                };
                // ADR 0086 packet 002: the list column shows no window
                // group, so each of its rows is one item.
                let items: Vec<_> = rows.into_iter().collect();
                // ADR 0089 Decision 3: the Tracks section comes first, then
                // Updates and Setup. A section with no item is absent, and
                // an Inbox with no item shows the empty state of the rows.
                let inbox = self.vm.inbox_page(self.tag_update_snapshot.as_ref());
                let shows_track_pane = inbox
                    .sections()
                    .first()
                    .is_none_or(|section| section.section == InboxSection::Tracks);
                let track_pane = shows_track_pane.then(|| {
                    render_library_content_list(
                        self.vm.content_list_page(),
                        items,
                        &content_row_thumbs,
                        &self.inbox_scroll,
                        (ContentListSelectionTarget::ListColumn, None),
                        Some(menu),
                        cx,
                    )
                });
                let sections = render_inbox_sections(&inbox, cx);
                let pane = match (track_pane, sections) {
                    (Some(track_pane), Some(sections)) => div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_h_0()
                        .min_w_0()
                        .overflow_hidden()
                        .child(track_pane)
                        .child(sections)
                        .into_any_element(),
                    (Some(track_pane), None) => track_pane,
                    (None, Some(sections)) => sections,
                    (None, None) => div().into_any_element(),
                };
                if let Some(bar) = self.content_list_selection_action_bar() {
                    action_bar = Some(self.render_selection_action_bar(
                        bar,
                        Self::run_content_list_selection_action,
                        Self::add_selected_tracks_to_playlist,
                        Self::create_playlist_and_add_selected_tracks,
                        cx,
                    ));
                }
                pane
            }
            MusicDestination::PublicPlaylists => {
                page_head = Some(self.render_public_playlists_page_head(cx));
                self.render_public_playlists_pane(window, cx)
            }
            MusicDestination::Live => {
                page_head = Some(self.render_live_page_head(cx));
                self.render_live_pane(window, cx)
            }
            MusicDestination::Releases => {
                // ADR 0084 task 037: the rows come from the cache, and the
                // virtual list renders only the items in view. Each item
                // gets the Releases selection's checkbox facts, its image
                // and its menu in `render_releases_item`.
                page_head = Some(self.render_releases_page_head(cx));
                let rows = self.releases_rows();
                let pane = match self.vm.list_view(ListPageKey::Releases) {
                    ListView::Rows => {
                        self.releases_list_state.sync(rows.item_ids(), window);
                        // ADR 0084 task 031: the release row menu. The
                        // right-click menu gives the focus back to the list
                        // column.
                        let menu = Rc::new(self.release_row_menu_context(
                            cx.entity().downgrade(),
                            self.list_column_focus.clone(),
                        ));
                        let item_count = rows.items().len();
                        render_library_releases_list(
                            &self.releases_list_state,
                            item_count,
                            cx.processor(move |this, index: usize, _window, cx| {
                                this.render_releases_item(&rows, index, &menu, cx)
                            }),
                            cx,
                        )
                    }
                    ListView::Tiles => self.render_releases_tiles(&rows, cx),
                };
                if let Some(bar) = self.releases_selection_action_bar() {
                    action_bar = Some(self.render_selection_action_bar(
                        bar,
                        Self::run_releases_selection_action,
                        Self::add_selected_releases_to_playlist,
                        Self::create_playlist_and_add_selected_releases,
                        cx,
                    ));
                }
                pane
            }
            MusicDestination::Artists => {
                let artists_scope = self.vm.list_scope(ListPageKey::Artists);
                let rows = self.vm.artists_list(artists_scope, &self.artist_name_page);
                let library_count = self
                    .vm
                    .artists_list(ListScope::InLibrary, &self.artist_name_page)
                    .len();
                page_head = Some(self.render_artists_page_head(library_count, cx));
                // ADR 0084 packet 016 decision D8: the loading row, the
                // error row or "Load more" of the scope "All".
                let status =
                    LibraryViewModel::artists_list_status(artists_scope, &self.artist_name_page);
                // ADR 0084 task 037, operator answer Q1 option B: the
                // Artists list is a virtual list with no cache of its
                // merge. Each item reads its art only when it renders.
                let items = Rc::new(ArtistsListItemVm::of(rows, status));
                let ids: Vec<String> = items
                    .iter()
                    .map(|item| item.element_id().to_owned())
                    .collect();
                self.artists_list_state.sync(&ids, window);
                let item_count = items.len();
                render_library_destination_rows(
                    &self.artists_list_state,
                    item_count,
                    cx.processor(move |this, index: usize, _window, cx| {
                        this.render_artists_item(&items, index, cx)
                    }),
                    cx,
                )
            }
            MusicDestination::Playlists => {
                let sidebar = self
                    .vm
                    .playlist_sidebar()
                    .with_marked(self.marked_entry.as_ref());
                let entity = cx.entity();
                let reverse_entity = entity.clone();
                let on_reverse: ListReverseHandler = Rc::new(move |reversed, _window, cx| {
                    reverse_entity.update(cx, |this, cx| {
                        this.set_list_reversed(ListPageKey::MyPlaylists, reversed, cx);
                    });
                });
                let display = ListPageHeadVm::titled(
                    MusicDestination::Playlists.label(),
                    Some(SearchPageActionDisplay {
                        id: sidebar.add_button_id,
                        label: sidebar.add_label,
                        a11y_label: sidebar.add_label.to_owned(),
                        available: true,
                        style: SearchPageActionStyle::Action,
                    }),
                )
                .with_reverse_of(
                    ListPageKey::MyPlaylists,
                    self.vm.list_page_state(ListPageKey::MyPlaylists),
                );
                page_head = Some(Self::render_page_head_with_reverse(
                    display,
                    Some(on_reverse),
                    move |_window, cx| {
                        entity.update(cx, LibraryApp::toggle_creating_playlist);
                    },
                    cx,
                ));
                let mut arts = RowArts::new();
                for row in &sidebar.rows {
                    let image = collection_art_image(self, &row.art, ThumbnailSize::Sm, cx);
                    arts.insert(row.element_id.clone(), image);
                }
                let row_menus = self.playlist_row_menus(&sidebar, cx);
                render_library_playlists_list(
                    sidebar,
                    row_menus,
                    &self.new_playlist_input,
                    &arts,
                    &self.playlists_scroll,
                    cx,
                )
            }
        };

        let (has_row_focus, row_count) = match self.vm.active_destination() {
            MusicDestination::Releases => (true, self.releases_rows().selection_rows().len()),
            MusicDestination::Inbox => (true, self.content_list_visible_rows().len()),
            _ => (false, 0),
        };
        let list_tab_stop = list_is_tab_stop(has_row_focus, row_count);
        div()
            .size_full()
            .bg(color::bg_canvas())
            .text_color(color::text_primary())
            .text_sm()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .overflow_hidden()
            .track_focus(&self.list_column_focus.clone().tab_stop(list_tab_stop))
            .children(page_head)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .child(content_list_pane),
            )
            .children(action_bar)
            .into_any_element()
    }

    /// Renders the title row of a page with a title and at most one action
    /// (ADR 0089 Decision 6).
    fn render_titled_page_head(
        title: &str,
        action: Option<SearchPageActionDisplay>,
        on_action: impl Fn(&mut Window, &mut App) + 'static,
        cx: &App,
    ) -> AnyElement {
        Self::render_page_head_with_reverse(
            ListPageHeadVm::titled(title, action),
            None,
            on_action,
            cx,
        )
    }

    /// Renders a title row with at most one action and, when `reverse` is
    /// given, the reverse icon (musicindex ADR 0043 R-LIST-7).
    fn render_page_head_with_reverse(
        display: ListPageHeadVm,
        reverse: Option<ListReverseHandler>,
        on_action: impl Fn(&mut Window, &mut App) + 'static,
        cx: &App,
    ) -> AnyElement {
        list_page_head(
            display,
            ListPageHeadHandlers {
                actions: vec![Rc::new(on_action)],
                reverse,
                ..ListPageHeadHandlers::default()
            },
            cx,
        )
    }

    /// Puts the keyboard cursor on the letter bar of `display`, while the bar
    /// has the keyboard focus.
    pub(crate) fn with_letter_cursor(&self, mut display: ListPageHeadVm) -> ListPageHeadVm {
        if self.letter_cursor_shows {
            display.letters = display
                .letters
                .map(|letters| letters.with_cursor_shown(self.letter_cursor));
        }
        display
    }

    /// The keyboard side of the letter bar of the page `key` (walk of
    /// 2026-11-06). A move only changes the cursor. Enter or Space runs the
    /// letter, because a letter sends a name request.
    pub(crate) fn letter_bar_keys(
        &self,
        key: ListPageKey,
        cx: &mut Context<Self>,
    ) -> LetterBarKeys {
        let step_entity = cx.entity();
        let activate_entity = cx.entity();
        LetterBarKeys {
            focus: self.letter_bar_focus.clone(),
            return_focus: self.list_column_focus.clone(),
            on_step: Rc::new(move |step, _window, cx| {
                step_entity.update(cx, |this, cx| {
                    if let Some(letters) = this.vm.list_page_head(key, None, None).letters {
                        this.letter_cursor = Some(letters.moved(this.letter_cursor, step));
                        cx.notify();
                    }
                });
            }),
            on_activate: Rc::new(move |_window, cx| {
                activate_entity.update(cx, |this, cx| {
                    if let Some(letters) = this.vm.list_page_head(key, None, None).letters {
                        let letter = letters.cursor_key(this.letter_cursor);
                        this.letter_cursor = Some(letter);
                        this.set_list_letter(key, Some(letter), cx);
                    }
                });
            }),
        }
    }

    /// Renders the title row of the Releases list (ADR 0089 Decision 6).
    /// Its scope and its Select action call the view model through the
    /// handlers of the app.
    fn render_releases_page_head(&self, cx: &mut Context<Self>) -> AnyElement {
        let display = self.with_letter_cursor(self.releases_page_head());
        let letter_keys = self.letter_bar_keys(ListPageKey::Releases, cx);
        let entity = cx.entity();
        let scope_entity = entity.clone();
        // A mouse click on a control moves the focus to it (ADR 0069). The
        // arrow keys act on the list, so the focus goes back to the list. A
        // keyboard user stays on the control.
        let list_focus = self.list_column_focus.clone();
        let refocus_list = move |window: &mut Window, cx: &mut App| {
            if !crate::ui::focus_ring::focus_ring_shows(cx) {
                window.focus(&list_focus, cx);
            }
        };
        let scope_refocus = refocus_list.clone();
        let view_refocus = refocus_list.clone();
        let on_scope: ListScopeHandler = Rc::new(move |scope, window, cx| {
            let scope = *scope;
            scope_entity.update(cx, |this, cx| this.set_music_list_scope(scope, cx));
            scope_refocus(window, cx);
        });
        let sort_entity = entity.clone();
        let on_sort: ListSortHandler = Rc::new(move |sort, _window, cx| {
            let sort = *sort;
            sort_entity.update(cx, |this, cx| {
                this.set_list_sort(ListPageKey::Releases, sort, cx);
            });
        });
        let reverse_entity = entity.clone();
        let on_reverse: ListReverseHandler = Rc::new(move |reversed, _window, cx| {
            reverse_entity.update(cx, |this, cx| {
                this.set_list_reversed(ListPageKey::Releases, reversed, cx);
            });
        });
        let letter_entity = entity.clone();
        let on_letter: LetterHandler = Rc::new(move |letter, _window, cx| {
            let letter = *letter;
            letter_entity.update(cx, |this, cx| {
                this.set_list_letter(ListPageKey::Releases, Some(letter), cx);
            });
        });
        let view_entity = entity.clone();
        let on_view: ListViewHandler = Rc::new(move |view, window, cx| {
            let view = *view;
            view_entity.update(cx, |this, cx| {
                this.set_list_view(ListPageKey::Releases, view, cx);
            });
            view_refocus(window, cx);
        });
        let on_select: ListHeadButtonHandler = Rc::new(move |window, cx| {
            entity.update(cx, Self::toggle_list_column_selection_mode);
            refocus_list(window, cx);
        });
        list_page_head(
            display,
            ListPageHeadHandlers {
                scope: Some(on_scope),
                select: Some(on_select),
                sort: Some(on_sort),
                view: Some(on_view),
                reverse: Some(on_reverse),
                letter: Some(on_letter),
                letter_keys: Some(letter_keys),
                actions: Vec::new(),
            },
            cx,
        )
    }

    /// Renders the title row of the Artists list (ADR 0089 Decision 6).
    fn render_artists_page_head(&self, library_count: usize, cx: &mut Context<Self>) -> AnyElement {
        let display = self.with_letter_cursor(self.artists_page_head(library_count));
        let letter_keys = self.letter_bar_keys(ListPageKey::Artists, cx);
        let entity = cx.entity();
        let scope_entity = entity.clone();
        let on_scope: ListScopeHandler = Rc::new(move |scope, _window, cx| {
            let scope = *scope;
            scope_entity.update(cx, |this, cx| this.set_music_list_scope(scope, cx));
        });
        let sort_entity = entity.clone();
        let on_sort: ListSortHandler = Rc::new(move |sort, _window, cx| {
            let sort = *sort;
            sort_entity.update(cx, |this, cx| {
                this.set_list_sort(ListPageKey::Artists, sort, cx);
            });
        });
        let reverse_entity = entity.clone();
        let on_reverse: ListReverseHandler = Rc::new(move |reversed, _window, cx| {
            reverse_entity.update(cx, |this, cx| {
                this.set_list_reversed(ListPageKey::Artists, reversed, cx);
            });
        });
        let on_letter: LetterHandler = Rc::new(move |letter, _window, cx| {
            let letter = *letter;
            entity.update(cx, |this, cx| {
                this.set_list_letter(ListPageKey::Artists, Some(letter), cx);
            });
        });
        list_page_head(
            display,
            ListPageHeadHandlers {
                scope: Some(on_scope),
                select: None,
                sort: Some(on_sort),
                view: None,
                reverse: Some(on_reverse),
                letter: Some(on_letter),
                letter_keys: Some(letter_keys),
                actions: Vec::new(),
            },
            cx,
        )
    }

    /// Renders the item `index` of the cached Releases `rows` (ADR 0084
    /// task 037, Design step 9). The virtual list calls this only for an
    /// item in view or in the overdraw area. The item gets the selection
    /// facts of the present selection (Design step 10). Its image loads
    /// only now, so a row out of view starts no image request (decision
    /// D4).
    fn render_releases_item(
        &mut self,
        rows: &ReleasesListRows,
        index: usize,
        menu: &ReleaseRowMenuContext,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(item) = rows.items().get(index) else {
            return div().into_any_element();
        };
        let (item, thumbnail, art) = self.releases_item_parts(item, cx);
        // ADR 0084 packet 016 decision D8: the loading row, the error row
        // or "Load more" of the scope "Everything".
        let actions = Self::releases_status_actions();
        let release_column = if self.releases_sort() == ListSort::Newest {
            LeadingDisclosure::None
        } else {
            LeadingDisclosure::Empty
        };
        render_releases_list_item(item, thumbnail, art, actions, menu, release_column, cx)
    }

    /// The item of the Releases list with its selection facts and its
    /// mark, its cover and its collection art (ADR 0084 task 037, Design
    /// step 10; ADR 0089 packet 005 Part C). A cover loads only when the
    /// item renders, so an item out of view starts no image request.
    fn releases_item_parts(
        &mut self,
        item: &ReleasesListItemVm,
        cx: &mut Context<Self>,
    ) -> (
        ReleasesListItemVm,
        Option<Arc<gpui::Image>>,
        Option<CollectionArtImage>,
    ) {
        let item = LibraryViewModel::releases_item_with_selection(
            item,
            &self.releases_selection,
            self.releases_selection_mode,
            self.list_row_focus_shows,
        )
        .with_marked(self.marked_entry.as_ref());
        let thumbnail = item.thumbnail_href().and_then(|href| {
            let animated = self.vm.hovered_thumb_url() == Some(href);
            self.thumbnail_for_url(Some(href), animated, cx)
        });
        // ADR 0084 Decision 12: the group row shows its art in the logo
        // frame.
        let art = match &item {
            ReleasesListItemVm::Group(group) => Some(collection_art_image(
                self,
                &group.art,
                ThumbnailSize::Sm,
                cx,
            )),
            ReleasesListItemVm::WindowGroup(tile) => {
                Some(collection_art_image(self, &tile.art, ThumbnailSize::Sm, cx))
            }
            ReleasesListItemVm::LeadingStatus(_)
            | ReleasesListItemVm::Release(_)
            | ReleasesListItemVm::Single(_)
            | ReleasesListItemVm::TrailingStatus(_) => None,
        };
        (item, thumbnail, art)
    }

    /// The actions of a status row of the Releases list (ADR 0084 packet
    /// 016 decision D8).
    fn releases_status_actions() -> IndexListActions {
        IndexListActions {
            retry: Self::retry_release_names,
            load_more: Some(Self::load_more_release_names),
        }
    }

    /// Renders the Tiles view of the Releases list (ADR 0089 packet 005
    /// Part C). Each item has the tile of its kind. The status rows keep
    /// their row at the start or at the end of the list.
    fn render_releases_tiles(
        &mut self,
        rows: &ReleasesListRows,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mut leading = Vec::new();
        let mut tiles = Vec::new();
        let mut trailing = Vec::new();
        for (index, item) in rows.items().iter().enumerate() {
            let is_leading = matches!(item, ReleasesListItemVm::LeadingStatus(_));
            let is_status = is_leading || matches!(item, ReleasesListItemVm::TrailingStatus(_));
            let (item, thumbnail, art) = self.releases_item_parts(item, cx);
            let mut element = render_releases_list_tile(
                item,
                thumbnail,
                art,
                Self::releases_status_actions(),
                cx,
            );
            // The reveal target carries the probe, so the grid scrolls to it.
            if self.releases_tile_target.as_deref() == rows.item_element_at(index) {
                element = div()
                    .relative()
                    .child(element)
                    .child(self.releases_tile_reveal.probe())
                    .into_any_element();
            }
            if is_leading {
                leading.push(element);
            } else if is_status {
                trailing.push(element);
            } else {
                tiles.push(element);
            }
        }
        // A status row shows while the list loads (leading), or when more
        // rows wait for a "Load more" or a retry (trailing).
        let has_more = !trailing.is_empty();
        let is_loading = !leading.is_empty();
        render_releases_tile_grid(
            &self.releases_tiles_scroll,
            leading,
            tiles,
            trailing,
            has_more,
            is_loading,
            cx,
        )
    }

    /// Renders the item `index` of the Artists `items` (ADR 0084 task 037,
    /// operator answer Q1 option B). The item reads its art only when it
    /// renders.
    fn render_artists_item(
        &mut self,
        items: &[ArtistsListItemVm],
        index: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(item) = items.get(index) else {
            return div().into_any_element();
        };
        let item = item.clone().with_marked(self.marked_entry.as_ref());
        let art = match &item {
            ArtistsListItemVm::Row(row) => row
                .art
                .as_ref()
                .map(|art| collection_art_image(self, art, ThumbnailSize::Sm, cx)),
            ArtistsListItemVm::Status(_) => None,
        };
        let actions = IndexListActions {
            retry: Self::retry_artist_names,
            load_more: Some(Self::load_more_artist_names),
        };
        render_artists_list_item(item, art, actions, cx)
    }

    /// Builds the action bar composite of a selection (ADR 0084 Decision
    /// 10). `run` dispatches a Download, Remove or Clear action; `add`
    /// and `create` wire the "Add to playlist" popover.
    fn render_selection_action_bar(
        &mut self,
        bar: SelectionActionBarVm,
        run: fn(&mut Self, SelectionActionKind, &mut Window, &mut Context<Self>),
        add: fn(&mut Self, i64, &mut Context<Self>),
        create: fn(&mut Self, &str, &mut Context<Self>),
        cx: &mut Context<Self>,
    ) -> AnyElement {
        use crate::ui::composites::{SelectionActionBar, SelectionActionBarPlaylists};

        let playlists = SelectionActionBarPlaylists {
            playlists: playlist_options(self.vm.playlists()),
            new_playlist_a11y_label: SharedString::from("Create a new playlist"),
            back_a11y_label: SharedString::from("Back to playlist choices"),
            create_a11y_label: SharedString::from("Create playlist and add the selection"),
        };
        // `cx.listener` always gives a handler that reads an event by
        // reference. Download, Remove and Clear act on no event, so
        // each wraps the listener with a throwaway `&()`.
        let download = cx.listener(move |this, (): &(), window, cx| {
            run(this, SelectionActionKind::Download, window, cx);
        });
        let remove = cx.listener(move |this, (): &(), window, cx| {
            run(this, SelectionActionKind::Remove, window, cx);
        });
        let clear = cx.listener(move |this, (): &(), window, cx| {
            run(this, SelectionActionKind::Clear, window, cx);
        });
        let select_all = cx.listener(move |this, (): &(), window, cx| {
            run(this, SelectionActionKind::SelectAll, window, cx);
        });
        SelectionActionBar::new(bar, playlists)
            .on_download(move |window, cx| download(&(), window, cx))
            .on_remove(move |window, cx| remove(&(), window, cx))
            .on_clear(move |window, cx| clear(&(), window, cx))
            .on_select_all(move |window, cx| select_all(&(), window, cx))
            .on_add_to_playlist(cx.listener(move |this, playlist_id: &i64, _window, cx| {
                add(this, *playlist_id, cx);
            }))
            .on_create_playlist(cx.listener(move |this, name: &String, _window, cx| {
                create(this, name.as_str(), cx);
            }))
            .into_any_element()
    }

    /// Renders the Music detail column (ADR 0084 Decision 1). It shows the
    /// Library page when a page is open, or else the empty state that
    /// `render_library_detail` gives from `LibraryChromeDisplay`.
    pub(crate) fn render_detail_column(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let detail_pane = self.render_library_detail_pane(cx);

        div()
            .size_full()
            .bg(color::bg_canvas())
            .text_color(color::text_primary())
            .text_sm()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .overflow_hidden()
            .child(detail_pane)
            .into_any_element()
    }

    /// Renders the Library release, track, artist and playlist pages, and
    /// the empty detail state (ADR 0084 Decision 1).
    fn render_library_detail_pane(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let chrome = LibraryViewModel::chrome_display()
            .with_playback_availability(self.playback_availability);

        let mut urls: Vec<String> = Vec::new();
        if let LibraryDetail::Album(album) = &self.detail {
            urls.extend(album_thumbnail_urls(album));
        }
        // ADR 0084 task 015, operator decision of 2026-10-17: a group page
        // row is the Library track row of a release page. It reads its art
        // by URL, the same as a release page track row. A `MusicIndex` row
        // reads its release image by URL (packet 026 decision D13).
        let page_rows = match &self.detail {
            LibraryDetail::SinglesGroup(key) => Some(self.singles_group_page(key).rows),
            // ADR 0086 Decision 4: the window group page has the same rows.
            LibraryDetail::WindowGroup(key) => Some(self.vm.window_group_page(key).rows),
            _ => None,
        };
        if let Some(page_rows) = page_rows {
            urls.extend(page_rows.iter().filter_map(|row| {
                match row {
                    SinglesGroupPageEntryVm::Library(row) => row
                        .track
                        .track_image_href
                        .clone()
                        .or_else(|| row.track.album_image_href.clone()),
                    SinglesGroupPageEntryVm::Index(row) => row.image_url.clone(),
                }
            }));
        }
        // Walk fix F5 of 2026-10-20: a playlist page row reads its art by
        // URL, the same as a release page track row. Only the drawn rows
        // give a URL: the ready rows of the paged path, or each row of
        // the eager path.
        if let LibraryDetail::Playlist(detail) = &self.detail {
            let paged = self
                .playlist_actor
                .as_ref()
                .filter(|state| state.playlist_id == detail.playlist.id);
            urls.extend(match paged {
                Some(state) => {
                    crate::view_models::paged_playlist_detail::PagedPlaylistDetailVm::new(
                        &detail.playlist,
                        &state.snapshot,
                    )
                    .thumbnail_urls()
                }
                None => {
                    crate::view_models::playlist_detail::playlist_row_thumbnail_urls(&detail.tracks)
                }
            });
        }
        let hovered_url = self.vm.hovered_thumb_url().map(str::to_string);
        let mut album_thumbs: BTreeMap<String, Option<Arc<Image>>> = BTreeMap::new();
        for url in &urls {
            if !album_thumbs.contains_key(url.as_str()) {
                let animated = hovered_url.as_deref() == Some(url.as_str());
                let img = self.thumbnail_for_url(Some(url), animated, cx);
                album_thumbs.insert(url.clone(), img);
            }
        }

        let track_named_feeds = self.mounted_track_named_feeds();
        let album_cover_color = match &self.detail {
            LibraryDetail::Album(album) => {
                let url = album.image_href.clone();
                self.cover_color_for_url(url.as_deref(), cx)
            }
            _ => None,
        };
        // ADR 0084 packet 018: each row of a release page and of a singles
        // group page carries the detail selection facts.
        let row_slots = self.library_detail_row_slots();
        // ADR 0090 Decisions 1 and 2: a group page and a release page have
        // their list bar above their rows.
        let list_bar = match &self.detail {
            LibraryDetail::SinglesGroup(_)
            | LibraryDetail::WindowGroup(_)
            | LibraryDetail::Album(_) => self.library_detail_page_rows().and_then(|rows| {
                crate::view_models::detail_selection::ListBarVm::new(&rows, &self.detail_selection)
                    .map(|bar| self.with_group_list_controls(bar))
            }),
            _ => None,
        };
        let scroll = self.detail_scroll_handle();
        // ADR 0084 Decision 12: a singles group page and a playlist page
        // show their header art in the logo frame.
        let header_art = match &self.detail {
            LibraryDetail::SinglesGroup(key) => self.singles_group_page(key).art,
            LibraryDetail::WindowGroup(key) => self.vm.window_group_page(key).art,
            LibraryDetail::Playlist(detail) => Some(
                crate::view_models::library::PlaylistDetailVm::new(
                    &detail.playlist,
                    &detail.tracks,
                )
                .collection_art(),
            ),
            _ => None,
        }
        .map(|art| collection_art_image(self, &art, ThumbnailSize::XXl, cx));
        render_library_detail(
            &self.detail,
            self.vm.mb_status(),
            &self.vm,
            &album_thumbs,
            self.vm.playlists(),
            &chrome,
            self.rename_playlist_input.clone(),
            self.vm.renaming_playlist_id(),
            self.playlist_actor.as_ref(),
            self.playlist_rss_snapshot.as_ref(),
            &track_named_feeds,
            album_cover_color,
            header_art,
            list_bar,
            &self.list_bar_pin,
            row_slots,
            &scroll,
            &self.detail_column_focus,
            cx,
        )
    }

    /// Renders one Music column (ADR 0084 Decision 1). `LibraryColumnView`
    /// calls this through `Entity::update`, so each Library listener keeps
    /// using `cx.listener` on `Context<LibraryApp>` as it does today.
    pub(crate) fn render_column(
        &mut self,
        column: super::LibraryColumn,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        match column {
            super::LibraryColumn::Sidebar => self.render_sidebar_column(window, cx),
            super::LibraryColumn::List => self.render_list_column(window, cx),
            super::LibraryColumn::Detail => self.render_detail_column(window, cx),
        }
    }
}

// ADR 0075 keeps typed provider state in the mounted frame.
fn retain_track_source_context(
    frame: &mut InspectorFrame,
    entity_id: i64,
    context: crate::metadata::TrackContext,
) {
    if frame.entity_id == entity_id {
        frame.source_context = Some(context);
    }
}

// ---------------------------------------------------------------------------
// Rendering helpers
// ---------------------------------------------------------------------------

pub(crate) fn playlist_options(playlists: &[db::Playlist]) -> Vec<PlaylistOption> {
    playlist_option_displays(playlists)
        .into_iter()
        .map(|option| {
            PlaylistOption::new(PlaylistOptionDisplay {
                id: option.id,
                name: SharedString::from(option.name),
                a11y_label: SharedString::from(option.a11y_label),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::queries::library::{
        apply_local_track_metadata_defaults, fetch_library_track_context_with_local_fallback,
    };
    use crate::metadata::TrackContext;

    fn test_track(id: i64, feed_id: i64, is_in_library: bool) -> TrackRow {
        TrackRow {
            id,
            feed_id,
            item_guid: format!("track-{id}"),
            track_title: Some(format!("Track {id}")),
            is_in_library,
            local_path: is_in_library
                .then(|| LibraryRelativePath::for_test(&format!("music/track-{id}.mp3"))),
            ..TrackRow::default()
        }
    }

    fn test_album(feed_id: i64, tracks: Vec<TrackRow>) -> AlbumNode {
        AlbumNode {
            name: "Test Album".into(),
            feed_id: Some(feed_id),
            feed_guid: Some("feed-guid".into()),
            feed_url: Some("https://example.test/feed.xml".into()),
            language: None,
            description: None,
            image_href: Some("https://example.test/art.png".into()),
            named_feeds: Vec::new(),
            identity_facts: LocalIdentityFacts::default(),
            metadata_facts: Box::<crate::views::FeedMetadataFacts>::default(),
            stored_values: None,
            tracks,
        }
    }

    #[test]
    fn build_tree_loads_feed_language_for_unsubscribed_local_feed() -> anyhow::Result<()> {
        let conn = Connection::open_in_memory()?;
        db::init_schema(&conn)?;
        conn.execute(
            "INSERT INTO feeds (feed_url, title, language, is_subscribed)
             VALUES (?1, ?2, ?3, 0)",
            rusqlite::params!["https://example.test/feed.xml", "Example Feed", "fr"],
        )?;
        let feed_id = conn.last_insert_rowid();
        let track = TrackRow {
            id: 1,
            feed_id,
            item_guid: "track-1".into(),
            track_title: Some("Track 1".into()),
            artist_name: Some("Example Artist".into()),
            album_title: Some("Example Feed".into()),
            is_in_library: true,
            ..TrackRow::default()
        };

        let tree = crate::application::queries::library::build_tree(&[track], &conn)
            .expect("build the tree");
        let album = &tree.artists[0].albums[0];

        assert_eq!(album.feed_id, Some(feed_id));
        assert_eq!(album.language.as_deref(), Some("fr"));

        Ok(())
    }

    #[test]
    fn build_tree_loads_feed_metadata_facts() -> anyhow::Result<()> {
        let mut conn = Connection::open_in_memory()?;
        db::init_schema(&conn)?;
        conn.execute(
            "INSERT INTO feeds (feed_url, title, is_subscribed)
             VALUES (?1, ?2, 0)",
            rusqlite::params!["https://example.test/feed.xml", "Example Feed"],
        )?;
        let feed_id = conn.last_insert_rowid();
        db::replace_local_metadata_facts(
            &mut conn,
            db::LocalMetadataOwner::Feed(feed_id),
            "musicindex",
            &[db::LocalMetadataFactInput {
                fact_key: "publisher_text".into(),
                value: db::LocalMetadataValue::Text("Example Publisher".into()),
                extraction_path: None,
                observed_at: None,
                raw_json: None,
            }],
        )?;
        let track = TrackRow {
            id: 1,
            feed_id,
            item_guid: "track-1".into(),
            track_title: Some("Track 1".into()),
            artist_name: Some("Example Artist".into()),
            album_title: Some("Example Feed".into()),
            is_in_library: true,
            ..TrackRow::default()
        };

        let tree = crate::application::queries::library::build_tree(&[track], &conn)
            .expect("build the tree");
        let album = &tree.artists[0].albums[0];

        assert_eq!(
            album.metadata_facts.publisher_text.as_deref(),
            Some("Example Publisher")
        );

        Ok(())
    }

    #[test]
    fn library_track_context_falls_back_to_local_hydrated_context() -> anyhow::Result<()> {
        let mut conn = Connection::open_in_memory()?;
        db::init_schema(&conn)?;
        db::migrate_schema(&conn)?;
        conn.execute(
            "INSERT INTO feeds (id, feed_url, feed_guid, title)
             VALUES (2, ?1, ?2, ?3)",
            rusqlite::params!["https://example.test/feed.xml", "feed-guid", "Feed"],
        )?;
        conn.execute(
            "INSERT INTO tracks (id, feed_id, item_guid, track_title)
             VALUES (9, 2, ?1, ?2)",
            rusqlite::params!["track-guid", "Track"],
        )?;
        db::replace_local_metadata_facts(
            &mut conn,
            db::LocalMetadataOwner::Track(9),
            "musicindex",
            &[db::LocalMetadataFactInput {
                fact_key: "description".into(),
                value: db::LocalMetadataValue::Text("Local track description".into()),
                extraction_path: None,
                observed_at: None,
                raw_json: None,
            }],
        )?;
        let track = TrackRow {
            id: 9,
            feed_id: 2,
            feed_guid: Some("feed-guid".into()),
            item_guid: "track-guid".into(),
            track_title: Some("Track".into()),
            feed_title: Some("Feed".into()),
            ..TrackRow::default()
        };
        let conn = Arc::new(Mutex::new(conn));

        let context = fetch_library_track_context_with_local_fallback(
            &conn,
            &track,
            &"http://127.0.0.1:9".into(),
        )?;

        assert_eq!(
            context.track.description.as_deref(),
            Some("Local track description")
        );
        assert_eq!(context.observation_receipts.len(), 3);
        Ok(())
    }

    #[test]
    fn local_track_metadata_fills_missing_remote_context_fields() {
        let mut remote_context = TrackContext {
            rss_observation: None,
            observation_receipts: Vec::new(),
            provider_state: crate::provider_observation::ProviderTrackState::default(),
            confirmed_labels: Vec::new(),
            track: crate::api::Track {
                title: Some("Remote Track".into()),
                publisher_text: Some("Remote Publisher".into()),
                ..crate::api::Track::default()
            },
            feed: None,
        };
        let local_context = TrackContext {
            rss_observation: None,
            observation_receipts: Vec::new(),
            provider_state: crate::provider_observation::ProviderTrackState::default(),
            confirmed_labels: Vec::new(),
            track: crate::api::Track {
                publisher_text: Some("Local Publisher".into()),
                description: Some("Local track description".into()),
                pub_date: Some(1_700_000_000),
                explicit: Some(true),
                ..crate::api::Track::default()
            },
            feed: None,
        };

        apply_local_track_metadata_defaults(&mut remote_context, &local_context);

        assert_eq!(
            remote_context.track.publisher_text.as_deref(),
            Some("Remote Publisher"),
            "remote source text should not be overwritten"
        );
        assert_eq!(
            remote_context.track.description.as_deref(),
            Some("Local track description")
        );
        assert_eq!(remote_context.track.pub_date, Some(1_700_000_000));
        assert_eq!(remote_context.track.explicit, Some(true));
    }

    #[test]
    fn adr_0075_snapshot_mounted_frame_preserves_typed_state_and_legacy_fields() {
        use crate::provider_observation::{
            AcceptedSnapshot, CollectionState, ProviderBinding, ProviderCollection,
            ProviderTrackState, SubjectKey,
        };
        let mut context = TrackContext::new(
            crate::api::Track {
                title: Some("Source title".into()),
                track_guid: Some("track-1".into()),
                ..Default::default()
            },
            None,
        );
        context.provider_state = ProviderTrackState {
            binding: ProviderBinding::Proven,
            request_refresh: None,
            collections: vec![ProviderCollection {
                provider_id: 1,
                subject: SubjectKey::guid("feed", Some("track-1")),
                collection: "source_ids".into(),
                refresh: None,
                state: CollectionState::CompleteEmpty(AcceptedSnapshot {
                    snapshot_id: 1,
                    generation: 3,
                    observation_id: 2,
                    scope_ordinal: 0,
                    fetched_at_us: Some(10),
                    occurrence: serde_json::json!({}),
                    members: Vec::new(),
                }),
            }],
        };
        let legacy = serde_json::to_value((&context.track, &context.feed)).unwrap();
        let expected = context.provider_state.clone();
        crate::metadata::sanitize_track_context_source_text(&mut context);
        let mut frame = test_inspector_frame(test_track(1, 1, true));
        retain_track_source_context(&mut frame, 1, context.clone());
        let mounted = frame.clone().source_context.unwrap();
        assert_eq!(mounted.provider_state, expected);
        assert_eq!(
            serde_json::to_value((&mounted.track, &mounted.feed)).unwrap(),
            legacy
        );
        context.provider_state = crate::provider_observation::ProviderTrackState::default();
        retain_track_source_context(&mut frame, 99, context);
        assert_eq!(frame.source_context.unwrap().provider_state, expected);
    }

    fn test_inspector_frame(track: TrackRow) -> InspectorFrame {
        InspectorFrame {
            entity_id: track.id,
            title: "Track".into(),
            track,
            source_context: None,
            image: None,
            expanded_id3_frame_groups: BTreeSet::new(),
            expanded_metadata_cells: BTreeSet::new(),
            pending_id3_edits: BTreeMap::new(),
            suppressed_auto_id3_edits: BTreeSet::new(),
            applying_id3_edits: false,
            id3_apply_error: Some("stale error".into()),
            local_subscription: true,
            subscription_busy: true,
            subscription_message: None,
            tag_compare: LazyPanel::Empty("stale comparison".into()),
            musicbrainz_lookup: LazyPanel::Hidden,
            musicbrainz_selected: 0,
            inspector_state: LibraryTrackInspectorState::default(),
        }
    }

    fn library_observation_receipt(
        generation: i64,
    ) -> crate::provider_observation::ObservationReceipt {
        crate::provider_observation::ObservationReceipt {
            observation_id: generation,
            generation,
            provider_id: 1,
            resource_id: 1,
            request_uri: "private URI".into(),
            response_uri: None,
            body_key: Some("private body".into()),
            started_at_us: 10,
            finished_at_us: 20,
            fetched_at_us: Some(19),
            occurrence: serde_json::json!({}),
            outcome: crate::provider_observation::ObservationOutcome::Success,
            retention: crate::provider_observation::ObservationRetention::Unknown,
            collections: Vec::new(),
        }
    }

    #[test]
    fn adr_0075_library_observation_live_callbacks_keep_success_and_failure_after_navigation() {
        use crate::application::errors::command::ObservedQueryFailure;
        use crate::provider_observation::{ObservationCommandFailure, ProviderReadError};
        let mut vm = LibraryViewModel::new();
        let mut detail =
            LibraryDetail::Track(Box::new(test_inspector_frame(test_track(99, 1, true))));
        let mut context = track_row_to_track_context(&test_track(1, 1, true));
        context.observation_receipts = vec![library_observation_receipt(1)];
        let comparison = LibraryTrackCompare {
            track_context: context,
            tag_compare: crate::metadata::TagCompareResult {
                path: "local.mp3".into(),
                rows: Vec::new(),
                file_image: None,
                contributors: Vec::new(),
                value_routes: Vec::new(),
                id3_fields: Vec::new(),
                total_tracks: None,
                format: None,
            },
        };
        let status = vm.status_snapshot().text;
        apply_library_comparison_result(&mut vm, &mut detail, 1, comparison);
        assert_eq!(vm.retained_observation_evidence_counts(), (1, 0));
        let LibraryDetail::Track(frame) = &detail else {
            panic!("track remains selected")
        };
        assert!(frame.source_context.is_none());
        assert!(matches!(&frame.tag_compare, LazyPanel::Empty(text) if text == "stale comparison"));
        let error = CommandError::ObservedQueryFailure(Arc::new(ObservedQueryFailure::new(
            "private ordinary cause".into(),
            vec![library_observation_receipt(2)],
        )));
        assert_eq!(
            error.to_string(),
            CommandError::Query("private ordinary cause".into()).to_string()
        );
        assert_eq!(
            command_error_detail(error.clone()),
            command_error_detail(CommandError::Query("private ordinary cause".into()))
        );
        assert!(!format!("{error:?}").contains("private"));
        apply_library_comparison_failure(&mut vm, &mut detail, 1, error.clone());
        assert_eq!(vm.retained_observation_evidence_counts(), (2, 0));
        assert_eq!(vm.status_snapshot().text, status);
        apply_library_comparison_failure(&mut vm, &mut detail, 99, error.clone());
        let LibraryDetail::Track(frame) = &detail else {
            panic!("track remains selected")
        };
        assert!(
            matches!(&frame.tag_compare, LazyPanel::Empty(text) if text == &LibraryViewModel::deferred_panel_error_message(error.clone()))
        );
        assert_eq!(vm.status_snapshot().text, status);
        detail = LibraryDetail::None;
        apply_library_hydration_result(
            &mut vm,
            &mut detail,
            1,
            AlbumIdentityHydration {
                identity_facts: LocalIdentityFacts::default(),
                metadata_facts: crate::views::FeedMetadataFacts::default(),
                stored_values:
                    crate::application::queries::stored_values::FeedStoredValues::default(),
                description: Some("Hydrated description".into()),
                named_feeds: Vec::new(),
                observation_receipts: vec![library_observation_receipt(3)],
            },
        );
        assert!(matches!(detail, LibraryDetail::None));
        assert_eq!(vm.retained_observation_evidence_counts(), (3, 0));
        retain_library_query_failure(&mut vm, &error);
        assert_eq!(vm.status_snapshot().text, status);
        let conn = Connection::open_in_memory().unwrap();
        db::upgrades::create_fixture(&conn, 12).unwrap();
        conn.execute_batch("CREATE TEMP TRIGGER reject_observation AFTER INSERT ON metadata_observations BEGIN SELECT RAISE(ABORT,'fixture rejection'); END").unwrap();
        let recorder = crate::provider_observation::ProviderObservationRecorder::new(Arc::new(
            Mutex::new(conn),
        ));
        let token = recorder
            .begin(crate::provider_observation::contracts::rss_request(
                "https://example.test/feed",
                Some("track-1"),
                None,
            ))
            .unwrap();
        let capsule = recorder
            .record(
                token,
                crate::provider_observation::ProviderObservation {
                    body: Some(Arc::from(b"private response".as_slice())),
                    http_status: Some(200),
                    response_uri: None,
                    interpretation: serde_json::json!({}),
                    source_revision: None,
                    source_times: serde_json::json!({}),
                    decoder_version: "fixture".into(),
                    outcome: crate::provider_observation::ObservationOutcome::Partial,
                    failure: None,
                    finished_at_us: 20,
                    fetched_at_us: Some(19),
                    occurrence: serde_json::json!({}),
                    coverage: Vec::new(),
                },
            )
            .unwrap_err();
        let storage = CommandError::ObservationWriteFailure(Arc::new(ObservationCommandFailure {
            write_failure: Some(Arc::new(capsule)),
            storage_error: None,
            read_error: Some(ProviderReadError::Storage),
            receipts: vec![library_observation_receipt(4)].into(),
        }));
        apply_library_comparison_failure(&mut vm, &mut detail, 1, storage.clone());
        assert!(vm.status_snapshot().is_error);
        assert_eq!(vm.retained_observation_evidence_counts(), (4, 1));
        retain_library_query_failure(&mut vm, &storage);
        assert!(vm
            .status_snapshot()
            .text
            .contains("could not confirm response retention"));
        assert!(!vm.status_snapshot().text.contains("private"));
        apply_library_hydration_result(
            &mut vm,
            &mut detail,
            1,
            AlbumIdentityHydration {
                identity_facts: LocalIdentityFacts::default(),
                metadata_facts: crate::views::FeedMetadataFacts::default(),
                stored_values:
                    crate::application::queries::stored_values::FeedStoredValues::default(),
                description: None,
                named_feeds: Vec::new(),
                observation_receipts: vec![library_observation_receipt(5)],
            },
        );
        retain_library_query_failure(&mut vm, &error);
        assert_eq!(vm.retained_observation_evidence_counts(), (5, 1));
    }

    #[test]
    fn adr_0075_library_observation_comparison_callback_keeps_empty_state_and_album_updates() {
        use crate::provider_observation::{
            AcceptedSnapshot, CollectionState, ProviderBinding, ProviderCollection,
            ProviderTrackState, RefreshState, RequestRefresh, SubjectKey,
        };
        let mut context = track_row_to_track_context(&test_track(1, 1, true));
        context
            .observation_receipts
            .push(library_observation_receipt(5));
        context.provider_state = ProviderTrackState {
            binding: ProviderBinding::Proven,
            request_refresh: Some(RequestRefresh {
                request: crate::provider_observation::contracts::rss_request(
                    "https://example.test/feed",
                    Some("track-1"),
                    None,
                ),
                generation: 5,
                state: RefreshState::Failed,
                observation_id: Some(5),
                failure_id: Some(5),
            }),
            collections: vec![ProviderCollection {
                provider_id: 1,
                subject: SubjectKey::guid("feed", Some("track-1")),
                collection: "source_ids".into(),
                refresh: None,
                state: CollectionState::CompleteEmpty(AcceptedSnapshot {
                    snapshot_id: 1,
                    generation: 4,
                    observation_id: 4,
                    scope_ordinal: 0,
                    fetched_at_us: Some(19),
                    occurrence: serde_json::json!({}),
                    members: Vec::new(),
                }),
            }],
        };
        let expected = context.provider_state.clone();
        let comparison = LibraryTrackCompare {
            track_context: context,
            tag_compare: crate::metadata::TagCompareResult {
                path: "local.mp3".into(),
                rows: Vec::new(),
                file_image: None,
                contributors: Vec::new(),
                value_routes: Vec::new(),
                id3_fields: Vec::new(),
                total_tracks: None,
                format: None,
            },
        };
        let mut vm = LibraryViewModel::new();
        let mut detail =
            LibraryDetail::Track(Box::new(test_inspector_frame(test_track(1, 1, true))));
        apply_library_comparison_result(&mut vm, &mut detail, 1, comparison);
        let LibraryDetail::Track(frame) = &detail else {
            panic!("matching track")
        };
        assert_eq!(
            frame.source_context.as_ref().unwrap().provider_state,
            expected
        );
        assert!(matches!(&frame.tag_compare, LazyPanel::Loaded(_)));
        let album = test_album(1, vec![test_track(1, 1, true)]);
        vm.replace_tree(crate::view_models::library::LibraryTree {
            artists: vec![crate::view_models::library::ArtistNode {
                name: "Artist".into(),
                albums: vec![album.clone()],
            }],
        });
        detail = LibraryDetail::Album(album);
        let named_feeds = vec![crate::view_models::named_feeds::NamedFeed {
            feed_guid: "haleen-guid".into(),
            title: Some("Haleen".into()),
            position: Some(0),
            role: Some("artist".into()),
            role_agreement: Some("both".into()),
            feed_rel: None,
            album_rel: None,
            confirmed: true,
            image_url: None,
        }];
        apply_library_hydration_result(
            &mut vm,
            &mut detail,
            1,
            AlbumIdentityHydration {
                identity_facts: LocalIdentityFacts::default(),
                metadata_facts: crate::views::FeedMetadataFacts::default(),
                stored_values:
                    crate::application::queries::stored_values::FeedStoredValues::default(),
                description: Some("Current description".into()),
                named_feeds: named_feeds.clone(),
                observation_receipts: vec![library_observation_receipt(6)],
            },
        );
        let LibraryDetail::Album(album) = &detail else {
            panic!("matching album")
        };
        assert_eq!(album.description.as_deref(), Some("Current description"));
        assert_eq!(
            vm.tree().artists[0].albums[0].description,
            album.description
        );
        // R34-02 (ADR 0084 task 034): the hydration result reaches the
        // open album detail and the tree album's own `named_feeds`.
        assert_eq!(album.named_feeds, named_feeds);
        assert_eq!(vm.tree().artists[0].albums[0].named_feeds, named_feeds);
        assert_eq!(vm.retained_observation_evidence_counts(), (2, 0));
    }

    /// R34-11 (ADR 0084 task 034): the backfill candidate order holds
    /// every tree album with a feed id and a feed GUID, newest feed id
    /// first, and drops an album with no feed GUID.
    #[test]
    fn adr_0084_task_034_backfill_order_is_newest_feed_id_first() {
        let mut low = test_album(5, Vec::new());
        low.feed_guid = Some("feed-5".into());
        let mut high = test_album(20, Vec::new());
        high.feed_guid = Some("feed-20".into());
        let mut no_guid = test_album(30, Vec::new());
        no_guid.feed_guid = None;
        let tree = crate::view_models::library::LibraryTree {
            artists: vec![crate::view_models::library::ArtistNode {
                name: "Artist".into(),
                albums: vec![low, high, no_guid],
            }],
        };

        let order = album_identity_backfill_order(&tree);

        assert_eq!(
            order,
            vec![(20, "feed-20".to_owned()), (5, "feed-5".to_owned())]
        );
    }

    #[test]
    fn removal_keeps_album_detail_row_as_index_content() {
        let mut detail = LibraryDetail::Album(test_album(
            7,
            vec![test_track(1, 7, true), test_track(2, 7, true)],
        ));

        apply_library_removal_to_album_detail(&mut detail, LibraryRemovalTarget::Track(1));

        let LibraryDetail::Album(album) = detail else {
            panic!("detail remains an album");
        };
        assert!(
            !album.tracks[0].is_in_library,
            "removed row should remain in the album as Index content"
        );
        assert!(
            album.tracks[0].local_path.is_none(),
            "removed row must stop advertising a local file"
        );
        assert!(
            album.tracks[1].is_in_library,
            "unrelated rows should retain their local membership"
        );
    }

    #[test]
    fn removal_resets_open_track_inspector_to_downloadable_state() {
        let mut frame = test_inspector_frame(test_track(9, 7, true));

        apply_library_removal_to_inspector_frame(&mut frame, LibraryRemovalTarget::Track(9));

        assert!(!frame.subscription_busy);
        assert!(!frame.local_subscription);
        assert!(!frame.track.is_in_library);
        assert!(frame.track.local_path.is_none());
        assert!(matches!(frame.tag_compare, LazyPanel::Hidden));
        assert_eq!(frame.subscription_message.as_deref(), Some("Removed track"));
    }

    #[test]
    fn subscription_updates_album_detail_row_to_library_content() {
        let mut detail = LibraryDetail::Album(test_album(
            7,
            vec![test_track(1, 7, false), test_track(2, 7, true)],
        ));

        apply_track_subscription_to_album_detail(
            &mut detail,
            1,
            Some(LibraryRelativePath::for_test("music/track-1.mp3")),
            true,
        );

        let LibraryDetail::Album(album) = detail else {
            panic!("detail remains an album");
        };
        assert!(
            album.tracks[0].is_in_library,
            "downloaded row should move back to Library content in place"
        );
        assert_eq!(
            album.tracks[0]
                .local_path
                .as_ref()
                .map(LibraryRelativePath::as_stored),
            Some("music/track-1.mp3")
        );
        assert!(
            album.tracks[1].is_in_library,
            "unrelated rows should retain their membership"
        );
    }

    #[test]
    fn album_thumbnail_urls_include_open_detail_artwork_after_tree_removal() {
        let mut track = test_track(1, 7, false);
        track.track_image_href = Some("https://example.test/track.png".into());
        let album = test_album(7, vec![track]);

        let urls = album_thumbnail_urls(&album);

        assert!(
            urls.iter().any(|url| url == "https://example.test/art.png"),
            "open album detail artwork must stay prefetched even when the album is no longer in the library tree"
        );
        assert!(
            urls.iter()
                .any(|url| url == "https://example.test/track.png"),
            "track row artwork must stay prefetched for index-content rows"
        );
    }

    #[test]
    fn api_feed_from_album_preserves_track_download_sources() {
        let mut track = test_track(1, 7, false);
        track.enclosure_url = Some("https://example.test/audio.mp3".into());
        track.enclosure_type = Some("audio/mpeg".into());
        let album = test_album(7, vec![track]);

        let feed = api_feed_from_album(&album);
        let track = feed
            .tracks
            .as_deref()
            .and_then(|tracks| tracks.first())
            .expect("album track");

        assert_eq!(
            track.enclosure_url.as_deref(),
            Some("https://example.test/audio.mp3")
        );
        assert_eq!(track.enclosure_type.as_deref(), Some("audio/mpeg"));
    }

    /// The window of the R31-12 test: one drag source with a selection
    /// payload, and one playlist row with the id 7.
    struct SelectionDropTest {
        drag: crate::view_models::library::TrackDragVm,
        drops:
            std::rc::Rc<std::cell::RefCell<Vec<crate::ui::composites::track_drag::PlaylistDrop>>>,
    }

    impl gpui::Render for SelectionDropTest {
        fn render(
            &mut self,
            _: &mut gpui::Window,
            cx: &mut gpui::Context<Self>,
        ) -> impl gpui::IntoElement {
            use crate::ui::composites::track_drag::{
                attach_track_drag, playlist_drop_handler, playlist_drop_target, TrackDragPayload,
            };
            use gpui::{div, prelude::*};
            let drops = std::rc::Rc::clone(&self.drops);
            let on_drop = playlist_drop_handler(move |drop, _, _| {
                drops.borrow_mut().push(drop.clone());
            });
            let source = attach_track_drag(
                div()
                    .id("selection-drop-source")
                    .debug_selector(|| "selection-drop-source".to_owned())
                    .h(gpui::px(40.))
                    .w(gpui::px(200.))
                    .child("Face Pale"),
                TrackDragPayload::from_vm(&self.drag),
            );
            div().size_full().flex().flex_col().child(source).child(
                div()
                    .debug_selector(|| "selection-drop-target".to_owned())
                    .child(playlist_drop_target(
                        crate::ui::composites::ListRow::new("playlist-7")
                            .child("Focus")
                            .into_any_element(),
                        7,
                        &on_drop,
                        cx,
                    )),
            )
        }
    }

    /// R31-12 (ADR 0083 Decision 6, amended 2026-10-21): the drag of a
    /// checked row in `Selecting` carries several track ids. The drop on
    /// a playlist row gives one `PlaylistDrop` with each id, in list
    /// order. `add_tracks_to_playlist` then starts one append, and the
    /// status names the count. The guard
    /// `adr_0083_track_drag_r83_8_07_drop_uses_the_one_add_path` holds
    /// the host call `add_tracks_to_playlist(drop.track_ids.clone(), ..)`.
    /// This test does the view model step of that call.
    #[gpui::test]
    fn adr_0083_selection_drag_r31_12_drop_status_names_the_count(cx: &mut gpui::TestAppContext) {
        use crate::ui::composites::track_drag::PlaylistDrop;
        use crate::view_models::library::{PlaylistAppendOutcome, TrackDragVm};
        use gpui::{Modifiers, MouseButton};

        let drag = TrackDragVm::selection(true, &[9, 3, 5]).expect("a checked row drags");
        assert_eq!(drag.label, "3 tracks");
        let drops = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
        let window_drops = std::rc::Rc::clone(&drops);
        let (_view, cx) = cx.add_window_view(move |_, _| SelectionDropTest {
            drag,
            drops: window_drops,
        });
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let source = cx
            .debug_bounds("selection-drop-source")
            .expect("the source")
            .center();
        let target = cx
            .debug_bounds("selection-drop-target")
            .expect("the playlist row")
            .center();
        cx.simulate_mouse_down(source, MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_move(target, MouseButton::Left, Modifiers::default());
        cx.simulate_mouse_up(target, MouseButton::Left, Modifiers::default());
        assert_eq!(
            *drops.borrow(),
            [PlaylistDrop {
                track_ids: vec![9, 3, 5],
                playlist_id: 7,
            }]
        );

        let mut vm = LibraryViewModel::new();
        vm.replace_playlists(vec![db::Playlist {
            id: 7,
            name: "Focus".to_owned(),
            description: None,
            track_count: 0,
            created_at: 0,
            updated_at: 0,
        }]);
        let intents: Vec<_> = drops
            .borrow()
            .iter()
            .filter_map(|drop| vm.begin_playlist_append(drop.playlist_id, drop.track_ids.clone()))
            .collect();
        let [intent] = intents.as_slice() else {
            panic!("one drop must give one append, not {}", intents.len());
        };
        assert_eq!(intent.track_ids(), &[9, 3, 5]);
        assert_eq!(vm.status(), "Downloading 3 tracks...");
        vm.finish_playlist_append(intent, PlaylistAppendOutcome::new(3, 0, 0));
        assert_eq!(vm.status(), "Added 3 of 3 to Focus");
    }
}
