//! Steps of a show that Mixxx plays (ADR 0111 Decisions 6 and 7).
//!
//! The functions in this file run one step through `ServiceControl` or the
//! encoder. They are generic over the command runner, so a test uses a fake
//! runner. The stored fact of the producer switch lives in
//! `db::show_producer_switch`.

use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::application::{ApplicationCommand, CommandContext, CommandError, CommandOutcome};
use crate::broadcast::control::{CommandRunner, ServiceControl, ServiceState, UnitRef};
use crate::broadcast::encoder::{self, EncoderTarget};
use crate::broadcast::producer_switch::{run_switch, ProducerSwitch, RestartResult, SwitchOutcome};
use crate::broadcast::transport::Transport;
use crate::db::show_producer_switch as stored;
use crate::runtime::BroadcastServiceWatchSnapshot;
use crate::view_models::show::StartedParts;

/// Whether the producer of the private instance is active in a watch snapshot.
pub(super) fn private_producer_was_active(
    snapshot: Option<&BroadcastServiceWatchSnapshot>,
    private_instance: &str,
) -> bool {
    snapshot.is_some_and(|snapshot| {
        snapshot.instances.iter().any(|instance| {
            instance.name == private_instance && instance.producer == ServiceState::Active
        })
    })
}

/// Step 1 of "Go on air" with Mixxx: the producer switch with the stored fact.
///
/// The fact is stored before the switch stops the private producer. It is
/// cleared when the switch fails and the private producer runs again, or was
/// never stopped. It stays when the private producer is stopped and did not
/// start again, so "End show" can start it.
///
/// Returns `Ok(true)` when the switch stopped the private producer.
pub(super) fn switch_to_show<R: CommandRunner>(
    conn: &Connection,
    control: &ServiceControl<R>,
    transport: &Transport,
    private_instance: &str,
    show_instance: &str,
    private_was_active: bool,
) -> Result<bool, String> {
    let switch = ProducerSwitch::new(private_instance, show_instance, private_was_active)
        .map_err(|error| format!("{error:#}"))?;
    if private_was_active {
        stored::store_private_stopped(conn, show_instance, private_instance)
            .map_err(|error| format!("{error:#}"))?;
    }
    match run_switch(control, transport, &switch) {
        SwitchOutcome::Done => Ok(private_was_active),
        SwitchOutcome::Failed(failure) => {
            let mut message = failure.error;
            if let Some(holder) = failure.lock_holder {
                message = format!("{holder} holds Mixxx. {message}");
            }
            match failure.restart {
                RestartResult::Started | RestartResult::NotNeeded => {
                    stored::clear(conn).map_err(|error| format!("{error:#}"))?;
                }
                RestartResult::Failed(error) => {
                    message = format!(
                        "{message} The producer of {private_instance} did not start again: {error}"
                    );
                }
            }
            Err(message)
        }
    }
}

/// Step 5 of "End show" with Mixxx: start the private producer, then clear the fact.
pub(super) fn start_private_producer<R: CommandRunner>(
    conn: &Connection,
    control: &ServiceControl<R>,
    transport: &Transport,
    private_instance: &str,
) -> Result<(), String> {
    let unit = UnitRef::producer(private_instance).map_err(|error| format!("{error:#}"))?;
    control
        .start(transport, &unit)
        .map_err(|error| format!("Start the producer of {private_instance}: {error:#}"))?;
    stored::clear(conn).map_err(|error| format!("{error:#}"))
}

/// Which unit of the show instance a stop step addresses.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ShowUnit {
    Producer,
    Publisher,
}

