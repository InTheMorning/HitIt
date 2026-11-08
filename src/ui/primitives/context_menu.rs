//! Context-menu primitive for row commands and ADR 0063 text selection.
//!
//! GPUI does not currently expose a native arbitrary-element context-menu API,
//! so this primitive uses the shared [`Popover`] infrastructure while owning
//! the menu row chrome and action contract. Screens should pass display-ready
//! menu items instead of hand-rolling floating row action panels.
//! Pointer menus reuse those rows and the shared surface at the click position.
//! An item can open a second level in the same menu surface (ADR 0083 task
//! 006). Both menu forms show that level.

#![warn(clippy::pedantic)]

use std::rc::Rc;

use gpui::{
    actions, anchored, deferred, div, prelude::*, AnyElement, App, ElementId, Entity, FocusHandle,
    IntoElement, KeyBinding, MouseButton, Pixels, Point, RenderOnce, SharedString, Window,
};

use super::button::MenuActivate;
use crate::ui::control_styles::ControlStyle;
use crate::ui::icons::IconName;
use crate::ui::primitives::{
    Button, Divider, Popover, PopoverAlignment, PopoverPlacement, Surface, SurfaceElevation,
};
use crate::ui::tokens::{Size, Spacing};

type SelectHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;
type OpenLevelHandler = Rc<dyn Fn(usize, &mut Window, &mut App) + 'static>;

/// The builder of the second level of a menu item. It gets the controls of
/// the menu and gives the content of the level.
pub(crate) type MenuLevelBuilder =
    Rc<dyn Fn(&MenuLevelControls, &mut Window, &mut App) -> AnyElement + 'static>;

/// The controls that the second level of a menu item can use.
#[derive(Clone)]
pub(crate) struct MenuLevelControls {
    /// Shows the first level of the menu again.
    pub(crate) back: SelectHandler,
    /// Closes the menu.
    pub(crate) dismiss: SelectHandler,
}

const POINTER_MENU_KEY_CONTEXT: &str = "PointerContextMenu";
/// The movement and activation keys leave a text field of the menu alone.
const POINTER_MENU_ENTRY_KEY_CONTEXT: &str = "PointerContextMenu && !Input";

/// The list keys of the pane that do nothing while a menu has the focus.
const SWALLOWED_KEYS: &[&str] = &[
    "alt-up",
    "alt-down",
    "left",
    "right",
    "pageup",
    "pagedown",
    "shift-f10",
    "menu",
];

/// The most entries that one menu walk visits. It bounds the walk.
const MENU_WALK_LIMIT: usize = 512;

actions!(
    pointer_context_menu,
    [Dismiss, FocusNext, FocusPrev, FocusFirst, FocusLast, SwallowKey]
);

/// The keys of a pointer menu with the action each one runs. Escape closes
/// the menu. The other keys move the focus between its entries or run the
/// focused entry. Each action stops at the menu, so the pane never gets it
/// while the menu is open (ADR 0071, ADR 0092).
fn pointer_menu_key_bindings() -> Vec<KeyBinding> {
    [
        KeyBinding::new("escape", Dismiss, Some(POINTER_MENU_KEY_CONTEXT)),
        KeyBinding::new("down", FocusNext, Some(POINTER_MENU_ENTRY_KEY_CONTEXT)),
        KeyBinding::new("up", FocusPrev, Some(POINTER_MENU_ENTRY_KEY_CONTEXT)),
        KeyBinding::new("home", FocusFirst, Some(POINTER_MENU_ENTRY_KEY_CONTEXT)),
        KeyBinding::new("end", FocusLast, Some(POINTER_MENU_ENTRY_KEY_CONTEXT)),
        KeyBinding::new("enter", MenuActivate, Some(POINTER_MENU_ENTRY_KEY_CONTEXT)),
        KeyBinding::new("space", MenuActivate, Some(POINTER_MENU_ENTRY_KEY_CONTEXT)),
    ]
    .into_iter()
    .chain(
        SWALLOWED_KEYS
            .iter()
            .map(|key| KeyBinding::new(key, SwallowKey, Some(POINTER_MENU_ENTRY_KEY_CONTEXT))),
    )
    .collect()
}

