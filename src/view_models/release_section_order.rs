//! The order and the controls of the release sections of a feed page and of
//! a name level page (ADR 0090 task 004, musicindex ADR 0053 R-LIST-9 row
//! "The releases of an artist, a label or a name", R-LIST-10, R-LIST-11
//! item 3).
//!
//! Each release section has its own sort, view and reverse state, by the
//! text of its heading (musicindex ADR 0043 R-LIST-7 item 6). From
//! `SECTION_CONTROL_MIN_ROWS` rows, the heading of a section has the sort
//! menu, the view icons and the reverse icon. A section with fewer rows has
//! no control, and it keeps its default order and the Rows view, whatever
//! the stored state is. [`order_release_sections`] puts the rows of each
//! section in the shown order, so the render, the selection, the focus order
//! and each row command read one order. This module has no renderer type.

#![warn(clippy::pedantic)]

use crate::view_models::list_page::{
    ListBarControlsVm, ListPageKey, ListPageState, ListSort, ListView,
};
use crate::view_models::name_order;
use crate::view_models::publisher_page::PublisherPageSectionVm;

/// The typed id of a release section: the text of its heading. A command of
/// the heading sends it back, so each section keeps its own state.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ReleaseSectionId(String);

impl ReleaseSectionId {
    /// The id of the section with the heading `heading`.
    #[must_use]
    pub(crate) fn of(heading: &str) -> Self {
        Self(heading.to_owned())
    }

    /// The heading text that names the section.
    #[must_use]
    pub(crate) fn heading(&self) -> &str {
        &self.0
    }
}

/// The controls at the right end of a section heading, with the id of the
/// section that they change.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReleaseSectionControlsVm {
    /// The section that the controls change.
    pub(crate) id: ReleaseSectionId,
    /// The sort menu, the view icons and the reverse icon.
    pub(crate) controls: ListBarControlsVm,
}

/// A command of a section heading. The renderer sends it with the
/// [`ReleaseSectionId`] of the section.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReleaseSectionCommand {
    /// A choice in the sort menu.
    Sort(ListSort),
    /// A click on a view icon.
    View(ListView),
    /// A click on the reverse icon. It carries the state that the click asks
    /// for.
    Reverse(bool),
}

/// Gives each section its controls, its view and the order of its rows.
/// `state_of` gives the stored state of a heading.
///
/// A section offers Newest only when each of its rows has a date, because a
/// sort shows only where its scope can give it (musicindex ADR 0053 R-LIST-9
/// item 1). Then its sorts are Newest and A to Z, and Newest is the default.
/// Else its only sort is A to Z, and a stored Newest reads as A to Z. Newest
/// orders by the date, newest first. A to Z orders by the sort key of the
/// visible title, the same key as the group pages. Reverse reverses the
/// result. A section below `SECTION_CONTROL_MIN_ROWS` rows keeps the source
/// order and the Rows view, with no control.
#[must_use]
pub(crate) fn order_release_sections(
    sections: Vec<PublisherPageSectionVm>,
    state_of: impl Fn(&str) -> ListPageState,
) -> Vec<PublisherPageSectionVm> {
    sections
        .into_iter()
        .map(|section| {
            let state = state_of(&section.heading);
            order_section(section, state)
        })
        .collect()
}

/// Whether each row of `section` has a date, so the section can offer
/// Newest.
fn offers_newest(section: &PublisherPageSectionVm) -> bool {
    section.rows.iter().all(|row| row.date.is_some())
}

