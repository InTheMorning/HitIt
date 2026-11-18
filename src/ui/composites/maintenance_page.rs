//! Separate actions, instructions and report viewports (ADR 0074).

#![warn(clippy::pedantic)]

use std::rc::Rc;

use gpui::{
    div, prelude::*, AnyElement, App, IntoElement, RenderOnce, ScrollHandle, SharedString, Window,
};
use gpui_component::scroll::ScrollableElement;

use crate::ui::composites::confirmation_dialog::destructive_button_space;
use crate::ui::composites::page_scroll_content::page_scroll_content;
use crate::ui::control_styles::ControlStyle;
use crate::ui::icons::IconName;
use crate::ui::layouts::{self, scaled_dimension};
use crate::ui::primitives::{Button, Divider, Popover};
use crate::ui::tokens::{color, FontSize, ScaleFactor, SemanticColor, Size, Spacing};
use crate::view_models::maintenance::{MaintenanceLayout, MaintenanceView, PageChoice};
use crate::view_models::startup::StartupAvailability;

pub(crate) type PageCallback<T> = Rc<dyn Fn(T, &mut Window, &mut App)>;
type MenuSelection = Rc<dyn Fn(&mut Window, &mut App)>;

pub(crate) struct PageNavigation {
    pub(crate) view: MaintenanceView,
    pub(crate) select: PageCallback<MaintenanceView>,
    pub(crate) scroll: ScrollHandle,
}

/// A navigation menu keeps all choices reachable without reserving a row per level.
#[derive(IntoElement)]
pub(crate) struct PageMenu<T: Copy + 'static> {
    id: &'static str,
    label: &'static str,
    choices: Vec<PageChoice<T>>,
    select: PageCallback<T>,
}

impl<T: Copy + 'static> PageMenu<T> {
    pub(crate) fn new(
        id: &'static str,
        label: &'static str,
        choices: Vec<PageChoice<T>>,
        select: PageCallback<T>,
    ) -> Self {
        Self {
            id,
            label,
            choices,
            select,
        }
    }
}

impl<T: Copy + 'static> RenderOnce for PageMenu<T> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(
            SharedString::from(format!("{}-selection-state", self.id)),
            cx,
            |_, _| false,
        );
        let open = *state.read(cx);
        let focus = window.use_keyed_state(
            SharedString::from(format!("{}-trigger-focus", self.id)),
            cx,
            |_, cx| cx.focus_handle(),
        );
        let label = self
            .choices
            .iter()
            .find(|choice| choice.selected)
            .map_or(self.label, |choice| choice.label);
        let trigger = Button::styled(
            SharedString::from(format!("{}-trigger", self.id)),
            ControlStyle::RowAction,
        )
        .compact_inset()
        .label(label)
        .a11y_label(self.label)
        .track_focus(focus.read(cx))
        .leading_icon(IconName::ChevronDown);
        let id = self.id;
        let popover = Popover::new(self.id)
            .open(open)
            .on_open_change({
                let state = state.clone();
                move |open, _, cx| {
                    state.update(cx, |state, cx| {
                        *state = *open;
                        cx.notify();
                    });
                }
            })
            .trigger(trigger)
            .content(move |_, cx| {
                div()
                    .id("maintenance-page-options")
                    .flex()
                    .flex_col()
                    .min_w_0()
                    .w(Size::MenuRegular.scaled(cx))
                    .max_h(Size::MenuRegular.scaled(cx))
                    .overflow_y_scrollbar()
                    .children(self.choices.iter().map(|choice| {
                        let state = state.clone();
                        let select = self.select.clone();
                        let value = choice.value;
                        let activate: MenuSelection = Rc::new(move |window, cx| {
                            state.update(cx, |state, cx| {
                                *state = false;
                                cx.notify();
                            });
                            select(value, window, cx);
                        });
                        let click = activate.clone();
                        let button = Button::styled(
                            SharedString::from(format!("{}-{}", self.id, choice.label)),
                            if choice.selected {
                                ControlStyle::SelectedTab
                            } else {
                                ControlStyle::Neutral
                            },
                        )
                        .label(choice.label)
                        .a11y_label(choice.a11y_label.clone())
                        .full_width()
                        .align_leading()
                        .disabled(choice.availability != StartupAvailability::Available)
                        .on_activate(move |window, cx| click(window, cx));
                        let button = if choice.selected {
                            button.leading_icon(IconName::Check)
                        } else {
                            button
                        };
                        div().child(button).when(
                            choice.availability == StartupAvailability::Available,
                            |row| {
                                row.on_action(move |_: &gpui_base::actions::Confirm, window, cx| {
                                    // Popover binds Enter/Space before raw button keys.
                                    // Select this row instead of toggling the enclosing menu.
                                    cx.stop_propagation();
                                    activate(window, cx);
                                })
                            },
                        )
                    }))
                    .debug_selector(|| "maintenance-page-options".to_owned())
            });
        div()
            .flex_shrink_0()
            .child(popover)
            .debug_selector(move || id.to_owned())
    }
}

