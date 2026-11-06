//! The Search page list column (ADR 0089 Decision 4). It shows "New Saved
//! Search", then one row for each saved search, then "How to Search" with
//! its steps and example queries.
//!
//! The view model gives each label and each action. This shell owns the
//! layout and the click wiring. A click sends a `SearchPageAction` to the
//! top app, which owns the toolbar field.

#![warn(clippy::pedantic)]

use gpui::{div, prelude::*, AnyElement, ClickEvent, Context, FontWeight, SharedString, Styled};

use crate::library::{LibraryApp, LibraryAppEvent};
use crate::ui::composites::music_column_scroll::music_column_tracked_scrollbar;
use crate::ui::composites::page_scroll_content::page_scroll_content;
use crate::ui::composites::{ListRow, ListRowA11yLabel};
use crate::ui::icons::{Icon, IconName, IconSize};
use crate::ui::primitives::Label;
use crate::ui::style::spacing;
use crate::ui::tokens::{FontSize, SemanticColor};
use crate::view_models::search_page::{
    HowToSearchDisclosure, SearchPageAction, SearchPageRowVm, SearchPageVm,
};

/// Renders the Search page list column (ADR 0089 Decision 4).
///
/// `how_to_search_open` shows the steps and examples when at least one
/// saved search is listed. With no saved search the steps always show.
pub(crate) fn render_search_page_list(
    vm: &SearchPageVm,
    how_to_search: HowToSearchDisclosure,
    scroll: &gpui::ScrollHandle,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let mut content = page_scroll_content(cx)
        .gap(spacing::XXS)
        .child(new_saved_search_row(cx));
    content = content.children(vm.rows().iter().map(|row| render_saved_search_row(row, cx)));
    let shown = how_to_search == HowToSearchDisclosure::Shown;
    if vm.steps_show_first() {
        content = content.child(render_how_to_search(vm, cx));
    } else {
        content = content.child(how_to_search_disclosure_row(shown, cx));
        if shown {
            content = content.child(render_how_to_search(vm, cx));
        }
    }
    let container = div()
        .id("music-search-page-list")
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .min_w_0()
        .px(spacing::MD)
        .pb(spacing::MD)
        .child(content);
    music_column_tracked_scrollbar(container, scroll, None)
}

fn new_saved_search_row(cx: &mut Context<LibraryApp>) -> AnyElement {
    ListRow::new(SharedString::from("music-search-new-saved-search"))
        .a11y_label(ListRowA11yLabel {
            label: SharedString::from("New Saved Search. Puts the focus in the search field."),
        })
        .on_click(cx.listener(|_this, _: &ClickEvent, _window, cx| {
            cx.emit(LibraryAppEvent::SearchPage(
                SearchPageAction::NewSavedSearch,
            ));
        }))
        .child(
            div()
                .flex_shrink_0()
                .pl(spacing::MD)
                .child(Icon::new(IconName::Search).size(IconSize::Action)),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .pl(spacing::SM)
                .child(
                    Label::new("New Saved Search")
                        .color(SemanticColor::Label)
                        .weight(FontWeight::MEDIUM),
                ),
        )
        .into_any_element()
}

fn render_saved_search_row(row: &SearchPageRowVm, cx: &mut Context<LibraryApp>) -> AnyElement {
    let search = row.search.clone();
    ListRow::new(SharedString::from(row.element_id.clone()))
        .a11y_label(ListRowA11yLabel {
            label: SharedString::from(row.a11y_label.clone()),
        })
        .selected(row.selected)
        .on_click(cx.listener(move |this, _: &ClickEvent, _window, cx| {
            this.open_saved_search_row(&search, cx);
        }))
        .child(
            div()
                .flex_shrink_0()
                .pl(spacing::MD)
                .child(Icon::new(IconName::Search).size(IconSize::Action)),
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
                    Label::new(row.subtitle)
                        .size(FontSize::Micro)
                        .color(SemanticColor::TertiaryLabel),
                ),
        )
        .into_any_element()
}

fn how_to_search_disclosure_row(open: bool, cx: &mut Context<LibraryApp>) -> AnyElement {
    ListRow::new(SharedString::from("music-search-how-to-search"))
        .a11y_label(ListRowA11yLabel {
            label: SharedString::from(if open {
                "How to Search. Hides the steps."
            } else {
                "How to Search. Shows the steps."
            }),
        })
        .on_click(cx.listener(|this, _: &ClickEvent, _window, cx| {
            this.toggle_how_to_search(cx);
        }))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .pl(spacing::MD)
                .child(
                    Label::new("How to Search")
                        .color(SemanticColor::Label)
                        .weight(FontWeight::MEDIUM),
                ),
        )
        .into_any_element()
}

fn render_how_to_search(vm: &SearchPageVm, cx: &mut Context<LibraryApp>) -> AnyElement {
    let steps = SearchPageVm::steps()
        .iter()
        .enumerate()
        .map(|(index, step)| {
            Label::new(format!("{}. {step}", index + 1))
                .size(FontSize::Micro)
                .color(SemanticColor::TertiaryLabel)
        });
    let examples = vm.examples().iter().map(|example| {
        let query = example.query;
        ListRow::new(SharedString::from(example.element_id.clone()))
            .a11y_label(ListRowA11yLabel {
                label: SharedString::from(example.a11y_label.clone()),
            })
            .on_click(cx.listener(move |_this, _: &ClickEvent, _window, cx| {
                cx.emit(LibraryAppEvent::SearchPage(SearchPageAction::RunQuery(
                    query.to_owned(),
                )));
            }))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .pl(spacing::MD)
                    .child(
                        Label::new(query)
                            .color(SemanticColor::Label)
                            .weight(FontWeight::MEDIUM),
                    ),
            )
            .into_any_element()
    });
    div()
        .id("music-search-how-to-search-body")
        .flex()
        .flex_col()
        .gap(spacing::XXS)
        .pl(spacing::MD)
        .children(steps.map(IntoElement::into_any_element))
        .children(examples)
        .into_any_element()
}
