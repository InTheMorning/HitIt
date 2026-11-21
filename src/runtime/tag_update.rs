//! Tag update actor for ADR 0076 Decision 8 (packet 004).
//!
//! The actor owns the difference scan and the confirmed tag write. Both read
//! files, so both run on the blocking pool of the ADR 0040 runtime and never
//! on the render thread. The actor publishes a snapshot that the Music
//! section reads through its view model.
//!
//! After a confirm, the actor sends `VmEvent::TrackChanged` for each written
//! track, and then it scans again. The new count includes each file that
//! the show played or held.

#![warn(clippy::pedantic)]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;
use tokio::sync::{mpsc, watch};

use crate::application::commands::tag_update::{
    delete_stale_skips, record_skips, write_tag_updates, TagUpdateWriteReport,
};
use crate::application::queries::tag_update::{
    compare_planned_files, plan_tag_update_scan, TagUpdateFile, TagUpdateScan,
};
use crate::application::session_lifecycle::SessionLifecycle;
use crate::db::tag_update_skips::TagUpdateSkip;
use crate::runtime::vm_bus::{VmBus, VmEvent};

const INBOX_CAPACITY: usize = 8;

/// The latest scan and the latest confirm of the actor.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct TagUpdateSnapshot {
    /// The latest completed scan.
    pub(crate) scan: Option<TagUpdateScan>,
    /// The error of the latest scan, when it failed.
    pub(crate) scan_error: Option<String>,
    /// `true` while a scan runs.
    pub(crate) scanning: bool,
    /// `true` while a confirm writes files.
    pub(crate) writing: bool,
    /// The result of the latest confirm.
    pub(crate) last_write: Option<TagUpdateWriteReport>,
    /// The error of the latest confirm, when it wrote no file.
    pub(crate) write_error: Option<String>,
    /// The count of the confirms that ended. Each ended confirm adds one,
    /// so a reader that misses a snapshot still finds each write end
    /// (ADR 0101 Decision 7).
    pub(crate) writes_ended: u64,
}

#[derive(Clone, Debug)]
enum TagUpdateMessage {
    Scan {
        music_dir: PathBuf,
    },
    Confirm {
        music_dir: PathBuf,
        files: Vec<TagUpdateFile>,
        skips: Vec<TagUpdateSkip>,
    },
}

/// Caller-side handle of the actor.
#[derive(Clone, Debug)]
pub(crate) struct TagUpdateHandle {
    inbox: mpsc::Sender<TagUpdateMessage>,
    snapshot: watch::Receiver<TagUpdateSnapshot>,
}

impl TagUpdateHandle {
    /// Request a scan. Returns `false` when the actor stopped or its inbox
    /// is full.
    #[must_use]
    pub(crate) fn scan(&self, music_dir: PathBuf) -> bool {
        self.inbox
            .try_send(TagUpdateMessage::Scan { music_dir })
            .is_ok()
    }

    /// Write the listed files, and then scan again. Returns `false` when
    /// the actor stopped or its inbox is full.
    #[must_use]
    pub(crate) fn confirm(
        &self,
        music_dir: PathBuf,
        files: Vec<TagUpdateFile>,
        skips: Vec<TagUpdateSkip>,
    ) -> bool {
        self.inbox
            .try_send(TagUpdateMessage::Confirm {
                music_dir,
                files,
                skips,
            })
            .is_ok()
    }

    /// Subscribe to snapshots.
    #[must_use]
    pub(crate) fn subscribe(&self) -> watch::Receiver<TagUpdateSnapshot> {
        self.snapshot.clone()
    }

    /// The latest snapshot.
    #[must_use]
    pub(crate) fn latest(&self) -> TagUpdateSnapshot {
        self.snapshot.borrow().clone()
    }
}

/// Spawn the actor on the current tokio runtime.
#[must_use]
pub(crate) fn spawn(
    conn: Arc<Mutex<Connection>>,
    bus: VmBus,
    session: &SessionLifecycle,
) -> TagUpdateHandle {
    let (snapshot_tx, snapshot_rx) = watch::channel(TagUpdateSnapshot::default());
    let (inbox_tx, mut inbox_rx) = mpsc::channel::<TagUpdateMessage>(INBOX_CAPACITY);
    let stop = session.stop_token();
    session.spawn_actor("Library tag update", async move {
        loop {
            tokio::select! {
                biased;
                () = stop.cancelled() => break,
                message = inbox_rx.recv() => {
                    let Some(message) = message else {
                        break;
                    };
                    handle(&conn, &bus, &snapshot_tx, message).await;
                }
            }
        }
    });
    TagUpdateHandle {
        inbox: inbox_tx,
        snapshot: snapshot_rx,
    }
}

