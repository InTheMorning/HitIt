//! The one scrollbar choice for a Music column (ADR 0084 Decision 1,
//! amended 2026-10-17, and ADR 0063's shared scrollbar gutter).
//!
//! A Music column that scrolls calls one of the two functions below. No
//! shell in Music picks its own scroll method. The caller still wraps its
//! own content in [`page_scroll_content`](super::page_scroll_content), so
//! the scrollbar covers no content.
//!
//! A long list uses [`music_column_list_scrollbar`] with a
//! [`MusicColumnList`]: a virtual list that renders only the items in view
//! and in the overdraw area (ADR 0084 task 037).

#![warn(clippy::pedantic)]

use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    canvas, div, point, prelude::*, AnyElement, App, Entity, List, ListAlignment, ListState,
    Pixels, ScrollHandle, Window,
};
use gpui_component::scroll::ScrollableElement;

use crate::ui::focus_ring::{begin_focus_reveal, take_focus_reveal};
use crate::ui::tokens::{Size, Spacing};
use crate::view_models::detail_selection::list_bar_is_pinned;
use crate::view_models::list_selection::FocusStep;
use crate::view_models::releases_everything::item_id_runs;

/// Makes `container` a Music column that scrolls with `scroll`, with the
/// scrollbar on the returned parent (ADR 0084 Decision 1, amended
/// 2026-10-17). Use this when the column must keep a scroll position
/// across a navigation change, for example Back to "New releases"
/// (task 006 V3). It also scrolls a focused control into view (ADR 0090
/// Invariants). `container` already has a stable `.id(..)`. When one call
/// site renders more than one page, each page needs its own handle.
///
/// `probe` is an optional layer over the column. A caller that reads the
/// scroll position after each change, as `src/ui/composites/log_frame.rs`
/// does, passes an empty layer. A caller that draws an overlay, such as the
/// pinned list bar of [`ListBarPin`], passes that overlay.
///
/// The returned parent is a plain block box, following the gutter
/// pattern of `src/ui/composites/settings.rs`. `container` gets
/// `.size_full()` here, so its own `flex_1()`/`min_h_0()` from a flex
/// ancestor further up stay harmless, not load-bearing, inside it.
pub(crate) fn music_column_tracked_scrollbar<E>(
    container: E,
    scroll: &ScrollHandle,
    probe: Option<AnyElement>,
) -> AnyElement
where
    E: StatefulInteractiveElement + Styled + IntoElement + 'static,
{
    // GPUI prepaints children in order, so only a control in `container`
    // gives its bounds between these two layers (ADR 0090 invariant "keyboard
    // reach").
    let mut wrapper = div()
        .relative()
        .flex_1()
        .min_h_0()
        .min_w_0()
        .child(
            canvas(
                |_bounds, _window, cx| begin_focus_reveal(cx),
                |_bounds, (), _window, _cx| {},
            )
            .absolute(),
        )
        .child(
            container
                .size_full()
                .overflow_y_scroll()
                .track_scroll(scroll),
        )
        .child(focus_reveal_layer(scroll));
    if let Some(probe) = probe {
        wrapper = wrapper.child(probe);
    }
    wrapper.vertical_scrollbar(scroll).into_any_element()
}

/// The layer after the content of a Music column. It scrolls the column by
/// the smallest distance that shows the control that took the focus. A margin
/// keeps its focus ring clear of the edge of the column.
fn focus_reveal_layer(scroll: &ScrollHandle) -> impl IntoElement {
    let scroll = scroll.clone();
    canvas(
        move |_bounds, window, cx| {
            let Some(control) = take_focus_reveal(cx) else {
                return;
            };
            let control = control.dilate(Spacing::SM.scaled(cx));
            let viewport = scroll.bounds();
            let offset = scroll.offset();
            if let Some(offset_y) = reveal_offset_y(
                (f32::from(viewport.top()), f32::from(viewport.bottom())),
                (f32::from(control.top()), f32::from(control.bottom())),
                f32::from(offset.y),
                f32::from(scroll.max_offset().y),
            ) {
                scroll.set_offset(point(offset.x, Pixels::from(offset_y)));
                window.on_next_frame(|window, _cx| window.refresh());
            }
        },
        |_bounds, (), _window, _cx| {},
    )
    .absolute()
}

/// The pinned copy of a list bar above the rows of a Music column (ADR 0090
/// Decision 5). GPUI has no sticky style, so the caller draws the copy in
/// an overlay when [`Self::is_pinned`] gives `true`.
///
/// The in-flow bar carries [`Self::probe`]. Each paint, the probe records
/// where the bar is, so the bar position does not depend on the scroll
/// offset. A render reads the current offset and the recorded position, so
/// a scroll shows or hides the copy in the same frame. The probe asks for
/// one more redraw only when the pinned state differs from the state that
/// the last render showed.
#[derive(Clone, Debug, Default)]
pub(crate) struct ListBarPin {
    /// The top of the bar at zero scroll offset, in pixels from the top of
    /// the viewport. `None` before the first paint of the bar.
    rest_top: Rc<Cell<Option<f32>>>,
    /// The height of the bar. `None` before the first paint of the bar.
    height: Rc<Cell<Option<f32>>>,
    /// The height that the pinned copy covers, with its top padding.
    /// `None` before the first paint of the pinned copy.
    pinned_height: Rc<Cell<Option<f32>>>,
    /// `true` when the last render drew the pinned copy.
    shown: Rc<Cell<bool>>,
}

