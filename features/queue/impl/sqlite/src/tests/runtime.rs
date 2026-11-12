use crate::{RuntimeOpen, Sqlite};
use anyq_queue_api::{Error, Result};
use rusqlite::Connection;
use std::{
    fs,
    path::Path,
    sync::{Arc, atomic::AtomicBool},
};

fn open(path: &Path) -> RuntimeOpen {
    Sqlite::open_existing_for_runtime(path, Arc::new(AtomicBool::new(false)), |_| Ok(())).unwrap()
}

fn files(path: &Path) -> Vec<(String, Vec<u8>)> {
    let mut files = fs::read_dir(path)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            (
                entry.file_name().to_str().unwrap().to_owned(),
                fs::read(entry.path()).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    files.sort();
    files
}

#[test]
fn runtime_does_not_create_a_missing_database_or_parent() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("missing/queue.db");

    assert!(matches!(open(&path), RuntimeOpen::Missing));
    assert!(!path.parent().unwrap().exists());
}

#[test]
fn runtime_rejects_empty_legacy_future_and_corrupt_files_without_side_effects() {
    for version in [-1, 0, 2, 3, 4, 5, 7, 99] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("queue.db");
        if version == -1 {
            fs::write(&path, []).unwrap();
        } else {
            let connection = Connection::open(&path).unwrap();
            connection.execute_batch("CREATE TABLE preserved(value TEXT); INSERT INTO preserved VALUES ('original');").unwrap();
            connection
                .pragma_update(None, "user_version", version)
                .unwrap();
            connection
                .pragma_update(None, "journal_mode", "WAL")
                .unwrap();
        }
        let before = files(dir.path());

        let outcome = open(&path);

        match version {
            -1 | 0 => assert!(matches!(outcome, RuntimeOpen::Uninitialized)),
            2..=5 => assert!(matches!(outcome, RuntimeOpen::UpgradeRequired(v) if v == version)),
            _ => assert!(matches!(outcome, RuntimeOpen::Incompatible(_))),
        }
        assert_eq!(files(dir.path()), before);
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("queue.db");
    fs::write(&path, b"not a sqlite database").unwrap();
    let before = files(dir.path());

    assert!(matches!(open(&path), RuntimeOpen::Incompatible(_)));
    assert_eq!(files(dir.path()), before);
}

#[test]
fn runtime_validates_all_queue_and_work_tables_before_opening_real_storage() {
    for table in [
        "topics",
        "tasks",
        "task_topics",
        "queue_identity",
        "queue_events",
        "task_directories",
        "managed_work",
        "completion_intents",
        "completion_receipts",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("queue.db");
        drop(Sqlite::open(&path).unwrap());
        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch(&format!("PRAGMA foreign_keys=OFF; DROP TABLE {table};"))
            .unwrap();
        drop(connection);
        let before = files(dir.path());

        assert!(
            matches!(open(&path), RuntimeOpen::Incompatible(_)),
            "{table}"
        );
        assert_eq!(files(dir.path()), before, "{table}");
    }
}

#[test]
fn runtime_rejects_missing_details_identity_and_injected_plugin_schema() {
    for mutation in [
        "ALTER TABLE managed_work DROP COLUMN details",
        "DELETE FROM queue_identity",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("queue.db");
        drop(Sqlite::open(&path).unwrap());
        let connection = Connection::open(&path).unwrap();
        connection.execute_batch(mutation).unwrap();
        drop(connection);
        let before = files(dir.path());

        assert!(matches!(open(&path), RuntimeOpen::Incompatible(_)));
        assert_eq!(files(dir.path()), before);
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("queue.db");
    drop(Sqlite::open(&path).unwrap());
    let before = files(dir.path());
    fn missing_plugin(connection: &Connection) -> Result<()> {
        connection
            .prepare("SELECT version FROM plugin_schema")
            .map_err(|e| Error::Storage(e.to_string()))?;
        Ok(())
    }

    let outcome =
        Sqlite::open_existing_for_runtime(&path, Arc::new(AtomicBool::new(false)), missing_plugin)
            .unwrap();

    assert!(matches!(outcome, RuntimeOpen::Incompatible(_)));
    assert_eq!(files(dir.path()), before);
}

#[test]
fn runtime_reads_committed_wal_and_preserves_ownership_without_synthetic_events() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("queue.db");
    let original = Sqlite::open(&path).unwrap();
    super::topic(&original, "cpu", 1);
    super::register(&original, "owner");
    super::add(&original, "owner", "cpu");
    let events: i64 = original
        .connection
        .borrow()
        .query_row("SELECT count(*) FROM queue_events", [], |r| r.get(0))
        .unwrap();
    assert!(
        path.with_file_name("queue.db-wal")
            .metadata()
            .unwrap()
            .len()
            > 32
    );

    let RuntimeOpen::Ready(reopened) = open(&path) else {
        panic!("valid WAL database rejected")
    };

    assert_eq!(
        super::task(&reopened, "owner"),
        super::task(&original, "owner")
    );
    assert_eq!(super::usage(&reopened, "cpu"), 1);
    assert_eq!(
        reopened
            .connection
            .borrow()
            .query_row("SELECT count(*) FROM queue_events", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        events
    );
}

#[test]
fn runtime_copy_honors_shutdown_before_opening_selected_storage() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("queue.db");
    drop(Sqlite::open(&path).unwrap());
    let before = files(dir.path());

    let outcome =
        Sqlite::open_existing_for_runtime(&path, Arc::new(AtomicBool::new(true)), |_| Ok(()));

    assert!(matches!(outcome, Err(Error::Interrupted)));
    assert_eq!(files(dir.path()), before);
}

#[test]
fn runtime_revalidates_replaced_file_before_changing_its_journal_mode() {
    use std::sync::OnceLock;

    static REPLACEMENT: OnceLock<(std::path::PathBuf, std::path::PathBuf)> = OnceLock::new();

    fn replace_after_validation(_: &Connection) -> Result<()> {
        let (selected, replacement) = REPLACEMENT.get().unwrap();
        fs::remove_file(selected).unwrap();
        fs::rename(replacement, selected).unwrap();

        Ok(())
    }

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("queue.db");
    let replacement = dir.path().join("replacement.db");
    drop(Sqlite::open(&path).unwrap());
    let connection = Connection::open(&replacement).unwrap();
    connection
        .execute_batch("CREATE TABLE preserved(value TEXT); PRAGMA user_version=4;")
        .unwrap();
    drop(connection);
    let original = fs::read(&replacement).unwrap();
    REPLACEMENT.set((path.clone(), replacement)).unwrap();

    let outcome = Sqlite::open_existing_for_runtime(
        &path,
        Arc::new(AtomicBool::new(false)),
        replace_after_validation,
    )
    .unwrap();

    assert!(matches!(outcome, RuntimeOpen::UpgradeRequired(4)));
    assert_eq!(fs::read(&path).unwrap(), original);
    assert_eq!(files(dir.path()).len(), 1);
}
