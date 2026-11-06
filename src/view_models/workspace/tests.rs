use super::{
    music_focus_moves, music_focus_target, music_page_content, release_download_updates_open_page,
    resolve_release_page_source, resolve_track_page_source, track_download_updates_open_page,
    track_release_download_updates_open_page, FrameNavigationEntry, FrameNavigationState,
    FrameShellDisplay, MusicColumns, MusicDestination, MusicFocusTarget, MusicPageContent,
    ReleaseKey, ReleasePageSource, TrackKey, TrackPageSource, WorkspaceFrameConfig,
    WorkspaceFrameId, WorkspaceFrameKind, WorkspaceFrameState, WorkspaceLayout,
    WorkspaceLayoutConfig, WorkspaceModelError,
};
use crate::view_models::artist_identity::ArtistNameKey;
use crate::view_models::search_results::SavedSearchId;

/// A saved-search entry with the text `query`.
fn saved_entry(id: i64, query: &str) -> FrameNavigationEntry {
    FrameNavigationEntry::SavedSearch {
        id: SavedSearchId(id),
        query: query.to_owned(),
    }
}

/// ADR 0089 Decision 4 (review of 2026-11-06): a keep renames the saved
/// search in every history entry with its id. Back, the current entry and
/// Forward keep the place of each entry, and an entry of another id does not
/// change.
#[test]
fn adr_0089_saved_search_rename_updates_each_history_entry_by_id() {
    let mut nav = FrameNavigationState::new(saved_entry(1, "old"));
    nav.push(FrameNavigationEntry::Destination(
        MusicDestination::Releases,
    ));
    nav.push(saved_entry(2, "other"));
    assert_eq!(
        nav.go_back().cloned().ok(),
        Some(FrameNavigationEntry::Destination(
            MusicDestination::Releases
        ))
    );
    // Back stack: saved 1 (old). Current: Releases. Forward: saved 2.

    nav.rename_saved_search_entries(SavedSearchId(1), "new");

    assert_eq!(nav.go_back().cloned().ok(), Some(saved_entry(1, "new")));
    assert_eq!(
        nav.go_forward().cloned().ok(),
        Some(FrameNavigationEntry::Destination(
            MusicDestination::Releases
        ))
    );
    assert_eq!(
        nav.go_forward().cloned().ok(),
        Some(saved_entry(2, "other"))
    );
}

/// ADR 0089 Decision 4 (review of 2026-11-06): the rename also reaches the
/// current location, including an ancestor on its path.
#[test]
fn adr_0089_saved_search_rename_updates_the_current_path() {
    let mut nav =
        FrameNavigationState::new(FrameNavigationEntry::Destination(MusicDestination::Search));
    nav.push(saved_entry(3, "bluegrass"));
    nav.rename_saved_search_entries(SavedSearchId(3), "bluegrass banjo");
    assert_eq!(
        nav.path_entries(),
        vec![
            FrameNavigationEntry::Destination(MusicDestination::Search),
            saved_entry(3, "bluegrass banjo"),
        ]
    );
}

fn frame(id: u64, kind: WorkspaceFrameKind) -> WorkspaceFrameState {
    WorkspaceFrameState::with_default_title(WorkspaceFrameId::new(id), kind)
}

/// Test helper: the default Music destination root, as a navigation entry.
const fn discover() -> FrameNavigationEntry {
    FrameNavigationEntry::Destination(MusicDestination::Search)
}

/// Test helper: a `Track` entry with no release, keyed by `id` as its item
/// GUID. Distinct `id` values give distinct, comparable entries.
fn track(id: i64) -> FrameNavigationEntry {
    FrameNavigationEntry::Track(TrackKey::from_index_track_target(None, id.to_string()))
}

#[test]
fn frame_shell_display_hides_close_when_not_allowed() {
    let frame = frame(7, WorkspaceFrameKind::Detail);

    let fixed_frame = FrameShellDisplay::from_frame(&frame, false);
    let closable_frame = FrameShellDisplay::from_frame(&frame, true);

    assert_eq!(
        fixed_frame.close, None,
        "non-closable frames should not expose a close command"
    );
    assert_eq!(
        closable_frame.close.map(|close| close.id),
        Some("workspace-frame-7-close".to_string()),
        "closable frames should expose a frame-scoped close command"
    );
}

#[test]
fn frame_shell_display_passes_through_header_text_and_slot_id() {
    let frame = WorkspaceFrameState::new(
        WorkspaceFrameId::new(7),
        WorkspaceFrameKind::ContentList,
        "Playlist",
    )
    .with_subtitle("Seven tracks")
    .with_status("Ready");

    let display = FrameShellDisplay::from_frame(&frame, true);

    assert_eq!(display.frame_id, WorkspaceFrameId::new(7));
    assert_eq!(display.title, "Playlist");
    assert_eq!(
        display.subtitle,
        Some("Seven tracks".to_string()),
        "subtitle should pass through from frame state"
    );
    assert_eq!(
        display.status,
        Some("Ready".to_string()),
        "status should pass through from frame state"
    );
    assert_eq!(
        display.content_slot_id, "workspace-frame-7-content",
        "content slot id should be stable and frame-scoped"
    );
    assert_eq!(
        display.action_menu_items,
        Vec::new(),
        "transitional workspace must not expose multi-frame actions before real frame content owners exist"
    );
}

#[test]
fn frame_shell_display_hides_header_when_no_header_facts_exist() {
    let frame = WorkspaceFrameState::new(
        WorkspaceFrameId::new(7),
        WorkspaceFrameKind::ContentList,
        "",
    );

    let display = FrameShellDisplay::from_frame(&frame, false);

    assert!(
        !display.header_visible(),
        "Durable ADR 0061 element hierarchy guard: root content frame must not render a redundant section header"
    );
}

#[test]
fn frame_shell_display_shows_header_when_frame_facts_exist() {
    let frame = WorkspaceFrameState::new(
        WorkspaceFrameId::new(7),
        WorkspaceFrameKind::ContentList,
        "Search Results",
    );

    let display = FrameShellDisplay::from_frame(&frame, false);

    assert!(
        display.header_visible(),
        "Durable ADR 0061 element hierarchy guard: informative frame headers must remain visible"
    );
}

#[test]
fn workspace_frame_navigation_isolated_per_frame() {
    let mut layout = WorkspaceLayout::default_layout();
    let first = WorkspaceFrameId::new(2);
    let second = WorkspaceFrameId::new(3);

    layout
        .reset_nav(first, FrameNavigationEntry::PlaylistDetail(7))
        .expect("first frame should exist");
    layout
        .reset_nav(second, track(42))
        .expect("second frame should exist");

    assert_eq!(
        layout
            .frame_nav(first)
            .expect("first frame navigation should exist")
            .current(),
        &FrameNavigationEntry::PlaylistDetail(7)
    );
    assert_eq!(
        layout
            .frame_nav(second)
            .expect("second frame navigation should exist")
            .current(),
        &track(42)
    );

    assert_eq!(
        layout.pop_nav(first),
        None,
        "a single-entry history should not go back"
    );
    layout
        .push_nav(first, track(99))
        .expect("first frame should exist");
    assert_eq!(
        layout.pop_nav(first),
        Some(FrameNavigationEntry::PlaylistDetail(7)),
        "popping one frame should not affect another frame's history"
    );
    assert_eq!(
        layout
            .frame_nav(second)
            .expect("second frame navigation should remain intact")
            .current(),
        &track(42)
    );
}

#[test]
fn default_layout_has_expected_workspace_shape() {
    let layout = WorkspaceLayout::default_layout();
    let kinds: Vec<_> = layout
        .frames()
        .iter()
        .map(WorkspaceFrameState::kind)
        .collect();

    assert_eq!(
        kinds,
        [WorkspaceFrameKind::ContentList, WorkspaceFrameKind::Detail],
        "default workspace should expose the ADR 0089 frame order: list, then detail"
    );
    assert_eq!(
        focused_frame_of(&layout).map(WorkspaceFrameState::kind),
        Some(WorkspaceFrameKind::ContentList),
        "default workspace should focus the primary content frame"
    );
    assert_eq!(
        layout
            .frames()
            .iter()
            .filter(|frame| frame.is_focused())
            .count(),
        1,
        "default workspace should mark exactly one focused frame"
    );
}

