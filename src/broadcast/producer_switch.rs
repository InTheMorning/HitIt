#![warn(clippy::pedantic)]

//! The producer switch of ADR 0111 Decision 5.
//!
//! One Mixxx lock allows one running producer. A switch from instance A to
//! instance B stops the producer of A, starts the producer of B and reads the
//! state of B. [`ProducerSwitch`] is the pure model. It does no I/O.
//! [`run_switch`] runs the model through `ServiceControl` and `Transport`.

use anyhow::{anyhow, Result};

use crate::broadcast::control::{
    validate_instance_name, CommandRunner, ServiceControl, ServiceState, UnitRef,
};
use crate::broadcast::transport::Transport;

/// Exit status of a producer that cannot get the Mixxx lock (ADR 0111
/// Decision 5).
pub const LOCK_HELD_EXIT_STATUS: i32 = 3;

/// One step of a producer switch, in the order of ADR 0111 Decision 5.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SwitchStep {
    /// Stop the producer of this instance.
    StopProducer(String),
    /// Start the producer of this instance. The start also starts its
    /// publisher (publisher ADR 0021).
    StartProducer(String),
    /// Read the state of the producer of this instance.
    ReadProducerState(String),
}

/// What `HitIt` does after the producer of B failed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwitchRecovery {
    /// The instance that holds Mixxx. Set only for exit status 3, and only
    /// when a different producer runs.
    pub lock_holder: Option<String>,
    /// The instance whose producer starts again. Set only when A was active.
    pub restart: Option<String>,
}

/// Pure model of a switch from instance A to instance B.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProducerSwitch {
    from: String,
    to: String,
    from_was_active: bool,
}

impl ProducerSwitch {
    /// Create the model.
    ///
    /// `from_was_active` is step 1 of ADR 0111 Decision 5: the fact that the
    /// producer of A was active.
    ///
    /// # Errors
    ///
    /// Returns an error when an instance name does not match the pattern of
    /// ADR 0111 Decision 1, or when both instances are the same.
    pub fn new(from: &str, to: &str, from_was_active: bool) -> Result<Self> {
        validate_instance_name(from)?;
        validate_instance_name(to)?;
        if from == to {
            return Err(anyhow!(
                "ADR 0111 Decision 5: a producer switch needs two different instances, not {from:?} twice"
            ));
        }
        Ok(Self {
            from: from.to_owned(),
            to: to.to_owned(),
            from_was_active,
        })
    }

    /// Return the instance A.
    #[must_use]
    pub fn from_instance(&self) -> &str {
        &self.from
    }

    /// Return the instance B.
    #[must_use]
    pub fn to_instance(&self) -> &str {
        &self.to
    }

    /// Return whether the producer of A was active.
    #[must_use]
    pub const fn from_was_active(&self) -> bool {
        self.from_was_active
    }

    /// Return the steps in order: stop A, start B, read B.
    #[must_use]
    pub fn steps(&self) -> [SwitchStep; 3] {
        [
            SwitchStep::StopProducer(self.from.clone()),
            SwitchStep::StartProducer(self.to.clone()),
            SwitchStep::ReadProducerState(self.to.clone()),
        ]
    }

    /// Return the recovery after a failure of B.
    ///
    /// `exit_status` is the `ExecMainStatus` of the producer of B.
    /// `running_producers` holds the instance names that
    /// `systemctl --user list-units 'mixxx-now-playing@*'` shows as running.
    #[must_use]
    pub fn recovery(
        &self,
        exit_status: Option<i32>,
        running_producers: &[String],
    ) -> SwitchRecovery {
        let lock_holder = if exit_status == Some(LOCK_HELD_EXIT_STATUS) {
            running_producers
                .iter()
                .find(|instance| **instance != self.to)
                .cloned()
        } else {
            None
        };
        SwitchRecovery {
            lock_holder,
            restart: self.from_was_active.then(|| self.from.clone()),
        }
    }
}

/// Result of the start of A after a failure of B.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RestartResult {
    /// A was not active, so `HitIt` did not start it.
    NotNeeded,
    /// The producer of A runs again.
    Started,
    /// The start of A failed with this error.
    Failed(String),
}

/// The failure of a producer switch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwitchFailure {
    /// The error of the failed step.
    pub error: String,
    /// The instance that holds Mixxx, when known.
    pub lock_holder: Option<String>,
    /// The result of the start of A.
    pub restart: RestartResult,
}

/// Typed result of a producer switch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SwitchOutcome {
    /// The producer of B runs.
    Done,
    /// The switch failed.
    Failed(SwitchFailure),
}