/// Install the key bindings of the pointer menu before opening windows
/// (ADR 0071).
pub(crate) fn init(cx: &mut App) {
    cx.bind_keys(pointer_menu_key_bindings());
}

/// A move of the focus between the entries of a menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MenuFocusStep {
    Next,
    Prev,
    First,
    Last,
}

/// The entry that a step reaches, from the entry `current` of `len` entries.
/// `Next` and `Prev` wrap at the ends. No entry gives `None`.
fn stepped_index(len: usize, current: Option<usize>, step: MenuFocusStep) -> Option<usize> {
    if len == 0 {
        return None;
    }
    Some(match step {
        MenuFocusStep::First => 0,
        MenuFocusStep::Last => len - 1,
        MenuFocusStep::Next => current.map_or(0, |index| (index + 1) % len),
        MenuFocusStep::Prev => current.map_or(len - 1, |index| (index + len - 1) % len),
    })
}

/// The focus handles of the entries of a menu, in Tab order. GPUI gives no
/// list of the tab stops inside one element. Thus the walk starts at the menu
/// and follows `focus_next` until the focus leaves the menu. The focus ends on
/// the last entry, so the caller sets the focus after the walk.
fn menu_entries(menu: &FocusHandle, window: &mut Window, cx: &mut App) -> Vec<FocusHandle> {
    let mut entries: Vec<FocusHandle> = Vec::new();
    menu.focus(window, cx);
    for _ in 0..MENU_WALK_LIMIT {
        window.focus_next(cx);
        let Some(focused) = window.focused(cx) else {
            break;
        };
        if focused == *menu || !menu.contains_focused(window, cx) || entries.contains(&focused) {
            break;
        }
        entries.push(focused);
    }
    entries
}

/// Moves the focus to another entry of the menu. A menu with no entry that
/// takes the focus keeps the focus on itself.
fn step_menu_focus(menu: &FocusHandle, step: MenuFocusStep, window: &mut Window, cx: &mut App) {
    let current = window.focused(cx);
    let entries = menu_entries(menu, window, cx);
    let index = current.and_then(|current| entries.iter().position(|entry| *entry == current));
    match stepped_index(entries.len(), index, step) {
        Some(target) => entries[target].focus(window, cx),
        None => menu.focus(window, cx),
    }
}

struct ContextMenuState {
    open: bool,
    /// The index of the item whose second level shows. `None` shows the
    /// first level.
    level: Option<usize>,
}

/// The second-level state of a [`PointerContextMenu`].
struct PointerMenuLevel {
    level: Option<usize>,
    /// `true` after the menu took the focus and before the focus reached the
    /// first entry.
    focus_entry_pending: bool,
}

/// The open state of a [`ContextMenu`] that its host holds (ADR 0084 task
/// 030). A host that must read the open state makes the handle at its own
/// element position and gives it to the menu. GPUI puts each stateless
/// component under its own element id. Thus a host that reads a keyed
/// state of the menu at another position reads a different state.
#[derive(Clone)]
pub(crate) struct ContextMenuHandle(Entity<ContextMenuState>);

impl ContextMenuHandle {
    /// The held open state of the menu `id` at this element position. Call
    /// it at the same position at each render.
    pub(crate) fn new(id: &SharedString, window: &mut Window, cx: &mut App) -> Self {
        Self(
            window.use_keyed_state(ContextMenu::state_key(id), cx, |_, _| {
                ContextMenuState::closed()
            }),
        )
    }

    /// `true` while the menu is open.
    pub(crate) fn is_open(&self, cx: &App) -> bool {
        self.0.read(cx).open
    }
}

/// Row families expected to expose context-menu commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContextMenuScope {
    FeedList,
    TrackList,
    WorkspaceFrame,
}

