//! Workspace-frame shell.
//!
//! ADR 0046 Task 007 keeps existing Library, Search, and Settings screens
//! mounted whole. This shell only arranges workspace frames and delegates frame
//! chrome to the shared `frame_shell` composite.

#![warn(clippy::pedantic)]

use std::rc::Rc;

use gpui::{
    div, prelude::FluentBuilder, AnyElement, App, IntoElement, ParentElement, Pixels, RenderOnce,
    Styled, Window,
};

use crate::ui::composites::{frame_shell, FrameShellSlots, SplitPane};
use crate::ui::icons::{Icon, IconName, IconSize};
use crate::ui::layouts::{
    CONTENT_PANE_DEFAULT_WIDTH, CONTENT_PANE_MIN_WIDTH, MUSIC_DETAIL_MIN_WIDTH,
    WORKSPACE_QUEUE_COLLAPSE_BREAKPOINT, WORKSPACE_SECONDARY_DETAIL_COLLAPSE_BREAKPOINT,
};
use crate::ui::tokens::{resolve_color, FontSize, SemanticColor, Spacing};
use crate::view_models::workspace::{
    FrameHeaderActionDisplay, FrameShellDisplay, MusicColumns, MusicPageContent,
    WorkspaceFrameKind, WorkspaceFrameState, WorkspaceLayout,
};

type WorkspaceHeaderActionHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

type WorkspaceContentPaneResizeHandler = Box<dyn Fn(Pixels, &mut Window, &mut App) + 'static>;
type WorkspaceContentPaneResizeStartHandler = Box<dyn Fn(&mut Window, &mut App) + 'static>;
type WorkspaceContentPaneResizeMoveHandler = Box<dyn Fn(f32, &mut Window, &mut App) + 'static>;

/// Whole-screen content mounts keyed by transitional workspace frame kind.
#[derive(Default)]
#[must_use]
pub(crate) struct WorkspaceSlots {
    content_list: Option<AnyElement>,
    detail: Option<AnyElement>,
    content_list_header_action: Option<FrameHeaderActionDisplay>,
    detail_header_action: Option<FrameHeaderActionDisplay>,
    music_columns: Option<MusicColumns>,
    on_content_list_header_action: Option<WorkspaceHeaderActionHandler>,
    on_detail_header_action: Option<WorkspaceHeaderActionHandler>,
    content_pane_width: Option<Pixels>,
    on_content_pane_resize_start: Option<WorkspaceContentPaneResizeStartHandler>,
    on_content_pane_resize_move: Option<WorkspaceContentPaneResizeMoveHandler>,
    on_content_pane_resize_end: Option<WorkspaceContentPaneResizeHandler>,
}

impl WorkspaceSlots {
    /// Creates an empty slot map.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Supplies the visible Music frame (ADR 0089 Decisions 1 and 5). The
    /// shell shows the list frame or the detail frame from this fact.
    pub(crate) fn music_columns(mut self, columns: MusicColumns) -> Self {
        self.music_columns = Some(columns);
        self
    }

    /// Supplies content for the content-list frame.
    pub(crate) fn content_list(mut self, content: impl IntoElement) -> Self {
        self.content_list = Some(content.into_any_element());
        self
    }

    /// Supplies content for the detail frame (ADR 0084 Decision 1). The
    /// Music mount is the only caller today.
    pub(crate) fn detail_slot(mut self, content: impl IntoElement) -> Self {
        self.detail = Some(content.into_any_element());
        self
    }

    /// Takes the content of the detail frame, so the caller can wrap it
    /// and supply it again (ADR 0084 packet 018: the focus and the action
    /// bar of the Music detail column).
    pub(crate) fn take_detail_slot(&mut self) -> Option<AnyElement> {
        self.detail.take()
    }

    /// Supplies the content-list frame's trailing header action (ADR
    /// 0084 Decision 10 Amendment, 2026-10-19): the Music list column's
    /// "Select" and "Done" button.
    pub(crate) fn content_list_header_action(mut self, display: FrameHeaderActionDisplay) -> Self {
        self.content_list_header_action = Some(display);
        self
    }

    /// Supplies the content-list header action's click callback.
    pub(crate) fn on_content_list_header_action(
        mut self,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_content_list_header_action = Some(Rc::new(handler));
        self
    }

