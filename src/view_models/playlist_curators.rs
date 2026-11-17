//! The curator facts of the public playlists (musicindex ADR 0081
//! R-CURATOR-4 and R-CURATOR-5, Stophammer ADR 0102).
//!
//! This module gives the Curator row of a playlist page and the groups of
//! the Public playlists list. Each part shows only when the node lists the
//! capability `playlist_links` (Stophammer ADR 0102 §7). The caller then
//! gives a [`PlaylistCurators`] value. Without the capability, the caller
//! gives `None`, and each page shows as before. `MusicIndex` responses are
//! untrusted input. The module has no renderer type.

#![warn(clippy::pedantic)]

use std::collections::BTreeMap;

use crate::api::{Feed, PublisherRelationship, RoleAgreement};
use crate::view_models::artist_identity::UNKNOWN_ARTIST_TEXT;
use crate::view_models::index_name_list::name_sort_key;
use crate::view_models::name_order::compare_names;
use crate::view_models::publisher_page::{header_role_for, HeaderRole};
use crate::view_models::role_words::{known_roles, type_label, KnownRole};

/// The direction of the link row that a playlist gives for the feed that it
/// names (Stophammer ADR 0082 Decision 2).
const MUSIC_TO_PUBLISHER: &str = "music_to_publisher";

/// The label of the Curator row of a playlist page (R-CURATOR-4).
pub(crate) const CURATOR_ROW_LABEL: &str = "Curator";
/// The note after the author text of a playlist with a one-way link
/// (R-CURATOR-4 item 2), and after a row of "Other playlists" with
/// `unconfirmed_publisher` (R-CURATOR-5).
pub(crate) const UNVERIFIED_LINK_NOTE: &str = "(unverified link)";
/// The heading of the last group of the Public playlists list
/// (R-CURATOR-5).
pub(crate) const OTHER_PLAYLISTS_HEADING: &str = "Other playlists";
/// The word of a feed with no known role.
const FEED_WORD: &str = "Feed";

/// The facts of the node that the curator parts read: the rows of
/// `GET /v1/feeds/playlist-publishers`, by feed GUID. The value exists only
/// when the node lists `playlist_links`. The map is empty while the read
/// runs, or when the read failed. The parts then use the link facts only.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct PlaylistCurators {
    publishers: BTreeMap<String, PlaylistPublisher>,
}

/// One row of `GET /v1/feeds/playlist-publishers`.
#[derive(Clone, Debug, PartialEq, Eq)]
struct PlaylistPublisher {
    title: Option<String>,
    agreed_roles: Vec<KnownRole>,
}

impl PlaylistCurators {
    /// The facts of the rows of `GET /v1/feeds/playlist-publishers`. A row
    /// with no feed GUID is ignored.
    #[must_use]
    pub(crate) fn from_publisher_rows(rows: &[Feed]) -> Self {
        let publishers = rows
            .iter()
            .filter_map(|row| {
                let guid = non_empty(row.feed_guid.as_deref())?;
                let roles = row.agreed_roles.as_deref().unwrap_or_default().join(",");
                Some((
                    guid.to_owned(),
                    PlaylistPublisher {
                        title: non_empty(row.title.as_deref()).map(str::to_owned),
                        agreed_roles: known_roles(Some(&roles)),
                    },
                ))
            })
            .collect();
        Self { publishers }
    }

    /// The agreed release roles of the feed `feed_guid`, from the route.
    fn release_roles(&self, feed_guid: &str) -> &[KnownRole] {
        self.publishers
            .get(feed_guid)
            .map_or(&[], |publisher| publisher.agreed_roles.as_slice())
    }

    /// The title of the feed `feed_guid`, from the route.
    fn title(&self, feed_guid: &str) -> Option<&str> {
        self.publishers
            .get(feed_guid)
            .and_then(|publisher| publisher.title.as_deref())
    }
}

/// The dot and the role words of a curator feed (musicindex ADR 0032
/// R-DOT-1). The words show with the dot, so color is never the only
/// signal (ADR 0101 Decision 2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CuratorBadgeVm {
    /// The role of the dot color: the first known role, else `Feed`.
    pub(crate) role: HeaderRole,
    /// The type label, for example "Label · Curator", else "Feed".
    pub(crate) word: String,
    /// The dot is a ring when the feed has no known role.
    pub(crate) hollow: bool,
}

