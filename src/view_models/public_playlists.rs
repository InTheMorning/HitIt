//! The Public playlists of the Index (ADR 0089 Decision 1, Decision 5 and
//! packet 009; musicindex ADR 0040 for the mosaic of a playlist with no
//! image).
//!
//! This module gives the rows of the Public playlists list, the item page
//! of one playlist, and the load state of that page. A playlist is a musicL
//! feed. Its entries are the `remote_items` of the feed, in `position`
//! sequence. The module has no renderer type, and it reads no data beyond
//! the response it is given. `MusicIndex` responses are untrusted input.

#![warn(clippy::pedantic)]

use crate::api::{Feed, RemoteItem};
use crate::view_models::collection_art::{CollectionArtVm, CollectionKind};
use crate::view_models::index_name_list::{IndexListStatusRowVm, IndexNameList, IndexNameListPage};
use crate::view_models::library::{DestinationListRowTarget, DestinationListRowVm};
use crate::view_models::list_selection::RowSelectionVm;
use crate::view_models::release_page::{public_playlist_copy_menu_vm, TrackRowMenuVm};
use crate::view_models::workspace::{FrameNavigationEntry, ReleaseKey, TrackKey};

/// The title of the copy dialog.
pub(crate) const COPY_DIALOG_TITLE: &str = "Copy to My Playlists";
/// The label of the confirm button of the copy dialog.
pub(crate) const COPY_CONFIRM_LABEL: &str = "Copy";
/// The label of the cancel button of the copy dialog.
pub(crate) const COPY_CANCEL_LABEL: &str = "Cancel";
/// The placeholder of the name field of the copy dialog.
pub(crate) const COPY_NAME_PLACEHOLDER: &str = "Name of the new playlist";
/// The text of a row, and of a page, while its copy runs.
pub(crate) const COPYING_TEXT: &str = "Copying to My Playlists\u{2026}";
/// The message of the copy dialog when the name is blank.
const COPY_NAME_EMPTY_TEXT: &str = "Enter a name for the playlist.";

/// The label of a playlist with no title. The Index gives no title for
/// it, so the row names the kind instead.
const UNTITLED_PLAYLIST: &str = "Untitled playlist";
/// The label of an entry with no track title and no feed title.
const UNTITLED_ITEM: &str = "Untitled item";

/// One item of the Public playlists list (ADR 0089 packet 009). A row
/// opens its playlist page. A status row is the loading row, the error row
/// or "Load more".
#[derive(Clone, Debug, PartialEq, Eq)]
#[expect(
    clippy::large_enum_variant,
    reason = "the list holds at most one status item; a box for each row adds one allocation for each row"
)]
pub(crate) enum PublicPlaylistListItemVm {
    /// A playlist row.
    Row(PublicPlaylistRowVm),
    /// The loading row, the error row or "Load more".
    Status(IndexListStatusRowVm),
}

/// A playlist row with its "More actions" menu. `menu` is `None` while a
/// copy runs, because the menu cannot act then (ADR 0089 Decision 7).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PublicPlaylistRowVm {
    pub(crate) row: DestinationListRowVm,
    pub(crate) menu: Option<TrackRowMenuVm>,
}

impl PublicPlaylistListItemVm {
    /// The items of `rows`, with `status` before or after them (ADR 0084
    /// packet 016 decision D8).
    #[must_use]
    pub(crate) fn of(
        rows: Vec<PublicPlaylistRowVm>,
        status: Option<IndexListStatusRowVm>,
    ) -> Vec<Self> {
        let (leading, trailing) = match status {
            Some(row) if row.at_list_start() => (Some(row), None),
            other => (None, other),
        };
        leading
            .map(Self::Status)
            .into_iter()
            .chain(rows.into_iter().map(Self::Row))
            .chain(trailing.map(Self::Status))
            .collect()
    }

    /// The element id of the item.
    #[must_use]
    pub(crate) fn element_id(&self) -> &str {
        match self {
            Self::Row(row) => &row.row.element_id,
            Self::Status(row) => row.element_id(),
        }
    }
}

/// The rows of the Public playlists list, one for each feed that has a
/// GUID. A feed with no GUID cannot open its page, so it has no row.
#[must_use]
pub(crate) fn public_playlist_rows(feeds: &[Feed]) -> Vec<DestinationListRowVm> {
    feeds.iter().filter_map(public_playlist_row).collect()
}