pub(crate) fn instructions(cx: &App) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .min_w_0()
        .flex_shrink_0()
        .gap(Spacing::MD.scaled(cx))
        .text_size(FontSize::Body.scaled(cx))
        .whitespace_normal()
}

pub(crate) fn page_heading(title: impl Into<SharedString>, cx: &App) -> AnyElement {
    div()
        .min_w_0()
        .text_size(FontSize::Headline.scaled(cx))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .child(title.into())
        .into_any_element()
}

fn view_toggle(view: MaintenanceView, select: PageCallback<MaintenanceView>) -> AnyElement {
    let toggle = view.toggle();
    div()
        .flex_shrink_0()
        .child(
            Button::styled("maintenance-view-toggle", ControlStyle::RowAction)
                .label(toggle.label)
                .a11y_label(toggle.a11y_label)
                .disabled(toggle.availability != StartupAvailability::Available)
                .on_activate(move |window, cx| {
                    select(toggle.value, window, cx);
                }),
        )
        .debug_selector(|| "maintenance-view-toggle".to_owned())
        .into_any_element()
}

/// One action control of a maintenance page.
pub(crate) struct MaintenanceControl {
    element: AnyElement,
    destructive: bool,
}

impl MaintenanceControl {
    /// A control that changes stored data with no undo. The page puts it
    /// last in the column and first in the band
    /// ([`MaintenanceLayout::action_sequence`]).
    pub(crate) fn destructive(element: impl IntoElement) -> Self {
        Self {
            element: element.into_any_element(),
            destructive: true,
        }
    }
}

impl From<AnyElement> for MaintenanceControl {
    fn from(element: AnyElement) -> Self {
        Self {
            element,
            destructive: false,
        }
    }
}

/// The report is a sibling of the instruction viewport, never its child.
#[derive(IntoElement)]
pub(crate) struct MaintenancePage {
    title: &'static str,
    navigation: PageNavigation,
    actions: Vec<MaintenanceControl>,
    instructions: AnyElement,
    report: AnyElement,
    selector: Option<AnyElement>,
}

impl MaintenancePage {
    pub(crate) fn new(
        title: &'static str,
        navigation: PageNavigation,
        actions: impl IntoIterator<Item = impl Into<MaintenanceControl>>,
        instructions: AnyElement,
        report: AnyElement,
    ) -> Self {
        Self {
            title,
            navigation,
            actions: actions.into_iter().map(Into::into).collect(),
            instructions,
            report,
            selector: None,
        }
    }

    pub(crate) fn selector(mut self, selector: impl IntoElement) -> Self {
        self.selector = Some(selector.into_any_element());
        self
    }
}

impl RenderOnce for MaintenancePage {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // ADR 0039: un-scaling the viewport for a breakpoint is geometry —
        // CHROME domain.
        let width =
            f32::from(window.viewport_size().width) / ScaleFactor::current(cx).chrome_multiplier();
        self.render_layout(MaintenanceLayout::for_width(width), cx)
    }
}

/// The controls in the sequence of [`MaintenanceLayout::action_sequence`].
/// In the column, a divider comes before the destructive controls, as in a
/// menu. In the band, a larger space follows them (musicindex ADR 0064
/// R-ACT-8 item 4).
fn ordered_controls(
    controls: Vec<MaintenanceControl>,
    layout: MaintenanceLayout,
    cx: &App,
) -> Vec<AnyElement> {
    let flags: Vec<bool> = controls.iter().map(|control| control.destructive).collect();
    let mut slots: Vec<Option<MaintenanceControl>> = controls.into_iter().map(Some).collect();
    let mut elements = Vec::with_capacity(slots.len() + 1);
    let mut previous: Option<bool> = None;
    for index in layout.action_sequence(&flags) {
        let Some(control) = slots[index].take() else {
            continue;
        };
        let destructive = control.destructive;
        if previous.is_some_and(|flag| flag != destructive) {
            elements.push(match layout {
                MaintenanceLayout::Columns => Divider::horizontal().into_any_element(),
                MaintenanceLayout::Stacked => destructive_button_space(cx).into_any_element(),
            });
        }
        previous = Some(destructive);
        elements.push(control.element);
    }
    elements
}

