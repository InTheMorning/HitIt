//! Shared Settings form, wrapping control rows and bounded content (ADR 0069).

#![warn(clippy::pedantic)]

use std::rc::Rc;

use gpui::{div, prelude::*, AnyElement, App, Entity, ScrollHandle, SharedString, Window};
use gpui_component::input::{Input, InputState};
use gpui_component::scroll::ScrollableElement;
use gpui_component::Size;

use crate::ui::composites::music_column_scroll::music_column_tracked_scrollbar;
use crate::ui::composites::page_scroll_content::page_scroll_content;
use crate::ui::composites::segmented_control::{Segment, SegmentDisplay, SegmentedControl};
use crate::ui::control_styles::ControlStyle;
use crate::ui::icons::IconName;
use crate::ui::primitives::primary_selection::PrimarySelectionExt as _;
use crate::ui::primitives::Button;
use crate::ui::sizable_bridge::SizableScaled;
use crate::ui::tokens::{color, FontSize, ScaleFactor, SemanticColor, Spacing};
use crate::view_models::maintenance::MaintenanceLayout;
use crate::view_models::settings::{
    SettingsAction, SettingsActionDisplay, SettingsGroup, SettingsGroupControl, SettingsVm,
};
use crate::view_models::startup::StartupAvailability;

pub(crate) type SettingsCallback = Rc<dyn Fn(SettingsAction, &mut Window, &mut App)>;

/// The app root retains each group's page offset while that group is hidden.
#[derive(Default)]
pub(crate) struct SettingsScrollHandles {
    general: ScrollHandle,
    library: ScrollHandle,
    diagnostics: ScrollHandle,
    about: ScrollHandle,
    pages: [ScrollHandle; 5],
}

impl SettingsScrollHandles {
    pub(crate) fn diagnostic_handle(
        &self,
        page: crate::view_models::settings::DiagnosticPage,
    ) -> &ScrollHandle {
        &self.pages[page as usize]
    }

    pub(crate) fn show_start(&self, group: SettingsGroup) {
        self.handle(group).set_offset(gpui::Point::default());
    }

    pub(crate) const fn handle(&self, group: SettingsGroup) -> &ScrollHandle {
        match group {
            SettingsGroup::General => &self.general,
            SettingsGroup::Library => &self.library,
            SettingsGroup::Diagnostics => &self.diagnostics,
            SettingsGroup::About => &self.about,
        }
    }
}

pub(crate) fn settings_frame(
    tab_row: AnyElement,
    navigation: Vec<AnyElement>,
    content: Vec<AnyElement>,
    scroll_handle: &ScrollHandle,
    workspace: bool,
    cx: &App,
) -> AnyElement {
    div()
        .id("settings")
        .text_size(FontSize::Body.scaled(cx))
        .flex()
        .flex_col()
        .flex_1()
        .min_h_0()
        .min_w_0()
        .overflow_hidden()
        .bg(color(cx, SemanticColor::SystemBackground))
        .child(SettingsBars {
            tab_row,
            navigation,
        })
        .child(if workspace {
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_h_0()
                .min_w_0()
                .overflow_hidden()
                .p(Spacing::MD.scaled(cx))
                .children(content)
                .into_any_element()
        } else {
            music_column_tracked_scrollbar(
                div()
                    .id("settings-page")
                    .min_w_0()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .p(Spacing::LG.scaled(cx))
                    .child(
                        page_scroll_content(cx)
                            .gap(Spacing::LG.scaled(cx))
                            .children(content),
                    ),
                scroll_handle,
                None,
            )
        })
        .into_any_element()
}

/// ADR 0093 Decision 3 and ADR 0074: a narrow page is a page below the
/// stacked width of the maintenance pages. ADR 0039: the breakpoint uses the
/// unscaled viewport (CHROME domain).
fn settings_page_is_narrow(window: &Window, cx: &App) -> bool {
    let width =
        f32::from(window.viewport_size().width) / ScaleFactor::current(cx).chrome_multiplier();
    MaintenanceLayout::for_width(width) == MaintenanceLayout::Stacked
}

