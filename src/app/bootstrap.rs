//! Desktop composition and one startup window lifecycle (ADRs 0040, 0066).

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::Duration;

use gpui::{size, AppContext, Bounds, Context, Entity, Window, WindowBounds, WindowOptions};
use gpui_component::Root;

use crate::application::capability::{
    CapabilityFailure, CapabilityObservation, CapabilityObservations, Dependency,
};
use crate::config;
use crate::media::ImageCache;
use crate::playback_driver::ConfiguredPlaybackDriver;
use crate::playback_owner::PlaybackOwner;
use crate::presentation::startup_presenter::{
    quit_after_window_close, window_disposition, WindowDisposition,
};
use crate::presentation::{
    maintenance_executor::{MaintenanceClient, MaintenanceWorker},
    RuntimeHost,
};
use crate::startup::{CoreCheckOutcome, PreparedCore, StartupIssue, StartupStage};
use crate::ui::layouts as layout;
use crate::view_models::startup::{format_report, normal_startup_status};

use super::{
    keyboard::install_key_bindings, menu::install_app_menu, startup::StartupScreen, TopApp,
};

/// Run one desktop session. False means recovery closed before normal startup.
///
/// Optional constructor failures remain scoped to their dependent operations.
/// Programmer invariants are not caught or converted into recovery results.
#[must_use]
pub fn run_app() -> bool {
    let worker = MaintenanceWorker::start();
    let client = worker.as_ref().ok().map(|worker| worker.client.clone());
    let opened = Arc::new(AtomicBool::new(false));
    let session_opened = opened.clone();
    // ADR 0101 Decision 10: this source serves the default `gpui-component`
    // icon bundle, and the extra Lucide icons `crate::ui::icons::IconName`
    // draws from the complete catalog. Registering only the default bundle
    // would leave those extra icons unresolved.
    let app = gpui_platform::application().with_assets(crate::ui::icons::InterfaceAssets);
    app.run(move |cx| {
        gpui_component::init(cx);
        crate::ui::primitives::primary_selection::init(cx);
        crate::ui::primitives::context_menu::init(cx);
        crate::ui::focus_ring::init(cx);
        install_key_bindings(cx);
        install_app_menu(cx);
        // ADR 0101 Decision 3: load the interface font before the first
        // window opens. An error here does not stop the app. GPUI falls
        // back to its own default font stack for a style that names
        // Figtree.
        if let Err(error) = crate::ui::fonts::install(cx) {
            eprintln!("Figtree fonts are not available. The app continues with the system font. {error:#}");
        }
        // Pre-config: install with default scale; ADR 0066 recovery requires
        // Dark/Medium before configuration can be decoded.
        crate::ui::theme_bridge::install_theme(
            crate::theme_profile::ThemeProfile::Dark,
            crate::ui::tokens::ScaleFactor::Medium,
            cx,
        );
        watch_accessibility_settings(cx);
        let window = cx.open_window(
            main_window_options(Bounds::centered(
                None,
                size(layout::WINDOW_WIDTH, layout::WINDOW_HEIGHT),
                cx,
            )),
            |window, cx| {
                window.on_window_should_close(cx, |_, cx| {
                    quit_after_window_close(cx);
                    true
                });
                let view = cx.new(|cx| {
                    let mut view = StartupScreen::new(client, session_opened);
                    view.begin(window, cx);
                    view
                });
                cx.new(|cx| Root::new(view, window, cx))
            },
        );
        if let Ok(window) = window {
            nudge_window(gpui::AnyWindowHandle::from(window), cx);
            if window
                .update(cx, |_, window, cx| {
                    window.activate_window();
                    window.refresh();
                    cx.activate(true);
                })
                .is_err()
            {
                let issue = StartupIssue::new(
                    StartupStage::Activation,
                    None,
                    "The desktop window was no longer available for activation.",
                    "Relaunch the app if the window was closed.",
                );
                eprintln!("{}", format_report(&CoreCheckOutcome::blocked(issue)));
                if window_disposition(true, !cx.windows().is_empty())
                    == WindowDisposition::ExitUnsuccessful
                {
                    cx.quit();
                }
            }
        } else {
            let issue = StartupIssue::new(
                StartupStage::Window,
                None,
                "The window system rejected the app's initial window. App did not open another error window.",
                "Check the desktop session and display environment, then relaunch the app.",
            );
            eprintln!("{}", format_report(&CoreCheckOutcome::blocked(issue)));
            cx.quit();
        }
    });
    if let Ok(worker) = worker {
        worker.finish();
    }
    opened.load(Ordering::Acquire)
}

