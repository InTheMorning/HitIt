//! The actions and the name links of an album page (ADR 0101 Decision 5,
//! task 005).
//!
//! The Library album page and the Index album page use this one model
//! (ADR 0037). The screen maps each action to its command.

#![warn(clippy::pedantic)]

use crate::view_models::entity_detail::ReleaseMembershipState;
use crate::view_models::release_kind::ReleaseKind;
use crate::view_models::singles_group::SinglesGroupKey;

/// The accessibility label of the "⋯" menu of an album page.
pub const ALBUM_PAGE_MENU_A11Y_LABEL: &str = "More actions for this album";

/// One action of an album page.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum AlbumPageAction {
    /// Download the album feed.
    DownloadAlbum,
    /// Add the Library tracks of the album to a playlist.
    AddToPlaylist,
    /// Copy the feed URL of the album.
    CopyFeedUrl,
    /// Look up the missing `MusicBrainz` fields of the album.
    MusicBrainzLookup,
    /// Remove the album from the Library, after a confirmation.
    RemoveAlbum,
}

/// The result that a copy action shows on the "⋯" button of a page header
/// (musicindex ADR 0028 R-PAGE-2 item 4).
pub(crate) const COPIED_LABEL: &str = "Copied";

impl AlbumPageAction {
    /// The result that the "⋯" button shows after the action. Only a copy
    /// action has one.
    #[must_use]
    pub(crate) const fn result_label(self) -> Option<&'static str> {
        match self {
            Self::CopyFeedUrl => Some(COPIED_LABEL),
            Self::DownloadAlbum
            | Self::AddToPlaylist
            | Self::MusicBrainzLookup
            | Self::RemoveAlbum => None,
        }
    }

    /// The visible label. A label that opens a confirmation ends with "…".
    /// `kind` names the release (ADR 0084 Decision 5) for "Download …" and
    /// "Remove …", for example "Download Single". The kind is in title
    /// case (musicindex ADR 0023 R-CASE-1).
    #[must_use]
    pub(crate) fn label(self, kind: ReleaseKind) -> String {
        match self {
            Self::DownloadAlbum => format!("Download {}", kind.label()),
            Self::AddToPlaylist => "Add to Playlist".to_string(),
            Self::CopyFeedUrl => "Copy Feed URL".to_string(),
            Self::MusicBrainzLookup => "MusicBrainz Lookup".to_string(),
            Self::RemoveAlbum => format!("Remove {}…", kind.label()),
        }
    }

    /// The accessibility label. `kind` names the release (ADR 0084
    /// Decision 5), the same way as [`Self::label`].
    #[must_use]
    pub(crate) fn a11y_label(self, kind: ReleaseKind) -> String {
        match self {
            Self::DownloadAlbum => format!("Download this {}", kind.noun()),
            Self::AddToPlaylist => "Add this album to a playlist".to_string(),
            Self::CopyFeedUrl => "Copy the feed URL of this album".to_string(),
            Self::MusicBrainzLookup => {
                "Look up missing MusicBrainz fields for this album".to_string()
            }
            Self::RemoveAlbum => format!("Remove this {} from the Library", kind.noun()),
        }
    }

    /// A destructive action is last in its menu and asks for confirmation.
    #[must_use]
    pub const fn is_destructive(self) -> bool {
        matches!(self, Self::RemoveAlbum)
    }

    /// The command changes stored data. The button role follows this fact
    /// (musicindex ADR 0042 R-ACT-4 item 3, `ControlStyle::by_result`).
    #[must_use]
    pub const fn changes_stored_data(self) -> bool {
        matches!(
            self,
            Self::DownloadAlbum | Self::AddToPlaylist | Self::RemoveAlbum
        )
    }
}

/// A display-ready album page action with its typed availability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AlbumPageActionDisplay {
    /// The action.
    pub action: AlbumPageAction,
    /// The action can run now.
    pub available: bool,
}

impl AlbumPageActionDisplay {
    /// Make a display for `action`.
    #[must_use]
    pub const fn new(action: AlbumPageAction, available: bool) -> Self {
        Self { action, available }
    }
}

