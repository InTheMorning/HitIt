//! Session-local Settings navigation and existing control contracts (ADR 0069).

#![warn(clippy::pedantic)]

use crate::config::UiScale;
use crate::theme_profile::ThemeProfile;

use super::maintenance::{MaintenanceView, PageChoice};
use super::startup::StartupAvailability;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum SettingsGroup {
    #[default]
    General,
    Library,
    Diagnostics,
}

impl SettingsGroup {
    pub(crate) const ALL: [Self; 3] = [Self::General, Self::Library, Self::Diagnostics];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Library => "Library",
            Self::Diagnostics => "Diagnostics",
        }
    }

    pub(crate) const fn contents(self) -> &'static [SettingsContent] {
        use SettingsContent::{
            BackgroundReports, CachedFiles, DatabaseTools, Endpoint, FlacPath, MusicDirectory,
            Scale, SessionMaintenance, Theme,
        };
        match self {
            Self::General => &[Scale, Theme],
            Self::Library => &[Endpoint, MusicDirectory, FlacPath],
            Self::Diagnostics => &[
                SessionMaintenance,
                DatabaseTools,
                BackgroundReports,
                CachedFiles,
            ],
        }
    }
}

/// One Diagnostics page is mounted at a time (ADR 0074).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum DiagnosticPage {
    #[default]
    Database,
    Configuration,
    Background,
    Session,
    CachedFiles,
}

impl DiagnosticPage {
    pub(crate) const ALL: [Self; 5] = [
        Self::Database,
        Self::Configuration,
        Self::Background,
        Self::Session,
        Self::CachedFiles,
    ];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Database => "Database",
            Self::Configuration => "Configuration",
            Self::Background => "Background tools",
            Self::Session => "App session",
            Self::CachedFiles => "Cached files",
        }
    }

    pub(crate) const fn contents(self) -> &'static [SettingsContent] {
        match self {
            Self::Database => &[SettingsContent::DatabaseTools],
            Self::Configuration => &[],
            Self::Background => &[SettingsContent::BackgroundReports],
            Self::Session => &[SettingsContent::SessionMaintenance],
            Self::CachedFiles => &[SettingsContent::CachedFiles],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SettingsContent {
    Scale,
    Theme,
    Endpoint,
    MusicDirectory,
    FlacPath,
    BackgroundReports,
    CachedFiles,
    SessionMaintenance,
    DatabaseTools,
}

impl SettingsContent {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Scale => "UI scale",
            Self::Theme => "Theme",
            Self::Endpoint => "Music Index endpoint",
            Self::MusicDirectory => "Music directory",
            Self::FlacPath => "Audio converter (optional)",
            Self::BackgroundReports => "Background tools",
            Self::CachedFiles => "Cached files",
            Self::SessionMaintenance => "App session",
            Self::DatabaseTools => "Database tools",
        }
    }

    pub(crate) const fn help(self) -> &'static str {
        match self {
            Self::Scale => "Scales the interface. Applies immediately; click Save to persist.",
            Self::Theme => "Applies immediately. Click Save to persist.",
            Self::Endpoint => "Use api.musicindex.org or a full http/https URL.",
            Self::MusicDirectory => "Current session folder. Open Configuration repair to test and save a different existing folder after ending this session.",
            Self::FlacPath => "Test FLAC and ffmpeg availability, edit the FLAC executable path, and save it with an original-file backup in Converter setup.",
            Self::BackgroundReports | Self::CachedFiles | Self::SessionMaintenance | Self::DatabaseTools => "",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SettingsAction {
    Open,
    OpenReport,
    OpenRepair,
    OpenConverter,
    SelectGroup(SettingsGroup),
    SelectDiagnostic(DiagnosticPage),
    SelectView(MaintenanceView),
    Save,
    UseDefaults,
    SetScale(UiScale),
    SetTheme(ThemeProfile),
}

impl SettingsAction {
    /// True for an action that changes stored data with no undo. "Use
    /// Defaults…" resets and saves the defaults, so it is destructive: it has
    /// the red tint and a confirmation (ADR 0020 R-ACT-3 items 1 and 4,
    /// musicindex ADR 0064 R-ACT-12).
    #[must_use]
    pub(crate) const fn is_destructive(self) -> bool {
        matches!(self, Self::UseDefaults)
    }
}

/// The text of the confirmation of "Use Defaults…".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct UseDefaultsConfirmation {
    pub(crate) title: &'static str,
    pub(crate) message: &'static str,
    pub(crate) cancel_button_id: &'static str,
    pub(crate) cancel_label: &'static str,
    pub(crate) cancel_a11y_label: &'static str,
    pub(crate) confirm_button_id: &'static str,
    pub(crate) confirm_label: &'static str,
    pub(crate) confirm_a11y_label: &'static str,
}

