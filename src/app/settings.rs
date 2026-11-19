//! Settings composition and persistent input/focus wiring (ADR 0069).

#![warn(clippy::pedantic)]

use std::rc::Rc;

use gpui::{AnyElement, ClipboardItem, Context, Empty, IntoElement, ParentElement, Styled, Window};

use crate::config;
use crate::theme_profile::ThemeProfile;
use crate::ui::composites::maintenance_page::PageNavigation;
use crate::ui::composites::selectable_text::SelectableText;
use crate::ui::composites::settings::{
    settings_actions, settings_cached_row, settings_frame, settings_heading, settings_message,
    settings_section_heading, settings_tab_row, settings_text_input, settings_value_and_action,
    SettingsCallback,
};
use crate::ui::composites::settings_row::{settings_row, SettingsRowDisplay};
use crate::ui::control_styles::ControlStyle;
use crate::ui::primitives::Button;
use crate::view_models::cached_files::{CachedFileAction, CachedFileActionDisplay, CachedFileRow};
use crate::view_models::settings::SettingsGroup;
use crate::view_models::settings::{
    SettingsAction, SettingsContent, SettingsEffect, SettingsRowControl, SettingsVm,
};
use crate::view_models::startup::StartupAvailability;

/// The mpv state text when the background worker cannot run the check.
const MPV_CHECK_WORKER_ERROR: &str = "the background worker is not available.";

use super::TopApp;

impl TopApp {
    pub(super) fn maintenance_navigation(&self, cx: &mut Context<Self>) -> PageNavigation {
        let owner = cx.weak_entity();
        PageNavigation {
            view: self.settings.view(),
            scroll: self
                .settings_scroll
                .diagnostic_handle(self.settings.diagnostic())
                .clone(),
            select: Rc::new(move |view, window, cx| {
                let _ = owner.update(cx, |this, cx| {
                    this.settings_action(SettingsAction::SelectView(view), window, cx);
                });
            }),
        }
    }

    pub(super) fn settings_action(
        &mut self,
        action: SettingsAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let check_audio = action == SettingsAction::SelectGroup(SettingsGroup::Audio);
        match action {
            SettingsAction::SelectDriver(driver) => {
                self.save_audio_driver(driver, window, cx);
                return;
            }
            SettingsAction::PlayTestSound => {
                self.play_test_sound(cx);
                return;
            }
            _ => {}
        }
        match self.settings.dispatch(action) {
            // Group buttons stay mounted and retain their own keyboard/mouse focus.
            SettingsEffect::Navigate => {}
            SettingsEffect::CheckMpv => self.check_mpv(window, cx),
            SettingsEffect::ContinueFirstStart => {
                self.first_start_audio = false;
                self.select_tab(super::AppTab::Music, window, cx);
            }
            SettingsEffect::AnswerTestSound(heard) => self.answer_test_sound(heard, cx),
            SettingsEffect::ConfigureConverter => self.capability_action(
                crate::view_models::startup::capabilities::CapabilityAction::Configure(
                    crate::application::capability::Dependency::Converter,
                ),
                window,
                cx,
            ),
            SettingsEffect::Save => self.save_settings(window, cx),
            SettingsEffect::CopyVersion => {
                cx.write_to_clipboard(ClipboardItem::new_string(SettingsVm::version_line()));
            }
            SettingsEffect::ConfirmUseDefaults => {
                crate::ui::shells::settings_defaults_confirmation::open_use_defaults_dialog(
                    window,
                    cx,
                    crate::view_models::settings::use_defaults_confirmation(),
                    Self::use_default_settings,
                );
            }
            SettingsEffect::SetScale(scale) => self.set_ui_scale(scale, window, cx),
            SettingsEffect::SetTheme(profile) => self.set_theme_profile(profile, window, cx),
        }
        if check_audio {
            self.check_mpv(window, cx);
        }
        cx.notify();
    }