impl ListBarPin {
    /// `true` when the pinned copy must show for the scroll container
    /// `scroll`. A render calls this once. It records the answer for the
    /// probe.
    #[must_use]
    pub(crate) fn is_pinned(&self, scroll: &ScrollHandle) -> bool {
        let pinned = list_bar_is_pinned(
            f32::from(scroll.offset().y),
            f32::from(scroll.max_offset().y),
            self.rest_top.get(),
        );
        self.shown.set(pinned);
        pinned
    }

    /// The height that the pinned bar covers at the top of the viewport at
    /// `offset_y`: the bar height when the bar is pinned at that offset, and
    /// 0 in all other conditions.
    #[must_use]
    pub(crate) fn pinned_inset(&self, offset_y: f32, max_offset_y: f32) -> f32 {
        if list_bar_is_pinned(offset_y, max_offset_y, self.rest_top.get()) {
            self.pinned_height
                .get()
                .or_else(|| self.height.get())
                .unwrap_or(0.0)
        } else {
            0.0
        }
    }

    /// An empty layer over the pinned copy. It paints nothing. At each
    /// paint it records the height that the copy covers, for
    /// [`Self::pinned_inset`].
    #[must_use]
    pub(crate) fn pinned_probe(&self) -> AnyElement {
        let pinned_height = Rc::clone(&self.pinned_height);
        canvas(
            move |bounds, _window, _cx| {
                pinned_height.set(Some(f32::from(bounds.size.height)));
            },
            |_bounds, (), _window, _cx| {},
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .into_any_element()
    }

    /// An empty layer over the in-flow bar. It paints nothing. At each
    /// paint it records the bar position and the bar height. When the pinned state differs
    /// from the state of the last render, it asks `entity` for one redraw.
    #[must_use]
    pub(crate) fn probe<T: 'static>(
        &self,
        scroll: &ScrollHandle,
        entity: &Entity<T>,
    ) -> AnyElement {
        let scroll = scroll.clone();
        let rest_top = Rc::clone(&self.rest_top);
        let height = Rc::clone(&self.height);
        let shown = Rc::clone(&self.shown);
        let entity = entity.downgrade();
        canvas(
            move |bounds, window, _cx| {
                let viewport = scroll.bounds();
                let offset = scroll.offset();
                let top = f32::from(bounds.top() - viewport.top() - offset.y);
                rest_top.set(Some(top));
                height.set(Some(f32::from(bounds.size.height)));
                let pinned = list_bar_is_pinned(
                    f32::from(offset.y),
                    f32::from(scroll.max_offset().y),
                    Some(top),
                );
                if pinned != shown.get() {
                    window.on_next_frame(move |_, cx| {
                        let _ = entity.update(cx, |_, cx| cx.notify());
                    });
                }
            },
            |_bounds, (), _window, _cx| {},
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .into_any_element()
    }
}

/// The state of one virtual Music list (ADR 0084 task 037, decision D1):
/// the gpui [`ListState`] and the element id of each item.
///
/// [`Self::sync`] compares the new ids with the old ids. Each run of added
/// or removed ids is one [`ListState::splice`], so the first row in view
/// stays in view, and an item with the same id keeps its measured size.
pub(crate) struct MusicColumnList {
    state: ListState,
    ids: Vec<String>,
    /// The height that an item counts before its first layout, so the
    /// scrollbar thumb has its correct size.
    item_height: Pixels,
    /// The list width when the items last got [`Self::item_height`].
    /// `None` before the first sync.
    hinted_width: Option<Pixels>,
}

impl MusicColumnList {
    /// An empty list. The overdraw and the item height come from named
    /// tokens.
    #[must_use]
    pub(crate) fn new(cx: &App) -> Self {
        Self {
            state: ListState::new(0, ListAlignment::Top, Size::ListOverdraw.scaled(cx)),
            ids: Vec::new(),
            item_height: Size::MinHitTarget.scaled(cx),
            hinted_width: None,
        }
    }

    /// The gpui state of the list, for the `list` element and its
    /// scrollbar.
    #[must_use]
    pub(crate) const fn state(&self) -> &ListState {
        &self.state
    }

    /// Makes the list hold the items `ids`. Call it at each render of the
    /// list. It never calls [`ListState::reset`], because that clears the
    /// scroll position.
    pub(crate) fn sync(&mut self, ids: &[String], window: &Window) {
        let changed = self.ids != ids;
        if changed {
            for run in item_id_runs(&self.ids, ids).into_iter().rev() {
                self.state.splice(run.old, run.new_count);
            }
            self.ids = ids.to_vec();
        }
        // An item has no height until its first layout, so the scrollbar
        // counts it as 0 px. A splice gives a new item no height, and a
        // layout at a new width removes the height of each item. This call
        // gives each item with no height the item height. A measured item
        // keeps its size, and the scroll position does not change.
        // `ListState` is a shared handle, so the call on a clone changes
        // this state.
        let width = self.state.viewport_bounds().size.width;
        let new_width = self.hinted_width != Some(width);
        if changed || new_width {
            let _ = self
                .state
                .clone()
                .with_uniform_item_height(self.item_height);
            self.hinted_width = Some(width);
        }
        // The layout after a new width removes the heights again, so the
        // next frame gives them one more time.
        if new_width {
            window.request_animation_frame();
        }
    }