/// The title line and the tab row of Settings.
#[derive(IntoElement)]
struct SettingsBars {
    tab_row: AnyElement,
    navigation: Vec<AnyElement>,
}

impl RenderOnce for SettingsBars {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // Operator choice, 2026-11-16 (ADR 0093 task 001): on a narrow page,
        // the Diagnostics page menu is on the title line, so a short window
        // keeps space for the task content (ADR 0074). On a wide page, it is
        // at the end of the tab row line, so the title line keeps its height.
        let narrow = settings_page_is_narrow(window, cx);
        let SettingsBars {
            tab_row,
            navigation,
        } = self;
        let (title_menu, row_menu) = if narrow {
            (navigation, Vec::new())
        } else {
            (Vec::new(), navigation)
        };
        div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(Spacing::XS.scaled(cx))
                    .px(Spacing::MD.scaled(cx))
                    .pt(Spacing::MD.scaled(cx))
                    .child(
                        div()
                            .child(settings_heading(SettingsVm::TITLE, cx))
                            .debug_selector(|| "settings-title".to_owned()),
                    )
                    .children(title_menu)
                    .map(|header| {
                        #[cfg(test)]
                        let header = header.debug_selector(|| "settings-header".to_owned());
                        header
                    }),
            )
            // The tab row is directly below the page bar (ADR 0093 Decision 1).
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(Spacing::XS.scaled(cx))
                    .px(Spacing::MD.scaled(cx))
                    .pt(Spacing::XS.scaled(cx))
                    .child(tab_row)
                    .children(row_menu),
            )
    }
}

pub(crate) fn settings_actions(
    actions: Vec<SettingsActionDisplay>,
    callback: &SettingsCallback,
    cx: &App,
) -> AnyElement {
    div()
        .flex()
        .flex_wrap()
        .items_center()
        .gap(Spacing::XS.scaled(cx))
        .children(actions.into_iter().flat_map(|display| {
            let callback = Rc::clone(callback);
            let action = display.action;
            let style = if action == SettingsAction::Save {
                ControlStyle::Primary
            } else if display.selected {
                ControlStyle::SelectedTab
            } else if action.is_destructive() {
                // musicindex ADR 0064 R-ACT-12: "Use Defaults…" has the red
                // tint of ADR 0020 R-ACT-3.
                ControlStyle::Destructive
            } else {
                ControlStyle::Neutral
            };
            let mut button = Button::styled(SharedString::from(display.id), style)
                .label(display.label)
                .a11y_label(display.a11y_label)
                .disabled(display.availability != StartupAvailability::Available)
                .on_activate(move |window, cx| callback(action, window, cx));
            if display.selected {
                button = button.leading_icon(IconName::Check);
            }
            // Each action row holds one button, so no space follows the
            // destructive button (musicindex ADR 0064 R-ACT-8).
            [button.into_any_element()]
        }))
        .into_any_element()
}

pub(crate) fn settings_plain_page(
    content: Vec<AnyElement>,
    scroll: &ScrollHandle,
    cx: &App,
) -> AnyElement {
    div()
        .id("diagnostic-files")
        .flex_1()
        .min_h_0()
        .min_w_0()
        .overflow_y_scroll()
        .track_scroll(scroll)
        .child(
            page_scroll_content(cx)
                .gap(Spacing::SM.scaled(cx))
                .children(content),
        )
        .vertical_scrollbar(scroll)
        .into_any_element()
}

pub(crate) fn settings_heading(label: impl Into<SharedString>, cx: &App) -> AnyElement {
    div()
        .min_w_0()
        .text_size(FontSize::Title2.scaled(cx))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .child(label.into())
        .into_any_element()
}

pub(crate) fn settings_message(message: impl Into<SharedString>, cx: &App) -> AnyElement {
    div()
        .min_w_0()
        .text_size(FontSize::Caption.scaled(cx))
        .text_color(color(cx, SemanticColor::SecondaryLabel))
        .child(message.into())
        .into_any_element()
}

