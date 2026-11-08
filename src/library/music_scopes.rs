//! The scope "In Library" or "All" of the Releases list and the Artists
//! list (ADR 0084 Decision 8, packet 016; ADR 0089 Decision 5).
//!
//! `releases_rows` gives the rows of the Releases list from the cache of
//! ADR 0084 task 037. The list builds again only when one of its inputs
//! changes.
//!
//! `LibraryViewModel` keeps the scope of each list page (ADR 0089
//! Decision 5). The scope "All" loads the `MusicIndex` release name pages or artist
//! name pages through `present_command` on the ADR 0040 command runner. A
//! result of an older request generation changes nothing (decision D10).
//! Each release page load also reads the `MusicIndex` singles of each new
//! artist feed of its singles, in the same command (packet 026 decision
//! D4).
//!
//! The singles group page and the page of a single read the `MusicIndex`
//! singles of one feed when the app has no result for it (packet 026 Part
//! B, decisions D12 and D14). That request also runs through
//! `present_command`, and an answer of an older generation changes
//! nothing.

#![warn(clippy::pedantic)]

use std::rc::Rc;

use gpui::Context;

use super::app_impl::command_error_detail;
use super::{LibraryApp, LibraryDetail};
use crate::application::command_bus::ApplicationCommand;
use crate::application::queries::name_lists::{
    FetchArtistNamePage, FetchFeedSingles, FetchReleaseNamePage,
};
use crate::application::queries::node_sorts::FetchNodeSorts;
use crate::application::CommandContext;
use crate::presentation::present_command;
use crate::view_models::index_feed_singles::{finish_release_page, FeedSinglesResult};
use crate::view_models::index_name_list::{
    IndexNameListBatch, IndexNameListLoadIntent, IndexNameListPage,
};
use crate::view_models::letter_bar::{first_letter_index, LetterKey};
use crate::view_models::library::{
    AlbumNode, ArtistsListItemVm, LibraryViewModel, ReleasesListItemVm,
};
use crate::view_models::list_page::{needs_first_page, ListPageKey, ListScope, ListSort, ListView};
use crate::view_models::list_selection::{ListSelection, SelectionMode};
use crate::view_models::releases_everything::{reset_selection_on_scope_change, ReleasesListRows};
use crate::view_models::singles_group::{
    group_page_feed_to_read, index_page_feed_to_read, library_page_feed_to_read, SinglesGroupKey,
};
use crate::view_models::workspace::MusicDestination;
use crate::views::FeedView;

impl LibraryApp {
    /// The rows of the Releases list in its present scope (ADR 0084 task
    /// 037, Design step 5). Each reader of the list calls this. The list
    /// builds again only when one of its inputs changes, so the header
    /// action and the rows of one frame give one build. The selection,
    /// the hover and the images are not inputs (decision D3).
    pub(crate) fn releases_rows(&self) -> Rc<ReleasesListRows> {
        self.vm.cached_releases_rows(
            &mut self.releases_list_cache.borrow_mut(),
            self.vm.list_scope(ListPageKey::Releases),
            self.releases_sort(),
            &self.release_name_page,
            &self.recent_music_page,
            &self.expanded_singles_groups,
        )
    }

    /// Sets the scope of the active destination (ADR 0089 Decision 5). The
    /// scope "All" loads page 1 of its name list when no request was sent.
    /// A new Releases scope clears the Releases selection and ends
    /// selection mode (decision D15). Each other page keeps its scope.
    pub(crate) fn set_music_list_scope(&mut self, scope: ListScope, cx: &mut Context<Self>) {
        match self.vm.active_destination() {
            MusicDestination::Releases => {
                reset_selection_on_scope_change(
                    self.vm.list_scope(ListPageKey::Releases),
                    scope,
                    &mut self.releases_selection,
                    &mut self.releases_selection_mode,
                );
                self.vm.set_list_scope(ListPageKey::Releases, scope);
            }
            MusicDestination::Artists => {
                self.vm.set_list_scope(ListPageKey::Artists, scope);
            }
            MusicDestination::Search
            | MusicDestination::Inbox
            | MusicDestination::Playlists
            | MusicDestination::PublicPlaylists
            | MusicDestination::Live => return,
        }
        self.request_default_name_list(cx);
        cx.notify();
    }

