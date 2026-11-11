//! The field list under the toolbar search field (ADR 0087 Decision 3).
//!
//! The list draws over the other content with `deferred` and `anchored`. It
//! has no focus handle, so the keyboard focus stays in the search field.
//! The search field view captures the list keys. This composite only draws
//! the surface, the notices and the rows of [`FieldListVm`]. Each id, text
//! and label comes from the view model. No text calls `truncate()` (ADR
//! 0063). Long text wraps.
//!
//! Each row carries `flex_shrink_0()`, so it keeps the height of its two
//! lines. The row list has a maximum height and scrolls, with a scroll bar
//! (ADR 0087 Decision 3, ADR 0076 Decision 8). A row that shrank would paint
//! its second line over the next row.

#![warn(clippy::pedantic)]

use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    anchored, canvas, deferred, div, prelude::*, AnyElement, App, Bounds, MouseButton, Pixels,
    ScrollHandle, SharedString, Window,
};
use gpui_component::scroll::ScrollableElement;

use crate::ui::primitives::status_badge::render_row_badge;
use crate::ui::primitives::surface::{Surface, SurfaceElevation};
use crate::ui::tokens::{color, FontSize, Radius, SemanticColor, Size, Spacing, Weight};
use crate::view_models::search_field_list::{
    FieldExampleKind, FieldListNoticeKind, FieldListNoticeVm, FieldListRowVm, FieldListVm,
};

/// The handler of a mouse down on a row. It gets the row index.
type RowHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// The handler of a mouse down on an example text. It gets the row index and
/// the example index.
type ExampleHandler = Rc<dyn Fn(usize, usize, &mut Window, &mut App)>;

/// The field list under the toolbar search field.
#[derive(IntoElement)]
#[must_use]
pub(crate) struct SearchFieldList {
    vm: FieldListVm,
    scroll: ScrollHandle,
    on_row: Option<RowHandler>,
    on_example: Option<ExampleHandler>,
    bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
}

impl SearchFieldList {
    /// Makes the list for the view model. `scroll` scrolls the rows. The
    /// list writes its drawn area to `bounds`, so the view can tell a click
    /// in the list from a click outside it.
    pub(crate) fn new(
        vm: FieldListVm,
        scroll: ScrollHandle,
        bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    ) -> Self {
        Self {
            vm,
            scroll,
            on_row: None,
            on_example: None,
            bounds,
        }
    }

