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
use crate::application::queries::name_lists::{
    FetchArtistNamePage, FetchFeedSingles, FetchReleaseNamePage,
};
use crate::application::queries::node_sorts::FetchNodeSorts;
use crate::application::CommandContext;
use crate::presentation::present_command;
use crate::ui::shells::library::index_list_status::IndexListActions;
use crate::view_models::artist_groups::{ArtistGroup, ArtistGroupIntent};
use crate::view_models::index_feed_singles::{finish_release_page, FeedSinglesResult};
use crate::view_models::index_name_list::IndexNameListLoadIntent;
use crate::view_models::letter_bar::{first_letter_index, LetterKey};
use crate::view_models::library::{AlbumNode, ArtistsListItemVm, ReleasesListItemVm};
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
        // Musicindex ADR 0053 R-LIST-9: the Artists node list reads the sort
        // from its request, so a new sort starts the list again at page 1,
        // with no `from` text. A list that never started loads with its state.
        if key == ListPageKey::Artists {
            self.vm.set_list_sort(key, sort);
            if !self.artist_pages.is_not_started() {
                let intents = self.artist_pages.begin_from(None);
                self.send_artist_group_requests(intents, cx);
            }
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

    /// Sets the sort and the direction of the list page `key` together, from
    /// one entry of the sort menu (musicindex ADR 0054 R-LIST-13). A new sort
    /// has the effects of [`Self::set_list_sort`], and a new direction has
    /// those of [`Self::set_list_reversed`]. A change of both starts each Index
    /// list once, with the new direction.
    pub(crate) fn set_list_order(
        &mut self,
        key: ListPageKey,
        sort: ListSort,
        reversed: bool,
        cx: &mut Context<Self>,
    ) {
        if self.vm.list_sort(key) == Some(sort) {
            self.set_list_reversed(key, reversed, cx);
            return;
        }
        let reversed_changed = self.vm.list_page_reversed_state(key) != reversed;
        // The restart of the new sort reads the new direction.
        self.vm.set_list_reversed(key, reversed);
        self.set_list_sort(key, sort, cx);
        // The Releases sort Newest keeps its pager when it starts, so a new
        // direction needs its own restart.
        if reversed_changed && key == ListPageKey::Releases && sort == ListSort::Newest {
            self.restart_index_list_for_reverse(key, cx);
            cx.notify();
        }
    }

    /// Sets the reversed state of the list page `key` (musicindex ADR 0054
    /// R-LIST-14, the Reverse toggle). The mounted list builds again in place. The letter, the
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

    /// The sort and the direction of the Artists node request now. A change
    /// of this pair after a load needs a new first page.
    fn artists_node_request(&self) -> (crate::api::ArtistNameSort, bool) {
        (
            self.vm.artist_node_sort(),
            self.vm.list_index_reversed(ListPageKey::Artists),
        )
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
                if !self.artist_pages.is_not_started() {
                    let from = self.artist_pages.from().map(str::to_owned);
                    let intents = self.artist_pages.begin_from(from.as_deref());
                    self.send_artist_group_requests(intents, cx);
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
                let before = this.artists_node_request();
                this.vm.set_index_sorts(node);
                // The first Artists request can leave before this answer. A
                // node that lists `artist_newest` gives the sort Newest, so
                // the loaded list starts again with the request that the
                // answer gives (musicindex ADR 0053 R-LIST-9).
                if this.artists_node_request() != before && !this.artist_pages.is_not_started() {
                    let from = (this.vm.artist_node_sort() == crate::api::ArtistNameSort::Name)
                        .then(|| this.artist_pages.from().map(str::to_owned))
                        .flatten();
                    let intents = this.artist_pages.begin_from(from.as_deref());
                    this.send_artist_group_requests(intents, cx);
                }
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
                if self.artist_pages.from() == from.as_deref() {
                    return;
                }
                let intents = self.artist_pages.begin_from(from.as_deref());
                self.send_artist_group_requests(intents, cx);
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
                let sections = self.vm.artists_sections(scope, &self.artist_pages);
                let items = ArtistsListItemVm::of_sections(sections);
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
                    self.artist_pages.is_not_started(),
                ) {
                    let intents = self.artist_pages.begin_first_pages();
                    self.send_artist_group_requests(intents, cx);
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

    /// "Load more" at the end of the stated artists (packet 016 decision D7,
    /// musicindex ADR 0057 R-SITE-15 item 2). It loads the next page of the
    /// group.
    pub(crate) fn load_more_artist_stated(&mut self, cx: &mut Context<Self>) {
        let intent = self
            .artist_pages
            .page_mut(ArtistGroup::Stated)
            .begin_load(true);
        self.send_artist_group_request(ArtistGroup::Stated, intent, cx);
    }

    /// "Show more" of the inferred artists (musicindex ADR 0057 R-SITE-15
    /// item 2).
    pub(crate) fn show_more_artist_inferred(&mut self, cx: &mut Context<Self>) {
        self.show_more_artist_group(ArtistGroup::Inferred, cx);
    }

    /// "Show more" of the artist names with no feed (musicindex ADR 0057
    /// R-SITE-15 item 2).
    pub(crate) fn show_more_artist_names(&mut self, cx: &mut Context<Self>) {
        self.show_more_artist_group(ArtistGroup::Names, cx);
    }

    /// Shows the next rows of `group`. The next page loads at once when the
    /// loaded rows are fewer than the rows that the group shows now. The
    /// Artists list has no list focus, so the focus does not move to the
    /// first new row (musicindex ADR 0057 R-SITE-15 item 4).
    fn show_more_artist_group(&mut self, group: ArtistGroup, cx: &mut Context<Self>) {
        self.artist_pages.show_more(group);
        self.continue_artist_group(group, cx);
        cx.notify();
    }

    /// "Try again" on the error row of the Artists list (packet 016
    /// decision D8). It sends again each request that failed.
    pub(crate) fn retry_artist_names(&mut self, cx: &mut Context<Self>) {
        let intents = self.artist_pages.begin_retries();
        self.send_artist_group_requests(intents, cx);
    }

    /// The actions of the status rows of `group` in the Artists list.
    #[must_use]
    pub(crate) fn artist_group_actions(group: ArtistGroup) -> IndexListActions {
        IndexListActions {
            retry: Self::retry_artist_names,
            load_more: Some(match group {
                ArtistGroup::Stated => Self::load_more_artist_stated,
                ArtistGroup::Inferred => Self::show_more_artist_inferred,
                ArtistGroup::Names => Self::show_more_artist_names,
            }),
        }
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

    /// Sends the request of page 1 of each group (musicindex ADR 0057
    /// R-ARTIST-1). Each group has its own page and its own request.
    fn send_artist_group_requests(
        &mut self,
        intents: Vec<ArtistGroupIntent>,
        cx: &mut Context<Self>,
    ) {
        for (group, intent) in intents {
            self.send_artist_group_request(group, intent, cx);
        }
    }

    /// Sends the request of `intent` for the page of `group`. The result
    /// changes the page only when its generation is current. A limited group
    /// that shows more rows than it holds loads its next page at once.
    fn send_artist_group_request(
        &mut self,
        group: ArtistGroup,
        intent: Option<IndexNameListLoadIntent>,
        cx: &mut Context<Self>,
    ) {
        let endpoint = self.musicindex_endpoint.clone();
        let reverse = self.vm.list_index_reversed(ListPageKey::Artists);
        // Musicindex ADR 0053 R-LIST-9: the node list of the sort Newest
        // sends `sort=date`, and the command sends no `from` with it.
        let sort = self.vm.artist_node_sort();
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
            FetchArtistNamePage::new(endpoint, cursor, from, reverse, sort, group),
            CommandContext::next(),
            cx,
            move |this, batch, cx| {
                if this
                    .artist_pages
                    .page_mut(group)
                    .finish_load(batch, generation)
                {
                    this.continue_artist_group(group, cx);
                    cx.notify();
                }
            },
            move |this, error, cx| {
                if this
                    .artist_pages
                    .page_mut(group)
                    .fail_load(command_error_detail(error), generation)
                {
                    cx.notify();
                }
            },
        );
    }

    /// Loads the next page of `group` when the group shows more rows than it
    /// holds and the node has more pages.
    fn continue_artist_group(&mut self, group: ArtistGroup, cx: &mut Context<Self>) {
        if !self.artist_pages.needs_rows(group) {
            return;
        }
        let intent = self.artist_pages.page_mut(group).begin_load(true);
        self.send_artist_group_request(group, intent, cx);
    }
}