impl CuratorBadgeVm {
    fn of(mut roles: Vec<KnownRole>) -> Self {
        roles.sort();
        roles.dedup();
        match roles.first() {
            Some(&first) => Self {
                role: header_role_for(first),
                word: type_label(&roles),
                hollow: false,
            },
            None => Self {
                role: HeaderRole::Feed,
                word: FEED_WORD.to_owned(),
                hollow: true,
            },
        }
    }
}

/// The Curator row of a playlist page (R-CURATOR-4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PlaylistCuratorVm {
    /// A confirmed link: the row opens the feed page of the curator.
    Feed {
        feed_guid: String,
        title: String,
        badge: CuratorBadgeVm,
        a11y_label: String,
    },
    /// No confirmed link: the author text of the playlist. It ends with
    /// [`UNVERIFIED_LINK_NOTE`] for a one-way link.
    Author { text: String },
}

/// The facts of the Curator row that the playlist response gives with
/// `include=publisher`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct PlaylistCuratorFacts {
    link: Option<CuratorLink>,
    author: Option<String>,
}

/// The `music_to_publisher` row of a playlist.
#[derive(Clone, Debug, PartialEq, Eq)]
struct CuratorLink {
    feed_guid: String,
    title: String,
    two_way: bool,
    confirmed: bool,
    agreed_roles: Vec<KnownRole>,
}

impl PlaylistCuratorFacts {
    /// The facts of the playlist `feed`, read with `include=publisher`.
    #[must_use]
    pub(crate) fn from_feed(feed: &Feed) -> Self {
        let link = feed
            .publisher
            .iter()
            .flatten()
            .find(|row| {
                row.direction.as_deref() == Some(MUSIC_TO_PUBLISHER)
                    && non_empty(row.publisher_feed_guid.as_deref()).is_some()
            })
            .map(|row| CuratorLink::of(row, feed));
        Self {
            link,
            author: author_text(feed),
        }
    }

    /// The Curator row. `None` when the node does not list `playlist_links`
    /// (`curators` is `None`), or when the playlist gives no curator.
    #[must_use]
    pub(crate) fn row(&self, curators: Option<&PlaylistCurators>) -> Option<PlaylistCuratorVm> {
        let curators = curators?;
        if let Some(link) = self.link.as_ref().filter(|link| link.confirmed) {
            let mut roles = curators.release_roles(&link.feed_guid).to_vec();
            roles.extend(link.agreed_roles.iter().copied());
            return Some(PlaylistCuratorVm::Feed {
                feed_guid: link.feed_guid.clone(),
                title: link.title.clone(),
                badge: CuratorBadgeVm::of(roles),
                a11y_label: format!("Curator, {}. Open the feed page", link.title),
            });
        }
        let one_way = self.link.as_ref().is_some_and(|link| !link.two_way);
        let text = self
            .author
            .clone()
            .or_else(|| self.link.as_ref().map(|link| link.title.clone()))?;
        let text = if one_way {
            format!("{text} {UNVERIFIED_LINK_NOTE}")
        } else {
            text
        };
        Some(PlaylistCuratorVm::Author { text })
    }
}

impl CuratorLink {
    fn of(row: &PublisherRelationship, feed: &Feed) -> Self {
        let feed_guid = non_empty(row.publisher_feed_guid.as_deref())
            .unwrap_or_default()
            .to_owned();
        let title = non_empty(row.remote_feed_title.as_deref())
            .or_else(|| non_empty(feed.publisher_feed_title.as_deref()))
            .or_else(|| non_empty(row.publisher_feed_url.as_deref()))
            .unwrap_or(&feed_guid)
            .to_owned();
        let two_way = row.two_way_validated == Some(true);
        // Stophammer ADR 0102 §2: `one_side` and null count as confirmed.
        let confirmed = two_way && row.role_agreement != Some(RoleAgreement::Conflict);
        let agreed_roles = if row.role_agreement == Some(RoleAgreement::Both) {
            known_roles(row.role.as_deref())
        } else {
            Vec::new()
        };
        Self {
            feed_guid,
            title,
            two_way,
            confirmed,
            agreed_roles,
        }
    }
}

/// The author text of a playlist. The Index gives the placeholder "Unknown
/// Artist" for a playlist with no named author, and a placeholder is not a
/// name.
fn author_text(feed: &Feed) -> Option<String> {
    non_empty(feed.release_artist.as_deref())
        .filter(|name| name_sort_key(name) != name_sort_key(UNKNOWN_ARTIST_TEXT))
        .map(str::to_owned)
}