async fn handle(
    conn: &Arc<Mutex<Connection>>,
    bus: &VmBus,
    snapshot_tx: &watch::Sender<TagUpdateSnapshot>,
    message: TagUpdateMessage,
) {
    match message {
        TagUpdateMessage::Scan { music_dir } => scan(conn, snapshot_tx, music_dir).await,
        TagUpdateMessage::Confirm {
            music_dir,
            files,
            skips,
        } => {
            snapshot_tx.send_modify(|snapshot| {
                snapshot.writing = true;
                snapshot.write_error = None;
            });
            let conn_for_skips = Arc::clone(conn);
            let recorded = tokio::task::spawn_blocking(move || {
                record_skips(&conn_for_skips, &skips).map_err(|error| format!("{error:#}"))
            })
            .await
            .unwrap_or_else(|error| Err(format!("The tag skip record stopped: {error}")));
            if let Err(error) = recorded {
                snapshot_tx.send_modify(|snapshot| {
                    snapshot.writing = false;
                    snapshot.scanning = true;
                    snapshot.writes_ended += 1;
                    snapshot.write_error = Some(error);
                });
                scan(conn, snapshot_tx, music_dir).await;
                return;
            }
            let conn_for_write = Arc::clone(conn);
            let bus = bus.clone();
            let written = tokio::task::spawn_blocking(move || {
                write_tag_updates(&conn_for_write, &files, |track_id| {
                    bus.publish(VmEvent::TrackChanged { track_id });
                })
                .map_err(|error| format!("{error:#}"))
            })
            .await
            .unwrap_or_else(|error| Err(format!("The tag write stopped: {error}")));
            // The rescan starts in the same snapshot that ends the write. A
            // snapshot with neither flag set would show the count of the
            // earlier scan after the files changed.
            snapshot_tx.send_modify(|snapshot| {
                snapshot.writing = false;
                snapshot.scanning = true;
                snapshot.writes_ended += 1;
                match written {
                    Ok(report) => snapshot.last_write = Some(report),
                    Err(error) => snapshot.write_error = Some(error),
                }
            });
            scan(conn, snapshot_tx, music_dir).await;
        }
    }
}

