use gpui::App;

use crate::view_models::app_toolbar::{
    AppToolbarVm, ToolbarHistoryControls, ToolbarNavigationDisplay,
};
use crate::view_models::workspace::{FrameNavigationEntry, ReleaseKey, TrackKey};

use super::{TopApp, WorkspaceScreenMount};

/// The loaded facts that name a page (ADR 0089 Decision 9). The app reads
/// them from the Library, the Index and the publisher page. The title
/// choice in [`music_page_title`] reads no data itself.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PageTitleFacts {
    /// The name that the loaded data gives the page, for example the Library
    /// album name, the loaded Index title or the feed title. `None` while the
    /// data is missing.
    pub(crate) loaded_title: Option<String>,
    /// The label that the opening row gave the page, for the time before the
    /// data loads. `None` when the row gave none.
    pub(crate) opening_label: Option<String>,
}

/// The type title of a page, before its own name is known (ADR 0084
/// Decisions 4 and 5). Pure: it reads only the entry.
#[must_use]
pub(crate) fn page_type_title(entry: &FrameNavigationEntry) -> String {
    match entry {
        FrameNavigationEntry::Destination(_) | FrameNavigationEntry::Settings => String::new(),
        FrameNavigationEntry::DiscoverView(view) => view.label().to_string(),
        FrameNavigationEntry::SavedSearch { .. } => {
            crate::view_models::search_results::SavedSearch::FRAME_TITLE.to_string()
        }
        FrameNavigationEntry::Search(_) => "Search Results".to_string(),
        FrameNavigationEntry::PlaylistDetail(_) => "Playlist".to_string(),
        // ADR 0084 Decision 4/5: the curator sees "Track" for a track and
        // "Release" for a feed of music, whatever its origin.
        FrameNavigationEntry::Track(_) => "Track".to_string(),
        FrameNavigationEntry::Release(_) => "Release".to_string(),
        // ADR 0084 packet 017 decision D5: the title of the name level page.
        FrameNavigationEntry::ArtistName(_) => {
            crate::view_models::name_level_page::NameLevelPageVm::FRAME_TITLE.to_string()
        }
        // ADR 0085 Decision 7: the feed title, before the page facts load.
        FrameNavigationEntry::PublisherDetail(_) => "Feed".to_string(),
        FrameNavigationEntry::SinglesGroup(_) | FrameNavigationEntry::WindowGroup(_) => {
            "Singles".to_string()
        }
        FrameNavigationEntry::QueueNowPlaying => "Queue".to_string(),
    }
}

/// The title of a page of the Music surface (ADR 0089 Decision 9). Pure:
/// the entry and its loaded facts give the title, with no read of data.
///
/// A named page (a release, a track, a singles group, a window group or a
/// feed) shows its loaded name. Until the name loads, it shows the label of
/// its opening row. Then it shows its type title.
#[must_use]
pub(crate) fn music_page_title(entry: &FrameNavigationEntry, facts: &PageTitleFacts) -> String {
    match entry {
        FrameNavigationEntry::SinglesGroup(_)
        | FrameNavigationEntry::WindowGroup(_)
        | FrameNavigationEntry::PublisherDetail(_)
        | FrameNavigationEntry::Release(_)
        | FrameNavigationEntry::Track(_) => facts
            .loaded_title
            .clone()
            .or_else(|| facts.opening_label.clone())
            .unwrap_or_else(|| page_type_title(entry)),
        _ => page_type_title(entry),
    }
}

impl TopApp {
    /// Reads the facts that name the page `entry` (ADR 0089 Decision 9). The
    /// choice of the title is in [`music_page_title`].
    fn page_title_facts(&self, entry: &FrameNavigationEntry, cx: &App) -> PageTitleFacts {
        match entry {
            // ADR 0084 Decision 6 and ADR 0086 Decision 4: the Library gives
            // the title with the count. For example: "Singles by Haleen".
            FrameNavigationEntry::SinglesGroup(_) | FrameNavigationEntry::WindowGroup(_) => {
                PageTitleFacts {
                    loaded_title: Some(self.library.read(cx).navigation_label(entry)),
                    opening_label: None,
                }
            }
            // ADR 0085 Decision 7: the feed title, when the page loads.
            FrameNavigationEntry::PublisherDetail(_) => PageTitleFacts {
                loaded_title: self.publisher_page_frame_title(),
                opening_label: None,
            },
            FrameNavigationEntry::Release(key) => self.release_page_facts(key, cx),
            FrameNavigationEntry::Track(key) => self.track_page_facts(key, cx),
            _ => PageTitleFacts::default(),
        }
    }

    /// The facts of a release page (ADR 0084 Decision 4): the Library album
    /// name, or the loaded Index title, and the label of the opening row.
    fn release_page_facts(&self, key: &ReleaseKey, cx: &App) -> PageTitleFacts {
        match key {
            ReleaseKey::FeedGuid(feed_guid) => PageTitleFacts {
                loaded_title: self
                    .library
                    .read(cx)
                    .release_label(key)
                    .or_else(|| self.index_feed_loaded_title(feed_guid)),
                opening_label: Some(self.release_opening_label(feed_guid)),
            },
            ReleaseKey::LibraryFeed(_) => PageTitleFacts {
                loaded_title: self.library.read(cx).release_label(key),
                opening_label: None,
            },
        }
    }

    /// The facts of a track page (ADR 0084 Decision 4): the Library track
    /// title, or the loaded Index title, and the label of the opening row.
    fn track_page_facts(&self, key: &TrackKey, cx: &App) -> PageTitleFacts {
        let target = key.index_target();
        PageTitleFacts {
            loaded_title: self
                .library
                .read(cx)
                .track_label(key)
                .or_else(|| self.index_track_loaded_title(&target)),
            opening_label: Some(self.track_opening_label(&target)),
        }
    }