fn public_playlist_row(feed: &Feed) -> Option<DestinationListRowVm> {
    let feed_guid = non_empty(feed.feed_guid.as_deref())?;
    let label = non_empty(feed.title.as_deref()).unwrap_or(UNTITLED_PLAYLIST);
    let secondary_text = non_empty(feed.release_artist.as_deref())
        .unwrap_or_default()
        .to_owned();
    let art = CollectionArtVm::resolve(
        non_empty(feed.image_url.as_deref()),
        Vec::<&str>::new(),
        CollectionKind::Playlist,
    );
    Some(DestinationListRowVm {
        element_id: format!("music-public-playlist-{feed_guid}"),
        label: label.to_owned(),
        secondary_text,
        a11y_label: format!("Public playlist, {label}"),
        thumbnail_href: None,
        art: Some(art),
        open: DestinationListRowTarget::PublicPlaylist(feed_guid.to_owned()),
        selection: RowSelectionVm::none(),
        release_kind: None,
        menu: None,
        download_state: None,
        marked: false,
    })
}

/// The items of the Public playlists list: the loaded rows, with the
/// status row of the page (ADR 0084 packet 016 decision D8). The mark
/// follows `marked`.
///
/// `copying` is the playlist whose copy runs. Its row shows the busy text,
/// and no row has a menu while a copy runs.
#[must_use]
pub(crate) fn public_playlist_list_items(
    page: &IndexNameListPage<Feed>,
    marked: Option<&FrameNavigationEntry>,
    copying: Option<&str>,
) -> Vec<PublicPlaylistListItemVm> {
    let rows = public_playlist_rows(page.rows())
        .into_iter()
        .map(|row| {
            let row = row.with_marked(marked);
            let feed_guid = match &row.open {
                DestinationListRowTarget::PublicPlaylist(guid) => guid.clone(),
                _ => String::new(),
            };
            let busy = copying == Some(feed_guid.as_str());
            let menu = public_playlist_copy_menu(&feed_guid, &row.label, copying.is_some());
            let row = if busy {
                DestinationListRowVm {
                    secondary_text: COPYING_TEXT.to_owned(),
                    ..row
                }
            } else {
                row
            };
            PublicPlaylistRowVm { row, menu }
        })
        .collect();
    PublicPlaylistListItemVm::of(rows, page.status_row(IndexNameList::PublicPlaylists))
}

/// The "More actions" menu of a public playlist, with its one item "Copy
/// to My Playlists…" (ADR 0089 packet 010). `None` while a copy runs.
#[must_use]
pub(crate) fn public_playlist_copy_menu(
    feed_guid: &str,
    title: &str,
    copying: bool,
) -> Option<TrackRowMenuVm> {
    if copying || feed_guid.is_empty() {
        return None;
    }
    let id = format!("music-public-playlist-{feed_guid}-menu");
    Some(public_playlist_copy_menu_vm(id, title))
}

/// The texts of the copy dialog of the public playlist `title` (ADR 0089
/// packet 010).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PublicPlaylistCopyDialogVm {
    /// The explanation under the title. It names the playlist.
    pub(crate) body: String,
    /// The message of a rejected name, or `None`.
    pub(crate) message: Option<String>,
}

impl PublicPlaylistCopyDialogVm {
    /// The dialog of the playlist `title`, with the message of the last
    /// rejected name.
    #[must_use]
    pub(crate) fn new(title: &str, message: Option<String>) -> Self {
        Self {
            body: format!("Copy {title} to a new local playlist. The copy downloads no file."),
            message,
        }
    }
}

/// The name in the copy dialog, checked against the playlists of the
/// Library (ADR 0089 packet 010). `playlists.name` is `UNIQUE`, so a name
/// that exists is a conflict, and the dialog stays open.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PublicPlaylistCopyName {
    /// The trimmed name. It is free.
    Ready(String),
    /// The name is blank.
    Empty,
    /// A playlist holds the trimmed name.
    Taken(String),
}

impl PublicPlaylistCopyName {
    /// Checks `input` against the names of the existing playlists.
    #[must_use]
    pub(crate) fn check(input: &str, existing: &[&str]) -> Self {
        let name = input.trim();
        if name.is_empty() {
            Self::Empty
        } else if existing.contains(&name) {
            Self::Taken(name.to_owned())
        } else {
            Self::Ready(name.to_owned())
        }
    }