/// The heading of one group of the Public playlists list (R-CURATOR-5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PlaylistGroupHeadingVm {
    /// Stable element id.
    pub(crate) element_id: String,
    /// The group.
    pub(crate) group: PlaylistGroup,
}

/// One group of the Public playlists list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PlaylistGroup {
    /// The playlists with a confirmed link to one feed. The heading opens
    /// the feed page.
    Curator {
        feed_guid: String,
        title: String,
        badge: CuratorBadgeVm,
        a11y_label: String,
    },
    /// Each other playlist. This group is always last.
    Other,
}

impl PlaylistGroupHeadingVm {
    /// The visible title of the heading.
    #[must_use]
    pub(crate) fn title(&self) -> &str {
        match &self.group {
            PlaylistGroup::Curator { title, .. } => title,
            PlaylistGroup::Other => OTHER_PLAYLISTS_HEADING,
        }
    }
}

/// The sequence of the groups (main session choice 1 of task 002).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlaylistGroupOrder {
    /// The groups follow the sequence of their first row.
    RowSequence,
    /// The groups follow the feed titles, A to Z, or Z to A when
    /// `reversed`.
    Title { reversed: bool },
}

/// The rows of `feeds` in their groups. Each item is the index of a feed in
/// `feeds`, or a heading. A feed with a confirmed publisher goes into the
/// group of that feed, and each other feed goes into "Other playlists",
/// which is last. `unverified` of a feed index is true when the feed gives
/// `unconfirmed_publisher`.
#[must_use]
pub(crate) fn playlist_groups(
    feeds: &[Feed],
    curators: &PlaylistCurators,
    order: PlaylistGroupOrder,
) -> Vec<PlaylistGroupItem> {
    struct Group {
        feed_guid: String,
        title: String,
        roles: Vec<KnownRole>,
        rows: Vec<usize>,
    }
    let mut groups: Vec<Group> = Vec::new();
    let mut other: Vec<usize> = Vec::new();
    for (index, feed) in feeds.iter().enumerate() {
        let confirmed = feed
            .confirmed_publisher
            .as_ref()
            .filter(|publisher| !publisher.feed_guid.trim().is_empty());
        let Some(publisher) = confirmed else {
            other.push(index);
            continue;
        };
        let guid = publisher.feed_guid.trim();
        let link_roles = if publisher.role_agreement.as_deref() == Some("both") {
            known_roles(publisher.role.as_deref())
        } else {
            Vec::new()
        };
        if let Some(group) = groups.iter_mut().find(|group| group.feed_guid == guid) {
            group.rows.push(index);
            group.roles.extend(link_roles);
        } else {
            let title = non_empty(publisher.title.as_deref())
                .or_else(|| curators.title(guid))
                .unwrap_or(guid)
                .to_owned();
            let mut roles = curators.release_roles(guid).to_vec();
            roles.extend(link_roles);
            groups.push(Group {
                feed_guid: guid.to_owned(),
                title,
                roles,
                rows: vec![index],
            });
        }
    }
    if let PlaylistGroupOrder::Title { reversed } = order {
        // Musicindex ADR 0053 R-LIST-12 item 1: the names compare in the
        // language of the reader, and a number compares as a number.
        groups.sort_by(|a, b| {
            compare_names(&a.title, &b.title).then_with(|| a.feed_guid.cmp(&b.feed_guid))
        });
        if reversed {
            groups.reverse();
        }
    }
    let mut items = Vec::new();
    for group in groups {
        let badge = CuratorBadgeVm::of(group.roles);
        let count = group.rows.len();
        items.push(PlaylistGroupItem::Heading(PlaylistGroupHeadingVm {
            element_id: format!("music-public-playlist-group-{}", group.feed_guid),
            group: PlaylistGroup::Curator {
                a11y_label: format!(
                    "{}, {}, {count} {}. Open the feed page",
                    group.title,
                    badge.word,
                    if count == 1 { "playlist" } else { "playlists" }
                ),
                feed_guid: group.feed_guid,
                title: group.title,
                badge,
            },
        }));
        items.extend(group.rows.into_iter().map(|index| PlaylistGroupItem::Row {
            index,
            unverified: false,
        }));
    }
    if !other.is_empty() {
        items.push(PlaylistGroupItem::Heading(PlaylistGroupHeadingVm {
            element_id: "music-public-playlist-group-other".to_owned(),
            group: PlaylistGroup::Other,
        }));
        items.extend(other.into_iter().map(|index| PlaylistGroupItem::Row {
            index,
            unverified: feeds[index].unconfirmed_publisher.is_some(),
        }));
    }
    items
}

