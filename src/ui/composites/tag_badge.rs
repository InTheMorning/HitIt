//! Entity-type tag badge — a small dot in the entity color, followed by the
//! kind word in a label color, identifying what kind of record (artist,
//! feed, track, …) a card represents.
//!
//! ADR 0083 Decision 2: an entity color shows only as a dot. No text is
//! drawn on an entity color. Light/dark palettes are handled by choosing
//! token colors per [`Appearance`]. We never hand-pick hex. The dot shape
//! and the shared entity values are musicindex ADR 0032 (R-DOT-1, R-DOT-2).

#![warn(clippy::pedantic)]

use gpui::{
    div, App, FontWeight, IntoElement, ParentElement, RenderOnce, Rgba, SharedString, Styled,
    Window,
};

use crate::ui::icons::IconName;
use crate::ui::tokens::{
    color, resolve_color, Appearance, FontSize, Radius, SemanticColor, Spacing, TypeDot,
};
use crate::view_models::publisher_page::PublisherPageVm;
use crate::view_models::track_metadata_grid::TrackMetadataComparisonRole;

/// Domain entity kinds the badge knows how to color.
///
/// Kept as a closed enum so the compiler forces a decision when a new
/// kind is introduced; the legacy string-keyed helpers are bridged via
/// [`EntityKind::from_legacy_str`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind {
    Artist,
    Feed,
    Track,
    Release,
    Recording,
    Playlist,
    /// A live item. Its color is the live red, reserved for live items.
    Live,
    Host,
    Author,
    Label,
    /// The shared `publisher` entity color. The label is "feed" here. A
    /// screen that shows this kind reads musicindex ADR 0033 R-WORD-3 item 3
    /// first, which gives the word "Publisher" to a feed with no role.
    Publisher,
    Producer,
    Network,
    Hosting,
    Sponsor,
    /// Unknown / generic — uses the `Accent` token.
    Generic,
}

impl EntityKind {
    /// Map the legacy `&'static str` keys (e.g. `"feed"`) used throughout
    /// the existing screen code. The keys of the entity types in the shared
    /// `tokens.json` are the same strings. Unknown values fall back to
    /// [`EntityKind::Generic`] so this never panics on stale call sites.
    #[must_use]
    pub fn from_legacy_str(s: &str) -> Self {
        match s {
            "artist" => Self::Artist,
            "feed" => Self::Feed,
            "track" => Self::Track,
            "release" => Self::Release,
            "recording" => Self::Recording,
            "playlist" => Self::Playlist,
            "live" => Self::Live,
            "host" => Self::Host,
            "author" => Self::Author,
            "label" => Self::Label,
            "producer" => Self::Producer,
            "network" => Self::Network,
            "hosting" => Self::Hosting,
            "sponsor" => Self::Sponsor,
            _ => Self::Generic,
        }
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Artist => "Artist",
            Self::Feed => "Feed",
            Self::Track => "Track",
            Self::Release => "Release",
            Self::Recording => "Recording",
            Self::Playlist => "Playlist",
            Self::Live => "Live",
            Self::Host => "Host",
            Self::Author => "Author",
            Self::Label => "Label",
            // ADR 0033 R-WORD-3 item 3: the word of a feed that is definitely
            // not an artist. It is the frame title constant of ADR 0085 Decision 7.
            Self::Publisher => PublisherPageVm::FRAME_TITLE_PUBLISHER,
            Self::Producer => "Producer",
            Self::Network => "Network",
            Self::Hosting => "Hosting",
            Self::Sponsor => "Sponsor",
            Self::Generic => "Item",
        }
    }

    /// Entity-palette token for this kind's identity dot.
    ///
    /// ADR 0083 Decision 2: an entity color marks a stated kind and never
    /// reuses a status, accent, or diff token. `Release` takes the feed
    /// color and `Recording` takes the track color, because `search.html`
    /// groups them in the same parent color. `Generic` states no kind, so
    /// it uses a neutral fill, not an entity color.
    #[must_use]
    pub fn fill_token(self) -> SemanticColor {
        match self {
            Self::Artist => SemanticColor::EntityArtist,
            Self::Feed | Self::Release => SemanticColor::EntityFeed,
            Self::Track | Self::Recording => SemanticColor::EntityTrack,
            Self::Playlist => SemanticColor::EntityPlaylist,
            Self::Live => SemanticColor::EntityLive,
            Self::Host => SemanticColor::EntityHost,
            Self::Author => SemanticColor::EntityAuthor,
            Self::Label => SemanticColor::EntityLabel,
            Self::Publisher => SemanticColor::EntityPublisher,
            Self::Producer => SemanticColor::EntityProducer,
            Self::Network => SemanticColor::EntityNetwork,
            Self::Hosting => SemanticColor::EntityHosting,
            Self::Sponsor => SemanticColor::EntitySponsor,
            Self::Generic => SemanticColor::SystemFill,
        }
    }

    #[must_use]
    pub fn fill_color(self, cx: &App) -> Rgba {
        color(cx, self.fill_token())
    }
}

