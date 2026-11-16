//! Loads the glow of a page header (ADR 0101 Decision 4, task 002).
//!
//! The view model gives a [`HeaderGlowVm`]. This module prepares its glow
//! through the async command runner (ADR 0040), so no decode, no blur and
//! no grain run on the UI thread. The UI thread only reads the cache and
//! the load state. It shows no glow until the glow is prepared.
//!
//! The load state keeps at most [`HEADER_GLOW_CAPACITY`] entries and
//! removes the oldest first. The renderer images of a removed entry leave
//! the atlas of each window in the callback of the next load, because a
//! window that renders is not in the window list of the app.

#![warn(clippy::pedantic)]

use std::collections::HashMap;
use std::sync::Arc;

use gpui::{Context, RenderImage};

use crate::application::command_context::CommandContext;
use crate::application::queries::images::FetchHeaderGlow;
use crate::media::header_glow::{GlowImage, GlowKey, GlowRequest};
use crate::media::image_cache::HEADER_GLOW_CAPACITY;
use crate::presentation::collection_art::CollectionArtHost;
use crate::presentation::present_command;
use crate::ui::composites::EntityKind;
use crate::ui::tokens::{color, CoverBackdrop};
use crate::view_models::header_glow::{GlowEntity, HeaderGlowVm};

/// The load state of one glow.
enum GlowLoad {
    Loading,
    Loaded(Arc<GlowImage>),
    /// The preparation failed. The page shows no glow.
    Failed,
}

/// The load states of the header glows of one window entity.
#[derive(Default)]
pub(crate) struct HeaderGlowLoads {
    glows: HashMap<GlowKey, GlowLoad>,
    /// The keys of `glows`, the oldest first.
    order: Vec<GlowKey>,
    /// The glow of a key with a candidate that did not read. It is not in
    /// `glows`, so the cache does not keep it. The page shows it until the
    /// page asks for another key, and the next visit of the page prepares
    /// the glow again.
    partial: Option<(GlowKey, Arc<GlowImage>)>,
    /// The last glow of a page whose candidates change (an artist page or a
    /// label page), with the scope of that page.
    held: Option<(String, Arc<GlowImage>)>,
    /// The renderer images that left the load state and still sit in an
    /// atlas.
    pending_drop: Vec<Arc<RenderImage>>,
}

/// What a call of [`HeaderGlowLoads::state`] finds for a key.
enum Found {
    /// The glow is ready.
    Ready(Arc<GlowImage>),
    /// A preparation runs, or it failed. There is no glow yet.
    Waiting,
    /// Nothing exists for the key. The caller starts a preparation.
    Missing,
}

impl HeaderGlowLoads {
    /// Marks `key` as the newest entry.
    fn touch(&mut self, key: &GlowKey) {
        if let Some(position) = self.order.iter().position(|known| known == key) {
            let known = self.order.remove(position);
            self.order.push(known);
        }
    }

    /// Keeps the images of `glow` for the removal from the atlas.
    fn release(&mut self, glow: &GlowImage) {
        self.pending_drop.extend(glow.owned_images());
    }

    /// Sets the load state of `key` as the newest entry, and removes the
    /// oldest entries while there are more than [`HEADER_GLOW_CAPACITY`].
    fn insert(&mut self, key: GlowKey, load: GlowLoad) {
        if let Some(GlowLoad::Loaded(old)) = self.glows.insert(key.clone(), load) {
            self.release(&old);
        }
        self.order.retain(|known| *known != key);
        self.order.push(key);
        while self.order.len() > HEADER_GLOW_CAPACITY {
            let oldest = self.order.remove(0);
            if let Some(GlowLoad::Loaded(old)) = self.glows.remove(&oldest) {
                self.release(&old);
            }
        }
    }

    /// Removes the entry of `key`.
    fn remove(&mut self, key: &GlowKey) {
        self.order.retain(|known| known != key);
        if let Some(GlowLoad::Loaded(old)) = self.glows.remove(key) {
            self.release(&old);
        }
    }