pub(super) struct NormalStartup {
    preparation_receipt: crate::db::startup::PreparationReceipt,
    cfg: config::ConfigSnapshot,
    session: crate::application::session_lifecycle::SessionLifecycle,
    cfg_path: std::path::PathBuf,
    conn: Arc<Mutex<rusqlite::Connection>>,
    musicindex_endpoint: config::MusicIndexEndpoint,
    playback_owner: Option<Arc<Mutex<PlaybackOwner<ConfiguredPlaybackDriver>>>>,
    runtime_host: Option<Arc<RuntimeHost>>,
    image_cache: Arc<ImageCache>,
    audio_cache_dir: std::path::PathBuf,
    capability_observations: CapabilityObservations,
    notices: Vec<StartupIssue>,
}

/// Runs on the independent worker after core admission, never in a UI callback.
pub(super) fn prepare_normal(core: PreparedCore) -> Result<NormalStartup, CoreCheckOutcome> {
    let PreparedCore {
        config_path: cfg_path,
        snapshot,
        connection,
        preparation_receipt,
        notices,
    } = core;
    let cfg = snapshot;
    let audio_cache_dir = config::audio_cache_dir().map_err(|error| {
        CoreCheckOutcome::blocked(StartupIssue::new(
            StartupStage::ConfigField,
            None,
            format!("App could not determine the audio cache folder: {error:#}"),
            "Check the user cache directory, then relaunch the app.",
        ))
    })?;
    let capability_observations = CapabilityObservations::default();
    let music_dir = cfg
        .music_dir
        .as_ref()
        .expect("core admission verified music path");
    let db_path = cfg
        .db_path
        .as_ref()
        .expect("core admission verified database path");
    if crate::startup::prepare_local_paths(&connection, music_dir, db_path).map_err(
        |mut outcome| {
            outcome
                .preparation_receipts
                .push(preparation_receipt.clone());
            outcome
        },
    )? {
        capability_observations.record(
            CapabilityObservation::new(
                Dependency::LibraryPaths,
                Some(CapabilityFailure::PathRepair),
            )
            .at_path(db_path),
        );
    }
    crate::db::stop_playback_session_after_restart(
        &connection,
        crate::playback::DEFAULT_SESSION_ID,
    )
    .map_err(|error| {
        CoreCheckOutcome::blocked(StartupIssue::new(
            StartupStage::Database(crate::db::startup::DbStage::Open),
            Some(db_path),
            format!("Could not stop playback session after restart: {error:#}"),
            "Check the library database, then relaunch the app.",
        ))
    })?;
    let playback_owner =
        crate::startup::prepare_playback(&cfg, &cfg_path, &capability_observations)
            .map(|owner| Arc::new(Mutex::new(owner)));
    let musicindex_endpoint =
        config::MusicIndexEndpoint::from_field(cfg.musicindex_endpoint.clone());
    let conn = Arc::new(Mutex::new(connection));
    let session = crate::application::session_lifecycle::SessionLifecycle::new();
    let runtime = RuntimeHost::for_config(&cfg_path, session.clone());
    capability_observations.record(CapabilityObservation::new(
        Dependency::BackgroundRuntime,
        runtime
            .as_ref()
            .err()
            .map(|error| CapabilityFailure::RuntimeStart(error.kind())),
    ));
    let runtime_host = runtime.ok();
    let thumbnail_cache_dir = cfg_path
        .parent()
        .expect("config path has parent")
        .join("thumbnail-cache");
    let image_cache = ImageCache::new_observed(
        thumbnail_cache_dir,
        capability_observations.clone(),
        cache_worker_for_config(&cfg_path),
        session.clone(),
    );
    Ok(NormalStartup {
        preparation_receipt,
        cfg,
        session,
        cfg_path,
        conn,
        musicindex_endpoint,
        playback_owner,
        runtime_host,
        image_cache,
        audio_cache_dir,
        capability_observations,
        notices,
    })
}