fn order_section(
    mut section: PublisherPageSectionVm,
    state: ListPageState,
) -> PublisherPageSectionVm {
    let key = ListPageKey::ReleaseSection;
    let rows = section.rows.len();
    let dated = offers_newest(&section);
    // A stored Newest on a section that cannot give it reads as A to Z.
    let state = ListPageState {
        sort: if dated {
            state.sort
        } else {
            Some(ListSort::AToZ)
        },
        ..state
    };
    let Some(mut controls) = ListBarControlsVm::for_rows(key, state, rows) else {
        section.controls = None;
        section.view = ListView::Rows;
        return section;
    };
    if !dated {
        if let Some(sort) = controls.sort.as_mut() {
            sort.options.retain(|option| option.sort == ListSort::AToZ);
        }
    }
    section.controls = Some(ReleaseSectionControlsVm {
        id: ReleaseSectionId::of(&section.heading),
        controls,
    });
    section.view = state.view;
    if state.sort == Some(ListSort::AToZ) {
        name_order::sort_by_name(&mut section.rows, |row| row.title.text());
    } else {
        section.rows.sort_by_key(|row| std::cmp::Reverse(row.date));
    }
    if state.reversed_for_rows(key, rows) {
        section.rows.reverse();
    }
    section
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_models::detail_selection::{feed_page_release_rows, DetailPageRows};
    use crate::view_models::list_page::{ListPages, SECTION_CONTROL_MIN_ROWS};
    use crate::view_models::list_selection::RowSelectionVm;
    use crate::view_models::publisher_page::{
        PublisherPageAlbumVm, ReleaseRowOpenAction, TitleDisplay,
    };
    use crate::view_models::release_download_state::ReleaseDownloadStateVm;
    use crate::view_models::release_kind::ReleaseKind;
    use crate::view_models::workspace::ReleaseKey;

    fn album(title: &str) -> PublisherPageAlbumVm {
        let guid = format!("guid-{title}");
        PublisherPageAlbumVm {
            feed_guid: Some(guid.clone()),
            title: TitleDisplay::Stated(title.to_owned()),
            image_url: None,
            artist: None,
            not_listed: false,
            in_library: false,
            download_state: ReleaseDownloadStateVm::in_library(false),
            detail_line: None,
            open: Some(ReleaseRowOpenAction {
                key: ReleaseKey::from_index_feed_guid(guid),
                a11y_label: format!("Open the release {title}"),
            }),
            selection: RowSelectionVm::none(),
            kind: ReleaseKind::Album,
            menu: None,
            date: None,
        }
    }

    /// A section of rows with no date, as an Index feed page gives.
    fn section(heading: &str, titles: &[&str]) -> PublisherPageSectionVm {
        PublisherPageSectionVm {
            heading: heading.to_owned(),
            rows: titles.iter().map(|title| album(title)).collect(),
            note: None,
            status: None,
            controls: None,
            view: ListView::Rows,
        }
    }

    /// The dates of the rows of [`EIGHT`], in the source order. They are
    /// distinct, and they do not follow the source order.
    const DATES: [i64; 8] = [3, 8, 1, 6, 2, 7, 4, 5];

    /// A section whose rows have a date, as a Library feed page and a name
    /// level page give. The source order is [`EIGHT`], which is not the
    /// order of the dates.
    fn dated_section(heading: &str, titles: &[&str]) -> PublisherPageSectionVm {
        let mut section = section(heading, titles);
        for (row, date) in section.rows.iter_mut().zip(DATES) {
            row.date = Some(date);
        }
        section
    }

    /// The titles of [`EIGHT`] by date, newest first.
    fn newest_first() -> Vec<&'static str> {
        let mut dated: Vec<(i64, &str)> = DATES.into_iter().zip(EIGHT).collect();
        dated.sort_by_key(|(date, _)| std::cmp::Reverse(*date));
        dated.into_iter().map(|(_, title)| title).collect()
    }

    /// Eight titles in the source order
    const EIGHT: [&str; 8] = [
        "delta", "Bravo", "echo", "Alpha", "golf", "Charlie", "hotel", "foxtrot",
    ];

    fn titles(section: &PublisherPageSectionVm) -> Vec<&str> {
        section.rows.iter().map(|row| row.title.text()).collect()
    }

    fn ordered(
        pages: &ListPages,
        sections: Vec<PublisherPageSectionVm>,
    ) -> Vec<PublisherPageSectionVm> {
        order_release_sections(sections, |heading| pages.section_state(heading))
    }

    /// Musicindex ADR 0053 R-LIST-9: a section with 8 rows has the heading
    /// controls in the sequence of R-LIST-6. A section with 7 rows has none,
    /// and a stored reverse and a stored Tiles view have no effect on it.
    #[test]
    fn adr_0053_r9_section_controls_start_at_eight_rows() {
        let mut pages = ListPages::default();
        pages.set_section_reversed("Albums", true);
        pages.set_section_view("Albums", ListView::Tiles);
        assert_eq!(SECTION_CONTROL_MIN_ROWS, 8);

        let seven = &EIGHT[..7];
        let result = ordered(&pages, vec![dated_section("Albums", seven)]);
        assert!(result[0].controls.is_none(), "7 rows have no control");
        assert_eq!(result[0].view, ListView::Rows, "7 rows show rows");
        assert_eq!(titles(&result[0]), seven, "7 rows keep the source order");

        let result = ordered(&pages, vec![dated_section("Albums", &EIGHT)]);
        let controls = result[0].controls.as_ref().expect("8 rows have controls");
        assert_eq!(controls.id.heading(), "Albums");
        let parts = &controls.controls;
        let sort = parts.sort.as_ref().expect("a sort menu");
        let labels: Vec<_> = sort.options.iter().map(|option| option.sort).collect();
        assert_eq!(labels, [ListSort::Newest, ListSort::AToZ]);
        assert_eq!(sort.label, "Oldest", "a reversed Newest reads Oldest");
        let views: Vec<_> = parts
            .view
            .as_ref()
            .expect("view icons")
            .options
            .iter()
            .map(|option| option.view)
            .collect();
        assert_eq!(views, [ListView::Rows, ListView::Tiles]);
        assert!(parts.reverse.as_ref().expect("a reverse icon").pressed);
        assert_eq!(result[0].view, ListView::Tiles);
    }

    /// Musicindex ADR 0043 R-LIST-7 item 6: each heading keeps its own state.
    /// "Albums" reversed does not reverse "Singles".
    #[test]
    fn adr_0053_r9_section_each_heading_keeps_its_own_state() {
        let mut pages = ListPages::default();
        pages.set_section_reversed("Albums", true);
        pages.set_section_sort("Singles", ListSort::AToZ);
        pages.set_section_view("Singles", ListView::Tiles);
        let result = ordered(
            &pages,
            vec![
                dated_section("Albums", &EIGHT),
                dated_section("Singles", &EIGHT),
            ],
        );
        let mut reversed = newest_first();
        reversed.reverse();
        assert_eq!(titles(&result[0]), reversed);
        assert_eq!(result[0].view, ListView::Rows);
        let mut a_to_z: Vec<&str> = EIGHT.to_vec();
        name_order::sort_by_name(&mut a_to_z, |title| *title);
        assert_eq!(titles(&result[1]), a_to_z);
        assert_eq!(result[1].view, ListView::Tiles);
        assert_eq!(
            pages.section_state("Other"),
            ListPageState::default_for(ListPageKey::ReleaseSection),
            "a heading with no change has the defaults"
        );
    }

    /// Musicindex ADR 0053 R-LIST-9: Newest, A to Z, Oldest and Z to A.
    #[test]
    fn adr_0053_r9_section_orders_newest_a_to_z_oldest_and_z_to_a() {
        let source = newest_first();
        let mut a_to_z = EIGHT.to_vec();
        name_order::sort_by_name(&mut a_to_z, |title| *title);
        let reverse = |items: &[&str]| -> Vec<String> {
            items
                .iter()
                .rev()
                .map(|title| (*title).to_owned())
                .collect()
        };
        for (sort, reversed, expected) in [
            (
                ListSort::Newest,
                false,
                source.iter().map(|t| (*t).to_owned()).collect::<Vec<_>>(),
            ),
            (
                ListSort::AToZ,
                false,
                a_to_z.iter().map(|t| (*t).to_owned()).collect(),
            ),
            (ListSort::Newest, true, reverse(&source)),
            (ListSort::AToZ, true, reverse(&a_to_z)),
        ] {
            let mut pages = ListPages::default();
            pages.set_section_sort("Albums", sort);
            pages.set_section_reversed("Albums", reversed);
            let result = ordered(&pages, vec![dated_section("Albums", &EIGHT)]);
            let shown: Vec<String> = titles(&result[0]).into_iter().map(str::to_owned).collect();
            assert_eq!(shown, expected, "{sort:?} reversed={reversed}");
            let label = result[0]
                .controls
                .as_ref()
                .and_then(|controls| controls.controls.sort.as_ref())
                .map(|sort| sort.label);
            assert_eq!(
                label,
                Some(sort.name(reversed)),
                "the sort name follows the order"
            );
        }
    }

    /// The selectable rows follow the shown order across two sections.
    #[test]
    fn adr_0053_r9_section_selectable_rows_follow_the_shown_order() {
        let mut pages = ListPages::default();
        pages.set_section_sort("Albums", ListSort::AToZ);
        pages.set_section_reversed("Singles", true);
        let result = ordered(
            &pages,
            vec![
                dated_section("Albums", &EIGHT),
                dated_section("Singles", &EIGHT),
            ],
        );
        let shown: Vec<ReleaseKey> = result
            .iter()
            .flat_map(|section| section.rows.iter())
            .filter_map(|row| row.open.as_ref().map(|open| open.key.clone()))
            .collect();
        let rows = DetailPageRows::Releases(feed_page_release_rows(&result, |_| None));
        let selectable: Vec<ReleaseKey> = rows
            .members()
            .into_iter()
            .filter_map(|key| match key {
                crate::view_models::detail_selection::DetailRowKey::Release(key) => Some(key),
                _ => None,
            })
            .collect();
        assert_eq!(selectable.len(), 16);
        assert_eq!(selectable, shown);
        assert_eq!(
            result[0].rows.first().map(|row| row.title.text()),
            Some("Alpha")
        );
        assert_eq!(
            result[1].rows.first().map(|row| row.title.text()),
            Some(newest_first().last().copied().expect("rows"))
        );
    }

    /// Musicindex ADR 0053 R-LIST-9 item 1: an Index feed section has no date,
    /// so it offers A to Z only, and it defaults to it. A stored Newest reads
    /// as A to Z. The reverse and the views stay.
    #[test]
    fn adr_0053_r9_section_without_dates_offers_a_to_z_only() {
        let mut pages = ListPages::default();
        let result = ordered(&pages, vec![section("Albums", &EIGHT)]);
        let controls = result[0].controls.as_ref().expect("8 rows have controls");
        let sort = controls.controls.sort.as_ref().expect("a sort menu");
        let offered: Vec<_> = sort.options.iter().map(|option| option.sort).collect();
        assert_eq!(offered, [ListSort::AToZ]);
        assert_eq!(sort.label, "A to Z");
        assert!(sort.options[0].selected);
        assert!(controls.controls.reverse.is_some());
        assert_eq!(
            controls
                .controls
                .view
                .as_ref()
                .expect("views")
                .options
                .len(),
            2
        );
        let mut a_to_z = EIGHT.to_vec();
        name_order::sort_by_name(&mut a_to_z, |title| *title);
        assert_eq!(titles(&result[0]), a_to_z, "the default order is A to Z");

        // The state is Newest by default, and an explicit Newest changes
        // nothing. Reverse reads Z to A.
        pages.set_section_sort("Albums", ListSort::Newest);
        pages.set_section_reversed("Albums", true);
        let result = ordered(&pages, vec![section("Albums", &EIGHT)]);
        let sort = result[0]
            .controls
            .as_ref()
            .and_then(|controls| controls.controls.sort.as_ref())
            .expect("a sort menu");
        assert_eq!(sort.label, "Z to A");
        a_to_z.reverse();
        assert_eq!(titles(&result[0]), a_to_z);

        // Below 8 rows the order of the source stays.
        let seven = &EIGHT[..7];
        let result = ordered(&pages, vec![section("Albums", seven)]);
        assert!(result[0].controls.is_none());
        assert_eq!(titles(&result[0]), seven);
    }

    /// Musicindex ADR 0053 R-LIST-9 item 1: a Library feed section with a
    /// date in each row offers Newest and A to Z, defaults to Newest, and
    /// sorts by the date. One row with no date removes Newest.
    #[test]
    fn adr_0053_r9_section_with_dates_offers_newest_and_sorts_by_date() {
        let pages = ListPages::default();
        let result = ordered(&pages, vec![dated_section("Albums", &EIGHT)]);
        let sort = result[0]
            .controls
            .as_ref()
            .and_then(|controls| controls.controls.sort.as_ref())
            .expect("a sort menu");
        let offered: Vec<_> = sort.options.iter().map(|option| option.sort).collect();
        assert_eq!(offered, [ListSort::Newest, ListSort::AToZ]);
        assert_eq!(sort.label, "Newest");
        assert_eq!(titles(&result[0]), newest_first());

        let mut partly = dated_section("Albums", &EIGHT);
        partly.rows[3].date = None;
        let result = ordered(&pages, vec![partly]);
        let sort = result[0]
            .controls
            .as_ref()
            .and_then(|controls| controls.controls.sort.as_ref())
            .expect("a sort menu");
        assert_eq!(sort.options.len(), 1, "a row with no date removes Newest");
        assert_eq!(sort.label, "A to Z");
    }

    /// Musicindex ADR 0053 R-LIST-9 item 1: a name level section has a date in
    /// each row, from the date that the page already sorts by. It offers
    /// Newest.
    #[test]
    fn adr_0053_r9_section_name_level_rows_have_dates_and_offer_newest() {
        use crate::view_models::artist_identity::ArtistNameKey;
        use crate::view_models::name_level_page::{
            NameLevelIndexState, NameLevelLibraryRelease, NameLevelPageFacts, NameLevelPageVm,
        };
        let library = (0..8_i64)
            .map(|id| NameLevelLibraryRelease {
                release_key: Some(ReleaseKey::LibraryFeed(id + 1)),
                feed_guid: Some(format!("guid-{id}")),
                title: format!("Album {id}"),
                image_url: None,
                artist_text: "Haleen".to_owned(),
                kind: ReleaseKind::Album,
                newest_date: Some(DATES[usize::try_from(id).expect("index")]),
                in_library: true,
            })
            .collect();
        let vm = NameLevelPageVm::new(NameLevelPageFacts {
            key: ArtistNameKey::from_text("Haleen"),
            library,
            index: NameLevelIndexState::Loaded {
                releases: Vec::new(),
                more_available: false,
            },
            other_identity_feed_guids: std::collections::BTreeSet::new(),
            matched_feed: None,
        });
        let result = ordered(&ListPages::default(), vm.sections());
        assert_eq!(result.len(), 1);
        assert!(result[0].rows.iter().all(|row| row.date.is_some()));
        let sort = result[0]
            .controls
            .as_ref()
            .and_then(|controls| controls.controls.sort.as_ref())
            .expect("a sort menu");
        assert_eq!(sort.options[0].sort, ListSort::Newest);
        let dates: Vec<i64> = result[0].rows.iter().filter_map(|row| row.date).collect();
        assert!(dates.windows(2).all(|pair| pair[0] >= pair[1]));
    }

    /// Musicindex ADR 0053 R-LIST-12: the A to Z of a section reads a number
    /// as a number and ignores case.
    #[test]
    fn adr_0053_r12_section_a_to_z_reads_numbers_and_ignores_case() {
        let mut pages = ListPages::default();
        pages.set_section_sort("Albums", ListSort::AToZ);
        let result = ordered(
            &pages,
            vec![section(
                "Albums",
                &[
                    "Track 10", "track 2", "Track 1", "echo", "Echo", "golf", "hotel", "alpha",
                ],
            )],
        );
        let order = titles(&result[0]);
        assert_eq!(&order[..3], ["alpha", "echo", "Echo"]);
        assert_eq!(
            &order[3..],
            ["golf", "hotel", "Track 1", "track 2", "Track 10"]
        );
    }
}