    /// Drops the partial glow unless it belongs to `key`.
    fn keep_partial_for(&mut self, key: &GlowKey) {
        if self
            .partial
            .as_ref()
            .is_some_and(|(partial_key, _)| partial_key != key)
        {
            if let Some((_, old)) = self.partial.take() {
                self.release(&old);
            }
        }
    }

    /// The state of `key`. A call for a key that is not the key of the
    /// partial glow removes the partial glow, so a later visit of that page
    /// prepares its glow again.
    fn state(&mut self, key: &GlowKey) -> Found {
        self.keep_partial_for(key);
        if let Some(GlowLoad::Loaded(glow)) = self.glows.get(key) {
            let glow = Arc::clone(glow);
            self.touch(key);
            return Found::Ready(glow);
        }
        if let Some((_, glow)) = &self.partial {
            return Found::Ready(Arc::clone(glow));
        }
        match self.glows.get(key) {
            Some(_) => Found::Waiting,
            None => Found::Missing,
        }
    }

    /// Records the result of the preparation of `key`. A complete glow is
    /// an entry. A glow with a candidate that did not read is the partial
    /// glow, and `key` has no entry, so the next visit tries again.
    fn finish(&mut self, key: GlowKey, glow: Arc<GlowImage>) {
        if glow.is_complete() {
            if let Some((partial_key, old)) = self.partial.take() {
                if partial_key == key {
                    self.release(&old);
                } else {
                    self.partial = Some((partial_key, old));
                }
            }
            self.insert(key, GlowLoad::Loaded(glow));
        } else {
            self.remove(&key);
            if let Some((_, old)) = self.partial.take() {
                self.release(&old);
            }
            self.partial = Some((key, glow));
        }
    }

    /// Sets the glow that a page with `scope` shows while a new glow
    /// prepares. A glow of another scope does not stay.
    fn hold(&mut self, scope: &str, glow: &Arc<GlowImage>) {
        if let Some((held_scope, held)) = &self.held {
            if held_scope == scope && Arc::ptr_eq(held, glow) {
                return;
            }
        }
        if let Some((_, old)) = self.held.take() {
            self.release(&old);
        }
        self.held = Some((scope.to_owned(), Arc::clone(glow)));
    }

    /// The glow that a page with `scope` held, when its key has no glow yet.
    fn held_for(&self, scope: &str) -> Option<Arc<GlowImage>> {
        self.held
            .as_ref()
            .filter(|(held_scope, _)| held_scope == scope)
            .map(|(_, glow)| Arc::clone(glow))
    }

    /// The renderer images to remove from the atlas now. An image that the
    /// held glow still uses stays out of the list.
    fn take_pending_drop(&mut self) -> Vec<Arc<RenderImage>> {
        let held = self.held.as_ref().map(|(_, glow)| glow.owned_images());
        std::mem::take(&mut self.pending_drop)
            .into_iter()
            .filter(|image| {
                !held
                    .as_ref()
                    .is_some_and(|held| held.iter().any(|kept| Arc::ptr_eq(kept, image)))
            })
            .collect()
    }
}

/// The entity kind of the color of a glow with no image.
const fn entity_kind(entity: GlowEntity) -> EntityKind {
    match entity {
        GlowEntity::Release => EntityKind::Release,
        GlowEntity::Track => EntityKind::Track,
        GlowEntity::Artist => EntityKind::Artist,
        GlowEntity::Label => EntityKind::Label,
    }
}

/// The request of `vm` in the live theme.
fn request_for<T: 'static>(vm: &HeaderGlowVm, cx: &Context<T>) -> GlowRequest {
    let rgba = color(cx, entity_kind(vm.entity).fill_token());
    let byte = |channel: f32| {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "the value is rounded and clamped to the range of a byte"
        )]
        let byte = (channel * 255.0).round().clamp(0.0, 255.0) as u8;
        byte
    };
    GlowRequest {
        urls: vm.urls.clone(),
        fallback_urls: vm.fallback_urls.clone(),
        entity: [byte(rgba.r), byte(rgba.g), byte(rgba.b)],
        style: CoverBackdrop::glow_style_current(cx),
    }
}