/// Steps 3 and 4 of "End show" with Mixxx: stop one unit of the show instance.
///
/// ADR 0111 Decision 4: `HitIt` never stops the publisher of the private instance.
pub(super) fn stop_show_unit<R: CommandRunner>(
    control: &ServiceControl<R>,
    transport: &Transport,
    private_instance: &str,
    show_instance: &str,
    unit: ShowUnit,
) -> Result<(), String> {
    if show_instance == private_instance {
        return Err(format!(
            "ADR 0111 Decision 4: HitIt does not stop the relay sender or the producer of the private instance {private_instance} in End show."
        ));
    }
    let unit_ref = match unit {
        ShowUnit::Producer => UnitRef::producer(show_instance),
        ShowUnit::Publisher => UnitRef::publisher(show_instance),
    }
    .map_err(|error| format!("{error:#}"))?;
    control
        .stop(transport, &unit_ref)
        .map_err(|error| format!("Stop {}: {error:#}", unit_ref.unit()))
}

/// Rebuild the started parts from the stored fact after a restart of `HitIt`.
///
/// Returns `None` when the fact is clear. The show instance and the private
/// instance come from the row. `HitIt` did not connect a stream or start a
/// recording in this run, so the parts have none.
pub(super) fn started_parts_from_store(conn: &Connection) -> Option<StartedParts> {
    let row = stored::read(conn).ok().flatten()?;
    row.private_stopped.then(|| StartedParts {
        show_instance: Some(row.show_instance),
        private_stopped: Some(row.private_instance),
        ..StartedParts::default()
    })
}

/// One step of a show that Mixxx plays, with the inputs that it needs.
pub(super) enum MixxxStep {
    Switch {
        private_instance: String,
        show_instance: String,
        private_was_active: bool,
    },
    StartRecording {
        target: EncoderTarget,
        private_instance: String,
        show_instance: String,
    },
    StopRecording {
        target: EncoderTarget,
    },
    StopShowUnit {
        private_instance: String,
        show_instance: String,
        unit: ShowUnit,
    },
    StartPrivateProducer {
        private_instance: String,
    },
}

/// Command that runs one `MixxxStep` off the UI thread.
pub(super) struct MixxxStepCommand {
    pub(super) conn: Arc<Mutex<Connection>>,
    pub(super) transport: Transport,
    pub(super) step: MixxxStep,
}

/// Result of one step.
pub(super) struct MixxxStepDone {
    pub(super) result: Result<(), String>,
    /// Wall clock start of a recording that this step started.
    pub(super) recording_started_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl MixxxStepDone {
    pub(super) fn failed(error: String) -> Self {
        Self::of(Err(error))
    }

    fn of(result: Result<(), String>) -> Self {
        Self {
            result,
            recording_started_at: None,
        }
    }
}

impl ApplicationCommand for MixxxStepCommand {
    type Output = MixxxStepDone;

