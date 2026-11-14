//! ADR 0097 task 002: fit checks for the Library list destinations.

use super::{draw_and_read, init, top_app, LONG_TITLE, WIDTHS};
use crate::library::fit_fixtures;
use crate::ui::layouts::WINDOW_HEIGHT;
use crate::ui::primitives::fit_probe;
use crate::view_models::list_page::{ListScope, ListView};
use crate::view_models::sidebar::SidebarDestination;
use gpui::{size, AppContext, Entity};
use std::{cell::RefCell, rc::Rc};

fn page_faults(
    width: gpui::Pixels,
    destination: SidebarDestination,
    scope: Option<ListScope>,
    view: Option<ListView>,
    search_results: bool,
) -> (usize, Vec<String>, Vec<String>) {
    let mut cx = fit_probe::figtree_test_context();
    cx.update(init);
    let app_slot: Rc<RefCell<Option<Entity<super::TopApp>>>> = Rc::default();
    let slot = Rc::clone(&app_slot);
    let (_, window) = cx.add_window_view(move |window, cx| {
        let entity = cx.new(|cx| top_app(window, cx));
        *slot.borrow_mut() = Some(entity.clone());
        gpui_component::Root::new(entity, window, cx)
    });
    window.simulate_resize(size(width, WINDOW_HEIGHT));
    window.run_until_parked();

    let entity = app_slot.borrow().clone().expect("the window made TopApp");
    window.update(|window, cx| {
        entity.update(cx, |app, cx| {
            app.open_sidebar_row(destination, window, cx);
            if let Some(destination) = destination.music_destination() {
                app.library.update(cx, |library, _| {
                    library.load_fit_fixtures();
                    library.configure_fit_list(destination, scope, view);
                });
            }
        });
    });
    // The destination event is queued. Let it open its page before the
    // results page opens, or it replaces the results page.
    window.run_until_parked();
    if search_results {
        window.update(|window, cx| {
            entity.update(cx, |app, cx| {
                app.open_search_results_in_content_list(LONG_TITLE, window, cx);
                if let Some(page) = &mut app.search_results_detail {
                    page.replace_index_results(fit_fixtures::index_search_results());
                }
            });
        });
        window.run_until_parked();
    }
    let result = draw_and_read(window);
    cx.quit();
    let names = fit_probe::recorded_names();
    (result.0, result.1, names)
}

#[expect(
    clippy::too_many_arguments,
    reason = "each argument gives one part of a page case"
)]
fn check_case(
    width: gpui::Pixels,
    page: &str,
    scope: &str,
    view: &str,
    destination: SidebarDestination,
    list_scope: Option<ListScope>,
    list_view: Option<ListView>,
    search_results: bool,
) -> Vec<String> {
    let (count, faults, names) =
        page_faults(width, destination, list_scope, list_view, search_results);
    let context = format!(
        "{page}, scope {scope}, view {view}, {} px",
        f32::from(width)
    );
    let mut failures = Vec::new();
    if count == 0 {
        failures.push(format!("{context}: no fit probe was recorded"));
    }
    if !names.iter().any(|name| name.contains(LONG_TITLE)) {
        failures.push(format!(
            "{context}: no probe recorded fixture text {LONG_TITLE}"
        ));
    }
    failures.extend(
        faults
            .into_iter()
            .map(|fault| format!("{context}:\n{fault}")),
    );
    failures
}

/// ADR 0097 Decision 2: the test checks each list page at 390 px and 940 px.
#[test]
fn adr_0097_the_list_pages_fit_at_the_two_widths() {
    let mut failures = Vec::new();
    for width in WIDTHS {
        for (page, destination) in [
            ("Releases", SidebarDestination::Releases),
            ("Artists", SidebarDestination::Artists),
        ] {
            for scope in [ListScope::InLibrary, ListScope::All] {
                for view in [ListView::Rows, ListView::Tiles] {
                    failures.extend(check_case(
                        width,
                        page,
                        match scope {
                            ListScope::InLibrary => "Library",
                            ListScope::All => "All",
                        },
                        view.label(),
                        destination,
                        Some(scope),
                        Some(view),
                        false,
                    ));
                }
            }
        }
        for (page, destination) in [
            ("My playlists", SidebarDestination::MyPlaylists),
            ("Public playlists", SidebarDestination::PublicPlaylists),
            ("Live", SidebarDestination::Live),
            ("Inbox", SidebarDestination::Inbox),
            ("Search saved searches", SidebarDestination::Search),
        ] {
            failures.extend(check_case(
                width,
                page,
                "default",
                "default",
                destination,
                None,
                None,
                false,
            ));
        }
        failures.extend(check_case(
            width,
            "Search results",
            "All",
            "rows",
            SidebarDestination::Search,
            None,
            None,
            true,
        ));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
