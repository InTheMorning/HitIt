//! Sidebar rows (ADR 0089 Decision 1).
//!
//! The sidebar is one list with no group labels. It holds the upper
//! destinations, a separator, then the lower rows: Show and Settings. This
//! module owns the rows, their labels, counts and selected state. It carries
//! no renderer type. The renderer maps each semantic icon to a named icon.

#![warn(clippy::pedantic)]

use super::workspace::MusicDestination;

/// One sidebar row (ADR 0089 Decision 1). A Music destination is one of the
/// rows. Show and Settings are screen mounts, so they are not
/// `MusicDestination` values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SidebarDestination {
    Search,
    Releases,
    Artists,
    MyPlaylists,
    NeedsAttention,
    Show,
    Settings,
}

impl SidebarDestination {
    /// Every row, in sidebar order.
    pub(crate) const ALL: [Self; 7] = [
        Self::Search,
        Self::Releases,
        Self::Artists,
        Self::MyPlaylists,
        Self::NeedsAttention,
        Self::Show,
        Self::Settings,
    ];

    /// The Music destination that this row opens, or `None` for Show and
    /// Settings.
    #[must_use]
    pub(crate) const fn music_destination(self) -> Option<MusicDestination> {
        match self {
            Self::Search => Some(MusicDestination::Search),
            Self::Releases => Some(MusicDestination::Releases),
            Self::Artists => Some(MusicDestination::Artists),
            Self::MyPlaylists => Some(MusicDestination::Playlists),
            Self::NeedsAttention => Some(MusicDestination::NeedsAttention),
            Self::Show | Self::Settings => None,
        }
    }

    /// The row of a Music destination.
    #[must_use]
    pub(crate) const fn for_music(destination: MusicDestination) -> Self {
        match destination {
            MusicDestination::Search => Self::Search,
            MusicDestination::Releases => Self::Releases,
            MusicDestination::Artists => Self::Artists,
            MusicDestination::Playlists => Self::MyPlaylists,
            MusicDestination::NeedsAttention => Self::NeedsAttention,
        }
    }
}

/// The semantic icon of a sidebar row (ADR 0084 Decision 1, ADR 0089
/// Decision 1). The renderer maps each name to a named `IconName`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SidebarIcon {
    Search,
    Release,
    Artist,
    Playlist,
    Warning,
    Show,
    Settings,
}

/// Display data for one sidebar row (ADR 0089 Decision 1).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SidebarRowVm {
    /// The row's destination.
    pub(crate) destination: SidebarDestination,
    /// Stable element id.
    pub(crate) element_id: &'static str,
    /// The row's icon.
    pub(crate) icon: SidebarIcon,
    /// The visible row name.
    pub(crate) label: &'static str,
    /// The tooltip text, with the count when the row has one.
    pub(crate) tooltip: String,
    /// The accessibility label, with the count when the row has one.
    pub(crate) a11y_label: String,
    /// The row's count, when it has one (Needs attention and My playlists).
    pub(crate) count: Option<usize>,
    /// Whether the row is the destination of the current screen.
    pub(crate) selected: bool,
    /// Whether the icon shows an attention mark. The tooltip and the
    /// accessibility label give the count in text too.
    pub(crate) shows_attention_mark: bool,
    /// The row opens its destination while a dragged track stays over it
    /// (ADR 0083 Decision 6, task 008).
    pub(crate) spring_loaded: bool,
}

/// One entry of the sidebar: a row, or the line that follows the upper rows.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SidebarEntryVm {
    Row(SidebarRowVm),
    Separator,
}

/// The counts that the sidebar rows show. The Library stores them from its
/// own state, so the render path runs no query.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct SidebarCounts {
    /// Tracks that need attention.
    pub(crate) needs_attention: usize,
    /// Playlists of the curator.
    pub(crate) playlists: usize,
}

/// The sidebar view model (ADR 0089 Decision 1).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SidebarVm {
    entries: Vec<SidebarEntryVm>,
}

impl SidebarVm {
    /// Gives the sidebar entries in order. `selected` is the row of the
    /// current screen, or `None` when no row is selected.
    #[must_use]
    pub(crate) fn new(counts: SidebarCounts, selected: Option<SidebarDestination>) -> Self {
        let upper = [
            SidebarDestination::Search,
            SidebarDestination::Releases,
            SidebarDestination::Artists,
            SidebarDestination::MyPlaylists,
            SidebarDestination::NeedsAttention,
        ];
        let lower = [SidebarDestination::Show, SidebarDestination::Settings];
        let mut entries: Vec<SidebarEntryVm> = upper
            .into_iter()
            .map(|destination| SidebarEntryVm::Row(row(destination, counts, selected)))
            .collect();
        entries.push(SidebarEntryVm::Separator);
        entries.extend(
            lower
                .into_iter()
                .map(|destination| SidebarEntryVm::Row(row(destination, counts, selected))),
        );
        Self { entries }
    }

    /// The entries in sidebar order.
    #[must_use]
    pub(crate) fn entries(&self) -> &[SidebarEntryVm] {
        &self.entries
    }
}

