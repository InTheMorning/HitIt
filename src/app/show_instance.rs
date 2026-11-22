//! The show instance on the Show page (ADR 0111 Decisions 3, 4 and 6).
//!
//! This file holds the dialog "Make show instance" and the handlers that select
//! a show instance and the player. The dialog composite owns only the layout.
//! The setup helper runs in a command, off the UI thread.

use std::path::PathBuf;

use crate::ui::primitives::CheckboxState;
use gpui::{AppContext as _, Context, Entity};
use gpui_component::input::InputState;

use crate::application::{ApplicationCommand, CommandContext, CommandError, CommandOutcome};
use crate::broadcast::setup_helper::{
    add_show_instance, refuse_station_host, ForceConfirmation, SetupHelperControl,
    SetupHelperRequest,
};
use crate::config::{self, BroadcastHostConfig, ShowInstanceConfig};
use crate::presentation::present_command;
use crate::ui::composites::{
    open_confirmation_dialog, open_confirmation_dialog_from, ConfirmationDialogCheck,
    ConfirmationDialogDisplay, ConfirmationDialogField, ConfirmationDialogForm,
    ConfirmationDialogHandlers, ConfirmationDialogItem,
};
use crate::view_models::show::{EventKind, ShowPlayer};

use super::TopApp;

/// Give the player the drop folder of the selection (ADR 0111 Decision 6).
///
/// A show instance has its own folder below the runtime folder. A show never writes to the
/// folder of the private instance. When the show instance has no drop folder, the producer is
/// off and the error says why.
pub(super) fn apply_drop_producer(
    broadcast: &config::BroadcastCapabilities,
    owner: Option<
        &std::sync::Arc<
            std::sync::Mutex<
                crate::playback_owner::PlaybackOwner<
                    crate::playback_driver::ConfiguredPlaybackDriver,
                >,
            >,
        >,
    >,
) -> Result<(), String> {
    let Some(owner) = owner else {
        return Ok(());
    };
    let (producer, problem) = match broadcast.drop_file_producer() {
        Ok(producer) => (producer, None),
        Err(error) => (None, Some(error.to_string())),
    };
    if let Some(producer) = &producer {
        producer
            .prepare_directory()
            .map_err(|error| format!("The drop folder is not ready: {error:#}"))?;
    }
    owner
        .lock()
        .map_err(|_| "The player is not available.".to_owned())?
        .replace_idle_producer(producer)
        .map_err(|error| error.to_string())?;
    problem.map_or(Ok(()), Err)
}

const ROW_ID_PREFIX: &str = "show-make-instance-row-";

/// One reserved row of the registry that the dialog offers.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ReservedChoice {
    event_id: String,
    endpoint: String,
    token_path: String,
    label: Option<String>,
}

/// What the dialog shows besides its inputs.
#[derive(Clone, Debug, Eq, PartialEq)]
enum MakeOutcome {
    Idle,
    Working,
    Made(String),
    /// The helper reported that the instance exists. The next click on Make asks for `--force`.
    Exists(String),
    Failed(String),
}

struct MakeDialogState {
    choices: Vec<ReservedChoice>,
    selected: Option<usize>,
    outcome: MakeOutcome,
}

/// The dialog "Make show instance". The screen owns the inputs.
pub(super) struct MakeInstanceDialog {
    instance: Entity<InputState>,
    target: Entity<InputState>,
    state: Entity<MakeDialogState>,
}

/// The text of the second dialog. It names the instance and the event.
#[must_use]
pub(super) fn force_message(instance: &str, event_id: &str) -> String {
    format!(
        "Instance {instance} exists. Its configuration changes to the event {event_id}. The setup helper runs again with --force."
    )
}

/// The label of one choice in the list.
fn choice_title(choice: &ReservedChoice) -> String {
    choice
        .label
        .as_deref()
        .filter(|label| !label.trim().is_empty())
        .map_or_else(|| choice.event_id.clone(), str::to_owned)
}