#[test]
fn empty_layout_has_no_focus_and_rejects_focus_mutation() {
    let mut layout =
        WorkspaceLayout::new(Vec::new(), None).expect("an empty frame list is a valid layout");

    assert!(
        layout.frames().is_empty(),
        "empty layout should contain no frames"
    );
    assert_eq!(
        layout.focused_frame_id(),
        None,
        "empty layout should not carry a focused frame id"
    );
    assert_eq!(
        layout.focus_frame(WorkspaceFrameId::new(1)),
        Err(WorkspaceModelError::EmptyLayout),
        "empty layout should not accept focus mutation"
    );
}

#[test]
fn single_frame_layout_marks_only_frame_focused() {
    let layout = WorkspaceLayout::new(
        vec![frame(10, WorkspaceFrameKind::Detail)],
        Some(WorkspaceFrameId::new(10)),
    )
    .expect("single-frame layout should be valid");

    assert_eq!(
        focused_frame_of(&layout).map(WorkspaceFrameState::id),
        Some(WorkspaceFrameId::new(10)),
        "single-frame layout should focus its only frame"
    );
    assert!(
        layout.frames()[0].is_focused(),
        "single-frame layout should mirror focus into the frame state"
    );
}

#[test]
fn single_frame_layout_without_requested_focus_focuses_only_frame() {
    let layout = WorkspaceLayout::new(vec![frame(10, WorkspaceFrameKind::Detail)], None)
        .expect("single-frame layout should be valid");

    assert_eq!(
        layout.focused_frame_id(),
        Some(WorkspaceFrameId::new(10)),
        "non-empty layouts should preserve a focus invariant"
    );
    assert!(
        layout.frames()[0].is_focused(),
        "implicit focus should mirror into the frame state"
    );
}

#[test]
fn multi_frame_focus_moves_between_existing_frames() {
    let mut layout = WorkspaceLayout::new(
        vec![
            frame(1, WorkspaceFrameKind::QueueNowPlaying),
            frame(2, WorkspaceFrameKind::ContentList),
            frame(3, WorkspaceFrameKind::Detail),
        ],
        Some(WorkspaceFrameId::new(1)),
    )
    .expect("multi-frame layout should be valid");

    layout
        .focus_frame(WorkspaceFrameId::new(3))
        .expect("existing frame should be focusable");

    assert_eq!(
        focused_frame_of(&layout).map(WorkspaceFrameState::id),
        Some(WorkspaceFrameId::new(3)),
        "focus should move to the requested frame"
    );
    assert_eq!(
        layout
            .frames()
            .iter()
            .filter(|frame| frame.is_focused())
            .count(),
        1,
        "multi-frame layout should mark exactly one focused frame"
    );
}

#[test]
fn invalid_layout_mutations_return_errors() {
    let mut layout = WorkspaceLayout::new(
        vec![frame(1, WorkspaceFrameKind::QueueNowPlaying)],
        Some(WorkspaceFrameId::new(1)),
    )
    .expect("initial layout should be valid");

    assert_eq!(
        layout.focus_frame(WorkspaceFrameId::new(99)),
        Err(WorkspaceModelError::FrameNotFound(WorkspaceFrameId::new(
            99
        ))),
        "focusing a missing frame should return an error"
    );
}

#[test]
fn duplicate_frames_are_rejected_at_construction() {
    assert_eq!(
        WorkspaceLayout::new(
            vec![
                frame(1, WorkspaceFrameKind::QueueNowPlaying),
                frame(1, WorkspaceFrameKind::Detail),
            ],
            Some(WorkspaceFrameId::new(1)),
        ),
        Err(WorkspaceModelError::DuplicateFrameId(
            WorkspaceFrameId::new(1)
        )),
        "constructor should reject duplicate frame ids"
    );
}

#[test]
fn replace_nav_preserves_full_history_for_visible_layout_projection() {
    let mut source = WorkspaceLayout::default_layout();
    let detail_id = detail_frame_id(&source);
    source
        .reset_nav(detail_id, FrameNavigationEntry::Search("jazz".to_string()))
        .expect("detail frame should exist");
    source
        .push_nav(detail_id, track(9))
        .expect("detail frame should accept drill-down");

    let mut projected = WorkspaceLayout::new(
        vec![frame(detail_id.value(), WorkspaceFrameKind::Detail)],
        Some(detail_id),
    )
    .expect("projected layout should be valid");
    projected
        .replace_nav(
            detail_id,
            source
                .frame_nav(detail_id)
                .expect("source detail nav should exist")
                .clone(),
        )
        .expect("projected detail frame should accept copied nav");

    let nav = projected
        .frame_nav(detail_id)
        .expect("projected detail nav should exist");
    assert_eq!(nav.current(), &track(9));
    assert_eq!(
        back_page(nav),
        Some(FrameNavigationEntry::Search("jazz".to_string())),
        "full navigation history should survive projection"
    );
}

#[test]
fn workspace_layout_config_round_trips() {
    let added_id = WorkspaceFrameId::new(4);
    let layout = WorkspaceLayout::new(
        vec![
            frame(1, WorkspaceFrameKind::QueueNowPlaying),
            frame(2, WorkspaceFrameKind::ContentList),
            frame(3, WorkspaceFrameKind::Detail),
            frame(added_id.value(), WorkspaceFrameKind::Detail),
        ],
        Some(added_id),
    )
    .expect("layout with an extra frame should be valid");

    let restored = WorkspaceLayout::from_config(Some(&layout.to_config()));

    assert_eq!(
        restored.to_config(),
        layout.to_config(),
        "config conversion should preserve frame order, kinds, and focus"
    );
    assert_eq!(
        restored.focused_frame_id(),
        Some(added_id),
        "config conversion should preserve focused frame id"
    );
}

#[test]
fn workspace_layout_config_from_before_broadcast_frame_still_loads() {
    let pre_broadcast_config = WorkspaceLayoutConfig {
        frames: vec![
            WorkspaceFrameConfig {
                id: 1,
                kind: Some(WorkspaceFrameKind::QueueNowPlaying),
            },
            WorkspaceFrameConfig {
                id: 2,
                kind: Some(WorkspaceFrameKind::ContentList),
            },
            WorkspaceFrameConfig {
                id: 3,
                kind: Some(WorkspaceFrameKind::Detail),
            },
            WorkspaceFrameConfig {
                id: 4,
                kind: Some(WorkspaceFrameKind::QueueNowPlaying),
            },
        ],
        focused_frame_id: Some(2),
    };

    let restored = WorkspaceLayout::from_config(Some(&pre_broadcast_config));

    assert_eq!(
        restored.to_config(),
        pre_broadcast_config,
        "layouts written before Broadcast existed should still load"
    );
    assert_eq!(
        focused_frame_of(&restored).map(WorkspaceFrameState::kind),
        Some(WorkspaceFrameKind::ContentList),
        "old layouts should preserve focus"
    );
}

