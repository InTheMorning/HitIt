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
    /// The version line of musicindex ADR 0083 R-VERSION-4. Operator choice of
    /// 2026-11-18: the line shows on this tab only (ADR 0093 Decision 1).
    About,
}

impl SettingsGroup {
    pub(crate) const ALL: [Self; 4] =
        [Self::General, Self::Library, Self::Diagnostics, Self::About];
    /// The most groups that the tab row can hold (ADR 0093 Decision 2).
    pub(crate) const MAX_GROUPS: usize = 5;

    const fn id(self) -> &'static str {
        match self {
            Self::General => "settings-group-general",
            Self::Library => "settings-group-library",
            Self::Diagnostics => "settings-group-diagnostics",
            Self::About => "settings-group-about",
        }
    }

    const fn a11y_label(self) -> &'static str {
        match self {
            Self::General => "Show General settings",
            Self::Library => "Show Library settings",
            Self::Diagnostics => "Show Diagnostics tools",
            Self::About => "Show About",
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Library => "Library",
            Self::Diagnostics => "Diagnostics",
            Self::About => "About",
        }
    }

    /// The sections and rows of the group (ADR 0093 Decision 7). Diagnostics
    /// holds tools, not settings, so it has no section. About holds only the
    /// version line.
    pub(crate) const fn sections(self) -> &'static [SettingsSection] {
        use SettingsContent::{Defaults, Endpoint, FlacPath, MusicDirectory, Scale, Theme};
        match self {
            Self::General => &[
                SettingsSection {
                    heading: "Appearance",
                    rows: &[Scale, Theme],
                },
                SettingsSection {
                    heading: "Defaults",
                    rows: &[Defaults],
                },
            ],
            Self::Library => &[
                SettingsSection {
                    heading: "Music Index",
                    rows: &[Endpoint],
                },
                SettingsSection {
                    heading: "Files",
                    rows: &[MusicDirectory, FlacPath],
                },
            ],
            Self::Diagnostics | Self::About => &[],
        }
    }
}

/// One section of a group. The heading is the section heading of the page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SettingsSection {
    pub(crate) heading: &'static str,
    pub(crate) rows: &'static [SettingsContent],
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
    Defaults,
    BackgroundReports,
    CachedFiles,
    SessionMaintenance,
    DatabaseTools,
}

/// The control of one row (ADR 0093 Decision 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SettingsRowControl {
    /// A set of choices, such as the UI scale.
    Choices,
    /// One action.
    Action,
    /// A text field. It is always below the name of its row.
    TextField,
    /// The current value and one action.
    ValueAndAction,
}

impl SettingsRowControl {
    /// True when the control must go below the name, whatever the width.
    pub(crate) const fn always_below(self) -> bool {
        matches!(self, Self::TextField)
    }
}

impl SettingsContent {
    /// The control type of a setting row, or `None` for a Diagnostics item.
    pub(crate) const fn control(self) -> Option<SettingsRowControl> {
        match self {
            Self::Scale | Self::Theme => Some(SettingsRowControl::Choices),
            Self::Defaults => Some(SettingsRowControl::Action),
            Self::Endpoint => Some(SettingsRowControl::TextField),
            Self::MusicDirectory | Self::FlacPath => Some(SettingsRowControl::ValueAndAction),
            Self::BackgroundReports
            | Self::CachedFiles
            | Self::SessionMaintenance
            | Self::DatabaseTools => None,
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Scale => "UI scale",
            Self::Theme => "Theme",
            Self::Endpoint => "Music Index endpoint",
            Self::MusicDirectory => "Music directory",
            Self::FlacPath => "Audio converter",
            Self::Defaults => "Default values",
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
            Self::Defaults => "Resets the General and Library settings to the defaults and saves them.",
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
    CopyVersion,
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
    CopyVersion,
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

/// One segment of the group control.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SettingsGroupSegment {
    pub(crate) group: SettingsGroup,
    pub(crate) id: &'static str,
    pub(crate) label: &'static str,
    pub(crate) a11y_label: &'static str,
}

/// The group control below the page bar (ADR 0093 Decision 1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SettingsGroupControl {
    pub(crate) selected: SettingsGroup,
    pub(crate) segments: Vec<SettingsGroupSegment>,
}

#[derive(Debug, Default)]
pub(crate) struct SettingsVm {
    selected: SettingsGroup,
    diagnostic: DiagnosticPage,
    views: [MaintenanceView; 5],
}

impl SettingsVm {
    pub(crate) const TITLE: &'static str = "Settings";
    pub(crate) const DIAGNOSTIC_MENU: &'static str = "Choose diagnostics page";
    pub(crate) const SAVE_SCOPE: &'static str =
        "Save applies General and Library controls together. Use Defaults resets and saves these controls; core paths use Configuration repair.";

