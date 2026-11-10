use crate::{SqlitePluginStore, initialize};
use anyq_plugin_api::*;
use anyq_plugin_dispatch::*;
use anyq_queue_api::{Metadata, MetadataValue};
use rusqlite::Connection;

fn database(path: &std::path::Path) {
    let mut connection = Connection::open(path).unwrap();
    let tx = connection.transaction().unwrap();
    tx.execute_batch("CREATE TABLE queue_events(sequence INTEGER PRIMARY KEY AUTOINCREMENT,payload TEXT); INSERT INTO queue_events(sequence,payload) VALUES (10,'{}'); CREATE TABLE managed_work(owner TEXT,state TEXT);").unwrap();
    initialize(&tx).unwrap();
    tx.commit().unwrap();
}

fn enable(store: &SqlitePluginStore) {
    EnablePluginInteractor::new(store.clone())
        .enable(EnablePluginRequest {
            name: "fixture".into(),
            executable: "fixture-program".into(),
            arguments: vec![],
        })
        .unwrap();
}

#[test]
fn future_watermark_and_ack_state_timer_are_atomic_and_resume_at_original_deadline() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("plugins.db");
    database(&path);
    let mut store = SqlitePluginStore::open(&path).unwrap();
    enable(&store);
    assert_eq!(
        GetPluginStatusInteractor::new(store.clone())
            .status("fixture")
            .unwrap()
            .cursor,
        10
    );
    let mut state = Metadata::default();
    state.set("unknown", MetadataValue::Bool(true)).unwrap();
    let timer = PluginTimer {
        id: "timer".into(),
        deadline: 100,
        origin_sequence: 11,
        payload: Metadata::default(),
    };
    let ack = AcknowledgeRequest {
        plugin: "fixture".into(),
        sequence: Some(11),
        timer: None,
        state: Some(state.clone()),
        timers: vec![TimerUpdate::Put(timer.clone())],
    };
    acknowledge(&mut store, ack.clone()).unwrap();
    DisablePluginInteractor::new(store.clone())
        .disable("fixture")
        .unwrap();
    assert!(
        acknowledge(
            &mut store,
            AcknowledgeRequest {
                sequence: Some(12),
                ..ack.clone()
            }
        )
        .is_err()
    );
    drop(store);
    let mut store = SqlitePluginStore::open(&path).unwrap();
    enable(&store);
    let plugin = store
        .transaction(|records| Ok(records.registration("fixture")?.unwrap()))
        .unwrap();
    let timers = store
        .transaction(|records| records.timers("fixture"))
        .unwrap();

    assert_eq!(plugin.cursor, 11);
    assert_eq!(plugin.state, state);
    assert_eq!(timers, vec![timer]);
    acknowledge(&mut store, ack).unwrap();
    assert_eq!(
        store
            .transaction(|records| records.timers("fixture"))
            .unwrap(),
        timers
    );
    acknowledge(
        &mut store,
        AcknowledgeRequest {
            plugin: "fixture".into(),
            sequence: None,
            timer: Some(("timer".into(), 100)),
            state: None,
            timers: vec![],
        },
    )
    .unwrap();
    assert!(
        store
            .transaction(|records| records.timers("fixture"))
            .unwrap()
            .is_empty()
    );
}

#[test]
fn rejected_timer_batch_rolls_back_cursor_state_and_all_partial_timer_writes() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("plugins.db");
    database(&path);
    let mut store = SqlitePluginStore::open(&path).unwrap();
    enable(&store);
    let mut state = Metadata::default();
    state.set("changed", MetadataValue::Bool(true)).unwrap();
    let updates = (0..128)
        .map(|index| {
            TimerUpdate::Put(PluginTimer {
                id: format!("t{index}"),
                deadline: 100,
                origin_sequence: 11,
                payload: Metadata::default(),
            })
        })
        .collect();
    acknowledge(
        &mut store,
        AcknowledgeRequest {
            plugin: "fixture".into(),
            sequence: Some(11),
            timer: None,
            state: None,
            timers: updates,
        },
    )
    .unwrap();
    let failed = acknowledge(
        &mut store,
        AcknowledgeRequest {
            plugin: "fixture".into(),
            sequence: Some(12),
            timer: None,
            state: Some(state),
            timers: vec![TimerUpdate::Put(PluginTimer {
                id: "overflow".into(),
                deadline: 101,
                origin_sequence: 12,
                payload: Metadata::default(),
            })],
        },
    );

    assert!(failed.is_err());
    let plugin = store
        .transaction(|records| Ok(records.registration("fixture")?.unwrap()))
        .unwrap();
    assert_eq!(plugin.cursor, 11);
    assert_eq!(plugin.state, Metadata::default());
    assert_eq!(
        store
            .transaction(|records| records.timers("fixture"))
            .unwrap()
            .len(),
        128
    );
}