#[test]
fn malformed_and_empty_workspace_layout_config_falls_back_to_default() {
    let empty = WorkspaceLayoutConfig {
        frames: Vec::new(),
        focused_frame_id: None,
    };
    let duplicate = WorkspaceLayoutConfig {
        frames: vec![
            WorkspaceFrameConfig {
                id: 1,
                kind: Some(WorkspaceFrameKind::QueueNowPlaying),
            },
            WorkspaceFrameConfig {
                id: 1,
                kind: Some(WorkspaceFrameKind::Detail),
            },
        ],
        focused_frame_id: Some(1),
    };
    let missing_focus = WorkspaceLayoutConfig {
        frames: vec![WorkspaceFrameConfig {
            id: 8,
            kind: Some(WorkspaceFrameKind::Detail),
        }],
        focused_frame_id: Some(99),
    };
    let default = WorkspaceLayout::default_layout().to_config();

    assert_eq!(
        WorkspaceLayout::from_config(None).to_config(),
        default,
        "missing config should fall back to default layout"
    );
    assert_eq!(
        WorkspaceLayout::from_config(Some(&empty)).to_config(),
        default,
        "empty config should fall back to default layout"
    );
    assert_eq!(
        WorkspaceLayout::from_config(Some(&duplicate)).to_config(),
        default,
        "duplicate frame ids should fall back to default layout"
    );
    assert_eq!(
        WorkspaceLayout::from_config(Some(&missing_focus)).to_config(),
        default,
        "missing focused frame should fall back to default layout"
    );
}

#[test]
fn navigation_back_forward_boundaries_return_errors() {
    let mut nav = FrameNavigationState::new(discover());

    assert!(
        !nav.can_go_back(),
        "new navigation state should not have back history"
    );
    assert!(
        !nav.can_go_forward(),
        "new navigation state should not have forward history"
    );
    assert_eq!(
        nav.go_back(),
        Err(WorkspaceModelError::CannotNavigateBack),
        "back at the first entry should return an error"
    );
    assert_eq!(
        nav.go_forward(),
        Err(WorkspaceModelError::CannotNavigateForward),
        "forward without forward history should return an error"
    );
}

#[test]
fn navigation_push_pop_round_trip() {
    let mut nav = FrameNavigationState::new(FrameNavigationEntry::PlaylistDetail(7));
    nav.push(track(42));

    assert_eq!(
        nav.current(),
        &track(42),
        "push should update the current entry"
    );
    assert!(nav.can_go_back(), "push should create back history");
    assert!(!nav.can_go_forward(), "push should clear forward history");

    assert_eq!(
        nav.go_back().cloned(),
        Ok(FrameNavigationEntry::PlaylistDetail(7)),
        "go_back should restore the previous entry"
    );
    assert!(
        nav.can_go_forward(),
        "go_back should create forward history"
    );

    assert_eq!(
        nav.go_forward().cloned(),
        Ok(track(42)),
        "go_forward should restore the pushed entry"
    );
    assert!(
        !nav.can_go_forward(),
        "round-trip should consume forward history"
    );
}

/// R15-03 (ADR 0046 Task 015): a push after a Back clears the forward
/// history, and the chrome display exposes Forward as unavailable again.
#[test]
fn adr_0046_forward_push_after_back_clears_forward_history() {
    let mut nav = FrameNavigationState::new(FrameNavigationEntry::PlaylistDetail(1));
    nav.push(track(42));
    nav.go_back()
        .expect("pushed navigation should allow going back");
    assert!(
        nav.can_go_forward(),
        "a Back press must leave forward history for this test to be meaningful"
    );

    nav.push(FrameNavigationEntry::Release(ReleaseKey::LibraryFeed(5)));

    assert!(
        !nav.can_go_forward(),
        "a push after a Back must clear forward history"
    );
}

/// R15-02 (ADR 0046 Task 015): `WorkspaceLayout::pop_nav_forward` restores
/// the entry that `pop_nav` (Back) left, and the chrome then shows Forward
/// as unavailable again.
#[test]
fn adr_0046_forward_pop_nav_forward_restores_entry_back_left() {
    let mut layout = WorkspaceLayout::default_layout();
    let content_list_id = layout
        .frames()
        .iter()
        .find(|frame| frame.kind() == WorkspaceFrameKind::ContentList)
        .map(WorkspaceFrameState::id)
        .expect("default layout should include a content-list frame");

    layout
        .push_nav(content_list_id, track(42))
        .expect("content-list frame should exist");

    let entry_back_left = layout
        .pop_nav(content_list_id)
        .expect("a pushed entry should allow going back");
    assert_eq!(
        entry_back_left,
        // ADR 0089 Decision 8: the default layout opens on Releases. A pushed
        // page therefore sits on that destination.
        FrameNavigationEntry::Destination(MusicDestination::Releases),
        "Back should restore the entry that preceded the push"
    );

    let restored_by_forward = layout
        .pop_nav_forward(content_list_id)
        .expect("a Back press should leave a forward entry");
    assert_eq!(
        restored_by_forward,
        track(42),
        "Forward should restore the entry that Back left"
    );

    let nav = layout
        .frame_nav(content_list_id)
        .expect("content-list frame navigation should exist");
    assert!(
        !nav.can_go_forward(),
        "Forward must become unavailable once its only entry is restored"
    );
}

/// R4-04 (ADR 0077 packet 004): the publisher navigation entry keys on the
/// publisher feed GUID, so two different GUIDs are two different entries
/// and pushing a repeated GUID is a no-op, the same as every other entry.
#[test]
fn adr_0077_publisher_navigation_entry_keys_on_guid() {
    let mut nav = FrameNavigationState::new(discover());

    nav.push(FrameNavigationEntry::PublisherDetail("guid-a".into()));
    assert_eq!(
        nav.current(),
        &FrameNavigationEntry::PublisherDetail("guid-a".into())
    );
    assert!(nav.can_go_back());

    nav.push(FrameNavigationEntry::PublisherDetail("guid-a".into()));
    assert!(
        !nav.can_go_forward(),
        "repeating the same publisher feed GUID must not push a duplicate entry"
    );

    nav.push(FrameNavigationEntry::PublisherDetail("guid-b".into()));
    assert_eq!(
        nav.current(),
        &FrameNavigationEntry::PublisherDetail("guid-b".into())
    );
    assert_ne!(
        nav.current(),
        &FrameNavigationEntry::PublisherDetail("guid-a".into()),
        "two different publisher feed GUIDs must be two different entries"
    );
}

#[test]
fn navigation_push_current_entry_is_noop() {
    let mut nav = FrameNavigationState::new(FrameNavigationEntry::PlaylistDetail(7));

    nav.push(FrameNavigationEntry::PlaylistDetail(7));

    assert_eq!(
        nav.current(),
        &FrameNavigationEntry::PlaylistDetail(7),
        "same-entry push should preserve the current destination"
    );
    assert!(
        !nav.can_go_back(),
        "same-entry push should not create synthetic back history"
    );
    assert!(
        !nav.can_go_forward(),
        "same-entry push should not create synthetic forward history"
    );
}

#[test]
fn navigation_reset_clears_back_and_forward_history() {
    let mut nav = FrameNavigationState::new(FrameNavigationEntry::PlaylistDetail(7));
    nav.push(track(42));
    nav.go_back()
        .expect("pushed navigation should allow going back");

    nav.reset(track(99));

    assert_eq!(
        nav.current(),
        &track(99),
        "reset should replace the current entry"
    );
    assert!(!nav.can_go_back(), "reset should clear back history");
    assert!(!nav.can_go_forward(), "reset should clear forward history");
}

#[test]
fn open_search_results_in_content_list_pushes_search_onto_content_list_nav() {
    let mut layout = WorkspaceLayout::default_layout();

    let result = layout.open_search_results_in_content_list("ambient");

    assert_eq!(
        result,
        Ok(WorkspaceFrameId::new(2)),
        "should return the ContentList frame id"
    );
    let nav = layout
        .frame_nav(WorkspaceFrameId::new(2))
        .expect("ContentList should have navigation state");
    assert_eq!(
        nav.current(),
        &FrameNavigationEntry::Search("ambient".to_string()),
        "ContentList nav top should be the search query"
    );
    assert_eq!(
        layout.focused_frame_id(),
        Some(WorkspaceFrameId::new(2)),
        "ContentList should be focused"
    );
}

