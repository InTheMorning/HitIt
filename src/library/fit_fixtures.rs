//! ADR 0097 task 002: test rows for each Library destination.

use crate::api::{ArtistNameKind, ArtistNameRow, Feed, LiveItem};
use crate::application::queries::broadcast::{
    BroadcastReadinessReport, BroadcastReadinessState, BroadcastReadinessSummary,
    BroadcastReadinessTrack,
};
use crate::db::{Playlist, TrackRow};
use crate::view_models::artist_groups::ArtistGroupPages;
use crate::view_models::index_name_list::IndexNameListBatch;
use crate::view_models::library::{AlbumNode, ArtistNode, LibraryTree};
use crate::view_models::list_page::{ListPageKey, ListScope, ListView};
use crate::view_models::recent_feeds::RecentFeedsPageBatch;
use crate::view_models::search_results::{
    FeedResultDisplay, IndexSearchResultRows, SearchResultItemId, SearchResultOrigin,
};
use crate::view_models::workspace::MusicDestination;
use crate::views::{FeedMetadataFacts, LocalIdentityFacts};

use super::LibraryApp;

const LONG_TITLE: &str = "Supercalifragilisticexpialidociousness40";

impl LibraryApp {
    /// Loads long fixture rows without a command or a network request.
    pub(crate) fn load_fit_fixtures(&mut self) {
        let tracks = (1..=3)
            .map(|id| TrackRow {
                id,
                feed_id: id,
                feed_guid: Some(format!("fit-feed-{id}")),
                item_guid: format!("fit-track-{id}"),
                track_title: Some(LONG_TITLE.to_owned()),
                artist_name: Some(LONG_TITLE.to_owned()),
                album_title: Some(LONG_TITLE.to_owned()),
                is_in_library: true,
                ..TrackRow::default()
            })
            .collect::<Vec<_>>();
        self.vm.replace_tree(LibraryTree {
            artists: tracks
                .iter()
                .map(|track| ArtistNode {
                    name: LONG_TITLE.to_owned(),
                    albums: vec![AlbumNode {
                        name: LONG_TITLE.to_owned(),
                        feed_id: Some(track.feed_id),
                        feed_guid: Some(format!("fit-feed-{}", track.feed_id)),
                        feed_url: None,
                        language: None,
                        description: None,
                        image_href: None,
                        named_feeds: Vec::new(),
                        identity_facts: LocalIdentityFacts::default(),
                        metadata_facts: Box::new(FeedMetadataFacts::default()),
                        stored_values: None,
                        tracks: vec![track.clone()],
                    }],
                })
                .collect(),
        });
        self.vm.replace_playlists(
            (1..=3)
                .map(|id| Playlist {
                    id,
                    name: LONG_TITLE.to_owned(),
                    description: Some(LONG_TITLE.to_owned()),
                    track_count: 14,
                    created_at: id,
                    updated_at: id,
                    position: id,
                })
                .collect(),
        );

        let index_feeds = (1..=3).map(index_feed).collect::<Vec<_>>();
        let recent_rows = index_feeds
            .iter()
            .enumerate()
            .map(|(index, feed)| {
                let result = FeedResultDisplay::new(
                    format!("fit-feed-{}", index + 1),
                    feed.title.clone().unwrap_or_default(),
                    SearchResultOrigin::Index,
                )
                .with_secondary_parts(vec![
                    LONG_TITLE.to_owned(),
                    "14 albums · 38 singles".to_owned(),
                ]);
                (
                    SearchResultItemId::try_from(index + 1).expect("small fixture id"),
                    result,
                )
            })
            .collect();
        self.recent_music_page.load_recorded_page(
            RecentFeedsPageBatch {
                rows: recent_rows,
                cursor: None,
                has_more: false,
            },
            false,
        );

        let intent = self
            .release_name_page
            .begin_load(false)
            .expect("first page starts");
        self.release_name_page.finish_load(
            IndexNameListBatch {
                rows: index_feeds,
                cursor: None,
                has_more: false,
            },
            intent.generation,
        );
        let artist_rows = (1..=3)
            .map(|id| ArtistNameRow {
                kind: ArtistNameKind::Publisher,
                name: LONG_TITLE.to_owned(),
                sort_name: LONG_TITLE.to_owned(),
                key: format!("fit-artist-{id}"),
                feed_guid: Some(format!("fit-artist-{id}")),
                image_url: None,
                release_count: Some(52),
                agreed_roles: Some(vec!["artist".to_owned()]),
                confirmed_release_artists: None,
                first_named_count: None,
                stated_rels: Some(vec!["artist".to_owned()]),
                two_way_link_count: Some(52),
                release_track_counts: Some(std::collections::BTreeMap::from([
                    ("1".to_owned(), 14),
                    ("2".to_owned(), 38),
                ])),
                newest_release_at: Some(1_700_000_000),
            })
            .collect::<Vec<_>>();
        self.artist_pages = ArtistGroupPages::loaded(&artist_rows, false);

        let public_intent = self
            .public_playlist_list
            .begin_load(false)
            .expect("page starts");
        self.public_playlist_list.finish_load(
            IndexNameListBatch {
                rows: (1..=3).map(index_feed).collect(),
                cursor: None,
                has_more: false,
            },
            public_intent.generation,
        );
        let live_intent = self.live_list.begin_load(false).expect("page starts");
        self.live_list.finish_load(
            IndexNameListBatch {
                rows: (1..=3)
                    .map(|id| LiveItem {
                        feed_guid: Some(format!("fit-live-{id}")),
                        live_item_guid: Some(format!("fit-live-item-{id}")),
                        title: Some(LONG_TITLE.to_owned()),
                        feed_title: Some(LONG_TITLE.to_owned()),
                        ..LiveItem::default()
                    })
                    .collect(),
                cursor: None,
                has_more: false,
            },
            live_intent.generation,
        );

        {
            let conn = self.conn.lock().expect("fit fixture database lock");
            for index in 1..=3 {
                let query = if index == 1 {
                    LONG_TITLE.to_owned()
                } else {
                    format!("{}{index}", &LONG_TITLE[..LONG_TITLE.len() - 1])
                };
                crate::db::saved_searches::insert_saved_search(
                    &conn,
                    &query,
                    crate::db::saved_searches::StoredStartTab::Releases,
                    i64::from(index),
                )
                .expect("insert fit fixture search");
            }
        }
        self.reload_saved_searches();

        for (page, rows) in [
            (ListPageKey::Releases, ListView::Rows),
            (ListPageKey::Artists, ListView::Rows),
        ] {
            self.vm.set_list_view(page, rows);
        }
        self.vm
            .set_list_scope(ListPageKey::Releases, ListScope::All);
        self.vm.set_list_scope(ListPageKey::Artists, ListScope::All);

        let report = BroadcastReadinessReport {
            summary: BroadcastReadinessSummary {
                no_route_tag: 3,
                ..Default::default()
            },
            tracks: (1..=3)
                .map(|id| BroadcastReadinessTrack {
                    track_id: id,
                    title: LONG_TITLE.to_owned(),
                    artist: Some(LONG_TITLE.to_owned()),
                    album: Some(LONG_TITLE.to_owned()),
                    path: None,
                    state: BroadcastReadinessState::NoRouteTag,
                    reason: "Fixture row".to_owned(),
                })
                .collect(),
        };
        self.vm.show_inbox_report(&report, &[]);
    }