    /// The sort of the Releases list (ADR 0089 Decision 5). A page with no
    /// sort gives the default, Newest.
    #[must_use]
    pub(crate) fn releases_sort(&self) -> ListSort {
        self.vm
            .list_sort(ListPageKey::Releases)
            .unwrap_or(ListSort::Newest)
    }

    /// Sets the sort of the list page `key` (ADR 0089 Decision 5). The
    /// letter of the page goes with the old sort. The Releases selection
    /// ends too, because the rows of the new sort are another list.
    pub(crate) fn set_list_sort(
        &mut self,
        key: ListPageKey,
        sort: ListSort,
        cx: &mut Context<Self>,
    ) {
        if self.vm.list_sort(key) == Some(sort) {
            return;
        }
        // Musicindex ADR 0053 R-LIST-9: a group page, the tracks of a release
        // and the entries of a playlist sort rows that the app already holds. No request starts and the selection stays,
        // because it is by item.
        if matches!(
            key,
            ListPageKey::SinglesGroup
                | ListPageKey::WindowGroup
                | ListPageKey::ReleaseTracks
                | ListPageKey::PublicPlaylistItems
        ) {
            self.vm.set_list_sort(key, sort);
            cx.notify();
            return;
        }
        // ADR 0089 packet 009: the Public playlists list reads another
        // endpoint in each sort, so a new sort starts its list again.
        if key == ListPageKey::PublicPlaylists {
            self.vm.set_list_sort(key, sort);
            self.restart_public_playlist_list(None, cx);
            cx.notify();
            return;
        }
        // The by-name list starts again at its first page. The sort Newest
        // reads the recent pager instead, which keeps its own state.
        if sort == ListSort::AToZ {
            self.restart_name_list_at(key, None, cx);
        }
        self.vm.set_list_sort(key, sort);
        if key == ListPageKey::Releases {
            self.releases_selection = ListSelection::new();
            self.releases_selection_mode = SelectionMode::Normal;
            self.request_default_name_list(cx);
        }
        cx.notify();
    }

    /// Sets the reversed state of the list page `key` (musicindex ADR 0043
    /// R-LIST-7). The mounted list builds again in place. The letter, the
    /// selection and the Select state stay, because only a new sort ends
    /// them.
    pub(crate) fn set_list_reversed(
        &mut self,
        key: ListPageKey,
        reversed: bool,
        cx: &mut Context<Self>,
    ) {
        if self.vm.list_page_reversed_state(key) == reversed {
            return;
        }
        self.vm.set_list_reversed(key, reversed);
        self.restart_index_list_for_reverse(key, cx);
        cx.notify();
    }

    /// Starts the Index list of `key` again when its reverse changed
    /// (musicindex ADR 0053 R-LIST-8 item 4, ADR 0043 R-LIST-7 item 9). A
    /// reversed cursor works only with `reverse=true`, so the list starts at
    /// its first row, from the same `from` text. The restart runs in each
    /// scope, so the loaded Index list keeps the order of its page. A list
    /// that never started loads with its state.
    fn restart_index_list_for_reverse(&mut self, key: ListPageKey, cx: &mut Context<Self>) {
        match key {
            ListPageKey::Releases => {
                if self.releases_sort() == ListSort::Newest {
                    if !self.recent_music_page.is_not_started() {
                        self.start_recent_music_load(false, cx);
                    }
                } else if !self.release_name_page.is_not_started() {
                    let from = self.release_name_page.from().map(str::to_owned);
                    let intent = self.release_name_page.begin_from(from);
                    self.send_release_name_request(intent, cx);
                }
            }
            ListPageKey::Artists => {
                if !self.artist_name_page.is_not_started() {
                    let from = self.artist_name_page.from().map(str::to_owned);
                    let intent = self.artist_name_page.begin_from(from);
                    self.send_artist_name_request(intent, cx);
                }
            }
            ListPageKey::PublicPlaylists => {
                if !self.public_playlist_list.is_not_started() {
                    let from = self.public_playlist_list.from().map(str::to_owned);
                    self.restart_public_playlist_list(from, cx);
                }
            }
            ListPageKey::MyPlaylists
            | ListPageKey::Live
            | ListPageKey::Results
            | ListPageKey::SavedSearch
            | ListPageKey::SinglesGroup
            | ListPageKey::WindowGroup
            | ListPageKey::ReleaseTracks
            | ListPageKey::PublicPlaylistItems
            | ListPageKey::ReleaseSection => {}
        }
    }