    fn execute(self, context: &CommandContext) -> crate::application::CommandResult<Self::Output> {
        if context.cancellation().is_cancelled() {
            return Err(CommandError::Cancelled);
        }
        let conn = self
            .conn
            .lock()
            .map_err(|_| CommandError::Other("database lock poisoned".to_owned()))?;
        let control = ServiceControl::default();
        let done = match &self.step {
            MixxxStep::Switch {
                private_instance,
                show_instance,
                private_was_active,
            } => match switch_to_show(
                &conn,
                &control,
                &self.transport,
                private_instance,
                show_instance,
                *private_was_active,
            ) {
                Ok(_) => MixxxStepDone::of(Ok(())),
                Err(error) => MixxxStepDone::of(Err(error)),
            },
            MixxxStep::StartRecording {
                target,
                private_instance,
                show_instance,
            } => match encoder::start_recording(target) {
                Ok(()) => {
                    let at = chrono::Utc::now();
                    let stored_result = stored::store_recording_started(
                        &conn,
                        show_instance,
                        private_instance,
                        &at.to_rfc3339(),
                    );
                    MixxxStepDone {
                        recording_started_at: Some(at),
                        ..MixxxStepDone::of(stored_result.map_err(|error| format!("{error:#}")))
                    }
                }
                Err(error) => MixxxStepDone::of(Err(format!("Start the recording: {error:#}"))),
            },
            MixxxStep::StopRecording { target } => MixxxStepDone::of(
                encoder::stop_recording(target)
                    .map_err(|error| format!("Stop the recording: {error:#}")),
            ),
            MixxxStep::StopShowUnit {
                private_instance,
                show_instance,
                unit,
            } => MixxxStepDone::of(stop_show_unit(
                &control,
                &self.transport,
                private_instance,
                show_instance,
                *unit,
            )),
            MixxxStep::StartPrivateProducer { private_instance } => MixxxStepDone::of(
                start_private_producer(&conn, &control, &self.transport, private_instance),
            ),
        };
        Ok(CommandOutcome::without_events(done))
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use anyhow::Result;

    use crate::broadcast::control::CommandOutput;
    use crate::view_models::show::{EndShowSequence, EndShowStep};

    use super::*;

    type Calls = Rc<RefCell<Vec<String>>>;

    /// A fake runner. It records each call. The producer of `fail_start` does not start.
    struct Fake {
        calls: Calls,
        fail_start: Option<&'static str>,
        exit_status: i32,
    }

    impl Fake {
        fn new(fail_start: Option<&'static str>) -> (Self, Calls) {
            let calls = Calls::default();
            (
                Self {
                    calls: Rc::clone(&calls),
                    fail_start,
                    exit_status: 1,
                },
                calls,
            )
        }
    }

    impl CommandRunner for Fake {
        fn run(&self, _program: &str, args: &[String]) -> Result<CommandOutput> {
            let line = args.join(" ");
            self.calls.borrow_mut().push(line.clone());
            if line.contains("ExecMainStatus") {
                return Ok(CommandOutput::success(format!(
                    "ExecMainStatus={}\n",
                    self.exit_status
                )));
            }
            if line.contains("list-units") {
                return Ok(CommandOutput::success(""));
            }
            if args[1] == "show" {
                return Ok(CommandOutput::success(
                    "LoadState=loaded\nActiveState=active\n",
                ));
            }
            if args[1] == "start" && self.fail_start.is_some_and(|name| line.contains(name)) {
                return Ok(CommandOutput::failure(Some(1), "", "start failed"));
            }
            Ok(CommandOutput::success(""))
        }
    }

    fn conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        stored::apply(&conn).unwrap();
        conn
    }

    fn private_stopped(conn: &Connection) -> bool {
        stored::read(conn)
            .unwrap()
            .is_some_and(|row| row.private_stopped)
    }