/// Removes the images that left the load state from the atlas of each
/// window. The call comes from a completion callback, where no window of
/// the app is in the middle of an update.
fn drop_released<T: CollectionArtHost>(this: &mut T, cx: &mut Context<T>) {
    let images = this
        .collection_art_parts()
        .loads
        .header_glows
        .take_pending_drop();
    for image in images {
        cx.drop_image(image, None);
    }
}

/// The glow of `vm`, or `None` while it is not prepared. The first call
/// starts the preparation. The host renders again when the glow arrives.
/// A page with a scope keeps its last glow while a new key prepares.
pub(crate) fn header_glow<T: CollectionArtHost>(
    host: &mut T,
    vm: &HeaderGlowVm,
    cx: &mut Context<T>,
) -> Option<Arc<GlowImage>> {
    let request = request_for(vm, cx);
    let key = request.key();
    let parts = host.collection_art_parts();
    let loads = &mut parts.loads.header_glows;
    if let Some(glow) = parts.cache.peek_header_glow(&key) {
        if matches!(loads.glows.get(&key), None | Some(GlowLoad::Loading)) {
            loads.insert(key.clone(), GlowLoad::Loaded(Arc::clone(&glow)));
        }
    }
    let found = loads.state(&key);
    let waiting = matches!(found, Found::Waiting);
    let shown = match found {
        Found::Ready(glow) => {
            if let Some(scope) = &vm.scope {
                loads.hold(scope, &glow);
            }
            return Some(glow);
        }
        Found::Waiting | Found::Missing => {
            vm.scope.as_deref().and_then(|scope| loads.held_for(scope))
        }
    };
    if waiting || parts.runner.availability().is_err() {
        return shown;
    }
    loads.insert(key.clone(), GlowLoad::Loading);
    let command = FetchHeaderGlow::new(Arc::clone(parts.cache), request);
    let error_key = key.clone();
    present_command(
        parts.runner,
        command,
        CommandContext::next(),
        cx,
        move |this: &mut T, glow, cx| {
            this.collection_art_parts()
                .loads
                .header_glows
                .finish(key, glow);
            drop_released(this, cx);
            cx.notify();
        },
        move |this: &mut T, _, cx| {
            this.collection_art_parts()
                .loads
                .header_glows
                .insert(error_key, GlowLoad::Failed);
            drop_released(this, cx);
        },
    );
    shown
}

#[cfg(test)]
mod tests {
    use image::{Rgba, RgbaImage};

    use super::*;
    use crate::media::header_glow::{GlowFrame, GlowStyle, PreparedGlow};

    fn style() -> GlowStyle {
        CoverBackdrop::glow_style(crate::ui::tokens::Environment::default())
    }

    fn key(name: &str) -> GlowKey {
        GlowKey::new(&[name.to_owned()], &[], [1, 2, 3], &style())
    }

    fn glow(frames: usize) -> Arc<GlowImage> {
        let frame = || GlowFrame {
            pixels: RgbaImage::from_pixel(2, 2, Rgba([10, 20, 30, 255])),
            delay_ms: 40,
        };
        Arc::new(GlowImage::new(&PreparedGlow {
            frames: (0..frames).map(|_| frame()).collect(),
            opacity: 0.5,
            grain_opacity: 0.2,
            color: None,
        }))
    }

    fn incomplete() -> Arc<GlowImage> {
        let image = Arc::try_unwrap(glow(1)).expect("one owner").incomplete();
        Arc::new(image)
    }

