//! Broadcast readiness watch actor for ADR 0059.
//!
//! The actor owns the blocking local file scan and publishes cached snapshots
//! so Show renders payment-route readiness without touching the filesystem.

#![warn(clippy::pedantic)]

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::application::session_lifecycle::SessionLifecycle;
use rusqlite::Connection;

use tokio::sync::{mpsc, oneshot, watch};

use crate::application::queries::broadcast::{
    broadcast_readiness_plan_from_config, BroadcastReadinessPlan, BroadcastReadinessReport,
};

const BROADCAST_READINESS_POLL_INTERVAL: Duration = Duration::from_mins(5);
const INBOX_CAPACITY: usize = 4;

/// Plans one readiness scan under the database lock. The app reads the music
/// directory from the configuration. A test gives a fixed directory, so it
/// does not read the configuration of the person who runs it.
type PlanSource = fn(&Connection) -> anyhow::Result<BroadcastReadinessPlan>;

/// Cached local broadcast-readiness snapshot.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct BroadcastReadinessSnapshot {
    /// Latest successful readiness report.
    pub(crate) report: Option<BroadcastReadinessReport>,
    /// Latest scan error, when the report could not be refreshed.
    pub(crate) error: Option<String>,
}

/// Caller-side handle for the broadcast readiness actor.
pub(crate) struct BroadcastReadinessWatchHandle {
    snapshot: watch::Receiver<BroadcastReadinessSnapshot>,
    inbox: mpsc::Sender<BroadcastReadinessCommand>,
    shutdown: Option<oneshot::Sender<()>>,
}

impl BroadcastReadinessWatchHandle {
    /// Subscribe to readiness snapshots.
    #[must_use]
    pub(crate) fn subscribe(&self) -> watch::Receiver<BroadcastReadinessSnapshot> {
        self.snapshot.clone()
    }

    /// Return the latest cached snapshot.
    #[must_use]
    pub(crate) fn latest(&self) -> BroadcastReadinessSnapshot {
        self.snapshot.borrow().clone()
    }

    /// Request an immediate readiness scan.
    ///
    /// Returns `false` when the actor has shut down or its inbox is full.
    #[must_use]
    pub(crate) fn refresh_now(&self) -> bool {
        self.inbox
            .try_send(BroadcastReadinessCommand::RefreshNow)
            .is_ok()
    }
}

