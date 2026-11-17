//! The status bar of the window (ADR 0101 Decision 7).
//!
//! `StatusBarView` is a small view, like the download status item. Library
//! changes (a tag write, a feed check) redraw only the bar. The top app
//! pushes its own facts into the view at the start of its render.

use gpui::{Context, Entity, IntoElement, Render, Subscription, WeakEntity, Window};

use super::{AppTab, TopApp};
use crate::application::capability::CapabilityAction;
use crate::library::LibraryApp;
use crate::ui::composites::{status_bar, StatusBarSlots};
use crate::view_models::app_events::AppEventList;
use crate::view_models::live_status::LiveStatusDisplay;
use crate::view_models::status_bar::{StatusBarDisplay, StatusBarInput, StatusItemDetail};

/// The facts that the top app owns. The view compares them before it
/// redraws.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct StatusBarTopFacts {
    pub(super) events: AppEventList,
    pub(super) live: Option<LiveStatusDisplay>,
    pub(super) report_count: usize,
}

/// The status bar view.
pub(super) struct StatusBarView {
    top: WeakEntity<TopApp>,
    library: Entity<LibraryApp>,
    facts: StatusBarTopFacts,
    _library_sub: Subscription,
}

impl StatusBarView {
    pub(super) fn new(
        top: WeakEntity<TopApp>,
        library: Entity<LibraryApp>,
        cx: &mut Context<Self>,
    ) -> Self {
        let library_sub = cx.observe(&library, |_, _, cx| cx.notify());
        Self {
            top,
            library,
            facts: StatusBarTopFacts::default(),
            _library_sub: library_sub,
        }
    }

    /// Takes the facts of the top app. A change redraws the bar.
    pub(super) fn set_facts(&mut self, facts: StatusBarTopFacts, cx: &mut Context<Self>) {
        if self.facts != facts {
            self.facts = facts;
            cx.notify();
        }
    }
}

impl Render for StatusBarView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let library = self.library.read(cx);
        let live = self.facts.live.clone().unwrap_or_else(|| {
            LiveStatusDisplay::from_show_page(&crate::view_models::show::ShowPageVm::idle())
        });
        let display = StatusBarDisplay::project(&StatusBarInput {
            events: &self.facts.events,
            live: &live,
            tag_update: library.tag_update_snapshot(),
            feed_checks: library.playlist_rss_snapshot(),
            report_count: self.facts.report_count,
        });
        let top = self.top.clone();
        status_bar(
            display,
            StatusBarSlots::new().on_open_item(move |detail, window, cx| {
                let _ = top.update(cx, |this, cx| this.open_status_item(detail, window, cx));
            }),
        )
    }
}

impl TopApp {
    /// Pushes the facts of the top app into the status bar view.
    pub(super) fn sync_status_bar(&self, cx: &mut Context<Self>) {
        let facts = StatusBarTopFacts {
            events: self.app_events.clone(),
            live: Some(LiveStatusDisplay::from_show_page(&self.show_page)),
            report_count: self.capability_vm.report_count(),
        };
        self.status_bar_view
            .update(cx, |view, cx| view.set_facts(facts, cx));
    }

    /// Opens the detail of a status bar item.
    pub(super) fn open_status_item(
        &mut self,
        detail: StatusItemDetail,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match detail {
            StatusItemDetail::FeedCheck { playlist_id } => {
                self.select_tab(AppTab::Music, window, cx);
                self.open_music_page(
                    &crate::view_models::workspace::FrameNavigationEntry::PlaylistDetail(
                        playlist_id,
                    ),
                    crate::view_models::workspace::NavigationPathStart::NewPath,
                    cx,
                );
            }
            StatusItemDetail::TagUpdatePopup => {
                self.library
                    .update(cx, |library, cx| library.open_tag_update_popup(window, cx));
            }
            StatusItemDetail::ReportsNotice => {
                if self.tab == AppTab::Settings {
                    self.capability_action(CapabilityAction::OpenReport, window, cx);
                } else {
                    self.reports_open = !self.reports_open;
                    cx.notify();
                }
            }
            StatusItemDetail::ShowSection => self.select_tab(AppTab::Show, window, cx),
        }
    }
}
