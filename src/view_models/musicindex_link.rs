//! Page addresses for musicindex.org (`HitIt` ADR 0101 task 009; musicindex ADR 0085 R-LINK-1).

#![warn(clippy::pedantic)]

/// Build a musicindex.org fragment address for a feed, track, or artist name.
#[must_use]
pub(crate) fn page_url(target: MusicIndexTarget<'_>) -> String {
    match target {
        MusicIndexTarget::Feed(guid) => format!("https://musicindex.org/#feed={}", encode(guid)),
        MusicIndexTarget::Track { guid, feed_guid } => format!(
            "https://musicindex.org/#track={}&in={}",
            encode(guid),
            encode(feed_guid)
        ),
        MusicIndexTarget::Name(key) => {
            format!("https://musicindex.org/#name={}", encode(key))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MusicIndexTarget<'a> {
    Feed(&'a str),
    Track { guid: &'a str, feed_guid: &'a str },
    Name(&'a str),
}

fn encode(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            result.push(char::from(byte));
        } else {
            use std::fmt::Write as _;
            let _ = write!(result, "%{byte:02X}");
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_encoded_feed_address() {
        assert_eq!(
            page_url(MusicIndexTarget::Feed("feed guid")),
            "https://musicindex.org/#feed=feed%20guid"
        );
    }

    #[test]
    fn builds_track_address_with_feed() {
        assert_eq!(
            page_url(MusicIndexTarget::Track {
                guid: "track guid",
                feed_guid: "feed guid"
            }),
            "https://musicindex.org/#track=track%20guid&in=feed%20guid"
        );
    }

    #[test]
    fn builds_name_address() {
        assert_eq!(
            page_url(MusicIndexTarget::Name("The & Artist")),
            "https://musicindex.org/#name=The%20%26%20Artist"
        );
    }
}