    /// The message that the dialog shows for a rejected name.
    #[must_use]
    pub(crate) fn message(&self) -> Option<String> {
        match self {
            Self::Ready(_) => None,
            Self::Empty => Some(COPY_NAME_EMPTY_TEXT.to_owned()),
            Self::Taken(name) => Some(format!(
                "A playlist named \"{name}\" already exists. Enter another name."
            )),
        }
    }
}

/// The report of a finished copy (ADR 0089 packet 010). It names each
/// count: the tracks copied, the tracks offered, and the items skipped.
#[must_use]
pub(crate) fn public_playlist_copy_report(name: &str, copied: usize, total: usize) -> String {
    let noun = if total == 1 { "track" } else { "tracks" };
    format!(
        "Copied {copied} of {total} {noun} to {name}. {} could not be read.",
        total.saturating_sub(copied)
    )
}

/// One entry of a public playlist page (ADR 0089 packet 009).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PublicPlaylistItemVm {
    /// Stable element id, from the sequence number of the entry.
    pub(crate) element_id: String,
    /// The sequence number of the entry, from 1.
    pub(crate) number: usize,
    /// The visible label: the track title, else the feed title.
    pub(crate) label: String,
    /// The secondary text: the feed title when the label is a track
    /// title, and the `rel` word that the response gives.
    pub(crate) secondary_text: String,
    /// The art of the entry: the image of the named feed, else the entity
    /// color of a release. An entry names a release, so it uses that color.
    pub(crate) art: CollectionArtVm,
    /// What the entry opens. `None` when the entry names no feed and no
    /// track, so its row cannot open a page.
    pub(crate) target: Option<PublicPlaylistItemTarget>,
    /// The accessibility label of the row.
    pub(crate) a11y_label: String,
}

/// The page that a public playlist entry opens (ADR 0089 packet 009). A
/// track with an item GUID opens the track page by its key. An entry with
/// only a feed GUID opens the release page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PublicPlaylistItemTarget {
    /// The track page of the track key.
    Track(TrackKey),
    /// The release page of the feed.
    Release(ReleaseKey),
}

impl PublicPlaylistItemTarget {
    /// The navigation entry that this target opens.
    #[must_use]
    pub(crate) fn entry(&self) -> FrameNavigationEntry {
        match self {
            Self::Track(key) => FrameNavigationEntry::Track(key.clone()),
            Self::Release(key) => FrameNavigationEntry::Release(key.clone()),
        }
    }
}

/// The item page of one public playlist (ADR 0089 packet 009).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PublicPlaylistPageVm {
    /// The title of the playlist, or a name for a playlist with no title.
    pub(crate) title: String,
    /// The art of the playlist: its own image, else the mosaic of the
    /// images of its entries (musicindex ADR 0040), else the entity color.
    pub(crate) art: CollectionArtVm,
    /// The entries, in `position` sequence.
    pub(crate) items: Vec<PublicPlaylistItemVm>,
}

impl PublicPlaylistPageVm {
    /// Builds the page from one feed read with `include=remote_items`. The
    /// entries sort by `position`, and an entry with no position comes last.
    #[must_use]
    pub(crate) fn from_feed(feed: &Feed) -> Self {
        let mut remote: Vec<&RemoteItem> = feed.remote_items.iter().flatten().collect();
        remote.sort_by_key(|item| item.position.unwrap_or(i64::MAX));
        let items: Vec<PublicPlaylistItemVm> = remote
            .iter()
            .enumerate()
            .map(|(index, item)| PublicPlaylistItemVm::from_remote_item(index + 1, item))
            .collect();
        let covers: Vec<&str> = remote
            .iter()
            .filter_map(|item| non_empty(item.remote_feed_image_url.as_deref()))
            .collect();
        let art = CollectionArtVm::resolve(
            non_empty(feed.image_url.as_deref()),
            covers,
            CollectionKind::Playlist,
        );
        Self {
            title: non_empty(feed.title.as_deref())
                .unwrap_or(UNTITLED_PLAYLIST)
                .to_owned(),
            art,
            items,
        }
    }
}

