//! The feed host of an RSS URL and the `Retry-After` text of HTTP 429
//! (ADR 0076 Decision 2, ADR 0084 Decision 10 as amended 2026-10-24).
//!
//! The playlist RSS check and the download queue share these functions.
//! This module holds no state and does no I/O.

use std::time::Duration;

/// HTTP 429 (Too Many Requests) from a feed host. `rss::fetch_feed_document`
/// returns it in the `anyhow` chain, and `DownloadError` and `CommandError`
/// keep its typed form (ADR 0084 Decision 10, task 039).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostThrottled {
    /// The lowercase host of the feed URL.
    pub host: String,
    /// The parsed `Retry-After` value, or `None` when the response has none.
    pub retry_after: Option<Duration>,
}

impl std::fmt::Display for HostThrottled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} sent HTTP 429 (Too Many Requests)", self.host)
    }
}

impl std::error::Error for HostThrottled {}

/// The lowercase host of an HTTP or HTTPS feed URL.
pub(crate) fn feed_host(feed_url: &str) -> Option<String> {
    let url = reqwest::Url::parse(feed_url.trim()).ok()?;
    if !matches!(url.scheme(), "http" | "https") {
        return None;
    }
    url.host_str()
        .filter(|host| !host.is_empty())
        .map(str::to_ascii_lowercase)
}

/// Reads a `Retry-After` text: a number of seconds, or an HTTP date.
/// A date in the past gives a zero wait.
pub(crate) fn parse_retry_after(text: &str) -> Option<Duration> {
    let text = text.trim();
    if let Ok(seconds) = text.parse::<u64>() {
        return Some(Duration::from_secs(seconds));
    }
    let at = chrono::DateTime::parse_from_rfc2822(text).ok()?;
    let wait = at.with_timezone(&chrono::Utc) - chrono::Utc::now();
    Some(wait.to_std().unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ADR 0084 Decision 10 as amended 2026-10-24, task 039 (R39-12). The
    /// parse reads seconds and an HTTP date, and `feed_host` lowercases the
    /// host. Failure: the download queue and the RSS check use one owner of
    /// the feed host and of `Retry-After` (task 039, step 2).
    #[test]
    fn adr_0084_task_039_host_and_retry_after_parse() {
        assert_eq!(parse_retry_after("30"), Some(Duration::from_secs(30)));
        assert_eq!(parse_retry_after(" 60 "), Some(Duration::from_secs(60)));
        let date = chrono::Utc::now() + chrono::Duration::seconds(120);
        let text = date.to_rfc2822();
        let wait = parse_retry_after(&text).expect("an HTTP date parses");
        assert!(wait.as_secs() >= 110 && wait.as_secs() <= 120, "{wait:?}");
        assert_eq!(parse_retry_after("soon"), None);
        assert_eq!(
            feed_host("https://WavLake.com/feed/music/x").as_deref(),
            Some("wavlake.com")
        );
        assert_eq!(feed_host("ftp://example.test/feed"), None);
        assert_eq!(feed_host(""), None);
    }

    /// ADR 0084 Decision 10 as amended 2026-10-24, task 039 (R39-11 support):
    /// the typed throttle names its host and keeps its wait.
    #[test]
    fn adr_0084_task_039_host_throttled_names_host_and_keeps_wait() {
        let error = HostThrottled {
            host: "wavlake.com".to_owned(),
            retry_after: Some(Duration::from_secs(30)),
        };
        assert_eq!(
            error.to_string(),
            "wavlake.com sent HTTP 429 (Too Many Requests)"
        );
        assert_eq!(error.retry_after, Some(Duration::from_secs(30)));
    }
}