pub(super) fn mount_normal<T: 'static>(
    prepared: NormalStartup,
    worker: Option<MaintenanceClient>,
    window: &mut Window,
    cx: &mut Context<T>,
) -> Entity<TopApp> {
    let NormalStartup {
        preparation_receipt,
        session,
        cfg,
        cfg_path,
        conn,
        musicindex_endpoint,
        playback_owner,
        runtime_host,
        image_cache,
        audio_cache_dir,
        capability_observations,
        notices,
    } = prepared;
    let workspace_layout_prefs = cfg
        .workspace_preferences()
        .and_then(|workspace| workspace.layout);
    let broadcast = cfg.broadcast();
    let theme_profile = cfg.theme_profile.unwrap_or_default();
    let ui_scale = cfg.ui_scale.unwrap_or_default();
    crate::ui::theme_bridge::install_theme_for_window(theme_profile, ui_scale.into(), window, cx);
    cx.new(|cx| {
        let mut app = TopApp::new(
            conn,
            image_cache,
            cfg_path,
            musicindex_endpoint,
            cfg.music_dir.expect("core admission verified music path"),
            audio_cache_dir,
            cfg.flac_path,
            broadcast,
            cfg.workspace_layout.unwrap_or_default(),
            workspace_layout_prefs.as_ref(),
            ui_scale,
            theme_profile,
            playback_owner,
            runtime_host,
            session,
            window,
            cx,
        );
        app.previous_session_report =
            crate::view_models::startup::preparation_report(&preparation_receipt);
        app.focus_active_tab(window, cx);
        app.install_capability_controls(capability_observations, worker, cx);
        app.maybe_start_broadcast_readiness_watch(cx);
        app.maybe_start_broadcast_service_watch(cx);
        app.refresh_show_page(cx);
        let status = normal_startup_status(&notices);
        if !status.is_empty() {
            app.settings_status = status;
        }
        app
    })
}

/// Called only during background preparation/retry, never by a renderer.
pub(super) fn cache_worker_for_config(
    path: &std::path::Path,
) -> crate::media::image_cache::CacheWorker {
    #[cfg(debug_assertions)]
    if crate::startup::fixture::injected_failure(path, Dependency::ThumbnailMaintenance) {
        return |_| Err(std::io::Error::other("fixture cache worker unavailable"));
    }
    #[cfg(not(debug_assertions))]
    let _ = path;
    crate::media::image_cache::spawn_cache_worker
}

/// ADR 0101 Decision 11: read the system accessibility settings, and install
/// the theme again when a setting changes while the app runs. The portal read
/// waits on the zbus runtime, not on the UI thread.
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn watch_accessibility_settings(cx: &mut gpui::App) {
    use crate::ui::system_settings::{read_or_default, AccessibilitySource, PortalSource};
    use crate::ui::theme_bridge::apply_accessibility_settings;

    // Presentation lifecycle: the watch lives as long as the app and has no
    // domain work, as `nudge_window` below.
    cx.spawn(async move |cx: &mut gpui::AsyncApp| {
        let mut source = match PortalSource::connect().await {
            Ok(source) => source,
            Err(error) => {
                eprintln!(
                    "The system accessibility settings are not available. HitIt uses no preference. {error:#}"
                );
                return;
            }
        };
        loop {
            let settings = read_or_default(&source).await;
            cx.update(|cx| apply_accessibility_settings(settings, cx));
            if let Err(error) = source.changed().await {
                eprintln!("HitIt stops the watch of the system accessibility settings. {error:#}");
                return;
            }
        }
    })
    .detach();
}

/// Other systems give no setting source yet. Each setting stays `false`.
#[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
fn watch_accessibility_settings(_cx: &mut gpui::App) {}

fn nudge_window(window_handle: gpui::AnyWindowHandle, cx: &mut gpui::App) {
    cx.defer(move |cx| {
        let _ = window_handle.update(cx, |_, window, cx| {
            window.activate_window();
            window.refresh();
            cx.refresh_windows();
        });
        cx.activate(true);
        cx.refresh_windows();
    });
    // Window-activation nudge: GPUI needs this 16ms/100ms deferred refresh after creation; presentation lifecycle, not domain work, exempted by `cx_spawn_is_restricted_to_presentation_runtime_and_bootstrap`.
    cx.spawn(async move |cx: &mut gpui::AsyncApp| {
        cx.background_executor()
            .timer(Duration::from_millis(16))
            .await;
        cx.update(|cx| {
            let _ = window_handle.update(cx, |_, window, cx| {
                window.activate_window();
                window.refresh();
                cx.refresh_windows();
            });
            cx.activate(true);
            cx.refresh_windows();
        });
        cx.background_executor()
            .timer(Duration::from_millis(100))
            .await;
        cx.refresh();
    })
    .detach();
}