    /// Scrolls the list by the smallest distance that shows the item
    /// `index` (ADR 0084 Decision 10).
    pub(crate) fn scroll_to_reveal_item(&self, index: usize) {
        self.state.scroll_to_reveal_item(index);
    }
}

/// Makes the virtual `list` a Music column with the scrollbar of `state`
/// (ADR 0084 Decision 1 and task 037). The scrollbar and its gutter are
/// the same as in [`music_column_tracked_scrollbar`]. Each item of `list`
/// keeps the gutter clear in its own
/// [`page_scroll_content`](super::page_scroll_content).
pub(crate) fn music_column_list_scrollbar(list: List, state: &ListState) -> AnyElement {
    div()
        .relative()
        .flex_1()
        .min_h_0()
        .min_w_0()
        .child(list.size_full())
        .vertical_scrollbar(state)
        .into_any_element()
}

/// A request to keep the focused row of a Music detail page in view (ADR
/// 0084 Decision 10, packet 018).
///
/// The rows of a detail page are not direct children of its scroll
/// container, so `ScrollHandle::scroll_to_item` cannot find them. The
/// focused row carries [`Self::probe`] instead. After a key moves the
/// focus, the caller calls [`Self::request`]. At the next layout, the
/// probe reads the bounds of its row and scrolls the column by the
/// smallest distance that shows the full row.
#[derive(Clone, Debug, Default)]
pub(crate) struct ScrollReveal {
    scroll: ScrollHandle,
    pending: Rc<Cell<bool>>,
    pending_row: Rc<Cell<Option<usize>>>,
    section_top: Rc<Cell<Option<f32>>>,
    row_height: Rc<Cell<Option<f32>>>,
    /// The pinned list bar that covers the top of the viewport.
    pin: Option<ListBarPin>,
}

impl ScrollReveal {
    /// A reveal request for the column that scrolls with `scroll`.
    #[must_use]
    pub(crate) fn new(scroll: ScrollHandle) -> Self {
        Self {
            scroll,
            pending: Rc::new(Cell::new(false)),
            pending_row: Rc::new(Cell::new(None)),
            section_top: Rc::new(Cell::new(None)),
            row_height: Rc::new(Cell::new(None)),
            pin: None,
        }
    }

    /// Makes the reveal keep a row below the pinned list bar of `pin`. The
    /// reveal and the bar use the same cells, because a clone of a pin
    /// shares them.
    #[must_use]
    pub(crate) fn below_pin(mut self, pin: ListBarPin) -> Self {
        self.pin = Some(pin);
        self
    }

    /// Asks the next layout to show the row that carries the probe.
    pub(crate) fn request(&self) {
        self.pending.set(true);
    }

    /// Requests that row `index` of a windowed section comes into view.
    pub(crate) fn request_row(&self, index: usize) {
        self.pending_row.set(Some(index));
    }

    /// An empty layer over the focused row. It paints nothing. The row
    /// gives it its own size.
    #[must_use]
    pub(crate) fn probe(&self) -> AnyElement {
        let scroll = self.scroll.clone();
        let pending = Rc::clone(&self.pending);
        let pin = self.pin.clone();
        canvas(
            move |bounds, window, cx| {
                if !pending.replace(false) {
                    return;
                }
                reveal_row(
                    &scroll,
                    pin.as_ref(),
                    bounds.dilate(Spacing::SM.scaled(cx)),
                    window,
                );
            },
            |_bounds, (), _window, _cx| {},
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .into_any_element()
    }

    /// An empty layer over row `index` that completes a windowed row reveal.
    #[must_use]
    pub(crate) fn probe_row(&self, index: usize) -> AnyElement {
        let scroll = self.scroll.clone();
        let pending_row = Rc::clone(&self.pending_row);
        let pin = self.pin.clone();
        canvas(
            move |bounds, window, cx| {
                if pending_row.get() != Some(index) {
                    return;
                }
                pending_row.set(None);
                reveal_row(
                    &scroll,
                    pin.as_ref(),
                    bounds.dilate(Spacing::SM.scaled(cx)),
                    window,
                );
            },
            |_bounds, (), _window, _cx| {},
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .into_any_element()
    }
}

/// Scrolls `scroll` by the smallest distance that shows the row with
/// `bounds` below the pinned bar of `pin`. The caller adds the margin that
/// keeps the focus ring clear of the edge, as [`focus_reveal_layer`] does. The bar covers the top of the
/// viewport only at a pinned offset. When the first offset needs a larger
/// inset, the offset is calculated again with that inset.
fn reveal_row(
    scroll: &ScrollHandle,
    pin: Option<&ListBarPin>,
    bounds: gpui::Bounds<Pixels>,
    window: &mut Window,
) {
    let viewport = scroll.bounds();
    let offset = scroll.offset();
    let max_offset_y = f32::from(scroll.max_offset().y);
    let row = (f32::from(bounds.top()), f32::from(bounds.bottom()));
    let reveal = |inset: f32| {
        reveal_offset_y(
            (
                f32::from(viewport.top()) + inset,
                f32::from(viewport.bottom()),
            ),
            row,
            f32::from(offset.y),
            max_offset_y,
        )
    };
    let inset_at = |offset_y: f32| pin.map_or(0.0, |pin| pin.pinned_inset(offset_y, max_offset_y));
    let mut next = reveal(inset_at(f32::from(offset.y)));
    if let Some(first) = next {
        let inset = inset_at(first);
        if inset > inset_at(f32::from(offset.y)) {
            next = reveal(inset);
        }
    }
    if let Some(offset_y) = next {
        scroll.set_offset(point(offset.x, Pixels::from(offset_y)));
        window.on_next_frame(|window, _cx| window.refresh());
    }
}

/// Builds only the rows near the visible part of a release track section.
pub(crate) fn windowed_track_rows<F>(
    scroll: &ScrollHandle,
    reveal: &ScrollReveal,
    count: usize,
    row_space: Pixels,
    initial_row_height: Pixels,
    mut make_row: F,
) -> AnyElement
where
    F: FnMut(usize) -> AnyElement,
{
    let row_height = reveal
        .row_height
        .get()
        .unwrap_or_else(|| f32::from(initial_row_height));
    let row_space_value = f32::from(row_space);
    let pitch = row_height + row_space_value;
    let scroll = scroll.clone();
    let reveal = reveal.clone();
    let start_end = windowed_track_range(&scroll, &reveal, count, row_space);
    let (start, end) = (start_end.start, start_end.end);

    let first_height = Rc::clone(&reveal.row_height);
    let section_top = Rc::clone(&reveal.section_top);
    let pending_row = Rc::clone(&reveal.pending_row);
    let scroll_probe = scroll.clone();
    let probe_layer = canvas(
        move |bounds, window, _cx| {
            let viewport = scroll_probe.bounds();
            let offset = scroll_probe.offset();
            let top = f32::from(bounds.top() - viewport.top() - offset.y);
            section_top.set(Some(top));
            if let Some(index) = pending_row.get() {
                if index >= count || (index >= start && index < end) {
                    return;
                }
                let measured = first_height.get().unwrap_or(row_height);
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "Pixel layout precision is sufficient for release row offsets."
                )]
                let row_index = u32::try_from(index).expect("release rows fit in u32") as f32;
                let row_top = f32::from(viewport.top())
                    + top
                    + f32::from(offset.y)
                    + row_index * (measured + row_space_value);
                if let Some(next) = reveal_offset_y(
                    (f32::from(viewport.top()), f32::from(viewport.bottom())),
                    (row_top, row_top + measured),
                    f32::from(offset.y),
                    f32::from(scroll_probe.max_offset().y),
                ) {
                    scroll_probe.set_offset(point(offset.x, Pixels::from(next)));
                    window.on_next_frame(|window, _cx| window.refresh());
                }
            }
        },
        |_bounds, (), _window, _cx| {},
    )
    .absolute()
    .size_full();

