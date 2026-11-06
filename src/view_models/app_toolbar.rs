//! App-toolbar display contracts.
//!
//! The app shell owns toolbar rendering and command wiring. This module owns
//! the stable ids, labels, and accessibility text so the shell does not grow
//! screen-local display policy.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AppToolbarTabKey {
    Music,
    Show,
    Settings,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AppToolbarTabDisplay {
    pub(crate) key: AppToolbarTabKey,
    pub(crate) id: &'static str,
    pub(crate) label: &'static str,
    pub(crate) a11y_label: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GlobalSearchDisplay {
    /// Local search remains available independently of optional Index requests.
    pub(crate) availability: crate::view_models::startup::StartupAvailability,
    pub(crate) input_id: &'static str,
    pub(crate) placeholder: &'static str,
    pub(crate) search_button_id: &'static str,
    pub(crate) search_button_label: &'static str,
    pub(crate) search_button_a11y_label: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AppToolbarDisplay {
    pub(crate) id: &'static str,
    pub(crate) leading_id: &'static str,
    pub(crate) center_id: &'static str,
    pub(crate) mark_id: &'static str,
    pub(crate) mark_a11y_label: &'static str,
    pub(crate) tabs: [AppToolbarTabDisplay; 3],
    pub(crate) global_search: GlobalSearchDisplay,
}

/// When the toolbar shows its Back and Forward controls (ADR 0089 Decision
/// 9, R-NAV-2).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ToolbarHistoryControls {
    /// Each control shows only when its history is not empty. Show and
    /// Settings use this rule.
    WhenAvailable,
    /// Both controls always show. An unavailable control shows as disabled.
    /// Music uses this rule, so Forward shows disabled with no later page.
    Always,
}

/// One Back or Forward control of the toolbar. The shell renders it from
/// this contract and decides neither its label nor its availability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ToolbarHistoryButtonDisplay {
    /// Stable element identifier.
    pub(crate) id: &'static str,
    /// The visible tooltip and the accessible name.
    pub(crate) label: &'static str,
    /// Whether the command renders unavailable.
    pub(crate) disabled: bool,
}

