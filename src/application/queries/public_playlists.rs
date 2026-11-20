//! The Public playlists of the Index (ADR 0089 Decision 1, Decision 5,
//! packet 009).
//!
//! `FetchPublicPlaylistPage` reads one page of the musicL feeds: the sort
//! Newest uses `GET /v1/feeds/recent`, and the sort A to Z uses
//! `GET /v1/feeds/by-name` with the `from` text of a letter. Both send
//! `medium=musicL`. `FetchPublicPlaylist` reads one playlist with its
//! `remote_items` (ADR 0089 packet 009, the item page). `MusicIndex`
//! responses are untrusted input, so each one decodes through the API
//! client and no value is trusted.

#![warn(clippy::pedantic)]

use crate::api::{Client, Feed, FeedMedium};
use crate::application::command_bus::{ApplicationCommand, CommandOutcome, CommandResult};
use crate::application::command_context::CommandContext;
use crate::application::errors::command::CommandError;
use crate::application::request_profiles::{
    INDEX_PLAYLIST_PUBLISHERS, INDEX_PUBLIC_PLAYLIST_DETAIL, INDEX_PUBLIC_PLAYLIST_LIST,
};
use crate::view_models::index_name_list::IndexNameListBatch;
use crate::view_models::list_page::ListSort;

/// The row count of one page of the Public playlists list.
pub(crate) const PUBLIC_PLAYLIST_PAGE_LIMIT: i32 = 50;

/// Fetches one page of the public playlists in the sort of the list.
#[derive(Clone, Debug)]
pub(crate) struct FetchPublicPlaylistPage {
    endpoint: crate::config::MusicIndexEndpoint,
    sort: ListSort,
    cursor: Option<String>,
    /// The `from` text of the letter, or `None` for the first row. The
    /// sort Newest gives no `from`.
    from: Option<String>,
    /// Whether the request sends `reverse=true` (musicindex ADR 0053
    /// R-LIST-8 item 4).
    reverse: bool,
}

impl FetchPublicPlaylistPage {
    /// Creates the command for the page at `cursor` of the list in `sort`.
    /// `from` starts the sort A to Z at a letter. `reverse` gives the
    /// reversed order of the sort.
    #[must_use]
    pub(crate) fn new(
        endpoint: impl Into<crate::config::MusicIndexEndpoint>,
        sort: ListSort,
        cursor: Option<String>,
        from: Option<String>,
        reverse: bool,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            sort,
            cursor,
            from,
            reverse,
        }
    }
}

impl ApplicationCommand for FetchPublicPlaylistPage {
    type Output = IndexNameListBatch<Feed>;

    fn execute(self, context: &CommandContext) -> CommandResult<Self::Output> {
        if context.cancellation().is_cancelled() {
            return Err(CommandError::Cancelled);
        }
        let client = Client::new_with_base_url(self.endpoint);
        let batch = match self.sort {
            // Public playlists do not offer Relevance, Track Order, Playlist
            // Order or Curator Order, so each reads as Newest.
            ListSort::Newest
            | ListSort::Relevance
            | ListSort::TrackOrder
            | ListSort::PlaylistOrder
            | ListSort::CuratorOrder => {
                let response = client
                    .fetch_recent_feeds(
                        Some(PUBLIC_PLAYLIST_PAGE_LIMIT),
                        self.cursor.as_deref(),
                        FeedMedium::MusicL,
                        self.reverse,
                    )
                    .map_err(|error| CommandError::Query(format!("{error:#}")))?;
                IndexNameListBatch {
                    rows: response.data,
                    cursor: response.pagination.cursor,
                    has_more: response.pagination.has_more,
                }
            }
            ListSort::AToZ => {
                let response = client
                    .fetch_feeds_by_name(
                        PUBLIC_PLAYLIST_PAGE_LIMIT,
                        self.cursor.as_deref(),
                        None,
                        self.from.as_deref(),
                        FeedMedium::MusicL,
                        &INDEX_PUBLIC_PLAYLIST_LIST,
                        self.reverse,
                    )
                    .map_err(|error| CommandError::Query(format!("{error:#}")))?;
                IndexNameListBatch {
                    rows: response.data,
                    cursor: response.pagination.cursor,
                    has_more: response.pagination.has_more,
                }
            }
        };
        Ok(CommandOutcome::without_events(batch))
    }
}

/// Fetches at most five pages of playlist publisher feeds.
#[derive(Clone, Debug)]
pub(crate) struct FetchPlaylistPublishers {
    endpoint: crate::config::MusicIndexEndpoint,
}

impl FetchPlaylistPublishers {
    /// Creates the command for the given Index endpoint.
    #[must_use]
    pub(crate) fn new(endpoint: impl Into<crate::config::MusicIndexEndpoint>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }
}

impl ApplicationCommand for FetchPlaylistPublishers {
    type Output = Vec<Feed>;

    fn execute(self, context: &CommandContext) -> CommandResult<Self::Output> {
        let client = Client::new_with_base_url(self.endpoint);
        let mut rows = Vec::new();
        let mut cursor = None;
        for _ in 0..5 {
            if context.cancellation().is_cancelled() {
                return Err(CommandError::Cancelled);
            }
            let response = client
                .fetch_playlist_publishers(200, cursor.as_deref(), &INDEX_PLAYLIST_PUBLISHERS)
                .map_err(|error| CommandError::Query(format!("{error:#}")))?;
            rows.extend(response.data);
            if !response.pagination.has_more {
                break;
            }
            cursor = response.pagination.cursor;
            if cursor.is_none() {
                break;
            }
        }
        Ok(CommandOutcome::without_events(rows))
    }
}

