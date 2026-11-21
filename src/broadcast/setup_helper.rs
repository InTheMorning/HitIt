#![warn(clippy::pedantic)]

//! Publisher setup helper client for ADR 0111 Decision 3.
//!
//! This module runs `setup-mixxx-musicindex` to make a show instance. Publisher
//! ADR 0021 owns the flags and the JSON output. The helper runs through a
//! `Transport`, so it runs on the host that owns the units. `HitIt` writes no
//! unit file and no publisher configuration.

use std::error::Error;
use std::fmt;
use std::path::Path;

use serde::Deserialize;

use crate::broadcast::control::{
    validate_instance_name, CommandOutput, CommandRunner, ProcessCommandRunner,
};
use crate::broadcast::transport::Reachability;
use crate::config::{BroadcastHostConfig, ShowInstanceConfig};

const SETUP_HELPER_BINARY: &str = "setup-mixxx-musicindex";

type SetupResult<T> = std::result::Result<T, SetupHelperError>;

/// Inputs of one setup helper run. The event comes from a registry row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SetupHelperRequest<'a> {
    /// Name of the show instance.
    pub instance: &'a str,
    /// Publisher target name.
    pub target: &'a str,
    /// Relay endpoint of the registry row.
    pub endpoint: &'a str,
    /// Event ID of the registry row.
    pub event_id: &'a str,
    /// Token file path of the registry row. The helper reads the file.
    pub token_file: &'a Path,
}

/// The confirmation of the operator to overwrite an existing instance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ForceConfirmation {
    /// The operator gave no confirmation. The helper gets no `--force`.
    NotGiven,
    /// The operator confirmed that the instance changes to the selected event.
    Given,
}

/// Setup helper failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SetupHelperError {
    /// The request names the private instance (ADR 0111 Decision 4).
    PrivateInstance {
        /// Private instance name.
        instance: String,
    },
    /// A request field is not valid.
    InvalidInput {
        /// Detail of the failure.
        detail: String,
    },
    /// The host cannot be reached through its transport.
    NotReachable,
    /// The command could not be spawned or waited on.
    CommandIo {
        /// Detail of the failure.
        detail: String,
    },
    /// The helper exited with a failure status.
    CommandFailed {
        /// Process exit code, when the platform reported one.
        code: Option<i32>,
        /// Detail from the command output.
        detail: String,
    },
    /// The last stdout line is not the JSON object of publisher ADR 0021.
    InvalidOutput {
        /// Detail of the failure.
        detail: String,
    },
}

impl fmt::Display for SetupHelperError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PrivateInstance { instance } => write!(
                formatter,
                "ADR 0111 Decision 4: HitIt does not run the setup helper on the private instance {instance}. Use a show instance."
            ),
            Self::NotReachable => formatter.write_str("publisher host is not reachable"),
            Self::InvalidInput { detail }
            | Self::CommandIo { detail }
            | Self::InvalidOutput { detail } => formatter.write_str(detail),
            Self::CommandFailed { code, detail } => match code {
                Some(code) => write!(formatter, "setup helper failed with exit code {code}: {detail}"),
                None => write!(formatter, "setup helper failed: {detail}"),
            },
        }
    }
}

impl Error for SetupHelperError {}

/// The JSON object of publisher ADR 0021 §The Setup Helper.
#[derive(Debug, Deserialize)]
struct SetupHelperJson {
    instance: String,
    target: String,
    config_file: String,
    publisher_unit: String,
    producer_unit: String,
}