    /// The fact is stored before the private producer stops and stays after a good switch.
    #[test]
    fn adr_0111_switch_stores_the_fact_before_it_stops_the_private_producer() {
        struct Probe<'a> {
            inner: Fake,
            conn: &'a Connection,
            seen: Rc<RefCell<Option<bool>>>,
        }
        impl CommandRunner for Probe<'_> {
            fn run(&self, program: &str, args: &[String]) -> Result<CommandOutput> {
                if args[1] == "stop" && self.seen.borrow().is_none() {
                    *self.seen.borrow_mut() = Some(private_stopped(self.conn));
                }
                self.inner.run(program, args)
            }
        }
        let conn = conn();
        let (runner, calls) = Fake::new(None);
        let seen_before_stop = Rc::new(RefCell::new(None));
        // The first call of the switch is the stop. A second runner reads the row at that time.
        let probe = Probe {
            inner: runner,
            conn: &conn,
            seen: Rc::clone(&seen_before_stop),
        };
        let stopped = switch_to_show(
            &conn,
            &ServiceControl::new(probe),
            &Transport::local(),
            "mixxx",
            "show-1",
            true,
        )
        .unwrap();
        assert!(stopped);
        assert_eq!(*seen_before_stop.borrow(), Some(true));
        assert!(private_stopped(&conn));
        assert!(calls.borrow()[0].contains("stop mixxx-now-playing@mixxx.service"));
    }

    /// Mechanical criterion 4: a failed switch with a start of the private
    /// producer clears the stored fact.
    #[test]
    fn adr_0111_failed_switch_with_a_restart_clears_the_fact() {
        let conn = conn();
        let (runner, calls) = Fake::new(Some("show-1"));
        let error = switch_to_show(
            &conn,
            &ServiceControl::new(runner),
            &Transport::local(),
            "mixxx",
            "show-1",
            true,
        )
        .unwrap_err();
        assert!(error.contains("start failed"), "{error}");
        assert!(!private_stopped(&conn), "the fact is clear");
        let calls = calls.borrow();
        assert!(calls
            .iter()
            .any(|call| call == "--user start mixxx-now-playing@mixxx.service"));
    }

    /// A failed switch where the private producer does not start again keeps the fact.
    #[test]
    fn adr_0111_failed_switch_without_a_restart_keeps_the_fact() {
        let conn = conn();
        let (runner, _) = Fake::new(Some("service"));
        let error = switch_to_show(
            &conn,
            &ServiceControl::new(runner),
            &Transport::local(),
            "mixxx",
            "show-1",
            true,
        )
        .unwrap_err();
        assert!(error.contains("did not start again"), "{error}");
        assert!(private_stopped(&conn), "End show can still start it");
    }

    /// A private producer that was not active gives no fact.
    #[test]
    fn adr_0111_switch_with_an_inactive_private_producer_stores_no_fact() {
        let conn = conn();
        let (runner, _) = Fake::new(None);
        let stopped = switch_to_show(
            &conn,
            &ServiceControl::new(runner),
            &Transport::local(),
            "mixxx",
            "show-1",
            false,
        )
        .unwrap();
        assert!(!stopped);
        assert!(!private_stopped(&conn));
    }

    /// ADR 0111 criterion 11: after a restart, the model made from the stored
    /// state gives the start of the private producer, and the fact is clear
    /// after that step.
    #[test]
    fn adr_0111_end_show_after_a_restart_starts_the_private_producer_and_clears_the_fact() {
        let conn = conn();
        stored::store_private_stopped(&conn, "show-1", "mixxx").unwrap();
        let parts = started_parts_from_store(&conn).expect("the fact is set");
        let mut sequence = EndShowSequence::for_mixxx(&parts, false);
        let mut steps = Vec::new();
        while let Some(step) = sequence.next_step() {
            steps.push(step);
            sequence.complete(Ok(()));
        }
        assert_eq!(
            steps,
            [
                EndShowStep::StopShowProducer,
                EndShowStep::StopShowPublisher,
                EndShowStep::StartPrivateProducer,
            ]
        );
        let (runner, calls) = Fake::new(None);
        start_private_producer(
            &conn,
            &ServiceControl::new(runner),
            &Transport::local(),
            "mixxx",
        )
        .unwrap();
        assert_eq!(
            calls.borrow().as_slice(),
            ["--user start mixxx-now-playing@mixxx.service"]
        );
        assert!(!private_stopped(&conn));
        assert!(started_parts_from_store(&conn).is_none());
    }

    /// A failed start of the private producer keeps the fact.
    #[test]
    fn adr_0111_failed_private_start_keeps_the_fact() {
        let conn = conn();
        stored::store_private_stopped(&conn, "show-1", "mixxx").unwrap();
        let (runner, _) = Fake::new(Some("mixxx"));
        assert!(start_private_producer(
            &conn,
            &ServiceControl::new(runner),
            &Transport::local(),
            "mixxx"
        )
        .is_err());
        assert!(private_stopped(&conn));
    }

    /// ADR 0111 Decision 4: End show never stops a unit of the private instance.
    #[test]
    fn adr_0111_end_show_never_stops_the_private_publisher() {
        let (runner, calls) = Fake::new(None);
        let error = stop_show_unit(
            &ServiceControl::new(runner),
            &Transport::local(),
            "mixxx",
            "mixxx",
            ShowUnit::Publisher,
        )
        .unwrap_err();
        assert!(error.contains("ADR 0111 Decision 4"), "{error}");
        assert!(calls.borrow().is_empty(), "no command may run");
    }
}