impl PublicPlaylistItemVm {
    fn from_remote_item(number: usize, item: &RemoteItem) -> Self {
        let feed_guid = non_empty(item.remote_feed_guid.as_deref());
        let track_guid = non_empty(item.remote_item_guid.as_deref());
        let target = match (feed_guid, track_guid) {
            (Some(feed), Some(track)) => Some(PublicPlaylistItemTarget::Track(
                TrackKey::from_index_track_target(Some(feed), track),
            )),
            (None, Some(track)) => Some(PublicPlaylistItemTarget::Track(
                TrackKey::from_index_track_target(None, track),
            )),
            (Some(feed), None) => Some(PublicPlaylistItemTarget::Release(
                ReleaseKey::from_index_feed_guid(feed),
            )),
            (None, None) => None,
        };
        let item_title = non_empty(item.remote_item_title.as_deref());
        let feed_title = non_empty(item.remote_feed_title.as_deref());
        let label = item_title.or(feed_title).unwrap_or(UNTITLED_ITEM);
        let mut parts: Vec<&str> = Vec::new();
        if item_title.is_some() {
            parts.extend(feed_title);
        }
        parts.extend(non_empty(item.rel.as_deref()));
        let secondary_text = parts.join(", ");
        let a11y_label = if secondary_text.is_empty() {
            format!("{number}. {label}")
        } else {
            format!("{number}. {label}, {secondary_text}")
        };
        Self {
            element_id: format!("music-public-playlist-item-{number}"),
            number,
            label: label.to_owned(),
            secondary_text,
            art: CollectionArtVm::resolve(
                non_empty(item.remote_feed_image_url.as_deref()),
                Vec::<&str>::new(),
                CollectionKind::SinglesGroup,
            ),
            target,
            a11y_label,
        }
    }
}

/// The load state of one public playlist page (ADR 0089 packet 009).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PublicPlaylistLoadState {
    /// The read of the playlist is in flight.
    Loading,
    /// The playlist loaded. Boxed, so the variants stay close in size.
    Loaded(Box<PublicPlaylistPageVm>),
    /// The read failed. `detail` names the cause.
    Failed { detail: String },
}

/// The item page of the open public playlist: the feed GUID, its load
/// state, and the generation of the read in flight. A result of an older
/// generation changes nothing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct PublicPlaylistPages {
    feed_guid: Option<String>,
    state: Option<PublicPlaylistLoadState>,
    generation: u64,
}

impl PublicPlaylistPages {
    /// Starts the read of the playlist `feed_guid`. It gives the generation
    /// of the new read, or `None` when the page already holds the playlist
    /// or its read runs. A failed read starts again.
    #[must_use]
    pub(crate) fn begin(&mut self, feed_guid: &str) -> Option<u64> {
        let same = self.feed_guid.as_deref() == Some(feed_guid);
        if same
            && matches!(
                self.state,
                Some(PublicPlaylistLoadState::Loading | PublicPlaylistLoadState::Loaded(_))
            )
        {
            return None;
        }
        self.generation = self.generation.wrapping_add(1);
        self.feed_guid = Some(feed_guid.to_owned());
        self.state = Some(PublicPlaylistLoadState::Loading);
        Some(self.generation)
    }

    /// Stores the feed of the read `generation`. It gives `false` and
    /// changes nothing when `generation` is old or `feed_guid` is not the
    /// playlist of the page.
    pub(crate) fn finish(&mut self, feed_guid: &str, feed: &Feed, generation: u64) -> bool {
        if !self.accepts(feed_guid, generation) {
            return false;
        }
        self.state = Some(PublicPlaylistLoadState::Loaded(Box::new(
            PublicPlaylistPageVm::from_feed(feed),
        )));
        true
    }

    /// Records the failed read `generation`. It gives `false` and changes
    /// nothing when `generation` is old or `feed_guid` is not the playlist
    /// of the page.
    pub(crate) fn fail(
        &mut self,
        feed_guid: &str,
        detail: impl Into<String>,
        generation: u64,
    ) -> bool {
        if !self.accepts(feed_guid, generation) {
            return false;
        }
        self.state = Some(PublicPlaylistLoadState::Failed {
            detail: detail.into(),
        });
        true
    }

    /// The GUID of the playlist that the page holds, or `None` when none.
    #[must_use]
    pub(crate) fn feed_guid(&self) -> Option<&str> {
        self.feed_guid.as_deref()
    }

    /// The load state of the playlist `feed_guid`, or `None` when the page
    /// holds another playlist or none.
    #[must_use]
    pub(crate) fn state_of(&self, feed_guid: &str) -> Option<&PublicPlaylistLoadState> {
        if self.feed_guid.as_deref() == Some(feed_guid) {
            self.state.as_ref()
        } else {
            None
        }
    }