#[test]
fn open_search_results_in_content_list_pushes_from_non_search_nav() {
    let mut layout = WorkspaceLayout::default_layout();
    let content_list_id = WorkspaceFrameId::new(2);

    layout
        .reset_nav(content_list_id, FrameNavigationEntry::PlaylistDetail(7))
        .expect("should reset ContentList nav");
    layout
        .open_search_results_in_content_list("drums")
        .expect("should push search onto ContentList");

    let nav = layout
        .frame_nav(content_list_id)
        .expect("ContentList should have navigation state");
    assert_eq!(
        nav.current(),
        &FrameNavigationEntry::Search("drums".to_string()),
        "current should be the new search"
    );
    assert!(
        nav.can_go_back(),
        "back history should contain the playlist detail entry"
    );
    assert_eq!(
        back_page(nav),
        Some(FrameNavigationEntry::PlaylistDetail(7)),
        "back should go to the playlist detail"
    );
}

#[test]
fn open_search_results_in_content_list_replaces_current_search_nav() {
    let mut layout = WorkspaceLayout::default_layout();
    let content_list_id = WorkspaceFrameId::new(2);

    layout
        .reset_nav(content_list_id, FrameNavigationEntry::PlaylistDetail(7))
        .expect("should reset ContentList nav");
    layout
        .open_search_results_in_content_list("drums")
        .expect("should push the first search");
    layout
        .open_search_results_in_content_list("survival guide")
        .expect("should replace the current search");

    let nav = layout
        .frame_nav(content_list_id)
        .expect("ContentList should have navigation state");
    assert_eq!(
        nav.current(),
        &FrameNavigationEntry::Search("survival guide".to_string()),
        "current should be the latest search"
    );
    assert_eq!(
        back_page(nav),
        Some(FrameNavigationEntry::PlaylistDetail(7)),
        "back should skip the overwritten search and return to prior content"
    );
}

/// ADR 0084 Decisions 2 and 9, walk of 2026-10-21: a new search after a
/// release page of an earlier search starts a path with the search page
/// only. The release page is not in the new path, so the detail column
/// shows no page of the earlier search. Back returns to the release page.
#[test]
fn adr_0084_search_list_new_search_keeps_no_page_of_the_earlier_search() {
    let mut layout = WorkspaceLayout::default_layout();
    let content_list_id = WorkspaceFrameId::new(2);
    let release = FrameNavigationEntry::Release(ReleaseKey::FeedGuid("haleen-music".to_string()));

    layout
        .open_search_results_in_content_list("haleen")
        .expect("should open the first search");
    layout
        .push_nav(content_list_id, release.clone())
        .expect("should open the release page");
    layout
        .open_search_results_in_content_list("survival guide")
        .expect("should open the second search");

    let nav = layout
        .frame_nav(content_list_id)
        .expect("ContentList should have navigation state");
    let search = FrameNavigationEntry::Search("survival guide".to_string());
    assert_eq!(nav.current(), &search);
    assert_eq!(nav.path_entries(), vec![search]);
    assert_eq!(nav.active_search_query(), Some("survival guide"));
    assert_eq!(back_page(nav), Some(release));
}

#[test]
fn open_search_results_in_content_list_replaces_search_ancestor_nav() {
    let mut layout = WorkspaceLayout::default_layout();
    let content_list_id = WorkspaceFrameId::new(2);

    layout
        .reset_nav(content_list_id, FrameNavigationEntry::PlaylistDetail(7))
        .expect("should reset ContentList nav");
    layout
        .open_search_results_in_content_list("heycitizen")
        .expect("should push the first search");
    layout
        .push_nav(
            content_list_id,
            FrameNavigationEntry::ArtistName(ArtistNameKey::from_text("HeyCitizen")),
        )
        .expect("should push selected result detail");
    layout
        .open_search_results_in_content_list("survival guide")
        .expect("should replace the active search flow");

    let nav = layout
        .frame_nav(content_list_id)
        .expect("ContentList should have navigation state");
    assert_eq!(
        nav.current(),
        &FrameNavigationEntry::Search("survival guide".to_string()),
        "current should be the latest search"
    );
    assert_eq!(
        nav.path_entries(),
        vec![FrameNavigationEntry::Search("survival guide".to_string())],
        "ADR 0084 Decision 9: a search from a detail page starts a new path"
    );
    assert_eq!(
        back_page(nav),
        Some(FrameNavigationEntry::ArtistName(ArtistNameKey::from_text(
            "HeyCitizen"
        ))),
        "ADR 0084 Decision 9: Back returns to the selected detail of the earlier search"
    );
}

/// N1-01 (ADR 0084 Decision 9): the location path holds each page one time
/// at most, also after a sequence that opens a page two times.
#[test]
fn adr_0084_nav_location_path_holds_each_page_once() {
    let mut nav = FrameNavigationState::new(discover());
    nav.start_path(discover(), Some(FrameNavigationEntry::PlaylistDetail(0)));
    nav.push(track(7));
    nav.push(FrameNavigationEntry::Release(ReleaseKey::LibraryFeed(3)));
    nav.push(track(7));

    assert_eq!(
        nav.path_entries(),
        vec![
            discover(),
            FrameNavigationEntry::PlaylistDetail(0),
            track(7),
        ],
        "the second open of the track must cut the path at the track"
    );
}

/// N1-02 (ADR 0084 Decision 9): a push of a page that the path holds cuts
/// the path at that page, and Back returns to the longer path.
#[test]
fn adr_0084_nav_push_of_path_page_cuts_path_and_back_restores() {
    let mut nav = FrameNavigationState::new(discover());
    nav.push(FrameNavigationEntry::PlaylistDetail(0));
    nav.push(track(7));
    let longer_path = nav.path_entries();

    nav.push(discover());

    assert_eq!(
        nav.path_entries(),
        vec![discover()],
        "a page must cut the path at that page"
    );
    assert_eq!(
        nav.go_back().cloned(),
        Ok(track(7)),
        "Back must return to the page before the cut"
    );
    assert_eq!(
        nav.path_entries(),
        longer_path,
        "Back must restore the longer path"
    );
}

/// ADR 0084 Decision 9, operator check of 2026-10-18: a second row of the
/// Artists list replaced no page. The path read "Artists > `HeyCitizen` >
/// Haleen". A list row now opens directly under the destination, and Back
/// returns to the first row's page.
#[test]
fn adr_0084_nav_list_row_opens_under_the_destination() {
    let artists = FrameNavigationEntry::Destination(MusicDestination::Artists);
    let heycitizen = FrameNavigationEntry::PublisherDetail("heycitizen-guid".to_string());
    let haleen = FrameNavigationEntry::ArtistName(ArtistNameKey::from_text("Haleen"));
    let mut nav = FrameNavigationState::new(artists.clone());

    nav.open_list_row(heycitizen.clone());
    nav.open_list_row(haleen.clone());
    assert_eq!(nav.path_entries(), vec![artists.clone(), haleen.clone()]);

    nav.push(track(7));
    nav.open_list_row(heycitizen.clone());
    assert_eq!(
        nav.path_entries(),
        vec![artists.clone(), heycitizen],
        "a list row replaces each open page under the destination"
    );

    nav.go_back().expect("Back must be available");
    assert_eq!(nav.path_entries(), vec![artists.clone(), haleen, track(7)]);

    let mut root_only = FrameNavigationState::new(artists.clone());
    root_only.open_list_row(artists.clone());
    assert_eq!(
        root_only.path_entries(),
        vec![artists],
        "the destination itself stays one entry"
    );
}

/// N1-03 (ADR 0084 Decision 9): a new path holds the destination and the
/// page only, and Back returns to the earlier location.
#[test]
fn adr_0084_nav_start_path_holds_destination_and_page() {
    let mut nav = FrameNavigationState::new(discover());
    nav.push(FrameNavigationEntry::PlaylistDetail(0));
    nav.push(track(7));
    let earlier_path = nav.path_entries();

    nav.start_path(
        discover(),
        Some(FrameNavigationEntry::ArtistName(ArtistNameKey::from_text(
            "Haleen",
        ))),
    );

    assert_eq!(
        nav.path_entries(),
        vec![
            discover(),
            FrameNavigationEntry::ArtistName(ArtistNameKey::from_text("Haleen")),
        ],
        "a sidebar click must start a new path"
    );
    assert!(nav.can_go_back(), "a sidebar click must add history");
    nav.go_back().expect("Back must be available");
    assert_eq!(
        nav.path_entries(),
        earlier_path,
        "Back must restore the earlier location"
    );
}