fn maintenance_actions(
    controls: Vec<MaintenanceControl>,
    layout: MaintenanceLayout,
    cx: &App,
) -> AnyElement {
    let stacked = layout == MaintenanceLayout::Stacked;
    let mut action_content = page_scroll_content(cx)
        .p(Spacing::SM.scaled(cx))
        .gap(Spacing::XS.scaled(cx));
    let controls = ordered_controls(controls, layout, cx);
    if stacked {
        action_content = action_content.flex_row().flex_wrap();
        div()
            .id("maintenance-actions")
            .min_w_0()
            .min_h_0()
            .flex_shrink_0()
            .w_full()
            .min_h(Size::MinHitTarget.scaled(cx) + Spacing::SM.scaled(cx) * 2.)
            .max_h(gpui::relative(layouts::MAINTENANCE_ACTION_BAND_FRACTION))
            .overflow_hidden()
            .bg(color(cx, SemanticColor::GroupBackground))
            .child(
                div()
                    .size_full()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .child(action_content.children(controls)),
            )
            .debug_selector(|| "maintenance-actions".to_owned())
            .into_any_element()
    } else {
        div()
            .id("maintenance-actions")
            .min_w_0()
            .min_h_0()
            .flex_shrink_0()
            .overflow_y_scrollbar()
            .bg(color(cx, SemanticColor::GroupBackground))
            .w(scaled_dimension(
                layouts::MAINTENANCE_ACTION_COLUMN_WIDTH,
                cx,
            ))
            .child(action_content.children(controls))
            .debug_selector(|| "maintenance-actions".to_owned())
            .into_any_element()
    }
}

impl MaintenancePage {
    fn render_layout(self, layout: MaintenanceLayout, cx: &App) -> impl IntoElement {
        let has_actions = !self.actions.is_empty();
        let stacked = layout == MaintenanceLayout::Stacked;
        let actions = maintenance_actions(self.actions, layout, cx);

        let content = match self.navigation.view {
            MaintenanceView::Instructions => div()
                .id("maintenance-instructions")
                .size_full()
                .min_h_0()
                .min_w_0()
                .overflow_y_scroll()
                .track_scroll(&self.navigation.scroll)
                .child(page_scroll_content(cx).child(self.instructions))
                .vertical_scrollbar(&self.navigation.scroll)
                .into_any_element(),
            MaintenanceView::Report => self.report,
        };
        let viewport = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .overflow_hidden()
            .child(content);
        #[cfg(test)]
        let viewport = viewport.debug_selector(|| "maintenance-content".to_owned());
        let main = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_w_0()
            .min_h_0()
            .overflow_hidden()
            .gap(Spacing::SM.scaled(cx))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .justify_between()
                    .min_w_0()
                    .flex_shrink_0()
                    .gap(Spacing::SM.scaled(cx))
                    .child(
                        self.selector
                            .unwrap_or_else(|| page_heading(self.title, cx)),
                    )
                    .child(view_toggle(self.navigation.view, self.navigation.select))
                    .debug_selector(|| "maintenance-toolbar".to_owned()),
            )
            .child(viewport)
            .debug_selector(|| "maintenance-main".to_owned());
        div()
            .size_full()
            .flex()
            .min_h_0()
            .min_w_0()
            .overflow_hidden()
            .text_size(FontSize::Body.scaled(cx))
            .gap(Spacing::MD.scaled(cx))
            .when(stacked, gpui::Styled::flex_col)
            .when(has_actions, |body| body.child(actions))
            .child(main)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::composites::log_frame::{LogFrame, LogFrames};
    use crate::view_models::log_view::LogSource;
    use crate::view_models::maintenance::RecoveryPage;
    use gpui::{Context, Render, ScrollDelta, ScrollWheelEvent, TestAppContext, TouchPhase};

    struct MenuTest {
        selected: RecoveryPage,
        selections: usize,
    }