    /// Reads whether the Index node lists `sort`, once per app session
    /// (musicindex ADR 0053 R-LIST-8 item 4). The answer sets the reverse
    /// button of each Index list. A failed read lets the next list load ask
    /// again.
    fn request_index_sorts(&mut self, cx: &mut Context<Self>) {
        if self.index_sorts_requested {
            return;
        }
        self.index_sorts_requested = true;
        present_command(
            &self.command_runner,
            FetchNodeSorts::new(self.musicindex_endpoint.clone()),
            CommandContext::next(),
            cx,
            move |this, node, cx| {
                this.vm.set_index_sorts(node);
                cx.notify();
            },
            move |this, _error, _cx| {
                this.index_sorts_requested = false;
            },
        );
    }

    /// Sets the view of the list page `key` (ADR 0089 Decision 5, R-LIST-1
    /// item 3). Each page keeps its own view. The rows and the tiles show
    /// the same items, so the selection stays.
    pub(crate) fn set_list_view(
        &mut self,
        key: ListPageKey,
        view: ListView,
        cx: &mut Context<Self>,
    ) {
        if self.vm.list_view(key) == view {
            return;
        }
        self.vm.set_list_view(key, view);
        cx.notify();
    }

    /// Selects the letter `letter` of the list page `key` (ADR 0089
    /// Decision 6, R-LIST-1 item 2). In the scope "All" the letter starts
    /// the name list at its key, and the rows of the list are replaced. In
    /// the scope "In Library" the list moves to the first row of the key.
    pub(crate) fn set_list_letter(
        &mut self,
        key: ListPageKey,
        letter: Option<LetterKey>,
        cx: &mut Context<Self>,
    ) {
        self.vm.set_list_letter(key, letter);
        match self.vm.list_scope(key) {
            ListScope::All => {
                self.restart_name_list_at(key, letter.and_then(LetterKey::to_from_text), cx);
            }
            ListScope::InLibrary => {
                if let Some(letter) = letter {
                    self.scroll_list_to_letter(key, letter);
                }
            }
        }
        cx.notify();
    }

    /// Starts the name list of `key` again at `from`, when the list starts
    /// at another text now. The request is sent in every scope, so the
    /// list state keeps the text of its letter.
    fn restart_name_list_at(
        &mut self,
        key: ListPageKey,
        from: Option<String>,
        cx: &mut Context<Self>,
    ) {
        match key {
            ListPageKey::Releases => {
                if self.release_name_page.from() == from.as_deref() {
                    return;
                }
                let intent = self.release_name_page.begin_from(from);
                self.send_release_name_request(intent, cx);
            }
            ListPageKey::Artists => {
                if self.artist_name_page.from() == from.as_deref() {
                    return;
                }
                let intent = self.artist_name_page.begin_from(from);
                self.send_artist_name_request(intent, cx);
            }
            ListPageKey::PublicPlaylists => {
                if self.public_playlist_list.from() == from.as_deref() {
                    return;
                }
                self.restart_public_playlist_list(from, cx);
            }
            ListPageKey::MyPlaylists
            | ListPageKey::Live
            | ListPageKey::Results
            | ListPageKey::SavedSearch
            | ListPageKey::SinglesGroup
            | ListPageKey::WindowGroup
            | ListPageKey::ReleaseTracks
            | ListPageKey::PublicPlaylistItems
            | ListPageKey::ReleaseSection => {}
        }
    }