/// N1-04 (ADR 0084 Decision 9): a push after a Back clears the forward
/// history.
#[test]
fn adr_0084_nav_push_after_back_clears_forward_history() {
    let mut nav = FrameNavigationState::new(discover());
    nav.start_path(discover(), Some(FrameNavigationEntry::PlaylistDetail(0)));
    nav.go_back().expect("Back must be available");
    assert!(nav.can_go_forward(), "Back must create forward history");

    nav.push(FrameNavigationEntry::Release(ReleaseKey::LibraryFeed(3)));

    assert!(
        !nav.can_go_forward(),
        "a push after Back must clear forward history"
    );
}

/// N1-05 (ADR 0084 Decision 9): a new query on the search page replaces
/// the query in place. A search from another page adds one history entry.
#[test]
fn adr_0084_nav_search_replaces_in_place_or_adds_one_entry() {
    let mut nav = FrameNavigationState::new(discover());
    nav.push(FrameNavigationEntry::PlaylistDetail(0));

    nav.replace_active_search_or_push(FrameNavigationEntry::Search("drums".to_string()));
    nav.replace_active_search_or_push(FrameNavigationEntry::Search("bass".to_string()));

    assert_eq!(
        nav.path_entries(),
        vec![FrameNavigationEntry::Search("bass".to_string())],
        "a search must start its own path, and a new query must replace it"
    );
    assert_eq!(
        nav.go_back().cloned(),
        Ok(FrameNavigationEntry::PlaylistDetail(0)),
        "the two queries must add one history entry"
    );
}

/// R2-01 (ADR 0084 Decision 4): the release key of a Library album with a
/// feed GUID equals the release key of an Index row with the same feed
/// GUID.
#[test]
fn adr_0084_release_feed_guid_key_matches_across_origins() {
    let library_key = ReleaseKey::FeedGuid("feed-guid".to_string());
    let index_key = ReleaseKey::from_index_feed_guid("feed-guid");

    assert_eq!(library_key, index_key);
}

/// R2-02 (ADR 0084 Decision 4): the release key of a Library album with
/// no feed GUID is `LibraryFeed` with its feed id.
#[test]
fn adr_0084_release_library_feed_key_carries_its_feed_id() {
    let key = ReleaseKey::LibraryFeed(42);

    assert_eq!(key, ReleaseKey::LibraryFeed(42));
    assert_ne!(key, ReleaseKey::FeedGuid("42".to_string()));
}

/// R2-03 (ADR 0084 Decision 4): the resolver gives the Library source for
/// `LibraryFeed`, and for `FeedGuid` when the lookup finds a feed. It gives
/// the Index source when the lookup finds no feed.
#[test]
fn adr_0084_release_resolver_follows_the_lookup_result() {
    assert_eq!(
        resolve_release_page_source(ReleaseKey::LibraryFeed(7), None),
        ReleasePageSource::Library { feed_id: 7 },
        "a Library feed key never needs a lookup"
    );
    assert_eq!(
        resolve_release_page_source(ReleaseKey::FeedGuid("guid".to_string()), Some(9)),
        ReleasePageSource::Library { feed_id: 9 },
        "a found feed resolves to the Library source"
    );
    assert_eq!(
        resolve_release_page_source(ReleaseKey::FeedGuid("guid".to_string()), None),
        ReleasePageSource::Index {
            feed_guid: "guid".to_string()
        },
        "no found feed resolves to the Index source"
    );
}

/// R2-04 (ADR 0084 Decision 4): a path that opens one release from the
/// Library and then from the Index holds the release one time. The two
/// opens push the same key, so the second push cuts the path at the first.
#[test]
fn adr_0084_release_path_holds_one_release_opened_from_both_origins() {
    let mut nav = FrameNavigationState::new(discover());
    let library_open = FrameNavigationEntry::Release(ReleaseKey::FeedGuid("feed-guid".into()));
    let index_open = FrameNavigationEntry::Release(ReleaseKey::from_index_feed_guid("feed-guid"));

    nav.push(library_open);
    nav.push(FrameNavigationEntry::ArtistName(ArtistNameKey::from_text(
        "Haleen",
    )));
    nav.push(index_open);

    assert_eq!(
        nav.path_entries(),
        vec![
            discover(),
            FrameNavigationEntry::Release(ReleaseKey::FeedGuid("feed-guid".into())),
        ],
        "the Index open must cut the path at the earlier Library open of the same release"
    );
}

/// Returns the page that a Back press would show, without a change to
/// `nav`.
fn back_page(nav: &FrameNavigationState) -> Option<FrameNavigationEntry> {
    let mut copy = nav.clone();
    copy.go_back().ok().cloned()
}

/// R2-07 (ADR 0084 Decision 4): a completed download changes only the open
/// release page of the downloaded feed GUID.
#[test]
fn adr_0084_release_download_updates_only_the_open_release_page() {
    let open = FrameNavigationEntry::Release(ReleaseKey::FeedGuid("feed-a".to_string()));
    let other = FrameNavigationEntry::Release(ReleaseKey::FeedGuid("feed-b".to_string()));

    assert!(release_download_updates_open_page(
        Some(&open),
        Some("feed-a")
    ));
    assert!(
        !release_download_updates_open_page(Some(&other), Some("feed-a")),
        "a download must not change another open release"
    );
    assert!(
        !release_download_updates_open_page(Some(&discover()), Some("feed-a")),
        "a download must not change a page that is not a release"
    );
    assert!(
        !release_download_updates_open_page(Some(&open), None),
        "a download with no feed GUID must not change the open page"
    );
}

/// R3-01 (ADR 0084 Decision 4): the track key of a Library track with a
/// feed GUID equals the track key of an Index target with the same feed
/// GUID and item GUID.
#[test]
fn adr_0084_track_feed_guid_key_matches_across_origins() {
    let library_key = TrackKey {
        release: Some(ReleaseKey::FeedGuid("feed-guid".to_string())),
        item_guid: "item-guid".to_string(),
    };
    let index_key = TrackKey::from_index_track_target(Some("feed-guid"), "item-guid");

    assert_eq!(library_key, index_key);
}

/// R3-02 (ADR 0084 Decision 4): the track key of a Library track in a feed
/// with no feed GUID carries `ReleaseKey::LibraryFeed`.
#[test]
fn adr_0084_track_library_feed_key_carries_its_feed_id() {
    let key = TrackKey {
        release: Some(ReleaseKey::LibraryFeed(42)),
        item_guid: "item-guid".to_string(),
    };

    assert_eq!(key.release, Some(ReleaseKey::LibraryFeed(42)));
    assert_ne!(key.release, Some(ReleaseKey::FeedGuid("42".to_string())));
}

/// R3-03 (ADR 0084 Decision 4): the resolver gives the Library source when
/// the lookup finds the track, and the Index source when it does not. A
/// key with no release gives the Index source, whatever the lookup gives.
#[test]
fn adr_0084_track_resolver_follows_the_lookup_result() {
    let feed_guid_key = TrackKey::from_index_track_target(Some("guid"), "item-guid");
    assert_eq!(
        resolve_track_page_source(&feed_guid_key, Some(9)),
        TrackPageSource::Library { track_id: 9 },
        "a found track resolves to the Library source"
    );
    assert_eq!(
        resolve_track_page_source(&feed_guid_key, None),
        TrackPageSource::Index {
            target: feed_guid_key.index_target()
        },
        "no found track resolves to the Index source"
    );

    let no_release_key = TrackKey::from_index_track_target(None, "item-guid");
    assert_eq!(
        resolve_track_page_source(&no_release_key, None),
        TrackPageSource::Index {
            target: no_release_key.index_target()
        },
        "a key with no release gives the Index source"
    );
    assert_eq!(
        resolve_track_page_source(&no_release_key, Some(9)),
        TrackPageSource::Index {
            target: no_release_key.index_target()
        },
        "a key with no release gives the Index source even when a lookup result is given"
    );
}

