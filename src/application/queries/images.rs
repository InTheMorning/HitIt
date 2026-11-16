//! Presentation image query family.

use std::fmt;
use std::sync::Arc;

use crate::application::command_bus::{ApplicationCommand, CommandOutcome, CommandResult};
use crate::application::command_context::CommandContext;
use crate::application::errors::command::CommandError;
use crate::media::header_glow::{GlowImage, GlowRequest};
use crate::media::logo_frame::{BakedShadow, LogoMosaicKey};
use crate::media::{CachedImage, ImageCache};

/// Fetches one cached thumbnail for presentation.
#[derive(Clone)]
pub(crate) struct FetchThumbnail {
    cache: Arc<ImageCache>,
    url: String,
    animated: bool,
}

impl FetchThumbnail {
    /// Creates a thumbnail fetch query command.
    #[must_use]
    pub(crate) fn new(cache: Arc<ImageCache>, url: impl Into<String>, animated: bool) -> Self {
        Self {
            cache,
            url: url.into(),
            animated,
        }
    }
}

impl fmt::Debug for FetchThumbnail {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FetchThumbnail")
            .field("url", &self.url)
            .field("animated", &self.animated)
            .finish_non_exhaustive()
    }
}

impl ApplicationCommand for FetchThumbnail {
    type Output = Option<CachedImage>;

    fn execute(self, context: &CommandContext) -> CommandResult<Self::Output> {
        if context.cancellation().is_cancelled() {
            return Err(CommandError::Cancelled);
        }
        let image = if self.animated {
            self.cache.fetch_blocking(&self.url)
        } else {
            self.cache.fetch_static_blocking(&self.url)
        };
        Ok(CommandOutcome::without_events(image))
    }
}

/// Composes the logo mosaic of one collection (ADR 0084 Decision 12).
///
/// The image cache reads the covers and composes them off the UI thread.
/// The UI thread only peeks at the result.
#[derive(Clone)]
pub(crate) struct FetchLogoMosaic {
    cache: Arc<ImageCache>,
    key: LogoMosaicKey,
    shadow: Option<BakedShadow>,
}

impl FetchLogoMosaic {
    /// Creates a logo mosaic query command. `shadow` gives the baked
    /// shadow values when `key.shadow` is true.
    #[must_use]
    pub(crate) fn new(
        cache: Arc<ImageCache>,
        key: LogoMosaicKey,
        shadow: Option<BakedShadow>,
    ) -> Self {
        Self { cache, key, shadow }
    }
}

impl fmt::Debug for FetchLogoMosaic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FetchLogoMosaic")
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}

impl ApplicationCommand for FetchLogoMosaic {
    type Output = Option<CachedImage>;

    fn execute(self, context: &CommandContext) -> CommandResult<Self::Output> {
        if context.cancellation().is_cancelled() {
            return Err(CommandError::Cancelled);
        }
        Ok(CommandOutcome::without_events(
            self.cache
                .compose_logo_mosaic_blocking(&self.key, self.shadow),
        ))
    }
}

/// Prepares the glow of one page header (ADR 0101 Decision 4, task 002).
///
/// The image cache reads the candidate covers, blurs them and adds the
/// grain off the UI thread. The UI thread only peeks at the result.
#[derive(Clone)]
pub(crate) struct FetchHeaderGlow {
    cache: Arc<ImageCache>,
    request: GlowRequest,
}

impl FetchHeaderGlow {
    /// Creates a header glow query command.
    #[must_use]
    pub(crate) fn new(cache: Arc<ImageCache>, request: GlowRequest) -> Self {
        Self { cache, request }
    }
}

impl fmt::Debug for FetchHeaderGlow {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FetchHeaderGlow")
            .field("urls", &self.request.urls)
            .finish_non_exhaustive()
    }
}

impl ApplicationCommand for FetchHeaderGlow {
    type Output = Arc<GlowImage>;