    let rows = (start..end).map(|index| {
        let row = make_row(index);
        let row_height = Rc::clone(&reveal.row_height);
        let measurement = canvas(
            move |bounds, window, _cx| {
                let height = f32::from(bounds.size.height);
                if row_height.get() != Some(height) {
                    row_height.set(Some(height));
                    window.on_next_frame(|window, _cx| window.refresh());
                }
            },
            |_bounds, (), _window, _cx| {},
        )
        .absolute()
        .size_full();
        let row = div().relative().child(row).child(measurement);
        if index + 1 < count {
            row.mb(row_space).into_any_element()
        } else {
            row.into_any_element()
        }
    });
    #[expect(
        clippy::cast_precision_loss,
        reason = "Pixel layout precision is sufficient for release row offsets."
    )]
    let top_rows = u32::try_from(start).expect("release rows fit in u32") as f32;
    let top_height = Pixels::from(top_rows * pitch);
    let bottom_height = if end < count {
        #[expect(
            clippy::cast_precision_loss,
            reason = "Pixel layout precision is sufficient for release row offsets."
        )]
        let hidden_rows = u32::try_from(count - end).expect("release rows fit in u32") as f32;
        Pixels::from((hidden_rows * pitch - row_space_value).max(0.0))
    } else {
        Pixels::from(0.)
    };
    div()
        .relative()
        .flex()
        .flex_col()
        .child(div().h(top_height))
        .children(rows)
        .child(div().h(bottom_height))
        .child(probe_layer)
        .into_any_element()
}

/// Gives the row indices for the viewport and its ten-row margins.
///
/// The row pitch comes from the measured row height. Before the first
/// measurement, the range is the first 40 rows. Thus the host that resolves
/// the row thumbnails and the section that shows the rows use the same range.
pub(crate) fn windowed_track_range(
    scroll: &ScrollHandle,
    reveal: &ScrollReveal,
    count: usize,
    row_space: Pixels,
) -> std::ops::Range<usize> {
    const ROW_MARGIN: usize = 10;
    const INITIAL_ROWS: usize = 40;
    let Some(row_height) = reveal.row_height.get() else {
        return 0..count.min(INITIAL_ROWS);
    };
    let pitch = (row_height + f32::from(row_space)).max(1.0);
    let viewport = scroll.bounds();
    let section_top = reveal.section_top.get().unwrap_or(0.0);
    // The top of the section relative to the top of the viewport.
    let current_top = section_top + f32::from(scroll.offset().y);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "The nonnegative viewport row index is bounded by the row count."
    )]
    let visible_start = ((0.0 - current_top) / pitch).floor().max(0.0) as usize;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "The nonnegative viewport row index is bounded by the row count."
    )]
    let visible_end = ((f32::from(viewport.size.height) - current_top) / pitch)
        .ceil()
        .max(0.0) as usize;
    let start = visible_start.saturating_sub(ROW_MARGIN).min(count);
    let end = visible_end.saturating_add(ROW_MARGIN).min(count).max(start);
    start..end
}

/// The scroll offset that shows a full row, or `None` when the row is
/// already in view. `viewport` and `row` are the top and the bottom edge
/// in pixels, with the present `offset_y` applied. The offset is 0 at the
/// top and `-max_offset_y` at the bottom.
fn reveal_offset_y(
    viewport: (f32, f32),
    row: (f32, f32),
    offset_y: f32,
    max_offset_y: f32,
) -> Option<f32> {
    let (view_top, view_bottom) = viewport;
    let (row_top, row_bottom) = row;
    let next = if row_top < view_top {
        offset_y + (view_top - row_top)
    } else if row_bottom > view_bottom {
        offset_y - (row_bottom - view_bottom)
    } else {
        return None;
    };
    Some(next.clamp(-max_offset_y.max(0.0), 0.0))
}