/// R3-06 support (ADR 0084 Decision 4): a completed single-track download
/// changes only the open track page of that exact track.
#[test]
fn adr_0084_track_download_updates_only_the_open_track_page() {
    let open =
        FrameNavigationEntry::Track(TrackKey::from_index_track_target(Some("feed-a"), "item-a"));
    let other =
        FrameNavigationEntry::Track(TrackKey::from_index_track_target(Some("feed-a"), "item-b"));
    let downloaded = TrackKey::from_index_track_target(Some("feed-a"), "item-a");

    assert!(track_download_updates_open_page(
        Some(&open),
        Some(&downloaded)
    ));
    assert!(
        !track_download_updates_open_page(Some(&other), Some(&downloaded)),
        "a download must not change another open track"
    );
    assert!(
        !track_download_updates_open_page(Some(&discover()), Some(&downloaded)),
        "a download must not change a page that is not a track"
    );
    assert!(
        !track_download_updates_open_page(Some(&open), None),
        "a download with no key must not change the open page"
    );
}

/// R3-06 support (ADR 0084 Decision 4): a completed album download changes
/// only the open track page of a track in the downloaded release.
#[test]
fn adr_0084_track_release_download_updates_only_the_open_track_page() {
    let open =
        FrameNavigationEntry::Track(TrackKey::from_index_track_target(Some("feed-a"), "item-a"));
    let other =
        FrameNavigationEntry::Track(TrackKey::from_index_track_target(Some("feed-b"), "item-b"));
    let no_release = FrameNavigationEntry::Track(TrackKey::from_index_track_target(None, "item-c"));

    assert!(track_release_download_updates_open_page(
        Some(&open),
        Some("feed-a")
    ));
    assert!(
        !track_release_download_updates_open_page(Some(&other), Some("feed-a")),
        "a download must not change a track of a different release"
    );
    assert!(
        !track_release_download_updates_open_page(Some(&no_release), Some("feed-a")),
        "a track with no release never matches a downloaded release"
    );
    assert!(
        !track_release_download_updates_open_page(Some(&discover()), Some("feed-a")),
        "a download must not change a page that is not a track"
    );
    assert!(
        !track_release_download_updates_open_page(Some(&open), None),
        "a download with no feed GUID must not change the open page"
    );
}

/// Returns the focused frame of `layout`, through its live accessors.
fn focused_frame_of(layout: &WorkspaceLayout) -> Option<&WorkspaceFrameState> {
    let focused_id = layout.focused_frame_id()?;
    layout
        .frames()
        .iter()
        .find(|frame| frame.id() == focused_id)
}

/// Returns the id of the Detail frame of `layout`.
fn detail_frame_id(layout: &WorkspaceLayout) -> WorkspaceFrameId {
    layout
        .frames()
        .iter()
        .find(|frame| frame.kind() == WorkspaceFrameKind::Detail)
        .map(WorkspaceFrameState::id)
        .expect("the default layout has a Detail frame")
}

// ---------------------------------------------------------------------------
// ADR 0084 Decision 1: the three-column Music shell (task 004).
// ---------------------------------------------------------------------------

/// ADR 0089 Decision 5: a page with no content and nothing loading shows its
/// list root, and a loading page shows the detail frame with its loading
/// state. The content state, not the entry alone, decides this.
#[test]
fn adr_0089_page_without_content_falls_back_to_the_list_root() {
    let release = FrameNavigationEntry::Release(ReleaseKey::FeedGuid("feed".to_string()));
    assert_eq!(
        MusicColumns::for_entry(&release, MusicPageContent::Absent),
        MusicColumns::ListRoot
    );
    assert_eq!(
        MusicColumns::for_entry(&release, MusicPageContent::Loading),
        MusicColumns::Page
    );
    assert_eq!(
        MusicColumns::for_entry(&release, MusicPageContent::Loaded),
        MusicColumns::Page
    );
}

/// ADR 0089 Decision 5: the page content state. An own-content page is loaded.
/// A Library page is loaded when its detail holds, loading while the tree
/// reloads, and absent otherwise.
#[test]
fn adr_0089_music_page_content_state_table() {
    assert_eq!(
        music_page_content(true, false, false),
        MusicPageContent::Loaded
    );
    assert_eq!(
        music_page_content(false, true, true),
        MusicPageContent::Loaded
    );
    assert_eq!(
        music_page_content(false, false, true),
        MusicPageContent::Loading
    );
    assert_eq!(
        music_page_content(false, false, false),
        MusicPageContent::Absent
    );
}

/// ADR 0089 Invariants: the keyboard target follows the visible frame. A page
/// takes the detail column, a saved-search page takes its own focus, and a
/// list root takes the list column or the search results.
#[test]
fn adr_0089_focus_target_follows_the_visible_frame() {
    let release = FrameNavigationEntry::Release(ReleaseKey::FeedGuid("feed".to_string()));
    let saved = FrameNavigationEntry::SavedSearch {
        id: SavedSearchId(1),
        query: "drums".to_string(),
    };
    let list_root = FrameNavigationEntry::Destination(MusicDestination::Releases);
    let search = FrameNavigationEntry::Search("drums".to_string());

    assert_eq!(
        music_focus_target(MusicColumns::Page, &release, false),
        MusicFocusTarget::DetailColumn
    );
    assert_eq!(
        music_focus_target(MusicColumns::Page, &saved, false),
        MusicFocusTarget::SavedSearchPage
    );
    assert_eq!(
        music_focus_target(MusicColumns::ListRoot, &list_root, false),
        MusicFocusTarget::ListColumn
    );
    assert_eq!(
        music_focus_target(MusicColumns::ListRoot, &search, true),
        MusicFocusTarget::SearchResults
    );
    assert_eq!(
        music_focus_target(
            MusicColumns::ListRoot,
            &FrameNavigationEntry::Destination(MusicDestination::Search),
            false
        ),
        MusicFocusTarget::SearchField
    );
}

/// C4-02 (ADR 0084 Decisions 1 and 2): a list root opens no detail page, so
/// the narrow layout keeps the list column for it.
#[test]
fn adr_0084_columns_list_roots_open_no_detail_page() {
    for root in [
        FrameNavigationEntry::Destination(MusicDestination::Search),
        FrameNavigationEntry::Search("drums".to_string()),
        FrameNavigationEntry::Destination(MusicDestination::Inbox),
    ] {
        assert!(!root.opens_detail_page(), "{root:?} must open no page");
    }
    for page in [
        FrameNavigationEntry::PlaylistDetail(1),
        FrameNavigationEntry::Release(ReleaseKey::FeedGuid("feed".to_string())),
        FrameNavigationEntry::ArtistName(ArtistNameKey::from_text("Haleen")),
    ] {
        assert!(page.opens_detail_page(), "{page:?} must open a page");
    }
}

/// D5-02 (ADR 0084 Decisions 2 and 9): a sidebar click starts a new path
/// at the destination alone. A row of a destination list then gives the
/// path the destination, then the page.
#[test]
fn adr_0084_destinations_d5_02_sidebar_click_starts_a_path_a_row_extends_it() {
    let mut nav = FrameNavigationState::new(discover());

    nav.start_path(
        FrameNavigationEntry::Destination(MusicDestination::Releases),
        None,
    );
    assert_eq!(
        nav.path_entries(),
        vec![FrameNavigationEntry::Destination(
            MusicDestination::Releases
        )],
        "a sidebar click must give a path with the destination alone"
    );

    let release = FrameNavigationEntry::Release(ReleaseKey::LibraryFeed(7));
    nav.push(release.clone());
    assert_eq!(
        nav.path_entries(),
        vec![
            FrameNavigationEntry::Destination(MusicDestination::Releases),
            release
        ],
        "a row must give the path the destination, then the page"
    );
}