/// The confirmation of "Use Defaults…". Only its confirm button resets and
/// saves the defaults.
#[must_use]
pub(crate) const fn use_defaults_confirmation() -> UseDefaultsConfirmation {
    UseDefaultsConfirmation {
        title: "Use the default settings?",
        message: "This resets the General and Library settings to the defaults and saves them. You cannot undo this.",
        cancel_button_id: "settings-default-cancel",
        cancel_label: "Cancel",
        cancel_a11y_label: "Keep the current settings",
        confirm_button_id: "settings-default-confirm",
        confirm_label: "Use Defaults",
        confirm_a11y_label: "Reset and save the General and Library defaults",
    }
}

/// Only explicit editing actions can produce an effect outside navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SettingsEffect {
    Navigate,
    ConfigureConverter,
    Save,
    /// Opens the confirmation of "Use Defaults…". Its confirm button resets
    /// and saves the defaults.
    ConfirmUseDefaults,
    SetScale(UiScale),
    SetTheme(ThemeProfile),
}

#[derive(Clone, Debug)]
pub(crate) struct SettingsActionDisplay {
    pub(crate) action: SettingsAction,
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) a11y_label: String,
    pub(crate) availability: StartupAvailability,
    pub(crate) selected: bool,
}

#[derive(Debug, Default)]
pub(crate) struct SettingsVm {
    selected: SettingsGroup,
    diagnostic: DiagnosticPage,
    views: [MaintenanceView; 5],
}

impl SettingsVm {
    pub(crate) const TITLE: &'static str = "Settings";
    pub(crate) const GROUP_MENU: &'static str = "Choose settings group";
    pub(crate) const DIAGNOSTIC_MENU: &'static str = "Choose diagnostics page";
    pub(crate) const SAVE_SCOPE: &'static str =
        "Save applies General and Library controls together. Use Defaults resets and saves these controls; core paths use Configuration repair.";

    pub(crate) const fn shows_configuration_repair(&self) -> bool {
        matches!(self.selected, SettingsGroup::Diagnostics)
            && matches!(self.diagnostic, DiagnosticPage::Configuration)
    }

    pub(crate) const fn diagnostic(&self) -> DiagnosticPage {
        self.diagnostic
    }

    pub(crate) const fn view(&self) -> MaintenanceView {
        self.views[self.diagnostic as usize]
    }