    fn save_audio_driver(
        &mut self,
        driver: crate::config::PlaybackDriver,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let path = self.cfg_path.clone();
        let Some(worker) = self.maintenance_worker.clone() else {
            self.settings_status = "App could not save the player settings.".into();
            self.audio_save_error = Some(self.settings_status.clone());
            cx.notify();
            return;
        };
        if let Ok(receiver) = worker.submit(move || {
            let source = match crate::config::correction::CorrectionSource::read(&path) {
                Ok(source) => source,
                Err(error) => {
                    return crate::application::commands::maintenance::CorrectionResult::Failed(
                        format!("App could not load the player settings: {error:#}"),
                    );
                }
            };
            let command = crate::application::commands::maintenance::CorrectionCommand {
                path,
                source: Some(std::sync::Arc::new(source)),
                draft: crate::config::correction::CorrectionDraft {
                    raw: None,
                    fields: vec![(
                        crate::config::correction::CorrectionField("playback.driver"),
                        driver.as_str().into(),
                    )],
                },
                access: crate::application::commands::maintenance::CorrectionAccess::OptionalOnly,
                operation: crate::application::commands::maintenance::CorrectionOperation::Save,
            };
            command.execute()
        }) {
            crate::presentation::startup_presenter::present_startup(
                receiver,
                worker,
                window,
                cx,
                move |this, result, window, cx| {
                    match result {
                        Ok(crate::application::commands::maintenance::CorrectionResult::Saved(
                            receipt,
                        )) => {
                            let snapshot = std::sync::Arc::new(receipt.fresh);
                            this.audio_driver = driver;
                            this.audio_save_error = None;
                            this.refresh_corrected_settings(&snapshot, window, cx);
                            this.check_saved_capabilities(
                                vec![crate::application::capability::Dependency::Playback],
                                window,
                                cx,
                            );
                            this.settings_status.clear();
                        }
                        Ok(
                            crate::application::commands::maintenance::CorrectionResult::Failed(
                                error,
                            ),
                        ) => {
                            this.audio_save_error = Some(error.clone());
                            this.settings_status = error;
                        }
                        _ => {
                            this.settings_status = "App could not save the player settings.".into();
                            this.audio_save_error = Some(this.settings_status.clone());
                        }
                    }
                    cx.notify();
                },
            );
        } else {
            self.settings_status = "App could not save the player settings. Try again.".into();
            self.audio_save_error = Some(self.settings_status.clone());
            cx.notify();
        }
    }

    pub(super) fn check_mpv(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(worker) = self.maintenance_worker.clone() else {
            self.mpv_status = crate::view_models::audio_setup::MpvStatus::DidNotStart(
                MPV_CHECK_WORKER_ERROR.into(),
            );
            cx.notify();
            return;
        };
        let path = self.cfg_path.clone();
        if let Ok(receiver) = worker.submit(move || {
            let snapshot = crate::config::ConfigSnapshot::read_existing(&path)
                .map_err(|error| format!("{error:#}"))?;
            let mut config = snapshot.playback().map_err(|error| format!("{error:?}"))?;
            config.driver = crate::config::PlaybackDriver::Mpv;
            crate::playback_driver::ConfiguredPlaybackDriver::from_config_isolated(
                &config,
                "audio-status",
            )
            .and_then(|driver| driver.ping_version())
            .map_err(|error| format!("{error:#}"))
        }) {
            crate::presentation::startup_presenter::present_startup(
                receiver,
                worker,
                window,
                cx,
                |this, result, _, cx| {
                    this.mpv_status =
                        crate::view_models::audio_setup::MpvStatus::from_ping(match result {
                            Ok(ping) => ping,
                            Err(error) => Err(format!("{error:#}")),
                        });
                    cx.notify();
                },
            );
        } else {
            self.mpv_status = crate::view_models::audio_setup::MpvStatus::DidNotStart(
                MPV_CHECK_WORKER_ERROR.into(),
            );
            cx.notify();
        }
    }