    /// Moves the In Library list of `key` to the first row of `letter`. A
    /// list with no row of the letter stays where it is.
    fn scroll_list_to_letter(&mut self, key: ListPageKey, letter: LetterKey) {
        match key {
            ListPageKey::Releases => {
                let rows = self.releases_rows();
                let labels = rows.items().iter().map(ReleasesListItemVm::label);
                if let Some(index) = first_letter_index(labels, letter) {
                    if matches!(self.vm.list_view(ListPageKey::Releases), ListView::Tiles) {
                        self.reveal_releases_tile(rows.item_element_at(index));
                    } else {
                        self.releases_list_state.scroll_to_reveal_item(index);
                    }
                }
            }
            ListPageKey::Artists => {
                let scope = self.vm.list_scope(ListPageKey::Artists);
                let rows = self.vm.artists_list(scope, &self.artist_name_page);
                let status = LibraryViewModel::artists_list_status(scope, &self.artist_name_page);
                let items = ArtistsListItemVm::of(rows, status);
                let labels = items.iter().map(ArtistsListItemVm::label);
                if let Some(index) = first_letter_index(labels, letter) {
                    if matches!(self.vm.list_view(ListPageKey::Artists), ListView::Tiles) {
                        let element = items.get(index).map(ArtistsListItemVm::element_id);
                        self.reveal_artists_tile(element);
                    } else {
                        self.artists_list_state.scroll_to_reveal_item(index);
                    }
                }
            }
            // The Public playlists list has no scope, so the In Library
            // scroll never reaches it (ADR 0089 packet 009).
            ListPageKey::MyPlaylists
            | ListPageKey::PublicPlaylists
            | ListPageKey::Live
            | ListPageKey::Results
            | ListPageKey::SavedSearch
            | ListPageKey::SinglesGroup
            | ListPageKey::WindowGroup
            | ListPageKey::ReleaseTracks
            | ListPageKey::PublicPlaylistItems
            | ListPageKey::ReleaseSection => {}
        }
    }

    /// Sends page 1 of the name list of the active destination, when its
    /// scope is "All" and no request was sent yet (ADR 0089 Decision 5).
    /// The Library rows show before the answer. Each list sends at most one
    /// page 1 request, because the request changes the list state.
    pub(crate) fn request_default_name_list(&mut self, cx: &mut Context<Self>) {
        match self.vm.active_destination() {
            MusicDestination::Releases => {
                self.request_index_sorts(cx);
                let scope = self.vm.list_scope(ListPageKey::Releases);
                if self.releases_sort() == ListSort::Newest {
                    // ADR 0089 Decision 5: the sort Newest sends the first
                    // recent request, once. The pager keeps its rows.
                    if needs_first_page(scope, self.recent_music_page.is_not_started()) {
                        self.start_recent_music_load(false, cx);
                    }
                } else if needs_first_page(scope, self.release_name_page.is_not_started()) {
                    let intent = self.release_name_page.begin_load(false);
                    self.send_release_name_request(intent, cx);
                }
            }
            MusicDestination::Artists => {
                self.request_index_sorts(cx);
                if needs_first_page(
                    self.vm.list_scope(ListPageKey::Artists),
                    self.artist_name_page.is_not_started(),
                ) {
                    let intent = self.artist_name_page.begin_load(false);
                    self.send_artist_name_request(intent, cx);
                }
            }
            MusicDestination::PublicPlaylists => {
                self.request_index_sorts(cx);
                self.request_public_playlists_first_page(cx);
            }
            // ADR 0089 packet 011: the Live list reads again when its
            // destination shows, from the navigation hook, not here. A
            // Library load does not start a Live read.
            MusicDestination::Search
            | MusicDestination::Inbox
            | MusicDestination::Playlists
            | MusicDestination::Live => {}
        }
    }

    /// "Load more" at the end of the Artists list (packet 016 decision D7).
    pub(crate) fn load_more_artist_names(&mut self, cx: &mut Context<Self>) {
        let intent = self.artist_name_page.begin_load(true);
        self.send_artist_name_request(intent, cx);
    }