/// Build the arguments of ADR 0111 Decision 3, in that order.
///
/// `--force` is last and only with the confirmation.
///
/// # Errors
///
/// Returns `InvalidInput` when a field is empty or the instance name does not
/// match the pattern of ADR 0111 Decision 1, and `PrivateInstance` when the
/// request names the private instance.
pub fn setup_helper_args(
    private_instance: &str,
    request: &SetupHelperRequest<'_>,
    force: ForceConfirmation,
) -> SetupResult<Vec<String>> {
    if request.instance.trim() == private_instance.trim() {
        return Err(SetupHelperError::PrivateInstance {
            instance: request.instance.trim().to_owned(),
        });
    }
    validate_instance_name(request.instance).map_err(|error| SetupHelperError::InvalidInput {
        detail: format!("{error:#}"),
    })?;
    let target = required("target name", request.target)?;
    let endpoint = required("endpoint", request.endpoint)?;
    let event_id = required("event id", request.event_id)?;
    let token_file = request
        .token_file
        .to_str()
        .ok_or_else(|| SetupHelperError::InvalidInput {
            detail: format!(
                "token file path must be UTF-8: {}",
                request.token_file.display()
            ),
        })
        .and_then(|path| required("token file path", path))?;
    let mut args = vec![
        "--instance".to_owned(),
        request.instance.to_owned(),
        "--target".to_owned(),
        target,
        "--endpoint".to_owned(),
        endpoint,
        "--event-id".to_owned(),
        event_id,
        "--token-file".to_owned(),
        token_file,
        "--display".to_owned(),
        "--no-start".to_owned(),
        "--json".to_owned(),
    ];
    if force == ForceConfirmation::Given {
        args.push("--force".to_owned());
    }
    Ok(args)
}

/// Read the last stdout line as the JSON object of publisher ADR 0021.
///
/// # Errors
///
/// Returns `InvalidOutput` when the last line is not a JSON object with the
/// five fields of the helper output.
pub fn parse_setup_output(stdout: &str) -> SetupResult<ShowInstanceConfig> {
    let last = stdout.trim_end().lines().next_back().unwrap_or("").trim();
    if !last.starts_with('{') {
        return Err(SetupHelperError::InvalidOutput {
            detail: "setup helper output has no JSON object on the last line".to_owned(),
        });
    }
    let json = serde_json::from_str::<SetupHelperJson>(last).map_err(|error| {
        SetupHelperError::InvalidOutput {
            detail: format!("parse setup helper JSON: {error}"),
        }
    })?;
    Ok(ShowInstanceConfig {
        name: json.instance,
        target: json.target,
        config_file: json.config_file,
        publisher_unit: json.publisher_unit,
        producer_unit: json.producer_unit,
    })
}

/// Add a show instance to the host. An entry of the same name is replaced.
///
/// # Errors
///
/// Returns `PrivateInstance` for the private instance, and `InvalidInput` for
/// a name that does not match the pattern of ADR 0111 Decision 1.
pub fn add_show_instance(
    host: &mut BroadcastHostConfig,
    instance: ShowInstanceConfig,
) -> SetupResult<()> {
    if instance.name == host.instance_name {
        return Err(SetupHelperError::PrivateInstance {
            instance: instance.name,
        });
    }
    validate_instance_name(&instance.name).map_err(|error| SetupHelperError::InvalidInput {
        detail: format!("{error:#}"),
    })?;
    match host
        .show_instances
        .iter_mut()
        .find(|existing| existing.name == instance.name)
    {
        Some(existing) => *existing = instance,
        None => host.show_instances.push(instance),
    }
    Ok(())
}

/// Setup helper control with an explicit command runner.
#[derive(Clone, Debug)]
pub struct SetupHelperControl<R> {
    runner: R,
}

impl Default for SetupHelperControl<ProcessCommandRunner> {
    fn default() -> Self {
        Self {
            runner: ProcessCommandRunner,
        }
    }
}

impl<R: CommandRunner> SetupHelperControl<R> {
    /// Build setup helper control with an explicit command runner.
    #[must_use]
    pub const fn new(runner: R) -> Self {
        Self { runner }
    }