    /// Sets the handler of a left mouse down on an example text that has its
    /// own click.
    pub(crate) fn on_example(
        mut self,
        handler: impl Fn(usize, usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_example = Some(Rc::new(handler));
        self
    }

    /// Sets the handler of a left mouse down on a row.
    pub(crate) fn on_row(
        mut self,
        handler: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_row = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for SearchFieldList {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let Self {
            vm,
            scroll,
            on_row,
            on_example,
            bounds,
        } = self;
        let notices: Vec<AnyElement> = vm
            .notices
            .into_iter()
            .map(|notice| render_notice(notice, cx))
            .collect();
        let rows: Vec<AnyElement> = vm
            .rows
            .into_iter()
            .enumerate()
            .map(|(index, row)| render_row(index, row, on_row.clone(), on_example.as_ref(), cx))
            .collect();
        let row_list = div()
            .id(FieldListVm::LIST_ID)
            .debug_selector(|| FieldListVm::LIST_ID.to_owned())
            .flex()
            .flex_col()
            .max_h(Size::ColumnRegular.scaled(cx))
            .overflow_y_scroll()
            .track_scroll(&scroll)
            .children(rows);
        // The scroll bar sits on the parent of the scrolling row list, as
        // `music_column_tracked_scrollbar` does for a Music column.
        let row_scroll = div()
            .relative()
            .min_w_0()
            .child(row_list)
            .vertical_scrollbar(&scroll);
        let surface = div()
            .relative()
            .flex()
            .flex_col()
            .occlude()
            .w(Size::ColumnTall.scaled(cx))
            .mt(Spacing::XS.scaled(cx))
            .child(
                canvas(move |area, _, _| bounds.set(Some(area)), |_, (), _, _| {})
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full(),
            )
            .child(
                Surface::new(SurfaceElevation::Raised)
                    .padding(Spacing::XS)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .min_w_0()
                            .children(notices)
                            .child(row_scroll),
                    ),
            );
        div()
            .absolute()
            .left_0()
            .bottom_0()
            .size_0()
            .child(deferred(
                anchored()
                    .snap_to_window_with_margin(Spacing::SM.scaled(cx))
                    .child(surface),
            ))
    }
}

/// Draws one notice. The text names the problem, so the color is not the
/// only cue.
fn render_notice(notice: FieldListNoticeVm, cx: &App) -> AnyElement {
    let text_color = match notice.kind {
        FieldListNoticeKind::FieldError => SemanticColor::WarningLabel,
        FieldListNoticeKind::ScopeConflict => SemanticColor::SecondaryLabel,
    };
    div()
        .min_w_0()
        .px(Spacing::SM.scaled(cx))
        .py(Spacing::XS.scaled(cx))
        .text_size(FontSize::Caption.scaled(cx))
        .text_color(color(cx, text_color))
        .child(SharedString::from(notice.text))
        .into_any_element()
}

/// Draws one row. The selected row has a fill and a border. An unavailable
/// row draws its label in a tertiary color, and its note gives the reason.
fn render_row(
    index: usize,
    row: FieldListRowVm,
    on_row: Option<RowHandler>,
    on_example: Option<&ExampleHandler>,
    cx: &App,
) -> AnyElement {
    let available = row.availability.is_available();
    let label_color = if available {
        SemanticColor::Label
    } else {
        SemanticColor::TertiaryLabel
    };
    let border_color = if row.selected {
        SemanticColor::Focus
    } else {
        SemanticColor::SecondarySystemBackground
    };
    let title_line = div()
        .flex()
        .flex_wrap()
        .items_center()
        .gap(Spacing::XS.scaled(cx))
        .min_w_0()
        .child(
            div()
                .min_w_0()
                .text_size(FontSize::Body.scaled(cx))
                .font_weight(Weight::Semibold.into())
                .text_color(color(cx, label_color))
                .child(SharedString::from(row.label)),
        )
        .when_some(row.mark, |line, mark| {
            line.child(render_row_badge(mark.label(), cx))
        });
    let detail_line = div()
        .flex()
        .flex_wrap()
        .gap(Spacing::MD.scaled(cx))
        .min_w_0()
        .when_some(row.values_text, |line, text| {
            line.child(caption(text, SemanticColor::SecondaryLabel, cx))
        })
        .children(
            row.examples
                .into_iter()
                .enumerate()
                .map(|(position, example)| {
                    let text = caption(example.text, example_color(example.kind), cx);
                    if example.click.is_none() {
                        return text;
                    }
                    let handler = on_example.cloned();
                    let id = SharedString::from(example.element_id);
                    div()
                        .id(id.clone())
                        .debug_selector(move || id.to_string())
                        .min_w_0()
                        .cursor_pointer()
                        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            cx.stop_propagation();
                            if let Some(handler) = &handler {
                                handler(index, position, window, cx);
                            }
                        })
                        .child(text)
                        .into_any_element()
                }),
        )
        .when_some(row.note, |line, text| {
            line.child(caption(text, SemanticColor::SecondaryLabel, cx))
        });
    let element_id = row.element_id;
    div()
        .id(SharedString::from(element_id.clone()))
        .debug_selector(move || element_id)
        .flex()
        .flex_col()
        .flex_shrink_0()
        .justify_center()
        .gap(Spacing::XXS.scaled(cx))
        .min_w_0()
        .min_h(Size::RowMd.scaled(cx))
        .px(Spacing::SM.scaled(cx))
        .py(Spacing::XS.scaled(cx))
        .rounded(Radius::MD.scaled(cx))
        .border_1()
        .border_color(color(cx, border_color))
        .when(row.selected, |el| {
            el.bg(color(cx, SemanticColor::SelectedContent))
        })
        .when(available, gpui::Styled::cursor_pointer)
        .on_mouse_down(MouseButton::Left, move |_, window, cx| {
            cx.stop_propagation();
            if let Some(handler) = &on_row {
                handler(index, window, cx);
            }
        })
        .child(title_line)
        .child(detail_line)
        .into_any_element()
}

/// The text color of an example. A placeholder is muted, as the placeholder
/// of the search field (musicindex ADR 0058 R-QUERY-11 item 2).
fn example_color(kind: FieldExampleKind) -> SemanticColor {
    match kind {
        FieldExampleKind::Placeholder | FieldExampleKind::Fixed => SemanticColor::TertiaryLabel,
        FieldExampleKind::Runnable => SemanticColor::SecondaryLabel,
    }
}

