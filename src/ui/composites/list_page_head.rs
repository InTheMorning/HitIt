//! The title row of a Music list page (ADR 0089 Decision 6, musicindex ADR
//! 0027 R-LIST-1 item 4).
//!
//! The title is at the left. The sort menu, the view icons and the Select
//! action are at the right. The scope control sits below the title, at the
//! left, and the letter bar sits below the scope control. The view model
//! gives each text, each id and each availability fact. This composite only
//! lays them out.
//!
//! ADR 0089 Decision 6 names this row as the one owner of the list page
//! head. A list page renders it above its rows.

#![warn(clippy::pedantic)]

use std::rc::Rc;

use gpui::{
    div, prelude::*, AnyElement, App, FontWeight, IntoElement, ParentElement, SharedString, Styled,
    Window,
};

use crate::ui::composites::letter_bar::{letter_bar, LetterHandler};
use crate::ui::composites::{Segment, SegmentDisplay, SegmentedControl};
use crate::ui::control_styles::ControlStyle;
use crate::ui::icons::IconName;
use crate::ui::primitives::{
    Button, ContextMenu, ContextMenuItem, ContextMenuItemDisplay, ContextMenuScope,
};
use crate::ui::tokens::{FontSize, Spacing};
use crate::view_models::list_page::{
    ListPageHeadVm, ListScope, ListScopeControlVm, ListSort, ListSortButtonVm,
};

/// The callback of a scope selection (ADR 0089 Decision 5).
pub(crate) type ListScopeHandler = Rc<dyn Fn(&ListScope, &mut Window, &mut App) + 'static>;

/// The callback of a sort choice in the sort menu (R-LIST-1 item 1).
pub(crate) type ListSortHandler = Rc<dyn Fn(&ListSort, &mut Window, &mut App) + 'static>;

/// The callback of a title row button, such as "Select".
pub(crate) type ListHeadButtonHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

/// The callbacks of the title row. A `None` callback leaves its control
/// without a click.
#[derive(Default)]
pub(crate) struct ListPageHeadHandlers {
    /// The scope selection.
    pub(crate) scope: Option<ListScopeHandler>,
    /// The Select action.
    pub(crate) select: Option<ListHeadButtonHandler>,
    /// The sort choice of the sort menu.
    pub(crate) sort: Option<ListSortHandler>,
    /// The letter click of the letter bar.
    pub(crate) letter: Option<LetterHandler>,
}

/// Renders the title row of a list page.
#[must_use]
pub(crate) fn list_page_head(
    display: ListPageHeadVm,
    handlers: ListPageHeadHandlers,
    cx: &App,
) -> AnyElement {
    let mut trailing = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::XS.scaled(cx));

    if let Some(sort) = display.sort {
        trailing = trailing.child(sort_menu(sort, handlers.sort.as_ref()));
    }

    if let Some(select) = display.select {
        let a11y_label = select.a11y_label.clone();
        let mut button = Button::styled(SharedString::from(select.id), ControlStyle::Ghost)
            .label(select.label)
            .a11y_label(a11y_label.clone())
            .tooltip(a11y_label);
        if let Some(handler) = handlers.select {
            button = button.on_click(move |_, window, cx| {
                handler(window, cx);
            });
        }
        trailing = trailing.child(button);
    }

    let title_row = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::SM.scaled(cx))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_size(FontSize::Title.scaled(cx))
                .font_weight(FontWeight::SEMIBOLD)
                .truncate()
                .child(SharedString::from(display.title)),
        )
        .child(trailing);

    let mut head = div()
        .flex()
        .flex_col()
        .gap(Spacing::XS.scaled(cx))
        .px(Spacing::MD.scaled(cx))
        .pt(Spacing::XS.scaled(cx))
        .pb(Spacing::XS.scaled(cx))
        .child(title_row);

    if let Some(scope) = display.scope {
        head = head.child(scope_control(scope, handlers.scope));
    }

    if let Some(letters) = display.letters {
        head = head.child(letter_bar(letters, handlers.letter.as_ref(), cx));
    }

    head.into_any_element()
}

/// The sort menu of a list page (R-LIST-1 item 1). Its trigger shows the
/// current sort. Each entry of the menu is one sort, and the current sort
/// has a check mark.
fn sort_menu(display: ListSortButtonVm, on_sort: Option<&ListSortHandler>) -> AnyElement {
    let mut menu = ContextMenu::new(
        SharedString::from(display.id),
        ContextMenuScope::WorkspaceFrame,
        SharedString::from(display.a11y_label),
    )
    .trigger_label(SharedString::from(display.label))
    .trigger_icon(IconName::ChevronDown);
    for option in display.options {
        let sort = option.sort;
        let mut item = ContextMenuItem::new(ContextMenuItemDisplay {
            id: SharedString::from(option.id),
            label: SharedString::from(option.label),
            a11y_label: SharedString::from(option.a11y_label),
            destructive: false,
            disabled: false,
        })
        .checked(option.selected);
        if let Some(handler) = on_sort {
            let handler = handler.clone();
            item = item.on_select(move |window, cx| handler(&sort, window, cx));
        }
        menu = menu.item(item);
    }
    menu.into_any_element()
}

/// The scope control of a list page: a `SegmentedControl` with the
/// `Filter` style, "All | In Library".
fn scope_control(display: ListScopeControlVm, on_scope: Option<ListScopeHandler>) -> AnyElement {
    let mut control = SegmentedControl::new(display.selected)
        .filter_style()
        .segments(display.options.map(|option| {
            Segment::new(SegmentDisplay {
                id: SharedString::from(option.id).into(),
                key: option.scope,
                label: SharedString::from(option.label),
                a11y_label: SharedString::from(option.a11y_label),
            })
        }));
    if let Some(handler) = on_scope {
        control = control.on_select(move |scope, window, cx| {
            handler(scope, window, cx);
        });
    }
    div()
        .id(SharedString::from(display.control_id))
        .child(control)
        .into_any_element()
}
