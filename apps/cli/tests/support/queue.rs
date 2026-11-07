use super::process::TestProcess;
use rusqlite::Connection;
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};

pub struct Queue {
    pub dir: tempfile::TempDir,
    pub db: PathBuf,
}

impl Queue {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("tasks.db");

        Self { dir, db }
    }

    pub fn cmd(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_anyq"));
        command.env("ANYQ_DB", &self.db).args(args);
        command
    }

    pub fn raw(&self, args: &[&str]) -> Output {
        TestProcess::spawn(self.cmd(args).stdout(Stdio::piped()).stderr(Stdio::piped())).output()
    }

    pub fn call(&self, args: &[&str]) -> String {
        let output = self.raw(args);

        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        String::from_utf8(output.stdout).unwrap().trim_end().into()
    }

    pub fn topic(&self, name: &str) {
        self.call(&["topic", "create", name]);
    }

    pub fn task(&self, name: &str, topics: &[&str]) {
        self.call(&["task", "add", name]);

        for topic in topics {
            self.call(&["topic", topic, "add", name]);
        }
    }

    pub fn state(&self, name: &str) -> String {
        let connection = Connection::open(&self.db).unwrap();
        connection.busy_timeout(Duration::from_secs(5)).unwrap();
        connection
            .query_row("SELECT state FROM tasks WHERE name=?1", [name], |row| {
                row.get(0)
            })
            .unwrap()
    }

    pub fn usage(&self, topic: &str) -> i64 {
        Connection::open(&self.db).unwrap().query_row(
            "SELECT count(*) FROM tasks t JOIN task_topics m ON t.id=m.task_id WHERE t.state='running' AND m.topic_name=?1",
            [topic], |row| row.get(0),
        ).unwrap()
    }

    pub fn log(&self, label: &str) -> PathBuf {
        self.dir.path().join(format!("{label}.log"))
    }

    pub fn spawn_wait(&self, name: &str, label: &str) -> TestProcess {
        TestProcess::spawn(
            self.cmd(&["task", "wait", name])
                .stdout(Stdio::piped())
                .stderr(File::create(self.log(label)).unwrap()),
        )
    }

    pub fn wait_for_log(&self, label: &str, expected: &str) {
        wait_for_file(&self.log(label), Some(expected));
    }
}

pub fn wait_for_file(path: &Path, expected: Option<&str>) {
    let deadline = Instant::now() + Duration::from_secs(15);

    loop {
        if let Ok(text) = fs::read_to_string(path) {
            if expected.is_none_or(|needle| text.contains(needle)) {
                return;
            }
        }

        assert!(
            Instant::now() < deadline,
            "did not observe fixture output at {}",
            path.display()
        );
        thread::sleep(Duration::from_millis(10));
    }
}
