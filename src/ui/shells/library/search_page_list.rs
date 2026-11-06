//! The Search page list column (ADR 0089 Decision 4). It shows the "New
//! Saved Search" button, then one row for each saved search, then "How to
//! Search" with its steps and example queries.
//!
//! The view model gives each label, each menu item and each action. This
//! shell owns the layout and the click wiring. A click on the new-search
//! button sends a `SearchPageAction` to the top app, which owns the toolbar
//! field. The item menu of a row acts on the Library app, except the delete
//! item, which the top app confirms.

#![warn(clippy::pedantic)]

use gpui::{
    div, prelude::*, AnyElement, ClickEvent, Context, Entity, FontWeight, SharedString, Styled,
};
use gpui_component::input::{Input, InputState};
use gpui_component::Size;

use crate::library::{LibraryApp, LibraryAppEvent};
use crate::ui::composites::music_column_scroll::music_column_tracked_scrollbar;
use crate::ui::composites::page_scroll_content::page_scroll_content;
use crate::ui::composites::{ListRow, ListRowA11yLabel};
use crate::ui::control_styles::ControlStyle;
use crate::ui::icons::{Icon, IconName, IconSize};
use crate::ui::primitives::primary_selection::PrimarySelectionExt as _;
use crate::ui::primitives::{
    Button as UiButton, ContextMenu, ContextMenuItem, ContextMenuItemDisplay, ContextMenuScope,
    Label,
};
use crate::ui::sizable_bridge::SizableScaled;
use crate::ui::style::spacing;
use crate::ui::tokens::{FontSize, SemanticColor};
use crate::view_models::search_page::{
    HowToSearchDisclosure, SearchPageAction, SearchPageRowVm, SearchPageVm,
};
use crate::view_models::search_results::{
    SavedSearch, SavedSearchId, SavedSearchMenuItem, SearchPageActionDisplay,
};

/// The rename state of the saved search rows: the row that shows the input,
/// and the input itself.
pub(crate) struct SearchPageListRename<'a> {
    pub(crate) renaming: Option<SavedSearchId>,
    pub(crate) input: &'a Entity<InputState>,
}

/// Renders the Search page list column (ADR 0089 Decision 4).
///
/// The steps and examples show when at least one saved search is listed.
/// With no saved search the steps always show.
pub(crate) fn render_search_page_list(
    vm: &SearchPageVm,
    how_to_search: HowToSearchDisclosure,
    rename: &SearchPageListRename<'_>,
    scroll: &gpui::ScrollHandle,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let mut content = page_scroll_content(cx)
        .gap(spacing::XXS)
        .child(new_saved_search_button(vm.new_action(), cx));
    if let Some(message) = vm.message() {
        content = content.child(
            Label::new(message.to_owned())
                .size(FontSize::Micro)
                .color(SemanticColor::TertiaryLabel),
        );
    }
    content = content.children(
        vm.rows()
            .iter()
            .map(|row| render_saved_search_row(row, rename, cx)),
    );
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

/// "New Saved Search", a button from its typed action state (ADR 0089
/// Decision 4).
fn new_saved_search_button(
    action: &SearchPageActionDisplay,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    UiButton::styled(SharedString::from(action.id), ControlStyle::Primary)
        .label(action.label)
        .a11y_label(action.a11y_label.clone())
        .disabled(!action.available)
        .on_click(cx.listener(|_this, _: &ClickEvent, _window, cx| {
            cx.emit(LibraryAppEvent::SearchPage(
                SearchPageAction::NewSavedSearch,
            ));
        }))
        .into_any_element()
}

fn render_saved_search_row(
    row: &SearchPageRowVm,
    rename: &SearchPageListRename<'_>,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    if rename.renaming == Some(row.search.id) {
        return render_rename_row(row, rename.input, cx);
    }
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
        .trailing(saved_search_row_menu(row, cx))
        .into_any_element()
}

/// The item menu of a saved search row, in the sequence of ADR 0089
/// Decision 4. An item acts on the Library app.
fn saved_search_row_menu(row: &SearchPageRowVm, cx: &mut Context<LibraryApp>) -> AnyElement {
    let entity = cx.entity();
    let search = row.search.clone();
    let mut menu = ContextMenu::new(
        SharedString::from(format!("{}-menu", row.element_id)),
        ContextMenuScope::WorkspaceFrame,
        SharedString::from(format!("Actions for {}", row.label)),
    )
    .trigger_icon(IconName::More);
    for entry in &row.menu {
        let item = entry.item;
        let entity = entity.clone();
        let search = search.clone();
        menu = menu.item(
            ContextMenuItem::new(ContextMenuItemDisplay {
                id: SharedString::from(entry.id),
                label: SharedString::from(entry.label),
                a11y_label: SharedString::from(entry.a11y_label.clone()),
                destructive: entry.destructive,
                disabled: !entry.available,
            })
            .on_select(move |window, cx| {
                entity.update(cx, |this, cx| {
                    run_saved_search_menu_item(this, item, &search, window, cx);
                });
            }),
        );
    }
    menu.into_any_element()
}

fn run_saved_search_menu_item(
    this: &mut LibraryApp,
    item: SavedSearchMenuItem,
    search: &SavedSearch,
    window: &mut gpui::Window,
    cx: &mut Context<LibraryApp>,
) {
    match item {
        SavedSearchMenuItem::EditQuery => this.open_saved_search_row(search, cx),
        SavedSearchMenuItem::Rename => this.begin_saved_search_rename(search, window, cx),
        SavedSearchMenuItem::Duplicate => this.duplicate_saved_search_row(search.id, cx),
        SavedSearchMenuItem::Delete => cx.emit(LibraryAppEvent::SearchPage(
            SearchPageAction::DeleteSavedSearch(search.clone()),
        )),
    }
}

/// The row of a saved search while it is renamed: the input, "Save" and
/// "Cancel" (ADR 0089 Decision 4).
fn render_rename_row(
    row: &SearchPageRowVm,
    input: &Entity<InputState>,
    cx: &mut Context<LibraryApp>,
) -> AnyElement {
    let rename = &row.rename;
    div()
        .id(SharedString::from(format!("{}-rename", row.element_id)))
        .flex()
        .flex_row()
        .items_center()
        .gap(spacing::SM)
        .px(spacing::MD)
        .child(
            div().flex_1().min_w_0().child(
                Input::new(input)
                    .cleanable(false)
                    .scaled(Size::Small, cx)
                    .with_primary_selection(input),
            ),
        )
        .child(
            UiButton::styled(SharedString::from(rename.save_id), ControlStyle::Primary)
                .label(rename.save_label)
                .a11y_label(rename.save_a11y_label.clone())
                .on_click(cx.listener(|this, _: &ClickEvent, _window, cx| {
                    this.submit_saved_search_rename(cx);
                })),
        )
        .child(
            UiButton::styled(SharedString::from(rename.cancel_id), ControlStyle::Primary)
                .label(rename.cancel_label)
                .a11y_label(rename.cancel_a11y_label.clone())
                .on_click(cx.listener(|this, _: &ClickEvent, _window, cx| {
                    this.cancel_saved_search_rename(cx);
                })),
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
