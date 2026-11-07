//! One background load at a time, with one "run again" flag.
//!
//! The Library starts a load from many events. A load that is already in
//! flight does not start a second one. Instead it sets one run-again flag,
//! and the flag starts one more run when the first run ends. Each run has a
//! number, so a result of an old run never replaces a newer state
//! (ADR 0084 Decision 10 as amended 2026-10-24, task 039, steps 3 and 4).

#![warn(clippy::pedantic)]

/// One started run of a load.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Run {
    number: u64,
    generation: u64,
}

/// What a finished run means for the state of the view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Finish {
    /// The run is not the current run. Its result must not be applied.
    Stale,
    /// The run is current, and no run is waiting. Apply the result.
    Done,
    /// The run is current, and a request arrived during it. Apply the
    /// result, then call [`SingleFlight::request`] to start the waiting run.
    RunAgain,
}

/// The state of one load: whether a run is in flight, and whether one more
/// run is waiting.
#[derive(Debug, Default)]
pub(crate) struct SingleFlight {
    running: Option<Run>,
    run_again: bool,
    last_number: u64,
}

impl SingleFlight {
    /// Creates a load with no run.
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self {
            running: None,
            run_again: false,
            last_number: 0,
        }
    }

    /// Asks for a run in `generation` of the session.
    ///
    /// When no run is in flight, this starts one and returns it. When a run
    /// is in flight and `is_live` accepts its generation, the request sets the
    /// run-again flag and returns `None`. When `is_live` refuses the in-flight
    /// generation, its result never arrives, so this drops it and starts a
    /// new run.
    pub(crate) fn request(
        &mut self,
        generation: u64,
        is_live: impl Fn(u64) -> bool,
    ) -> Option<Run> {
        if let Some(run) = self.running {
            if is_live(run.generation) {
                self.run_again = true;
                return None;
            }
            self.running = None;
            self.run_again = false;
        }
        self.last_number += 1;
        let run = Run {
            number: self.last_number,
            generation,
        };
        self.running = Some(run);
        Some(run)
    }

    /// Records the end of `run`. A run that is not the current run gives
    /// [`Finish::Stale`].
    pub(crate) fn finish(&mut self, run: Run) -> Finish {
        if self.running != Some(run) {
            return Finish::Stale;
        }
        self.running = None;
        if std::mem::take(&mut self.run_again) {
            Finish::RunAgain
        } else {
            Finish::Done
        }
    }

    /// True while a run is in flight.
    #[cfg(test)]
    pub(crate) const fn is_running(&self) -> bool {
        self.running.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::{Finish, SingleFlight};

    /// Task 039, step 4 and step 3. A request during a run sets one flag.
    /// Ten requests during a run still start only one more run.
    #[test]
    fn adr_0084_task_039_burst_of_requests_gives_one_more_run() {
        let mut flight = SingleFlight::new();
        let first = flight
            .request(1, |_| true)
            .expect("the first request starts a run");
        for _ in 0..10 {
            assert!(flight.request(1, |_| true).is_none(), "a run is in flight");
        }

        assert_eq!(flight.finish(first), Finish::RunAgain);
        let second = flight
            .request(1, |_| true)
            .expect("the run-again flag starts one run");
        assert_eq!(flight.finish(second), Finish::Done);
        assert!(!flight.is_running());
    }

    /// Task 039, step 3. Each run ends once, and the result of a run that
    /// already ended is stale.
    #[test]
    fn adr_0084_task_039_only_the_current_run_is_applied() {
        let mut flight = SingleFlight::new();
        let first = flight.request(1, |_| true).expect("a run starts");
        assert_eq!(flight.finish(first), Finish::Done);
        assert_eq!(
            flight.finish(first),
            Finish::Stale,
            "a second end of the same run is stale"
        );
    }

    /// Task 039, step 3. When the session of the in-flight run is gone, its
    /// result never arrives. The next request starts a run, and a late end of
    /// the old run cannot overwrite the newer state.
    #[test]
    fn adr_0084_task_039_a_run_of_a_gone_session_never_overwrites_a_newer_one() {
        let mut flight = SingleFlight::new();
        let old = flight.request(1, |_| true).expect("a run starts");

        let new = flight
            .request(2, |generation| generation == 2)
            .expect("the session of the old run is gone, so a new run starts");

        assert_eq!(flight.finish(old), Finish::Stale, "the old run is stale");
        assert_eq!(flight.finish(new), Finish::Done);
    }

    /// Task 039, step 4. Requests from a session that still runs only set
    /// the flag, and a run that is current stays current until it ends.
    #[test]
    fn adr_0084_task_039_requests_during_a_live_run_keep_it_current() {
        let mut flight = SingleFlight::new();
        let run = flight.request(7, |_| true).expect("a run starts");
        assert!(flight.request(7, |generation| generation == 7).is_none());
        assert_eq!(
            flight.finish(run),
            Finish::RunAgain,
            "the last state is not lost"
        );
    }
}
