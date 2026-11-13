//! ADR 0097 phase 1: the fit check of the shell at the two widths of
//! musicindex ADR 0065 R-FIT-2.
//!
//! The test mounts `TopApp` with an in-memory database and no runtime host.
//! It draws the start page, then the Settings page, at each width. It also
//! mounts the list bar at the content width of each window width. The fit
//! probes of the shared primitives give the faults (ADR 0097 Decision 4).

use std::{
    cell::RefCell,
    path::PathBuf,
    rc::Rc,
    sync::{Arc, Mutex},
};

use gpui::{
    div, size, AppContext, Context, Entity, IntoElement, ParentElement, Pixels, Render, Styled,
    Window,
};

use super::TopApp;
use crate::theme_profile::ThemeProfile;
use crate::ui::composites::selection_action_bar::{
    list_bar_playlists, render_list_bar, ListBarCopy, ListBarHandler,
};
use crate::ui::layouts::{
    MUSIC_SIDEBAR_WIDTH, SHELL_TAB_BAR_BREAKPOINT, WINDOW_HEIGHT, WINDOW_MIN_WIDTH,
};
use crate::ui::primitives::fit_probe;
use crate::view_models::detail_selection::{
    singles_group_release_rows, DetailPageRows, DetailRowKey, DetailSelection, LibraryReleaseFacts,
    ListBarVm, ReleaseDownloaded,
};
use crate::view_models::list_page::{ListPageKey, ListPageState};
use crate::view_models::sidebar::SidebarDestination;
use crate::view_models::singles_group::{SinglesGroupPageEntryVm, SinglesGroupPageRowVm};
use crate::view_models::workspace::ReleaseKey;

/// The two widths of the check (ADR 0097 Decision 2).
const WIDTHS: [Pixels; 2] = [WINDOW_MIN_WIDTH, SHELL_TAB_BAR_BREAKPOINT];

/// A title of 40 characters with no space, so that a fault cannot hide
/// behind a short name (ADR 0097 phase plan, Risks).
const LONG_TITLE: &str = "Supercalifragilisticexpialidociousness40";

fn init(cx: &mut gpui::App) {
    gpui_component::init(cx);
    crate::ui::primitives::primary_selection::init(cx);
    crate::ui::primitives::context_menu::init(cx);
    crate::ui::focus_ring::init(cx);
    crate::ui::theme_bridge::install_theme(
        ThemeProfile::default(),
        crate::config::UiScale::Medium.into(),
        cx,
    );
}

fn top_app(window: &mut Window, cx: &mut Context<TopApp>) -> TopApp {
    let conn = rusqlite::Connection::open_in_memory().expect("an in-memory database");
    crate::db::init_schema(&conn).expect("the schema");
    crate::db::migrate_schema(&conn).expect("the migrations");
    let dir = std::env::temp_dir().join("hitit-fit-check");
    let not_set = |field: &'static str| crate::config::ConfigFieldIssue {
        field,
        kind: crate::config::ConfigIssueKind::Missing,
        explanation: "The fit check sets no broadcast host.",
    };
    let broadcast = crate::config::BroadcastCapabilities {
        hosts: Ok(Vec::new()),
        selected_host: Ok(None),
        drop_directory: Ok(None),
        drop_file_target: Err(not_set("broadcast.drop_file_target")),
        encoder: Ok(None),
    };
    TopApp::new(
        Arc::new(Mutex::new(conn)),
        crate::media::ImageCache::new(dir.join("cache")),
        dir.join("config.toml"),
        "http://127.0.0.1:9".into(),
        PathBuf::from(&dir).join("music"),
        Ok(None),
        broadcast,
        None,
        None,
        crate::config::UiScale::Medium,
        ThemeProfile::default(),
        None,
        None,
        crate::application::session_lifecycle::SessionLifecycle::new(),
        window,
        cx,
    )
}

/// The probe count and the faults of the shell at `width`. `open` is the
/// sidebar destination that the test opens before the draw, if any.
fn shell_faults(width: Pixels, open: Option<SidebarDestination>) -> (usize, Vec<String>) {
    let mut cx = fit_probe::figtree_test_context();
    cx.update(init);
    let app: Rc<RefCell<Option<Entity<TopApp>>>> = Rc::default();
    let slot = Rc::clone(&app);
    let (_, window) = cx.add_window_view(move |window, cx| {
        let entity = cx.new(|cx| top_app(window, cx));
        *slot.borrow_mut() = Some(entity.clone());
        gpui_component::Root::new(entity, window, cx)
    });
    window.simulate_resize(size(width, WINDOW_HEIGHT));
    window.run_until_parked();
    if let Some(row) = open {
        let entity = app.borrow().clone().expect("the window made TopApp");
        window.update(|window, cx| {
            entity.update(cx, |app, cx| app.open_sidebar_row(row, window, cx));
        });
        window.run_until_parked();
    }
    let found = draw_and_read(window);
    cx.quit();
    found
}