/// The options of the main window.
fn main_window_options(bounds: Bounds<gpui::Pixels>) -> WindowOptions {
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        // ADR 0097 Decision 1. This record gives no minimum height.
        window_min_size: Some(size(layout::WINDOW_MIN_WIDTH, gpui::Pixels::ZERO)),
        // ADR 0096 Decision 4.
        titlebar: Some(gpui::TitlebarOptions {
            title: Some(crate::config::APP_DISPLAY_NAME.into()),
            ..Default::default()
        }),
        app_id: Some(crate::config::APP_ID.to_owned()),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ADR 0097 Decision 1: the window cannot become narrower than 390 px.
    #[test]
    fn adr_0097_the_window_has_a_minimum_width_of_390_px() {
        let bounds = Bounds::new(
            gpui::point(gpui::px(0.0), gpui::px(0.0)),
            size(gpui::px(1.0), gpui::px(1.0)),
        );
        let minimum = main_window_options(bounds)
            .window_min_size
            .map(|minimum| minimum.width);
        assert_eq!(
            minimum,
            Some(gpui::px(390.0)),
            "ADR 0097 Decision 1: the narrowest supported width is 390 px. \
             Set window_min_size in main_window_options to layout::WINDOW_MIN_WIDTH."
        );
    }

    #[test]
    fn startup_stops_a_stored_session_before_playback_owner_loads_it() {
        use crate::startup::{CheckIntent, CoreResult, StartupBackend};

        let temp = tempfile::tempdir().unwrap();
        let music = temp.path().join("music");
        std::fs::create_dir(&music).unwrap();
        let cfg_path = temp.path().join("config.toml");
        std::fs::write(
            &cfg_path,
            format!(
                "music_dir = {:?}\ndb_path = {:?}\n[playback]\ndriver = \"null\"\n",
                music,
                temp.path().join("library.sqlite")
            ),
        )
        .unwrap();
        let mut backend = StartupBackend::new(Some(cfg_path));
        let CoreResult::Prepared(core) = backend.execute(CheckIntent::Initial) else {
            panic!("valid core");
        };
        core.connection
            .execute(
                "INSERT INTO feeds(feed_url, feed_guid, title) VALUES ('https://startup.test/feed', 'startup', 'Startup')",
                [],
            )
            .unwrap();
        let feed_id = core.connection.last_insert_rowid();
        core.connection
            .execute(
                "INSERT INTO tracks(feed_id, item_guid, track_title, is_in_library) VALUES (?1, 'track', 'Track', 1)",
                [feed_id],
            )
            .unwrap();
        let track_id = core.connection.last_insert_rowid();
        crate::db::set_playback_session_track(
            &core.connection,
            crate::playback::DEFAULT_SESSION_ID,
            track_id,
            None,
            None,
            "started",
            0,
        )
        .unwrap();

        let prepared = prepare_normal(*core).unwrap_or_else(|_| panic!("valid preparation"));
        let conn = prepared.conn.lock().unwrap();
        let (state, stored_track_id): (String, i64) = conn
            .query_row(
                "SELECT state, local_track_id FROM playback_sessions WHERE session_id = ?1",
                [crate::playback::DEFAULT_SESSION_ID],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(state, "stopped");
        assert_eq!(stored_track_id, track_id);
        if let Some(owner) = &prepared.playback_owner {
            let mut owner = owner.lock().unwrap();
            assert!(owner.load_current_session(&conn).unwrap().is_none());
        }
    }

    #[test]
    #[expect(
        clippy::too_many_lines,
        reason = "this test narrates one sequential drain, correction and resume path, and splitting it would scatter shared session and path state"
    )]
    fn adr_0066_drained_core_reopens_fresh_only_after_successful_checks() {
        use crate::application::commands::maintenance::{
            CorrectionAccess, CorrectionCommand, CorrectionOperation, CorrectionResult,
        };
        use crate::application::session_lifecycle::{SessionDrain, SessionPhase};
        use crate::config::correction::{CorrectionDraft, CorrectionField, CorrectionSource};
        use crate::presentation::session_transition::SessionTransition;
        use crate::startup::{CheckIntent, CoreResult, StartupBackend};
        fn drain(
            prepared: NormalStartup,
        ) -> (
            crate::application::session_lifecycle::MaintenanceSession,
            crate::application::session_lifecycle::SessionLifecycle,
        ) {
            let NormalStartup {
                conn,
                session,
                playback_owner,
                runtime_host,
                cfg_path,
                ..
            } = prepared;
            session.begin_drain();
            let mut transition = SessionTransition {
                drain: SessionDrain::new(session.clone(), conn, playback_owner),
                runtime: runtime_host,
                conversions: None,
                #[cfg(debug_assertions)]
                config_path: cfg_path,
            };
            transition.wait_for_work().unwrap();
            (transition.close().unwrap(), session)
        }
        let temp = tempfile::tempdir().unwrap();
        let music = temp.path().join("music");
        std::fs::create_dir(&music).unwrap();
        let cfg_path = temp.path().join("config.toml");
        let original = format!(
            "music_dir = {:?}\ndb_path = {:?}\n[playback]\ndriver = \"null\"\n",
            music,
            temp.path().join("library.sqlite")
        );
        std::fs::write(&cfg_path, &original).unwrap();
        let mut backend = StartupBackend::new(Some(cfg_path.clone()));
        let CoreResult::Prepared(core) = backend.execute(CheckIntent::Initial) else {
            panic!("valid core");
        };
        let first = prepare_normal(*core).unwrap_or_else(|_| panic!("valid preparation"));
        let (mut authority, old) = drain(first);
        assert_eq!(old.phase(), SessionPhase::Maintenance);
        assert!(authority.begin_resume());
        std::fs::rename(&music, temp.path().join("music.saved")).unwrap();
        let CoreResult::Checked(failed) = backend.execute(CheckIntent::Check) else {
            panic!("missing storage must stay in recovery");
        };
        assert!(!failed.can_open());
        authority.resume_failed();
        assert_eq!(old.phase(), SessionPhase::Maintenance);
        assert!(!old.accepts(old.generation()));
        // Task 006: correct both paths only after the old owners have closed.
        let replacement_db = temp.path().join("selected-existing.sqlite");
        drop(crate::db::open_db(&replacement_db).unwrap());
        let command = CorrectionCommand {
            path: cfg_path.clone(),
            source: Some(Arc::new(CorrectionSource::read(&cfg_path).unwrap())),
            access: CorrectionAccess::CoreRecovery,
            operation: CorrectionOperation::Save,
            draft: CorrectionDraft {
                raw: None,
                fields: vec![
                    (
                        CorrectionField("music_dir"),
                        temp.path().join("music.saved").display().to_string(),
                    ),
                    (
                        CorrectionField("db_path"),
                        replacement_db.display().to_string(),
                    ),
                ],
            },
        };
        let CorrectionResult::Saved(receipt) = command.execute() else {
            panic!("correction must save after drain");
        };
        assert_eq!(std::fs::read_to_string(&receipt.backup).unwrap(), original);
        assert_eq!(
            old.phase(),
            SessionPhase::Maintenance,
            "Save cannot resume a session"
        );
        let CoreResult::Checked(stale) = backend.execute(CheckIntent::Open {
            checked_bytes: original.as_bytes().to_vec(),
        }) else {
            panic!("old settings cannot reopen");
        };
        assert_eq!(
            stale.checked_bytes().unwrap(),
            receipt.fresh.original_bytes()
        );
        let CoreResult::Checked(checked) = backend.execute(CheckIntent::Check) else {
            panic!("check-only cannot open");
        };
        assert!(authority.begin_resume());
        let CoreResult::Prepared(core) = backend.execute(CheckIntent::Open {
            checked_bytes: checked.checked_bytes().unwrap().to_vec(),
        }) else {
            panic!("fresh checked preparation");
        };
        let fresh = prepare_normal(*core).unwrap_or_else(|_| panic!("fresh preparation"));
        assert_eq!(fresh.cfg.db_path.as_ref().unwrap(), &replacement_db);
        assert_eq!(
            fresh.cfg.music_dir.as_ref().unwrap(),
            &temp.path().join("music.saved")
        );
        assert!(fresh.session.generation() > old.generation());
        assert!(fresh.session.accepts(fresh.session.generation()));
        assert!(!fresh.session.accepts(old.generation()));
        let _ = drain(fresh);
        assert_eq!(
            std::fs::read(cfg_path).unwrap(),
            receipt.fresh.original_bytes()
        );
    }

    #[test]
    fn adr_0066_normal_factory_scopes_each_optional_group_and_keeps_config_bytes() {
        let cases = [
            "musicindex_endpoint = false\n",
            "playback = false\n",
            "broadcast = false\n",
            "theme_profile = 12\n",
            "ui_scale = false\n",
            "workspace_layout = []\n",
            "flac_path = false\n",
            "[workspace]\nlayout = false\n",
            "[playback]\ndriver = 'broken'\n",
            "[broadcast]\nhosts = []\n",
            "[broadcast]\nselected_host = 12\n",
            "[broadcast]\ndrop_directory = false\n",
            "[broadcast]\nencoder = false\n",
            "[playback]\nmpv_path = false\n",
        ];
        for case in cases {
            let temp = tempfile::tempdir().unwrap();
            let music = temp.path().join("music");
            std::fs::create_dir(&music).unwrap();
            let path = temp.path().join("config.toml");
            let bytes = format!(
                "music_dir = {:?}\ndb_path = {:?}\n{case}",
                music,
                temp.path().join("library.sqlite")
            );
            std::fs::write(&path, &bytes).unwrap();
            let mut backend = crate::startup::StartupBackend::new(Some(path.clone()));
            let crate::startup::CoreResult::Prepared(core) =
                backend.execute(crate::startup::CheckIntent::Initial)
            else {
                panic!("optional setting blocked core admission: {case}");
            };
            let prepared = prepare_normal(*core)
                .unwrap_or_else(|_| panic!("optional setting blocked normal construction: {case}"));
            assert!(
                prepared
                    .capability_observations
                    .snapshot()
                    .values()
                    .any(|entry| entry.failure.is_some()),
                "{case}"
            );
            if case.contains("musicindex_endpoint") {
                assert!(prepared.musicindex_endpoint.require().is_err());
            }
            if case == "playback = false\n" || case.contains("driver = 'broken'") {
                assert!(prepared.playback_owner.is_none());
            }
            if case.contains("mpv_path = false") {
                assert!(
                    prepared.playback_owner.is_some(),
                    "unused mpv setting cannot disable configured Null"
                );
            }
            assert!(normal_startup_status(&prepared.notices).is_empty());
            drop(prepared);
            assert_eq!(std::fs::read_to_string(path).unwrap(), bytes);
        }
    }

    #[test]
    fn adr_0066_optional_notices_have_one_normal_report_owner() {
        let temp = tempfile::tempdir().unwrap();
        let music = temp.path().join("music");
        std::fs::create_dir(&music).unwrap();
        let artists = music.join("artists");
        std::fs::write(&artists, b"preserve download-directory blocker").unwrap();
        let config_path = temp.path().join("config.toml");
        let bytes = format!(
            "music_dir = {:?}\ndb_path = {:?}\ntheme_profile = 42\nui_scale = false\n[workspace.layout]\ncontent_pane_width = false\n",
            music, temp.path().join("library.sqlite"),
        );
        std::fs::write(&config_path, &bytes).unwrap();
        let mut backend = crate::startup::StartupBackend::new(Some(config_path.clone()));
        let crate::startup::CoreResult::Prepared(core) =
            backend.execute(crate::startup::CheckIntent::Initial)
        else {
            panic!("optional failures must permit normal startup");
        };
        assert_eq!(core.notices.len(), 4);
        let download_notice = core
            .notices
            .iter()
            .find(|issue| issue.stage == StartupStage::Artists)
            .unwrap()
            .cause
            .clone();
        let prepared = prepare_normal(*core).unwrap_or_else(|_| panic!("core remains usable"));
        assert_eq!(normal_startup_status(&prepared.notices), download_notice);
        let reports = crate::view_models::startup::capabilities::CapabilityReportVm::new(
            prepared.capability_observations.snapshot(),
            true,
        );
        for field in [
            "theme_profile",
            "ui_scale",
            "workspace.layout.content_pane_width",
        ] {
            assert_eq!(
                reports
                    .issues()
                    .filter(|issue| issue.dependency == Dependency::Configuration(field))
                    .count(),
                1
            );
            assert!(reports.report().contains(field));
        }
        assert_eq!(
            std::fs::read(&artists).unwrap(),
            b"preserve download-directory blocker"
        );
        assert_eq!(std::fs::read_to_string(config_path).unwrap(), bytes);
    }
}