    /// "Try again" on the error row of the Artists list (packet 016
    /// decision D8).
    pub(crate) fn retry_artist_names(&mut self, cx: &mut Context<Self>) {
        let intent = self.artist_name_page.begin_retry();
        self.send_artist_name_request(intent, cx);
    }

    /// "Load more" at the end of the Releases list (packet 016 decision
    /// D7). The sort Newest loads the next recent page.
    pub(crate) fn load_more_release_names(&mut self, cx: &mut Context<Self>) {
        if self.releases_sort() == ListSort::Newest {
            self.start_recent_music_load(true, cx);
            return;
        }
        let intent = self.release_name_page.begin_load(true);
        self.send_release_name_request(intent, cx);
    }

    /// "Try again" on the error row of the Releases list (packet 016
    /// decision D8). When the rows of the page wait for a failed feed
    /// request, the command sends only the failed requests (packet 026
    /// decision D10). The sort Newest sends the recent request again.
    pub(crate) fn retry_release_names(&mut self, cx: &mut Context<Self>) {
        if self.releases_sort() == ListSort::Newest {
            let append = self.recent_music_page.row_count() > 0;
            self.start_recent_music_load(append, cx);
            return;
        }
        if let Some((pending, generation)) = self.release_name_page.begin_pending_retry() {
            let command =
                FetchReleaseNamePage::retry_pending(self.musicindex_endpoint.clone(), pending);
            self.send_release_page_command(command, generation, cx);
            return;
        }
        let intent = self.release_name_page.begin_retry();
        self.send_release_name_request(intent, cx);
    }

    fn send_release_name_request(
        &mut self,
        intent: Option<IndexNameListLoadIntent>,
        cx: &mut Context<Self>,
    ) {
        let Some(IndexNameListLoadIntent {
            cursor,
            generation,
            from,
        }) = intent
        else {
            return;
        };
        let reverse = self.vm.list_index_reversed(ListPageKey::Releases);
        let command =
            FetchReleaseNamePage::new(self.musicindex_endpoint.clone(), cursor, from, reverse);
        self.send_release_page_command(command, generation, cx);
    }