/// Run a switch through `ServiceControl` and `Transport`.
///
/// When the stop of A fails, A keeps its state and `HitIt` starts nothing.
/// When B fails, `HitIt` reads the exit status of B. With exit status 3, `HitIt`
/// reads the instance that holds Mixxx. `HitIt` then starts A again when A was
/// active.
#[must_use]
pub fn run_switch<R: CommandRunner>(
    control: &ServiceControl<R>,
    transport: &Transport,
    switch: &ProducerSwitch,
) -> SwitchOutcome {
    match run_steps(control, transport, switch) {
        Ok(StepResult::Done) => SwitchOutcome::Done,
        Ok(StepResult::StopFailed(error)) => SwitchOutcome::Failed(SwitchFailure {
            error,
            lock_holder: None,
            restart: RestartResult::NotNeeded,
        }),
        Ok(StepResult::StartFailed(error)) | Err(error) => {
            SwitchOutcome::Failed(recover(control, transport, switch, error))
        }
    }
}

enum StepResult {
    Done,
    StopFailed(String),
    StartFailed(String),
}

fn run_steps<R: CommandRunner>(
    control: &ServiceControl<R>,
    transport: &Transport,
    switch: &ProducerSwitch,
) -> Result<StepResult, String> {
    let from = UnitRef::producer(switch.from_instance()).map_err(|error| format!("{error:#}"))?;
    let to = UnitRef::producer(switch.to_instance()).map_err(|error| format!("{error:#}"))?;
    if let Err(error) = control.stop(transport, &from) {
        return Ok(StepResult::StopFailed(format!("{error:#}")));
    }
    if let Err(error) = control.start(transport, &to) {
        return Ok(StepResult::StartFailed(format!("{error:#}")));
    }
    match control.show(transport, &to) {
        Ok(ServiceState::Active | ServiceState::Starting) => Ok(StepResult::Done),
        Ok(ServiceState::Failed { reason }) => Ok(StepResult::StartFailed(format!(
            "the producer of {} failed: {reason}",
            switch.to_instance()
        ))),
        Ok(state) => Ok(StepResult::StartFailed(format!(
            "the producer of {} is not active: {state:?}",
            switch.to_instance()
        ))),
        Err(error) => Ok(StepResult::StartFailed(format!("{error:#}"))),
    }
}