/// The group control below the page bar (ADR 0093 Decision 1). It uses the
/// shared segmented control. A change of the group runs no command.
pub(crate) fn settings_tab_row(
    control: SettingsGroupControl,
    callback: &SettingsCallback,
) -> AnyElement {
    SettingsTabRow {
        control,
        callback: Rc::clone(callback),
    }
    .into_any_element()
}

#[derive(IntoElement)]
struct SettingsTabRow {
    control: SettingsGroupControl,
    callback: SettingsCallback,
}

impl RenderOnce for SettingsTabRow {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // ADR 0093 Decision 3: on a narrow page, the tab row fills the width.
        let narrow = settings_page_is_narrow(window, cx);
        let SettingsTabRow { control, callback } = self;
        div()
            .id("settings-group-control")
            .debug_selector(|| "settings-group-control".to_owned())
            .when(narrow, Styled::w_full)
            .child(
                SegmentedControl::new(control.selected)
                    .fill_width(narrow)
                    .segments(control.segments.into_iter().map(|segment| {
                        Segment::new(SegmentDisplay {
                            id: SharedString::from(segment.id).into(),
                            key: segment.group,
                            label: SharedString::from(segment.label),
                            a11y_label: SharedString::from(segment.a11y_label),
                        })
                    }))
                    .on_select(move |group, window, cx| {
                        callback(SettingsAction::SelectGroup(*group), window, cx);
                    }),
            )
    }
}

/// One section heading above its rows (ADR 0093 Decision 6).
pub(crate) fn settings_section_heading(label: &'static str, cx: &App) -> AnyElement {
    div()
        .min_w_0()
        .text_size(FontSize::Headline.scaled(cx))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .child(label)
        .into_any_element()
}

/// The control of a row with a value and one action. The value is at the
/// start of the control, and the action follows it. An empty value shows
/// "Not set".
pub(crate) fn settings_value_and_action(
    value: AnyElement,
    action: AnyElement,
    cx: &App,
) -> AnyElement {
    div()
        .flex()
        .flex_row()
        .flex_wrap()
        .items_center()
        .justify_end()
        .gap(Spacing::XS.scaled(cx))
        .child(div().min_w_0().child(value))
        .child(action)
        .into_any_element()
}

pub(crate) fn settings_text_input(input: &Entity<InputState>, cx: &App) -> AnyElement {
    div()
        .w_full()
        .min_w_0()
        .flex()
        .flex_row()
        .child(
            Input::new(input)
                .cleanable(true)
                .scaled(Size::Small, cx)
                .flex_1()
                .min_w_0()
                .with_primary_selection(input),
        )
        .into_any_element()
}

