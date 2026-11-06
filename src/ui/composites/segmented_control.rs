#![warn(clippy::pedantic)]
//! Apple-style segmented control composite.
//!
//! Mutually-exclusive horizontal selector built on the [`Button`] primitive.
//! The selected segment has a tint and an outline (musicindex ADR 0020 R-ACT-1
//! item 3); the rest render plain. Spacing between segments scales with the
//! global UI scale.
//!
//! Accessibility note (ADR 0038 task 005): every segment carries a VM/screen
//! display-contract label distinct from the visible abbreviation when needed.

use std::rc::Rc;

use gpui::{
    App, ClickEvent, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Window,
};

use crate::ui::control_styles::ControlStyle;
use crate::ui::primitives::{Button, HStack};
use crate::ui::tokens::Spacing;

type OnSelect<K> = Rc<dyn Fn(&K, &mut Window, &mut App) + 'static>;

/// The spacing of a [`SegmentedControl`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentedControlStyle {
    /// Settings and inspector-level choices.
    Prominent,
    /// A filter above a list.
    Filter,
}

impl SegmentedControlStyle {
    const fn spacing(self) -> Spacing {
        match self {
            Self::Prominent => Spacing::XS,
            Self::Filter => Spacing::XXS,
        }
    }
}

/// One segment in a [`SegmentedControl`].
pub struct Segment<K: Clone + PartialEq + 'static> {
    pub id: ElementId,
    pub key: K,
    pub label: SharedString,
    pub a11y_label: SharedString,
}

/// Display-ready segment fields for a [`SegmentedControl`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SegmentDisplay<K: Clone + PartialEq + 'static> {
    pub id: ElementId,
    pub key: K,
    pub label: SharedString,
    pub a11y_label: SharedString,
}

impl<K: Clone + PartialEq + 'static> Segment<K> {
    pub fn new(display: SegmentDisplay<K>) -> Self {
        Self {
            id: display.id,
            key: display.key,
            label: display.label,
            a11y_label: display.a11y_label,
        }
    }
}

/// Segmented control — pick exactly one of N options.
#[derive(IntoElement)]
pub struct SegmentedControl<K: Clone + PartialEq + 'static> {
    segments: Vec<Segment<K>>,
    selected: K,
    on_select: Option<OnSelect<K>>,
    style: SegmentedControlStyle,
}

impl<K: Clone + PartialEq + 'static> SegmentedControl<K> {
    pub fn new(selected: K) -> Self {
        Self {
            segments: Vec::new(),
            selected,
            on_select: None,
            style: SegmentedControlStyle::Prominent,
        }
    }

    #[must_use]
    pub const fn filter_style(mut self) -> Self {
        self.style = SegmentedControlStyle::Filter;
        self
    }

    #[must_use]
    pub fn segment(mut self, segment: Segment<K>) -> Self {
        self.segments.push(segment);
        self
    }

    #[must_use]
    pub fn segments(mut self, segments: impl IntoIterator<Item = Segment<K>>) -> Self {
        self.segments.extend(segments);
        self
    }

    #[must_use]
    pub fn on_select(mut self, handler: impl Fn(&K, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
}

impl<K: Clone + PartialEq + 'static> RenderOnce for SegmentedControl<K> {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let on_select = self.on_select;
        let selected = self.selected;
        let style = self.style;
        let mut row = HStack::new().spacing(style.spacing()).center();

        for segment in self.segments {
            let key = segment.key.clone();
            let is_selected = key == selected;
            let role = if is_selected {
                ControlStyle::SelectedTab
            } else {
                ControlStyle::Neutral
            };
            let mut button = Button::styled(segment.id, role)
                .label(segment.label.clone())
                .a11y_label(segment.a11y_label.clone());
            if let Some(handler) = on_select.clone() {
                let key_for_click = key.clone();
                button = button.on_click(move |event: &ClickEvent, window, cx| {
                    let _ = event;
                    handler(&key_for_click, window, cx);
                });
            }
            row = row.child(button);
        }

        row
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_uses_display_contract() {
        let segment = Segment::new(SegmentDisplay {
            id: "scale-medium".into(),
            key: 2_u8,
            label: "M".into(),
            a11y_label: "Medium UI scale".into(),
        });

        assert_eq!(segment.key, 2);
        assert_eq!(segment.label, SharedString::from("M"));
        assert_eq!(segment.a11y_label, SharedString::from("Medium UI scale"));
    }

    #[test]
    fn filter_style_changes_the_spacing() {
        assert_eq!(SegmentedControlStyle::Prominent.spacing(), Spacing::XS);
        assert_eq!(SegmentedControlStyle::Filter.spacing(), Spacing::XXS);
    }
}