/// Display-ready fields for one context-menu item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextMenuItemDisplay {
    pub id: SharedString,
    pub label: SharedString,
    pub a11y_label: SharedString,
    pub destructive: bool,
    pub disabled: bool,
}

/// One selectable menu item in a [`ContextMenu`].
#[derive(Clone)]
#[must_use]
pub struct ContextMenuItem {
    description: Option<SharedString>,
    display: ContextMenuItemDisplay,
    on_select: Option<SelectHandler>,
    level: Option<MenuLevelBuilder>,
    checked: bool,
}

/// Shared context-menu surface.
#[derive(IntoElement)]
#[must_use]
pub struct ContextMenu {
    id: SharedString,
    scope: ContextMenuScope,
    trigger_label: SharedString,
    trigger_icon: IconName,
    trigger_tooltip: Option<SharedString>,
    disabled: bool,
    trigger_a11y_label: SharedString,
    items: Vec<ContextMenuItem>,
    handle: Option<ContextMenuHandle>,
    placement: PopoverPlacement,
    trigger_style: ControlStyle,
    trigger_in_tab_sequence: bool,
}

/// Controlled menu at a pointer position. The owner removes it on dismissal.
#[derive(IntoElement)]
#[must_use]
pub struct PointerContextMenu {
    id: ElementId,
    position: Point<Pixels>,
    return_focus: FocusHandle,
    items: Vec<ContextMenuItem>,
    on_dismiss: SelectHandler,
}

impl PointerContextMenu {
    pub fn new(
        id: impl Into<ElementId>,
        position: Point<Pixels>,
        return_focus: FocusHandle,
        on_dismiss: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            position,
            return_focus,
            items: Vec::new(),
            on_dismiss: Rc::new(on_dismiss),
        }
    }

    pub fn item(mut self, item: ContextMenuItem) -> Self {
        self.items.push(item);
        self
    }
}

impl RenderOnce for PointerContextMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus = window.use_keyed_state(self.id.clone(), cx, |_, cx| cx.focus_handle());
        let focus = focus.read(cx).clone();
        let level_key = SharedString::from(format!("{}-level", self.id));
        let level_state = window.use_keyed_state(level_key, cx, |_, _| PointerMenuLevel {
            level: None,
            focus_entry_pending: false,
        });
        if !focus.contains_focused(window, cx) {
            focus.focus(window, cx);
            // The entries paint after this render. The next render moves the
            // focus to the first entry that takes it, so Return runs an
            // entry at once. A change of the level loses the focus and does
            // this again.
            level_state.update(cx, |state, _| state.focus_entry_pending = true);
            window.refresh();
            // `refresh` during a draw can be lost on a real display. This
            // callback runs after the entries paint. It acts only while the
            // focus is still on the menu itself.
            let menu = focus.clone();
            window.on_next_frame(move |window, cx| {
                if window.focused(cx).is_some_and(|focused| focused == menu) {
                    step_menu_focus(&menu, MenuFocusStep::First, window, cx);
                    window.refresh();
                }
            });
        } else if level_state.read(cx).focus_entry_pending {
            level_state.update(cx, |state, _| state.focus_entry_pending = false);
            step_menu_focus(&focus, MenuFocusStep::First, window, cx);
        }
        let dismiss: SelectHandler = {
            let level_state = level_state.clone();
            Rc::new(move |window, cx| {
                level_state.update(cx, |state, _| state.level = None);
                self.return_focus.focus(window, cx);
                (self.on_dismiss)(window, cx);
            })
        };
        let open_level: OpenLevelHandler = {
            let level_state = level_state.clone();
            Rc::new(move |index, _, cx| {
                level_state.update(cx, |state, cx| {
                    state.level = Some(index);
                    cx.notify();
                });
            })
        };
        let back: SelectHandler = {
            let level_state = level_state.clone();
            Rc::new(move |_, cx| {
                level_state.update(cx, |state, cx| {
                    state.level = None;
                    cx.notify();
                });
            })
        };
        let level = level_state.read(cx).level;
        let outside_dismiss = dismiss.clone();
        let escape_dismiss = dismiss.clone();
        let menu = div()
            .id(self.id)
            .key_context(POINTER_MENU_KEY_CONTEXT)
            .track_focus(&focus)
            .occlude()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_mouse_down_out(move |_, window, cx| {
                outside_dismiss(window, cx);
                cx.stop_propagation();
            })
            // Bound app actions run before raw key handlers. Dismiss in the
            // menu's own context so Escape cannot reach the pane (ADR 0071).
            .on_action(move |_: &Dismiss, window, cx| {
                escape_dismiss(window, cx);
                cx.stop_propagation();
            })
            .on_action(step_action::<FocusNext>(&focus, MenuFocusStep::Next))
            .on_action(step_action::<FocusPrev>(&focus, MenuFocusStep::Prev))
            .on_action(step_action::<FocusFirst>(&focus, MenuFocusStep::First))
            .on_action(step_action::<FocusLast>(&focus, MenuFocusStep::Last))
            // An entry runs this action itself. It reaches the menu only when
            // no entry has the focus, and then the key does nothing.
            .on_action(|_: &MenuActivate, _, cx| cx.stop_propagation())
            .on_action(|_: &SwallowKey, _, cx| cx.stop_propagation())
            .child(
                Surface::new(SurfaceElevation::Floating)
                    .padding(Spacing::SM)
                    .child(build_menu_level(
                        &MenuLevelControls {
                            back,
                            dismiss: dismiss.clone(),
                        },
                        &open_level,
                        self.items,
                        level,
                        window,
                        cx,
                    )),
            );

        deferred(
            anchored().child(
                div()
                    .w(window.bounds().size.width)
                    .h(window.bounds().size.height)
                    .occlude()
                    .child(
                        anchored()
                            .position(self.position)
                            .snap_to_window_with_margin(Spacing::SM.scaled(cx))
                            .child(menu),
                    ),
            ),
        )
    }
}

