//! The glow fact of a page header (ADR 0101 Decision 4, musicindex ADR 0021
//! R-ART-5).
//!
//! An album page, a track page, an artist page and a label page show a glow
//! from the art of the item. This module gives the fact: the candidate image
//! URLs in the order of the page, or no URL when the item has no image. The
//! renderer maps [`GlowEntity`] to its entity color, and the image cache
//! prepares the pixels off the UI thread. The module carries no renderer
//! type.

#![warn(clippy::pedantic)]

use crate::media::header_glow::MAX_CANDIDATES;

/// The kind of the page. It selects the entity color of a glow with no
/// image, and the tint below a black and white cover.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) enum GlowEntity {
    /// An album page.
    Release,
    /// A track page.
    Track,
    /// An artist page.
    Artist,
    /// A label page.
    Label,
}

/// The glow of one page header.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HeaderGlowVm {
    /// The candidate image URLs in the order of the page. Empty when the
    /// item has no image. Then the glow has the entity color.
    pub(crate) urls: Vec<String>,
    /// The candidates that the glow uses when the own image in `urls` does
    /// not read. Empty when the item has no own image, or no other cover.
    pub(crate) fallback_urls: Vec<String>,
    pub(crate) entity: GlowEntity,
    /// The name of the page, for a page whose candidates change while the
    /// page is open. The page keeps its last glow while a new glow
    /// prepares. `None` for a page with fixed candidates.
    pub(crate) scope: Option<String>,
}

/// The first [`MAX_CANDIDATES`] distinct, trimmed and non-empty URLs of
/// `urls`, with the URLs of `skip` left out.
fn distinct_urls<I, S>(urls: I, skip: Option<&str>) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut kept: Vec<String> = Vec::with_capacity(MAX_CANDIDATES);
    for url in urls {
        let url = url.as_ref().trim();
        if url.is_empty() || Some(url) == skip || kept.iter().any(|known| known == url) {
            continue;
        }
        kept.push(url.to_owned());
        if kept.len() == MAX_CANDIDATES {
            break;
        }
    }
    kept
}

impl HeaderGlowVm {
    /// Gives the glow fact of one page.
    ///
    /// `own` is the image of the item. When it exists, it is the one
    /// candidate, and the first [`MAX_CANDIDATES`] distinct URLs of
    /// `covers` are the fallback for an own image that does not read.
    /// Otherwise the candidates are those URLs of `covers`. Each URL is
    /// trimmed, and an empty URL is dropped.
    #[must_use]
    pub(crate) fn resolve<I, S>(entity: GlowEntity, own: Option<&str>, covers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let own = own.map(str::trim).filter(|url| !url.is_empty());
        let (urls, fallback_urls) = match own {
            Some(url) => (vec![url.to_owned()], distinct_urls(covers, Some(url))),
            None => (distinct_urls(covers, None), Vec::new()),
        };
        Self {
            urls,
            fallback_urls,
            entity,
            scope: None,
        }
    }

    /// Names the page, so that it keeps its last glow while a new glow
    /// prepares.
    #[must_use]
    pub(crate) fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn covers(count: usize) -> Vec<String> {
        (0..count).map(|index| format!("cover-{index}")).collect()
    }

    /// M4 (ADR 0101 Decision 4, task 002): the own image is the one
    /// candidate, and it comes before any cover.
    #[test]
    fn adr_0101_header_glow_m4_own_image_is_the_one_candidate() {
        let glow = HeaderGlowVm::resolve(GlowEntity::Release, Some(" own "), covers(3));
        assert_eq!(glow.urls, vec!["own".to_owned()]);
        assert!(!glow.urls.is_empty());
    }

    /// M4: with no own image, the first eight distinct covers are the
    /// candidates, in the order of the page.
    #[test]
    fn adr_0101_header_glow_m4_covers_give_eight_candidates_at_most() {
        let mut urls = covers(12);
        urls.insert(1, "cover-0".to_owned());
        urls.insert(2, " ".to_owned());
        let glow = HeaderGlowVm::resolve(GlowEntity::Artist, None, urls);
        assert_eq!(glow.urls, covers(MAX_CANDIDATES));
    }

    /// M4: an item with no image gives no URL, so the glow has the entity
    /// color of its kind. This holds for an album, a track, an artist and a
    /// label page.
    #[test]
    fn adr_0101_header_glow_m4_no_image_gives_the_entity_color() {
        for entity in [
            GlowEntity::Release,
            GlowEntity::Track,
            GlowEntity::Artist,
            GlowEntity::Label,
        ] {
            let glow = HeaderGlowVm::resolve(entity, Some("  "), Vec::<String>::new());
            assert!(glow.urls.is_empty(), "{entity:?}");
            assert_eq!(glow.entity, entity);
        }
    }

    /// M4 (task 002 correction 6): an own image has the covers as its
    /// fallback, without the own image itself. With no own image, the
    /// covers are the candidates and there is no fallback.
    #[test]
    fn adr_0101_header_glow_m4_own_image_has_the_covers_as_fallback() {
        let glow = HeaderGlowVm::resolve(GlowEntity::Label, Some("cover-1"), covers(3));
        assert_eq!(glow.urls, vec!["cover-1".to_owned()]);
        assert_eq!(
            glow.fallback_urls,
            vec!["cover-0".to_owned(), "cover-2".to_owned()]
        );
        let none = HeaderGlowVm::resolve(GlowEntity::Label, None, covers(3));
        assert!(none.fallback_urls.is_empty());
        assert_eq!(none.scope, None);
        assert_eq!(none.with_scope("label").scope.as_deref(), Some("label"));
    }
}
