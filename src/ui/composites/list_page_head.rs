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
use gpui_component::Selectable;

use crate::ui::composites::letter_bar::{letter_bar, LetterBarKeys, LetterHandler};
use crate::ui::composites::{Segment, SegmentDisplay, SegmentedControl};
use crate::ui::control_styles::ControlStyle;
use crate::ui::icons::IconName;
use crate::ui::primitives::{
    Button, ContextMenu, ContextMenuItem, ContextMenuItemDisplay, ContextMenuScope,
};
use crate::ui::tokens::{FontSize, SemanticColor, Size, Spacing};
use crate::view_models::list_page::{
    ListPageHeadVm, ListScope, ListScopeControlVm, ListSort, ListSortButtonVm, ListView,
    ListViewControlVm, ReverseToggleVm,
};
use crate::view_models::search_results::{SearchPageActionDisplay, SearchPageActionStyle};

/// The callback of a scope selection (ADR 0089 Decision 5).
pub(crate) type ListScopeHandler = Rc<dyn Fn(&ListScope, &mut Window, &mut App) + 'static>;

/// The callback of a choice in the sort menu (R-LIST-1 item 1). It gets the
/// sort and the direction of the entry together (musicindex ADR 0054
/// R-LIST-13).
pub(crate) type ListSortHandler = Rc<dyn Fn(&ListSort, bool, &mut Window, &mut App) + 'static>;

/// The callback of a title row button, such as "Select".
pub(crate) type ListHeadButtonHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

/// The callback of a view icon (R-LIST-1 item 3).
pub(crate) type ListViewHandler = Rc<dyn Fn(&ListView, &mut Window, &mut App) + 'static>;

/// The callback of the Reverse toggle (musicindex ADR 0054 R-LIST-14). It
/// gets the reversed state that the click asks for.
pub(crate) type ListReverseHandler = Rc<dyn Fn(bool, &mut Window, &mut App) + 'static>;

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
    /// The view icon click (R-LIST-1 item 3).
    pub(crate) view: Option<ListViewHandler>,
    /// The letter click of the letter bar.
    pub(crate) letter: Option<LetterHandler>,
    /// The keyboard side of the letter bar.
    pub(crate) letter_keys: Option<LetterBarKeys>,
    /// The clicks of the typed actions of a search page (ADR 0089 Decision
    /// 4), one for each action of the view model, in the same order. A
    /// handler runs only when its action is available.
    pub(crate) actions: Vec<ListHeadButtonHandler>,
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
        trailing = trailing.child(sort_menu(sort, handlers.sort.as_ref(), false));
    }

    if let Some(view) = display.view {
        trailing = trailing.child(view_icons(view, handlers.view.as_ref(), false, cx));
    }

    if let Some(select) = display.select {
        let a11y_label = select.a11y_label.clone();
        let mut button = Button::styled(SharedString::from(select.id), ControlStyle::Neutral)
            .label(select.label)
            .a11y_label(a11y_label.clone())
            .tooltip(a11y_label);
        if let Some(handler) = handlers.select {
            // `on_activate` gives the tab stop, the ring and Enter or Space.
            button = button.on_activate(move |window, cx| {
                handler(window, cx);
            });
        }
        trailing = trailing.child(button);
    }

    for (index, action) in display.actions.into_iter().enumerate() {
        trailing = trailing.child(title_action(action, handlers.actions.get(index)));
    }

    // Column text does not call `truncate()` (ADR 0063). A query title wraps.
    let title_row = div()
        .flex()
        .flex_row()
        .items_center()
        // The title keeps one height, with or without a control beside it.
        .min_h(Size::MinHitTarget.scaled(cx))
        .gap(Spacing::SM.scaled(cx))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_size(FontSize::Title.scaled(cx))
                .font_weight(FontWeight::SEMIBOLD)
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
        head = head.child(letter_bar(
            letters,
            handlers.letter.as_ref(),
            handlers.letter_keys.as_ref(),
            cx,
        ));
    }

    head.into_any_element()
}

/// The typed action of a search page in the title row (ADR 0089 Decision
/// 4). The view model gives the label, the availability and the style.
fn title_action(
    display: SearchPageActionDisplay,
    on_click: Option<&ListHeadButtonHandler>,
) -> AnyElement {
    let style = match display.style {
        SearchPageActionStyle::Action => ControlStyle::Secondary,
        SearchPageActionStyle::Plain => ControlStyle::Neutral,
        SearchPageActionStyle::Destructive => ControlStyle::Destructive,
    };
    let button = Button::styled(SharedString::from(display.id), style)
        .label(display.label)
        .a11y_label(display.a11y_label)
        .disabled(!display.available);
    match on_click {
        Some(handler) if display.available => {
            let handler = Rc::clone(handler);
            button
                .on_activate(move |window, cx| handler(window, cx))
                .into_any_element()
        }
        _ => button.into_any_element(),
    }
}

