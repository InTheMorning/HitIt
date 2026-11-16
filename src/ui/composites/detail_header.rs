//! Detail-view header — thumbnail + entity badge + title + optional
//! subtitle. Used by the artist / feed / track inspectors.
//!
//! ## Display contract: `DetailHeaderDisplay`

#![warn(clippy::pedantic)]

use std::sync::Arc;

use gpui::{
    div, AnyElement, App, FontWeight, Image, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window,
};

use crate::ui::layouts as layout;
use crate::ui::primitives::{HStack, MultilineText, VStack};
use crate::ui::tokens::{resolve_color, Appearance, FontSize, SemanticColor, Spacing};

use super::tag_badge::{EntityKind, TagBadge, TagBadgeDisplay};
use super::thumbnail::{Thumbnail, ThumbnailSize};

/// The page kinds that have a header title. musicindex ADR 0068 R-TYPE-2 gives
/// the desktop title size of each kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageTitleKind {
    Album,
    Track,
    Playlist,
    Artist,
    Label,
}

impl PageTitleKind {
    /// The title size of the page kind. An album, track or playlist page uses
    /// Large Title (26). An artist or label page uses Display (42). Each
    /// title is bold.
    #[must_use]
    pub const fn size(self) -> FontSize {
        match self {
            Self::Album | Self::Track | Self::Playlist => FontSize::Title,
            Self::Artist | Self::Label => FontSize::Display,
        }
    }
}

#[derive(IntoElement)]
#[must_use]
pub struct DetailHeader {
    display: DetailHeaderDisplay,
    image: Option<Arc<Image>>,
    appearance: Option<Appearance>,
    title_size: FontSize,
    cover_size: ThumbnailSize,
    name_links: Vec<AnyElement>,
    art: Option<AnyElement>,
}

/// Display-ready header facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetailHeaderDisplay {
    pub kind: EntityKind,
    /// Draw the type dot as a hollow ring. A role that the app infers, or that
    /// one side gives, shows this way (musicindex ADR 0033 R-WORD-3 item 2).
    pub hollow: bool,
    /// The type word of the dot. `None` shows the word of the kind.
    pub label: Option<SharedString>,
    pub title: SharedString,
    pub subtitle: Option<SharedString>,
    pub data_rows: Vec<DetailHeaderDataRow>,
}

/// Display-ready metadata row shown under a detail header subtitle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetailHeaderDataRow {
    pub label: SharedString,
    pub value: SharedString,
    pub max_lines: usize,
}

impl DetailHeader {
    pub fn new(display: DetailHeaderDisplay) -> Self {
        Self {
            display,
            image: None,
            appearance: None,
            title_size: FontSize::Title,
            cover_size: ThumbnailSize::Lg,
            name_links: Vec::new(),
            art: None,
        }
    }

    pub fn image(mut self, image: Option<Arc<Image>>) -> Self {
        self.image = image;
        self
    }

    pub fn appearance(mut self, appearance: Appearance) -> Self {
        self.appearance = Some(appearance);
        self
    }

    /// Sets the title's `FontSize`. The default is `FontSize::Title`.
    ///
    /// ADR 0101 Decision 3 and musicindex ADR 0068 R-TYPE-2: a caller passes
    /// [`PageTitleKind::size`] for its page kind. An artist or label page
    /// uses `FontSize::Display`. An album, track or playlist page uses
    /// `FontSize::Title`.
    pub fn title_size(mut self, title_size: FontSize) -> Self {
        self.title_size = title_size;
        self
    }

    /// Sets the cover size. The default is `ThumbnailSize::Lg`.
    ///
    /// ADR 0101 Decision 4: the album page header passes
    /// `ThumbnailSize::XXl`.
    pub fn cover_size(mut self, cover_size: ThumbnailSize) -> Self {
        self.cover_size = cover_size;
        self
    }

    /// Sets the art element that replaces the cover thumbnail, for example
    /// the logo frame of a collection (ADR 0084 Decision 12).
    pub fn art(mut self, art: impl IntoElement) -> Self {
        self.art = Some(art.into_any_element());
        self
    }

    /// Sets the name links under the title, in one row. The screen builds
    /// each link from its view model.
    pub fn name_links(mut self, name_links: Vec<AnyElement>) -> Self {
        self.name_links = name_links;
        self
    }
}