fn make_display(
    instance: &Entity<InputState>,
    target: &Entity<InputState>,
    state: &Entity<MakeDialogState>,
    cx: &gpui::App,
) -> ConfirmationDialogDisplay {
    let state = state.read(cx);
    let items = state
        .choices
        .iter()
        .enumerate()
        .map(|(index, choice)| ConfirmationDialogItem {
            title: choice_title(choice).into(),
            subtitle: Some(choice.endpoint.clone().into()),
            mark: None,
            lines: vec![format!("Event ID {}", choice.event_id).into()],
            check: Some(ConfirmationDialogCheck {
                id: format!("{ROW_ID_PREFIX}{index}").into(),
                state: if state.selected == Some(index) {
                    CheckboxState::Checked
                } else {
                    CheckboxState::NotChecked
                },
                a11y_label: format!("Use event {} for the show instance", choice.event_id).into(),
            }),
            checked_lines: Vec::new(),
        })
        .collect::<Vec<_>>();
    let message = if state.choices.is_empty() {
        "The registry has no reserved event. Add a reserved event first."
    } else {
        "Enter the name of the instance and the target. Select one reserved event. The setup helper writes the event into the instance configuration."
    };
    let result = match &state.outcome {
        MakeOutcome::Idle => None,
        MakeOutcome::Working => Some(("The setup helper runs.".to_owned(), false)),
        MakeOutcome::Made(name) => Some((
            format!("Instance {name} is made and selected. Close the dialog."),
            false,
        )),
        MakeOutcome::Exists(name) => Some((
            format!("Instance {name} exists. Click Make again to confirm the change."),
            true,
        )),
        MakeOutcome::Failed(error) => Some((format!("The setup helper failed: {error}"), true)),
    };
    let finished = matches!(state.outcome, MakeOutcome::Working | MakeOutcome::Made(_));
    ConfirmationDialogDisplay {
        title: "Make show instance".into(),
        message: message.into(),
        cancel_button_id: "show-make-instance-close".into(),
        cancel_label: "Close".into(),
        cancel_a11y_label: "Close the dialog without a new show instance".into(),
        confirm_button_id: "show-make-instance-make".into(),
        confirm_label: "Make".into(),
        confirm_a11y_label: "Make the show instance with the setup helper".into(),
        destructive: false,
        confirm_enabled: !finished && state.selected.is_some(),
        items,
        section: None,
        form: Some(ConfirmationDialogForm {
            fields: vec![
                ConfirmationDialogField {
                    label: "Instance name".into(),
                    input: instance.clone(),
                },
                ConfirmationDialogField {
                    label: "Target name".into(),
                    input: target.clone(),
                },
            ],
            copy_text: None,
            result: result.map(|(text, failure)| (text.into(), failure)),
            keeps_open: true,
        }),
    }
}

/// Command that runs the setup helper, then writes the configuration entry.
struct MakeInstanceCommand {
    host: BroadcastHostConfig,
    instance: String,
    target: String,
    choice: ReservedChoice,
    force: ForceConfirmation,
    cfg_path: PathBuf,
}

impl ApplicationCommand for MakeInstanceCommand {
    type Output = ShowInstanceConfig;

    fn execute(self, context: &CommandContext) -> crate::application::CommandResult<Self::Output> {
        if context.cancellation().is_cancelled() {
            return Err(CommandError::Cancelled);
        }
        let token = PathBuf::from(&self.choice.token_path);
        let request = SetupHelperRequest {
            instance: &self.instance,
            target: &self.target,
            endpoint: &self.choice.endpoint,
            event_id: &self.choice.event_id,
            token_file: &token,
        };
        let made = SetupHelperControl::default()
            .run(&self.host, &request, self.force)
            .map_err(|error| CommandError::Other(error.to_string()))?;
        // ADR 0111 Decision 2: the entry is written only after the helper succeeds.
        config::save_show_instance(&self.cfg_path, &self.host.name, &made)
            .map_err(|error| CommandError::Other(format!("{error:#}")))?;
        Ok(CommandOutcome::without_events(made))
    }
}

impl TopApp {
    /// True while a run or a started part ties the Show page to its instance.
    fn show_choice_is_locked(&self) -> bool {
        self.started_parts.has_mixxx_parts()
            || self
                .on_air_sequence
                .as_ref()
                .is_some_and(|sequence| sequence.next_step().is_some())
            || self
                .end_show_sequence
                .as_ref()
                .is_some_and(|sequence| sequence.next_step().is_some())
    }

    /// Select the player of the show (ADR 0111 Decision 6).
    pub(super) fn set_show_player(&mut self, player: ShowPlayer, cx: &mut Context<Self>) {
        if self.show_choice_is_locked() || self.show_player == player {
            return;
        }
        self.show_player = player;
        self.reproject_show_page_from_current_queue();
        cx.notify();
    }