/// The sort menu of a list page (R-LIST-1 item 1). Its trigger shows the
/// current entry. Each entry of the menu is one sort in one direction
/// (musicindex ADR 0054 R-LIST-13), and the current entry has a check mark. A `pinned` copy of a list bar has its own element ids
/// and no tab stop (ADR 0090 Decision 5). The list bar above the rows uses
/// this function too (ADR 0090 Decision 2).
pub(crate) fn sort_menu(
    display: ListSortButtonVm,
    on_sort: Option<&ListSortHandler>,
    pinned: bool,
) -> AnyElement {
    sort_menu_scoped(display, on_sort, pinned, None)
}

/// [`sort_menu`] with the element id inside `scope`. A page with more than
/// one such control, as the release sections of a feed page, gives each
/// control its own scope, so the controls share no element state.
pub(crate) fn sort_menu_scoped(
    display: ListSortButtonVm,
    on_sort: Option<&ListSortHandler>,
    pinned: bool,
    scope: Option<&str>,
) -> AnyElement {
    let mut menu = ContextMenu::new(
        control_id(display.id, pinned, scope),
        ContextMenuScope::WorkspaceFrame,
        SharedString::from(display.a11y_label),
    )
    .trigger_label(SharedString::from(display.label))
    .trigger_style(ControlStyle::Neutral)
    .trigger_icon(IconName::ChevronDown);
    if pinned {
        menu = menu.trigger_out_of_tab_sequence();
    }
    for option in display.options {
        let (sort, reversed) = (option.sort, option.reversed);
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
            item = item.on_select(move |window, cx| handler(&sort, reversed, window, cx));
        }
        menu = menu.item(item);
    }
    menu.into_any_element()
}

/// The view icons of a list page (R-LIST-1 item 3). Each icon is one
/// button. The current view has the tint of a selected button. `pinned` is
/// as for [`sort_menu`].
pub(crate) fn view_icons(
    display: ListViewControlVm,
    on_view: Option<&ListViewHandler>,
    pinned: bool,
    cx: &App,
) -> AnyElement {
    view_icons_scoped(display, on_view, pinned, None, cx)
}

/// [`view_icons`] with the element ids inside `scope`, as
/// [`sort_menu_scoped`].
pub(crate) fn view_icons_scoped(
    display: ListViewControlVm,
    on_view: Option<&ListViewHandler>,
    pinned: bool,
    scope: Option<&str>,
    cx: &App,
) -> AnyElement {
    let mut icons = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::XXS.scaled(cx));
    for option in display.options {
        let icon = match option.view {
            ListView::Rows => IconName::RowsView,
            ListView::Tiles => IconName::TilesView,
        };
        let view = option.view;
        let mut button = Button::styled(
            control_id(option.id, pinned, scope),
            ControlStyle::ToolbarIcon,
        )
        .leading_icon(icon)
        .a11y_label(option.a11y_label.clone())
        .tooltip(option.a11y_label)
        .selected(option.selected);
        if pinned {
            button = button.out_of_tab_sequence();
        }
        // R-LIST-7 item 8: a view icon that is not current is muted.
        if !option.selected {
            button = button.foreground(SemanticColor::SecondaryLabel);
        }
        if let Some(handler) = on_view {
            let handler = handler.clone();
            button = button.on_activate(move |window, cx| handler(&view, window, cx));
        }
        icons = icons.child(button);
    }
    icons.into_any_element()
}

/// The Reverse toggle of a list with one fixed order (musicindex ADR 0054
/// R-LIST-14). It is a text button in the Neutral style, and in the reversed
/// order it has the style of a selected control. `pinned` is as for
/// [`sort_menu`].
pub(crate) fn reverse_toggle(
    display: &ReverseToggleVm,
    on_reverse: Option<&ListReverseHandler>,
    pinned: bool,
) -> AnyElement {
    reverse_toggle_scoped(display, on_reverse, pinned, None)
}

/// [`reverse_toggle`] with the element id inside `scope`, as
/// [`sort_menu_scoped`].
pub(crate) fn reverse_toggle_scoped(
    display: &ReverseToggleVm,
    on_reverse: Option<&ListReverseHandler>,
    pinned: bool,
    scope: Option<&str>,
) -> AnyElement {
    let requested = !display.pressed;
    let style = if display.pressed {
        ControlStyle::SelectedTab
    } else {
        ControlStyle::Neutral
    };
    let mut button = Button::styled(control_id(display.id, pinned, scope), style)
        .label(display.label)
        .a11y_label(display.a11y_label.clone())
        .tooltip(display.a11y_label.clone())
        .selected(display.pressed);
    if pinned {
        button = button.out_of_tab_sequence();
    }
    if let Some(handler) = on_reverse {
        let handler = handler.clone();
        button = button.on_activate(move |window, cx| handler(requested, window, cx));
    }
    button.into_any_element()
}

/// The element id of a control. The pinned copy of a list bar has its own,
/// and a control with a `scope` has an id inside it.
fn control_id(id: &'static str, pinned: bool, scope: Option<&str>) -> SharedString {
    match (pinned, scope) {
        (true, _) => SharedString::from(format!("pinned-{id}")),
        (false, Some(scope)) => SharedString::from(format!("{scope}:{id}")),
        (false, None) => SharedString::from(id),
    }
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