/// The handler of a focus-step action of the menu with the focus handle `menu`.
fn step_action<A: 'static>(
    menu: &FocusHandle,
    step: MenuFocusStep,
) -> impl Fn(&A, &mut Window, &mut App) + 'static {
    let menu = menu.clone();
    move |_, window, cx| {
        step_menu_focus(&menu, step, window, cx);
        cx.stop_propagation();
    }
}

impl ContextMenuItem {
    pub fn new(display: ContextMenuItemDisplay) -> Self {
        Self {
            display,
            description: None,
            on_select: None,
            level: None,
            checked: false,
        }
    }

    /// Marks the item with a check mark. A checked item is the current
    /// choice of a menu of choices, such as the sort of a list page.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Gives the item a second level. A selection of the item shows that
    /// level in the same menu surface instead of closing the menu.
    pub(crate) fn level(
        mut self,
        builder: impl Fn(&MenuLevelControls, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.level = Some(Rc::new(builder));
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn on_select(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
}

impl ContextMenu {
    pub fn new(
        id: impl Into<SharedString>,
        scope: ContextMenuScope,
        trigger_a11y_label: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            scope,
            trigger_label: SharedString::from("Actions"),
            trigger_icon: IconName::More,
            trigger_tooltip: None,
            disabled: false,
            trigger_a11y_label: trigger_a11y_label.into(),
            items: Vec::new(),
            handle: None,
            placement: PopoverPlacement::Below,
            trigger_style: ControlStyle::RowAction,
            trigger_in_tab_sequence: true,
        }
    }

    /// Keeps the trigger out of the Tab sequence. The menu of a list row uses
    /// this, because Tab does not pass each row action (WCAG 2.1.1 note of
    /// `Button`).
    pub(crate) const fn trigger_out_of_tab_sequence(mut self) -> Self {
        self.trigger_in_tab_sequence = false;
        self
    }

    /// Gives the trigger a control style. The default is the row action.
    pub(crate) fn trigger_style(mut self, style: ControlStyle) -> Self {
        self.trigger_style = style;
        self
    }

    /// Opens the menu above or below its trigger. The default is below. A
    /// trigger in a bottom bar opens above (ADR 0089 Decision 1).
    pub(crate) fn placement(mut self, placement: PopoverPlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Uses the open state that the host holds. Without a handle, the menu
    /// keeps its open state at its own element position.
    pub(crate) fn handle(mut self, handle: ContextMenuHandle) -> Self {
        self.handle = Some(handle);
        self
    }

    pub fn trigger_icon(mut self, icon: IconName) -> Self {
        self.trigger_icon = icon;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The hover help of the trigger, for a trigger with an icon and no
    /// text (ADR 0083 Decision 10).
    pub fn trigger_tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.trigger_tooltip = Some(tooltip.into());
        self
    }

    fn state_key(id: &SharedString) -> SharedString {
        SharedString::from(format!("{id}-state"))
    }

    pub fn trigger_label(mut self, label: impl Into<SharedString>) -> Self {
        self.trigger_label = label.into();
        self
    }

    pub fn item(mut self, item: ContextMenuItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ContextMenuItem>) -> Self {
        self.items.extend(items);
        self
    }

    #[must_use]
    pub const fn scope(&self) -> ContextMenuScope {
        self.scope
    }
}

impl ContextMenuState {
    const fn closed() -> Self {
        Self {
            open: false,
            level: None,
        }
    }
}

impl RenderOnce for ContextMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state: Entity<ContextMenuState> = match self.handle {
            Some(handle) => handle.0,
            None => ContextMenuHandle::new(&self.id, window, cx).0,
        };

        let open = state.read(cx).open && !self.disabled;
        let items = self.items;
        let trigger_id = SharedString::from(format!("{}-trigger", self.id));
        let mut trigger = Button::styled(trigger_id, self.trigger_style)
            .leading_icon(self.trigger_icon)
            .disabled(self.disabled)
            .label(self.trigger_label)
            .a11y_label(self.trigger_a11y_label);
        // The popover owns the click. Enter and Space open the menu from the
        // keyboard (ADR 0090 invariant "keyboard reach").
        if self.trigger_in_tab_sequence {
            let state = state.clone();
            trigger = trigger.on_key_activate(move |_, cx| {
                state.update(cx, |state, cx| {
                    state.open = !state.open;
                    state.level = None;
                    cx.notify();
                });
            });
        }
        if let Some(tooltip) = self.trigger_tooltip {
            trigger = trigger.tooltip(tooltip);
        }

        Popover::new(self.id)
            .placement(self.placement)
            .alignment(PopoverAlignment::End)
            .surface_padding(Spacing::SM)
            .open(open)
            .on_open_change({
                let state = state.clone();
                move |is_open, _window, cx| {
                    state.update(cx, |state, cx| {
                        state.open = *is_open;
                        state.level = None;
                        cx.notify();
                    });
                }
            })
            .trigger(trigger)
            .content(move |window, cx| {
                let dismiss: SelectHandler = {
                    let state = state.clone();
                    Rc::new(move |_, cx| {
                        state.update(cx, |state, cx| {
                            state.open = false;
                            state.level = None;
                            cx.notify();
                        });
                    })
                };
                let back: SelectHandler = {
                    let state = state.clone();
                    Rc::new(move |_, cx| {
                        state.update(cx, |state, cx| {
                            state.level = None;
                            cx.notify();
                        });
                    })
                };
                let open_level: OpenLevelHandler = {
                    let state = state.clone();
                    Rc::new(move |index, _, cx| {
                        state.update(cx, |state, cx| {
                            state.level = Some(index);
                            cx.notify();
                        });
                    })
                };
                let level = state.read(cx).level;
                build_menu_level(
                    &MenuLevelControls { back, dismiss },
                    &open_level,
                    items.clone(),
                    level,
                    window,
                    cx,
                )
            })
    }
}

/// The content of a menu: the second level of the item at `level`, or
/// the first level when `level` is `None` or names no such level.
fn build_menu_level(
    controls: &MenuLevelControls,
    open_level: &OpenLevelHandler,
    items: Vec<ContextMenuItem>,
    level: Option<usize>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    if let Some(builder) = level
        .and_then(|index| items.get(index))
        .and_then(|item| item.level.clone())
    {
        return builder(controls, window, cx);
    }
    build_menu_content(&controls.dismiss, open_level, items, cx).into_any_element()
}

fn build_menu_content(
    on_dismiss: &SelectHandler,
    open_level: &OpenLevelHandler,
    items: Vec<ContextMenuItem>,
    cx: &App,
) -> impl IntoElement {
    let is_empty = items.is_empty();
    let mut content = div()
        .id("context-menu-items")
        .max_h(Size::MenuRegular.scaled(cx))
        .overflow_y_scroll()
        .w(Size::MenuRegular.scaled(cx))
        .flex()
        .flex_col()
        .gap(Spacing::XXS.scaled(cx));

    let mut previous_destructive = true;
    for (index, item) in items.into_iter().enumerate() {
        let display = item.display;
        // ADR 0083 Decision 5: a divider separates a destructive item from
        // the items before it.
        if display.destructive && !previous_destructive {
            content = content.child(Divider::horizontal());
        }
        previous_destructive = display.destructive;
        let on_select = item.on_select;
        let on_dismiss = on_dismiss.clone();
        let mut button = if display.destructive {
            Button::styled(display.id, ControlStyle::DestructiveMenuItem)
        } else {
            Button::plain(display.id)
        }
        .full_width()
        .align_leading()
        .label(display.label)
        .a11y_label(display.a11y_label)
        .disabled(display.disabled);

        if let Some(description) = item.description {
            button = button.description(description);
        }
        if item.checked {
            button = button.leading_icon(IconName::Check);
        }
        if !display.disabled && item.level.is_some() {
            // An item with a second level shows that level. The menu stays
            // open.
            let open_level = open_level.clone();
            button = button.on_activate(move |window, cx| open_level(index, window, cx));
        } else if !display.disabled {
            button = button.on_activate(move |window, cx| {
                on_dismiss(window, cx);
                if let Some(handler) = &on_select {
                    handler(window, cx);
                }
            });
        }

        content = content.child(button);
    }

    if is_empty {
        content = content.child(
            div()
                .px(Spacing::MD.scaled(cx))
                .py(Spacing::SM.scaled(cx))
                .child("No actions"),
        );
    }

    #[cfg(test)]
    let content = content.debug_selector(|| "context-menu-items".to_owned());

    content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_context_menu_names_expected_scopes() {
        assert_eq!(
            [
                ContextMenuScope::FeedList,
                ContextMenuScope::TrackList,
                ContextMenuScope::WorkspaceFrame,
            ]
            .len(),
            3
        );
    }

    #[test]
    fn menu_items_keep_display_and_handler_contracts_separate() {
        let item = ContextMenuItem::new(ContextMenuItemDisplay {
            id: "remove".into(),
            label: "Remove".into(),
            a11y_label: "Remove track".into(),
            destructive: true,
            disabled: false,
        })
        .on_select(|_, _| {});

        assert_eq!(item.display.label, SharedString::from("Remove"));
        assert!(item.display.destructive);
        assert!(item.on_select.is_some());
    }

    #[test]
    fn adr_0092_menu_step_wraps_at_both_ends() {
        use MenuFocusStep::{First, Last, Next, Prev};
        assert_eq!(stepped_index(3, Some(2), Next), Some(0));
        assert_eq!(stepped_index(3, Some(0), Prev), Some(2));
        assert_eq!(stepped_index(3, Some(0), Next), Some(1));
        assert_eq!(stepped_index(3, Some(1), Prev), Some(0));
        assert_eq!(stepped_index(3, Some(1), First), Some(0));
        assert_eq!(stepped_index(3, Some(1), Last), Some(2));
        assert_eq!(stepped_index(3, None, Next), Some(0));
        assert_eq!(stepped_index(3, None, Prev), Some(2));
        assert_eq!(stepped_index(0, None, Next), None);
        assert_eq!(stepped_index(1, Some(0), Next), Some(0));
    }

    #[test]
    fn adr_0092_menu_key_table_binds_each_menu_key() {
        let bindings = pointer_menu_key_bindings();
        let keys: Vec<String> = bindings
            .iter()
            .map(|binding| binding.keystrokes()[0].unparse())
            .collect();
        for key in [
            "escape",
            "down",
            "up",
            "home",
            "end",
            "enter",
            "space",
            "alt-up",
            "alt-down",
            "left",
            "right",
            "pageup",
            "pagedown",
            "shift-f10",
            "menu",
        ] {
            assert!(keys.iter().any(|bound| bound == key), "{key} is bound");
        }
    }

    struct MenuKeyTest {
        pane: FocusHandle,
        selected: Rc<std::cell::RefCell<Vec<&'static str>>>,
    }

    impl gpui::Render for MenuKeyTest {
        fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
            let item = |id: &'static str, disabled: bool| {
                let selected = Rc::clone(&self.selected);
                ContextMenuItem::new(ContextMenuItemDisplay {
                    id: id.into(),
                    label: id.into(),
                    a11y_label: id.into(),
                    destructive: false,
                    disabled,
                })
                .on_select(move |_, _| selected.borrow_mut().push(id))
            };
            div().size_full().track_focus(&self.pane).child(
                PointerContextMenu::new(
                    "key-test-menu",
                    gpui::point(gpui::px(20.0), gpui::px(20.0)),
                    self.pane.clone(),
                    |_, _| {},
                )
                .item(item("off", true))
                .item(item("one", false))
                .item(item("two", false))
                .item(item("three", false)),
            )
        }
    }

    fn open_key_test(
        cx: &mut gpui::TestAppContext,
    ) -> (
        Rc<std::cell::RefCell<Vec<&'static str>>>,
        &mut gpui::VisualTestContext,
    ) {
        cx.update(gpui_component::init);
        cx.update(init);
        let selected = Rc::new(std::cell::RefCell::new(Vec::new()));
        let shared = Rc::clone(&selected);
        let (_view, cx) = cx.add_window_view(move |_, cx| MenuKeyTest {
            pane: cx.focus_handle(),
            selected: shared,
        });
        cx.update(|window, cx| {
            window.activate_window();
            let _ = window.draw(cx);
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        (selected, cx)
    }

    /// ADR 0092: Down and Up move between the entries and wrap, Return runs
    /// the focused entry, and a disabled entry never takes the focus.
    #[gpui::test]
    fn adr_0092_menu_keys_move_focus_wrap_and_run_the_entry(cx: &mut gpui::TestAppContext) {
        let (selected, cx) = open_key_test(cx);
        // A run of an entry closes the menu in the app. Here the menu stays
        // and takes the focus again, so each run starts at the first entry.
        // The first enabled entry has the focus at once.
        for keys in [
            "enter",
            "up enter",
            "down enter",
            "down down enter",
            "down down down enter",
            "end enter",
            "end home enter",
            "space",
        ] {
            cx.simulate_keystrokes(keys);
            // Two draws: the first gives the focus back to the menu, and the
            // second moves it to the first entry.
            for _ in 0..2 {
                cx.update(|window, cx| {
                    let _ = window.draw(cx);
                });
            }
        }
        assert_eq!(
            *selected.borrow(),
            vec!["one", "three", "two", "three", "one", "three", "one", "one"]
        );
    }

    #[test]
    fn context_menu_carries_row_scope() {
        let menu = ContextMenu::new(
            "track-actions",
            ContextMenuScope::TrackList,
            "Track actions",
        );

        assert_eq!(menu.scope(), ContextMenuScope::TrackList);
    }
}