/// The actions of an album page in the order of ADR 0101 Decision 5: the
/// filled button when the page has a main action, the plain buttons, and the
/// "⋯" menu.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AlbumPageActions {
    /// The filled button. It is `None` until a release page has a main
    /// action of its own (musicindex ADR 0045 R-ACT-7 item 4).
    pub filled: Option<AlbumPageActionDisplay>,
    /// The plain buttons. Download is first.
    pub plain: Vec<AlbumPageActionDisplay>,
    /// The "⋯" menu items. A destructive item is last.
    pub menu: Vec<AlbumPageActionDisplay>,
}

/// The origin and the state of an album page.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AlbumPageOrigin {
    /// An album that the Index page shows.
    Index,
    /// A Library album.
    Library {
        /// The membership of the album in the Library.
        membership: ReleaseMembershipState,
        /// A `MusicBrainz` lookup can start now.
        musicbrainz_available: bool,
    },
}

/// The actions of an album page. "Copy feed URL" shows when the album has a
/// feed URL.
#[must_use]
pub fn album_page_actions(origin: AlbumPageOrigin, feed_url: Option<&str>) -> AlbumPageActions {
    let mut menu: Vec<_> = feed_url
        .map(str::trim)
        .filter(|url| !url.is_empty())
        .map(|_| AlbumPageActionDisplay::new(AlbumPageAction::CopyFeedUrl, true))
        .into_iter()
        .collect();
    let AlbumPageOrigin::Library {
        membership,
        musicbrainz_available,
    } = origin
    else {
        return AlbumPageActions {
            filled: None,
            plain: vec![AlbumPageActionDisplay::new(
                AlbumPageAction::DownloadAlbum,
                true,
            )],
            menu,
        };
    };
    menu.push(AlbumPageActionDisplay::new(
        AlbumPageAction::MusicBrainzLookup,
        musicbrainz_available,
    ));
    let button = match membership {
        ReleaseMembershipState::RemoteOnly => {
            AlbumPageActionDisplay::new(AlbumPageAction::DownloadAlbum, true)
        }
        ReleaseMembershipState::Queued | ReleaseMembershipState::Downloading => {
            AlbumPageActionDisplay::new(AlbumPageAction::DownloadAlbum, false)
        }
        ReleaseMembershipState::InLibrary => {
            AlbumPageActionDisplay::new(AlbumPageAction::AddToPlaylist, true)
        }
        ReleaseMembershipState::Removing => {
            AlbumPageActionDisplay::new(AlbumPageAction::AddToPlaylist, false)
        }
    };
    match membership {
        ReleaseMembershipState::InLibrary => menu.push(AlbumPageActionDisplay::new(
            AlbumPageAction::RemoveAlbum,
            true,
        )),
        ReleaseMembershipState::Removing => menu.push(AlbumPageActionDisplay::new(
            AlbumPageAction::RemoveAlbum,
            false,
        )),
        ReleaseMembershipState::RemoteOnly
        | ReleaseMembershipState::Queued
        | ReleaseMembershipState::Downloading => {}
    }
    AlbumPageActions {
        filled: None,
        plain: vec![button],
        menu,
    }
}

/// The page that a name link under the album title opens.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AlbumNameLinkTarget {
    /// The artist page of a name. The Library opens its artist page. The
    /// Index opens the tracks that match the name.
    Artist(String),
    /// The page of a named feed, by its feed GUID (ADR 0077 Decision 1,
    /// ADR 0085 Decision 6).
    Publisher(String),
    /// The page of the Library singles group of a single (ADR 0084
    /// Decision 6).
    SinglesGroup(SinglesGroupKey),
}