pub(crate) fn settings_cached_row(title: String, action: Button, cx: &App) -> AnyElement {
    div()
        .min_w_0()
        .flex()
        .items_center()
        .gap(Spacing::XS.scaled(cx))
        .pl(Spacing::MD.scaled(cx))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_size(FontSize::Caption.scaled(cx))
                .child(title),
        )
        .child(action)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::composites::log_frame::LogFrames;
    use crate::ui::composites::maintenance_forms::{database_tools, DatabaseCallback};
    use crate::ui::composites::maintenance_page::{PageCallback, PageMenu, PageNavigation};
    use crate::ui::composites::startup_report::{startup_report, RecoveryNavigation};
    use crate::ui::tokens::ScaleFactor;
    use crate::view_models::maintenance::{MaintenanceView, RecoveryPage};
    use crate::view_models::startup::database::{DatabaseTask, DatabaseVm};
    use crate::view_models::startup::{StartupAction, StartupReportVm};
    use gpui::{
        AppContext, Context, Render, ScrollDelta, ScrollWheelEvent, TestAppContext, TouchPhase,
    };

    struct ShortWindowTest {
        recovery: bool,
        database: DatabaseVm,
        inputs: [Entity<InputState>; 3],
        logs: LogFrames,
        scroll: ScrollHandle,
    }

    impl Render for ShortWindowTest {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let command: DatabaseCallback = Rc::new(|_, _, _| {});
            let task: PageCallback<DatabaseTask> = Rc::new(|_, _, _| {});
            let page = database_tools(
                &self.database,
                [&self.inputs[0], &self.inputs[1], &self.inputs[2]],
                &command,
                &self.logs,
                PageNavigation {
                    view: self.database.view,
                    select: Rc::new(|_, _, _| {}),
                    scroll: self.scroll.clone(),
                },
                &task,
                cx,
            );
            if self.recovery {
                let command: PageCallback<StartupAction> = Rc::new(|_, _, _| {});
                return startup_report(
                    &StartupReportVm::new(true),
                    &command,
                    None,
                    Some(page),
                    &self.logs,
                    RecoveryNavigation {
                        page: RecoveryPage::Database,
                        select: Rc::new(|_, _, _| {}),
                        startup: PageNavigation {
                            view: MaintenanceView::Instructions,
                            select: Rc::new(|_, _, _| {}),
                            scroll: ScrollHandle::default(),
                        },
                    },
                    cx,
                )
                .into_any_element();
            }
            let mut vm = SettingsVm::default();
            vm.dispatch(SettingsAction::SelectGroup(SettingsGroup::Diagnostics));
            let callback: SettingsCallback = Rc::new(|_, _, _| {});
            div()
                .size_full()
                .flex()
                .flex_col()
                .child(settings_frame(
                    settings_tab_row(vm.group_control(), &callback),
                    vec![PageMenu::new(
                        "test-diagnostics-menu",
                        SettingsVm::DIAGNOSTIC_MENU,
                        vm.diagnostic_navigation(),
                        callback,
                    )
                    .into_any_element()],
                    vec![page],
                    &ScrollHandle::default(),
                    true,
                    cx,
                ))
                .into_any_element()
        }
    }

    /// Situational ADR 0074: complete navigation must leave usable content at short heights.
    #[gpui::test]
    fn adr_0074_short_windows_reserve_space_for_task_content(cx: &mut TestAppContext) {
        cx.update(gpui_component::init);
        for recovery in [false, true] {
            for scale in [ScaleFactor::Medium, ScaleFactor::XLarge] {
                cx.update(|cx| cx.set_global(scale));
                for width in [438., 1407.] {
                    let (page, cx) = cx.add_window_view(|window, cx| ShortWindowTest {
                        recovery,
                        database: DatabaseVm::new(true),
                        inputs: std::array::from_fn(|_| cx.new(|cx| InputState::new(window, cx))),
                        logs: LogFrames::default(),
                        scroll: ScrollHandle::default(),
                    });
                    cx.simulate_resize(gpui::size(gpui::px(width), gpui::px(382.)));
                    cx.run_until_parked();
                    for task in [DatabaseTask::Check, DatabaseTask::Repair] {
                        for view in [MaintenanceView::Instructions, MaintenanceView::Report] {
                            page.update(cx, |page, cx| {
                                page.database.task = task;
                                page.database.view = view;
                                page.database.report = "Database result\n".repeat(100);
                                cx.notify();
                            });
                            cx.update(|window, cx| {
                                let _ = window.draw(cx);
                            });
                            let main = cx.debug_bounds("maintenance-content").unwrap();
                            let actions = cx.debug_bounds("maintenance-actions").unwrap();
                            let header = cx.debug_bounds("settings-header");
                            let toolbar = cx.debug_bounds("maintenance-toolbar");
                            let container = cx.debug_bounds("maintenance-main").unwrap();
                            let toggle = cx.debug_bounds("maintenance-view-toggle").unwrap();
                            assert!(
                                toggle.left() >= container.left()
                                    && toggle.right() <= container.right()
                            );
                            assert!(
                                toggle.top() >= container.top() && toggle.bottom() <= main.top()
                            );
                            if !recovery {
                                let title = cx
                                    .debug_bounds("settings-title")
                                    .expect("Diagnostics retains Settings title");
                                let header = header.unwrap();
                                assert!(
                                    title.top() >= header.top()
                                        && title.bottom() <= header.bottom()
                                );
                            }
                            // ADR 0093 task 001: at XL in a short window, the task
                            // content keeps 60 px at a narrow width (operator
                            // choice A) and 160 px at a wide width (orchestrator
                            // choice A2), both of 2026-11-16. The tab row of
                            // ADR 0093 Decision 1 uses the rest. M is not changed.
                            let minimum = match (width > 760., scale) {
                                (true, ScaleFactor::XLarge) => 160.,
                                (true, _) => 200.,
                                (false, ScaleFactor::XLarge) => 60.,
                                (false, _) => 110.,
                            };
                            assert!(main.size.height >= gpui::px(minimum),
                                "ADR 0074 and ADR 0093 task 001 (operator choice A and orchestrator choice A2, 2026-11-16): the task content needs {minimum} px. Keep the Diagnostics page menu on the title line at a narrow width and on the tab row line at a wide width. Add no line above the content. recovery={recovery} scale={scale:?} width={width} task={task:?} view={view:?}: {main:?} actions={actions:?} header={header:?} toolbar={toolbar:?}");
                            if !recovery && width < 760. {
                                // ADR 0093 Decision 3: on a narrow page, the
                                // tab row fills the width.
                                let row = cx.debug_bounds("settings-group-control").unwrap();
                                let inset = cx.update(|_, cx| Spacing::MD.scaled(cx));
                                let expected = gpui::px(width) - inset * 2.;
                                assert!(
                                    (row.size.width - expected).abs() < gpui::px(1.),
                                    "ADR 0093 Decision 3: on a narrow page, the tab row fills the width. Call SegmentedControl::fill_width(true) below the stacked width. row={row:?} expected={expected:?}"
                                );
                            }
                            assert!(main.bottom() <= gpui::px(382.));
                            assert!(main.right() <= gpui::px(width));
                            if width < 760. {
                                assert!(actions.bottom() <= main.top());
                                // At XL, the 60 px minimum of the operator choice is
                                // less than the action band, so only M compares them.
                                if scale != ScaleFactor::XLarge {
                                    assert!(actions.size.height < main.size.height);
                                }
                            } else {
                                assert!(actions.right() <= main.left());
                            }
                            if view == MaintenanceView::Instructions {
                                cx.simulate_event(ScrollWheelEvent {
                                    position: main.center(),
                                    delta: ScrollDelta::Pixels(gpui::point(
                                        gpui::px(0.),
                                        gpui::px(-3000.),
                                    )),
                                    modifiers: gpui::Modifiers::default(),
                                    touch_phase: TouchPhase::Moved,
                                });
                                cx.update(|window, cx| {
                                    let _ = window.draw(cx);
                                });
                                assert_eq!(
                                    cx.debug_bounds("maintenance-actions").unwrap(),
                                    actions
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn adr_0069_settings_pages_retain_independent_scroll_offsets() {
        let pages = SettingsScrollHandles::default();
        for group in SettingsGroup::ALL {
            assert_eq!(pages.handle(group).offset(), gpui::Point::default());
        }
        // Use the same shared handles as the viewport, without creating a window.
        {
            let visible = pages.handle(SettingsGroup::Diagnostics).clone();
            visible.set_offset(gpui::point(gpui::px(0.0), gpui::px(-480.0)));
        }
        {
            let visible = pages.handle(SettingsGroup::Library).clone();
            visible.set_offset(gpui::point(gpui::px(0.0), gpui::px(-120.0)));
        }
        assert_eq!(
            pages.handle(SettingsGroup::General).offset().y,
            gpui::px(0.0)
        );
        assert_eq!(
            pages.handle(SettingsGroup::Diagnostics).offset().y,
            gpui::px(-480.0)
        );
        assert_eq!(
            pages.handle(SettingsGroup::Library).offset().y,
            gpui::px(-120.0)
        );
        pages
            .handle(SettingsGroup::Library)
            .set_offset(gpui::Point::default());
        assert_eq!(
            pages.handle(SettingsGroup::Diagnostics).offset().y,
            gpui::px(-480.0)
        );
    }
}