/// The history part of the toolbar: the two controls and the page title
/// at the left (ADR 0089 Decision 9, R-TOOLBAR-1 item 1).
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ToolbarNavigationDisplay {
    /// The Back control, or `None` when it is absent.
    pub(crate) back: Option<ToolbarHistoryButtonDisplay>,
    /// The Forward control, or `None` when it is absent.
    pub(crate) forward: Option<ToolbarHistoryButtonDisplay>,
    /// The page title. It is empty when the toolbar shows no title.
    pub(crate) title: String,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct AppToolbarVm;

impl AppToolbarVm {
    pub(crate) fn index_search_availability(
        features: crate::application::capability::FeatureAvailability,
    ) -> Result<(), crate::application::capability::ExecutionUnavailable> {
        features.require(crate::application::capability::Dependency::MusicIndex)
    }
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self
    }

    /// Gives the Back and Forward controls and the title from the current
    /// navigation state (R-NAV-2 items 1 and 4). `nav` is `None` when the
    /// mount has no navigation, and then no control shows its availability.
    #[must_use]
    pub(crate) fn navigation(
        self,
        nav: Option<&crate::view_models::workspace::FrameNavigationState>,
        title: String,
        controls: ToolbarHistoryControls,
    ) -> ToolbarNavigationDisplay {
        let _ = self;
        let can_back =
            nav.is_some_and(crate::view_models::workspace::FrameNavigationState::can_go_back);
        let can_forward =
            nav.is_some_and(crate::view_models::workspace::FrameNavigationState::can_go_forward);
        let button = |id, label, available: bool| match controls {
            ToolbarHistoryControls::Always => Some(ToolbarHistoryButtonDisplay {
                id,
                label,
                disabled: !available,
            }),
            ToolbarHistoryControls::WhenAvailable => {
                available.then_some(ToolbarHistoryButtonDisplay {
                    id,
                    label,
                    disabled: false,
                })
            }
        };
        ToolbarNavigationDisplay {
            back: button("app-toolbar-back", "Back", can_back),
            forward: button("app-toolbar-forward", "Forward", can_forward),
            title,
        }
    }

    #[must_use]
    pub(crate) const fn display(self) -> AppToolbarDisplay {
        let _ = self;
        AppToolbarDisplay {
            id: "app-toolbar",
            leading_id: "app-toolbar-leading",
            center_id: "app-toolbar-center",
            mark_id: "app-toolbar-mark",
            mark_a11y_label: "Application mark",
            tabs: [
                AppToolbarTabDisplay {
                    key: AppToolbarTabKey::Music,
                    id: "app-tab-music",
                    label: "Music",
                    a11y_label: "Open Music",
                },
                AppToolbarTabDisplay {
                    key: AppToolbarTabKey::Show,
                    id: "app-tab-show",
                    label: "Show",
                    a11y_label: "Open Show",
                },
                AppToolbarTabDisplay {
                    key: AppToolbarTabKey::Settings,
                    id: "app-tab-settings",
                    label: "Settings",
                    a11y_label: "Show Settings",
                },
            ],
            global_search: GlobalSearchDisplay {
                availability: crate::view_models::startup::StartupAvailability::Available,
                input_id: "app-toolbar-global-search-input",
                placeholder: "Search Library and Index",
                search_button_id: "app-toolbar-global-search-submit",
                search_button_label: "Search",
                search_button_a11y_label: "Search Library and Index",
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toolbar_display_projects_stable_regions() {
        let display = AppToolbarVm::new().display();

        assert_eq!(display.id, "app-toolbar");
        assert_eq!(display.leading_id, "app-toolbar-leading");
        assert_eq!(display.center_id, "app-toolbar-center");
    }

    #[test]
    fn toolbar_tabs_project_labels_and_a11y() {
        let display = AppToolbarVm::new().display();
        let labels: Vec<_> = display.tabs.iter().map(|tab| tab.label).collect();
        let a11y: Vec<_> = display.tabs.iter().map(|tab| tab.a11y_label).collect();

        assert_eq!(labels, ["Music", "Show", "Settings"]);
        assert_eq!(a11y, ["Open Music", "Open Show", "Show Settings"]);
    }

    #[test]
    fn global_search_display_projects_submit_contract() {
        let display = AppToolbarVm::new().display().global_search;

        assert_eq!(display.input_id, "app-toolbar-global-search-input");
        assert_eq!(display.placeholder, "Search Library and Index");
        assert_eq!(display.search_button_id, "app-toolbar-global-search-submit");
        assert_eq!(display.search_button_label, "Search");
        assert_eq!(display.search_button_a11y_label, "Search Library and Index");
    }

    #[test]
    fn music_navigation_always_shows_back_and_disabled_forward_without_history() {
        let nav = crate::view_models::workspace::FrameNavigationState::new(
            crate::view_models::workspace::FrameNavigationEntry::Destination(
                crate::view_models::workspace::MusicDestination::Releases,
            ),
        );
        let display = AppToolbarVm::new().navigation(
            Some(&nav),
            "Releases".to_owned(),
            ToolbarHistoryControls::Always,
        );

        let back = display.back.expect("Music always shows Back");
        let forward = display.forward.expect("Music always shows Forward");
        assert_eq!((back.label, back.disabled), ("Back", true));
        assert_eq!((forward.label, forward.disabled), ("Forward", true));
        assert_eq!(display.title, "Releases");
    }

    #[test]
    fn show_and_settings_navigation_hides_history_controls_without_history() {
        let nav = crate::view_models::workspace::FrameNavigationState::new(
            crate::view_models::workspace::FrameNavigationEntry::Settings,
        );
        let display = AppToolbarVm::new().navigation(
            Some(&nav),
            String::new(),
            ToolbarHistoryControls::WhenAvailable,
        );

        assert_eq!(display.back, None, "an absent control is not disabled");
        assert_eq!(display.forward, None);
    }

    #[test]
    fn navigation_enables_back_then_forward_after_a_back_move() {
        use crate::view_models::workspace::{FrameNavigationEntry, MusicDestination};
        let mut nav = crate::view_models::workspace::FrameNavigationState::new(
            FrameNavigationEntry::Destination(MusicDestination::Releases),
        );
        nav.push(FrameNavigationEntry::Settings);
        assert!(nav.go_back().is_ok());

        let display = AppToolbarVm::new().navigation(
            Some(&nav),
            String::new(),
            ToolbarHistoryControls::WhenAvailable,
        );
        assert_eq!(display.back, None, "no earlier page remains");
        assert!(display.forward.is_some_and(|forward| !forward.disabled));
    }

    #[test]
    fn toolbar_display_avoids_product_naming() {
        let display = AppToolbarVm::new().display();
        let strings = [
            display.mark_a11y_label,
            display.global_search.placeholder,
            display.global_search.search_button_label,
            display.global_search.search_button_a11y_label,
            display.tabs[0].label,
            display.tabs[1].label,
            display.tabs[2].label,
        ];

        assert!(
            strings
                .iter()
                .all(|value| !value.contains("MusicIndex") && !value.contains("V4V")),
            "top-level toolbar strings should remain neutral until naming is decided"
        );
    }
}
