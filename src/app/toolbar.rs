//! Top toolbar rendering (ADR 0089 Decisions 1 and 9, R-TOOLBAR-1).
//!
//! The toolbar holds Back, Forward, the page title and the global search.
//! It has no section tabs: the sidebar holds the sections (ADR 0089
//! Decision 1). The application mark sits at the top of the sidebar.

use std::sync::Arc;

use gpui::{
    div, img, prelude::*, Context, FontWeight, Image, ImageFormat, ObjectFit, SharedString, Styled,
    Window,
};
use gpui_component::input::Input;
use gpui_component::{IconName as InputIconName, Size};

use crate::ui::control_styles::ControlStyle;
use crate::ui::icons::IconName;
use crate::ui::layouts as layout;
use crate::ui::primitives::primary_selection::PrimarySelectionExt as _;
use crate::ui::primitives::{Button as UiButton, Tooltip};
use crate::ui::sizable_bridge::SizableScaled;
use crate::ui::tokens::{color, FontSize, SemanticColor, Size as TokenSize, Spacing};
use crate::view_models::app_toolbar::{
    AppToolbarVm, GlobalSearchDisplay, ToolbarNavigationDisplay,
};

use super::search_field_list::install_field_list_keys;
use super::{TopApp, WorkspaceScreenMount};

fn app_logo() -> Arc<Image> {
    Arc::new(Image::from_bytes(
        ImageFormat::Png,
        include_bytes!("../assets/music_network_logo.png").to_vec(),
    ))
}

/// Renders the application mark at the top of the sidebar (ADR 0089
/// Decision 1).
pub(super) fn render_app_mark(cx: &mut Context<TopApp>) -> gpui::AnyElement {
    let display = AppToolbarVm::new().display();
    let mark_tooltip = Tooltip::new(display.mark_a11y_label);
    let spacing_xs = Spacing::XS.scaled(cx);
    div()
        .flex()
        .flex_row()
        .items_center()
        .p(Spacing::SM.scaled(cx))
        .child(
            div()
                .w(layout::APP_ICON_SIZE)
                .h(layout::APP_ICON_SIZE)
                .rounded(spacing_xs)
                .overflow_hidden()
                .flex_shrink_0()
                .opacity(0.82)
                .id(display.mark_id)
                .tooltip(move |window, cx| mark_tooltip.build(window, cx))
                .child(
                    img(app_logo())
                        .w(layout::APP_ICON_SIZE)
                        .h(layout::APP_ICON_SIZE)
                        .object_fit(ObjectFit::Cover),
                ),
        )
        .into_any_element()
}

/// Renders the toolbar of the content area (ADR 0089 Decision 1, R-TOOLBAR-1):
/// Back, Forward and the page title, then the global search.
pub(super) fn render_toolbar(
    app: &mut TopApp,
    mount: WorkspaceScreenMount,
    window: &Window,
    cx: &mut Context<TopApp>,
) -> gpui::AnyElement {
    let display = AppToolbarVm::new().display();
    let navigation = app.toolbar_navigation(mount, cx);
    let bg_surface = color(cx, SemanticColor::SecondarySystemBackground);
    let border_subtle = color(cx, SemanticColor::Separator);
    let spacing_xs = Spacing::XS.scaled(cx);
    let spacing_md = Spacing::MD.scaled(cx);
    let tab_bar_height = TokenSize::RowLg.px();
    let toolbar_width = window.bounds().size.width;

    div()
        .id(display.id)
        .h(tab_bar_height)
        .flex_shrink_0()
        .bg(bg_surface)
        .border_b_1()
        .border_color(border_subtle)
        .px(spacing_md)
        .flex()
        .flex_row()
        .items_center()
        .gap(spacing_xs)
        .child(render_toolbar_navigation(&navigation, cx))
        .child(render_global_search(
            display.center_id,
            &display.global_search,
            app,
            toolbar_width,
            cx,
        ))
        .into_any_element()
}

