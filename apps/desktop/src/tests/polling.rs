use super::preference_writer::PreferenceWriter;
use crate::controller::Controller;
use anyq_config_api::{DatabaseSelection, DatabaseSource};
use anyq_desktop_api::{ConnectionState, PanelPreferences};
use anyq_queue_api::{Error, QueueSnapshot, Topic};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc,
    },
    time::Duration,
};

fn preferences(path: &str) -> PanelPreferences {
    PanelPreferences {
        database_override: Some(path.into()),
        ..PanelPreferences::default()
    }
}

fn selection(preferences: &PanelPreferences) -> anyq_queue_api::Result<DatabaseSelection> {
    Ok(DatabaseSelection {
        path: preferences.database_override.as_deref().unwrap().into(),
        source: DatabaseSource::Override,
    })
}

fn snapshot(capacity: u32) -> QueueSnapshot {
    QueueSnapshot::new(
        vec![Topic::restore("cpu".into(), capacity).unwrap()],
        vec![],
    )
    .unwrap()
}

#[test]
fn source_switch_rejects_late_data_coalesces_refresh_and_preserves_state_after_failed_save() {
    let failed = Arc::new(AtomicBool::new(false));
    let calls = Arc::new(AtomicUsize::new(0));
    let (started, start) = mpsc::channel();
    let (release, released) = mpsc::channel();
    let released = Mutex::new(released);
    let observed_calls = calls.clone();
    let observe = Arc::new(move |path: &std::path::Path, cancelled: Arc<AtomicBool>| {
        observed_calls.fetch_add(1, Ordering::SeqCst);
        if path == std::path::Path::new("A") {
            started.send(()).unwrap();
            released
                .lock()
                .unwrap()
                .recv_timeout(Duration::from_secs(5))
                .unwrap();
            assert!(cancelled.load(Ordering::SeqCst));
            return Ok(snapshot(1));
        }
        Ok(snapshot(2))
    });
    let controller = Arc::new(Controller::new(
        Ok(preferences("A")),
        Box::new(PreferenceWriter(failed.clone())),
        Arc::new(selection),
        observe,
        Arc::new(|| 1735732800000),
    ));
    let worker_controller = controller.clone();
    let (published, received) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        worker_controller.run(|| {
            published.send(()).unwrap();
        })
    });
    start.recv_timeout(Duration::from_secs(3)).unwrap();

    failed.store(true, Ordering::SeqCst);
    assert!(controller.save(preferences("B")).is_err());
    assert_eq!(
        controller
            .session()
            .preferences
            .database_override
            .as_deref(),
        Some("A")
    );
    assert_eq!(controller.session().model.generation, 1);
    failed.store(false, Ordering::SeqCst);
    for _ in 0..100 {
        controller.refresh();
    }
    controller.save(preferences("B")).unwrap();
    assert!(controller.session().model.snapshot.is_none());
    release.send(()).unwrap();
    received.recv_timeout(Duration::from_secs(3)).unwrap();
    received.recv_timeout(Duration::from_secs(3)).unwrap();
    controller.stop();
    worker.join().unwrap();

    let session = controller.session();
    assert_eq!(session.model.generation, 2);
    assert_eq!(session.model.status, ConnectionState::Live);
    assert_eq!(session.last_success, Some(1735732800000));
    assert_eq!(
        session.model.snapshot.as_ref().unwrap().topics[0].capacity,
        2
    );
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn connection_failure_retains_a_complete_stale_snapshot_and_recovers() {
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = calls.clone();
    let controller = Arc::new(Controller::new(
        Ok(preferences("A")),
        Box::new(PreferenceWriter(Arc::new(AtomicBool::new(false)))),
        Arc::new(selection),
        Arc::new(move |_, _| {
            if observed.fetch_add(1, Ordering::SeqCst) == 1 {
                Err(Error::Storage("owner restarted".into()))
            } else {
                Ok(snapshot(1))
            }
        }),
        Arc::new(|| 1735732800000),
    ));
    let worker_controller = controller.clone();
    let (published, received) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        worker_controller.run(|| {
            published.send(()).unwrap();
        })
    });
    received.recv_timeout(Duration::from_secs(3)).unwrap();

    controller.refresh();
    received.recv_timeout(Duration::from_secs(3)).unwrap();
    assert_eq!(controller.session().model.status, ConnectionState::Stale);
    assert!(controller.session().model.snapshot.is_some());
    controller.refresh();
    received.recv_timeout(Duration::from_secs(3)).unwrap();
    controller.stop();
    worker.join().unwrap();

    assert_eq!(controller.session().model.status, ConnectionState::Live);
    assert!(controller.session().model.error.is_none());
}