    /// A new selection moves the drop folder of the producer and the units that the watch reads.
    pub(super) fn apply_show_selection(&mut self, cx: &mut Context<Self>) {
        if let Err(text) = apply_drop_producer(&self.broadcast, self.playback_owner.as_ref()) {
            self.settings_status = text;
        }
        self.publisher_service_watch.take();
        self.maybe_start_broadcast_service_watch(cx);
    }

    /// Select a show instance, or the private instance to see why it is refused.
    pub(super) fn select_show_instance(&mut self, name: String, cx: &mut Context<Self>) {
        if self.show_choice_is_locked()
            || self.broadcast.selected_show_instance.as_deref() == Some(name.as_str())
        {
            return;
        }
        self.broadcast.selected_show_instance = Some(name);
        self.apply_show_selection(cx);
        self.refresh_show_page(cx);
        self.reproject_show_page_from_current_queue();
        cx.notify();
    }

    /// Open the dialog "Make show instance" (ADR 0111 Decision 3).
    pub(super) fn open_make_instance_dialog(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) {
        if self.make_instance_dialog.is_some() || self.show_choice_is_locked() {
            return;
        }
        // ADR 0112 Decision 3: a station host has no show instances. No dialog opens.
        if let Some(error) = self
            .broadcast
            .selected_host()
            .ok()
            .and_then(|host| refuse_station_host(host).err())
        {
            self.settings_status = error.to_string();
            cx.notify();
            return;
        }
        let choices = self
            .event_section_input
            .as_ref()
            .map(|input| {
                input
                    .registry
                    .events
                    .iter()
                    .filter(|event| event.kind == EventKind::Reserved)
                    .map(|event| ReservedChoice {
                        event_id: event.event_id.clone(),
                        endpoint: event.endpoint.clone(),
                        token_path: event.token_path.clone(),
                        label: event.label.clone(),
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let instance = cx.new(|cx: &mut Context<InputState>| {
            InputState::new(window, cx).placeholder("Instance name")
        });
        let target = cx.new(|cx: &mut Context<InputState>| {
            InputState::new(window, cx).placeholder("Target name")
        });
        target.update(cx, |input, cx| input.set_value("default", window, cx));
        let selected = (choices.len() == 1).then_some(0);
        let state = cx.new(|_| MakeDialogState {
            choices,
            selected,
            outcome: MakeOutcome::Idle,
        });
        self.make_instance_dialog = Some(MakeInstanceDialog {
            instance: instance.clone(),
            target: target.clone(),
            state: state.clone(),
        });
        let display_state = state.clone();
        let check_state = state;
        let cancel_entity = cx.weak_entity();
        let confirm_entity = cx.weak_entity();
        open_confirmation_dialog_from(
            window,
            cx,
            false,
            move |cx| make_display(&instance, &target, &display_state, cx),
            ConfirmationDialogHandlers::new(
                move |_, cx: &mut gpui::App| {
                    let _ = cancel_entity.update(cx, |this, _| this.make_instance_dialog = None);
                },
                move |window, cx: &mut gpui::App| {
                    let _ =
                        confirm_entity.update(cx, |this, cx| this.submit_make_instance(window, cx));
                },
            )
            .with_check_handler(move |check, _, cx: &mut gpui::App| {
                let Some(index) = check
                    .id
                    .strip_prefix(ROW_ID_PREFIX)
                    .and_then(|index| index.parse::<usize>().ok())
                else {
                    return;
                };
                check_state.update(cx, |state, _| {
                    if matches!(state.outcome, MakeOutcome::Working | MakeOutcome::Made(_)) {
                        return;
                    }
                    state.selected = Some(index);
                    state.outcome = MakeOutcome::Idle;
                });
                cx.refresh_windows();
            }),
        );
    }

    /// The button "Make". It asks for `--force` first when the instance exists.
    fn submit_make_instance(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) {
        let Some(dialog) = &self.make_instance_dialog else {
            return;
        };
        let instance = dialog.instance.read(cx).value().trim().to_owned();
        let target = dialog.target.read(cx).value().trim().to_owned();
        let state = dialog.state.clone();
        let (choice, helper_says_exists) = {
            let state = state.read(cx);
            (
                state
                    .selected
                    .and_then(|index| state.choices.get(index).cloned()),
                matches!(state.outcome, MakeOutcome::Exists(_)),
            )
        };
        let Some(choice) = choice else {
            return;
        };
        let exists_in_config = self.broadcast.selected_host().is_ok_and(|host| {
            host.show_instances
                .iter()
                .any(|existing| existing.name == instance)
        });
        if exists_in_config || helper_says_exists {
            Self::ask_for_force(window, instance, target, choice, cx);
            return;
        }
        self.run_make_instance(instance, target, choice, ForceConfirmation::NotGiven, cx);
    }

    /// The second dialog. It names the instance and the event.
    fn ask_for_force(
        window: &mut gpui::Window,
        instance: String,
        target: String,
        choice: ReservedChoice,
        cx: &mut Context<Self>,
    ) {
        let entity = cx.weak_entity();
        let message = force_message(&instance, &choice.event_id);
        open_confirmation_dialog(
            window,
            cx,
            ConfirmationDialogDisplay {
                confirm_enabled: true,
                form: None,
                section: None,
                title: "Change the instance?".into(),
                message: message.into(),
                cancel_button_id: "show-make-instance-force-cancel".into(),
                cancel_label: "Cancel".into(),
                cancel_a11y_label: "Keep the instance as it is".into(),
                confirm_button_id: "show-make-instance-force-confirm".into(),
                confirm_label: "Change instance".into(),
                confirm_a11y_label: format!("Change instance {instance} with --force").into(),
                destructive: true,
                items: Vec::new(),
            },
            ConfirmationDialogHandlers::new(
                |_, _: &mut gpui::App| {},
                move |_, cx: &mut gpui::App| {
                    let (instance, target, choice) =
                        (instance.clone(), target.clone(), choice.clone());
                    let _ = entity.update(cx, |this, cx| {
                        this.run_make_instance(
                            instance,
                            target,
                            choice,
                            ForceConfirmation::Given,
                            cx,
                        );
                    });
                },
            ),
        );
    }

    fn run_make_instance(
        &mut self,
        instance: String,
        target: String,
        choice: ReservedChoice,
        force: ForceConfirmation,
        cx: &mut Context<Self>,
    ) {
        let Some(dialog) = &self.make_instance_dialog else {
            return;
        };
        let state = dialog.state.clone();
        let host = match self.broadcast.selected_host() {
            Ok(host) => host.clone(),
            Err(error) => {
                state.update(cx, |state, _| {
                    state.outcome = MakeOutcome::Failed(format!("{error:#}"));
                });
                cx.refresh_windows();
                return;
            }
        };
        state.update(cx, |state, _| state.outcome = MakeOutcome::Working);
        cx.refresh_windows();
        let command = MakeInstanceCommand {
            host,
            instance: instance.clone(),
            target,
            choice,
            force,
            cfg_path: self.cfg_path.clone(),
        };
        let error_state = state.clone();
        present_command(
            &self.command_runner,
            command,
            CommandContext::next(),
            cx,
            move |this, made, cx| {
                let name = made.name.clone();
                if let Ok(hosts) = &mut this.broadcast.hosts {
                    let selected = this
                        .broadcast
                        .selected_host
                        .as_ref()
                        .ok()
                        .cloned()
                        .flatten();
                    let host = match selected.as_deref() {
                        Some(name) => hosts
                            .iter_mut()
                            .find(|host| host.name.trim() == name.trim()),
                        None => hosts.first_mut(),
                    };
                    if let Some(host) = host {
                        let _ = add_show_instance(host, made);
                    }
                }
                this.broadcast.selected_show_instance = Some(name.clone());
                this.apply_show_selection(cx);
                state.update(cx, |state, _| state.outcome = MakeOutcome::Made(name));
                cx.refresh_windows();
                this.refresh_show_page(cx);
                this.reproject_show_page_from_current_queue();
                cx.notify();
            },
            move |_, error, cx| {
                let text = error.to_string();
                error_state.update(cx, |state, _| {
                    state.outcome = if force == ForceConfirmation::NotGiven
                        && text.to_ascii_lowercase().contains("exist")
                    {
                        MakeOutcome::Exists(instance)
                    } else {
                        MakeOutcome::Failed(text)
                    };
                });
                cx.refresh_windows();
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ADR 0111 visual 2: the confirmation for `--force` names the instance and the event.
    #[test]
    fn adr_0111_force_message_names_the_instance_and_the_event() {
        let message = force_message("show-1", "event-one");
        assert!(message.contains("show-1") && message.contains("event-one"));
        assert!(message.contains("--force"));
    }
}