/// The Back and Forward controls and the page title at the left of the
/// toolbar (ADR 0089 Decision 9, R-TOOLBAR-1 item 1). Each control renders
/// from its typed availability; the handlers only move the one navigation
/// stack.
fn render_toolbar_navigation(
    display: &ToolbarNavigationDisplay,
    cx: &mut Context<TopApp>,
) -> gpui::AnyElement {
    let mut controls = div()
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::XXS.scaled(cx))
        .flex_shrink_0();
    if let Some(back) = display.back {
        controls = controls.child(
            UiButton::styled(back.id, ControlStyle::ToolbarIcon)
                .leading_icon(IconName::ChevronLeft)
                .tooltip(back.label)
                .a11y_label(back.label)
                .disabled(back.disabled)
                .on_click(cx.listener(|this, _, _window, cx| {
                    this.handle_content_list_back_select(cx);
                })),
        );
    }
    if let Some(forward) = display.forward {
        controls = controls.child(
            UiButton::styled(forward.id, ControlStyle::ToolbarIcon)
                .leading_icon(IconName::ChevronRight)
                .tooltip(forward.label)
                .a11y_label(forward.label)
                .disabled(forward.disabled)
                .on_click(cx.listener(|this, _, _window, cx| {
                    this.handle_content_list_forward_select(cx);
                })),
        );
    }

    let mut group = div()
        .id("app-toolbar-navigation")
        .flex()
        .flex_row()
        .items_center()
        .gap(Spacing::SM.scaled(cx))
        .min_w_0()
        .child(controls);
    if !display.title.is_empty() {
        group = group.child(
            div()
                .id("app-toolbar-title")
                .min_w_0()
                .max_w(TokenSize::ColumnTall.scaled(cx))
                .overflow_hidden()
                .whitespace_nowrap()
                .text_size(FontSize::Body.scaled(cx))
                .font_weight(FontWeight::SEMIBOLD)
                .child(SharedString::from(display.title.clone())),
        );
    }
    group.into_any_element()
}

fn render_global_search(
    center_id: &'static str,
    display: &GlobalSearchDisplay,
    app: &TopApp,
    toolbar_width: gpui::Pixels,
    cx: &mut Context<TopApp>,
) -> gpui::AnyElement {
    let use_compact_search = toolbar_width < layout::APP_TOOLBAR_GLOBAL_SEARCH_COMPACT_BREAKPOINT;

    // TODO(Phase 3): bind placeholder to focused frame descriptor.
    // InputState::set_placeholder requires &mut Window, which is not available in render context.
    // Consider updating placeholder outside render, e.g., on focus_changed event.

    let search_input = install_field_list_keys(
        div()
            .id(display.input_id)
            .relative()
            .flex_1()
            .min_w_0()
            .max_w(TokenSize::ColumnTall.scaled(cx)),
        &app.search_field_list,
    )
    .child(
        Input::new(&app.global_search_input)
            .prefix(InputIconName::Search)
            .cleanable(true)
            .scaled(Size::Small, cx)
            .with_primary_selection(&app.global_search_input),
    )
    .child(app.search_field_list.clone());

    let mut toolbar_search = div()
        .id(center_id)
        .flex_1()
        .min_w_0()
        .flex()
        .items_center()
        .justify_center()
        .gap(Spacing::XS.scaled(cx))
        .overflow_hidden()
        .child(search_input);

    toolbar_search = toolbar_search.child(render_global_search_submit_button(
        display,
        use_compact_search,
        cx,
    ));
    toolbar_search.into_any_element()
}

fn render_global_search_submit_button(
    display: &GlobalSearchDisplay,
    compact: bool,
    cx: &mut Context<TopApp>,
) -> UiButton {
    let mut button = if compact {
        UiButton::styled(display.search_button_id, ControlStyle::ToolbarIcon)
            .leading_icon(IconName::Search)
            .tooltip(display.search_button_a11y_label)
    } else {
        UiButton::styled(display.search_button_id, ControlStyle::Primary)
            .label(display.search_button_label)
    }
    .a11y_label(display.search_button_a11y_label)
    .disabled(display.availability != crate::view_models::startup::StartupAvailability::Available);

    button = button.on_click(cx.listener(|this, _, window, cx| {
        this.submit_global_search(window, cx);
    }));
    button
}