    pub(crate) fn version_line() -> String {
        crate::build_values::info().version_line()
    }

    /// musicindex ADR 0083 R-VERSION-4: the version line shows on About only.
    pub(crate) const fn shows_version(&self) -> bool {
        matches!(self.selected, SettingsGroup::About)
    }

    pub(crate) fn version_copy_action() -> Vec<SettingsActionDisplay> {
        vec![SettingsActionDisplay {
            action: SettingsAction::CopyVersion,
            id: "settings-copy-version".into(),
            label: "Copy".into(),
            a11y_label: "Copy the version line".into(),
            availability: StartupAvailability::Available,
            selected: false,
        }]
    }

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

    /// The Diagnostics items of the mounted page. The other groups show
    /// their rows through `sections`.
    pub(crate) const fn contents(&self) -> &'static [SettingsContent] {
        match self.selected {
            SettingsGroup::Diagnostics => self.diagnostic.contents(),
            _ => &[],
        }
    }

    /// The sections and rows of the selected group.
    pub(crate) const fn sections(&self) -> &'static [SettingsSection] {
        self.selected.sections()
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
            SettingsAction::CopyVersion => SettingsEffect::CopyVersion,
            SettingsAction::UseDefaults => SettingsEffect::ConfirmUseDefaults,
            SettingsAction::SetScale(scale) => SettingsEffect::SetScale(scale),
            SettingsAction::SetTheme(profile) => SettingsEffect::SetTheme(profile),
        }
    }

    /// The group control: one segment for each group, in the sequence of
    /// `SettingsGroup::ALL`.
    pub(crate) fn group_control(&self) -> SettingsGroupControl {
        debug_assert!(SettingsGroup::ALL.len() <= SettingsGroup::MAX_GROUPS);
        SettingsGroupControl {
            selected: self.selected,
            segments: SettingsGroup::ALL
                .into_iter()
                .map(|group| SettingsGroupSegment {
                    group,
                    id: group.id(),
                    label: group.label(),
                    a11y_label: group.a11y_label(),
                })
                .collect(),
        }
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

    fn form_availability(correction_idle: bool) -> StartupAvailability {
        if correction_idle {
            StartupAvailability::Available
        } else {
            StartupAvailability::Working
        }
    }

    /// The action of the row "Default values". It has the red tint and its
    /// confirmation (musicindex ADR 0064 R-ACT-12).
    pub(crate) fn defaults_action(&self, correction_idle: bool) -> Vec<SettingsActionDisplay> {
        if !self.editable() {
            return Vec::new();
        }
        vec![SettingsActionDisplay {
            action: SettingsAction::UseDefaults,
            id: "settings-default".into(),
            label: "Use Defaults…".into(),
            a11y_label: "Reset and save General and Library defaults".into(),
            availability: Self::form_availability(correction_idle),
            selected: false,
        }]
    }

    /// The Save action. It stays at the end of the form until ADR 0069 task
    /// 002 moves it to the page bar.
    pub(crate) fn edit_actions(&self, correction_idle: bool) -> Vec<SettingsActionDisplay> {
        if !self.editable() {
            return Vec::new();
        }
        vec![SettingsActionDisplay {
            action: SettingsAction::Save,
            id: "settings-save".into(),
            label: "Save".into(),
            a11y_label: "Save General and Library settings".into(),
            availability: Self::form_availability(correction_idle),
            selected: false,
        }]
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
    fn musicindex_adr_0083_settings_exposes_version_line_and_copy_action() {
        let info = crate::build_values::info();
        let line = SettingsVm::version_line();
        assert_eq!(line, info.version_line());
        assert!(line.starts_with("HitIt 0.1.0"));
        if let Some(revision) = info.git_revision {
            assert!(line.contains(revision));
        }
        if let Some(built_at) = info.built_at {
            assert!(line.contains(&built_at[..10]));
        }

        // Operator choice of 2026-11-18: the line shows on About only.
        let mut vm = SettingsVm::default();
        for group in SettingsGroup::ALL {
            vm.dispatch(SettingsAction::SelectGroup(group));
            assert_eq!(vm.shows_version(), group == SettingsGroup::About);
        }
        assert!(SettingsGroup::About.sections().is_empty());
        assert!(!vm.editable());

        let copy = SettingsVm::version_copy_action();
        assert_eq!(copy.len(), 1);
        assert_eq!(copy[0].action, SettingsAction::CopyVersion);
        assert_eq!(copy[0].label, "Copy");
        assert_eq!(
            SettingsVm::default().dispatch(copy[0].action),
            SettingsEffect::CopyVersion
        );
    }

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
    fn adr_0093_group_control_has_stable_segments_and_accessible_labels() {
        let mut vm = SettingsVm::default();
        assert_eq!(vm.selected(), SettingsGroup::General);
        for group in SettingsGroup::ALL {
            assert_eq!(
                vm.dispatch(SettingsAction::SelectGroup(group)),
                SettingsEffect::Navigate
            );
            let control = vm.group_control();
            assert_eq!(control.selected, group);
            assert!(control.segments.len() <= SettingsGroup::MAX_GROUPS);
            assert_eq!(
                control
                    .segments
                    .iter()
                    .map(|segment| segment.label)
                    .collect::<Vec<_>>(),
                ["General", "Library", "Diagnostics", "About"]
            );
            assert_eq!(
                control
                    .segments
                    .iter()
                    .map(|segment| segment.group)
                    .collect::<Vec<_>>(),
                SettingsGroup::ALL
            );
            for segment in &control.segments {
                assert!(!segment.a11y_label.is_empty());
                assert!(!segment.id.is_empty());
            }
        }
    }

    /// ADR 0093 Decision 7: the sections and rows of General and Library,
    /// and no row for Diagnostics.
    #[test]
    fn adr_0093_sections_give_the_rows_of_decision_7() {
        let rows = |group: SettingsGroup| {
            group
                .sections()
                .iter()
                .map(|section| (section.heading, section.rows.to_vec()))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            rows(SettingsGroup::General),
            [
                (
                    "Appearance",
                    vec![SettingsContent::Scale, SettingsContent::Theme]
                ),
                ("Defaults", vec![SettingsContent::Defaults]),
            ]
        );
        assert_eq!(
            rows(SettingsGroup::Library),
            [
                ("Music Index", vec![SettingsContent::Endpoint]),
                (
                    "Files",
                    vec![SettingsContent::MusicDirectory, SettingsContent::FlacPath]
                ),
            ]
        );
        assert!(SettingsGroup::Diagnostics.sections().is_empty());
        for group in [SettingsGroup::General, SettingsGroup::Library] {
            for content in group.sections().iter().flat_map(|section| section.rows) {
                assert!(content.control().is_some());
                assert!(!content.label().is_empty());
                assert!(!content.help().is_empty());
                assert!(!content.help().contains('\n'));
            }
        }
        assert!(SettingsContent::Endpoint
            .control()
            .is_some_and(SettingsRowControl::always_below));
        assert!(!SettingsContent::Scale
            .control()
            .is_some_and(SettingsRowControl::always_below));
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

    /// musicindex ADR 0064 R-ACT-12: "Use
    /// Defaults…" is destructive (ADR 0020 R-ACT-3 items 1 and 4). It opens
    /// a confirmation and does not save the defaults at once.
    #[test]
    fn adr_0020_r_act_3_use_defaults_is_destructive_and_asks_first() {
        const FIX: &str = "musicindex ADR 0064 R-ACT-12: \"Use Defaults…\" is destructive. Give it the red tint, and open the confirmation before the reset.";
        let mut vm = SettingsVm::default();
        vm.dispatch(SettingsAction::SelectGroup(SettingsGroup::General));
        let use_defaults = vm
            .defaults_action(true)
            .into_iter()
            .next()
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
            let defaults = vm.defaults_action(true);
            assert!(vm
                .edit_actions(false)
                .iter()
                .chain(vm.defaults_action(false).iter())
                .all(|action| action.availability == StartupAvailability::Working));
            assert_eq!(actions.len(), 1);
            assert_eq!(defaults.len(), 1);
            assert_eq!(
                vm.dispatch(defaults[0].action),
                SettingsEffect::ConfirmUseDefaults
            );
            assert_eq!(vm.dispatch(actions[0].action), SettingsEffect::Save);
            assert!(actions
                .iter()
                .chain(defaults.iter())
                .all(|action| action.a11y_label.contains("General and Library")));
            assert_eq!(vm.selected(), group);
        }
        vm.dispatch(SettingsAction::OpenReport);
        assert!(vm.edit_actions(true).is_empty());
        assert!(vm.defaults_action(true).is_empty());
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