impl RenderOnce for DetailHeader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let title_color = resolve_color(cx, SemanticColor::Label, self.appearance);
        let subtitle_color = resolve_color(cx, SemanticColor::SecondaryLabel, self.appearance);
        let metadata_label_color = resolve_color(cx, SemanticColor::TertiaryLabel, self.appearance);
        let mut badge = TagBadge::new(TagBadgeDisplay {
            kind: self.display.kind,
            label: self.display.label.clone(),
        })
        .hollow(self.display.hollow);
        if let Some(appearance) = self.appearance {
            badge = badge.appearance(appearance);
        }

        let mut text_block = VStack::new()
            .spacing(Spacing::XS)
            .leading()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_start()
                    .mb(Spacing::XS.scaled(cx))
                    .child(badge),
            )
            .child(
                div()
                    .text_size(self.title_size.scaled(cx))
                    .font_weight(FontWeight::BOLD)
                    .text_color(title_color)
                    .child(self.display.title),
            );

        if let Some(sub) = self.display.subtitle {
            text_block = text_block.child(
                div()
                    .text_size(FontSize::Body.scaled(cx))
                    .text_color(subtitle_color)
                    .child(sub),
            );
        }

        if !self.name_links.is_empty() {
            text_block = text_block.child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_center()
                    .gap(Spacing::XS.scaled(cx))
                    .children(self.name_links),
            );
        }

        if !self.display.data_rows.is_empty() {
            let mut metadata = VStack::new().spacing(Spacing::XXS).leading();
            for row in self.display.data_rows {
                metadata = metadata.child(header_data_row(
                    row.label,
                    row.value,
                    row.max_lines.max(1),
                    metadata_label_color,
                    self.appearance,
                    cx,
                ));
            }
            text_block = text_block.child(metadata);
        }

        HStack::new()
            .spacing(Spacing::LG)
            .top()
            .child(self.art.unwrap_or_else(|| {
                Thumbnail::new(self.display.kind, self.cover_size)
                    .image(self.image)
                    .into_any_element()
            }))
            .child(div().flex_1().min_w_0().child(text_block))
    }
}

fn header_data_row(
    label: SharedString,
    value: SharedString,
    max_lines: usize,
    label_color: gpui::Rgba,
    appearance: Option<Appearance>,
    cx: &App,
) -> impl IntoElement {
    let mut value_text = MultilineText::new(value)
        .max_lines(max_lines)
        .size(FontSize::Micro)
        .color(SemanticColor::SecondaryLabel);
    if let Some(appearance) = appearance {
        value_text = value_text.appearance(appearance);
    }

    div()
        .flex()
        .flex_row()
        .gap(Spacing::SM.scaled(cx))
        .min_w_0()
        .child(
            div()
                .w(layout::COMPACT_COLUMN_WIDTH)
                .flex_shrink_0()
                .text_size(FontSize::Micro.scaled(cx))
                .font_weight(FontWeight::MEDIUM)
                .text_color(label_color)
                .child(label),
        )
        .child(div().min_w_0().flex_1().child(value_text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_uses_display_contract() {
        let header = DetailHeader::new(DetailHeaderDisplay {
            kind: EntityKind::Artist,
            hollow: false,
            label: None,
            title: "Artist".into(),
            subtitle: Some("Subtitle".into()),
            data_rows: vec![DetailHeaderDataRow {
                label: "Publisher".into(),
                value: "Label".into(),
                max_lines: 0,
            }],
        });

        assert_eq!(header.display.kind, EntityKind::Artist);
        assert_eq!(header.display.title, SharedString::from("Artist"));
        assert_eq!(
            header.display.subtitle,
            Some(SharedString::from("Subtitle"))
        );
        assert_eq!(header.display.data_rows.len(), 1);
    }

    /// musicindex ADR 0068 R-TYPE-2: an album, a track and a playlist page
    /// title is 26 px. An artist and a label page title is 42 px. Fix: use
    /// `PageTitleKind::size` in the page shell.
    #[test]
    fn page_title_size_follows_the_page_kind() {
        let shared: serde_json::Value =
            serde_json::from_str(include_str!("../../assets/tokens.json")).expect("valid JSON");
        let display = shared["type"]["page-title"]["artist-label"]["display"][0]
            .as_f64()
            .expect("display size");
        let large_title = shared["type"]["desktop"]["large-title"][0]
            .as_f64()
            .expect("large title size");
        for kind in [
            PageTitleKind::Album,
            PageTitleKind::Track,
            PageTitleKind::Playlist,
        ] {
            assert_eq!(kind.size(), FontSize::Title, "{kind:?}");
            assert!((f64::from(f32::from(kind.size().px())) - large_title).abs() < f64::EPSILON);
        }
        for kind in [PageTitleKind::Artist, PageTitleKind::Label] {
            assert_eq!(kind.size(), FontSize::Display, "{kind:?}");
            assert!((f64::from(f32::from(kind.size().px())) - display).abs() < f64::EPSILON);
        }
    }

    /// ADR 0101 Decision 3: the title `FontSize` defaults to `Title`. A
    /// caller sets it to the size of its page kind.
    #[test]
    fn title_size_defaults_to_title_and_is_settable() {
        let header = DetailHeader::new(DetailHeaderDisplay {
            kind: EntityKind::Artist,
            hollow: false,
            label: None,
            title: "Artist".into(),
            subtitle: None,
            data_rows: Vec::new(),
        });
        assert_eq!(header.title_size, FontSize::Title);

        let header = header.title_size(FontSize::Display);
        assert_eq!(header.title_size, FontSize::Display);
    }
}