    /// Run the setup helper on the host and return the instance that it made.
    ///
    /// # Errors
    ///
    /// Returns an error before any command runs for the private instance or a
    /// bad input. Returns a failure state when the host, the command or the
    /// output does not satisfy publisher ADR 0021.
    pub fn run(
        &self,
        host: &BroadcastHostConfig,
        request: &SetupHelperRequest<'_>,
        force: ForceConfirmation,
    ) -> SetupResult<ShowInstanceConfig> {
        let args = setup_helper_args(&host.instance_name, request, force)?;
        let run = host
            .transport
            .run(&self.runner, SETUP_HELPER_BINARY, &args)
            .map_err(|error| SetupHelperError::CommandIo {
                detail: format!("run {SETUP_HELPER_BINARY}: {error:#}"),
            })?;
        if matches!(run.reachability, Reachability::NotReachable) {
            return Err(SetupHelperError::NotReachable);
        }
        let output = run.output;
        if !output.status.success() {
            return Err(SetupHelperError::CommandFailed {
                code: output.status.code(),
                detail: command_detail(&output),
            });
        }
        let made = parse_setup_output(&output.stdout)?;
        if made.name != request.instance {
            return Err(SetupHelperError::InvalidOutput {
                detail: format!(
                    "setup helper made instance {}, not {}",
                    made.name, request.instance
                ),
            });
        }
        Ok(made)
    }

    /// Run the setup helper and add the show instance to the host.
    ///
    /// The host changes only after the helper succeeds (ADR 0111 Decision 2).
    ///
    /// # Errors
    ///
    /// Returns the failure of [`Self::run`] or of [`add_show_instance`].
    pub fn make_show_instance(
        &self,
        host: &mut BroadcastHostConfig,
        request: &SetupHelperRequest<'_>,
        force: ForceConfirmation,
    ) -> SetupResult<ShowInstanceConfig> {
        let made = self.run(host, request, force)?;
        add_show_instance(host, made.clone())?;
        Ok(made)
    }
}

/// Make a show instance with the real command runner.
///
/// # Errors
///
/// Returns the failure of [`SetupHelperControl::make_show_instance`].
pub fn make_show_instance(
    host: &mut BroadcastHostConfig,
    request: &SetupHelperRequest<'_>,
    force: ForceConfirmation,
) -> SetupResult<ShowInstanceConfig> {
    SetupHelperControl::default().make_show_instance(host, request, force)
}

fn required(label: &str, value: &str) -> SetupResult<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(SetupHelperError::InvalidInput {
            detail: format!("{label} cannot be empty"),
        });
    }
    Ok(value.to_owned())
}

