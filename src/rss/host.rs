//! The feed host of an RSS URL and the `Retry-After` text of HTTP 429
//! (ADR 0076 Decision 2, ADR 0084 Decision 10 as amended 2026-10-24).
//!
//! The playlist RSS check, the download queue and the playlist copy use
//! these items. `musicindex-fetch` owns them (musicindex ADR 0092
//! R-STATION-1 item 5). This module only gives them their HitIt path.
//!
//! `HostThrottled` is HTTP 429 from a feed host. `rss::fetch_feed_document`
//! gives it in the `anyhow` chain, and `DownloadError` and `CommandError`
//! keep its typed form (ADR 0084 Decision 10, task 039).

pub use musicindex_fetch::{feed_host, parse_retry_after, HostThrottled};