async fn scan(
    conn: &Arc<Mutex<Connection>>,
    snapshot_tx: &watch::Sender<TagUpdateSnapshot>,
    music_dir: PathBuf,
) {
    snapshot_tx.send_modify(|snapshot| snapshot.scanning = true);
    let conn = Arc::clone(conn);
    let result = tokio::task::spawn_blocking(move || {
        // The database lock is held only for the plan step. The file reads
        // of the compare step hold no lock.
        let planned = {
            let conn = conn
                .lock()
                .map_err(|_| "The database lock is not available.".to_owned())?;
            plan_tag_update_scan(&conn, &music_dir).map_err(|error| format!("{error:#}"))?
        };
        let compared = compare_planned_files(planned);
        let delete_error = delete_stale_skips(&conn, &compared.stale_skips)
            .err()
            .map(|error| format!("{error:#}"));
        Ok::<_, String>((compared, delete_error))
    })
    .await
    .unwrap_or_else(|error| Err(format!("The tag scan stopped: {error}")));
    snapshot_tx.send_modify(|snapshot| {
        snapshot.scanning = false;
        match result {
            Ok((scan, delete_error)) => {
                snapshot.scan = Some(scan);
                snapshot.scan_error = delete_error;
            }
            Err(error) => snapshot.scan_error = Some(error),
        }
    });
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::time::timeout;

    use super::*;
    use crate::application::queries::tag_update::test_support::library;

    async fn wait_for(
        receiver: &mut watch::Receiver<TagUpdateSnapshot>,
        done: impl Fn(&TagUpdateSnapshot) -> bool,
    ) -> TagUpdateSnapshot {
        loop {
            {
                let snapshot = receiver.borrow_and_update();
                if done(&snapshot) {
                    return snapshot.clone();
                }
            }
            timeout(Duration::from_secs(5), receiver.changed())
                .await
                .expect("the actor publishes before the timeout")
                .expect("the actor keeps the watch channel open");
        }
    }

    /// R4-10: the scan runs in the runtime actor on the blocking pool. A
    /// confirm writes the files, sends `VmEvent::TrackChanged` for each
    /// written track, and then scans again.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn adr_0108_tag_update_actor_records_skip_and_writes_selected_file() {
        let dir = tempfile::tempdir().unwrap();
        let conn = Arc::new(Mutex::new(library(dir.path(), &[1, 2])));
        let bus = VmBus::new();
        let mut events = bus.subscribe();
        let session = SessionLifecycle::new();
        let handle = spawn(Arc::clone(&conn), bus.clone(), &session);
        let mut receiver = handle.subscribe();

        assert!(handle.scan(dir.path().to_path_buf()));
        let scanned = wait_for(&mut receiver, |snapshot| {
            snapshot.scan.is_some() && !snapshot.scanning
        })
        .await;
        let files = scanned.scan.unwrap().files;
        assert_eq!(files.len(), 2);
        let track_two = files.iter().find(|file| file.track_id == 2).unwrap();
        let settling_edits = track_two
            .edits()
            .into_iter()
            .filter(|edit| edit.frame_label != "TIT2")
            .collect::<Vec<_>>();
        crate::audio_tags::write_id3v24_edits(&track_two.path, &settling_edits).unwrap();
        crate::audio_tags::write_id3v24_edits(
            &track_two.path,
            &[crate::audio_tags::Id3v24Edit {
                frame_label: "TIT2".into(),
                value: "Old title".into(),
            }],
        )
        .unwrap();
        let files = crate::application::queries::tag_update::scan_tag_updates(
            &conn.lock().unwrap(),
            dir.path(),
        )
        .unwrap()
        .files;
        let track_two = files.iter().find(|file| file.track_id == 2).unwrap();
        let stored_title = match &track_two.content {
            crate::application::queries::tag_update::TagUpdateFileContent::Differs {
                frames,
                ..
            } => frames
                .iter()
                .find(|frame| frame.frame_label == "TIT2")
                .unwrap()
                .expected_value
                .clone(),
            crate::application::queries::tag_update::TagUpdateFileContent::Unreadable {
                ..
            } => unreachable!(),
        };
        let skip = TagUpdateSkip {
            track_id: 2,
            frame_label: "TIT2".into(),
            stored_value: stored_title,
        };

        assert!(handle.confirm(
            dir.path().to_path_buf(),
            vec![files[0].clone()],
            vec![skip.clone()]
        ));
        let confirmed = wait_for(&mut receiver, |snapshot| {
            snapshot.last_write.is_some() && !snapshot.writing && !snapshot.scanning
        })
        .await;

        assert_eq!(confirmed.last_write.unwrap().written_count(), 1);
        // ADR 0101 Decision 7: each ended confirm adds one to the count.
        assert_eq!(scanned.writes_ended, 0);
        assert_eq!(confirmed.writes_ended, 1);
        let final_scan = confirmed.scan.unwrap();
        assert_eq!(final_scan.count(), 0);
        assert_eq!(final_scan.skip_count(), 1);
        assert_eq!(
            crate::db::tag_update_skips::tag_update_skips(&conn.lock().unwrap()).unwrap(),
            vec![skip]
        );
        let mut changed = Vec::new();
        while let Ok(VmEvent::TrackChanged { track_id }) = events.try_recv() {
            changed.push(track_id);
        }
        assert_eq!(changed, vec![1]);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn adr_0108_tag_update_actor_deletes_stale_skips_after_scan() {
        let dir = tempfile::tempdir().unwrap();
        let conn = Arc::new(Mutex::new(library(dir.path(), &[1])));
        let stale = TagUpdateSkip {
            track_id: 1,
            frame_label: "TIT2".into(),
            stored_value: "Old stored title".into(),
        };
        crate::db::tag_update_skips::record_tag_update_skips(
            &conn.lock().unwrap(),
            std::slice::from_ref(&stale),
        )
        .unwrap();
        let session = SessionLifecycle::new();
        let handle = spawn(Arc::clone(&conn), VmBus::new(), &session);
        let mut receiver = handle.subscribe();

        assert!(handle.scan(dir.path().to_path_buf()));
        let snapshot = wait_for(&mut receiver, |snapshot| {
            snapshot.scan.is_some() && !snapshot.scanning
        })
        .await;

        assert!(snapshot.scan_error.is_none());
        assert!(
            crate::db::tag_update_skips::tag_update_skips(&conn.lock().unwrap())
                .unwrap()
                .is_empty()
        );
    }
}