    fn execute(self, context: &CommandContext) -> CommandResult<Self::Output> {
        if context.cancellation().is_cancelled() {
            return Err(CommandError::Cancelled);
        }
        Ok(CommandOutcome::without_events(
            self.cache.prepare_header_glow_blocking(&self.request),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::header_glow::GlowStyle;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    use crate::application::command_bus::CommandBus;
    use crate::application::command_context::{CancellationToken, OperationId, TraceId};

    const TEST_IMAGE_BYTES: &[u8] = include_bytes!("../../assets/music_network_logo.png");

    fn cancelled_context() -> CommandContext {
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        CommandContext::new(OperationId::new(1), cancellation, TraceId::new(1))
    }

    fn serve_image_once(content_type: &'static str, body: &'static [u8]) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
        let addr = listener.local_addr().expect("listener addr");
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept request");
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request);
            let headers = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(headers.as_bytes()).expect("write headers");
            stream.write_all(body).expect("write body");
        });
        format!("http://{addr}/cover.png")
    }

    #[test]
    fn fetch_thumbnail_fetches_image_through_cache() {
        let temp = tempfile::tempdir().expect("tempdir");
        let cache = ImageCache::with_capacity(temp.path().join("thumbnails"), 2, 512, 1024 * 1024);
        let url = serve_image_once("image/png", TEST_IMAGE_BYTES);

        let outcome = CommandBus::new()
            .execute(
                FetchThumbnail::new(Arc::clone(&cache), url.clone(), false),
                &CommandContext::next(),
            )
            .expect("thumbnail fetch succeeds");

        assert!(outcome.value().is_some(), "thumbnail should be fetched");
        assert!(outcome.events().is_empty(), "query should not emit events");
        assert!(
            cache.peek_static(&url).is_some(),
            "thumbnail should be retained by the cache"
        );
    }

    /// A literal style, because the application layer does not import `ui`.
    fn glow_request(urls: Vec<String>, fallback_urls: Vec<String>) -> GlowRequest {
        GlowRequest {
            urls,
            fallback_urls,
            entity: [0x87, 0x5c, 0xcf],
            style: GlowStyle {
                background: [0x0b, 0x0b, 0x0d],
                label: [0xf5, 0xf5, 0xf7],
                secondary_label: [0xa1, 0xa1, 0xa6],
                veil_top: 0.25,
                grain_opacity: 0.22,
                max_opacity: 0.85,
                blur: 90.0,
                saturation: 1.7,
                entity_tint: 0.45,
                muted_tint: 0.40,
                muted_art: 0.35,
                min_colorfulness: 18.0,
                profile: 1,
                increase_contrast: false,
            },
        }
    }

    /// Task 002 correction 6: a candidate that does not read gives an
    /// incomplete glow, and the cache does not keep it. When the own image
    /// does not read, the glow uses the fallback candidates.
    #[test]
    fn fetch_header_glow_does_not_keep_a_glow_with_a_failed_read() {
        let temp = tempfile::tempdir().expect("tempdir");
        let cache = ImageCache::with_capacity(temp.path().join("thumbnails"), 2, 512, 1024 * 1024);
        let fallback = serve_image_once("image/png", TEST_IMAGE_BYTES);
        let request = glow_request(
            vec!["http://127.0.0.1:1/own.png".to_owned()],
            vec![fallback],
        );

        let outcome = CommandBus::new()
            .execute(
                FetchHeaderGlow::new(Arc::clone(&cache), request.clone()),
                &CommandContext::next(),
            )
            .expect("header glow fetch succeeds");

        assert!(!outcome.value().is_complete(), "a read failed");
        assert!(
            outcome.value().color().is_some(),
            "the glow comes from the fallback cover"
        );
        assert!(
            cache.peek_header_glow(&request.key()).is_none(),
            "the cache keeps no glow with a failed read"
        );
    }

    #[test]
    fn fetch_header_glow_prepares_and_keeps_the_glow() {
        let temp = tempfile::tempdir().expect("tempdir");
        let cache = ImageCache::with_capacity(temp.path().join("thumbnails"), 2, 512, 1024 * 1024);
        let url = serve_image_once("image/png", TEST_IMAGE_BYTES);
        let request = glow_request(vec![url], Vec::new());

        let outcome = CommandBus::new()
            .execute(
                FetchHeaderGlow::new(Arc::clone(&cache), request.clone()),
                &CommandContext::next(),
            )
            .expect("header glow fetch succeeds");

        let kept = cache
            .peek_header_glow(&request.key())
            .expect("the cache keeps the glow");
        assert!(
            Arc::ptr_eq(&kept, outcome.value()),
            "the cache should keep the prepared glow"
        );
        assert!(kept.color().is_some(), "a decoded cover gives a color");
        assert!(kept.is_complete());
    }

    #[test]
    fn image_queries_honor_cancelled_context() {
        let temp = tempfile::tempdir().expect("tempdir");
        let cache = ImageCache::with_capacity(temp.path().join("thumbnails"), 1, 512, 1024 * 1024);
        let bus = CommandBus::new();

        let fetch_error = match bus.execute(
            FetchThumbnail::new(cache, "http://127.0.0.1:1/thumbnail.png", false),
            &cancelled_context(),
        ) {
            Ok(_) => panic!("cancelled thumbnail query should fail"),
            Err(error) => error,
        };

        assert_eq!(fetch_error, CommandError::Cancelled);
    }
}