fn recover<R: CommandRunner>(
    control: &ServiceControl<R>,
    transport: &Transport,
    switch: &ProducerSwitch,
    error: String,
) -> SwitchFailure {
    let exit_status = UnitRef::producer(switch.to_instance())
        .ok()
        .and_then(|unit| control.exec_main_status(transport, &unit).ok().flatten());
    let running = if exit_status == Some(LOCK_HELD_EXIT_STATUS) {
        control.running_producers(transport).unwrap_or_default()
    } else {
        Vec::new()
    };
    let recovery = switch.recovery(exit_status, &running);
    let restart = match recovery.restart.as_deref() {
        None => RestartResult::NotNeeded,
        Some(instance) => {
            match UnitRef::producer(instance).and_then(|unit| control.start(transport, &unit)) {
                Ok(()) => RestartResult::Started,
                Err(error) => RestartResult::Failed(format!("{error:#}")),
            }
        }
    };
    SwitchFailure {
        error,
        lock_holder: recovery.lock_holder,
        restart,
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::broadcast::control::CommandOutput;

    use super::*;

    fn switch(from_was_active: bool) -> ProducerSwitch {
        ProducerSwitch::new("mixxx", "show-1", from_was_active).expect("switch")
    }

    /// ADR 0111 criterion 6: the model gives the steps of Decision 5 in order.
    #[test]
    fn adr_0111_switch_model_gives_the_steps_in_order() {
        assert_eq!(
            switch(true).steps(),
            [
                SwitchStep::StopProducer("mixxx".to_owned()),
                SwitchStep::StartProducer("show-1".to_owned()),
                SwitchStep::ReadProducerState("show-1".to_owned()),
            ]
        );
    }

    /// ADR 0111 criterion 6: on a failure of B with exit status 3, the model
    /// gives the lock holder and the start of A.
    #[test]
    fn adr_0111_switch_model_gives_lock_holder_and_start_of_a_on_exit_status_3() {
        let running = vec!["show-1".to_owned(), "show-2".to_owned()];

        let recovery = switch(true).recovery(Some(3), &running);

        assert_eq!(recovery.lock_holder.as_deref(), Some("show-2"));
        assert_eq!(recovery.restart.as_deref(), Some("mixxx"));
    }

    /// ADR 0111 criterion 6: with A not active, the model gives no start of A.
    #[test]
    fn adr_0111_switch_model_gives_no_start_of_a_when_a_was_not_active() {
        let running = vec!["show-2".to_owned()];

        let recovery = switch(false).recovery(Some(3), &running);

        assert_eq!(recovery.lock_holder.as_deref(), Some("show-2"));
        assert_eq!(recovery.restart, None);
    }

    #[test]
    fn switch_model_gives_no_lock_holder_for_another_exit_status() {
        let running = vec!["show-2".to_owned()];

        assert_eq!(switch(true).recovery(Some(1), &running).lock_holder, None);
        assert_eq!(switch(true).recovery(None, &running).lock_holder, None);
    }

    #[test]
    fn switch_model_refuses_equal_and_invalid_instances() {
        assert!(ProducerSwitch::new("mixxx", "mixxx", true).is_err());
        assert!(ProducerSwitch::new("mixxx", "Show", true).is_err());
        assert!(ProducerSwitch::new("", "show-1", true).is_err());
    }

    type Calls = Rc<RefCell<Vec<String>>>;

    /// A fake runner. It records each call and answers from fixed text.
    struct FakeSystemctl {
        calls: Calls,
        fail_start_of: Option<&'static str>,
        b_state: &'static str,
        listing: &'static str,
    }

    impl FakeSystemctl {
        fn new(b_state: &'static str, fail_start_of: Option<&'static str>) -> (Self, Calls) {
            let calls = Calls::default();
            let runner = Self {
                calls: Rc::clone(&calls),
                fail_start_of,
                b_state,
                listing: "mixxx-now-playing@show-2.service loaded active running Mixxx\n\
                          mixxx-now-playing@show-1.service loaded failed failed Mixxx\n",
            };
            (runner, calls)
        }
    }

    impl CommandRunner for FakeSystemctl {
        fn run(&self, _program: &str, args: &[String]) -> Result<CommandOutput> {
            let line = args.join(" ");
            self.calls.borrow_mut().push(line.clone());
            if line.contains("ExecMainStatus") {
                return Ok(CommandOutput::success("ExecMainStatus=3\n"));
            }
            if line.contains("list-units") {
                return Ok(CommandOutput::success(self.listing));
            }
            if args[1] == "show" {
                return Ok(CommandOutput::success(self.b_state));
            }
            if args[1] == "start" && self.fail_start_of.is_some_and(|name| line.contains(name)) {
                return Ok(CommandOutput::failure(Some(1), "", "start failed"));
            }
            Ok(CommandOutput::success(""))
        }
    }

    const B_ACTIVE: &str = "LoadState=loaded\nActiveState=active\n";
    const B_FAILED: &str = "LoadState=loaded\nActiveState=failed\nResult=exit-code\n";

    #[test]
    fn run_switch_runs_stop_start_and_read_in_order() {
        let (runner, calls) = FakeSystemctl::new(B_ACTIVE, None);

        let outcome = run_switch(
            &ServiceControl::new(runner),
            &Transport::local(),
            &switch(true),
        );

        assert_eq!(outcome, SwitchOutcome::Done);
        let calls = calls.borrow();
        assert_eq!(calls.len(), 3);
        assert_eq!(calls[0], "--user stop mixxx-now-playing@mixxx.service");
        assert_eq!(calls[1], "--user start mixxx-now-playing@show-1.service");
        assert!(calls[2].starts_with("--user show mixxx-now-playing@show-1.service"));
    }

    #[test]
    fn run_switch_on_exit_status_3_gives_the_lock_holder_and_starts_a_again() {
        let (runner, calls) = FakeSystemctl::new(B_FAILED, None);

        let outcome = run_switch(
            &ServiceControl::new(runner),
            &Transport::local(),
            &switch(true),
        );

        let SwitchOutcome::Failed(failure) = outcome else {
            panic!("the switch must fail");
        };
        assert_eq!(failure.lock_holder.as_deref(), Some("show-2"));
        assert_eq!(failure.restart, RestartResult::Started);
        let calls = calls.borrow();
        assert!(calls.iter().any(|call| call.contains("list-units")));
        assert_eq!(
            calls.last().map(String::as_str),
            Some("--user start mixxx-now-playing@mixxx.service")
        );
    }

    #[test]
    fn run_switch_with_a_not_active_starts_nothing_after_a_failure() {
        let (runner, calls) = FakeSystemctl::new(B_FAILED, Some("show-1"));

        let outcome = run_switch(
            &ServiceControl::new(runner),
            &Transport::local(),
            &switch(false),
        );

        let SwitchOutcome::Failed(failure) = outcome else {
            panic!("the switch must fail");
        };
        assert_eq!(failure.restart, RestartResult::NotNeeded);
        let starts = calls
            .borrow()
            .iter()
            .filter(|call| call.starts_with("--user start"))
            .count();
        assert_eq!(starts, 1);
    }

    #[test]
    fn run_switch_stops_after_a_failed_stop_of_a() {
        struct FailStop;
        impl CommandRunner for FailStop {
            fn run(&self, _program: &str, _args: &[String]) -> Result<CommandOutput> {
                Ok(CommandOutput::failure(Some(1), "", "stop failed"))
            }
        }

        let outcome = run_switch(
            &ServiceControl::new(FailStop),
            &Transport::local(),
            &switch(true),
        );

        let SwitchOutcome::Failed(failure) = outcome else {
            panic!("the switch must fail");
        };
        assert!(failure.error.contains("stop failed"));
        assert_eq!(failure.restart, RestartResult::NotNeeded);
    }
}