impl From<&str> for EntityKind {
    fn from(value: &str) -> Self {
        Self::from_legacy_str(value)
    }
}

/// Visual role for metadata provenance/diff states.
///
/// Color and glyph resolve together so comparison state never depends on
/// color alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProvenanceRole {
    Match,
    Different,
    Missing,
}

impl ProvenanceRole {
    #[must_use]
    pub fn color_token(self) -> SemanticColor {
        match self {
            Self::Match => SemanticColor::DiffMatch,
            Self::Different => SemanticColor::DiffDifferent,
            Self::Missing => SemanticColor::DiffMissing,
        }
    }

    #[must_use]
    pub fn color(self, cx: &App) -> Rgba {
        color(cx, self.color_token())
    }

    #[must_use]
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Match => "=",
            Self::Different => "\u{2260}",
            Self::Missing => "\u{2205}",
        }
    }

    #[must_use]
    pub fn accessibility_label(self) -> &'static str {
        match self {
            Self::Match => "matches",
            Self::Different => "different",
            Self::Missing => "missing",
        }
    }
}

impl From<TrackMetadataComparisonRole> for ProvenanceRole {
    fn from(role: TrackMetadataComparisonRole) -> Self {
        match role {
            TrackMetadataComparisonRole::Match => Self::Match,
            TrackMetadataComparisonRole::Different => Self::Different,
            TrackMetadataComparisonRole::Missing => Self::Missing,
        }
    }
}

/// Visual role for general status messages.
///
/// Color and icon resolve together so status does not rely on color alone.
/// ADR 0083 Decision 10: the icon is an `IconName`, drawn as a Lucide icon.
/// A status mark keeps its word beside the icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusRole {
    Success,
    Warning,
    Danger,
}

impl StatusRole {
    #[must_use]
    pub fn color_token(self) -> SemanticColor {
        match self {
            Self::Success => SemanticColor::Success,
            Self::Warning => SemanticColor::Warning,
            Self::Danger => SemanticColor::Danger,
        }
    }

    #[must_use]
    pub fn color(self, cx: &App) -> Rgba {
        color(cx, self.color_token())
    }

    #[must_use]
    pub const fn icon(self) -> IconName {
        match self {
            Self::Success => IconName::Check,
            Self::Warning => IconName::Warning,
            Self::Danger => IconName::Close,
        }
    }
}

#[derive(IntoElement)]
#[must_use]
pub struct TagBadge {
    kind: EntityKind,
    label: Option<SharedString>,
    appearance: Option<Appearance>,
    hollow: bool,
}

/// Display-ready badge fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagBadgeDisplay {
    pub kind: EntityKind,
    pub label: Option<SharedString>,
}

impl TagBadge {
    pub fn new(display: TagBadgeDisplay) -> Self {
        Self {
            kind: display.kind,
            label: display.label,
            appearance: None,
            hollow: false,
        }
    }

    pub fn appearance(mut self, appearance: Appearance) -> Self {
        self.appearance = Some(appearance);
        self
    }

    /// Draw the dot as a ring with no fill. It shows a role that nothing
    /// confirms (musicindex ADR 0032 R-DOT-1, item 4).
    pub fn hollow(mut self, hollow: bool) -> Self {
        self.hollow = hollow;
        self
    }
}

