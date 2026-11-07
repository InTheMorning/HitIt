//! Button primitive — Apple HIG-aligned variants and sizes.
//!
//! Implemented natively (no `gpui_component::Button` dependency) so the
//! token system fully owns the visual contract: every color, padding, and
//! radius resolves through [`crate::ui::tokens`]. Click handlers are plain
//! callbacks — the primitive owns no state.
//! Keyboard actions share visible focus chrome (ADRs 0033/0034/0069).
//! Focused action buttons reserve Enter from pane shortcuts (ADR 0071).
//! A button with a label is a pill and an icon button is a circle
//! (musicindex ADR 0020 R-ACT-2).
//!
//! Size selection follows the HIG button rule: every variant ships with
//! ≥ 14pt **semibold** label text so filled / tinted variants qualify as
//! "large text" and meet WCAG AA at the 3:1 threshold (which our
//! `OnAccent` / `OnSuccess` / etc. tokens are designed to clear).

#![warn(clippy::pedantic)]

use std::rc::Rc;

use gpui::{
    div, prelude::*, px, App, ClickEvent, ElementId, FontWeight, IntoElement, KeyDownEvent,
    MouseButton, RenderOnce, Rgba, Window,
};

use crate::ui::control_styles::ControlStyle;
use crate::ui::focus_ring::{focus_ring_shows, focused_control_probe};
use crate::ui::icons::{Icon, IconName, IconSize};
use crate::ui::layouts as layout;
use crate::ui::tokens::{
    resolve_color, Appearance, FontSize, Radius, ScaleFactor, SemanticColor, Size, Spacing,
};

use super::tooltip::Tooltip;

/// HIG button styles.
///
/// * **Filled** — high-emphasis primary action. Solid accent fill.
/// * **Tinted** — medium-emphasis. The accent tint with accent text.
/// * **Plain** — an icon button or a menu item. Transparent background. An icon
///   button is accent. A menu item has the text color, and its icon is accent
///   (musicindex ADR 0043 R-LIST-6 item 3). With a text label and centered content
///   it draws as Neutral (musicindex ADR 0042 R-ACT-6).
/// * **Neutral** — a command that changes no stored data. The text color on the
///   neutral fill (musicindex ADR 0042 R-ACT-5).
/// * **Destructive** — dangerous action. A red label on the danger tint, never
///   a fill (musicindex ADR 0020 R-ACT-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Filled,
    Tinted,
    Plain,
    Neutral,
    Destructive,
}

/// Button visual-height tier. The primitive wraps every size in the shared
/// 44pt minimum hit-target contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    /// 28pt height — toolbar, inline list actions.
    Sm,
    /// 32pt height — default.
    Md,
    /// 40pt height — primary CTAs in dialogs / sheets.
    Lg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ButtonContentAlignment {
    Center,
    Leading,
}

/// Text treatment applied to a button label.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ButtonLabelTreatment {
    /// Render the label without text decoration.
    #[default]
    Plain,
    /// Render the label with strike-through text decoration.
    LineThrough,
}

impl ButtonLabelTreatment {
    const fn line_through(self) -> bool {
        matches!(self, Self::LineThrough)
    }
}

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
type ActivateHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

const BUTTON_KEY_CONTEXT: &str = "ActionButton";

/// A tint grows by this alpha under the pointer.
pub(crate) const TINT_HOVER_ALPHA_STEP: f32 = 0.03;

#[derive(IntoElement)]
#[must_use]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each flag is one independent builder option of the button"
)]
pub struct Button {
    id: ElementId,
    variant: ButtonVariant,
    size: ButtonSize,
    label: Option<gpui::SharedString>,
    description: Option<gpui::SharedString>,
    a11y_label: Option<gpui::SharedString>,
    leading_icon: Option<IconName>,
    narrow_hit_target: bool,
    out_of_tab_sequence: bool,
    compact_inset: bool,
    on_click: Option<ClickHandler>,
    on_activate: Option<ActivateHandler>,
    focus_handle: Option<gpui::FocusHandle>,
    appearance: Option<Appearance>,
    font_size: Option<FontSize>,
    foreground: Option<SemanticColor>,
    border: Option<SemanticColor>,
    control_style: Option<ControlStyle>,
    tooltip: Option<Tooltip>,
    full_width: bool,
    content_alignment: ButtonContentAlignment,
    disabled: bool,
    selected: bool,
    label_treatment: ButtonLabelTreatment,
}