    /// Supplies the detail frame's trailing header action (ADR 0084
    /// packet 018): the Music detail column's "Select" and "Done" button.
    pub(crate) fn detail_header_action(mut self, display: FrameHeaderActionDisplay) -> Self {
        self.detail_header_action = Some(display);
        self
    }

    /// Supplies the detail header action's click callback.
    pub(crate) fn on_detail_header_action(
        mut self,
        handler: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_detail_header_action = Some(Rc::new(handler));
        self
    }

    /// Supplies the content pane width for the `SplitPane`.
    pub(crate) fn content_pane_width(mut self, width: Pixels) -> Self {
        self.content_pane_width = Some(width);
        self
    }

    /// Supplies the content pane resize-start callback.
    pub(crate) fn on_content_pane_resize_start<F>(mut self, handler: F) -> Self
    where
        F: Fn(&mut Window, &mut App) + 'static,
    {
        self.on_content_pane_resize_start = Some(Box::new(handler));
        self
    }

    /// Supplies the content pane resize-move callback.
    pub(crate) fn on_content_pane_resize_move<F>(mut self, handler: F) -> Self
    where
        F: Fn(f32, &mut Window, &mut App) + 'static,
    {
        self.on_content_pane_resize_move = Some(Box::new(handler));
        self
    }

    /// Supplies the content pane resize-end callback.
    pub(crate) fn on_content_pane_resize_end<F>(mut self, handler: F) -> Self
    where
        F: Fn(Pixels, &mut Window, &mut App) + 'static,
    {
        self.on_content_pane_resize_end = Some(Box::new(handler));
        self
    }

    fn take(
        &mut self,
        kind: WorkspaceFrameKind,
        frame: &WorkspaceFrameState,
        cx: &App,
    ) -> AnyElement {
        match kind {
            WorkspaceFrameKind::ContentList => self.content_list.take(),
            WorkspaceFrameKind::Detail => self.detail.take(),
            // ADR 0081 Decision 2: a slot is added when a screen fills it.
            WorkspaceFrameKind::QueueNowPlaying => None,
        }
        .unwrap_or_else(|| placeholder(frame, cx))
    }

    fn header_action_for(&self, kind: WorkspaceFrameKind) -> Option<FrameHeaderActionDisplay> {
        match kind {
            WorkspaceFrameKind::ContentList => self.content_list_header_action.clone(),
            WorkspaceFrameKind::Detail => self.detail_header_action.clone(),
            WorkspaceFrameKind::QueueNowPlaying => None,
        }
    }

    fn header_action_handler_for(
        &self,
        kind: WorkspaceFrameKind,
    ) -> Option<WorkspaceHeaderActionHandler> {
        match kind {
            WorkspaceFrameKind::ContentList => self.on_content_list_header_action.clone(),
            WorkspaceFrameKind::Detail => self.on_detail_header_action.clone(),
            WorkspaceFrameKind::QueueNowPlaying => None,
        }
    }
}

/// Renders a workspace layout using shared frame chrome.
pub(crate) fn render_workspace(
    layout: &WorkspaceLayout,
    slots: WorkspaceSlots,
    _cx: &mut App,
) -> impl IntoElement {
    WorkspaceShell {
        layout: layout.clone(),
        slots,
    }
}

#[derive(IntoElement)]
struct WorkspaceShell {
    layout: WorkspaceLayout,
    slots: WorkspaceSlots,
}

impl RenderOnce for WorkspaceShell {
    #[allow(clippy::too_many_lines)]
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let workspace_width = window.bounds().size.width;
        let mut collapsed_frames = Vec::new();
        let mut content_list_element: Option<AnyElement> = None;
        let mut queue_element: Option<AnyElement> = None;
        let mut detail_element: Option<AnyElement> = None;

        // ADR 0089 Decision 1: only the Music mount supplies the visible
        // Music frame. Settings and Show supply none, so the content-list
        // frame fills the row for them.
        let is_music_layout = self.slots.music_columns.is_some();
        let content_list_frame_id = self
            .layout
            .frames()
            .iter()
            .find(|frame| frame.kind() == WorkspaceFrameKind::ContentList)
            .map(WorkspaceFrameState::id);
        let content_list_navigation =
            content_list_frame_id.and_then(|id| self.layout.frame_nav(id));
        // ADR 0089 Decision 1 and Invariants: the current entry of the one
        // navigation stack decides which frame shows. A list root shows the
        // list frame, and a page shows the detail frame.
        let music_columns = self.slots.music_columns.unwrap_or_else(|| {
            content_list_navigation.map_or(MusicColumns::ListRoot, |nav| {
                MusicColumns::for_entry(nav.current(), MusicPageContent::Loaded)
            })
        });