impl RenderOnce for TagBadge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // ADR 0083 Decision 2: the entity color shows only as a dot. The
        // kind word always renders in a label color, never on the entity
        // fill, so color is never the only carrier of the kind.
        let dot_color = resolve_color(cx, self.kind.fill_token(), self.appearance);
        let label_color = resolve_color(cx, SemanticColor::Label, self.appearance);
        let label = self
            .label
            .unwrap_or_else(|| SharedString::from(self.kind.label()));
        let dot_size = TypeDot::size(cx);
        let dot = div()
            .flex_none()
            .w(dot_size)
            .h(dot_size)
            .rounded(Radius::Full.scaled(cx));
        let dot = if self.hollow {
            dot.border(TypeDot::ring_width(cx)).border_color(dot_color)
        } else {
            dot.bg(dot_color)
        };

        div()
            .flex_none()
            .flex()
            .items_center()
            .gap(Spacing::XXS.scaled(cx))
            .child(dot)
            .child(
                div()
                    .text_size(FontSize::Micro.scaled(cx))
                    .font_weight(FontWeight::BOLD)
                    .text_color(label_color)
                    .child(label),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_KINDS: [EntityKind; 16] = [
        EntityKind::Artist,
        EntityKind::Feed,
        EntityKind::Track,
        EntityKind::Release,
        EntityKind::Recording,
        EntityKind::Playlist,
        EntityKind::Live,
        EntityKind::Host,
        EntityKind::Author,
        EntityKind::Label,
        EntityKind::Publisher,
        EntityKind::Producer,
        EntityKind::Network,
        EntityKind::Hosting,
        EntityKind::Sponsor,
        EntityKind::Generic,
    ];

    #[test]
    fn tag_badge_uses_display_contract() {
        let badge = TagBadge::new(TagBadgeDisplay {
            kind: EntityKind::Feed,
            label: Some(SharedString::from("podcast")),
        });

        assert_eq!(badge.kind, EntityKind::Feed);
        assert_eq!(badge.label, Some(SharedString::from("podcast")));
    }

    #[test]
    fn status_roles_resolve_color_and_icon_together() {
        assert_eq!(StatusRole::Success.color_token(), SemanticColor::Success);
        assert_eq!(StatusRole::Warning.color_token(), SemanticColor::Warning);
        assert_eq!(StatusRole::Danger.color_token(), SemanticColor::Danger);
        assert_eq!(StatusRole::Success.icon(), IconName::Check);
        assert_eq!(StatusRole::Warning.icon(), IconName::Warning);
        assert_eq!(StatusRole::Danger.icon(), IconName::Close);
    }

    /// R83-03: each `EntityKind` resolves to an entity token, and no two
    /// kinds with different `search.html` colors share one token.
    #[test]
    fn entity_kind_fill_tokens_match_the_website_color_grouping() {
        use std::collections::HashSet;

        // These kinds each have their own color in the shared entity palette
        // (musicindex ADR 0032 R-DOT-2).
        let distinctly_colored = [
            EntityKind::Artist,
            EntityKind::Feed,
            EntityKind::Track,
            EntityKind::Playlist,
            EntityKind::Live,
            EntityKind::Host,
            EntityKind::Author,
            EntityKind::Label,
            EntityKind::Publisher,
            EntityKind::Producer,
            EntityKind::Network,
            EntityKind::Hosting,
            EntityKind::Sponsor,
        ];
        let mut seen_tokens = HashSet::new();
        for kind in distinctly_colored {
            assert!(
                seen_tokens.insert(kind.fill_token()),
                "{kind:?} must not share its entity token with another \
                 distinctly colored kind"
            );
        }

        // ADR 0083 Decision 1: Release takes Feed's color, and Recording
        // takes Track's color.
        assert_eq!(
            EntityKind::Release.fill_token(),
            EntityKind::Feed.fill_token()
        );
        assert_eq!(
            EntityKind::Recording.fill_token(),
            EntityKind::Track.fill_token()
        );

        // Generic states no kind, so it never collides with a real entity
        // color.
        assert!(!seen_tokens.contains(&EntityKind::Generic.fill_token()));
    }

    /// R83-03 / R83-04: an entity kind never resolves to a status, accent,
    /// or diff token. ADR 0083 Decision 2.
    #[test]
    fn entity_kind_fill_tokens_never_resolve_a_status_accent_or_diff_token() {
        let forbidden = [
            SemanticColor::Success,
            SemanticColor::Warning,
            SemanticColor::Danger,
            SemanticColor::Info,
            SemanticColor::Accent,
            SemanticColor::DiffMatch,
            SemanticColor::DiffDifferent,
            SemanticColor::DiffMissing,
        ];
        for kind in ALL_KINDS {
            assert!(
                !forbidden.contains(&kind.fill_token()),
                "{kind:?} must not resolve a status, accent, or diff token"
            );
        }
    }

    /// musicindex ADR 0032 R-DOT-2: the entity colors and the type dot of this
    /// app match the shared copy in `src/assets/tokens.json`.
    #[test]
    fn entity_colors_and_type_dot_match_the_shared_tokens() {
        let shared: serde_json::Value =
            serde_json::from_str(include_str!("../../assets/tokens.json"))
                .expect("musicindex ADR 0032 R-DOT-2: src/assets/tokens.json is not valid JSON");

        let type_dot = &shared["shape"]["type-dot"];
        assert_dimension_matches("TypeDot::SIZE_PX", TypeDot::SIZE_PX, &type_dot["size-px"]);
        assert_dimension_matches(
            "TypeDot::HOLLOW_RING_PX",
            TypeDot::HOLLOW_RING_PX,
            &type_dot["hollow-ring-px"],
        );

        for (appearance, appearance_key) in
            [(Appearance::Dark, "dark"), (Appearance::Light, "light")]
        {
            let entries = shared["entity"][appearance_key].as_object().expect(
                "musicindex ADR 0032 R-DOT-2: entity.dark and entity.light must be objects",
            );
            for (key, value) in entries {
                let kind = kind_for_shared_key(key);
                assert_ne!(
                    kind,
                    EntityKind::Generic,
                    "musicindex ADR 0032 R-DOT-2: entity type `{key}` in src/assets/tokens.json has no \
                     EntityKind. Add it to kind_for_shared_key in the tests of src/ui/composites/tag_badge.rs, \
                     and add its color to SemanticColor in src/ui/tokens.rs."
                );
                let expected_hex = value
                    .as_str()
                    .expect("musicindex ADR 0032 R-DOT-2: an entity color must be a string");
                assert_color_matches(appearance, key, kind, expected_hex);
            }
        }
    }

    /// Map each entity key of the shared `tokens.json` to its kind. The key
    /// `publisher` maps to `Publisher`, whose label is "feed" (ADR 0085 §1).
    fn kind_for_shared_key(key: &str) -> EntityKind {
        match key {
            "feed" => EntityKind::Feed,
            "track" => EntityKind::Track,
            "playlist" => EntityKind::Playlist,
            "live" => EntityKind::Live,
            "artist" => EntityKind::Artist,
            "host" => EntityKind::Host,
            "author" => EntityKind::Author,
            "label" => EntityKind::Label,
            "publisher" => EntityKind::Publisher,
            "producer" => EntityKind::Producer,
            "network" => EntityKind::Network,
            "hosting" => EntityKind::Hosting,
            "sponsor" => EntityKind::Sponsor,
            _ => EntityKind::Generic,
        }
    }

    fn assert_dimension_matches(name: &str, actual: f32, shared: &serde_json::Value) {
        let expected = shared
            .as_f64()
            .expect("musicindex ADR 0032 R-DOT-2: shape.type-dot must hold numbers");
        assert!(
            (f64::from(actual) - expected).abs() < 1e-9,
            "musicindex ADR 0032 R-DOT-2: {name} is {actual} but shape.type-dot in \
             src/assets/tokens.json gives {expected}. Copy tokens.json from musicindex-assets \
             and change the value in src/ui/tokens.rs."
        );
    }

    fn assert_color_matches(
        appearance: Appearance,
        key: &str,
        kind: EntityKind,
        expected_hex: &str,
    ) {
        let token = kind.fill_token();
        let resolved = token.resolve(appearance);
        let actual = [resolved.r, resolved.g, resolved.b];
        let expected = channels_from_hex(expected_hex);
        for (actual_channel, expected_channel) in actual.iter().zip(expected) {
            assert!(
                (actual_channel - expected_channel).abs() < 0.5 / 255.0,
                "musicindex ADR 0032 R-DOT-2: the {appearance:?} color of entity `{key}` is \
                 SemanticColor::{token:?} in src/ui/tokens.rs, which differs from {expected_hex} in \
                 src/assets/tokens.json. Copy tokens.json from musicindex-assets and change the \
                 value in src/ui/tokens.rs."
            );
        }
    }

    fn channels_from_hex(hex: &str) -> [f32; 3] {
        let digits = hex
            .strip_prefix('#')
            .filter(|digits| digits.len() == 6)
            .expect("musicindex ADR 0032 R-DOT-2: an entity color must be #rrggbb");
        let channel = |range: std::ops::Range<usize>| {
            f32::from(u8::from_str_radix(&digits[range], 16).expect("hex digits")) / 255.0
        };
        [channel(0..2), channel(2..4), channel(4..6)]
    }
}
