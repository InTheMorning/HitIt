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
    INDEX_PUBLIC_PLAYLIST_DETAIL, INDEX_PUBLIC_PLAYLIST_LIST,
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
            // Public playlists do not offer Relevance, so it reads as Newest.
            ListSort::Newest | ListSort::Relevance => {
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

/// The answer of [`FetchPublicPlaylist`]: the feed GUID that was asked for,
/// and the feed with its remote items.
#[derive(Clone, Debug)]
pub(crate) struct PublicPlaylistLoad {
    pub(crate) feed_guid: String,
    pub(crate) feed: Feed,
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
        let feed = client
            .fetch_feed_with_profile(&self.feed_guid, &INDEX_PUBLIC_PLAYLIST_DETAIL)
            .map_err(|error| CommandError::Query(format!("{error:#}")))?;
        Ok(CommandOutcome::without_events(PublicPlaylistLoad {
            feed_guid: self.feed_guid,
            feed,
        }))
    }
}
