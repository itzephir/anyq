use serde_json::Value;
use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};

struct Queue {
    _dir: tempfile::TempDir,
    db: PathBuf,
}
impl Queue {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("q.db");
        Self { _dir: dir, db }
    }
    fn cmd(&self, args: &[&str]) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_anyq"));
        c.env("ANYQ_DB", &self.db).args(args);
        c
    }
    fn raw(&self, args: &[&str]) -> Output {
        self.cmd(args).output().unwrap()
    }
    fn call(&self, args: &[&str]) -> Value {
        let output = self.raw(args);
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }
    fn setup(&self, topic: &str, group: &str) {
        self.call(&["topic", "create", topic]);
        self.call(&["group", "create", topic, group]);
    }
    fn publish(&self, topic: &str) -> Value {
        self.call(&["publish", topic, "--payload", "{}"])
    }
}

#[test]
fn automatic_build_to_emulator_and_late_group_replay() {
    let q = Queue::new();
    q.setup("large-builds", "builders");
    q.setup("emulator-use", "tests");
    let mut producer = q
        .cmd(&[
            "publish",
            "large-builds",
            "--file",
            "-",
            "--key",
            "product-revision",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    producer
        .stdin
        .take()
        .unwrap()
        .write_all(br#"{"product":"demo","text":"$(false)"}"#)
        .unwrap();
    let output = producer.wait_with_output().unwrap();
    assert!(output.status.success());
    let first: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        first["id"],
        q.call(&[
            "publish",
            "large-builds",
            "--payload",
            "{}",
            "--key",
            "product-revision"
        ])["id"]
    );
    let output = q.raw(&[
        "worker",
        "large-builds",
        "--group",
        "builders",
        "--consumer",
        "bash",
        "--once",
        "--next-topic",
        "emulator-use",
        "--next-payload",
        r#"{"artifact":"/tmp/demo.apk"}"#,
        "--",
        "sh",
        "-c",
        "cat; printf command-output",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["state"], "succeeded");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("$(false)"));
    assert!(stderr.contains("command-output"));
    let jobs = q.call(&[
        "jobs",
        "list",
        "--topic",
        "emulator-use",
        "--group",
        "tests",
    ]);
    assert_eq!(jobs.as_array().unwrap().len(), 1);
    assert_eq!(jobs[0]["message"]["payload"]["artifact"], "/tmp/demo.apk");
    q.call(&["group", "create", "large-builds", "audit"]);
    let a = q.call(&[
        "claim",
        "large-builds",
        "--group",
        "audit",
        "--consumer",
        "human",
    ]);
    assert_eq!(a["message"]["id"], first["id"]);
    let id = first["id"].to_string();
    let token = a["delivery"]["token"].as_str().unwrap();
    q.call(&["ack", &id, "--group", "audit", "--token", token]);
    let view = q.call(&["jobs", "show", &id]);
    assert!(
        view["deliveries"]
            .as_array()
            .unwrap()
            .iter()
            .all(|d| d["state"] == "succeeded")
    );
}

#[test]
fn worker_failure_does_not_publish_next_stage() {
    let q = Queue::new();
    q.setup("builds", "builders");
    q.setup("emulator", "tests");
    q.publish("builds");
    let out = q.raw(&[
        "worker",
        "builds",
        "--group",
        "builders",
        "--consumer",
        "bash",
        "--once",
        "--next-topic",
        "emulator",
        "--next-payload",
        "{}",
        "--",
        "sh",
        "-c",
        "exit 7",
    ]);
    assert_eq!(out.status.code(), Some(1));
    let jobs = q.call(&["jobs", "list", "--topic", "builds"]);
    assert_eq!(jobs[0]["deliveries"][0]["state"], "pending");
    assert!(
        jobs[0]["deliveries"][0]["last_error"]
            .as_str()
            .unwrap()
            .contains('7')
    );
    assert_eq!(
        q.call(&["jobs", "list", "--topic", "emulator"]),
        serde_json::json!([])
    );
}

#[test]
fn concurrent_processes_obey_topic_capacity() {
    let q = Queue::new();
    q.setup("builds", "builders");
    for _ in 0..8 {
        q.publish("builds");
    }
    let children: Vec<_> = (0..8)
        .map(|_| {
            q.cmd(&[
                "claim",
                "builds",
                "--group",
                "builders",
                "--consumer",
                "worker",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap()
        })
        .collect();
    let claimed = children
        .into_iter()
        .map(|child| {
            let output = child.wait_with_output().unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let result: Value = serde_json::from_slice(&output.stdout).unwrap();
            usize::from(!result.is_null())
        })
        .sum::<usize>();
    assert_eq!(claimed, 1);
}

fn wait_for_file(path: &Path) {
    let end = Instant::now() + Duration::from_secs(5);
    while !path.exists() {
        assert!(Instant::now() < end, "worker failed to start");
        thread::sleep(Duration::from_millis(10));
    }
}
#[test]
fn real_worker_renews_lease_during_long_command() {
    let q = Queue::new();
    q.setup("builds", "builders");
    q.publish("builds");
    let marker = q._dir.path().join("started");
    let child = q
        .cmd(&[
            "worker",
            "builds",
            "--group",
            "builders",
            "--consumer",
            "bash",
            "--lease-ms",
            "300",
            "--once",
            "--",
            "sh",
            "-c",
            "cat >/dev/null; printf ready > \"$1\"; sleep 1",
            "_",
            marker.to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    wait_for_file(&marker);
    thread::sleep(Duration::from_millis(550));
    assert_eq!(
        q.call(&[
            "claim",
            "builds",
            "--group",
            "builders",
            "--consumer",
            "other"
        ]),
        Value::Null
    );
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn expired_token_failed_budget_and_manual_retry() {
    let q = Queue::new();
    q.setup("builds", "builders");
    let m = q.call(&[
        "publish",
        "builds",
        "--payload",
        "{}",
        "--max-attempts",
        "1",
    ]);
    let id = m["id"].to_string();
    let a = q.call(&[
        "claim",
        "builds",
        "--group",
        "builders",
        "--consumer",
        "agent",
        "--lease-ms",
        "20",
    ]);
    thread::sleep(Duration::from_millis(40));
    let out = q.raw(&[
        "ack",
        &id,
        "--group",
        "builders",
        "--token",
        a["delivery"]["token"].as_str().unwrap(),
    ]);
    assert_eq!(out.status.code(), Some(2));
    let error: Value = serde_json::from_slice(&out.stderr).unwrap();
    assert_eq!(error["error"]["kind"], "lease_lost");
    assert_eq!(
        q.call(&["jobs", "show", &id])["deliveries"][0]["state"],
        "failed"
    );
    q.call(&["retry", &id, "--group", "builders"]);
    assert_eq!(
        q.call(&[
            "claim",
            "builds",
            "--group",
            "builders",
            "--consumer",
            "agent"
        ])["delivery"]["attempts"],
        1
    );
}

#[test]
fn invalid_json_and_invalid_worker_configuration_have_no_delivery_side_effects() {
    let q = Queue::new();
    q.setup("builds", "builders");
    assert_eq!(
        q.raw(&["publish", "builds", "--payload", "broken"])
            .status
            .code(),
        Some(2)
    );
    assert_eq!(q.call(&["jobs", "list"]), serde_json::json!([]));
    q.publish("builds");
    assert_eq!(
        q.raw(&[
            "worker",
            "builds",
            "--group",
            "builders",
            "--consumer",
            "test",
            "--lease-ms",
            "1",
            "--once",
            "--",
            "true"
        ])
        .status
        .code(),
        Some(2)
    );
    assert_eq!(q.call(&["jobs", "list"])[0]["deliveries"][0]["attempts"], 0);
}

#[test]
fn sigterm_stops_process_group_and_releases_delivery() {
    let q = Queue::new();
    q.setup("builds", "builders");
    q.publish("builds");
    let marker = q._dir.path().join("command.pid");
    let child = q
        .cmd(&[
            "worker",
            "builds",
            "--group",
            "builders",
            "--consumer",
            "bash",
            "--once",
            "--",
            "sh",
            "-c",
            "printf '%s' $$ > \"$1\"; sleep 30",
            "_",
            marker.to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    wait_for_file(&marker);
    let command_pid = std::fs::read_to_string(&marker).unwrap();
    assert!(
        Command::new("kill")
            .args(["-TERM", &child.id().to_string()])
            .status()
            .unwrap()
            .success()
    );
    let out = child.wait_with_output().unwrap();
    assert_eq!(
        out.status.code(),
        Some(130),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !Command::new("kill")
            .args(["-0", &command_pid])
            .stderr(Stdio::null())
            .status()
            .unwrap()
            .success()
    );
    let jobs = q.call(&["jobs", "list", "--topic", "builds"]);
    assert_eq!(jobs[0]["deliveries"][0]["state"], "pending");
    assert_eq!(jobs[0]["deliveries"][0]["last_error"], "worker interrupted");
}