    pub(crate) const fn contents(&self) -> &'static [SettingsContent] {
        match self.selected {
            SettingsGroup::Diagnostics => self.diagnostic.contents(),
            group => group.contents(),
        }
    }

    pub(crate) const fn selected(&self) -> SettingsGroup {
        self.selected
    }

    pub(crate) const fn editable(&self) -> bool {
        matches!(
            self.selected,
            SettingsGroup::General | SettingsGroup::Library
        )
    }

    pub(crate) fn dispatch(&mut self, action: SettingsAction) -> SettingsEffect {
        match action {
            SettingsAction::Open => SettingsEffect::Navigate,
            SettingsAction::OpenConverter => SettingsEffect::ConfigureConverter,
            SettingsAction::OpenRepair => {
                self.selected = SettingsGroup::Diagnostics;
                self.diagnostic = DiagnosticPage::Configuration;
                SettingsEffect::Navigate
            }
            SettingsAction::OpenReport => {
                self.diagnostic = DiagnosticPage::Background;
                self.views[self.diagnostic as usize] = MaintenanceView::Report;
                self.selected = SettingsGroup::Diagnostics;
                SettingsEffect::Navigate
            }
            SettingsAction::SelectGroup(group) => {
                self.selected = group;
                SettingsEffect::Navigate
            }
            SettingsAction::SelectDiagnostic(page) => {
                self.diagnostic = page;
                SettingsEffect::Navigate
            }
            SettingsAction::SelectView(view) => {
                self.views[self.diagnostic as usize] = view;
                SettingsEffect::Navigate
            }
            SettingsAction::Save => SettingsEffect::Save,
            SettingsAction::UseDefaults => SettingsEffect::ConfirmUseDefaults,
            SettingsAction::SetScale(scale) => SettingsEffect::SetScale(scale),
            SettingsAction::SetTheme(profile) => SettingsEffect::SetTheme(profile),
        }
    }

    pub(crate) fn navigation(&self) -> Vec<PageChoice<SettingsAction>> {
        SettingsGroup::ALL
            .into_iter()
            .map(|group| {
                PageChoice::new(
                    SettingsAction::SelectGroup(group),
                    group.label(),
                    self.selected == group,
                )
            })
            .collect()
    }

    pub(crate) fn diagnostic_navigation(&self) -> Vec<PageChoice<SettingsAction>> {
        DiagnosticPage::ALL
            .into_iter()
            .map(|page| {
                PageChoice::new(
                    SettingsAction::SelectDiagnostic(page),
                    page.label(),
                    page == self.diagnostic,
                )
            })
            .collect()
    }

    pub(crate) fn repair_entry() -> Vec<SettingsActionDisplay> {
        vec![SettingsActionDisplay {
            action: SettingsAction::OpenRepair,
            id: "settings-configuration-page".into(),
            label: "Configuration repair".into(),
            a11y_label: "Open Configuration repair".into(),
            availability: StartupAvailability::Available,
            selected: false,
        }]
    }

    pub(crate) fn converter_setup() -> Vec<SettingsActionDisplay> {
        vec![SettingsActionDisplay {
            action: SettingsAction::OpenConverter,
            id: "settings-converter".into(),
            label: super::startup::converter::TITLE.into(),
            a11y_label: "Open Converter setup to edit and test the FLAC executable path".into(),
            availability: StartupAvailability::Available,
            selected: false,
        }]
    }

    pub(crate) fn edit_actions(&self, correction_idle: bool) -> Vec<SettingsActionDisplay> {
        if !self.editable() {
            return Vec::new();
        }
        // Use Defaults saves the default values with no undo. It is the
        // leftmost button, and Save is the rightmost button (musicindex ADR
        // 0064 R-ACT-8).
        [
            (
                SettingsAction::UseDefaults,
                "settings-default",
                "Use Defaults…",
                "Reset and save General and Library defaults",
            ),
            (
                SettingsAction::Save,
                "settings-save",
                "Save",
                "Save General and Library settings",
            ),
        ]
        .into_iter()
        .map(|(action, id, label, a11y_label)| SettingsActionDisplay {
            action,
            id: id.into(),
            label: label.into(),
            a11y_label: a11y_label.into(),
            availability: if correction_idle {
                StartupAvailability::Available
            } else {
                StartupAvailability::Working
            },
            selected: false,
        })
        .collect()
    }

    pub(crate) fn scale_choices(current: UiScale) -> Vec<SettingsActionDisplay> {
        [
            (UiScale::XSmall, "XS", "Extra small"),
            (UiScale::Small, "S", "Small"),
            (UiScale::Medium, "M", "Medium"),
            (UiScale::Large, "L", "Large"),
            (UiScale::XLarge, "XL", "Extra large"),
        ]
        .into_iter()
        .map(|(scale, label, full_label)| SettingsActionDisplay {
            action: SettingsAction::SetScale(scale),
            id: format!("ui-scale-{}", scale.as_str()),
            label: label.into(),
            a11y_label: format!("{full_label} UI scale"),
            availability: StartupAvailability::Available,
            selected: current == scale,
        })
        .collect()
    }

    pub(crate) fn theme_choices(current: ThemeProfile) -> Vec<SettingsActionDisplay> {
        ThemeProfile::USER_SELECTABLE
            .into_iter()
            .map(|profile| SettingsActionDisplay {
                action: SettingsAction::SetTheme(profile),
                id: profile.as_str().into(),
                label: profile.settings_label().into(),
                a11y_label: format!("{} theme profile", profile.settings_label()),
                availability: StartupAvailability::Available,
                selected: current == profile,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adr_0074_diagnostics_selects_one_page_without_edit_effects() {
        let mut vm = SettingsVm::default();
        vm.dispatch(SettingsAction::SelectGroup(SettingsGroup::Diagnostics));
        for page in DiagnosticPage::ALL {
            assert_eq!(
                vm.dispatch(SettingsAction::SelectDiagnostic(page)),
                SettingsEffect::Navigate
            );
            assert!(vm.contents().len() <= 1);
            assert_eq!(
                vm.shows_configuration_repair(),
                page == DiagnosticPage::Configuration
            );
            assert_eq!(
                vm.dispatch(SettingsAction::SelectView(MaintenanceView::Report)),
                SettingsEffect::Navigate
            );
            assert!(vm.edit_actions(true).is_empty());
        }
        for page in DiagnosticPage::ALL {
            vm.dispatch(SettingsAction::SelectDiagnostic(page));
            assert_eq!(vm.view(), MaintenanceView::Report);
        }
        vm.dispatch(SettingsAction::OpenRepair);
        assert_eq!(vm.diagnostic(), DiagnosticPage::Configuration);
        vm.dispatch(SettingsAction::OpenReport);
        assert_eq!(vm.diagnostic(), DiagnosticPage::Background);
    }

    #[test]
    fn adr_0069_group_contract_has_stable_membership_and_accessible_actions() {
        let mut vm = SettingsVm::default();
        let ids = vm
            .navigation()
            .into_iter()
            .map(|item| item.value)
            .collect::<Vec<_>>();
        assert_eq!(vm.selected(), SettingsGroup::General);
        for group in SettingsGroup::ALL {
            assert_eq!(
                vm.dispatch(SettingsAction::SelectGroup(group)),
                SettingsEffect::Navigate
            );
            let navigation = vm.navigation();
            assert_eq!(
                navigation
                    .iter()
                    .map(|item| &item.value)
                    .collect::<Vec<_>>(),
                ids.iter().collect::<Vec<_>>()
            );
            assert_eq!(
                navigation.iter().map(|item| item.label).collect::<Vec<_>>(),
                ["General", "Library", "Diagnostics"]
            );
            assert_eq!(navigation.iter().filter(|item| item.selected).count(), 1);
            for item in navigation {
                assert_eq!(item.availability, StartupAvailability::Available);
                assert!(!item.a11y_label.is_empty());
                assert_eq!(
                    item.selected,
                    item.value == SettingsAction::SelectGroup(group)
                );
            }
        }
        assert_eq!(
            SettingsGroup::General.contents(),
            &[SettingsContent::Scale, SettingsContent::Theme]
        );
        assert_eq!(
            SettingsGroup::Library.contents(),
            &[
                SettingsContent::Endpoint,
                SettingsContent::MusicDirectory,
                SettingsContent::FlacPath
            ]
        );
        assert_eq!(
            SettingsGroup::Diagnostics.contents(),
            &[
                SettingsContent::SessionMaintenance,
                SettingsContent::DatabaseTools,
                SettingsContent::BackgroundReports,
                SettingsContent::CachedFiles
            ]
        );
    }

    #[test]
    fn adr_0069_navigation_and_reentry_emit_no_edit_effect() {
        let mut vm = SettingsVm::default();
        for group in SettingsGroup::ALL {
            assert_eq!(
                vm.dispatch(SettingsAction::SelectGroup(group)),
                SettingsEffect::Navigate
            );
            for _ in 0..3 {
                assert_eq!(vm.dispatch(SettingsAction::Open), SettingsEffect::Navigate);
                assert_eq!(vm.selected(), group);
            }
        }
        vm.dispatch(SettingsAction::SelectGroup(SettingsGroup::Library));
        assert_eq!(
            vm.dispatch(SettingsAction::OpenReport),
            SettingsEffect::Navigate
        );
        assert_eq!(vm.selected(), SettingsGroup::Diagnostics);
    }

    /// musicindex ADR 0064 R-ACT-8: Use Defaults saves the default values
    /// with no undo. It is the leftmost button, and Save is the rightmost
    /// button.
    #[test]
    fn adr_0064_r_act_8_use_defaults_is_left_of_save() {
        use crate::view_models::button_row::{adr_0064_violation, ButtonRowRole};

        let mut vm = SettingsVm::default();
        for group in [SettingsGroup::General, SettingsGroup::Library] {
            vm.dispatch(SettingsAction::SelectGroup(group));
            let roles: Vec<ButtonRowRole> = vm
                .edit_actions(true)
                .iter()
                .map(|display| match display.action {
                    SettingsAction::UseDefaults => ButtonRowRole::Destructive,
                    SettingsAction::Save => ButtonRowRole::Safe,
                    _ => ButtonRowRole::Other,
                })
                .collect();
            if let Some(violation) = adr_0064_violation(&roles) {
                panic!("{violation} Change the order in `SettingsVm::edit_actions`.");
            }
        }
    }

    /// musicindex ADR 0064 R-ACT-12: "Use
    /// Defaults…" is destructive (ADR 0020 R-ACT-3 items 1 and 4). It opens
    /// a confirmation and does not save the defaults at once.
    #[test]
    fn adr_0020_r_act_3_use_defaults_is_destructive_and_asks_first() {
        const FIX: &str = "musicindex ADR 0064 R-ACT-12: \"Use Defaults…\" is destructive. Give it the red tint, and open the confirmation before the reset.";
        let mut vm = SettingsVm::default();
        vm.dispatch(SettingsAction::SelectGroup(SettingsGroup::General));
        let use_defaults = vm
            .edit_actions(true)
            .into_iter()
            .find(|display| display.action == SettingsAction::UseDefaults)
            .expect("General has Use Defaults");
        assert!(use_defaults.action.is_destructive(), "{FIX}");
        assert!(!SettingsAction::Save.is_destructive());
        assert_eq!(use_defaults.label, "Use Defaults…", "{FIX}");
        assert_eq!(
            vm.dispatch(SettingsAction::UseDefaults),
            SettingsEffect::ConfirmUseDefaults,
            "{FIX}"
        );
        let confirmation = use_defaults_confirmation();
        assert_eq!(confirmation.cancel_label, "Cancel");
        assert_eq!(confirmation.confirm_label, "Use Defaults");
        assert_ne!(
            confirmation.cancel_button_id,
            confirmation.confirm_button_id
        );
    }

    #[test]
    fn adr_0069_edit_actions_have_one_shared_whole_form_route() {
        let mut vm = SettingsVm::default();
        for group in [SettingsGroup::General, SettingsGroup::Library] {
            vm.dispatch(SettingsAction::SelectGroup(group));
            let actions = vm.edit_actions(true);
            assert!(vm
                .edit_actions(false)
                .iter()
                .all(|action| action.availability == StartupAvailability::Working));
            assert_eq!(actions.len(), 2);
            assert_eq!(
                vm.dispatch(actions[0].action),
                SettingsEffect::ConfirmUseDefaults
            );
            assert_eq!(vm.dispatch(actions[1].action), SettingsEffect::Save);
            assert!(actions
                .iter()
                .all(|action| action.a11y_label.contains("General and Library")));
            assert_eq!(vm.selected(), group);
        }
        vm.dispatch(SettingsAction::OpenReport);
        assert!(vm.edit_actions(true).is_empty());
        for (choices, expected) in [
            (
                SettingsVm::scale_choices(UiScale::Large),
                SettingsAction::SetScale(UiScale::Large),
            ),
            (
                SettingsVm::theme_choices(ThemeProfile::Light),
                SettingsAction::SetTheme(ThemeProfile::Light),
            ),
        ] {
            assert_eq!(choices.iter().filter(|choice| choice.selected).count(), 1);
            assert_eq!(
                choices
                    .iter()
                    .find(|choice| choice.selected)
                    .unwrap()
                    .action,
                expected
            );
            assert!(choices.iter().all(|choice| !choice.a11y_label.is_empty()));
        }
    }
}
