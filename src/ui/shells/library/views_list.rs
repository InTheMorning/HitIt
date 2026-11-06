//! The Discover views list column (ADR 0084 Decision 3): "New releases",
//! then one row for each saved search (packet 012).
//!
//! Moved out of `destination_lists.rs` to keep that file under the ADR
//! 0038 Task 007 line ceiling. The view model gives each label; this shell
//! owns only layout and interaction wiring.

#![warn(clippy::pedantic)]

use gpui::{div, prelude::*, AnyElement, ClickEvent, Context, FontWeight, SharedString, Styled};

use crate::library::LibraryApp;
use crate::ui::composites::music_column_scroll::music_column_tracked_scrollbar;
use crate::ui::composites::page_scroll_content::page_scroll_content;
use crate::ui::composites::{ListRow, ListRowA11yLabel};
use crate::ui::icons::{Icon, IconName, IconSize};
use crate::ui::primitives::Label;
use crate::ui::style::spacing;
use crate::ui::tokens::{FontSize, SemanticColor};
use crate::view_models::library::{DiscoverListIcon, DiscoverListRowTarget, DiscoverListRowVm};

/// Maps the semantic icon of a Discover view row to a named icon.
const fn icon_name(icon: DiscoverListIcon) -> IconName {
    match icon {
        DiscoverListIcon::SavedSearch => IconName::Search,
    }
}

/// Renders the Discover list column (ADR 0084 Decision 3): the saved
/// searches. The saved searches list goes to packet 006 (ADR 0089).
pub(crate) fn render_library_discover_list(
    rows: &[DiscoverListRowVm],
    scroll: &gpui::ScrollHandle,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let container = div()
        .id("music-discover-views-list")
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .min_w_0()
        .px(spacing::MD)
        .pb(spacing::MD)
        .child(
            page_scroll_content(cx)
                .gap(spacing::XXS)
                .children(rows.iter().map(|row| render_discover_list_row(row, cx))),
        );
    music_column_tracked_scrollbar(container, scroll, None)
}

fn render_discover_list_row(row: &DiscoverListRowVm, cx: &mut Context<LibraryApp>) -> AnyElement {
    let target = row.target.clone();

    ListRow::new(SharedString::from(row.element_id.clone()))
        .a11y_label(ListRowA11yLabel {
            label: SharedString::from(row.a11y_label.clone()),
        })
        .selected(row.selected)
        .on_click(
            cx.listener(move |this, _: &ClickEvent, _window, cx| match &target {
                DiscoverListRowTarget::SavedSearch(search) => {
                    this.open_saved_search_row(search, cx);
                }
            }),
        )
        .child(
            div()
                .flex_shrink_0()
                .pl(spacing::MD)
                .child(Icon::new(icon_name(row.icon)).size(IconSize::Action)),
        )
        .child(
            // ADR 0063: stacked column text calls no `truncate()`. The
            // container clips the text with `overflow_hidden()`.
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .pl(spacing::SM)
                .child(
                    Label::new(row.label.clone())
                        .color(if row.selected {
                            SemanticColor::Accent
                        } else {
                            SemanticColor::Label
                        })
                        .weight(FontWeight::MEDIUM),
                )
                .child(
                    Label::new(row.subtitle.clone())
                        .size(FontSize::Micro)
                        .color(SemanticColor::TertiaryLabel),
                ),
        )
        .into_any_element()
}