impl Drop for BroadcastReadinessWatchHandle {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BroadcastReadinessCommand {
    RefreshNow,
}

/// Spawns the broadcast readiness actor on the current tokio runtime.
#[must_use]
pub(crate) fn start(
    conn: Arc<Mutex<Connection>>,
    session: &SessionLifecycle,
) -> BroadcastReadinessWatchHandle {
    start_with_interval(
        conn,
        BROADCAST_READINESS_POLL_INTERVAL,
        session,
        broadcast_readiness_plan_from_config,
    )
}

fn start_with_interval(
    conn: Arc<Mutex<Connection>>,
    interval: Duration,
    session: &SessionLifecycle,
    plan_source: PlanSource,
) -> BroadcastReadinessWatchHandle {
    let (snapshot_tx, snapshot_rx) = watch::channel(BroadcastReadinessSnapshot::default());
    let (inbox_tx, mut inbox_rx) = mpsc::channel::<BroadcastReadinessCommand>(INBOX_CAPACITY);
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

    let stop = session.stop_token();
    session.spawn_actor("Library readiness scan", async move {
        publish_snapshot(&snapshot_tx, Arc::clone(&conn), plan_source).await;
        loop {
            tokio::select! {
                biased;
                () = stop.cancelled() => break,
                _ = &mut shutdown_rx => break,
                command = inbox_rx.recv() => {
                    let Some(BroadcastReadinessCommand::RefreshNow) = command else {
                        break;
                    };
                    publish_snapshot(&snapshot_tx, Arc::clone(&conn), plan_source).await;
                }
                () = tokio::time::sleep(interval) => {
                    publish_snapshot(&snapshot_tx, Arc::clone(&conn), plan_source).await;
                }
            }
        }
    });

    BroadcastReadinessWatchHandle {
        snapshot: snapshot_rx,
        inbox: inbox_tx,
        shutdown: Some(shutdown_tx),
    }
}

async fn publish_snapshot(
    snapshot_tx: &watch::Sender<BroadcastReadinessSnapshot>,
    conn: Arc<Mutex<Connection>>,
    plan_source: PlanSource,
) {
    let snapshot = read_broadcast_readiness(conn, plan_source)
        .await
        .unwrap_or_else(|error| BroadcastReadinessSnapshot {
            report: None,
            error: Some(format!("{error:#}")),
        });
    let _ = snapshot_tx.send(snapshot);
}

async fn read_broadcast_readiness(
    conn: Arc<Mutex<Connection>>,
    plan_source: PlanSource,
) -> Result<BroadcastReadinessSnapshot, tokio::task::JoinError> {
    tokio::task::spawn_blocking(move || read_broadcast_readiness_blocking(&conn, plan_source)).await
}

/// Reads the report. The rows and the stored facts are read under the
/// database lock. The file tags are read after the lock is released
/// (ADR 0084 Decision 10 as amended 2026-10-24, task 039, step 1).
fn read_broadcast_readiness_blocking(
    conn: &Arc<Mutex<Connection>>,
    plan_source: PlanSource,
) -> BroadcastReadinessSnapshot {
    let report = conn
        .lock()
        .map_err(|_| "database lock poisoned".to_owned())
        .and_then(|conn| plan_source(&conn).map_err(|error| format!("{error:#}")))
        .map(BroadcastReadinessPlan::read_files);
    match report {
        Ok(report) => BroadcastReadinessSnapshot {
            report: Some(report),
            error: None,
        },
        Err(error) => BroadcastReadinessSnapshot {
            report: None,
            error: Some(error),
        },
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use tokio::time::timeout;

    use crate::db;

    use super::*;

    fn setup_test_db() -> anyhow::Result<Arc<Mutex<Connection>>> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        db::init_schema(&conn)?;
        db::migrate_schema(&conn)?;
        Ok(Arc::new(Mutex::new(conn)))
    }

    fn fixed_music_dir_plan(conn: &Connection) -> anyhow::Result<BroadcastReadinessPlan> {
        crate::application::queries::broadcast::broadcast_readiness_plan(
            conn,
            std::path::Path::new("/nonexistent-hitit-music"),
        )
    }

    #[tokio::test(flavor = "current_thread")]
    async fn broadcast_readiness_actor_publishes_cached_report() -> anyhow::Result<()> {
        let conn = setup_test_db()?;

        let snapshot = read_broadcast_readiness(Arc::clone(&conn), fixed_music_dir_plan).await?;

        assert_eq!(
            snapshot.report.expect("report").summary,
            crate::application::queries::broadcast::BroadcastReadinessSummary::default()
        );
        assert!(snapshot.error.is_none());
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn broadcast_readiness_actor_refreshes_on_command() -> anyhow::Result<()> {
        let conn = setup_test_db()?;
        let handle = start_with_interval(
            Arc::clone(&conn),
            Duration::from_mins(1),
            &SessionLifecycle::new(),
            fixed_music_dir_plan,
        );
        let mut receiver = handle.subscribe();

        wait_for_report(&mut receiver).await;

        assert!(handle.refresh_now());
        wait_for_report(&mut receiver).await;
        drop(handle);
        Ok(())
    }

    async fn wait_for_report(receiver: &mut watch::Receiver<BroadcastReadinessSnapshot>) {
        loop {
            if receiver.borrow().report.is_some() {
                return;
            }
            // The limit only bounds a failing test. A parallel run of the full
            // suite can delay the actor for more than one second.
            timeout(Duration::from_secs(10), receiver.changed())
                .await
                .expect("actor should publish before timeout")
                .expect("actor should keep the watch channel open");
        }
    }
}