        for frame in self.layout.frames() {
            let frame_kind = frame.kind();
            if should_collapse_frame(frame_kind, workspace_width, is_music_layout) {
                collapsed_frames.push(frame.title().to_owned());
                continue;
            }
            if is_music_layout {
                let hidden_at_this_width = match frame_kind {
                    WorkspaceFrameKind::ContentList => !music_columns.shows_list(),
                    WorkspaceFrameKind::Detail => !music_columns.shows_detail(),
                    WorkspaceFrameKind::QueueNowPlaying => false,
                };
                if hidden_at_this_width {
                    continue;
                }
            }

            let header_action = self.slots.header_action_for(frame_kind);
            let on_header_action = self.slots.header_action_handler_for(frame_kind);
            let content = self.slots.take(frame.kind(), frame, cx);
            // ADR 0089 Decision 9: Back, Forward and the page title belong to
            // the app toolbar, so no frame shows them.
            let mut display = FrameShellDisplay::from_frame(frame, false);
            if let Some(header_action) = header_action {
                display = display.with_header_action(header_action);
            }
            let mut shell_slots = FrameShellSlots::new().content(content);
            if let Some(handler) = on_header_action {
                shell_slots = shell_slots.on_header_action(move |window, cx| {
                    handler(window, cx);
                });
            }

            let frame_element = frame_shell(display, shell_slots);

            let frame_container = div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .min_h_0()
                .child(frame_element);

            match frame_kind {
                WorkspaceFrameKind::ContentList => {
                    content_list_element = Some(frame_container.into_any_element());
                }
                WorkspaceFrameKind::QueueNowPlaying => {
                    queue_element = Some(frame_container.into_any_element());
                }
                WorkspaceFrameKind::Detail => {
                    detail_element = Some(frame_container.into_any_element());
                }
            }
        }

        // ADR 0089 Decision 1: the Music layout is one row: the one visible
        // page frame, at the remaining width of the content area. The
        // sidebar is not a frame; `TopApp::render` places it.
        let layout_element: AnyElement = if is_music_layout {
            let mut row = div()
                .size_full()
                .flex()
                .flex_row()
                .flex_1()
                .min_h_0()
                .min_w_0()
                .gap(Spacing::SM.scaled(cx))
                .p(Spacing::SM.scaled(cx))
                .overflow_hidden();

            if let Some(content_list) = content_list_element {
                row = row.child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_w_0()
                        .h_full()
                        .min_h_0()
                        .child(content_list),
                );
            }
            if let Some(detail) = detail_element {
                row = row.child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_w(MUSIC_DETAIL_MIN_WIDTH)
                        .h_full()
                        .min_h_0()
                        .child(detail),
                );
            }

            row.into_any_element()
        } else {
            let content_pane_width = self
                .slots
                .content_pane_width
                .unwrap_or(CONTENT_PANE_DEFAULT_WIDTH);

            let has_secondary_frames = queue_element.is_some();

            match content_list_element {
                Some(content) if has_secondary_frames => {
                    let mut split = SplitPane::new("workspace-split")
                        .leading_width(content_pane_width)
                        .leading_min_width(CONTENT_PANE_MIN_WIDTH)
                        .leading(content)
                        .trailing(queue_element.expect("secondary frame exists"));

                    if let Some(handler) = self.slots.on_content_pane_resize_start {
                        split = split.on_resize_start(
                            move |_event: &gpui::MouseDownEvent, window, cx| {
                                handler(window, cx);
                            },
                        );
                    }

                    if let Some(handler) = self.slots.on_content_pane_resize_move {
                        split = split.on_resize_move(
                            move |event: &gpui::MouseMoveEvent, window, cx| {
                                let x = f32::from(event.position.x);
                                handler(x, window, cx);
                            },
                        );
                    }

                    if let Some(handler) = self.slots.on_content_pane_resize_end {
                        split =
                            split.on_resize_end(move |_event: &gpui::MouseUpEvent, window, cx| {
                                handler(content_pane_width, window, cx);
                            });
                    }

                    div()
                        .size_full()
                        .flex()
                        .flex_col()
                        .min_h_0()
                        .min_w_0()
                        .p(Spacing::SM.scaled(cx))
                        .overflow_hidden()
                        .child(split)
                        .into_any_element()
                }
                content_list_element => {
                    let mut fallback_row = div()
                        .size_full()
                        .flex()
                        .flex_row()
                        .flex_1()
                        .min_h_0()
                        .min_w_0()
                        .gap(Spacing::SM.scaled(cx))
                        .p(Spacing::SM.scaled(cx))
                        .overflow_hidden();

                    if let Some(content) = content_list_element {
                        fallback_row = fallback_row.child(content);
                    }
                    if let Some(queue) = queue_element {
                        fallback_row = fallback_row.child(queue);
                    }

                    fallback_row.into_any_element()
                }
            }
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            .min_h_0()
            .min_w_0()
            .child(layout_element)
            .when(!collapsed_frames.is_empty(), |el| {
                el.child(collapsed_frames_hint(&collapsed_frames, cx))
            })
    }
}

