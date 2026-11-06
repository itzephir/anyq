use serde_json::Value;
use std::{
    path::PathBuf,
    process::{Command, Output},
};
pub struct Queue {
    pub _dir: tempfile::TempDir,
    pub db: PathBuf,
}
impl Queue {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("q.db");
        Self { _dir: dir, db }
    }
    pub fn cmd(&self, args: &[&str]) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_anyq"));
        c.env("ANYQ_DB", &self.db).args(args);
        c
    }
    pub fn raw(&self, args: &[&str]) -> Output {
        self.cmd(args).output().unwrap()
    }
    pub fn call(&self, args: &[&str]) -> Value {
        let output = self.raw(args);
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }
    pub fn setup(&self, topic: &str, group: &str) {
        self.call(&["topic", "create", topic]);
        self.call(&["group", "create", topic, group]);
    }
    pub fn publish(&self, topic: &str) -> Value {
        self.call(&["publish", topic, "--payload", "{}"])
    }
    pub fn worker(&self, topic: &str, mode: &str, extra: &[&str]) -> Command {
        let mut command = self.cmd(&[
            "worker",
            topic,
            "--group",
            "builders",
            "--consumer",
            "native",
            "--once",
        ]);
        command
            .args(extra)
            .arg("--")
            .args(super::fixture::arguments())
            .env("ANYQ_FIXTURE_MODE", mode);
        command
    }
}
