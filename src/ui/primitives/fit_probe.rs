//! ADR 0097 Decision 4: the fit probe of a test binary.
//!
//! A probed primitive adds one `fit_probe::probe` child. The child has no
//! size in the flow and paints nothing. During prepaint it records the bounds
//! of its parent, the content mask and the text layout of a text. A test
//! clears the probes, draws the window and then reads the faults of
//! musicindex ADR 0065 R-FIT-3 with `faults`.

use std::cell::RefCell;

use gpui::{canvas, prelude::*, Bounds, IntoElement, Pixels, SharedString, Size, TextLayout};

/// The tolerance for the rounding of a layout, in pixels.
const EPSILON: f32 = 0.5;

/// What one probed element showed in the last draw.
#[derive(Clone)]
pub(crate) struct FitProbe {
    name: SharedString,
    bounds: Bounds<Pixels>,
    mask: Bounds<Pixels>,
    viewport: Size<Pixels>,
    text: Option<TextLayout>,
    break_anywhere: bool,
    kind: ProbeKind,
}

/// What a probe examines.
#[derive(Clone, PartialEq)]
enum ProbeKind {
    /// A text primitive: the wrap, the bounds and the text width.
    Text,
    /// An interactive control with the key `key`: the bounds and the overlap
    /// with other controls.
    Control { key: SharedString },
    /// The text of the control with the key `owner`: the wrap, and the text
    /// width against the box of the control.
    ControlText { owner: SharedString },
}

thread_local! {
    static PROBES: RefCell<Vec<FitProbe>> = const { RefCell::new(Vec::new()) };
}

/// The probe child of a text primitive. `name` identifies the element in a
/// failure message. `text` is the layout of the text of the element, if any.
pub(crate) fn probe(
    name: SharedString,
    text: Option<TextLayout>,
    break_anywhere: bool,
) -> impl IntoElement {
    record(name, text, break_anywhere, ProbeKind::Text)
}

/// The probe child of the box of an interactive control. `key` is unique to
/// the control, for example its element id.
pub(crate) fn control_probe(name: SharedString, key: SharedString) -> impl IntoElement {
    record(name, None, false, ProbeKind::Control { key })
}

/// The probe child of the text of the control with the key `owner`.
pub(crate) fn control_text_probe(
    name: SharedString,
    owner: SharedString,
    text: TextLayout,
) -> impl IntoElement {
    record(name, Some(text), false, ProbeKind::ControlText { owner })
}