fn command_detail(output: &CommandOutput) -> String {
    let stderr = output.stderr.trim();
    let text = if stderr.is_empty() {
        output.stdout.trim()
    } else {
        stderr
    };
    if text.is_empty() {
        "no output".to_owned()
    } else {
        text.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use anyhow::{anyhow, Result};

    use super::*;

    #[derive(Debug)]
    struct StubRunner {
        outputs: RefCell<Vec<CommandOutput>>,
        calls: RefCell<Vec<(String, Vec<String>)>>,
    }

    impl StubRunner {
        fn new(outputs: Vec<CommandOutput>) -> Self {
            Self {
                outputs: RefCell::new(outputs),
                calls: RefCell::new(Vec::new()),
            }
        }
    }

    impl CommandRunner for &StubRunner {
        fn run(&self, program: &str, args: &[String]) -> Result<CommandOutput> {
            self.calls
                .borrow_mut()
                .push((program.to_owned(), args.to_vec()));
            let mut outputs = self.outputs.borrow_mut();
            if outputs.is_empty() {
                return Err(anyhow!("stub command output missing"));
            }
            Ok(outputs.remove(0))
        }
    }

    const HELPER_JSON: &str = r#"{"instance":"show-1","target":"default","config_file":"/fixture/show-1/config.toml","publisher_unit":"musicindex-live-publisher@show-1.service","producer_unit":"mixxx-now-playing@show-1.service"}"#;

    fn request(instance: &str) -> SetupHelperRequest<'_> {
        SetupHelperRequest {
            instance,
            target: "default",
            endpoint: "https://relay.example",
            event_id: "event-one",
            token_file: Path::new("/fixture/event-one.token"),
        }
    }

    /// ADR 0111 criterion 3: the arguments follow Decision 3. `--force` needs
    /// the confirmation.
    #[test]
    fn adr_0111_setup_arguments_follow_decision_3_and_force_needs_confirmation() {
        let expected = [
            "--instance",
            "show-1",
            "--target",
            "default",
            "--endpoint",
            "https://relay.example",
            "--event-id",
            "event-one",
            "--token-file",
            "/fixture/event-one.token",
            "--display",
            "--no-start",
            "--json",
        ];
        let plain = setup_helper_args("mixxx", &request("show-1"), ForceConfirmation::NotGiven)
            .expect("arguments");
        assert_eq!(plain, expected);
        let forced = setup_helper_args("mixxx", &request("show-1"), ForceConfirmation::Given)
            .expect("arguments");
        assert_eq!(forced[..expected.len()], expected);
        assert_eq!(forced[expected.len()..], ["--force"]);
    }

    /// ADR 0111 criterion 4: the parser reads the last stdout line and refuses
    /// an output with no JSON object on that line.
    #[test]
    fn adr_0111_parser_reads_the_last_line_and_refuses_other_output() {
        let stdout = format!("note: made the configuration\n{HELPER_JSON}\n");
        let made = parse_setup_output(&stdout).expect("parse");
        assert_eq!(made.name, "show-1");
        assert_eq!(made.target, "default");
        assert_eq!(made.config_file, "/fixture/show-1/config.toml");
        assert_eq!(made.producer_unit, "mixxx-now-playing@show-1.service");

        for bad in [
            "",
            "done\n",
            &format!("{HELPER_JSON}\ndone\n"),
            "[\"show-1\",\"default\",\"a\",\"b\",\"c\"]\n",
            "{\"instance\":\"show-1\"}\n",
        ] {
            assert!(
                matches!(
                    parse_setup_output(bad),
                    Err(SetupHelperError::InvalidOutput { .. })
                ),
                "{bad:?}"
            );
        }
    }

    /// ADR 0111 criterion 5: the setup helper refuses the private instance and
    /// runs no command.
    #[test]
    fn adr_0111_setup_helper_refuses_the_private_instance() {
        let runner = StubRunner::new(vec![]);
        let control = SetupHelperControl::new(&runner);
        let mut host = BroadcastHostConfig::default_local();
        host.instance_name = "mixxx".to_owned();

        let error = control
            .make_show_instance(&mut host, &request("mixxx"), ForceConfirmation::Given)
            .expect_err("private instance");

        assert!(matches!(error, SetupHelperError::PrivateInstance { .. }));
        assert!(error.to_string().contains("ADR 0111 Decision 4"));
        assert!(runner.calls.borrow().is_empty(), "no command may run");
        assert!(host.show_instances.is_empty());
    }

    /// ADR 0111 Decision 2: the host gets the entry only after the helper
    /// succeeds.
    #[test]
    fn adr_0111_host_changes_only_after_the_helper_succeeds() -> Result<()> {
        let mut host = BroadcastHostConfig::default_local();
        let failing = StubRunner::new(vec![CommandOutput::failure(Some(2), "", "instance exists")]);
        let error = SetupHelperControl::new(&failing)
            .make_show_instance(&mut host, &request("show-1"), ForceConfirmation::NotGiven)
            .expect_err("helper failure");
        assert!(matches!(
            error,
            SetupHelperError::CommandFailed { code: Some(2), .. }
        ));
        assert!(host.show_instances.is_empty());

        let bad_output = StubRunner::new(vec![CommandOutput::success("no json\n")]);
        SetupHelperControl::new(&bad_output)
            .make_show_instance(&mut host, &request("show-1"), ForceConfirmation::NotGiven)
            .expect_err("bad output");
        assert!(host.show_instances.is_empty());

        let working = StubRunner::new(vec![CommandOutput::success(format!("{HELPER_JSON}\n"))]);
        let made = SetupHelperControl::new(&working).make_show_instance(
            &mut host,
            &request("show-1"),
            ForceConfirmation::NotGiven,
        )?;
        assert_eq!(host.show_instances, vec![made]);
        let calls = working.calls.borrow();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, SETUP_HELPER_BINARY);

        let again = StubRunner::new(vec![CommandOutput::success(format!("{HELPER_JSON}\n"))]);
        SetupHelperControl::new(&again).make_show_instance(
            &mut host,
            &request("show-1"),
            ForceConfirmation::Given,
        )?;
        assert_eq!(host.show_instances.len(), 1, "same name replaces the entry");
        Ok(())
    }
}
