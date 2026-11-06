//! The Music list column's scroll and paging wire-up (ADR 0084 Decision
//! 1, amended 2026-10-17, task 014).
//!
//! Split out of `content_list.rs` to keep that file under the ADR 0038
//! Task 007 line ceiling.

#![warn(clippy::pedantic)]

use gpui::{canvas, prelude::*, AnyElement, Context, ScrollHandle, StatefulInteractiveElement};

use crate::library::LibraryApp;
use crate::ui::composites::music_column_scroll::music_column_tracked_scrollbar;
use crate::view_models::pagination::should_load_more_near_scroll_end;

/// Makes `container` the Music list column's scrolling area, with the
/// tracked handle and the shared scrollbar.
///
/// The near-end paging check runs on a `canvas` prepaint, after each
/// scroll change, following the reading pattern of `src/ui/composites/
/// log_frame.rs`. A wheel scroll, a touchpad gesture and a scrollbar drag
/// all repaint the window, so each one reaches this same check.
pub(crate) fn attach_content_list_scroll<E>(
    container: E,
    has_more: bool,
    is_loading: bool,
    scroll_handle: &ScrollHandle,
    cx: &mut Context<LibraryApp>,
) -> AnyElement
where
    E: StatefulInteractiveElement + Styled + IntoElement + 'static,
{
    attach_list_scroll_with(
        container,
        has_more,
        is_loading,
        scroll_handle,
        |this, cx| this.start_recent_music_load(true, cx),
        cx,
    )
}

/// Makes `container` a scrolling area that runs `load_more` near its end
/// (ADR 0089 packet 009). The Public playlists list pages its own rows, so
/// it gives its own action. [`attach_content_list_scroll`] runs the recent
/// Releases load.
pub(crate) fn attach_list_scroll_with<E>(
    container: E,
    has_more: bool,
    is_loading: bool,
    scroll_handle: &ScrollHandle,
    load_more: fn(&mut LibraryApp, &mut Context<LibraryApp>),
    cx: &mut Context<LibraryApp>,
) -> AnyElement
where
    E: StatefulInteractiveElement + Styled + IntoElement + 'static,
{
    let probe_scroll = scroll_handle.clone();
    let owner = cx.weak_entity();
    let probe = canvas(
        move |_bounds, _window, cx| {
            let viewport_height = f32::from(probe_scroll.bounds().size.height);
            let scroll_offset_y = f32::from(probe_scroll.offset().y);
            let content_height = viewport_height + f32::from(probe_scroll.max_offset().y);
            if should_load_more_near_scroll_end(
                scroll_offset_y,
                viewport_height,
                content_height,
                has_more,
                is_loading,
            ) {
                let _ = owner.update(cx, load_more);
            }
        },
        |_, (), _, _| {},
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
    .into_any_element();

    music_column_tracked_scrollbar(container, scroll_handle, Some(probe))
}