    impl Render for MenuTest {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let entity = cx.weak_entity();
            div().flex().items_start().child(PageMenu::new(
                "test-view-menu",
                RecoveryPage::MENU_LABEL,
                self.selected.choices(),
                Rc::new(move |view, _, cx| {
                    let _ = entity.update(cx, |this, cx| {
                        this.selected = view;
                        this.selections += 1;
                        cx.notify();
                    });
                }),
            ))
        }
    }

    /// Situational ADR 0074: compact navigation retains keyboard selection and dismissal.
    #[gpui::test]
    fn adr_0074_page_menu_supports_keyboard_and_pointer(cx: &mut TestAppContext) {
        cx.update(gpui_component::init);
        let (page, cx) = cx.add_window_view(|_, _| MenuTest {
            selected: RecoveryPage::Startup,
            selections: 0,
        });
        cx.update(|window, cx| {
            let _ = window.draw(cx);
            window.focus_next(cx);
        });
        cx.simulate_keystrokes("enter");
        cx.run_until_parked();
        assert!(cx.debug_bounds("maintenance-page-options").is_some());
        cx.simulate_keystrokes("escape");
        cx.run_until_parked();
        assert!(cx.debug_bounds("maintenance-page-options").is_none());
        assert_eq!(page.read_with(cx, |page, _| page.selections), 0);
        cx.simulate_keystrokes("space");
        cx.run_until_parked();
        assert!(cx.debug_bounds("maintenance-page-options").is_some());
        cx.update(|window, cx| {
            window.focus_next(cx);
            window.focus_next(cx);
        });
        cx.simulate_keystrokes("enter");
        cx.run_until_parked();
        assert_eq!(
            page.read_with(cx, |page, _| page.selected),
            RecoveryPage::Configuration
        );
        assert_eq!(page.read_with(cx, |page, _| page.selections), 1);
        assert!(cx.debug_bounds("maintenance-page-options").is_none());
        let trigger = cx.debug_bounds("test-view-menu").unwrap();
        cx.simulate_click(trigger.center(), gpui::Modifiers::default());
        cx.run_until_parked();
        let menu = cx.debug_bounds("maintenance-page-options").unwrap();
        cx.simulate_click(
            gpui::point(menu.center().x, menu.top() + gpui::px(22.)),
            gpui::Modifiers::default(),
        );
        cx.run_until_parked();
        assert_eq!(
            page.read_with(cx, |page, _| page.selected),
            RecoveryPage::Startup
        );
        assert_eq!(page.read_with(cx, |page, _| page.selections), 2);
        assert!(cx.debug_bounds("maintenance-page-options").is_none());
    }

    struct PageTest {
        width: f32,
        view: MaintenanceView,
        scroll: ScrollHandle,
        logs: LogFrames,
    }

    struct RestoreBandTest;

    impl Render for RestoreBandTest {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let labels = [
                "Review restore",
                "Upgrade backup",
                "Restore database",
                "Cancel",
                "Copy database report",
            ];
            let actions = labels.into_iter().map(|label| {
                Button::styled(label, ControlStyle::Secondary)
                    .label(label)
                    .a11y_label(label)
                    .into_any_element()
            });
            let page = MaintenancePage::new(
                "Database tools",
                PageNavigation {
                    view: MaintenanceView::Instructions,
                    scroll: ScrollHandle::default(),
                    select: Rc::new(|_, _, _| {}),
                },
                actions,
                instructions(cx).into_any_element(),
                div().into_any_element(),
            );
            div()
                .w(gpui::px(390.))
                .h(gpui::px(520.))
                .flex()
                .flex_col()
                .child(page.render_layout(MaintenanceLayout::Stacked, cx))
        }
    }

    /// Situational ADR 0074: the restore band scrolls to its final button at 390 px.
    #[gpui::test]
    fn adr_0074_task_001_the_restore_band_scrolls_at_390_px(cx: &mut TestAppContext) {
        cx.update(gpui_component::init);
        let (_, cx) = cx.add_window_view(|_, _| RestoreBandTest);
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let band = cx.debug_bounds("maintenance-actions").unwrap();
        let initial_last_button = cx.debug_bounds("Copy database report").unwrap();
        assert!(initial_last_button.bottom() > band.bottom());
        cx.simulate_event(ScrollWheelEvent {
            position: band.center(),
            delta: ScrollDelta::Pixels(gpui::point(gpui::px(0.), gpui::px(-1000.))),
            modifiers: gpui::Modifiers::default(),
            touch_phase: TouchPhase::Moved,
        });
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let band = cx.debug_bounds("maintenance-actions").unwrap();
        let last_button = cx.debug_bounds("Copy database report").unwrap();
        assert!(last_button.top() >= band.top());
        assert!(last_button.bottom() <= band.bottom());
    }

    impl Render for PageTest {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let content = instructions(cx).children((0..30).map(|index| {
                div()
                    .h(gpui::px(60.))
                    .flex_shrink_0()
                    .child(format!("Instruction {index}"))
                    .when(index == 29, |row| {
                        row.debug_selector(|| "last-instruction".to_owned())
                    })
            }));
            let commands: Vec<AnyElement> = (0..12)
                .map(|index| {
                    Button::styled(
                        SharedString::from(format!("test-action-{index}")),
                        ControlStyle::Secondary,
                    )
                    .label(format!("Action {index}"))
                    .a11y_label(format!("Test action {index}"))
                    .into_any_element()
                })
                .collect();
            let entity = cx.weak_entity();
            let page = MaintenancePage::new(
                "Database tools",
                PageNavigation {
                    view: self.view,
                    scroll: self.scroll.clone(),
                    select: Rc::new(move |view, _, cx| {
                        let _ = entity.update(cx, |this, cx| {
                            this.view = view;
                            cx.notify();
                        });
                    }),
                },
                commands,
                content.into_any_element(),
                LogFrame::new(
                    &self.logs,
                    LogSource::Database,
                    "Database result\n".repeat(100),
                )
                .fill()
                .into_any_element(),
            );
            div()
                .w(gpui::px(self.width))
                .h(gpui::px(520.))
                .flex()
                .flex_col()
                .child(page.render_layout(MaintenanceLayout::for_width(self.width), cx))
        }
    }

    /// Situational ADR 0074: wheel events cannot move a hidden instruction page through a report.
    #[gpui::test]
    fn adr_0074_reports_do_not_capture_instruction_scrolling(cx: &mut TestAppContext) {
        cx.update(gpui_component::init);
        for width in [438., 1050.] {
            let (page, cx) = cx.add_window_view(|_, _| PageTest {
                width,
                view: MaintenanceView::Instructions,
                scroll: ScrollHandle::default(),
                logs: LogFrames::default(),
            });
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            let actions = cx.debug_bounds("maintenance-actions").unwrap();
            let main = cx.debug_bounds("maintenance-content").unwrap();
            assert!(main.size.width > gpui::px(200.) && main.size.height > gpui::px(200.));
            assert!(main.bottom() <= gpui::px(520.));
            if width < 760. {
                assert!(actions.bottom() <= main.top());
            } else {
                assert!(actions.right() <= main.left());
            }
            cx.simulate_event(ScrollWheelEvent {
                position: main.center(),
                delta: ScrollDelta::Pixels(gpui::point(gpui::px(0.), gpui::px(-3000.))),
                modifiers: gpui::Modifiers::default(),
                touch_phase: TouchPhase::Moved,
            });
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            let last = cx.debug_bounds("last-instruction").unwrap();
            assert!(last.top() < main.bottom() && last.bottom() <= main.bottom());
            let offset = page.read_with(cx, |page, _| page.scroll.offset());
            assert!(offset.y < gpui::px(0.));
            assert_eq!(cx.debug_bounds("maintenance-actions").unwrap(), actions);
            let toggle = cx.debug_bounds("maintenance-view-toggle").unwrap();
            cx.simulate_click(toggle.center(), gpui::Modifiers::default());
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            assert!(cx.debug_bounds("last-instruction").is_none());
            assert_eq!(
                page.read_with(cx, |page, _| page.view),
                MaintenanceView::Report
            );
            cx.simulate_event(ScrollWheelEvent {
                position: main.center(),
                delta: ScrollDelta::Pixels(gpui::point(gpui::px(0.), gpui::px(500.))),
                modifiers: gpui::Modifiers::default(),
                touch_phase: TouchPhase::Moved,
            });
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            assert_eq!(page.read_with(cx, |page, _| page.scroll.offset()), offset);
            assert_eq!(cx.debug_bounds("maintenance-actions").unwrap(), actions);
            let toggle = cx.debug_bounds("maintenance-view-toggle").unwrap();
            cx.simulate_click(toggle.center(), gpui::Modifiers::default());
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            assert_eq!(page.read_with(cx, |page, _| page.scroll.offset()), offset);
            assert_eq!(
                page.read_with(cx, |page, _| page.view),
                MaintenanceView::Instructions
            );
        }
    }
}