/// The scroll offset in pixels after a key on a Music detail page with no
/// selectable row (ADR 0084 Decision 10, packet 018). Up and Down move one
/// `step`. Page Up and Page Down move one view height less one `step`.
/// Home and End go to the top and the bottom. The offset is 0 at the top
/// and `-max_offset_y` at the bottom.
#[must_use]
pub(crate) fn key_scroll_offset_y(
    key: FocusStep,
    offset_y: f32,
    view_height: f32,
    max_offset_y: f32,
    step: f32,
) -> f32 {
    let page = (view_height - step).max(step);
    let next = match key {
        FocusStep::Up => offset_y + step,
        FocusStep::Down => offset_y - step,
        FocusStep::PageUp => offset_y + page,
        FocusStep::PageDown => offset_y - page,
        FocusStep::Home => 0.0,
        FocusStep::End => -max_offset_y,
    };
    next.clamp(-max_offset_y.max(0.0), 0.0)
}

/// Scrolls `scroll` for one key on a Music detail page with no selectable
/// row. `step` is the scaled [`crate::ui::tokens::Size::KeyScrollStep`].
pub(crate) fn key_scroll(scroll: &ScrollHandle, key: FocusStep, step: Pixels) {
    let offset = scroll.offset();
    let offset_y = key_scroll_offset_y(
        key,
        f32::from(offset.y),
        f32::from(scroll.bounds().size.height),
        f32::from(scroll.max_offset().y),
        f32::from(step),
    );
    scroll.set_offset(point(offset.x, Pixels::from(offset_y)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{list, px, ListOffset, TestAppContext};

    /// The window of the R37-07 and R37-08 tests: a virtual Music list of
    /// constructed rows, 40 px high each, in a column 600 px high.
    struct VirtualListTest {
        list: MusicColumnList,
        ids: Vec<String>,
    }

    impl Render for VirtualListTest {
        fn render(
            &mut self,
            window: &mut gpui::Window,
            _cx: &mut gpui::Context<Self>,
        ) -> impl IntoElement {
            self.list.sync(&self.ids, window);
            let ids = Rc::new(self.ids.clone());
            let rows = list(self.list.state().clone(), move |index, _window, _cx| {
                let id = ids[index].clone();
                let selector = id.clone();
                div()
                    .h(px(40.))
                    .debug_selector(move || selector)
                    .child(id)
                    .into_any_element()
            });
            div()
                .w(px(400.))
                .h(px(600.))
                .flex()
                .flex_col()
                .child(music_column_list_scrollbar(rows, self.list.state()))
        }
    }

    gpui::actions!(focus_reveal_test, [FocusNextControl]);

    struct RevealRoot {
        focus: gpui::FocusHandle,
        scroll: ScrollHandle,
    }

    impl gpui::Render for RevealRoot {
        fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
            let button = |id: &'static str| {
                crate::ui::primitives::button::Button::plain(id)
                    .label(id)
                    .on_click(|_, _, _| {})
            };
            div()
                .key_context("RevealRoot")
                .track_focus(&self.focus)
                .on_action(|_: &FocusNextControl, window, cx| window.focus_next(cx))
                .size_full()
                .flex()
                .flex_col()
                .child(button("outside"))
                .child(music_column_tracked_scrollbar(
                    div()
                        .id("reveal-column")
                        .child(button("inside"))
                        .child(div().h(gpui::px(5000.))),
                    &self.scroll,
                    None,
                ))
        }
    }

    /// ADR 0090 invariant "keyboard reach", walk of 2026-11-07: with the page
    /// scrolled down, Tab passed buttons above the viewport with no visible
    /// focus. Tab now shows the button. A button outside the column does not
    /// move the column, and a later wheel scroll does not move back.
    #[gpui::test]
    fn adr_0090_tab_scrolls_a_focused_button_into_view(cx: &mut TestAppContext) {
        cx.update(|cx| {
            gpui_component::init(cx);
            crate::ui::focus_ring::init(cx);
            cx.bind_keys([gpui::KeyBinding::new(
                "tab",
                FocusNextControl,
                Some("RevealRoot"),
            )]);
        });
        let scroll = ScrollHandle::new();
        let root_scroll = scroll.clone();
        let (root, cx) = cx.add_window_view(|_, cx| RevealRoot {
            focus: cx.focus_handle(),
            scroll: root_scroll,
        });
        let focus = cx.update(|_, cx| root.read(cx).focus.clone());
        let draw = |cx: &mut gpui::VisualTestContext| {
            cx.update(|window, cx| window.draw(cx).clear(cx));
            cx.update(|window, cx| window.draw(cx).clear(cx));
            cx.run_until_parked();
        };
        let down = point(gpui::px(0.), gpui::px(-1000.));
        cx.update(|window, cx| {
            window.activate_window();
            window.focus(&focus, cx);
        });
        draw(cx);
        scroll.set_offset(down);
        draw(cx);

        cx.simulate_keystrokes("tab");
        draw(cx);
        assert_eq!(
            scroll.offset().y,
            down.y,
            "the outside button moves nothing"
        );

        cx.simulate_keystrokes("tab");
        draw(cx);
        assert_eq!(
            scroll.offset().y,
            gpui::px(0.),
            "Tab shows the inside button"
        );

        scroll.set_offset(down);
        draw(cx);
        assert_eq!(scroll.offset().y, down.y, "a wheel scroll stays");
    }

    fn row_ids(count: usize) -> Vec<String> {
        (1..=count).map(|row| format!("r37-row-{row}")).collect()
    }

    struct WindowedRowsTest {
        reveal: ScrollReveal,
        scroll: ScrollHandle,
        rendered: Rc<Cell<usize>>,
        /// The space above the scroll container in the window.
        top_gap: Pixels,
    }

    impl gpui::Render for WindowedRowsTest {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut gpui::Context<Self>,
        ) -> impl IntoElement {
            self.rendered.set(0);
            let rendered = Rc::clone(&self.rendered);
            let reveal = self.reveal.clone();
            let rows = windowed_track_rows(
                &self.scroll,
                &self.reveal,
                491,
                px(2.),
                px(40.),
                move |index| {
                    rendered.set(rendered.get() + 1);
                    let selector = format!("windowed-row-{index}");
                    div()
                        .h(px(40.))
                        .debug_selector(move || selector.clone())
                        .child(reveal.probe_row(index))
                        .into_any_element()
                },
            );
            div()
                .w(px(400.))
                .h(px(600.) + self.top_gap)
                .flex()
                .flex_col()
                .child(div().flex_none().h(self.top_gap))
                .child(music_column_tracked_scrollbar(
                    div().id("windowed-rows").child(rows),
                    &self.scroll,
                    None,
                ))
        }
    }

    fn draw_windowed_rows(cx: &mut gpui::VisualTestContext) {
        for _ in 0..3 {
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
        }
        cx.run_until_parked();
    }

    gpui::actions!(windowed_rows_tab_test, [NextFocus, PreviousFocus]);

    struct WindowedRowsTabTest {
        reveal: ScrollReveal,
        scroll: ScrollHandle,
        root_focus: gpui::FocusHandle,
        reverse_focus: gpui::FocusHandle,
        row_focus: Vec<gpui::FocusHandle>,
    }

    impl gpui::Render for WindowedRowsTabTest {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut gpui::Context<Self>,
        ) -> impl IntoElement {
            let reveal = self.reveal.clone();
            let row_focus = &self.row_focus;
            let rows = windowed_track_rows(
                &self.scroll,
                &self.reveal,
                self.row_focus.len(),
                px(2.),
                px(40.),
                |index| {
                    div()
                        .h(px(40.))
                        .debug_selector(move || format!("focus-row-{index}"))
                        .child(
                            crate::ui::primitives::button::Button::plain(format!("row-{index}"))
                                .label(format!("Row {index}"))
                                .track_focus(&row_focus[index])
                                .on_click(|_, _, _| {}),
                        )
                        .child(reveal.probe_row(index))
                        .into_any_element()
                },
            );
            let reverse = crate::ui::primitives::button::Button::plain("reverse")
                .label("Reverse")
                .track_focus(&self.reverse_focus)
                .on_click(|_, _, _| {});
            div()
                .key_context("WindowedRowsTabTest")
                .track_focus(&self.root_focus)
                .on_action(|_: &NextFocus, window, cx| window.focus_next(cx))
                .on_action(|_: &PreviousFocus, window, cx| window.focus_prev(cx))
                .w(px(400.))
                .h(px(600.))
                .flex()
                .flex_col()
                .child(music_column_tracked_scrollbar(
                    div().id("windowed-focus-list").child(reverse).child(rows),
                    &self.scroll,
                    None,
                ))
        }
    }

    #[gpui::test]
    fn adr_0084_task_040_tab_moves_between_reverse_and_the_windowed_track_list(
        cx: &mut TestAppContext,
    ) {
        cx.update(gpui_component::init);
        cx.update(|cx| {
            cx.bind_keys([
                gpui::KeyBinding::new("tab", NextFocus, Some("WindowedRowsTabTest")),
                gpui::KeyBinding::new("shift-tab", PreviousFocus, Some("WindowedRowsTabTest")),
            ]);
        });
        let scroll = ScrollHandle::new();
        let scroll_handle = scroll.clone();
        let scroll_for_view = scroll.clone();
        let (view, cx) = cx.add_window_view(|_, cx| WindowedRowsTabTest {
            reveal: ScrollReveal::new(scroll_handle),
            scroll: scroll_for_view,
            root_focus: cx.focus_handle(),
            reverse_focus: cx.focus_handle(),
            row_focus: (0..491).map(|_| cx.focus_handle()).collect(),
        });
        draw_windowed_rows(cx);
        scroll.set_offset(point(px(0.), px(-16_000.)));
        draw_windowed_rows(cx);
        let reverse_focus = view.update(cx, |test, _| test.reverse_focus.clone());
        let row_focus = view.update(cx, |test, _| test.row_focus.clone());
        cx.update(|window, cx| window.focus(&reverse_focus, cx));
        cx.simulate_keystrokes("tab");
        draw_windowed_rows(cx);
        assert!(
            cx.update(|window, _| row_focus.iter().any(|focus| focus.is_focused(window))),
            "Tab from Reverse reaches a rendered track row"
        );
        cx.simulate_keystrokes("shift-tab");
        draw_windowed_rows(cx);
        assert!(
            cx.update(|window, _| reverse_focus.is_focused(window)),
            "Shift+Tab from a track row reaches Reverse"
        );
    }

    #[gpui::test]
    fn adr_0084_task_040_windowed_rows_keep_full_height_and_render_the_view(
        cx: &mut TestAppContext,
    ) {
        cx.update(gpui_component::init);
        let scroll = ScrollHandle::new();
        let scroll_handle = scroll.clone();
        let (view, cx) = cx.add_window_view(|_, _| WindowedRowsTest {
            reveal: ScrollReveal::new(scroll_handle),
            scroll: scroll.clone(),
            rendered: Rc::new(Cell::new(0)),
            top_gap: px(0.),
        });
        draw_windowed_rows(cx);

        let rendered_at_top = view.update(cx, |test, _| test.rendered.get());
        assert!(rendered_at_top <= 40, "the first paint caps rows at 40");
        assert!(rendered_at_top < 491, "the first paint skips most rows");
        assert!(
            cx.debug_bounds("windowed-row-0").is_some(),
            "the first row is in the initial window"
        );
        assert!(
            cx.debug_bounds("windowed-row-490").is_none(),
            "the last row is outside the initial window"
        );
        let full_height_offset = scroll.max_offset().y;
        assert!(
            (f32::from(full_height_offset) - (491. * 42. - 2. - 600.)).abs() < 1.,
            "the scroll range counts all 491 rows: {full_height_offset:?}"
        );

        scroll.set_offset(point(px(0.), px(-16_000.)));
        draw_windowed_rows(cx);
        let rendered_at_middle = view.update(cx, |test, _| test.rendered.get());
        assert!(
            rendered_at_middle < 50,
            "the middle window has a row margin"
        );
        assert!(
            cx.debug_bounds("windowed-row-0").is_none(),
            "the first row leaves the middle window"
        );
        assert!(
            cx.debug_bounds("windowed-row-400").is_some(),
            "row 400 is in the middle window"
        );
        assert_eq!(scroll.max_offset().y, full_height_offset);
    }

    /// The window of rows uses the position of the viewport in the
    /// content, not its position in the window.
    #[gpui::test]
    fn adr_0084_task_040_window_does_not_move_with_the_viewport_position(cx: &mut TestAppContext) {
        cx.update(gpui_component::init);
        let scroll = ScrollHandle::new();
        let scroll_handle = scroll.clone();
        let (_view, cx) = cx.add_window_view(|_, _| WindowedRowsTest {
            reveal: ScrollReveal::new(scroll_handle),
            scroll: scroll.clone(),
            rendered: Rc::new(Cell::new(0)),
            top_gap: px(300.),
        });
        draw_windowed_rows(cx);
        assert!(
            (f32::from(scroll.bounds().top()) - 300.).abs() < 1.,
            "the viewport starts 300 px down: {:?}",
            scroll.bounds()
        );
        scroll.set_offset(point(px(0.), px(-16_000.)));
        draw_windowed_rows(cx);
        // Rows 380 to 395 are in view. The margin adds 10 rows on each side.
        assert!(
            cx.debug_bounds("windowed-row-371").is_some(),
            "the top margin keeps row 371"
        );
        assert!(
            cx.debug_bounds("windowed-row-380").is_some(),
            "row 380 is at the viewport top"
        );
        assert!(
            cx.debug_bounds("windowed-row-412").is_none(),
            "the bottom margin stops before row 412"
        );
    }

    #[gpui::test]
    fn adr_0084_task_040_reveal_scrolls_to_a_row_outside_the_window(cx: &mut TestAppContext) {
        cx.update(gpui_component::init);
        let scroll = ScrollHandle::new();
        let scroll_handle = scroll.clone();
        let (view, cx) = cx.add_window_view(|_, _| WindowedRowsTest {
            reveal: ScrollReveal::new(scroll_handle),
            scroll: scroll.clone(),
            rendered: Rc::new(Cell::new(0)),
            top_gap: px(0.),
        });
        draw_windowed_rows(cx);
        view.update(cx, |test, cx| {
            test.reveal.request_row(400);
            cx.notify();
        });
        draw_windowed_rows(cx);

        let row = cx
            .debug_bounds("windowed-row-400")
            .expect("the requested row renders");
        let viewport = scroll.bounds();
        assert!(
            row.top() >= viewport.top(),
            "row 400 is below the viewport top"
        );
        assert!(
            row.bottom() <= viewport.bottom(),
            "row 400 is above the viewport bottom"
        );
    }

    /// ADR 0084 task 041: a viewport top moved down by the inset of a
    /// pinned bar scrolls a row that is below the inset into view.
    #[test]
    fn adr_0084_task_041_reveal_keeps_a_row_below_the_inset() {
        let inset = 50.0;
        let view = (100.0 + inset, 500.0);
        // The row is in the full viewport but under the bar.
        assert_eq!(
            reveal_offset_y((100.0, 500.0), (110.0, 150.0), -50.0, 1000.0),
            None
        );
        assert_eq!(
            reveal_offset_y(view, (110.0, 150.0), -50.0, 1000.0),
            Some(-50.0 + 40.0)
        );
        // A row below the inset stays.
        assert_eq!(reveal_offset_y(view, (160.0, 200.0), -50.0, 1000.0), None);
    }

    /// ADR 0084 task 041: the inset is the bar height only while the bar
    /// is pinned.
    #[test]
    fn adr_0084_task_041_pinned_inset_is_the_bar_height_only_when_pinned() {
        let pin = ListBarPin::default();
        assert!((pin.pinned_inset(-100.0, 1000.0)).abs() < f32::EPSILON);
        pin.rest_top.set(Some(120.0));
        pin.height.set(Some(50.0));
        // At offset -100 the bar is 20 px below the top of the viewport.
        assert!((pin.pinned_inset(-100.0, 1000.0)).abs() < f32::EPSILON);
        assert!((pin.pinned_inset(-121.0, 1000.0) - 50.0).abs() < f32::EPSILON);
        // A page that does not scroll never pins.
        assert!((pin.pinned_inset(-121.0, 0.0)).abs() < f32::EPSILON);
        // A clone shares the cells.
        let shared = pin.clone();
        pin.height.set(Some(60.0));
        assert!((shared.pinned_inset(-121.0, 1000.0) - 60.0).abs() < f32::EPSILON);
        // After a paint of the pinned copy, its own height wins.
        pin.pinned_height.set(Some(84.0));
        assert!((shared.pinned_inset(-121.0, 1000.0) - 84.0).abs() < f32::EPSILON);
    }

    struct PinnedRowsTest {
        reveal: ScrollReveal,
        scroll: ScrollHandle,
    }

    impl gpui::Render for PinnedRowsTest {
        fn render(
            &mut self,
            _window: &mut gpui::Window,
            _cx: &mut gpui::Context<Self>,
        ) -> impl IntoElement {
            let reveal = self.reveal.clone();
            let rows = windowed_track_rows(&self.scroll, &self.reveal, 491, px(2.), px(40.), {
                move |index| {
                    let selector = format!("pinned-row-{index}");
                    div()
                        .h(px(40.))
                        .debug_selector(move || selector.clone())
                        .child(reveal.probe_row(index))
                        .into_any_element()
                }
            });
            div()
                .w(px(400.))
                .h(px(600.))
                .flex()
                .flex_col()
                .child(music_column_tracked_scrollbar(
                    div()
                        .id("pinned-rows")
                        .child(div().h(px(50.)).w_full())
                        .child(rows),
                    &self.scroll,
                    None,
                ))
        }
    }

    /// ADR 0084 task 041, defect E: a reveal of row 0 from a scrolled
    /// position puts the top of the row at or below the bottom of the
    /// pinned bar, 50 px high, at the top of the viewport.
    #[gpui::test]
    fn adr_0084_task_041_reveal_puts_the_row_below_the_pinned_bar(cx: &mut TestAppContext) {
        cx.update(gpui_component::init);
        let scroll = ScrollHandle::new();
        let pin = ListBarPin::default();
        // The in-flow bar is the first 50 px of the content.
        pin.rest_top.set(Some(0.0));
        pin.height.set(Some(50.0));
        let reveal = ScrollReveal::new(scroll.clone()).below_pin(pin);
        let (view, cx) = cx.add_window_view(|_, _| PinnedRowsTest {
            reveal,
            scroll: scroll.clone(),
        });
        draw_windowed_rows(cx);
        scroll.set_offset(point(px(0.), px(-16_000.)));
        draw_windowed_rows(cx);
        view.update(cx, |test, cx| {
            test.reveal.request_row(0);
            cx.notify();
        });
        draw_windowed_rows(cx);
        draw_windowed_rows(cx);

        let row = cx
            .debug_bounds("pinned-row-0")
            .expect("the requested row renders");
        let viewport = scroll.bounds();
        assert!(
            row.top() >= viewport.top() + px(50.) - px(0.5),
            "row 0 is below the pinned bar: row {:?}, viewport {:?}, offset {:?}",
            row,
            viewport,
            scroll.offset()
        );
    }

    /// R37-07 (ADR 0084 task 037, decision D1): a virtual list of 1,000
    /// rows in a column 600 px high renders row 1 and does not render row
    /// 1,000.
    #[gpui::test]
    fn adr_0084_task_037_r37_07_list_renders_only_the_rows_in_view(cx: &mut TestAppContext) {
        cx.update(gpui_component::init);
        let (view, cx) = cx.add_window_view(|_, cx| VirtualListTest {
            list: MusicColumnList::new(cx),
            ids: row_ids(1_000),
        });
        // The first layout gives the list its width. The next frame gives
        // each item that did not render its item height again.
        for _ in 0..2 {
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
        }
        assert!(cx.debug_bounds("r37-row-1").is_some(), "row 1 is in view");
        assert!(
            cx.debug_bounds("r37-row-1000").is_none(),
            "row 1,000 is out of view and renders nothing"
        );
        // A row that did not render counts the item height, so the
        // scrollbar thumb has its size for the full list (V5).
        let max_offset = view.update(cx, |test, _| test.list.state().max_offset_for_scrollbar().y);
        assert!(
            max_offset > px(30_000.),
            "the full list counts: {max_offset:?}"
        );
    }

    /// R37-08 (ADR 0084 task 037, Design step 8, packet 036 option A):
    /// with the list scrolled to row 500, a sync that adds one row above
    /// row 500 keeps row 500 as the logical scroll top.
    #[gpui::test]
    fn adr_0084_task_037_r37_08_a_row_above_keeps_the_scroll_top(cx: &mut TestAppContext) {
        cx.update(gpui_component::init);
        let (view, cx) = cx.add_window_view(|_, cx| VirtualListTest {
            list: MusicColumnList::new(cx),
            ids: row_ids(1_000),
        });
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        view.update(cx, |test, cx| {
            test.list.state().scroll_to(ListOffset {
                item_ix: 499,
                offset_in_item: px(0.),
            });
            cx.notify();
        });
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let top = cx.debug_bounds("r37-row-500").expect("row 500 is in view");

        view.update(cx, |test, cx| {
            test.ids.insert(10, "r37-row-added".to_owned());
            cx.notify();
        });
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let scroll_top = view.update(cx, |test, _| test.list.state().logical_scroll_top());
        assert_eq!(
            scroll_top.item_ix, 500,
            "row 500 is now item 500 and stays at the top"
        );
        assert_eq!(scroll_top.offset_in_item, px(0.));
        assert_eq!(
            cx.debug_bounds("r37-row-500").map(|bounds| bounds.top()),
            Some(top.top()),
            "row 500 does not move"
        );
    }

    /// R18-09: the focused row scrolls into view by the smallest distance,
    /// and a row that is in view does not scroll.
    #[test]
    fn adr_0084_detail_selection_r18_09_reveal_scrolls_the_smallest_distance() {
        let view = (100.0, 500.0);
        assert_eq!(reveal_offset_y(view, (200.0, 240.0), -50.0, 1000.0), None);
        assert_eq!(
            reveal_offset_y(view, (520.0, 560.0), -50.0, 1000.0),
            Some(-110.0)
        );
        assert_eq!(
            reveal_offset_y(view, (60.0, 100.0), -50.0, 1000.0),
            Some(-10.0)
        );
    }
}
