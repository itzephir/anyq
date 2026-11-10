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
        command
            .env("ANYQ_DB", &self.db)
            .env("HOME", self.dir.path())
            .env("USERPROFILE", self.dir.path())
            .args(args);
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
        let mut arguments = vec!["task", "create", name];

        for topic in topics {
            arguments.extend(["-t", topic]);
        }

        self.call(&arguments);
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

    pub fn spawn_owner(&self) -> TestProcess {
        let mut command = self.cmd(&["daemon", "run"]);
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;

            command.creation_flags(0x00000010);
        }

        let mut owner = TestProcess::spawn(&mut command);
        let deadline = Instant::now() + Duration::from_secs(10);

        while self.call(&["daemon", "status"]) == "daemon is stopped" {
            assert!(Instant::now() < deadline, "foreground daemon did not start");
            owner.assert_running();
            thread::sleep(Duration::from_millis(5));
        }

        owner.assert_running();

        owner
    }

    pub fn wait_for_log(&self, label: &str, expected: &str) {
        wait_for_file(&self.log(label), Some(expected));
    }
}

impl Drop for Queue {
    fn drop(&mut self) {
        fn databases(directory: &Path, found: &mut Vec<PathBuf>) {
            let Ok(entries) = fs::read_dir(directory) else {
                return;
            };

            for entry in entries.flatten() {
                if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                    databases(&entry.path(), found);
                } else if let Some(name) = entry
                    .file_name()
                    .to_str()
                    .and_then(|name| name.strip_suffix(".anyq-owner.lock"))
                {
                    found.push(entry.path().with_file_name(name));
                }
            }
        }

        let mut paths = Vec::new();
        databases(self.dir.path(), &mut paths);

        for path in paths {
            let Ok(mut child) = self
                .cmd(&["daemon", "stop"])
                .arg("--db")
                .arg(path)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            else {
                continue;
            };
            let deadline = Instant::now() + Duration::from_secs(10);

            while child.try_wait().is_ok_and(|status| status.is_none()) {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    break;
                }

                thread::sleep(Duration::from_millis(10));
            }
        }
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