    /// Task 002 correction 1: the loads keep at most 16 entries, remove the
    /// oldest first, and give the images of a removed glow for the atlas.
    #[test]
    fn adr_0101_header_glow_loads_keep_sixteen_entries() {
        let mut loads = HeaderGlowLoads::default();
        let first = glow(3);
        loads.insert(key("0"), GlowLoad::Loaded(Arc::clone(&first)));
        for index in 1..HEADER_GLOW_CAPACITY {
            loads.insert(key(&index.to_string()), GlowLoad::Loaded(glow(1)));
        }
        assert_eq!(loads.glows.len(), HEADER_GLOW_CAPACITY);
        assert!(loads.take_pending_drop().is_empty());

        // A read of the oldest entry makes it the newest.
        assert!(matches!(loads.state(&key("0")), Found::Ready(_)));
        loads.insert(key("new"), GlowLoad::Loading);
        assert_eq!(loads.glows.len(), HEADER_GLOW_CAPACITY);
        assert!(loads.glows.contains_key(&key("0")));
        assert!(!loads.glows.contains_key(&key("1")), "the oldest leaves");
        assert_eq!(loads.take_pending_drop().len(), 1);

        // A removed animated glow gives its frames image and its still copy.
        let mut loads = HeaderGlowLoads::default();
        loads.insert(key("first"), GlowLoad::Loaded(Arc::clone(&first)));
        for index in 1..HEADER_GLOW_CAPACITY {
            loads.insert(key(&index.to_string()), GlowLoad::Loading);
        }
        assert!(loads.glows.contains_key(&key("first")));
        loads.insert(key("last"), GlowLoad::Loading);
        assert!(!loads.glows.contains_key(&key("first")));
        let dropped = loads.take_pending_drop();
        assert_eq!(dropped.len(), 2, "a frames image and a still copy");
        assert!(first
            .owned_images()
            .iter()
            .all(|image| dropped.iter().any(|gone| Arc::ptr_eq(gone, image))));
        assert_eq!(loads.glows.len(), loads.order.len());
        assert!(loads.order.len() <= HEADER_GLOW_CAPACITY);
    }

    /// Task 002 correction 6: a glow with a candidate that did not read is
    /// not an entry. The page shows it until it asks for another key, and
    /// then the next visit prepares the glow again.
    #[test]
    fn adr_0101_header_glow_loads_do_not_keep_an_incomplete_glow() {
        let mut loads = HeaderGlowLoads::default();
        let page = key("page");
        loads.insert(page.clone(), GlowLoad::Loading);
        loads.finish(page.clone(), incomplete());
        assert!(!loads.glows.contains_key(&page), "no entry for the key");
        assert!(loads.order.is_empty());
        assert!(
            matches!(loads.state(&page), Found::Ready(_)),
            "the page shows the partial glow"
        );

        // The page asks for another key: the partial glow leaves.
        assert!(matches!(loads.state(&key("other")), Found::Missing));
        assert!(loads.partial.is_none());
        assert_eq!(loads.take_pending_drop().len(), 1);
        // The next visit of the first page prepares again.
        assert!(matches!(loads.state(&page), Found::Missing));

        // A complete glow is an entry.
        loads.finish(page.clone(), glow(1));
        assert!(loads.glows.contains_key(&page));
        assert!(matches!(loads.state(&page), Found::Ready(_)));
    }

    /// Task 002 correction 5: a page with a scope keeps the glow of its last
    /// key. A page with another scope does not get it.
    #[test]
    fn adr_0101_header_glow_loads_hold_the_last_glow_of_a_page() {
        let mut loads = HeaderGlowLoads::default();
        let last = glow(1);
        loads.hold("artist-name:a", &last);
        assert!(loads
            .held_for("artist-name:a")
            .is_some_and(|held| Arc::ptr_eq(&held, &last)));
        assert!(loads.held_for("artist-name:b").is_none());
        let next = glow(1);
        loads.hold("artist-name:b", &next);
        assert!(loads.held_for("artist-name:a").is_none());

        // The replaced glow leaves the atlas. The held glow keeps its images
        // out of the removal.
        assert_eq!(loads.take_pending_drop().len(), last.owned_images().len());
        loads.release(&next);
        assert!(loads.take_pending_drop().is_empty());
    }
}