fn draw_and_read(window: &mut gpui::VisualTestContext) -> (usize, Vec<String>) {
    fit_probe::clear();
    window.update(|window, cx| {
        let _ = window.draw(cx);
    });
    (fit_probe::count(), fit_probe::faults())
}

fn single_row(id: i64) -> SinglesGroupPageEntryVm {
    let guid = format!("fit-check-single-{id}");
    SinglesGroupPageEntryVm::Library(Box::new(SinglesGroupPageRowVm {
        track: crate::db::TrackRow {
            id,
            feed_id: id,
            feed_guid: Some(guid.clone()),
            item_guid: guid.clone(),
            track_title: Some(LONG_TITLE.to_owned()),
            is_in_library: true,
            ..crate::db::TrackRow::default()
        },
        date_text: None,
        work: crate::view_models::library::TrackRowWork::Idle,
        key: ReleaseKey::FeedGuid(guid),
    }))
}

/// The list bar of a singles group page with its controls. In `Selecting`
/// one of two rows is selected, so the bar shows each selection action.
fn list_bar(selecting: bool) -> ListBarVm {
    let rows = DetailPageRows::Singles(singles_group_release_rows(
        &[single_row(5), single_row(6)],
        |key| {
            Some(LibraryReleaseFacts {
                feed_id: match key {
                    ReleaseKey::FeedGuid(guid) if guid.ends_with('5') => 5,
                    _ => 6,
                },
                downloaded: ReleaseDownloaded::EachTrack,
                download_active: false,
                removing: false,
            })
        },
    ));
    let mut selection = DetailSelection::new();
    if selecting {
        selection.toggle_mode();
        selection.toggle_row(&DetailRowKey::Release(ReleaseKey::FeedGuid(
            "fit-check-single-5".to_owned(),
        )));
    }
    let key = ListPageKey::SinglesGroup;
    ListBarVm::new(&rows, &selection)
        .expect("a singles group page has a list bar")
        .with_list_controls(key, ListPageState::default_for(key))
}

/// The list bar of a group page in a box as wide as the content column.
struct ListBarCase {
    content_width: Pixels,
    selecting: bool,
}

impl Render for ListBarCase {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let handler: ListBarHandler = Rc::new(|_, _, _| {});
        let bar = render_list_bar(
            list_bar(self.selecting),
            &list_bar_playlists(Vec::new()),
            &handler,
            ListBarCopy::InFlow,
            cx,
        );
        div()
            .size_full()
            .font_family(crate::ui::fonts::FIGTREE_FAMILY)
            .child(div().w(self.content_width).overflow_hidden().child(bar))
    }
}

/// The content width of the window width `width`: the full width below the
/// breakpoint, and the width beside the sidebar at the breakpoint and above.
fn content_width(width: Pixels) -> Pixels {
    if width < SHELL_TAB_BAR_BREAKPOINT {
        width
    } else {
        width - MUSIC_SIDEBAR_WIDTH
    }
}

fn list_bar_faults(width: Pixels, selecting: bool) -> (usize, Vec<String>) {
    let mut cx = fit_probe::figtree_test_context();
    cx.update(init);
    let content_width = content_width(width);
    let (_, window) = cx.add_window_view(move |_, _| ListBarCase {
        content_width,
        selecting,
    });
    window.simulate_resize(size(width, WINDOW_HEIGHT));
    window.run_until_parked();
    let found = draw_and_read(window);
    cx.quit();
    found
}

/// ADR 0097 §How This Decision Is Checked: the shell has no break inside a
/// word, no edge cut and no overflow at 390 px and at 940 px.
#[test]
fn adr_0097_the_shell_fits_at_the_two_widths() {
    for width in WIDTHS {
        for open in [None, Some(SidebarDestination::Settings)] {
            let (count, faults) = shell_faults(width, open);
            assert!(
                count > 0,
                "ADR 0097 Decision 4: at {} px the shell ({open:?}) recorded no fit probe. \
                 Make sure that Label and Button add fit_probe::probe in a test binary.",
                f32::from(width)
            );
            assert!(faults.is_empty(), "{open:?}:\n{}", faults.join("\n"));
        }
    }
}

/// ADR 0097 §How This Decision Is Checked: the list bar fits the content
/// width of each window width, in `Normal` and in `Selecting`.
#[test]
fn adr_0097_the_list_bar_fits_at_the_two_widths() {
    for width in WIDTHS {
        for selecting in [false, true] {
            let (count, faults) = list_bar_faults(width, selecting);
            assert!(
                count > 0,
                "ADR 0097 Decision 4: the list bar recorded no fit probe."
            );
            assert!(
                faults.is_empty(),
                "selecting = {selecting}:\n{}",
                faults.join("\n")
            );
        }
    }
}