/// One caption text of a row.
fn caption(text: String, text_color: SemanticColor, cx: &App) -> AnyElement {
    div()
        .min_w_0()
        .text_size(FontSize::Caption.scaled(cx))
        .text_color(color(cx, text_color))
        .child(SharedString::from(text))
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use gpui::{px, Bounds, Context, Pixels, Render, TestAppContext, VisualTestContext, Window};

    use super::*;
    use crate::view_models::search_field_list::{FieldListOpening, FieldListState};
    use crate::view_models::search_field_values::FieldValueRows;

    #[test]
    fn adr_0058_placeholder_example_is_muted() {
        assert_eq!(
            example_color(FieldExampleKind::Placeholder),
            SemanticColor::TertiaryLabel
        );
        assert_ne!(
            example_color(FieldExampleKind::Runnable),
            example_color(FieldExampleKind::Placeholder)
        );
    }

    /// Shows one field list, so that a test can draw it again.
    struct FieldListTest {
        vm: FieldListVm,
    }

    impl Render for FieldListTest {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().size_full().child(SearchFieldList::new(
                self.vm.clone(),
                ScrollHandle::new(),
                Rc::default(),
            ))
        }
    }

    /// Leaks a selector name so a name built at test time satisfies
    /// `VisualTestContext::debug_bounds`'s `'static` key. The test process
    /// ends after the test, so the leak carries no lasting cost.
    fn leaked(name: &str) -> &'static str {
        Box::leak(name.to_owned().into_boxed_str())
    }

    /// The field list of the empty text, open with the focus.
    fn open_list(text: &str) -> FieldListVm {
        let state = FieldListState {
            opening: FieldListOpening::Open,
            focused: true,
            selected: None,
            visit: 0,
        };
        FieldListVm::build(text, text.len()..text.len(), &state, &FieldValueRows::EMPTY)
            .expect("the list shows")
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
    }

    fn row_bounds(cx: &mut VisualTestContext, element_id: &str) -> Bounds<Pixels> {
        cx.debug_bounds(leaked(element_id))
            .unwrap_or_else(|| panic!("row {element_id} did not render"))
    }

    /// Each row starts at or below the bottom of the row above it. A row
    /// that shrank paints its second line over the next row, so the check
    /// fails for that row.
    fn assert_rows_do_not_overlap(cx: &mut VisualTestContext, rows: &[FieldListRowVm]) {
        let bounds: Vec<Bounds<Pixels>> = rows
            .iter()
            .map(|row| row_bounds(cx, &row.element_id))
            .collect();
        for (index, pair) in bounds.windows(2).enumerate() {
            let (above, below) = (pair[0], pair[1]);
            assert!(
                below.origin.y + px(0.5) >= above.bottom(),
                "ADR 0087 Decision 3 (each row gives one field, its values and one example): row \"{}\" is {:?} high and overlaps the next row. A row in the scrolling field list must keep its content height. Keep `.flex_shrink_0()` on the row element in render_row of src/ui/composites/search_field_list.rs.",
                rows[index].label,
                above.size.height,
            );
        }
    }

    /// R87-10-01 (ADR 0087 Decision 3, ADR 0076 Decision 8): each row of the
    /// 12 fields keeps the height of its own content. The rows do not
    /// overlap, and `artist:` has the same height as in a list of one row.
    #[gpui::test]
    fn adr_0087_field_list_r87_10_01_rows_keep_their_height(cx: &mut TestAppContext) {
        cx.update(gpui_component::init);
        let full = open_list("");
        assert_eq!(full.rows.len(), 12, "the empty text shows the 12 fields");
        let artist = full
            .rows
            .iter()
            .find(|row| row.label == "artist:")
            .expect("the field artist: shows")
            .clone();
        let one_row = FieldListVm {
            rows: vec![artist.clone()],
            notices: Vec::new(),
            selected: None,
            a11y_label: String::new(),
        };
        let (view, cx) = cx.add_window_view(|_, _| FieldListTest { vm: one_row });
        draw(cx);
        let one_height = row_bounds(cx, &artist.element_id).size.height;

        view.update(cx, |view, cx| {
            view.vm = full.clone();
            cx.notify();
        });
        draw(cx);
        let height = row_bounds(cx, &artist.element_id).size.height;
        assert_eq!(
            height, one_height,
            "ADR 0087 Decision 3 (each row gives one field, its values and one example): row \"artist:\" is {height:?} high in the list and {one_height:?} high alone. Keep `.flex_shrink_0()` on the row element in render_row of src/ui/composites/search_field_list.rs.",
        );
        assert_rows_do_not_overlap(cx, &full.rows);
    }

    /// R87-10-03 (ADR 0087 Decision 3): 30 value rows keep the height of one
    /// row, and no two rows overlap.
    #[gpui::test]
    fn adr_0087_field_list_r87_10_03_many_value_rows_keep_their_height(cx: &mut TestAppContext) {
        cx.update(gpui_component::init);
        let template = open_list("")
            .rows
            .into_iter()
            .next()
            .expect("the field list has a row");
        let rows: Vec<FieldListRowVm> = (0..30)
            .map(|index| FieldListRowVm {
                element_id: format!("search-field-list-value-test-{index}"),
                label: format!("value{index}"),
                ..template.clone()
            })
            .collect();
        let one_row = FieldListVm {
            rows: vec![rows[0].clone()],
            notices: Vec::new(),
            selected: None,
            a11y_label: String::new(),
        };
        let (view, cx) = cx.add_window_view(|_, _| FieldListTest { vm: one_row });
        draw(cx);
        let one_height = row_bounds(cx, &rows[0].element_id).size.height;

        view.update(cx, |view, cx| {
            view.vm = FieldListVm {
                rows: rows.clone(),
                notices: Vec::new(),
                selected: None,
                a11y_label: String::new(),
            };
            cx.notify();
        });
        draw(cx);
        for row in &rows {
            let height = row_bounds(cx, &row.element_id).size.height;
            assert_eq!(
                height, one_height,
                "ADR 0087 Decision 3 (each row gives one field, its values and one example): row \"{}\" is {:?} high in a list of 30 and {:?} high alone. Keep `.flex_shrink_0()` on the row element in render_row of src/ui/composites/search_field_list.rs.",
                row.label,
                height,
                one_height
            );
        }
        assert_rows_do_not_overlap(cx, &rows);
    }
}