    /// The title of the page `entry` (ADR 0089 Decision 9): the facts are
    /// read here, and the title is chosen by [`music_page_title`].
    pub(super) fn music_page_title_now(&self, entry: &FrameNavigationEntry, cx: &App) -> String {
        let facts = self.page_title_facts(entry, cx);
        music_page_title(entry, &facts)
    }

    /// The toolbar's Back and Forward controls and page title (ADR 0089
    /// Decision 9, R-NAV-2, R-TOOLBAR-1). Music shows the title of the
    /// visible frame. It shows both controls, and Forward is disabled when
    /// no later page exists. Show and Settings show no title. Each control
    /// shows only when its history exists.
    pub(super) fn toolbar_navigation(
        &self,
        mount: WorkspaceScreenMount,
        cx: &App,
    ) -> ToolbarNavigationDisplay {
        let frame_id = self.content_list_frame_id();
        let nav = frame_id.and_then(|id| self.workspace_layout.frame_nav(id));
        match mount {
            WorkspaceScreenMount::Music => {
                // ADR 0089 Decision 1: the title follows the frame that shows.
                // A page with no content shows the list root, so its title is
                // the list title.
                let title = match (nav, frame_id) {
                    (Some(nav), Some(frame_id)) => {
                        let (columns, _) = self.music_frame_for(mount, Some(nav.current()), cx);
                        if columns.shows_detail() {
                            self.music_page_title_now(nav.current(), cx)
                        } else {
                            Self::music_list_frame_title(&self.workspace_layout, frame_id)
                        }
                    }
                    _ => String::new(),
                };
                AppToolbarVm::new().navigation(nav, title, ToolbarHistoryControls::Always)
            }
            WorkspaceScreenMount::Show | WorkspaceScreenMount::Settings => AppToolbarVm::new()
                .navigation(nav, String::new(), ToolbarHistoryControls::WhenAvailable),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{music_page_title, page_type_title, PageTitleFacts};
    use crate::view_models::artist_identity::ArtistIdentity;
    use crate::view_models::search_results::SavedSearchId;
    use crate::view_models::singles_group::SinglesGroupKey;
    use crate::view_models::workspace::{FrameNavigationEntry, ReleaseKey, TrackKey};

    fn facts(loaded: Option<&str>, opening: Option<&str>) -> PageTitleFacts {
        PageTitleFacts {
            loaded_title: loaded.map(str::to_owned),
            opening_label: opening.map(str::to_owned),
        }
    }

    fn release() -> FrameNavigationEntry {
        FrameNavigationEntry::Release(ReleaseKey::FeedGuid("guid-a".to_owned()))
    }

    fn track() -> FrameNavigationEntry {
        FrameNavigationEntry::Track(TrackKey::from_index_track_target(None, "item-1"))
    }

    fn singles_group() -> FrameNavigationEntry {
        FrameNavigationEntry::SinglesGroup(SinglesGroupKey(ArtistIdentity::Publisher(
            "haleen-guid".to_owned(),
        )))
    }

    fn feed() -> FrameNavigationEntry {
        FrameNavigationEntry::PublisherDetail("feed-guid".to_owned())
    }

    /// A release shows its loaded name, then its opening label, then "Release".
    #[test]
    fn adr_0089_title_release_uses_loaded_then_opening_then_type() {
        assert_eq!(
            music_page_title(&release(), &facts(Some("Loaded album"), Some("Row label"))),
            "Loaded album"
        );
        assert_eq!(
            music_page_title(&release(), &facts(None, Some("Row label"))),
            "Row label"
        );
        assert_eq!(music_page_title(&release(), &facts(None, None)), "Release");
    }

    /// A track shows its loaded name, then its opening label, then "Track".
    #[test]
    fn adr_0089_title_track_uses_loaded_then_opening_then_type() {
        assert_eq!(
            music_page_title(&track(), &facts(Some("Loaded track"), Some("Row label"))),
            "Loaded track"
        );
        assert_eq!(
            music_page_title(&track(), &facts(None, Some("Row label"))),
            "Row label"
        );
        assert_eq!(music_page_title(&track(), &facts(None, None)), "Track");
    }

    /// A singles group shows the loaded group title, then "Singles".
    #[test]
    fn adr_0089_title_singles_group_uses_loaded_then_type() {
        assert_eq!(
            music_page_title(&singles_group(), &facts(Some("Singles by Haleen"), None)),
            "Singles by Haleen"
        );
        assert_eq!(
            music_page_title(&singles_group(), &facts(None, None)),
            "Singles"
        );
    }

    /// A feed page shows the loaded feed title, then "Feed".
    #[test]
    fn adr_0089_title_feed_page_uses_loaded_then_type() {
        assert_eq!(
            music_page_title(&feed(), &facts(Some("Feed title"), None)),
            "Feed title"
        );
        assert_eq!(music_page_title(&feed(), &facts(None, None)), "Feed");
    }

    /// A page with no named facts shows its type title, whatever facts exist.
    #[test]
    fn adr_0089_title_unnamed_page_ignores_facts() {
        let saved = FrameNavigationEntry::SavedSearch {
            id: SavedSearchId(3),
            query: "bluegrass".to_owned(),
        };
        assert_eq!(
            music_page_title(&saved, &facts(Some("Ignored"), Some("Ignored"))),
            "Saved search"
        );
        assert_eq!(
            page_type_title(&FrameNavigationEntry::PlaylistDetail(1)),
            "Playlist"
        );
    }
}
