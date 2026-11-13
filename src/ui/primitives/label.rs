//! Text primitive — token-driven label.
//!
//! `Label` is a thin wrapper around a styled `div()` that locks the text
//! color to a [`SemanticColor`] and applies a HIG type preset. Use it instead
//! of raw `div().child("…")` whenever a piece of text is part of the design
//! system (which is to say: almost always).

#![warn(clippy::pedantic)]

use gpui::{
    div, prelude::*, App, FontWeight, IntoElement, RenderOnce, SharedString, StyledText, Window,
};

use crate::ui::tokens::{resolve_color, Appearance, FontSize, SemanticColor};

/// HIG type-role presets. The variant chooses both the size and the default
/// foreground token. Callers can override the foreground via [`Label::color`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelVariant {
    /// 20pt semibold — section titles.
    Title,
    /// 15pt semibold — list-item titles, popover headers.
    Headline,
    /// 13pt regular — primary body text.
    Body,
    /// 12pt regular — secondary text, footnotes.
    Caption,
    /// 11pt medium — uppercase labels, badges.
    Micro,
}

#[derive(IntoElement)]
#[must_use]
pub struct Label {
    text: SharedString,
    variant: LabelVariant,
    color: Option<SemanticColor>,
    appearance: Option<Appearance>,
    weight: Option<FontWeight>,
    size: Option<FontSize>,
    truncated: bool,
    // ADR 0097 Decision 4: only the fit probe of a test binary reads it.
    #[cfg(test)]
    break_anywhere: bool,
}

impl Label {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            variant: LabelVariant::Body,
            color: None,
            appearance: None,
            weight: None,
            size: None,
            truncated: false,
            #[cfg(test)]
            break_anywhere: false,
        }
    }

    pub fn variant(mut self, variant: LabelVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn color(mut self, color: SemanticColor) -> Self {
        self.color = Some(color);
        self
    }

    pub fn appearance(mut self, appearance: Appearance) -> Self {
        self.appearance = Some(appearance);
        self
    }

    /// Override the variant's default weight. Mirrors `SwiftUI`
    /// `.fontWeight(.medium)`.
    pub fn weight(mut self, weight: FontWeight) -> Self {
        self.weight = Some(weight);
        self
    }

    /// Override the variant's default size. Mirrors `SwiftUI`
    /// `.font(.caption)` when applied to text.
    pub fn size(mut self, size: FontSize) -> Self {
        self.size = Some(size);
        self
    }

    /// Clamp to a single line with tail truncation. Sets `min_w_0` so
    /// the label collapses correctly inside a flex row. Mirrors
    /// `SwiftUI` `.lineLimit(1).truncationMode(.tail)`.
    pub fn truncated(mut self) -> Self {
        self.truncated = true;
        self
    }

    /// The text can break inside a word, for example a URL or a GUID. The fit
    /// check of a test does not count that break (musicindex ADR 0065 R-FIT-3
    /// item 3, ADR 0097 Decision 4).
    pub fn break_anywhere(self) -> Self {
        #[cfg(test)]
        let self_ = Self {
            break_anywhere: true,
            ..self
        };
        #[cfg(not(test))]
        let self_ = self;
        self_
    }
}

impl RenderOnce for Label {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (default_size, default_weight, default_color) = match self.variant {
            LabelVariant::Title => (FontSize::Title2, FontWeight::SEMIBOLD, SemanticColor::Label),
            LabelVariant::Headline => (
                FontSize::Headline,
                FontWeight::SEMIBOLD,
                SemanticColor::Label,
            ),
            LabelVariant::Body => (FontSize::Body, FontWeight::NORMAL, SemanticColor::Label),
            LabelVariant::Caption => (
                FontSize::Caption,
                FontWeight::NORMAL,
                SemanticColor::SecondaryLabel,
            ),
            LabelVariant::Micro => (
                FontSize::Micro,
                FontWeight::MEDIUM,
                SemanticColor::TertiaryLabel,
            ),
        };
        let size = self.size.unwrap_or(default_size);
        let weight = self.weight.unwrap_or(default_weight);
        let color = self.color.unwrap_or(default_color);
        let mut el = div()
            .text_size(size.scaled(cx))
            .font_weight(weight)
            .text_color(resolve_color(cx, color, self.appearance));
        if self.truncated {
            el = el.min_w_0().truncate();
        }
        #[cfg(test)]
        let name = self.text.clone();
        let text = StyledText::new(self.text);
        #[cfg(test)]
        let probe = super::fit_probe::probe(name, Some(text.layout().clone()), self.break_anywhere);
        let el = el.child(text);
        #[cfg(test)]
        let el = el.child(probe);
        el
    }
}

#[cfg(test)]
mod tests {
    //! Builder behaviour pinned without `Window` / `App`. The render
    //! path itself is exercised by the screen tests; here we only
    //! verify the configuration surface.

    use super::*;

    #[test]
    fn defaults_have_no_overrides() {
        let l = Label::new("hi");
        assert_eq!(l.variant, LabelVariant::Body);
        assert!(l.color.is_none());
        assert!(l.appearance.is_none());
        assert!(l.weight.is_none());
        assert!(l.size.is_none());
        assert!(!l.truncated);
    }

    #[test]
    fn modifiers_set_their_fields() {
        let l = Label::new("hi")
            .variant(LabelVariant::Caption)
            .color(SemanticColor::Label)
            .appearance(Appearance::Dark)
            .weight(FontWeight::MEDIUM)
            .size(FontSize::Micro)
            .truncated();
        assert_eq!(l.variant, LabelVariant::Caption);
        assert_eq!(l.color, Some(SemanticColor::Label));
        assert_eq!(l.appearance, Some(Appearance::Dark));
        assert_eq!(l.weight, Some(FontWeight::MEDIUM));
        assert_eq!(l.size, Some(FontSize::Micro));
        assert!(l.truncated);
    }
}