    fn use_default_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.endpoint_input.update(cx, |input, cx| {
            input.set_value(crate::api::DEFAULT_BASE_URL, window, cx);
        });
        self.flac_path_input.update(cx, |input, cx| {
            input.set_value("", window, cx);
        });
        self.set_ui_scale(config::UiScale::Medium, window, cx);
        self.set_theme_profile(ThemeProfile::default(), window, cx);
        self.save_settings(window, cx);
    }

    pub(super) fn apply_corrected_presentation(
        &mut self,
        snapshot: &config::ConfigSnapshot,
        cx: &mut Context<Self>,
    ) {
        let preferences = snapshot.workspace_preferences();
        let preferences = preferences
            .as_ref()
            .and_then(|config| config.layout.as_ref());
        self.content_pane_width = Self::initial_content_pane_width(preferences);
        self.workspace_layout = Self::initial_workspace_layout(
            snapshot
                .workspace_layout
                .as_ref()
                .ok()
                .and_then(Option::as_ref),
            super::WorkspaceScreenMount::Music,
        );
        cx.notify();
    }

    pub(super) fn refresh_corrected_settings(
        &mut self,
        snapshot: &config::ConfigSnapshot,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Form values and appearance share the accepted task 006 owner.
        // Scoped capability checks are queued separately; Save never retries an action.
        if let Ok(endpoint) = &snapshot.musicindex_endpoint {
            self.endpoint_input.update(cx, |input, cx| {
                input.set_value(endpoint.clone(), window, cx);
            });
        }
        if let Ok(path) = &snapshot.flac_path {
            let value = path
                .as_ref()
                .map_or_else(String::new, |path| path.display().to_string());
            self.flac_path_input
                .update(cx, |input, cx| input.set_value(value, window, cx));
        }
        if let Ok(scale) = snapshot.ui_scale {
            self.set_ui_scale(scale, window, cx);
        }
        if let Ok(profile) = snapshot.theme_profile {
            self.set_theme_profile(profile, window, cx);
        }
        cx.notify();
    }
}

pub(super) fn render_settings(app: &mut TopApp, cx: &mut Context<TopApp>) -> AnyElement {
    let correction_idle = !app
        .configuration_editor
        .as_ref()
        .is_some_and(|editor| editor.read(cx).vm.is_working());
    let entity = cx.weak_entity();
    let callback: SettingsCallback = Rc::new(move |action, window, cx| {
        let _ = entity.update(cx, |this, cx| this.settings_action(action, window, cx));
    });
    let tab_row = settings_tab_row(app.settings.group_control(), &callback);
    let mut navigation = Vec::new();
    let workspace = app.settings.selected() == SettingsGroup::Diagnostics;
    if workspace {
        navigation.push(
            crate::ui::composites::maintenance_page::PageMenu::new(
                "diagnostics-page-menu",
                SettingsVm::DIAGNOSTIC_MENU,
                app.settings.diagnostic_navigation(),
                callback.clone(),
            )
            .into_any_element(),
        );
    }
    let mut content = settings_top_rows(app, workspace, &callback, cx);
    // musicindex ADR 0083 R-VERSION-4 and ADR 0093 Decision 2: the version
    // line is the first row of Diagnostics.
    if app.settings.shows_version() {
        content.push(settings_value_and_action(
            SelectableText::new("settings-version-line", SettingsVm::version_line())
                .into_any_element(),
            settings_actions(SettingsVm::version_copy_action(), &callback, cx),
            cx,
        ));
    }
    for section in app.settings.sections() {
        content.push(settings_section_heading(section.heading, cx));
        for &field in section.rows {
            if field == SettingsContent::CachedFiles {
                content.extend(render_cached_files(app, cx));
                continue;
            }
            let row = settings_content_row(app, field, correction_idle, &callback, cx);
            content.push(row);
        }
    }
    for &field in app.settings.contents() {
        match field {
            SettingsContent::SessionMaintenance => {
                content.extend(render_session_tools(app, correction_idle, cx));
            }
            SettingsContent::DatabaseTools => {
                use gpui::IntoElement as _;
                if let Some(tools) = &app.database_tools {
                    content.push(tools.clone().into_any_element());
                }
            }
            SettingsContent::BackgroundReports => {
                content.extend(app.render_capabilities(true, cx));
            }
            _ => {}
        }
    }
    if app.settings.editable() {
        content.push(settings_message(SettingsVm::SAVE_SCOPE, cx));
        content.push(settings_actions(
            app.settings.edit_actions(correction_idle),
            &callback,
            cx,
        ));
    }
    if app.settings.shows_configuration_repair() {
        if let Some(editor) = &app.configuration_editor {
            use gpui::IntoElement as _;
            content.push(editor.clone().into_any_element());
        }
    }
    if !app.settings_status.is_empty() && !workspace {
        content.push(settings_message(app.settings_status.clone(), cx));
    }
    let page_scroll = app.settings_scroll.handle(app.settings.selected());
    settings_frame(tab_row, navigation, content, page_scroll, workspace, cx)
}

