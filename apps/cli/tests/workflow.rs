mod support;
use serde_json::Value;
use std::{
    io::Write,
    process::{Command, Stdio},
    thread,
    time::Duration,
};
use support::{
    fixture::{self, wait_for_file},
    process_status::assert_exited,
    queue::Queue,
};
#[test]
fn fixture_process() {
    fixture::run();
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
    let output = q
        .worker(
            "large-builds",
            "echo",
            &[
                "--next-topic",
                "emulator-use",
                "--next-payload",
                r#"{"artifact":"demo.apk"}"#,
            ],
        )
        .output()
        .unwrap();
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
    assert_eq!(jobs[0]["message"]["payload"]["artifact"], "demo.apk");
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
    q.call(&[
        "ack",
        &id,
        "--group",
        "audit",
        "--token",
        a["delivery"]["token"].as_str().unwrap(),
    ]);
    assert!(
        q.call(&["jobs", "show", &id])["deliveries"]
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
    let out = q
        .worker(
            "builds",
            "fail",
            &["--next-topic", "emulator", "--next-payload", "{}"],
        )
        .output()
        .unwrap();
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
#[test]
fn real_worker_renews_lease_during_long_command() {
    let q = Queue::new();
    q.setup("builds", "builders");
    q.publish("builds");
    let marker = q._dir.path().join("started");
    let child = q
        .worker("builds", "sleep", &["--lease-ms", "2000"])
        .env("ANYQ_FIXTURE_MARKER", &marker)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    wait_for_file(&marker);
    thread::sleep(Duration::from_millis(2200));
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
        q.worker("builds", "echo", &["--lease-ms", "1"])
            .output()
            .unwrap()
            .status
            .code(),
        Some(2)
    );
    assert_eq!(q.call(&["jobs", "list"])[0]["deliveries"][0]["attempts"], 0);
}
#[test]
fn completed_parent_does_not_leave_descendants_running() {
    let q = Queue::new();
    q.setup("builds", "builders");
    q.publish("builds");
    let marker = q._dir.path().join("started");
    let pid = q._dir.path().join("leaf.pid");
    let out = q
        .worker("builds", "orphan", &[])
        .env("ANYQ_FIXTURE_MARKER", &marker)
        .env("ANYQ_FIXTURE_PID", &pid)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_exited(std::fs::read_to_string(pid).unwrap().parse().unwrap());
}
#[cfg(unix)]
#[test]
fn sigterm_stops_process_group_and_releases_delivery() {
    let q = Queue::new();
    q.setup("builds", "builders");
    q.publish("builds");
    let marker = q._dir.path().join("started");
    let pid = q._dir.path().join("leaf.pid");
    let child = q
        .worker("builds", "tree", &[])
        .env("ANYQ_FIXTURE_MARKER", &marker)
        .env("ANYQ_FIXTURE_PID", &pid)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    wait_for_file(&marker);
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
    assert_exited(std::fs::read_to_string(pid).unwrap().parse().unwrap());
    assert_eq!(
        q.call(&["jobs", "list"])[0]["deliveries"][0]["last_error"],
        "worker interrupted"
    );
}
#[cfg(windows)]
#[test]
fn console_interrupt_stops_tree_and_releases_delivery() {
    use std::os::windows::process::CommandExt;
    use windows_sys::Win32::System::Threading::CREATE_NEW_CONSOLE;
    for event in [0, 1] {
        let q = Queue::new();
        q.setup("builds", "builders");
        q.publish("builds");
        let marker = q._dir.path().join("started");
        let pid = q._dir.path().join("leaf.pid");
        let child = q
            .worker("builds", "tree", &[])
            .creation_flags(CREATE_NEW_CONSOLE)
            .env("ANYQ_FIXTURE_MARKER", &marker)
            .env("ANYQ_FIXTURE_PID", &pid)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        wait_for_file(&marker);
        let args = fixture::arguments();
        assert!(
            Command::new(&args[0])
                .args(&args[1..])
                .env("ANYQ_FIXTURE_MODE", "console-event")
                .env("ANYQ_FIXTURE_WORKER", child.id().to_string())
                .env("ANYQ_FIXTURE_EVENT", event.to_string())
                .stdout(Stdio::null())
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
        assert_exited(std::fs::read_to_string(pid).unwrap().parse().unwrap());
        assert_eq!(
            q.call(&["jobs", "list"])[0]["deliveries"][0]["state"],
            "pending"
        );
    }
}
#[cfg(windows)]
#[test]
fn windows_uses_local_app_data_without_overriding_explicit_db() {
    let q = Queue::new();
    let local = q._dir.path().join("local");
    let out = Command::new(env!("CARGO_BIN_EXE_anyq"))
        .args(["topic", "list"])
        .env_remove("ANYQ_DB")
        .env("LOCALAPPDATA", &local)
        .env("HOME", "unused")
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(local.join("anyq/queue.db").exists());
    let out = q
        .cmd(&["topic", "list"])
        .env_remove("LOCALAPPDATA")
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(q.db.exists());
}