    /// Sets the destination scope and view for one fixture page.
    pub(crate) fn configure_fit_list(
        &mut self,
        destination: MusicDestination,
        scope: Option<ListScope>,
        view: Option<ListView>,
    ) {
        let key = match destination {
            MusicDestination::Releases => Some(ListPageKey::Releases),
            MusicDestination::Artists => Some(ListPageKey::Artists),
            MusicDestination::Search
            | MusicDestination::Inbox
            | MusicDestination::Playlists
            | MusicDestination::PublicPlaylists
            | MusicDestination::Live => None,
        };
        if let Some(key) = key {
            if let Some(scope) = scope {
                self.vm.set_list_scope(key, scope);
            }
            if let Some(view) = view {
                self.vm.set_list_view(key, view);
            }
        }
    }
}

/// Builds long Index rows for the Search destination fit check.
pub(crate) fn index_search_results() -> IndexSearchResultRows {
    IndexSearchResultRows {
        feeds: (1..=3)
            .map(|id| {
                let result = FeedResultDisplay::new(
                    format!("fit-search-feed-{id}"),
                    LONG_TITLE,
                    SearchResultOrigin::Index,
                )
                .with_secondary_parts(vec![
                    LONG_TITLE.to_owned(),
                    "14 albums · 38 singles".to_owned(),
                ]);
                (
                    SearchResultItemId::try_from(id).expect("small fixture id"),
                    result,
                )
            })
            .collect(),
        ..IndexSearchResultRows::default()
    }
}

fn index_feed(id: i64) -> Feed {
    Feed {
        feed_guid: Some(format!("fit-feed-{id}")),
        title: Some(LONG_TITLE.to_owned()),
        release_artist: Some(LONG_TITLE.to_owned()),
        release_kind: Some("album".to_owned()),
        episode_count: Some(14),
        ..Feed::default()
    }
}