/// The answer of [`FetchPublicPlaylist`]: the feed GUID that was asked for,
/// and the feed with its remote items.
#[derive(Clone, Debug)]
pub(crate) struct PublicPlaylistLoad {
    pub(crate) feed_guid: String,
    /// The feed, or `None` when the index does not hold it: Music Index
    /// replied with HTTP `404` (musicindex ADR 0081 task 003).
    pub(crate) feed: Option<Feed>,
}

/// Reads one public playlist with its remote items (ADR 0089 packet 009).
#[derive(Clone, Debug)]
pub(crate) struct FetchPublicPlaylist {
    endpoint: crate::config::MusicIndexEndpoint,
    feed_guid: String,
}

impl FetchPublicPlaylist {
    /// Creates the command for the playlist `feed_guid`.
    #[must_use]
    pub(crate) fn new(
        endpoint: impl Into<crate::config::MusicIndexEndpoint>,
        feed_guid: impl Into<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            feed_guid: feed_guid.into(),
        }
    }
}

impl ApplicationCommand for FetchPublicPlaylist {
    type Output = PublicPlaylistLoad;

    fn execute(self, context: &CommandContext) -> CommandResult<Self::Output> {
        if context.cancellation().is_cancelled() {
            return Err(CommandError::Cancelled);
        }
        let client = Client::new_with_base_url(self.endpoint);
        let feed =
            match client.fetch_feed_with_profile(&self.feed_guid, &INDEX_PUBLIC_PLAYLIST_DETAIL) {
                Ok(feed) => Some(feed),
                Err(error) if is_not_found(&error) => None,
                Err(error) => return Err(CommandError::Query(format!("{error:#}"))),
            };
        Ok(CommandOutcome::without_events(PublicPlaylistLoad {
            feed_guid: self.feed_guid,
            feed,
        }))
    }
}

/// Tells if Music Index replied with HTTP `404`: the index does not hold
/// the feed. Each other error is a failed read.
fn is_not_found(error: &anyhow::Error) -> bool {
    error
        .downcast_ref::<crate::api::HttpStatusError>()
        .is_some_and(crate::api::HttpStatusError::is_not_found)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::queries::name_lists::tests::serve_routes;

    /// Musicindex ADR 0081 task 003: a `404` "feed not found" reply gives
    /// a load with no feed, not a command error.
    #[test]
    fn musicindex_adr_0081_task_003_feed_not_found_gives_no_feed() {
        let (endpoint, _, _) = serve_routes(|_| r#"404 {"error":"feed not found"}"#.to_owned(), 1);

        let load = FetchPublicPlaylist::new(endpoint, "missing")
            .execute(&CommandContext::next())
            .expect("a 404 reply is not a command error")
            .into_parts()
            .0;

        assert_eq!(load.feed_guid, "missing");
        assert!(load.feed.is_none());
    }

    /// Musicindex ADR 0081 task 003: a `500` reply stays a command error,
    /// so the page keeps the error state with "Try again".
    #[test]
    fn musicindex_adr_0081_task_003_server_error_stays_an_error() {
        let (endpoint, _, _) = serve_routes(|_| String::new(), 1);

        let result = FetchPublicPlaylist::new(endpoint, "any").execute(&CommandContext::next());

        assert!(matches!(result, Err(CommandError::Query(_))));
    }

    /// Musicindex ADR 0081 R-CURATOR-5 and main session choice 2 of task
    /// 002: the read of `/v1/feeds/playlist-publishers` follows the cursor,
    /// with 200 rows on each page, and stops after 5 pages.
    #[test]
    fn musicindex_adr_0081_playlist_publishers_follow_the_cursor_up_to_five_pages() {
        fn answer(path: &str) -> String {
            let page = path
                .split("cursor=")
                .nth(1)
                .and_then(|rest| rest.split('&').next())
                .map_or(0, |cursor| cursor.parse::<usize>().unwrap());
            serde_json::json!({
                "data": [{"feed_guid": format!("p{page}"), "agreed_roles": ["label"]}],
                "pagination": {"cursor": (page + 1).to_string(), "has_more": true}
            })
            .to_string()
        }
        let (endpoint, requests, _) = serve_routes(answer, 5);

        let rows = FetchPlaylistPublishers::new(endpoint)
            .execute(&CommandContext::next())
            .unwrap()
            .into_parts()
            .0;

        assert_eq!(rows.len(), 5);
        let requests = requests.lock().unwrap().clone();
        assert_eq!(requests.len(), 5, "at most 5 pages");
        assert_eq!(requests[0], "/v1/feeds/playlist-publishers?limit=200");
        assert_eq!(
            requests[1],
            "/v1/feeds/playlist-publishers?limit=200&cursor=1"
        );
    }

    /// The read stops at the page with `has_more` false.
    #[test]
    fn musicindex_adr_0081_playlist_publishers_stop_at_the_last_page() {
        fn answer(_path: &str) -> String {
            serde_json::json!({
                "data": [{"feed_guid": "chadf"}],
                "pagination": {"cursor": null, "has_more": false}
            })
            .to_string()
        }
        let (endpoint, requests, _) = serve_routes(answer, 1);

        let rows = FetchPlaylistPublishers::new(endpoint)
            .execute(&CommandContext::next())
            .unwrap()
            .into_parts()
            .0;

        assert_eq!(rows.len(), 1);
        assert_eq!(requests.lock().unwrap().len(), 1);
    }
}