fn record(
    name: SharedString,
    text: Option<TextLayout>,
    break_anywhere: bool,
    kind: ProbeKind,
) -> impl IntoElement {
    canvas(
        move |bounds, window, _| {
            let probe = FitProbe {
                name,
                bounds,
                mask: window.content_mask().bounds,
                viewport: window.viewport_size(),
                text,
                break_anywhere,
                kind,
            };
            PROBES.with(|probes| probes.borrow_mut().push(probe));
        },
        |_, (), _, _| {},
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
}

/// Removes each recorded probe. Call it before the draw that a test examines.
pub(crate) fn clear() {
    PROBES.with(|probes| probes.borrow_mut().clear());
}

/// The fault messages of each probe recorded since the last `clear`.
pub(crate) fn faults() -> Vec<String> {
    let probes = PROBES.with(|probes| probes.borrow().clone());
    let mut messages: Vec<String> = probes.iter().filter_map(FitProbe::fault).collect();
    let controls: Vec<&FitProbe> = probes
        .iter()
        .filter(|probe| matches!(probe.kind, ProbeKind::Control { .. }))
        .collect();
    for text in &probes {
        if let ProbeKind::ControlText { owner } = &text.kind {
            let fault = controls
                .iter()
                .filter(|control| control.kind == ProbeKind::Control { key: owner.clone() })
                .find_map(|control| text.outside_of(control));
            messages.extend(fault);
        }
    }
    for (index, first) in controls.iter().enumerate() {
        for second in &controls[index + 1..] {
            messages.extend(first.overlap(second));
        }
    }
    // GPUI can prepaint one element more than one time in a frame.
    messages.sort();
    messages.dedup();
    messages
}

/// The number of probes recorded since the last `clear`.
pub(crate) fn count() -> usize {
    PROBES.with(|probes| probes.borrow().len())
}

/// Returns the text names that the last frame recorded.
#[cfg(test)]
pub(crate) fn recorded_names() -> Vec<String> {
    PROBES.with(|probes| {
        probes
            .borrow()
            .iter()
            .map(|probe| probe.name.to_string())
            .collect()
    })
}

fn empty(bounds: &Bounds<Pixels>) -> bool {
    bounds.size.width <= gpui::px(0.) || bounds.size.height <= gpui::px(0.)
}

impl FitProbe {
    fn width(&self) -> f32 {
        f32::from(self.viewport.width)
    }

    fn fault(&self) -> Option<String> {
        let width = self.width();
        if let Some(word) = self.word_break() {
            return Some(format!(
                "musicindex ADR 0065 R-FIT-3: at {width} px the label \"{}\" breaks inside the word \"{word}\".\n\
                 Give the label more width, make the text shorter, or let the row wrap between the controls \
                 (musicindex ADR 0020 R-FIT-1 item 2). A URL or a GUID can use Label::break_anywhere (ADR 0097 Decision 4).",
                self.name
            ));
        }
        if matches!(self.kind, ProbeKind::ControlText { .. }) || empty(&self.bounds) {
            return None;
        }
        if empty(&self.mask) {
            // A collapsed area hides its content on purpose.
            return None;
        }
        if let Some(text_width) = self.text_width() {
            let box_width = f32::from(self.bounds.size.width);
            if text_width > box_width + EPSILON {
                return Some(format!(
                    "musicindex ADR 0065 R-FIT-3: at {width} px the text of \"{}\" overflows its box: \
                     the text is {text_width} px wide and the box is {box_width} px wide.\n\
                     Let the text wrap or truncate, make it shorter, or give the box more width (ADR 0097 Decision 4).",
                    self.name
                ));
            }
        }
        let (left, right) = (
            f32::from(self.bounds.left()),
            f32::from(self.bounds.right()),
        );
        let (mask_left, mask_right) = (f32::from(self.mask.left()), f32::from(self.mask.right()));
        let fully_out = right <= mask_left + EPSILON || left >= mask_right - EPSILON;
        let wider = f32::from(self.bounds.size.width) > f32::from(self.mask.size.width) + EPSILON;
        if wider || fully_out {
            return Some(format!(
                "musicindex ADR 0065 R-FIT-3: at {width} px \"{}\" overflows its area: {} px wide from x = {left}, \
                 and the window, panel or scroll area shows x = {mask_left} to {mask_right}.\n\
                 Let the content shrink or wrap at this width, or give the area more width (ADR 0097 Decision 4).",
                self.name,
                f32::from(self.bounds.size.width),
            ));
        }
        let (top, bottom) = (
            f32::from(self.bounds.top()),
            f32::from(self.bounds.bottom()),
        );
        let (mask_top, mask_bottom) = (f32::from(self.mask.top()), f32::from(self.mask.bottom()));
        let shows_vertically = bottom > mask_top + EPSILON && top < mask_bottom - EPSILON;
        let crosses =
            |low: f32, high: f32, edge: f32| low < edge - EPSILON && high > edge + EPSILON;
        let horizontal_cut = shows_vertically
            && (crosses(left, right, mask_left) || crosses(left, right, mask_right));
        let window_height = f32::from(self.viewport.height);
        // An element that its scroll area hides is not on the screen, so
        // the window edge cannot cut it.
        let vertical_cut =
            shows_vertically && (crosses(top, bottom, 0.) || crosses(top, bottom, window_height));
        if horizontal_cut || vertical_cut {
            return Some(format!(
                "musicindex ADR 0065 R-FIT-3: at {width} px an edge cuts \"{}\": it spans x = {left} to {right} and \
                 y = {top} to {bottom}, and the area shows x = {mask_left} to {mask_right} in a window {window_height} px high.\n\
                 Move the control into the area: let the row wrap, let the control shrink, or give the area more width \
                 (ADR 0097 Decision 4).",
                self.name
            ));
        }
        None
    }

    /// The overflow of a control text out of the box of its control.
    fn outside_of(&self, control: &FitProbe) -> Option<String> {
        if empty(&control.bounds) || empty(&control.mask) {
            return None;
        }
        let text_left = f32::from(self.bounds.left());
        let text_right = text_left + self.text_width()?;
        let (left, right) = (
            f32::from(control.bounds.left()),
            f32::from(control.bounds.right()),
        );
        if text_left >= left - EPSILON && text_right <= right + EPSILON {
            return None;
        }
        Some(format!(
            "musicindex ADR 0065 R-FIT-3: at {} px the text of the control \"{}\" overflows the control box: \
             the text spans x = {text_left} to {text_right}, and the control spans x = {left} to {right}.\n\
             Give the control more width, make the text shorter, or let the row wrap (ADR 0097 Decision 4).",
            self.width(),
            self.name
        ))
    }

    /// The overlap of two visible controls. Two boxes overlap when they
    /// intersect by more than 1 px on each axis and neither box holds the
    /// other.
    fn overlap(&self, other: &FitProbe) -> Option<String> {
        let first = self.bounds.intersect(&self.mask);
        let second = other.bounds.intersect(&other.mask);
        if empty(&first) || empty(&second) {
            return None;
        }
        let shared = first.intersect(&second);
        if f32::from(shared.size.width) <= 1. || f32::from(shared.size.height) <= 1. {
            return None;
        }
        if holds(&self.bounds, &other.bounds) || holds(&other.bounds, &self.bounds) {
            return None;
        }
        let (a, b) = if self.name <= other.name {
            (self, other)
        } else {
            (other, self)
        };
        Some(format!(
            "musicindex ADR 0065 R-FIT-3: at {} px the controls \"{}\" and \"{}\" overlap by {} px x {} px.\n\
             Put a gap between the controls, let the row wrap, or give the row more width (ADR 0097 Decision 4).",
            self.width(),
            a.name,
            b.name,
            f32::from(shared.size.width),
            f32::from(shared.size.height),
        ))
    }

    /// The width of the widest line of the text, after the wrap. A space at
    /// the end of a wrapped line does not count.
    fn text_width(&self) -> Option<f32> {
        let layout = self.text.as_ref()?;
        let text = layout.text();
        let mut widest: f32 = 0.;
        for (line, wrapped) in text.split('\n').zip(layout.line_layouts()) {
            let runs = &wrapped.unwrapped_layout.runs;
            let mut glyphs: Vec<(usize, f32)> = runs
                .iter()
                .flat_map(|run| {
                    run.glyphs
                        .iter()
                        .map(|glyph| (glyph.index, f32::from(glyph.position.x)))
                })
                .collect();
            glyphs.sort_by_key(|(index, _)| *index);
            let line_end = f32::from(wrapped.unwrapped_layout.width);
            let x_after = |position: usize| glyphs.get(position + 1).map_or(line_end, |(_, x)| *x);
            let mut starts: Vec<usize> = wrapped
                .wrap_boundaries()
                .iter()
                .map(|boundary| runs[boundary.run_ix].glyphs[boundary.glyph_ix].index)
                .filter_map(|index| glyphs.iter().position(|(glyph, _)| *glyph == index))
                .collect();
            starts.insert(0, 0);
            starts.push(glyphs.len());
            for segment in starts.windows(2) {
                let (first, end) = (segment[0], segment[1]);
                let last_visible = (first..end).rev().find(|&position| {
                    !line[glyphs[position].0..]
                        .chars()
                        .next()
                        .is_some_and(char::is_whitespace)
                });
                if let Some(last) = last_visible {
                    widest = widest.max(x_after(last) - glyphs[first].1);
                }
            }
        }
        Some(widest)
    }

    /// The first word that a soft wrap breaks between two letters or digits.
    fn word_break(&self) -> Option<String> {
        if self.break_anywhere {
            return None;
        }
        let layout = self.text.as_ref()?;
        let text = layout.text();
        for (line, wrapped) in text.split('\n').zip(layout.line_layouts()) {
            for boundary in wrapped.wrap_boundaries() {
                let index =
                    wrapped.unwrapped_layout.runs[boundary.run_ix].glyphs[boundary.glyph_ix].index;
                let before = line[..index].chars().next_back();
                let after = line[index..].chars().next();
                if before.is_some_and(char::is_alphanumeric)
                    && after.is_some_and(char::is_alphanumeric)
                {
                    let start = line[..index]
                        .rfind(|character: char| !character.is_alphanumeric())
                        .map_or(0, |found| found + 1);
                    let end = line[index..]
                        .find(|character: char| !character.is_alphanumeric())
                        .map_or(line.len(), |found| index + found);
                    return Some(line[start..end].to_owned());
                }
            }
        }
        None
    }
}

/// `true` when `outer` holds `inner`, within the tolerance.
fn holds(outer: &Bounds<Pixels>, inner: &Bounds<Pixels>) -> bool {
    f32::from(inner.left()) >= f32::from(outer.left()) - EPSILON
        && f32::from(inner.right()) <= f32::from(outer.right()) + EPSILON
        && f32::from(inner.top()) >= f32::from(outer.top()) - EPSILON
        && f32::from(inner.bottom()) <= f32::from(outer.bottom()) + EPSILON
}

/// A test context that measures text with Figtree (ADR 0097 Decision 3).
///
/// `#[gpui::test]` gives the no-op text system, so a fit test makes its own
/// context with this function and calls `quit` at the end.
pub(crate) fn figtree_test_context() -> gpui::TestAppContext {
    let cx = gpui::TestAppContext::build_with_text_system(
        gpui::TestDispatcher::new(0),
        None,
        std::sync::Arc::new(gpui_wgpu::CosmicTextSystem::new(
            crate::ui::fonts::FIGTREE_FAMILY,
        )),
    );
    cx.update(|cx| crate::ui::fonts::install(cx).expect("the Figtree files load in a test"));
    cx
}

#[cfg(test)]
mod tests {
    //! ADR 0097 §How This Decision Is Checked: each fault has a case that the
    //! finder must report, and a case that it must not report.

    use gpui::{div, px, size, Context, ParentElement, Render, Styled, Window};

    use super::{clear, faults, figtree_test_context};
    use crate::ui::primitives::{
        button::{Button, ButtonVariant},
        label::Label,
    };

    #[derive(Clone, Copy, PartialEq)]
    enum Case {
        LongWord,
        LongWordBreakAnywhere,
        Words,
        ButtonAcrossEdge,
        ButtonOutside,
        ButtonInside,
        NarrowButton,
        NoWrapLabel,
        ButtonsOverlap,
        ButtonsApart,
        ButtonAcrossWindowBottom,
        HiddenButtonAcrossWindowBottom,
    }

    struct FitCase(Case);

    impl Render for FitCase {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
            let root = div()
                .size_full()
                .font_family(crate::ui::fonts::FIGTREE_FAMILY);
            match self.0 {
                Case::LongWord => root.child(div().w(px(60.)).child(Label::new("Recordkeeping"))),
                Case::LongWordBreakAnywhere => root.child(
                    div()
                        .w(px(60.))
                        .child(Label::new("Recordkeeping").break_anywhere()),
                ),
                Case::Words => root.child(div().w(px(60.)).child(Label::new("One two three"))),
                Case::NoWrapLabel => root.child(
                    div()
                        .w(px(30.))
                        .whitespace_nowrap()
                        .child(Label::new("Recordkeeping")),
                ),
                Case::NarrowButton => root.child(div().flex().w(px(30.)).child(
                    Button::new("fit-case-narrow", ButtonVariant::Neutral).label("Recordkeeping"),
                )),
                Case::ButtonsOverlap | Case::ButtonsApart => {
                    let second = if self.0 == Case::ButtonsOverlap {
                        px(40.)
                    } else {
                        px(200.)
                    };
                    root.child(
                        div()
                            .relative()
                            .w(px(380.))
                            .h(px(80.))
                            .child(
                                div().absolute().left(px(10.)).child(
                                    Button::new("fit-case-first", ButtonVariant::Neutral)
                                        .label("First"),
                                ),
                            )
                            .child(
                                div().absolute().left(second).child(
                                    Button::new("fit-case-second", ButtonVariant::Neutral)
                                        .label("Second"),
                                ),
                            ),
                    )
                }
                Case::ButtonAcrossEdge
                | Case::ButtonOutside
                | Case::ButtonInside
                | Case::ButtonAcrossWindowBottom
                | Case::HiddenButtonAcrossWindowBottom => {
                    let offset = match self.0 {
                        Case::ButtonAcrossEdge => px(150.),
                        Case::ButtonOutside => px(260.),
                        _ => px(10.),
                    };
                    // The window is 300 px high. A top of 285 px puts the
                    // button across the window bottom.
                    let (area_height, top) = match self.0 {
                        Case::ButtonAcrossWindowBottom => (px(300.), px(285.)),
                        Case::HiddenButtonAcrossWindowBottom => (px(80.), px(285.)),
                        _ => (px(80.), px(10.)),
                    };
                    root.child(div().w(px(200.)).h(area_height).overflow_hidden().child(
                        div().absolute().left(offset).top(top).child(
                            Button::new("fit-case-button", ButtonVariant::Neutral).label("Save"),
                        ),
                    ))
                }
            }
        }
    }

    fn faults_of(case: Case) -> Vec<String> {
        let mut cx = figtree_test_context();
        cx.update(gpui_component::init);
        let (_, window) = cx.add_window_view(move |_, _| FitCase(case));
        window.simulate_resize(size(px(400.), px(300.)));
        window.run_until_parked();
        clear();
        window.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let found = faults();
        cx.quit();
        found
    }

    #[test]
    fn adr_0097_finds_a_break_inside_a_word() {
        let found = faults_of(Case::LongWord);
        assert!(
            found
                .iter()
                .any(|fault| fault.contains("breaks inside the word \"Recordkeeping\"")),
            "{found:#?}"
        );
        assert!(faults_of(Case::LongWordBreakAnywhere).is_empty());
        assert!(
            faults_of(Case::Words).is_empty(),
            "{:#?}",
            faults_of(Case::Words)
        );
    }

    #[test]
    fn adr_0097_finds_an_edge_cut_and_an_overflow() {
        let across = faults_of(Case::ButtonAcrossEdge);
        assert!(
            across
                .iter()
                .any(|fault| fault.contains("an edge cuts \"Save\"")),
            "{across:#?}"
        );
        let outside = faults_of(Case::ButtonOutside);
        assert!(
            outside
                .iter()
                .any(|fault| fault.contains("\"Save\" overflows its area")),
            "{outside:#?}"
        );
        let inside = faults_of(Case::ButtonInside);
        assert!(inside.is_empty(), "{inside:#?}");
    }

    /// ADR 0084 task 041 walk, 2026-11-16: a row that its scroll area hides
    /// crossed the window bottom, and the probe gave a cut. The window edge
    /// cuts only an element that its area shows.
    #[test]
    fn adr_0097_the_window_edge_cuts_only_a_shown_element() {
        let shown = faults_of(Case::ButtonAcrossWindowBottom);
        assert!(
            shown
                .iter()
                .any(|fault| fault.contains("an edge cuts \"Save\"")),
            "{shown:#?}"
        );
        let hidden = faults_of(Case::HiddenButtonAcrossWindowBottom);
        assert!(hidden.is_empty(), "{hidden:#?}");
    }

    #[test]
    fn adr_0097_finds_text_wider_than_its_control() {
        let found = faults_of(Case::NarrowButton);
        assert!(
            found.iter().any(|fault| fault
                .contains("the text of the control \"Recordkeeping\" overflows the control box")),
            "{found:#?}"
        );
        let label = faults_of(Case::NoWrapLabel);
        assert!(
            label
                .iter()
                .any(|fault| fault.contains("the text of \"Recordkeeping\" overflows its box")),
            "{label:#?}"
        );
    }

    #[test]
    fn adr_0097_finds_an_overlap_of_two_controls() {
        let found = faults_of(Case::ButtonsOverlap);
        assert!(
            found
                .iter()
                .any(|fault| fault.contains("the controls \"First\" and \"Second\" overlap")),
            "{found:#?}"
        );
        let apart = faults_of(Case::ButtonsApart);
        assert!(apart.is_empty(), "{apart:#?}");
    }
}
