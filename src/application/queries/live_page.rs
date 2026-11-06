//! The Live list of the Index (ADR 0089 Decision 1, packet 011).
//!
//! `FetchLivePage` reads one page of the live items that are live now:
//! `GET /v1/live-items` with `view=now` (The Contract). The Index gives no
//! upcoming item in this view. `MusicIndex` responses are untrusted input,
//! so each one decodes through the API client and no value is trusted.

#![warn(clippy::pedantic)]

use crate::api::{Client, LiveItem};
use crate::application::command_bus::{ApplicationCommand, CommandOutcome, CommandResult};
use crate::application::command_context::CommandContext;
use crate::application::errors::command::CommandError;
use crate::view_models::index_name_list::IndexNameListBatch;

/// The row count of one page of the Live list.
pub(crate) const LIVE_PAGE_LIMIT: i32 = 50;

/// Fetches one page of the live items that are live now.
#[derive(Clone, Debug)]
pub(crate) struct FetchLivePage {
    endpoint: crate::config::MusicIndexEndpoint,
    cursor: Option<String>,
}

impl FetchLivePage {
    /// Creates the command for the page at `cursor`, or the first page.
    #[must_use]
    pub(crate) fn new(
        endpoint: impl Into<crate::config::MusicIndexEndpoint>,
        cursor: Option<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            cursor,
        }
    }
}

impl ApplicationCommand for FetchLivePage {
    type Output = IndexNameListBatch<LiveItem>;

    fn execute(self, context: &CommandContext) -> CommandResult<Self::Output> {
        if context.cancellation().is_cancelled() {
            return Err(CommandError::Cancelled);
        }
        let client = Client::new_with_base_url(self.endpoint);
        let response = client
            .fetch_live_items(LIVE_PAGE_LIMIT, self.cursor.as_deref())
            .map_err(|error| CommandError::Query(format!("{error:#}")))?;
        Ok(CommandOutcome::without_events(IndexNameListBatch {
            rows: response.data,
            cursor: response.pagination.cursor,
            has_more: response.pagination.has_more,
        }))
    }
}