/// A name under the album title that links to a page.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AlbumNameLinkVm {
    /// The visible name.
    pub label: String,
    /// The accessibility label.
    pub a11y_label: String,
    /// The note under the link, for example "Unverified artist" (ADR 0085
    /// Decision 6). `None` when the link has no note.
    pub note: Option<String>,
    /// The page that the link opens.
    pub target: AlbumNameLinkTarget,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn actions_of(display: &[AlbumPageActionDisplay]) -> Vec<AlbumPageAction> {
        display.iter().map(|item| item.action).collect()
    }

    fn library(membership: ReleaseMembershipState) -> AlbumPageOrigin {
        AlbumPageOrigin::Library {
            membership,
            musicbrainz_available: true,
        }
    }

    const FEED_URL: Option<&str> = Some("https://feeds.example.test/album.xml");

    /// M5 (ADR 0101 task 008): a copy action gives "Copied" for its button,
    /// and no other action of a page header gives a result.
    #[test]
    fn adr_0101_task_008_copy_action_gives_copied_for_its_button() {
        use crate::view_models::track_detail::TrackPageAction;
        assert_eq!(AlbumPageAction::CopyFeedUrl.result_label(), Some("Copied"));
        assert_eq!(TrackPageAction::CopyFeedUrl.result_label(), Some("Copied"));
        for action in [
            AlbumPageAction::DownloadAlbum,
            AlbumPageAction::AddToPlaylist,
            AlbumPageAction::MusicBrainzLookup,
            AlbumPageAction::RemoveAlbum,
        ] {
            assert_eq!(action.result_label(), None, "{action:?}");
        }
        for action in [
            TrackPageAction::DownloadAlbum,
            TrackPageAction::DownloadTrack,
            TrackPageAction::AddToPlaylist,
            TrackPageAction::MusicBrainzLookup,
            TrackPageAction::InspectSources,
            TrackPageAction::RemoveTrack,
        ] {
            assert_eq!(action.result_label(), None, "{action:?}");
        }
    }

    /// ADR 0101 Decision 5 (M2): no album state gives a filled action. The
    /// plain buttons and the "⋯" items follow the state table of task 005.
    #[test]
    fn adr_0101_decision_5_album_page_actions_give_no_filled_action() {
        let index = album_page_actions(AlbumPageOrigin::Index, FEED_URL);
        assert_eq!(index.filled, None);
        assert_eq!(
            index.plain,
            vec![AlbumPageActionDisplay::new(
                AlbumPageAction::DownloadAlbum,
                true
            )]
        );
        assert_eq!(actions_of(&index.menu), vec![AlbumPageAction::CopyFeedUrl]);

        let remote = album_page_actions(library(ReleaseMembershipState::RemoteOnly), FEED_URL);
        assert_eq!(remote.filled, None);
        assert_eq!(
            remote.plain,
            vec![AlbumPageActionDisplay::new(
                AlbumPageAction::DownloadAlbum,
                true
            )]
        );
        assert_eq!(
            actions_of(&remote.menu),
            vec![
                AlbumPageAction::CopyFeedUrl,
                AlbumPageAction::MusicBrainzLookup
            ]
        );

        let in_library = album_page_actions(library(ReleaseMembershipState::InLibrary), FEED_URL);
        assert_eq!(in_library.filled, None);
        assert_eq!(
            in_library.plain,
            vec![AlbumPageActionDisplay::new(
                AlbumPageAction::AddToPlaylist,
                true
            )]
        );
        assert_eq!(
            actions_of(&in_library.menu),
            vec![
                AlbumPageAction::CopyFeedUrl,
                AlbumPageAction::MusicBrainzLookup,
                AlbumPageAction::RemoveAlbum
            ]
        );
    }

    /// ADR 0101 Decision 5 (M3, M4): the download and the add to a playlist
    /// change stored data, so they have the role Secondary. A header shows at
    /// most two buttons.
    #[test]
    fn adr_0101_decision_5_album_page_buttons_are_secondary_and_at_most_two() {
        assert!(AlbumPageAction::DownloadAlbum.changes_stored_data());
        assert!(AlbumPageAction::AddToPlaylist.changes_stored_data());
        assert!(!AlbumPageAction::CopyFeedUrl.changes_stored_data());
        for membership in [
            ReleaseMembershipState::RemoteOnly,
            ReleaseMembershipState::Queued,
            ReleaseMembershipState::Downloading,
            ReleaseMembershipState::InLibrary,
            ReleaseMembershipState::Removing,
        ] {
            let actions = album_page_actions(library(membership), FEED_URL);
            assert!(actions.plain.len() <= 2, "{membership:?}");
            assert!(actions.plain.iter().all(|button| {
                button.action.changes_stored_data() && !button.action.is_destructive()
            }));
        }
        let index = album_page_actions(AlbumPageOrigin::Index, FEED_URL);
        assert!(index.plain.len() <= 2);
    }

    /// R83-51: a busy download or removal makes the button and
    /// "Remove album…" unavailable. The lookup keeps its own availability.
    #[test]
    fn adr_0083_album_page_busy_states_disable_the_button_and_removal() {
        let downloading =
            album_page_actions(library(ReleaseMembershipState::Downloading), FEED_URL);
        assert_eq!(downloading.plain[0].action, AlbumPageAction::DownloadAlbum);
        assert!(!downloading.plain[0].available);

        let removing = album_page_actions(library(ReleaseMembershipState::Removing), FEED_URL);
        assert_eq!(removing.plain[0].action, AlbumPageAction::AddToPlaylist);
        assert!(!removing.plain[0].available);
        let remove = removing.menu.last().expect("removal stays in the menu");
        assert_eq!(remove.action, AlbumPageAction::RemoveAlbum);
        assert!(!remove.available);

        let lookup_busy = album_page_actions(
            AlbumPageOrigin::Library {
                membership: ReleaseMembershipState::InLibrary,
                musicbrainz_available: false,
            },
            FEED_URL,
        );
        assert!(lookup_busy
            .menu
            .iter()
            .any(|item| item.action == AlbumPageAction::MusicBrainzLookup && !item.available));
    }

    /// R83-51: an album with no feed URL has no "Copy feed URL" item.
    #[test]
    fn adr_0083_album_page_without_feed_url_has_no_copy_item() {
        for feed_url in [None, Some(""), Some("  ")] {
            let actions = album_page_actions(AlbumPageOrigin::Index, feed_url);
            assert!(actions.menu.is_empty(), "{feed_url:?}");
        }
    }

    /// R83-52: "Remove album…" is last and is the only destructive item.
    #[test]
    fn adr_0083_album_page_removal_is_last_and_destructive() {
        let actions = album_page_actions(library(ReleaseMembershipState::InLibrary), FEED_URL);
        let last = actions.menu.last().expect("the menu has items");
        assert_eq!(last.action, AlbumPageAction::RemoveAlbum);
        assert!(last.action.is_destructive());
        assert!(last.action.label(ReleaseKind::Album).ends_with('…'));
        assert_eq!(
            actions
                .menu
                .iter()
                .filter(|item| item.action.is_destructive())
                .count(),
            1
        );
        assert_eq!(actions.filled, None);
    }

    /// R7-05 (ADR 0084 Decision 5): the download and the removal label
    /// name the release kind, and the accessibility label names the same
    /// kind. ADR 0101 Decision 5 (M5): each label is in title case.
    #[test]
    fn adr_0084_release_kind_r7_05_download_and_remove_name_the_kind() {
        assert_eq!(
            AlbumPageAction::DownloadAlbum.label(ReleaseKind::Single),
            "Download Single"
        );
        assert_eq!(
            AlbumPageAction::DownloadAlbum.label(ReleaseKind::Ep),
            "Download EP"
        );
        assert_eq!(
            AlbumPageAction::DownloadAlbum.label(ReleaseKind::Album),
            "Download Album"
        );
        assert_eq!(
            AlbumPageAction::DownloadAlbum.a11y_label(ReleaseKind::Single),
            "Download this single"
        );
        assert_eq!(
            AlbumPageAction::RemoveAlbum.label(ReleaseKind::Single),
            "Remove Single…"
        );
        assert_eq!(
            AlbumPageAction::RemoveAlbum.a11y_label(ReleaseKind::Single),
            "Remove this single from the Library"
        );
        assert_eq!(
            AlbumPageAction::AddToPlaylist.label(ReleaseKind::Album),
            "Add to Playlist"
        );
        assert_eq!(
            AlbumPageAction::CopyFeedUrl.label(ReleaseKind::Album),
            "Copy Feed URL"
        );
        assert_eq!(
            AlbumPageAction::MusicBrainzLookup.label(ReleaseKind::Album),
            "MusicBrainz Lookup"
        );
    }
}