fn should_collapse_frame(kind: WorkspaceFrameKind, workspace_width: Pixels, music: bool) -> bool {
    match kind {
        WorkspaceFrameKind::QueueNowPlaying => {
            workspace_width < WORKSPACE_QUEUE_COLLAPSE_BREAKPOINT
        }
        // ADR 0084 Decision 1: the Detail frame appears only in the Music
        // layout, where `MusicColumns` already decides this frame's
        // visibility at the current width. This width-based rule predates
        // that frame having a screen, so it stays inert there.
        WorkspaceFrameKind::Detail => {
            !music && workspace_width < WORKSPACE_SECONDARY_DETAIL_COLLAPSE_BREAKPOINT
        }
        WorkspaceFrameKind::ContentList => false,
    }
}

fn collapsed_frames_hint(frame_titles: &[String], cx: &App) -> AnyElement {
    div()
        .flex()
        .gap(Spacing::XS.scaled(cx))
        .flex_shrink_0()
        .px(Spacing::SM.scaled(cx))
        .pb(Spacing::SM.scaled(cx))
        .text_size(FontSize::Caption.scaled(cx))
        .text_color(resolve_color(cx, SemanticColor::TertiaryLabel, None))
        .child(Icon::new(IconName::ChevronLeft).size(IconSize::Transport))
        .child(format!("Off-screen: {}", frame_titles.join(", ")))
        .into_any_element()
}

fn placeholder(frame: &WorkspaceFrameState, cx: &App) -> AnyElement {
    div()
        .flex()
        .flex_1()
        .min_h_0()
        .items_center()
        .justify_center()
        .text_size(FontSize::Caption.scaled(cx))
        .text_color(resolve_color(cx, SemanticColor::TertiaryLabel, None))
        .child(frame.title().to_owned())
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::should_collapse_frame;
    use crate::ui::layouts::{
        WORKSPACE_QUEUE_COLLAPSE_BREAKPOINT, WORKSPACE_SECONDARY_DETAIL_COLLAPSE_BREAKPOINT,
    };
    use crate::view_models::workspace::WorkspaceFrameKind;
    use gpui::px;

    /// ADR 0084 Decision 1: the width-based Detail collapse rule stays
    /// inert in the Music three-column layout, where `MusicColumns`
    /// decides this frame's visibility instead.
    #[test]
    fn adr_0084_columns_detail_width_collapse_is_inert_in_music_three_column_layout() {
        let narrow = WORKSPACE_SECONDARY_DETAIL_COLLAPSE_BREAKPOINT - px(1.0);
        assert!(!should_collapse_frame(
            WorkspaceFrameKind::Detail,
            narrow,
            true
        ));
        assert!(should_collapse_frame(
            WorkspaceFrameKind::Detail,
            narrow,
            false
        ));
    }

    /// `should_collapse_frame` keeps the Queue frame's own width-based
    /// collapse, in or out of the Music three-column layout.
    #[test]
    fn should_collapse_frame_keeps_queue_breakpoint_in_both_layouts() {
        let narrow = WORKSPACE_QUEUE_COLLAPSE_BREAKPOINT - px(1.0);
        assert!(should_collapse_frame(
            WorkspaceFrameKind::QueueNowPlaying,
            narrow,
            false
        ));
        assert!(should_collapse_frame(
            WorkspaceFrameKind::QueueNowPlaying,
            narrow,
            true
        ));
    }
}