/// One row of a General or Library section. The view model gives the name,
/// the hint and the control type. This function gives the control.
fn settings_content_row(
    app: &TopApp,
    field: SettingsContent,
    correction_idle: bool,
    callback: &SettingsCallback,
    cx: &mut Context<TopApp>,
) -> AnyElement {
    let callback = Rc::clone(callback);
    let control = match field {
        SettingsContent::Scale => {
            settings_actions(SettingsVm::scale_choices(app.ui_scale), &callback, cx)
        }
        SettingsContent::Theme => {
            settings_actions(SettingsVm::theme_choices(app.theme_profile), &callback, cx)
        }
        SettingsContent::ShowPlayer | SettingsContent::MpvStatus | SettingsContent::TestSound => {
            return audio_settings_control(app, field, &callback, cx);
        }
        SettingsContent::Defaults => {
            settings_actions(app.settings.defaults_action(correction_idle), &callback, cx)
        }
        SettingsContent::Endpoint => settings_text_input(&app.endpoint_input, cx),
        SettingsContent::MusicDirectory => settings_value_and_action(
            settings_message(app.music_dir.display().to_string(), cx),
            settings_actions(SettingsVm::repair_entry(), &callback, cx),
            cx,
        ),
        SettingsContent::FlacPath => {
            let path = app.flac_path_input.read(cx).value().to_string();
            let path = if path.is_empty() {
                "Not set".to_owned()
            } else {
                path
            };
            settings_value_and_action(
                settings_message(path, cx),
                settings_actions(SettingsVm::converter_setup(), &callback, cx),
                cx,
            )
        }
        SettingsContent::BackgroundReports
        | SettingsContent::CachedFiles
        | SettingsContent::SessionMaintenance
        | SettingsContent::DatabaseTools => return Empty.into_any_element(),
    };
    settings_row(
        SettingsRowDisplay {
            name: field.label().into(),
            hint: field.help().into(),
            control_below: field
                .control()
                .is_some_and(SettingsRowControl::always_below),
        },
        control,
        cx,
    )
}

/// The heading of the group, and the first start rows of ADR 0104 Decision 5.
/// The first start text shows only on the group "Audio".
fn settings_top_rows(
    app: &TopApp,
    workspace: bool,
    callback: &SettingsCallback,
    cx: &mut Context<TopApp>,
) -> Vec<AnyElement> {
    if workspace {
        return Vec::new();
    }
    let first_start = app.first_start_audio && app.settings.selected() == SettingsGroup::Audio;
    if !first_start {
        return vec![settings_heading(app.settings.selected().label(), cx)];
    }
    vec![
        settings_heading(crate::view_models::audio_setup::FIRST_START_TITLE, cx),
        settings_message(crate::view_models::audio_setup::FIRST_START_TEXT, cx),
        settings_actions(SettingsVm::first_start_continue(), callback, cx),
    ]
}

fn audio_settings_control(
    app: &TopApp,
    field: SettingsContent,
    callback: &SettingsCallback,
    cx: &mut Context<TopApp>,
) -> AnyElement {
    match field {
        SettingsContent::ShowPlayer => {
            let choices =
                settings_actions(SettingsVm::player_choices(app.audio_driver), callback, cx);
            if let Some(error) = app.audio_save_error.as_ref() {
                gpui::div()
                    .flex()
                    .flex_col()
                    .child(choices)
                    .child(settings_message(error.clone(), cx))
                    .into_any_element()
            } else {
                choices
            }
        }
        SettingsContent::MpvStatus if app.mpv_status.offers_check_again() => {
            settings_value_and_action(
                settings_message(app.mpv_status.label(), cx),
                settings_actions(
                    vec![crate::view_models::settings::SettingsActionDisplay {
                        action: SettingsAction::CheckMpv,
                        id: "audio-check-mpv".into(),
                        label: "Check again".into(),
                        a11y_label: "Check again whether mpv is installed".into(),
                        availability: StartupAvailability::Available,
                        selected: false,
                    }],
                    callback,
                    cx,
                ),
                cx,
            )
        }
        SettingsContent::MpvStatus => settings_message(app.mpv_status.label(), cx),
        SettingsContent::TestSound => audio_test_sound_control(app, callback, cx),
        _ => Empty.into_any_element(),
    }
}