    /// Sends the release page `command` of the page `generation`. Each page
    /// also reads each Library artist feed with one or more Library singles
    /// (packet 026). The command sends no request for a feed with a loaded
    /// result, and no request for a feed whose request already runs, for
    /// example a request that the group page or the single page started
    /// (packet 026 review). A feed that a Library change adds after page 1
    /// is thus read with the next page.
    fn send_release_page_command(
        &mut self,
        command: FetchReleaseNamePage,
        generation: u64,
        cx: &mut Context<Self>,
    ) {
        let library_feeds = self.vm.library_single_artist_feeds();
        let command = command.with_feeds(
            library_feeds,
            self.vm.index_feed_singles().loaded_or_running_feeds(),
        );
        let singles_generation = self.vm.index_feed_singles_mut().begin_request();
        cx.notify();
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, load, cx| {
                if finish_release_page(
                    &mut this.release_name_page,
                    this.vm.index_feed_singles_mut(),
                    load,
                    generation,
                    singles_generation,
                ) {
                    cx.notify();
                }
            },
            move |this, error, cx| {
                if this
                    .release_name_page
                    .fail_load(command_error_detail(error), generation)
                {
                    cx.notify();
                }
            },
        );
    }

    /// Reads the `MusicIndex` singles of the feed of the group page `key`,
    /// when the app has no result for it and no request runs (packet 026
    /// Part B). A name identity sends no request.
    pub(crate) fn request_singles_group_feed(
        &mut self,
        key: &SinglesGroupKey,
        cx: &mut Context<Self>,
    ) {
        if let Some(feed_guid) = group_page_feed_to_read(key, self.vm.index_feed_singles()) {
            self.request_feed_singles(feed_guid, cx);
        }
    }

    /// "Try again" on the error row of the open group page (packet 026
    /// Part B). It sends the request for the feed of the page again.
    pub(crate) fn retry_singles_group_feed(&mut self, cx: &mut Context<Self>) {
        if let LibraryDetail::SinglesGroup(key) = &self.detail {
            let key = key.clone();
            self.request_singles_group_feed(&key, cx);
        }
    }

    /// Reads the `MusicIndex` singles of the artist feed of the Library
    /// single `album`, when the app has no result for it (packet 026
    /// decision D14). The page then shows "All n singles".
    pub(crate) fn request_library_single_feed(
        &mut self,
        album: &AlbumNode,
        cx: &mut Context<Self>,
    ) {
        if let Some(feed_guid) = library_page_feed_to_read(album, self.vm.index_feed_singles()) {
            self.request_feed_singles(feed_guid, cx);
        }
    }

    /// Reads the `MusicIndex` singles of the artist feed of the loaded
    /// Index single `feed`, when the app has no result for it (packet 026
    /// decision D14).
    pub(crate) fn request_index_single_feed(&mut self, feed: &FeedView, cx: &mut Context<Self>) {
        if let Some(feed_guid) = index_page_feed_to_read(feed, self.vm.index_feed_singles()) {
            self.request_feed_singles(feed_guid, cx);
        }
    }

    /// Sends one request for the singles of `feed_guid`. It sends nothing
    /// when the app has a result or a request runs, so each page open
    /// sends at most one request (packet 026 Part B).
    fn request_feed_singles(&mut self, feed_guid: String, cx: &mut Context<Self>) {
        let Some(generation) = self
            .vm
            .index_feed_singles_mut()
            .begin_feed_request(&feed_guid)
        else {
            return;
        };
        cx.notify();
        let failed_guid = feed_guid.clone();
        present_command(
            &self.command_runner,
            FetchFeedSingles::new(self.musicindex_endpoint.clone(), feed_guid),
            CommandContext::next(),
            cx,
            move |this, result, cx| {
                if this
                    .vm
                    .index_feed_singles_mut()
                    .finish_feed_request(result, generation)
                {
                    cx.notify();
                }
            },
            move |this, error, cx| {
                let result = FeedSinglesResult {
                    feed_guid: failed_guid,
                    rows: Err(command_error_detail(error)),
                };
                if this
                    .vm
                    .index_feed_singles_mut()
                    .finish_feed_request(result, generation)
                {
                    cx.notify();
                }
            },
        );
    }

    fn send_artist_name_request(
        &mut self,
        intent: Option<IndexNameListLoadIntent>,
        cx: &mut Context<Self>,
    ) {
        let endpoint = self.musicindex_endpoint.clone();
        let reverse = self.vm.list_index_reversed(ListPageKey::Artists);
        self.send_name_request(
            intent,
            |cursor, from| FetchArtistNamePage::new(endpoint, cursor, from, reverse),
            |this| &mut this.artist_name_page,
            cx,
        );
    }

    /// Sends the request of `intent`, when there is one. `command` makes
    /// the command for the cursor, and `page` names the page state of the
    /// list. The result changes the page only when its generation is
    /// current.
    fn send_name_request<C, Row>(
        &mut self,
        intent: Option<IndexNameListLoadIntent>,
        command: impl FnOnce(Option<String>, Option<String>) -> C,
        page: fn(&mut Self) -> &mut IndexNameListPage<Row>,
        cx: &mut Context<Self>,
    ) where
        C: ApplicationCommand<Output = IndexNameListBatch<Row>>,
        Row: 'static,
    {
        let Some(IndexNameListLoadIntent {
            cursor,
            generation,
            from,
        }) = intent
        else {
            return;
        };
        cx.notify();
        present_command(
            &self.command_runner,
            command(cursor, from),
            CommandContext::next(),
            cx,
            move |this, batch, cx| {
                if page(this).finish_load(batch, generation) {
                    cx.notify();
                }
            },
            move |this, error, cx| {
                if page(this).fail_load(command_error_detail(error), generation) {
                    cx.notify();
                }
            },
        );
    }
}