fn row(
    destination: SidebarDestination,
    counts: SidebarCounts,
    selected: Option<SidebarDestination>,
) -> SidebarRowVm {
    // A count of zero shows nothing. Each count names its own items: tracks
    // for Needs attention, playlists for My playlists.
    let count = match destination {
        SidebarDestination::NeedsAttention => Some(counts.needs_attention),
        SidebarDestination::MyPlaylists => Some(counts.playlists),
        SidebarDestination::Search
        | SidebarDestination::Releases
        | SidebarDestination::Artists
        | SidebarDestination::Show
        | SidebarDestination::Settings => None,
    }
    .filter(|count| *count > 0);
    let label = label(destination);
    let noun = match destination {
        SidebarDestination::MyPlaylists => "playlist",
        _ => "track",
    };
    let (tooltip, a11y_label) = match count {
        Some(count) => {
            let text = format!("{label}, {count} {noun}{}", plural(count));
            (text.clone(), text)
        }
        None => (label.to_owned(), label.to_owned()),
    };
    SidebarRowVm {
        destination,
        element_id: element_id(destination),
        icon: icon(destination),
        label,
        tooltip,
        a11y_label,
        count,
        selected: selected == Some(destination),
        shows_attention_mark: matches!(destination, SidebarDestination::NeedsAttention)
            && count.is_some(),
        spring_loaded: matches!(destination, SidebarDestination::MyPlaylists),
    }
}

const fn label(destination: SidebarDestination) -> &'static str {
    match destination {
        SidebarDestination::Search => "Search",
        SidebarDestination::Releases => "Releases",
        SidebarDestination::Artists => "Artists",
        SidebarDestination::MyPlaylists => "My playlists",
        SidebarDestination::NeedsAttention => "Needs attention",
        SidebarDestination::Show => "Show",
        SidebarDestination::Settings => "Settings",
    }
}

const fn element_id(destination: SidebarDestination) -> &'static str {
    match destination {
        SidebarDestination::Search => "music-destination-search",
        SidebarDestination::Releases => "music-destination-releases",
        SidebarDestination::Artists => "music-destination-artists",
        SidebarDestination::MyPlaylists => "music-destination-playlists",
        SidebarDestination::NeedsAttention => "music-destination-needs-attention",
        SidebarDestination::Show => "sidebar-show",
        SidebarDestination::Settings => "sidebar-settings",
    }
}

const fn icon(destination: SidebarDestination) -> SidebarIcon {
    match destination {
        SidebarDestination::Search => SidebarIcon::Search,
        SidebarDestination::Releases => SidebarIcon::Release,
        SidebarDestination::Artists => SidebarIcon::Artist,
        SidebarDestination::MyPlaylists => SidebarIcon::Playlist,
        SidebarDestination::NeedsAttention => SidebarIcon::Warning,
        SidebarDestination::Show => SidebarIcon::Show,
        SidebarDestination::Settings => SidebarIcon::Settings,
    }
}

const fn plural(count: usize) -> &'static str {
    if count == 1 {
        ""
    } else {
        "s"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(vm: &SidebarVm) -> impl Iterator<Item = &SidebarRowVm> {
        vm.entries().iter().filter_map(|entry| match entry {
            SidebarEntryVm::Row(row) => Some(row),
            SidebarEntryVm::Separator => None,
        })
    }

    fn labels(vm: &SidebarVm) -> Vec<String> {
        vm.entries()
            .iter()
            .map(|entry| match entry {
                SidebarEntryVm::Row(row) => row.label.to_owned(),
                SidebarEntryVm::Separator => "---".to_owned(),
            })
            .collect()
    }

    #[test]
    fn sidebar_lists_rows_in_adr_0089_sequence_with_separator_before_show() {
        let vm = SidebarVm::new(SidebarCounts::default(), None);
        assert_eq!(
            labels(&vm),
            [
                "Search",
                "Releases",
                "Artists",
                "My playlists",
                "Needs attention",
                "---",
                "Show",
                "Settings",
            ]
        );
    }

    #[test]
    fn sidebar_selected_row_follows_the_given_destination() {
        let vm = SidebarVm::new(SidebarCounts::default(), Some(SidebarDestination::Show));
        let selected: Vec<_> = rows(&vm)
            .filter(|row| row.selected)
            .map(|row| row.destination)
            .collect();
        assert_eq!(selected, [SidebarDestination::Show]);
    }

    #[test]
    fn sidebar_counts_show_only_when_positive_and_name_their_items() {
        let counts = SidebarCounts {
            needs_attention: 2,
            playlists: 0,
        };
        let vm = SidebarVm::new(counts, None);
        let attention = rows(&vm)
            .find(|row| row.destination == SidebarDestination::NeedsAttention)
            .expect("Needs attention row");
        assert_eq!(attention.count, Some(2));
        assert!(attention.shows_attention_mark);
        assert_eq!(attention.a11y_label, "Needs attention, 2 tracks");
        let playlists = rows(&vm)
            .find(|row| row.destination == SidebarDestination::MyPlaylists)
            .expect("My playlists row");
        assert_eq!(playlists.count, None);
    }

    #[test]
    fn sidebar_destination_round_trips_music_destinations() {
        for destination in [
            MusicDestination::Search,
            MusicDestination::Releases,
            MusicDestination::Artists,
            MusicDestination::Playlists,
            MusicDestination::NeedsAttention,
        ] {
            let row = SidebarDestination::for_music(destination);
            assert_eq!(row.music_destination(), Some(destination));
        }
        assert_eq!(SidebarDestination::Show.music_destination(), None);
        assert_eq!(SidebarDestination::Settings.music_destination(), None);
    }
}