/// ADR 0089 Decision 8: Music opens on `Destination(Releases)`, the default
/// navigation entry of the content frame, and no child page is open.
#[test]
fn adr_0089_decision_8_music_opens_on_releases() {
    assert_eq!(
        super::default_navigation_entry(WorkspaceFrameKind::ContentList),
        FrameNavigationEntry::Destination(MusicDestination::Releases),
    );
    let layout = WorkspaceLayout::default_layout();
    let content_id = layout
        .frames()
        .iter()
        .find(|frame| frame.kind() == WorkspaceFrameKind::ContentList)
        .map(WorkspaceFrameState::id)
        .expect("the default layout has a content frame");
    let nav = layout
        .frame_nav(content_id)
        .expect("content frame navigation");
    assert_eq!(
        nav.current(),
        &FrameNavigationEntry::Destination(MusicDestination::Releases)
    );
    assert_eq!(
        nav.path_entries().len(),
        1,
        "no child page opens by default"
    );
}

/// ADR 0089 Decision 1: a saved layout that names the removed source-list
/// frame still loads, and that frame is dropped. The other frames keep their
/// order and focus.
#[test]
fn adr_0089_decision_1_saved_source_list_frame_loads_without_it() {
    let config: WorkspaceLayoutConfig = toml::from_str(
        r#"
focused_frame_id = 2

[[frames]]
id = 1
kind = "source_list"

[[frames]]
id = 2
kind = "content_list"

[[frames]]
id = 3
kind = "detail"
"#,
    )
    .expect("an old layout with the removed frame kind parses");

    let layout = WorkspaceLayout::from_config(Some(&config));

    let kinds: Vec<_> = layout
        .frames()
        .iter()
        .map(WorkspaceFrameState::kind)
        .collect();
    assert_eq!(
        kinds,
        [WorkspaceFrameKind::ContentList, WorkspaceFrameKind::Detail]
    );
    assert_eq!(layout.focused_frame_id(), Some(WorkspaceFrameId::new(2)));
}

/// R15-10 (ADR 0084 Decision 6): a singles group entry opens a page in the
/// detail column, and its fallback label is "Singles".
#[test]
fn adr_0084_singles_group_r15_10_group_entry_opens_a_detail_page() {
    use crate::view_models::artist_identity::ArtistIdentity;
    use crate::view_models::singles_group::SinglesGroupKey;

    let entry = FrameNavigationEntry::SinglesGroup(SinglesGroupKey(ArtistIdentity::Name(
        "haleen".to_string(),
    )));

    assert!(entry.opens_detail_page());
    assert_eq!(entry.display_label(), "Singles");
}

/// R30-09 (ADR 0084 task 030, decision D5): one pure function gives the
/// toolbar text of a restored entry. A search entry gives its query. Each
/// other entry gives `None`.
#[test]
fn adr_0084_music_open_defects_r30_09_restored_toolbar_text() {
    use super::nav::restored_toolbar_search_text;
    let search = FrameNavigationEntry::Search("haleen".to_owned());
    assert_eq!(restored_toolbar_search_text(&search), Some("haleen"));
    for entry in [
        discover(),
        track(1),
        FrameNavigationEntry::PlaylistDetail(1),
        FrameNavigationEntry::Destination(MusicDestination::Inbox),
    ] {
        assert_eq!(restored_toolbar_search_text(&entry), None, "{entry:?}");
    }
}

/// R30-10 (ADR 0084 task 030, decision D5). Back and Forward
/// restore an entry through one function. That function
/// records the query of a search entry. A render takes the text one time.
/// The guard `adr_0084_music_open_defects_r30_10_each_restore_records_the_toolbar_text`
/// proves that each restore path records it.
#[test]
fn adr_0084_music_open_defects_r30_10_pending_toolbar_text_is_taken_once() {
    use super::PendingToolbarSearchText;
    let mut nav = FrameNavigationState::new(FrameNavigationEntry::Search("haleen".to_owned()));
    nav.replace_active_search_or_push(FrameNavigationEntry::Search("survival guide".to_owned()));
    nav.push(track(1));
    let mut pending = PendingToolbarSearchText::default();

    // Back to the search page records its query.
    nav.go_back().unwrap();
    pending.record(nav.current());
    assert_eq!(pending.take().as_deref(), Some("survival guide"));
    assert_eq!(pending.take(), None, "a render takes the text one time");

    // Forward to the track page records nothing.
    nav.go_forward().unwrap();
    pending.record(nav.current());
    assert_eq!(pending.take(), None);

    // A history move to a search page restores it.
    let mut crumbs = FrameNavigationState::new(FrameNavigationEntry::Search("haleen".to_owned()));
    crumbs.push(track(2));
    crumbs.push(FrameNavigationEntry::Search("haleen".to_owned()));
    pending.record(crumbs.current());
    assert_eq!(pending.take().as_deref(), Some("haleen"));
}

/// Review of 2026-10-21, P5 point 4: `replace_current` keeps the page's
/// place in the history, but it is a different page, so the entry it
/// replaces must get a fresh memory. A later Back or Forward to it must
/// not show the new page at the old page's scroll offset.
#[test]
fn adr_0084_music_open_defects_review_replace_current_resets_detail_scroll_memory() {
    let mut nav = FrameNavigationState::new(FrameNavigationEntry::Search("haleen".to_owned()));
    nav.push(track(1));

    // Back to the search page, then the operator scrolls it: 400 is
    // stored for that history entry.
    nav.go_back().unwrap();
    let old_location = nav.detail_scroll_target().location;
    nav.record_detail_scroll(old_location, 400);
    assert_eq!(nav.detail_scroll_target().offset, 400);

    // The operator edits the query in place.
    nav.replace_active_search_or_push(FrameNavigationEntry::Search("survival guide".to_owned()));
    assert_eq!(
        nav.current(),
        &FrameNavigationEntry::Search("survival guide".to_owned())
    );

    // A move away and a move back to the replaced page must show it at
    // the top, never at the old query's offset.
    nav.push(track(2));
    nav.go_back().unwrap();
    let new_target = nav.detail_scroll_target();
    assert_ne!(
        new_target.location, old_location,
        "a replaced page must keep no memory of the page it replaced"
    );
    assert_eq!(
        new_target.offset, 0,
        "a replaced page must not inherit the old page's scroll offset"
    );
}

/// R30-15 (ADR 0084 task 030, decision D10): the keys `(None,
/// "urn:uuid:x")` and `(Some("urn"), "uuid:x")` give two different
/// targets. A label stored for one is not found for the other.
#[test]
fn adr_0084_music_open_defects_r30_15_typed_targets_do_not_collide() {
    use std::collections::BTreeMap;
    let bare = TrackKey::from_index_track_target(None, "urn:uuid:x").index_target();
    let split = TrackKey::from_index_track_target(Some("urn"), "uuid:x").index_target();
    assert_ne!(bare, split);
    // The text for a person is the same, so no map may key on it.
    assert_eq!(bare.to_string(), split.to_string());
    let mut labels = BTreeMap::new();
    labels.insert(bare.clone(), "Bare".to_owned());
    assert_eq!(labels.get(&bare).map(String::as_str), Some("Bare"));
    assert_eq!(labels.get(&split), None);
}

/// R87-3-12 (ADR 0087 Decision 4, packet 003 decision D5): the live runs of
/// `h`, `ha` and `hal` from the Discover page give one search entry. One
/// Back returns to the Discover page.
#[test]
fn adr_0087_live_results_r87_3_12_typed_text_adds_one_entry() {
    let mut layout = WorkspaceLayout::default_layout();
    let content_list_id = WorkspaceFrameId::new(2);
    layout
        .reset_nav(content_list_id, discover())
        .expect("should reset ContentList nav");

    for text in ["h", "ha", "hal"] {
        layout
            .open_search_results_in_content_list(text)
            .expect("should open the search page");
    }

    let nav = layout
        .frame_nav(content_list_id)
        .expect("ContentList should have navigation state");
    assert_eq!(
        nav.current(),
        &FrameNavigationEntry::Search("hal".to_owned())
    );
    assert_eq!(layout.pop_nav(content_list_id), Some(discover()));
}