    /// The title of the loaded playlist `feed_guid`, when it has loaded.
    #[must_use]
    pub(crate) fn loaded_title(&self, feed_guid: &str) -> Option<&str> {
        match self.state_of(feed_guid) {
            Some(PublicPlaylistLoadState::Loaded(page)) => Some(&page.title),
            Some(PublicPlaylistLoadState::Loading | PublicPlaylistLoadState::Failed { .. })
            | None => None,
        }
    }

    fn accepts(&self, feed_guid: &str, generation: u64) -> bool {
        generation == self.generation
            && self.feed_guid.as_deref() == Some(feed_guid)
            && matches!(self.state, Some(PublicPlaylistLoadState::Loading))
    }
}

/// The text of `value` without its surrounding white space, or `None` when
/// it is empty.
fn non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|text| !text.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_models::index_name_list::IndexNameListBatch;
    use crate::view_models::release_page::TrackRowMenuAction;

    fn remote(
        position: Option<i64>,
        feed: Option<&str>,
        track: Option<&str>,
        track_title: Option<&str>,
    ) -> RemoteItem {
        RemoteItem {
            position,
            remote_feed_guid: feed.map(str::to_owned),
            remote_item_guid: track.map(str::to_owned),
            remote_item_title: track_title.map(str::to_owned),
            remote_feed_title: Some("Album".to_owned()),
            remote_feed_image_url: feed.map(|guid| format!("https://img.test/{guid}.jpg")),
            rel: Some("artist".to_owned()),
            ..RemoteItem::default()
        }
    }

    fn playlist(title: Option<&str>, items: Vec<RemoteItem>) -> Feed {
        Feed {
            feed_guid: Some("playlist-guid".to_owned()),
            title: title.map(str::to_owned),
            image_url: None,
            remote_items: Some(items),
            ..Feed::default()
        }
    }

    fn list_feed(guid: Option<&str>, title: Option<&str>) -> Feed {
        Feed {
            feed_guid: guid.map(str::to_owned),
            title: title.map(str::to_owned),
            ..Feed::default()
        }
    }

    /// ADR 0089 packet 009: a playlist page lists its entries in `position`
    /// sequence, and an entry with no position comes last.
    #[test]
    fn adr_0089_public_playlist_items_follow_position_sequence() {
        let feed = playlist(
            Some("Late Night"),
            vec![
                remote(Some(3), Some("album-c"), None, None),
                remote(None, Some("album-x"), None, None),
                remote(Some(1), Some("album-a"), Some("track-1"), Some("Opener")),
            ],
        );
        let page = PublicPlaylistPageVm::from_feed(&feed);
        let guids: Vec<String> = page
            .items
            .iter()
            .map(|item| match &item.target {
                Some(PublicPlaylistItemTarget::Release(key)) => format!("{key:?}"),
                Some(PublicPlaylistItemTarget::Track(key)) => format!("track {}", key.item_guid),
                None => "none".to_owned(),
            })
            .collect();
        assert_eq!(page.items.len(), 3);
        assert_eq!(page.items[0].number, 1);
        assert_eq!(page.items[0].label, "Opener");
        assert_eq!(page.items[1].label, "Album");
        assert!(guids[0].contains("track track-1"));
        assert!(guids[1].contains("album-c"));
        assert!(guids[2].contains("album-x"));
    }

    /// ADR 0089 packet 009: a null `remote_item_guid` is tolerated, and the
    /// entry opens its release page by the feed GUID.
    #[test]
    fn adr_0089_public_playlist_null_item_guid_opens_the_release() {
        let feed = playlist(
            Some("Set"),
            vec![remote(Some(1), Some("album-a"), None, None)],
        );
        let page = PublicPlaylistPageVm::from_feed(&feed);
        let item = &page.items[0];
        assert_eq!(
            item.target,
            Some(PublicPlaylistItemTarget::Release(
                ReleaseKey::from_index_feed_guid("album-a")
            ))
        );
        assert_eq!(item.label, "Album");
        assert_eq!(item.secondary_text, "artist");
    }

    /// ADR 0089 packet 009: an entry with no feed GUID and no track GUID
    /// has no target, so its row cannot open a page.
    #[test]
    fn adr_0089_public_playlist_entry_without_guids_has_no_target() {
        let feed = playlist(Some("Set"), vec![remote(Some(1), None, None, None)]);
        let page = PublicPlaylistPageVm::from_feed(&feed);
        assert_eq!(page.items[0].target, None);
        assert_eq!(page.items[0].label, "Album");
    }

    /// ADR 0089 packet 009: the page art is the playlist image first, then
    /// the mosaic of the entry images (musicindex ADR 0040).
    #[test]
    fn adr_0089_public_playlist_art_is_the_image_then_the_mosaic() {
        let mut feed = playlist(
            None,
            vec![
                remote(Some(1), Some("a"), None, None),
                remote(Some(2), Some("b"), None, None),
            ],
        );
        let page = PublicPlaylistPageVm::from_feed(&feed);
        assert_eq!(page.title, "Untitled playlist");
        assert!(matches!(page.art, CollectionArtVm::Mosaic(_)));

        feed.image_url = Some("https://img.test/own.jpg".to_owned());
        let page = PublicPlaylistPageVm::from_feed(&feed);
        assert!(matches!(page.art, CollectionArtVm::ProvidedImage { .. }));
    }

    /// ADR 0089 packet 009: a feed with no GUID has no row, and the rows
    /// keep the order of the response.
    #[test]
    fn adr_0089_public_playlist_rows_skip_a_feed_with_no_guid() {
        let feeds = [
            list_feed(Some("one"), Some("First")),
            list_feed(None, Some("No guid")),
            list_feed(Some("two"), None),
        ];
        let rows = public_playlist_rows(&feeds);
        let labels: Vec<&str> = rows.iter().map(|row| row.label.as_str()).collect();
        assert_eq!(labels, ["First", "Untitled playlist"]);
        assert_eq!(
            rows[0].open,
            DestinationListRowTarget::PublicPlaylist("one".to_owned())
        );
    }

    /// ADR 0089 packet 009: the list gives the empty state for an empty
    /// response, and an error row for a failed request, through the status
    /// row of the page.
    #[test]
    fn adr_0089_public_playlist_list_status_gives_empty_and_error_states() {
        let mut page: IndexNameListPage<Feed> = IndexNameListPage::default();
        let intent = page.begin_load(false).expect("page 1 starts");
        assert!(page.finish_load(
            IndexNameListBatch {
                rows: Vec::new(),
                cursor: None,
                has_more: false,
            },
            intent.generation,
        ));
        assert!(public_playlist_list_items(&page, None, None).is_empty());

        let mut failed: IndexNameListPage<Feed> = IndexNameListPage::default();
        let intent = failed.begin_load(false).expect("page 1 starts");
        assert!(failed.fail_load("timeout", intent.generation));
        let items = public_playlist_list_items(&failed, None, None);
        assert!(matches!(
            items.as_slice(),
            [PublicPlaylistListItemVm::Status(
                IndexListStatusRowVm::Failed { .. }
            )]
        ));
    }

    /// ADR 0089 packet 009: a read whose generation is old, or whose feed
    /// is not the playlist of the page, changes nothing.
    #[test]
    fn adr_0089_public_playlist_pages_ignore_stale_and_foreign_reads() {
        let mut pages = PublicPlaylistPages::default();
        let old = pages.begin("one").expect("the first read starts");
        let new = pages
            .begin("two")
            .expect("a read of another playlist starts");
        let feed = playlist(Some("Two"), Vec::new());
        assert!(!pages.finish("one", &feed, old), "an old generation");
        assert!(!pages.finish("two", &feed, old), "an old generation");
        assert!(pages.finish("two", &feed, new));
        assert_eq!(pages.loaded_title("two"), Some("Two"));
        assert_eq!(pages.loaded_title("one"), None);
        assert!(
            pages.begin("two").is_none(),
            "a loaded page is not read again"
        );
    }

    /// ADR 0089 packet 009: a failed read starts again on the next open.
    #[test]
    fn adr_0089_public_playlist_pages_retry_after_a_failure() {
        let mut pages = PublicPlaylistPages::default();
        let generation = pages.begin("one").expect("starts");
        assert!(pages.fail("one", "timeout", generation));
        assert_eq!(
            pages.state_of("one"),
            Some(&PublicPlaylistLoadState::Failed {
                detail: "timeout".to_owned()
            })
        );
        assert!(pages.begin("one").is_some());
    }

    /// ADR 0089 packet 009: an item that opens a track or a release gives
    /// the matching navigation entry.
    #[test]
    fn adr_0089_public_playlist_target_gives_its_entry() {
        let track = PublicPlaylistItemTarget::Track(TrackKey::from_index_track_target(
            Some("album-a"),
            "track-1",
        ));
        assert_eq!(
            track.entry(),
            FrameNavigationEntry::Track(TrackKey::from_index_track_target(
                Some("album-a"),
                "track-1"
            ))
        );
    }

    /// ADR 0089 packet 010: the menu of a public playlist has the copy
    /// item, with its accessibility label. A copy that runs removes it.
    #[test]
    fn adr_0089_copy_to_my_playlists_menu_has_the_item_with_its_label() {
        let menu = public_playlist_copy_menu("playlist-guid", "Late Night", false)
            .expect("an idle playlist has a menu");
        assert_eq!(menu.items.len(), 1);
        let item = &menu.items[0];
        assert_eq!(item.action, TrackRowMenuAction::CopyToMyPlaylists);
        assert_eq!(item.label, "Copy to My Playlists\u{2026}");
        assert_eq!(
            item.a11y_label,
            "Copy the public playlist Late Night to My Playlists"
        );
        assert!(!item.destructive);
        assert!(public_playlist_copy_menu("playlist-guid", "Late Night", true).is_none());
    }

    /// ADR 0089 packet 010: the copying playlist shows the busy text, and
    /// no row has a menu while a copy runs.
    #[test]
    fn adr_0089_copy_to_my_playlists_busy_row_shows_the_copy_state() {
        let feeds = [
            Feed {
                feed_guid: Some("one".to_owned()),
                title: Some("First".to_owned()),
                ..Feed::default()
            },
            Feed {
                feed_guid: Some("two".to_owned()),
                title: Some("Second".to_owned()),
                ..Feed::default()
            },
        ];
        let mut page: IndexNameListPage<Feed> = IndexNameListPage::default();
        let intent = page.begin_load(false).expect("page 1 starts");
        assert!(page.finish_load(
            IndexNameListBatch {
                rows: feeds.to_vec(),
                cursor: None,
                has_more: false,
            },
            intent.generation,
        ));

        let idle = public_playlist_list_items(&page, None, None);
        let PublicPlaylistListItemVm::Row(first) = &idle[0] else {
            panic!("the first item is a row");
        };
        assert!(first.menu.is_some());
        assert_ne!(first.row.secondary_text, COPYING_TEXT);

        let busy = public_playlist_list_items(&page, None, Some("one"));
        let PublicPlaylistListItemVm::Row(first) = &busy[0] else {
            panic!("the first item is a row");
        };
        assert_eq!(first.row.secondary_text, COPYING_TEXT);
        assert!(first.menu.is_none());
        let PublicPlaylistListItemVm::Row(second) = &busy[1] else {
            panic!("the second item is a row");
        };
        assert!(second.menu.is_none(), "no menu runs while a copy runs");
    }

    /// ADR 0089 packet 010: a blank name and a name that exists are
    /// rejected with a message. A free name is trimmed.
    #[test]
    fn adr_0089_copy_to_my_playlists_name_is_checked_against_playlists() {
        let existing = ["Road Trip"];
        assert_eq!(
            PublicPlaylistCopyName::check("   ", &existing),
            PublicPlaylistCopyName::Empty
        );
        let taken = PublicPlaylistCopyName::check(" Road Trip ", &existing);
        assert_eq!(taken, PublicPlaylistCopyName::Taken("Road Trip".to_owned()));
        assert_eq!(
            taken.message().as_deref(),
            Some("A playlist named \"Road Trip\" already exists. Enter another name.")
        );
        assert_eq!(
            PublicPlaylistCopyName::check(" Night ", &existing),
            PublicPlaylistCopyName::Ready("Night".to_owned())
        );
        assert_eq!(
            PublicPlaylistCopyName::Ready("Night".to_owned()).message(),
            None
        );
    }

    /// ADR 0089 packet 010: the report names the copied count, the offered
    /// count and the skipped count.
    #[test]
    fn adr_0089_copy_to_my_playlists_report_names_each_count() {
        assert_eq!(
            public_playlist_copy_report("Road Trip", 12, 15),
            "Copied 12 of 15 tracks to Road Trip. 3 could not be read."
        );
        assert_eq!(
            public_playlist_copy_report("Solo", 1, 1),
            "Copied 1 of 1 track to Solo. 0 could not be read."
        );
    }
}