/// One item of [`playlist_groups`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PlaylistGroupItem {
    /// The heading of a group.
    Heading(PlaylistGroupHeadingVm),
    /// The feed at `index`. `unverified` is true when the row ends with
    /// [`UNVERIFIED_LINK_NOTE`].
    Row { index: usize, unverified: bool },
}

/// The text of `value` without its surrounding white space, or `None` when
/// it is empty.
fn non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|text| !text.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed(value: serde_json::Value) -> Feed {
        serde_json::from_value(value).unwrap()
    }

    fn playlist_with_link(two_way: bool, agreement: &str, role: &str) -> Feed {
        feed(serde_json::json!({
            "feed_guid": "iam",
            "raw_medium": "musicL",
            "title": "IAM",
            "release_artist": "ChadF",
            "publisher": [{
                "direction": "music_to_publisher",
                "publisher_feed_guid": "chadf",
                "remote_feed_title": "ChadF",
                "two_way_validated": two_way,
                "role": role,
                "role_agreement": agreement
            }]
        }))
    }

    fn curators() -> PlaylistCurators {
        PlaylistCurators::from_publisher_rows(&[
            feed(serde_json::json!({"feed_guid": "chadf", "title": "ChadF"})),
            feed(serde_json::json!({
                "feed_guid": "sir-libre",
                "title": "Sir Libre Records",
                "agreed_roles": ["label"]
            })),
        ])
    }

    /// Stophammer ADR 0102 §7: without `playlist_links`, no Curator row.
    #[test]
    fn musicindex_adr_0081_curator_4_no_row_without_the_capability() {
        let facts = PlaylistCuratorFacts::from_feed(&playlist_with_link(true, "both", "curator"));
        assert_eq!(facts.row(None), None);
    }

    /// R-CURATOR-4 item 1: a confirmed link opens the feed page. The badge
    /// gives the role words with the dot.
    #[test]
    fn musicindex_adr_0081_curator_4_confirmed_link_opens_the_feed() {
        let facts = PlaylistCuratorFacts::from_feed(&playlist_with_link(true, "both", "curator"));
        let Some(PlaylistCuratorVm::Feed {
            feed_guid,
            title,
            badge,
            a11y_label,
        }) = facts.row(Some(&curators()))
        else {
            panic!("a confirmed link gives a feed row");
        };
        assert_eq!(feed_guid, "chadf");
        assert_eq!(title, "ChadF");
        assert_eq!(badge.word, "Curator");
        assert!(!badge.hollow);
        assert_eq!(a11y_label, "Curator, ChadF. Open the feed page");

        // Stophammer ADR 0102 §2: `one_side` counts as confirmed, but it
        // gives no agreed role.
        let one_side =
            PlaylistCuratorFacts::from_feed(&playlist_with_link(true, "one_side", "curator"));
        let Some(PlaylistCuratorVm::Feed { badge, .. }) = one_side.row(Some(&curators())) else {
            panic!("one_side is confirmed");
        };
        assert_eq!(badge.word, FEED_WORD);
        assert!(badge.hollow);
    }

    /// R-CURATOR-4 item 2: a one-way link gives the author text and the
    /// note. A conflict link gives the author text only.
    #[test]
    fn musicindex_adr_0081_curator_4_unconfirmed_links_give_the_author_text() {
        let one_way =
            PlaylistCuratorFacts::from_feed(&playlist_with_link(false, "both", "curator"));
        assert_eq!(
            one_way.row(Some(&curators())),
            Some(PlaylistCuratorVm::Author {
                text: "ChadF (unverified link)".to_owned()
            })
        );
        let conflict =
            PlaylistCuratorFacts::from_feed(&playlist_with_link(true, "conflict", "label"));
        assert_eq!(
            conflict.row(Some(&curators())),
            Some(PlaylistCuratorVm::Author {
                text: "ChadF".to_owned()
            })
        );
        let no_link = PlaylistCuratorFacts::from_feed(&feed(serde_json::json!({
            "feed_guid": "x", "release_artist": "Unknown Artist"
        })));
        assert_eq!(no_link.row(Some(&curators())), None);
    }

    /// The release roles of the route join the curator role of the link.
    #[test]
    fn musicindex_adr_0081_curator_4_label_curator_badge() {
        let mut playlist = playlist_with_link(true, "both", "curator");
        playlist.publisher.as_mut().unwrap()[0].publisher_feed_guid = Some("sir-libre".to_owned());
        let Some(PlaylistCuratorVm::Feed { badge, .. }) =
            PlaylistCuratorFacts::from_feed(&playlist).row(Some(&curators()))
        else {
            panic!("a confirmed link gives a feed row");
        };
        assert_eq!(badge.word, "Label · Curator");
        assert_eq!(badge.role, HeaderRole::Label);
    }

    fn listed(guid: &str, publisher: Option<(&str, &str)>, unconfirmed: bool) -> Feed {
        let mut value = serde_json::json!({"feed_guid": guid, "raw_medium": "musicL"});
        if let Some((publisher_guid, title)) = publisher {
            value["confirmed_publisher"] = serde_json::json!({
                "feed_guid": publisher_guid, "title": title,
                "role": "curator", "role_agreement": "both"
            });
        }
        if unconfirmed {
            value["unconfirmed_publisher"] = serde_json::json!({"feed_guid": "z"});
        }
        feed(value)
    }

    fn heading_titles(items: &[PlaylistGroupItem]) -> Vec<&str> {
        items
            .iter()
            .filter_map(|item| match item {
                PlaylistGroupItem::Heading(heading) => Some(heading.title()),
                PlaylistGroupItem::Row { .. } => None,
            })
            .collect()
    }

    /// R-CURATOR-5 and main session choice 1 of task 002: the groups follow
    /// the row sequence or the titles. "Other playlists" is always last, and
    /// only a row with `unconfirmed_publisher` has the note.
    #[test]
    fn musicindex_adr_0081_curator_5_group_order_and_notes() {
        let feeds = [
            listed("a", Some(("sir-libre", "Sir Libre Records")), false),
            listed("b", None, true),
            listed("c", Some(("chadf", "ChadF")), false),
            listed("d", None, false),
            listed("e", Some(("sir-libre", "Sir Libre Records")), false),
        ];
        let items = playlist_groups(&feeds, &curators(), PlaylistGroupOrder::RowSequence);
        assert_eq!(
            heading_titles(&items),
            ["Sir Libre Records", "ChadF", OTHER_PLAYLISTS_HEADING]
        );
        let rows: Vec<(usize, bool)> = items
            .iter()
            .filter_map(|item| match item {
                PlaylistGroupItem::Row { index, unverified } => Some((*index, *unverified)),
                PlaylistGroupItem::Heading(_) => None,
            })
            .collect();
        assert_eq!(
            rows,
            [(0, false), (4, false), (2, false), (1, true), (3, false)]
        );
        let PlaylistGroupItem::Heading(PlaylistGroupHeadingVm {
            group: PlaylistGroup::Curator {
                badge, a11y_label, ..
            },
            ..
        }) = &items[0]
        else {
            panic!("the first item is a curator heading");
        };
        assert_eq!(badge.word, "Label · Curator");
        assert_eq!(
            a11y_label,
            "Sir Libre Records, Label · Curator, 2 playlists. Open the feed page"
        );

        let a_to_z = playlist_groups(
            &feeds,
            &curators(),
            PlaylistGroupOrder::Title { reversed: false },
        );
        assert_eq!(
            heading_titles(&a_to_z),
            ["ChadF", "Sir Libre Records", OTHER_PLAYLISTS_HEADING]
        );
        let z_to_a = playlist_groups(
            &feeds,
            &curators(),
            PlaylistGroupOrder::Title { reversed: true },
        );
        assert_eq!(
            heading_titles(&z_to_a),
            ["Sir Libre Records", "ChadF", OTHER_PLAYLISTS_HEADING]
        );
    }

    /// Musicindex ADR 0053 R-LIST-12 item 1: with A to Z, a number in a
    /// feed title compares as a number.
    #[test]
    fn musicindex_adr_0081_curator_5_a_to_z_compares_numbers() {
        let feeds = [
            listed("a", Some(("g10", "Curator 10")), false),
            listed("b", Some(("g2", "Curator 2")), false),
        ];
        let items = playlist_groups(
            &feeds,
            &curators(),
            PlaylistGroupOrder::Title { reversed: false },
        );
        assert_eq!(heading_titles(&items), ["Curator 2", "Curator 10"]);
    }

    /// With no unconfirmed playlist, the list has no "Other playlists".
    #[test]
    fn musicindex_adr_0081_curator_5_no_other_group_when_empty() {
        let feeds = [listed("c", Some(("chadf", "ChadF")), false)];
        let items = playlist_groups(&feeds, &curators(), PlaylistGroupOrder::RowSequence);
        assert_eq!(heading_titles(&items), ["ChadF"]);
    }
}