/// R87-3-13 (ADR 0087 Decision 4, ADR 0084 Decision 9): live runs from a
/// release page under a search start one new path, and later live runs
/// replace it. A result page over the search keeps the search in the
/// stack.
#[test]
fn adr_0087_live_results_r87_3_13_live_run_from_a_result_page_starts_one_path() {
    let mut layout = WorkspaceLayout::default_layout();
    let content_list_id = WorkspaceFrameId::new(2);
    let release = FrameNavigationEntry::Release(ReleaseKey::FeedGuid("haleen-music".to_owned()));
    layout
        .open_search_results_in_content_list("hal")
        .expect("should open the search page");
    layout
        .push_nav(content_list_id, release.clone())
        .expect("should open the release page");

    for text in ["halx", "halxy"] {
        layout
            .open_search_results_in_content_list(text)
            .expect("should open the search page");
    }
    let search = FrameNavigationEntry::Search("halxy".to_owned());
    let nav = layout
        .frame_nav(content_list_id)
        .expect("ContentList should have navigation state");
    assert_eq!(nav.current(), &search);
    assert_eq!(nav.path_entries(), vec![search.clone()]);
    assert_eq!(layout.pop_nav(content_list_id), Some(release.clone()));

    layout
        .open_search_results_in_content_list("halxy")
        .expect("should open the search page");
    layout
        .push_nav(content_list_id, release)
        .expect("should open the release page");
    assert_eq!(layout.pop_nav(content_list_id), Some(search));
}

/// A live run changes the list root while the curator types, so the search
/// field keeps the focus. A page that opens takes the focus (ADR 0087
/// Decision 4, ADR 0089 packet 001).
#[test]
fn adr_0089_list_root_does_not_take_the_focus_from_the_search_field() {
    assert!(!music_focus_moves(MusicFocusTarget::SearchResults, true));
    assert!(!music_focus_moves(MusicFocusTarget::ListColumn, true));
    assert!(music_focus_moves(MusicFocusTarget::DetailColumn, true));
    assert!(music_focus_moves(MusicFocusTarget::SavedSearchPage, true));
    assert!(music_focus_moves(MusicFocusTarget::SearchResults, false));
}

/// R-NAV-2 items 2 and 4 (ADR 0089 Decision 9): Back to a list root marks
/// the page that was open over it. The next row click clears the mark.
#[test]
fn adr_0089_nav_back_marks_the_page_open_over_the_list_root() {
    let release = FrameNavigationEntry::Release(ReleaseKey::FeedGuid("guid-a".to_owned()));
    let mut nav = FrameNavigationState::new(FrameNavigationEntry::Destination(
        MusicDestination::Releases,
    ));
    nav.open_list_row(release.clone());
    assert_eq!(nav.marked_entry(), None, "a new open marks nothing");

    nav.go_back()
        .expect("a page opened from its list can go back");
    assert_eq!(
        nav.current(),
        &FrameNavigationEntry::Destination(MusicDestination::Releases)
    );
    assert_eq!(nav.marked_entry(), Some(&release));

    nav.open_list_row(FrameNavigationEntry::Release(ReleaseKey::FeedGuid(
        "guid-b".to_owned(),
    )));
    assert_eq!(
        nav.marked_entry(),
        None,
        "the next row click clears the mark"
    );
}

/// R-NAV-2 item 4 (ADR 0089 Decision 9): Forward has no subsequent page
/// after its only page is restored, and a Back move enables it.
#[test]
fn adr_0089_nav_forward_is_unavailable_without_a_subsequent_page() {
    let mut nav = FrameNavigationState::new(FrameNavigationEntry::Destination(
        MusicDestination::Releases,
    ));
    nav.open_list_row(track(3));
    assert!(!nav.can_go_forward(), "a new open has no subsequent page");

    nav.go_back().expect("the opened page can go back");
    assert!(nav.can_go_forward(), "Back enables Forward");

    nav.go_forward().expect("Forward restores the page");
    assert!(!nav.can_go_forward(), "the restored page is the last page");
    assert_eq!(nav.marked_entry(), None, "a Forward move marks nothing");
}

/// R-NAV-2 item 3 (ADR 0089 Decision 9): a path that opens an item starts at
/// the destination that holds it. With no earlier page, Back opens that
/// list and marks the item.
#[test]
fn adr_0089_nav_item_path_goes_back_to_its_holding_list() {
    let cases = [
        (
            MusicDestination::Releases,
            FrameNavigationEntry::Release(ReleaseKey::FeedGuid("guid-a".to_owned())),
        ),
        (MusicDestination::Inbox, track(7)),
        (
            MusicDestination::Playlists,
            FrameNavigationEntry::PlaylistDetail(4),
        ),
    ];
    for (destination, item) in cases {
        let mut nav = FrameNavigationState::new_with_page(
            FrameNavigationEntry::Destination(destination),
            item.clone(),
        );
        assert_eq!(
            nav.root(),
            &FrameNavigationEntry::Destination(destination),
            "the path starts at the holding list"
        );
        assert!(nav.can_go_back(), "Back is available with no earlier page");

        nav.go_back().expect("Back opens the holding list");
        assert_eq!(
            nav.current(),
            &FrameNavigationEntry::Destination(destination)
        );
        assert_eq!(nav.marked_entry(), Some(&item), "the item is marked");
    }
}

/// ADR 0089 Decision 1 and Decision 5: each navigation entry shows one
/// Music frame. A list root shows the list frame, and a loaded page shows the
/// detail frame. The entry and its content state decide this, not the width.
#[test]
fn adr_0089_one_page_stack_shows_one_frame_for_each_entry() {
    let list_roots = [
        FrameNavigationEntry::Destination(MusicDestination::Releases),
        FrameNavigationEntry::Destination(MusicDestination::Search),
        FrameNavigationEntry::Search("drums".to_string()),
    ];
    let pages = [
        FrameNavigationEntry::PlaylistDetail(1),
        FrameNavigationEntry::Release(ReleaseKey::FeedGuid("feed".to_string())),
        FrameNavigationEntry::Track(TrackKey::from_index_track_target(None, "1".to_string())),
    ];
    for entry in &list_roots {
        let columns = MusicColumns::for_entry(entry, MusicPageContent::Loaded);
        assert_eq!(columns, MusicColumns::ListRoot, "{entry:?}");
        assert!(columns.shows_list(), "{entry:?}");
        assert!(!columns.shows_detail(), "{entry:?}");
    }
    for entry in &pages {
        let columns = MusicColumns::for_entry(entry, MusicPageContent::Loaded);
        assert_eq!(columns, MusicColumns::Page, "{entry:?}");
        assert!(!columns.shows_list(), "{entry:?}");
        assert!(columns.shows_detail(), "{entry:?}");
    }
}

/// R12-05 (ADR 0084 Decision 3, packet 012): the `SavedSearch` entry gives
/// the label "Saved: bluegrass". It opens a page, and its frame title is
/// "Saved search". The toolbar title shows the label (ADR 0089 Decision 9).
#[test]
fn adr_0084_saved_search_r12_05_label_and_frame_title() {
    use crate::view_models::search_results::{SavedSearch, SavedSearchId};

    let entry = FrameNavigationEntry::SavedSearch {
        id: SavedSearchId(7),
        query: "bluegrass".to_owned(),
    };
    assert_eq!(entry.display_label(), "Saved: bluegrass");
    assert!(entry.opens_detail_page());

    let nav = FrameNavigationState::new_with_page(
        FrameNavigationEntry::Destination(MusicDestination::Search),
        entry.clone(),
    );
    assert_eq!(nav.active_saved_search(), Some(SavedSearchId(7)));
    assert_eq!(SavedSearch::FRAME_TITLE, "Saved search");
}