fn audio_test_sound_control(
    app: &TopApp,
    callback: &SettingsCallback,
    cx: &mut Context<TopApp>,
) -> AnyElement {
    use crate::view_models::audio_setup::TestSoundOutcome;
    let outcome = app.test_sound_outcome();
    if outcome == TestSoundOutcome::NotTested {
        return settings_actions(
            vec![crate::view_models::settings::SettingsActionDisplay {
                action: SettingsAction::PlayTestSound,
                id: "audio-test-sound".into(),
                label: "Play test sound".into(),
                a11y_label: "Play test sound".into(),
                availability: StartupAvailability::Available,
                selected: false,
            }],
            callback,
            cx,
        );
    }
    let answers = [true, false].map(
        |heard| crate::view_models::settings::SettingsActionDisplay {
            action: SettingsAction::AnswerTestSound(heard),
            id: if heard {
                "audio-test-yes"
            } else {
                "audio-test-no"
            }
            .into(),
            label: if heard { "Yes" } else { "No" }.into(),
            a11y_label: if heard {
                "Yes, I heard the test sound"
            } else {
                "No, I did not hear the test sound"
            }
            .into(),
            availability: StartupAvailability::Available,
            selected: false,
        },
    );
    let mut row = gpui::div()
        .flex()
        .flex_col()
        .child(settings_message("Did you hear the test sound?", cx))
        .child(settings_actions(answers.to_vec(), callback, cx));
    if outcome == TestSoundOutcome::NotHeard {
        row = row.child(settings_message(
            crate::view_models::audio_setup::NOT_HEARD_HELP,
            cx,
        ));
    }
    row.into_any_element()
}

fn render_session_tools(
    app: &TopApp,
    correction_idle: bool,
    cx: &mut Context<TopApp>,
) -> Option<AnyElement> {
    if let Some(callback) = app.session_callback.clone() {
        let available = app.maintenance_worker.is_some()
            && !app.capability_vm.is_working()
            && correction_idle
            && !app
                .database_tools
                .as_ref()
                .is_some_and(|tools| tools.read(cx).vm.is_working());
        return Some(crate::ui::composites::maintenance_forms::session_entry(
            crate::view_models::startup::session::SessionReportVm::entry(available),
            app.command_runner.session().generation(),
            &app.previous_session_report,
            &app.log_frames,
            callback,
            app.maintenance_navigation(cx),
            cx,
        ));
    }
    None
}

/// The Audio cache section of Library (ADR 0100 Decision 6, ADR 0093
/// Decision 7): one row with the count and "Delete All Cached", then one row
/// for each cached file.
fn render_cached_files(app: &TopApp, cx: &mut Context<TopApp>) -> Vec<AnyElement> {
    let execution = app.command_runner.availability();
    let delete_all = app.cached_files.delete_all_action(execution).map_or_else(
        || Empty.into_any_element(),
        |action| {
            use gpui::IntoElement as _;
            cached_action(action, cx).into_any_element()
        },
    );
    let mut content = vec![settings_row(
        SettingsRowDisplay {
            name: app.cached_files.title().into(),
            hint: SettingsContent::CachedFiles.help().into(),
            control_below: false,
        },
        delete_all,
        cx,
    )];
    for row in app.cached_files.rows(&app.audio_cache_dir, execution) {
        content.push(match row {
            CachedFileRow::Artist(name) => settings_message(name, cx),
            CachedFileRow::Track { title, delete } => {
                settings_cached_row(title, cached_action(delete, cx), cx)
            }
        });
    }
    if let Some(message) = app.cached_files.status() {
        content.push(settings_message(message, cx));
    }
    content
}

fn cached_action(display: CachedFileActionDisplay, cx: &mut Context<TopApp>) -> Button {
    let entity = cx.weak_entity();
    Button::styled(
        gpui::SharedString::from(display.id),
        ControlStyle::Destructive,
    )
    .label(display.label)
    .a11y_label(display.a11y_label)
    .disabled(display.availability != StartupAvailability::Available)
    .on_activate(move |_, cx| {
        let _ = entity.update(cx, |this, cx| match &display.action {
            CachedFileAction::Delete(path) => this.delete_cached_file(path.clone(), cx),
            CachedFileAction::DeleteAll => this.delete_all_cached(cx),
        });
    })
}
