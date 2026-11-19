//! Identity-action button composite.
//!
//! Screens own the click behavior; this composite owns the shared visual
//! vocabulary for identity affordances such as website links and Nostr IDs.
//!
//! Accessibility note (ADR 0038 task 005): [`IdentityActionButtonDisplay`]
//! carries an `a11y_label` sourced from a view-model. GPUI 0.2.x does not yet
//! expose an accessibility-label sink on its button widget, so the field is
//! contract-only today.

#![warn(clippy::pedantic)]

use std::time::Duration;

use gpui::{prelude::*, AnyElement, SharedString};

use crate::ui::control_styles::ControlStyle;
use crate::ui::icons::IconName;
use crate::ui::primitives::{Button, Label, LabelVariant};
use crate::ui::tokens::SemanticColor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityActionKind {
    Website,
    Nostr,
    Rss,
    MusicIndex,
}

impl IdentityActionKind {
    const fn label(self) -> &'static str {
        match self {
            Self::Website => "Website",
            Self::Nostr => "Nostr",
            Self::Rss => "RSS",
            Self::MusicIndex => "",
        }
    }

    const fn leading_icon(self) -> Option<IconName> {
        match self {
            Self::Website => None,
            Self::Nostr => Some(IconName::Nostr),
            Self::Rss => Some(IconName::Rss),
            Self::MusicIndex => Some(IconName::ExternalLink),
        }
    }

    /// Default screen-reader label when the view-model has no richer text.
    /// The visible label only carries the protocol ("Website", "Nostr",
    /// "RSS"); this expands to a verb so the action reads unambiguously.
    #[must_use]
    pub const fn default_a11y_label(self) -> &'static str {
        match self {
            Self::Website => "Open website",
            Self::Nostr => "Copy Nostr identifier",
            Self::Rss => "Open RSS feed",
            Self::MusicIndex => "Open on musicindex.org",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityActionButtonDisplay {
    pub id: SharedString,
    pub kind: IdentityActionKind,
    pub a11y_label: SharedString,
}

pub fn identity_action_button(display: IdentityActionButtonDisplay) -> Button {
    let IdentityActionButtonDisplay {
        id,
        kind,
        a11y_label,
    } = display;
    // A link pill is in a page header, not in a row, so it keeps its tab stop
    // (ADR 0090 invariant "keyboard reach").
    let mut button = Button::styled(id, ControlStyle::RowAction)
        .keep_in_tab_sequence()
        .label(kind.label())
        .a11y_label(a11y_label);
    if kind == IdentityActionKind::MusicIndex {
        button = button
            .tooltip("Open on musicindex.org")
            .tooltip_show_delay(Duration::ZERO);
    }
    if let Some(icon) = kind.leading_icon() {
        button = button.leading_icon(icon);
    }
    button
}

/// The display contract of [`musicindex_link`] (ADR 0038).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MusicIndexLinkDisplay {
    pub id: SharedString,
    /// The musicindex.org address, or `None` when the page has none.
    pub url: Option<SharedString>,
    /// The muted text in place of the link when `url` is `None`.
    pub unavailable_text: &'static str,
}

/// The musicindex.org link of a page header (musicindex ADR 0085 R-LINK-1).
///
/// With an address, the link is the shared identity button. It opens the
/// address in the system browser. With no address, the header shows the
/// muted text that the view model gives (R-LINK-1 item 5).
#[must_use]
pub fn musicindex_link(display: MusicIndexLinkDisplay) -> AnyElement {
    let MusicIndexLinkDisplay {
        id,
        url,
        unavailable_text,
    } = display;
    match url {
        Some(url) => identity_action_button(IdentityActionButtonDisplay {
            id,
            kind: IdentityActionKind::MusicIndex,
            a11y_label: IdentityActionKind::MusicIndex.default_a11y_label().into(),
        })
        .on_click(move |_, _, _| {
            let _ = open::that(url.as_ref());
        })
        .into_any_element(),
        None => not_in_index_label(unavailable_text),
    }
}

/// The muted text in place of the musicindex.org link (R-LINK-1 item 5).
#[must_use]
pub fn not_in_index_label(text: &'static str) -> AnyElement {
    Label::new(text)
        .variant(LabelVariant::Caption)
        .color(SemanticColor::TertiaryLabel)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nostr_action_uses_brand_icon() {
        assert_eq!(
            IdentityActionKind::Nostr.leading_icon(),
            Some(IconName::Nostr)
        );
    }

    #[test]
    fn rss_action_uses_brand_icon() {
        assert_eq!(IdentityActionKind::Rss.leading_icon(), Some(IconName::Rss));
    }

    #[test]
    fn musicindex_action_uses_external_link_icon() {
        assert_eq!(
            IdentityActionKind::MusicIndex.leading_icon(),
            Some(IconName::ExternalLink)
        );
        assert_eq!(
            IdentityActionKind::MusicIndex.default_a11y_label(),
            "Open on musicindex.org"
        );
    }

    #[test]
    fn website_action_is_text_only_until_catalog_has_link_icon() {
        assert_eq!(IdentityActionKind::Website.leading_icon(), None);
    }

    #[test]
    fn default_a11y_label_disambiguates_action_for_each_kind() {
        assert_eq!(
            IdentityActionKind::Website.default_a11y_label(),
            "Open website"
        );
        assert_eq!(
            IdentityActionKind::Nostr.default_a11y_label(),
            "Copy Nostr identifier"
        );
        assert_eq!(
            IdentityActionKind::Rss.default_a11y_label(),
            "Open RSS feed"
        );
    }
}
