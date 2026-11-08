//! The sort capability of the Index node (musicindex ADR 0053 R-LIST-8 item
//! 4, ADR 0043 R-LIST-7 item 10).
//!
//! `FetchNodeSorts` reads `GET /v1/node/capabilities` once. The answer tells
//! whether the node lists `sort`. A node that lists it takes `reverse` on
//! the Index lists, so their reverse button shows.

#![warn(clippy::pedantic)]

use crate::api::Client;
use crate::application::command_bus::{ApplicationCommand, CommandOutcome, CommandResult};
use crate::application::command_context::CommandContext;
use crate::application::errors::command::CommandError;
use crate::view_models::list_page::NodeSorts;

/// Reads whether the Index node lists `sort` in its capabilities.
#[derive(Clone, Debug)]
pub(crate) struct FetchNodeSorts {
    endpoint: crate::config::MusicIndexEndpoint,
}

impl FetchNodeSorts {
    /// Creates the command for the node at `endpoint`.
    #[must_use]
    pub(crate) fn new(endpoint: impl Into<crate::config::MusicIndexEndpoint>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }
}

impl ApplicationCommand for FetchNodeSorts {
    type Output = NodeSorts;

    fn execute(self, context: &CommandContext) -> CommandResult<Self::Output> {
        if context.cancellation().is_cancelled() {
            return Err(CommandError::Cancelled);
        }
        let capabilities = Client::new_with_base_url(self.endpoint)
            .node_capabilities()
            .map_err(|error| CommandError::Query(format!("{error:#}")))?;
        Ok(CommandOutcome::without_events(
            NodeSorts::from_capabilities(&capabilities.capabilities),
        ))
    }
}