impl Button {
    pub fn new(id: impl Into<ElementId>, variant: ButtonVariant) -> Self {
        Self {
            id: id.into(),
            variant,
            size: ButtonSize::Md,
            label: None,
            description: None,
            a11y_label: None,
            leading_icon: None,
            narrow_hit_target: false,
            out_of_tab_sequence: false,
            compact_inset: false,
            on_click: None,
            on_activate: None,
            focus_handle: None,
            appearance: None,
            font_size: None,
            foreground: None,
            border: None,
            control_style: None,
            tooltip: None,
            full_width: false,
            content_alignment: ButtonContentAlignment::Center,
            disabled: false,
            selected: false,
            label_treatment: ButtonLabelTreatment::Plain,
        }
    }

    /// Secondary text for a menu choice; the shared control grows to fit both lines.
    pub fn description(mut self, description: impl Into<gpui::SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn filled(id: impl Into<ElementId>) -> Self {
        Self::new(id, ButtonVariant::Filled)
    }
    pub fn tinted(id: impl Into<ElementId>) -> Self {
        Self::new(id, ButtonVariant::Tinted)
    }
    pub fn plain(id: impl Into<ElementId>) -> Self {
        Self::new(id, ButtonVariant::Plain)
    }
    pub fn destructive(id: impl Into<ElementId>) -> Self {
        Self::new(id, ButtonVariant::Destructive)
    }

    pub fn label(mut self, label: impl Into<gpui::SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn a11y_label(mut self, label: impl Into<gpui::SharedString>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    pub fn tooltip(mut self, label: impl Into<gpui::SharedString>) -> Self {
        self.tooltip = Tooltip::non_empty(label);
        self
    }

    /// Removes the minimum width of the hit area, for a row of one-letter
    /// buttons that must fit on one line. The minimum height stays.
    /// Keeps the button out of the Tab sequence. Its click and its visual
    /// stay. A second copy of a control uses this, so only one copy is a
    /// tab stop (ADR 0090 Decision 5).
    pub const fn out_of_tab_sequence(mut self) -> Self {
        self.out_of_tab_sequence = true;
        self
    }

    pub const fn narrow_hit_target(mut self) -> Self {
        self.narrow_hit_target = true;
        self
    }

    /// Gives a pill the smaller side space of a menu item, for a menu trigger in
    /// a header that must stay on one line at each UI scale (ADR 0074).
    pub const fn compact_inset(mut self) -> Self {
        self.compact_inset = true;
        self
    }

    pub const fn leading_icon(mut self, icon: IconName) -> Self {
        self.leading_icon = Some(icon);
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn control_style(mut self, style: ControlStyle) -> Self {
        let spec = style.spec();
        self.variant = spec.variant;
        self.size = spec.size;
        self.font_size = Some(spec.font_size);
        self.foreground = spec.foreground;
        self.border = spec.border;
        self.control_style = Some(style);
        self
    }

    pub fn styled(id: impl Into<ElementId>, style: ControlStyle) -> Self {
        Self::new(id, style.spec().variant).control_style(style)
    }

    pub fn danger(self) -> Self {
        self.control_style(ControlStyle::Destructive)
    }

    pub fn full_width(mut self) -> Self {
        self.full_width = true;
        self
    }

    pub fn align_leading(mut self) -> Self {
        self.content_alignment = ButtonContentAlignment::Leading;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub const fn foreground(mut self, color: SemanticColor) -> Self {
        self.foreground = Some(color);
        self
    }

    pub const fn label_treatment(mut self, treatment: ButtonLabelTreatment) -> Self {
        self.label_treatment = treatment;
        self
    }

    pub fn appearance(mut self, appearance: Appearance) -> Self {
        self.appearance = Some(appearance);
        self
    }

    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_click = Some(Rc::new(handler));
        self
    }

    pub fn on_activate<F>(mut self, handler: F) -> Self
    where
        F: Fn(&mut Window, &mut App) + 'static,
    {
        self.on_activate = Some(Rc::new(handler));
        self
    }

    /// Track a caller-owned focus handle for keyboard activation and focus restoration.
    pub fn track_focus(mut self, focus_handle: &gpui::FocusHandle) -> Self {
        self.focus_handle = Some(focus_handle.clone());
        self
    }
}

impl gpui_component::Selectable for Button {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

impl Button {
    fn height(&self, cx: &App) -> gpui::Pixels {
        match self.size {
            ButtonSize::Sm => Size::ButtonSm.scaled(cx),
            ButtonSize::Md => Size::ButtonMd.scaled(cx),
            ButtonSize::Lg => Size::ButtonLg.scaled(cx),
        }
    }

    /// The focus handle of a button in the Tab sequence, and the probe that
    /// gives its bounds while it has the keyboard focus. A keyed handle tells
    /// the render if the button has the focus, so the column can show it (ADR
    /// 0090 invariant "keyboard reach").
    fn focus_for_reveal(
        &self,
        window: &mut Window,
        cx: &mut App,
    ) -> (Option<gpui::FocusHandle>, Option<impl IntoElement>) {
        let focus = self.focus_handle.clone().or_else(|| {
            self.in_tab_sequence().then(|| {
                let key = format!("{:?}-focus", self.id);
                window
                    .use_keyed_state(gpui::SharedString::from(key), cx, |_, cx| cx.focus_handle())
                    .read(cx)
                    .clone()
            })
        });
        let probe = focus
            .as_ref()
            .is_some_and(|focus| focus.is_focused(window) && focus_ring_shows(cx))
            .then(focused_control_probe);
        (focus, probe)
    }

    /// WCAG 2.1.1: each action outside a list row is reachable with Tab. A row
    /// action stays out, or Tab must pass each button of each row, and so does
    /// a letter of the letter bar.
    fn in_tab_sequence(&self) -> bool {
        if self.out_of_tab_sequence {
            return false;
        }
        let row_action = matches!(
            self.control_style,
            Some(ControlStyle::RowAction | ControlStyle::DestructiveRowAction)
        );
        self.on_activate.is_some()
            || self.focus_handle.is_some()
            || (self.on_click.is_some() && !self.narrow_hit_target && !row_action)
    }

    fn min_hit_target(cx: &App) -> gpui::Pixels {
        Size::MinHitTarget.scaled(cx)
    }

    /// R-ACT-2 items 1 and 2: a label gives a pill, and an icon alone gives a
    /// circle. A menu item is a row, so it has the radius of a row
    /// (musicindex ADR 0024).
    const fn radius(&self) -> Radius {
        match self.content_alignment {
            ButtonContentAlignment::Center => Radius::Full,
            ButtonContentAlignment::Leading => Radius::LG,
        }
    }

    /// An icon with no text. Its width is its height, so it is a circle.
    fn is_icon_only(&self) -> bool {
        self.leading_icon.is_some()
            && self.description.is_none()
            && self.label.as_ref().is_none_or(|label| label.is_empty())
    }

    fn border_width(&self) -> gpui::Pixels {
        if self.border.is_some() {
            px(1.0)
        } else {
            px(0.0)
        }
    }

    /// A border takes its width from the inset, so that a selected tab has the
    /// width of the same tab when it is not selected.
    fn with_width(&self, visual: gpui::Div, cx: &App) -> gpui::Div {
        if self.is_icon_only() {
            visual.w(self.height(cx)).flex_none()
        } else {
            visual.px(self.px_inset().scaled(cx) - self.border_width())
        }
    }

    /// A pill has more space at each side than a menu item or a letter of the
    /// letter bar, because the round end cuts into that space.
    fn px_inset(&self) -> Spacing {
        let compact = self.narrow_hit_target
            || self.compact_inset
            || matches!(self.content_alignment, ButtonContentAlignment::Leading);
        match (self.size, compact) {
            (ButtonSize::Sm, true) => Spacing::SM,
            (ButtonSize::Sm, false) | (ButtonSize::Md | ButtonSize::Lg, true) => Spacing::MD,
            (ButtonSize::Md | ButtonSize::Lg, false) => Spacing::LG,
        }
    }

    fn font_size(&self) -> FontSize {
        match self.size {
            ButtonSize::Sm => FontSize::Caption,
            ButtonSize::Md => FontSize::Body,
            ButtonSize::Lg => FontSize::Headline,
        }
    }

    fn effective_tooltip(&self) -> Option<Tooltip> {
        // R-TOOLTIP-1 item 3: a control with a text label has no tooltip.
        if self.label.as_ref().is_some_and(|label| !label.is_empty()) {
            return None;
        }
        if let Some(tooltip) = self.tooltip.clone() {
            return Some(tooltip);
        }

        if !self
            .control_style
            .is_some_and(ControlStyle::prefers_tooltip)
        {
            return None;
        }

        self.a11y_label
            .clone()
            .or_else(|| self.label.clone())
            .and_then(Tooltip::non_empty)
    }

    /// ADR 0039 task 002: the single label line at `font_role` must fit
    /// inside this button's fixed `.h(self.height(cx))` cap at the active
    /// scale step, before it clips silently. Always holds when `description`
    /// is set: that path swaps in `.h_auto()`, which is not a fixed-height
    /// cap. Debug-only — never changes what release builds render. If this
    /// ever fires, bring the measured mismatch back to ADR 0039; never
    /// shrink the font or grow chrome here to silence it.
    fn debug_assert_label_reservation_fits(&self, font_role: FontSize, cx: &App) {
        let border_width = self.border_width();
        debug_assert!(
            self.description.is_some()
                || layout::button_label_reservation(font_role, ScaleFactor::current(cx))
                    <= layout::available_inner_height(self.height(cx), px(0.0), border_width),
            "ADR 0039 (docs/adr/archive/0039-dynamic-type-ramp.md#wrapping-and-fixed-height-reservation): \
             Button {:?} label at {font_role:?} cannot fit inside its fixed height at this scale \
             step. Fix: report the measured mismatch to ADR 0039; do not shrink the font or grow \
             chrome to pass this check.",
            self.size,
        );
    }

    fn visual_base(&self, radius: gpui::Pixels, cx: &App) -> gpui::Div {
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(Spacing::XS.scaled(cx))
            .h(self.height(cx))
            .rounded(radius)
            .font_weight(FontWeight::SEMIBOLD)
    }

    /// A row of a menu or a list of choices: plain, with its content at the
    /// leading edge.
    fn is_menu_item(&self) -> bool {
        matches!(self.variant, ButtonVariant::Plain)
            && matches!(self.content_alignment, ButtonContentAlignment::Leading)
    }

    /// R-LIST-6 item 3: the check mark of a menu item is accent, and each other
    /// icon has the color of its label. A foreground of the caller, such as the
    /// red of a destructive item, stays on its icon too.
    fn icon_color(&self, label_color: Rgba, cx: &App, appearance: Option<Appearance>) -> Rgba {
        if self.is_menu_item() && self.foreground.is_none() {
            resolve_color(cx, SemanticColor::Accent, appearance)
        } else {
            label_color
        }
    }

    /// R-ACT-6 item 1: a button with a text label has a fill. A menu item is a
    /// row, and `Unfilled` names the other items of item 2.
    fn effective_variant(&self) -> ButtonVariant {
        let has_text = self.label.as_ref().is_some_and(|label| !label.is_empty());
        let centered = matches!(self.content_alignment, ButtonContentAlignment::Center);
        match self.variant {
            ButtonVariant::Plain if has_text && centered => match self.control_style {
                Some(ControlStyle::Unfilled) => ButtonVariant::Plain,
                Some(ControlStyle::DestructiveRowAction) => ButtonVariant::Destructive,
                _ => ButtonVariant::Neutral,
            },
            variant => variant,
        }
    }

    fn resolved_colors(&self, cx: &App, appearance: Option<Appearance>) -> (Rgba, Rgba, Rgba) {
        let (mut bg, mut fg, hover_bg) = match self.effective_variant() {
            ButtonVariant::Filled => (
                resolve_color(cx, SemanticColor::Accent, appearance),
                resolve_color(cx, SemanticColor::OnAccent, appearance),
                resolve_color(cx, SemanticColor::AccentHover, appearance),
            ),
            ButtonVariant::Tinted => {
                let tint = resolve_color(cx, SemanticColor::AccentTint, appearance);
                let mut hover = tint;
                hover.a += TINT_HOVER_ALPHA_STEP;
                (
                    tint,
                    resolve_color(cx, SemanticColor::Accent, appearance),
                    hover,
                )
            }
            ButtonVariant::Plain => {
                let mut hover =
                    resolve_color(cx, SemanticColor::SecondarySystemBackground, appearance);
                hover.a = 0.6;
                let foreground = if self.is_menu_item() {
                    SemanticColor::Label
                } else {
                    SemanticColor::Accent
                };
                (
                    gpui::transparent_black().into(),
                    resolve_color(cx, foreground, appearance),
                    hover,
                )
            }
            ButtonVariant::Neutral => (
                resolve_color(cx, SemanticColor::ButtonFill, appearance),
                resolve_color(cx, SemanticColor::Label, appearance),
                resolve_color(cx, SemanticColor::ButtonFillHover, appearance),
            ),
            ButtonVariant::Destructive => {
                let tint = resolve_color(cx, SemanticColor::DangerTint, appearance);
                let mut hover = tint;
                hover.a += TINT_HOVER_ALPHA_STEP;
                (
                    tint,
                    resolve_color(cx, SemanticColor::DangerLabel, appearance),
                    hover,
                )
            }
        };
        if let Some(foreground) = self.foreground {
            fg = resolve_color(cx, foreground, appearance);
        }
        // R-LIST-1 item 3: the current view icon has a tint.
        if self.selected {
            bg = resolve_color(cx, SemanticColor::AccentTint, appearance);
        }
        (bg, fg, hover_bg)
    }
}

impl RenderOnce for Button {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let min_hit_target = Self::min_hit_target(cx);
        let font_role = self.font_size.unwrap_or_else(|| self.font_size());
        let font = font_role.scaled(cx);
        let radius = self.radius().scaled(cx);
        let appearance = self.appearance;

        self.debug_assert_label_reservation_fits(font_role, cx);

        let (bg, fg, hover_bg) = self.resolved_colors(cx, appearance);
        let icon_color = self.icon_color(fg, cx, appearance);

        let label = self.label.clone().unwrap_or_default();
        let on_click = self.on_click.clone();
        let on_activate = self.on_activate.clone();
        let (focus, reveal_probe) = self.focus_for_reveal(window, cx);
        let disabled = self.disabled;
        let full_width = self.full_width;
        let content_alignment = self.content_alignment;
        let label_treatment = self.label_treatment;
        let tooltip = self.effective_tooltip();
        let in_tab_sequence = self.in_tab_sequence();
        let leading_icon = self.leading_icon;
        let icon_only = self.is_icon_only();

        let mut visual = self
            .visual_base(radius, cx)
            .bg(bg)
            .text_color(fg)
            .text_size(font);

        visual = self.with_width(visual, cx);

        visual = match content_alignment {
            ButtonContentAlignment::Center => visual.justify_center(),
            ButtonContentAlignment::Leading => visual.justify_start(),
        };

        if let Some(border) = self.border {
            visual = visual
                .border_1()
                .border_color(resolve_color(cx, border, appearance));
        }

        if full_width {
            visual = visual.w_full();
        }

        let mut hit_target = hit_target_base(self.id)
            .relative()
            .children(reveal_probe)
            .when(!self.narrow_hit_target, |el| el.min_w(min_hit_target))
            .min_h(min_hit_target)
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer();

        if full_width {
            hit_target = hit_target.w_full();
        }

        if let Some(tooltip) = tooltip {
            hit_target = hit_target.tooltip(move |window, cx| tooltip.build(window, cx));
        }

        if disabled {
            hit_target = hit_target.opacity(0.4).cursor_default();
        } else {
            visual = visual.hover(move |s| s.bg(hover_bg));
            if on_click.is_some() || on_activate.is_some() {
                hit_target = hit_target.on_mouse_down(MouseButton::Left, move |_, _, _| {});
            }
            if let Some(handler) = on_activate.clone() {
                hit_target = hit_target.on_click(move |event, window, cx| {
                    // GPUI also clicks on key-up. Key-down below already
                    // owns keyboard activation (ADR 0069).
                    if pointer_activation_click(event) {
                        handler(window, cx);
                    }
                });
            } else if let Some(handler) = on_click {
                hit_target = hit_target.on_click(move |event, window, cx| {
                    handler(event, window, cx);
                });
            }
            // Menu triggers delegate activation to their popover, but keep
            // the shared focus chrome and tab stop (ADR 0074).
            if in_tab_sequence {
                hit_target = keyboard_button_focus(hit_target, appearance, focus, cx)
                    .tab_index(0)
                    .rounded(radius);
            }
            if let Some(handler) = on_activate {
                hit_target = hit_target.on_key_down(move |event, window, cx| {
                    if keyboard_activation_key(event) {
                        cx.stop_propagation();
                        handler(window, cx);
                    }
                });
            }
        }

        if let Some(icon) = leading_icon {
            visual = visual.child(button_icon(icon, icon_color));
        }
        let label = div()
            .when(label_treatment.line_through(), Styled::line_through)
            .child(label);
        // An empty label takes a gap of its own, and that gap moves the icon of
        // an icon button away from the center of its circle.
        if icon_only {
            return hit_target.child(visual);
        }
        if let Some(description) = self.description {
            visual = visual.h_auto().py(Spacing::SM.scaled(cx)).min_w_0();
            hit_target.child(visual.child(button_description(label, description, cx)))
        } else {
            hit_target.child(visual.child(label))
        }
    }
}

/// The hit area of a button with the element id `id`. In a test, the id is
/// also the debug selector, so a test can find the button by its id (ADR
/// 0084 task 030).
fn hit_target_base(id: ElementId) -> gpui::Stateful<gpui::Div> {
    #[cfg(test)]
    let selector = id.to_string();
    let target = div().id(id);
    #[cfg(test)]
    let target = target.debug_selector(move || selector);
    target
}

fn button_icon(icon: IconName, color: Rgba) -> gpui::Div {
    let element = div()
        .flex()
        .flex_none()
        .child(Icon::new(icon).size(IconSize::Transport).color(color));
    #[cfg(test)]
    let element = element.debug_selector(|| "button-icon".to_owned());
    element
}

fn keyboard_button_focus(
    target: gpui::Stateful<gpui::Div>,
    appearance: Option<Appearance>,
    focus_handle: Option<gpui::FocusHandle>,
    cx: &App,
) -> gpui::Stateful<gpui::Div> {
    let focus_ring_width = layout::scaled_dimension(layout::CONTROL_FOCUS_RING_WIDTH, cx);
    // R-FOCUS-1 item 3: a mouse click shows no ring.
    let focus_color = if focus_ring_shows(cx) {
        resolve_color(cx, SemanticColor::Focus, appearance)
    } else {
        gpui::transparent_black().into()
    };
    target
        .key_context(BUTTON_KEY_CONTEXT)
        .border(focus_ring_width)
        .border_color(gpui::transparent_black())
        .focus(move |style| style.border_color(focus_color))
        .when_some(focus_handle, |button, handle| {
            // GPUI skips its automatic tab-stop setup for supplied handles.
            button.track_focus(&handle.tab_index(0).tab_stop(true))
        })
}

fn button_description(
    label: impl IntoElement,
    description: gpui::SharedString,
    cx: &App,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .min_w_0()
        .gap(Spacing::XXS.scaled(cx))
        .child(label)
        .child(
            div()
                .text_size(FontSize::Micro.scaled(cx))
                .font_weight(FontWeight::NORMAL)
                .child(description),
        )
}

fn pointer_activation_click(event: &ClickEvent) -> bool {
    matches!(event, ClickEvent::Mouse(_))
}

fn keyboard_activation_key(event: &KeyDownEvent) -> bool {
    !event.is_held
        && (matches!(event.keystroke.key.as_str(), "enter" | "space")
            || event.keystroke.key_char.as_deref() == Some(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// musicindex ADR 0020 R-ACT-2: each role gives a pill, a menu item keeps
    /// the radius of a row, and an icon alone is a circle.
    #[test]
    fn button_shape_follows_the_shared_rule() {
        for style in [
            ControlStyle::Primary,
            ControlStyle::Secondary,
            ControlStyle::Neutral,
            ControlStyle::Destructive,
            ControlStyle::RowAction,
            ControlStyle::MetadataAction,
            ControlStyle::SelectedTab,
        ] {
            assert_eq!(
                Button::styled("role", style).label("Open").radius(),
                Radius::Full,
                "musicindex ADR 0020 R-ACT-2: {style:?} must be a pill. Fix the radius in \
                 Button::radius, not at the caller."
            );
        }
        assert_eq!(Button::plain("menu").align_leading().radius(), Radius::LG);

        let icon = Button::styled("icon", ControlStyle::ToolbarIcon).leading_icon(IconName::Add);
        assert!(icon.is_icon_only());
        assert!(icon.label("").is_icon_only());
        assert!(!Button::plain("text")
            .leading_icon(IconName::Add)
            .label("Add")
            .is_icon_only());
    }

    /// musicindex ADR 0042 R-ACT-6: each role with a text label has a fill. Only
    /// `Unfilled`, a menu item and an icon button have none. Fix: give the button a
    /// role of R-ACT-4, or name the item in R-ACT-6 item 2.
    #[test]
    fn a_button_with_a_text_label_has_a_fill() {
        for style in [
            ControlStyle::Primary,
            ControlStyle::Secondary,
            ControlStyle::Neutral,
            ControlStyle::CompactAction,
            ControlStyle::Destructive,
            ControlStyle::RowAction,
            ControlStyle::ToolbarIcon,
            ControlStyle::DestructiveRowAction,
            ControlStyle::MetadataAction,
            ControlStyle::SelectedTab,
        ] {
            assert_ne!(
                Button::styled("role", style)
                    .label("Open")
                    .effective_variant(),
                ButtonVariant::Plain,
                "musicindex ADR 0042 R-ACT-6: {style:?} with a text label has no fill."
            );
        }
        assert_eq!(
            Button::plain("plain").label("Open").effective_variant(),
            ButtonVariant::Neutral
        );
        assert_eq!(
            Button::styled("remove", ControlStyle::DestructiveRowAction)
                .label("Remove")
                .effective_variant(),
            ButtonVariant::Destructive
        );
        assert_eq!(
            Button::styled("letter", ControlStyle::Unfilled)
                .label("A")
                .effective_variant(),
            ButtonVariant::Plain
        );
        assert_eq!(
            Button::plain("menu")
                .label("A to Z")
                .align_leading()
                .effective_variant(),
            ButtonVariant::Plain
        );
        assert_eq!(
            Button::styled("icon", ControlStyle::ToolbarIcon)
                .leading_icon(IconName::Add)
                .effective_variant(),
            ButtonVariant::Plain
        );
    }

    /// musicindex ADR 0043 R-LIST-6 item 3: a menu item has the text color and an
    /// accent icon. A destructive item keeps its red on both. Fix: `is_menu_item`.
    #[test]
    fn a_menu_item_has_the_text_color_and_an_accent_icon() {
        let item = Button::plain("menu").label("Newest").align_leading();
        assert!(item.is_menu_item());
        assert!(item.foreground.is_none());

        let centered = Button::plain("centered").label("Open");
        assert!(!centered.is_menu_item());

        let red = Button::styled("remove", ControlStyle::DestructiveRowAction)
            .label("Remove")
            .align_leading();
        assert_eq!(red.foreground, Some(SemanticColor::DangerLabel));
    }

    #[test]
    fn leading_icon_uses_semantic_icon_role() {
        let button = Button::plain("new-playlist").leading_icon(IconName::Add);

        assert_eq!(button.leading_icon, Some(IconName::Add));
    }

    #[test]
    fn menu_buttons_can_align_content_to_leading_edge() {
        let button = Button::plain("playlist-choice")
            .full_width()
            .align_leading();

        assert!(button.full_width);
        assert_eq!(button.content_alignment, ButtonContentAlignment::Leading);
    }

    #[test]
    fn button_carries_contract_accessibility_label() {
        let button = Button::plain("remove").a11y_label("Remove feed from library");

        assert_eq!(
            button.a11y_label,
            Some(gpui::SharedString::from("Remove feed from library"))
        );
    }

    #[test]
    fn row_action_tooltip_uses_accessibility_label() {
        let button = Button::styled("remove", ControlStyle::RowAction)
            .leading_icon(IconName::Close)
            .a11y_label("Remove feed from library");

        assert_eq!(
            button
                .effective_tooltip()
                .expect("row action has tooltip")
                .label(),
            gpui::SharedString::from("Remove feed from library")
        );
    }

    #[test]
    fn label_treatment_records_text_treatment() {
        let button = Button::plain("excluded").label_treatment(ButtonLabelTreatment::LineThrough);

        assert_eq!(button.label_treatment, ButtonLabelTreatment::LineThrough);
    }

    #[test]
    fn foreground_override_records_token_role() {
        let button = Button::plain("neutral").foreground(SemanticColor::SecondaryLabel);

        assert_eq!(button.foreground, Some(SemanticColor::SecondaryLabel));
    }

    #[test]
    fn activate_handler_records_keyboard_activation_path() {
        let button = Button::plain("cycle-filter").on_activate(|_, _| {});

        assert!(button.on_activate.is_some());
    }

    #[test]
    fn adr_0069_keyboard_press_and_synthesized_release_activate_once() {
        for (key, button) in [
            ("enter", gpui::KeyboardButton::Enter),
            ("space", gpui::KeyboardButton::Space),
        ] {
            let press = KeyDownEvent {
                keystroke: gpui::Keystroke::parse(key).unwrap(),
                is_held: false,
                prefer_character_input: false,
            };
            let held = KeyDownEvent {
                is_held: true,
                ..press.clone()
            };
            let release_click = ClickEvent::Keyboard(gpui::KeyboardClickEvent {
                button,
                ..Default::default()
            });
            // One physical press, repeat events, then GPUI's key-up click.
            let activations = [&press, &held, &held]
                .into_iter()
                .filter(|event| keyboard_activation_key(event))
                .count()
                + usize::from(pointer_activation_click(&release_click));
            assert_eq!(activations, 1, "{key} must not toggle twice");
            assert!(pointer_activation_click(&ClickEvent::Mouse(
                gpui::MouseClickEvent::default()
            )));
        }
    }

    struct IconRoot;

    impl gpui::Render for IconRoot {
        fn render(
            &mut self,
            _: &mut gpui::Window,
            _: &mut gpui::Context<Self>,
        ) -> impl gpui::IntoElement {
            div().size_full().child(
                Button::styled("centered", ControlStyle::ToolbarIcon)
                    .leading_icon(IconName::RowsView)
                    .a11y_label("Rows"),
            )
        }
    }

    /// Walk of 2026-11-06: an empty label took a gap, so the icon was off the
    /// center of its circle. Fix: an icon button renders no label.
    #[gpui::test]
    fn an_icon_is_in_the_center_of_its_circle(cx: &mut gpui::TestAppContext) {
        cx.update(gpui_component::init);
        let (_root, cx) = cx.add_window_view(|_, _| IconRoot);
        cx.update(|window, cx| {
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let button = cx.debug_bounds("centered").expect("the button has bounds");
        let icon = cx.debug_bounds("button-icon").expect("the icon has bounds");
        let offset_x = f32::from(button.center().x) - f32::from(icon.center().x);
        let offset_y = f32::from(button.center().y) - f32::from(icon.center().y);
        assert!(
            offset_x.abs() < 0.5 && offset_y.abs() < 0.5,
            "the icon is {offset_x} px and {offset_y} px off the center"
        );
    }

    struct ClickRoot {
        focus: gpui::FocusHandle,
        clicks: std::rc::Rc<std::cell::Cell<usize>>,
    }

    impl gpui::Render for ClickRoot {
        fn render(
            &mut self,
            _: &mut gpui::Window,
            _: &mut gpui::Context<Self>,
        ) -> impl gpui::IntoElement {
            let clicks = self.clicks.clone();
            div().size_full().track_focus(&self.focus).child(
                Button::plain("plain-click")
                    .label("Click")
                    .on_click(move |_, _, _| {
                        clicks.set(clicks.get() + 1);
                    }),
            )
        }
    }

    /// Walk of 2026-11-06: Tab reaches a button with a click handler, and a
    /// mouse click moves the focus to it (ADR 0069 keeps that focus transfer).
    #[gpui::test]
    fn tab_reaches_a_click_button_and_a_click_moves_the_focus(cx: &mut gpui::TestAppContext) {
        cx.update(gpui_component::init);
        let clicks = std::rc::Rc::new(std::cell::Cell::new(0));
        let root_clicks = clicks.clone();
        let (root, cx) = cx.add_window_view(|_, cx| ClickRoot {
            focus: cx.focus_handle(),
            clicks: root_clicks,
        });
        let focus = cx.update(|_, cx| root.read(cx).focus.clone());
        cx.update(|window, cx| {
            window.activate_window();
            window.focus(&focus, cx);
            window.draw(cx).clear(cx);
        });
        cx.run_until_parked();
        let center = cx
            .debug_bounds("plain-click")
            .expect("the button has bounds")
            .center();
        cx.update(|window, cx| {
            window.focus_next(cx);
            window.draw(cx).clear(cx);
        });
        assert!(
            !cx.update(|window, _| focus.is_focused(window)),
            "Tab reaches the button"
        );

        cx.update(|window, cx| {
            window.focus(&focus, cx);
            window.draw(cx).clear(cx);
        });
        cx.simulate_click(center, gpui::Modifiers::none());
        assert_eq!(clicks.get(), 1);
        assert!(
            !cx.update(|window, _| focus.is_focused(window)),
            "a mouse click moves the focus to the button"
        );
    }

    /// R-TOOLTIP-1 item 1: an icon button with an empty label keeps its tooltip.
    #[test]
    fn icon_button_with_an_empty_label_keeps_its_tooltip() {
        let button = Button::styled("download", ControlStyle::RowAction)
            .leading_icon(IconName::CloudDownload)
            .label("")
            .tooltip("Download");

        assert!(button.effective_tooltip().is_some());
    }

    /// R-TOOLTIP-1 item 3: a control with a text label has no tooltip, also
    /// when a caller gives one.
    #[test]
    fn button_with_a_text_label_has_no_tooltip() {
        let button = Button::styled("sort", ControlStyle::ToolbarIcon)
            .label("Sort A-Z")
            .a11y_label("Sort the list")
            .tooltip("Sort the list");

        assert!(button.effective_tooltip().is_none());
    }

    #[test]
    fn non_compact_button_does_not_invent_tooltip() {
        let button = Button::styled("search", ControlStyle::Primary).label("Search");

        assert!(button.effective_tooltip().is_none());
    }
}
