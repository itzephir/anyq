use crate::support::{process::TestProcess, queue::Queue};
use serde_json::{Value, json};
use std::{fs, process::Stdio};

fn get(queue: &Queue, kind: &str, name: &str) -> Value {
    serde_json::from_str(&queue.call(&[kind, name, "meta"])).unwrap()
}

#[test]
fn metadata_forms_types_unset_and_creation_precedence_share_one_contract() {
    let queue = Queue::new();
    let file = queue.dir.path().join("initial.json");
    fs::write(&file, r#"{"timeout":"1h","keep":true,"nested":{"a":1}}"#).unwrap();

    queue.call(&[
        "task",
        "job",
        "create",
        "--meta",
        "timeout",
        "2h",
        "--metafile",
        file.to_str().unwrap(),
        "--meta",
        "nested.b",
        "false",
        "--meta",
        "literal",
        "-twc",
        "--meta",
        "timeout",
        "3h",
    ]);

    assert_eq!(
        get(&queue, "task", "job"),
        json!({"timeout":"3h","keep":true,"nested":{"a":1,"b":false},"literal":"-twc"})
    );
    assert_eq!(queue.call(&["task", "meta", "job", "timeout"]), "\"3h\"");
    queue.call(&["task", "job", "meta", "n", "-1"]);
    assert_eq!(queue.call(&["task", "job", "meta", "n"]), "-1");
    queue.call(&["task", "meta", "job", "--unset", "nested.a"]);
    queue.call(&["task", "job", "meta", "--unset", "absent.deep"]);
    assert_eq!(get(&queue, "task", "job")["nested"], json!({"b":false}));
    assert_eq!(
        queue.raw(&["task", "job", "meta", "missing"]).status.code(),
        Some(2)
    );

    queue.call(&["topic", "create", "cpu", "--meta", "timeout", "1h"]);
    assert_eq!(get(&queue, "topic", "cpu"), json!({"timeout":"1h"}));
    assert_eq!(get(&queue, "task", "job")["timeout"], "3h");
    queue.call(&["task", "`meta`", "create", "--meta", "unknown.flag", "true"]);
    queue.call(&[
        "topic",
        "`metafile`",
        "create",
        "--meta",
        "unknown.flag",
        "false",
    ]);
    assert_eq!(get(&queue, "task", "`meta`")["unknown"]["flag"], true);
    assert_eq!(get(&queue, "topic", "`metafile`")["unknown"]["flag"], false);
}

#[test]
fn native_file_imports_preserve_exact_numbers_replace_objects_and_do_not_execute_shell_data() {
    let queue = Queue::new();
    let directory = queue.dir.path();
    let sources = [
        (
            "json",
            r#"{"n":9007199254740993,"precise":0.123456789012345678901,"shell":{"sh":["printf ok"],"autocomplete":true}}"#,
        ),
        (
            "yaml",
            "n: 9007199254740993\nprecise: 0.123456789012345678901\nshell: {sh: [printf ok], autocomplete: true}\n",
        ),
        (
            "toml",
            "n = 9007199254740993\nprecise = 0.123456789012345678901\n[shell]\nsh = ['printf ok']\nautocomplete = true\n",
        ),
    ];

    for (extension, source) in sources {
        let file = directory.join(format!("meta.{extension}"));
        fs::write(&file, source).unwrap();
        queue.call(&[
            "task",
            "create",
            extension,
            "--metafile",
            file.to_str().unwrap(),
        ]);
        let raw = queue.call(&["task", extension, "meta"]);

        assert!(raw.contains("9007199254740993"));
        assert!(raw.contains("0.123456789012345678901"));
        assert_eq!(queue.state(extension), "registered");
        queue.call(&["task", extension, "meta", "extra", "true"]);
        queue.call(&["task", "metafile", extension, file.to_str().unwrap()]);
        assert!(get(&queue, "task", extension).get("extra").is_none());
    }

    let yaml = directory.join("aliases.yml");
    fs::write(&yaml, "original: &a [true, ok]\ncopy: *a").unwrap();
    queue.call(&["topic", "create", "cpu"]);
    queue.call(&["topic", "cpu", "metafile", yaml.to_str().unwrap()]);
    let value = get(&queue, "topic", "cpu");
    assert_eq!(value["original"], value["copy"]);
}

#[test]
fn relative_files_resolve_at_the_client_after_an_owner_started_elsewhere() {
    let queue = Queue::new();
    queue.call(&["daemon", "start"]);
    let client = queue.dir.path().join("client files");
    fs::create_dir(&client).unwrap();
    fs::write(client.join("meta.json"), r#"{"origin":"client"}"#).unwrap();

    let output = TestProcess::spawn(
        queue
            .cmd(&["task", "create", "relative", "--metafile", "meta.json"])
            .current_dir(&client)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    )
    .output();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(get(&queue, "task", "relative"), json!({"origin":"client"}));
}

// macOS rejects non-UTF-8 filesystem names; exercise them on Linux natively.
#[cfg(target_os = "linux")]
#[test]
fn caller_directory_transport_preserves_non_utf8_native_paths() {
    use std::os::unix::ffi::OsStringExt;

    let queue = Queue::new();
    queue.call(&["daemon", "start"]);
    let client = queue
        .dir
        .path()
        .join(std::ffi::OsString::from_vec(vec![b'c', 0xff]));
    fs::create_dir(&client).unwrap();
    fs::write(client.join("meta.json"), r#"{"native":true}"#).unwrap();

    let output = TestProcess::spawn(
        queue
            .cmd(&["task", "create", "native", "--metafile", "meta.json"])
            .current_dir(&client)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    )
    .output();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(get(&queue, "task", "native"), json!({"native":true}));
}

#[test]
fn broken_imports_fail_before_startup_and_preserve_existing_metadata() {
    let queue = Queue::new();
    let sources = [
        ("json", r#"{"x":1,"x":2}"#),
        ("json", "[]"),
        ("json", "{"),
        ("yaml", "x: !run command"),
        ("yaml", "true: value"),
        ("yaml", "x: .inf"),
        ("yaml", "x: {y: 1, y: 2}"),
        ("toml", "x=2026-11-10"),
        ("toml", "x=nan"),
        ("toml", "x=1\nx=2"),
        ("txt", "x=1"),
    ];

    for (index, (extension, source)) in sources.iter().enumerate() {
        let file = queue.dir.path().join(format!("invalid{index}.{extension}"));
        fs::write(&file, source).unwrap();

        assert_eq!(
            queue
                .raw(&[
                    "task",
                    "create",
                    "failed",
                    "--metafile",
                    file.to_str().unwrap()
                ])
                .status
                .code(),
            Some(2)
        );
        assert!(!queue.db.exists());
    }
    for arguments in [
        vec!["task", "create", "failed", "--meta", "x"],
        vec!["task", "create", "failed", "--meta", "a..b", "true"],
        vec![
            "task",
            "create",
            "failed",
            "--meta",
            "x",
            "{\"a\":1,\"a\":2}",
        ],
    ] {
        assert_eq!(queue.raw(&arguments).status.code(), Some(2));
        assert!(!queue.db.exists());
    }
    queue.call(&["task", "create", "job", "--meta", "keep", "true"]);
    let before = get(&queue, "task", "job");

    for (index, (extension, _)) in sources.iter().enumerate() {
        let file = queue.dir.path().join(format!("invalid{index}.{extension}"));

        assert_eq!(
            queue
                .raw(&["task", "job", "metafile", file.to_str().unwrap()])
                .status
                .code(),
            Some(2)
        );
        assert_eq!(get(&queue, "task", "job"), before);
    }
    assert_eq!(queue.state("job"), "registered");
    assert_eq!(queue.call(&["task", "list"]), "- job");
}

#[test]
fn independent_cli_key_patches_survive_without_lost_updates_or_resource_changes() {
    let queue = Queue::new();
    queue.task("job", &[]);
    let mut clients = Vec::new();

    for index in 0..12 {
        clients.push(TestProcess::spawn(
            queue
                .cmd(&["task", "job", "meta", &format!("keys.k{index}"), "true"])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped()),
        ));
    }
    for client in clients {
        let output = client.output();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let metadata = get(&queue, "task", "job");
    assert_eq!(metadata["keys"].as_object().unwrap().len(), 12);
    assert_eq!(queue.state("job"), "registered");
    assert_eq!(queue.call(&["topic", "list"]), "");
}

#[test]
fn metadata_edit_racing_start_is_committed_before_running_or_rejected() {
    let queue = Queue::new();
    queue.topic("cpu");

    for index in 0..6 {
        let name = format!("job{index}");
        queue.call(&[
            "task", "create", &name, "-t", "cpu", "--stop", "--meta", "keep", "true",
        ]);
        let start = TestProcess::spawn(
            queue
                .cmd(&["task", "start", &name])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped()),
        );
        let patch = TestProcess::spawn(
            queue
                .cmd(&["task", &name, "meta", "racing", "true"])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped()),
        );
        let patch = patch.output();

        assert!(start.output().status.success());
        assert_eq!(queue.state(&name), "running");
        assert_eq!(queue.usage("cpu"), 1);
        let metadata = get(&queue, "task", &name);
        assert_eq!(metadata["keep"], true);
        assert_eq!(metadata.get("racing").is_some(), patch.status.success());
        if !patch.status.success() {
            assert!(String::from_utf8_lossy(&patch.stderr).contains("cannot change while running"));
        }
        queue.call(&["topic", "cpu", "meta", "future", "true"]);
        assert_eq!(get(&queue, "task", &name), metadata);
        queue.call(&["task", "complete", &name]);
        assert_eq!(
            queue
                .raw(&["task", &name, "meta", "--unset", "keep"])
                .status
                .code(),
            Some(2)
        );
    }
}

#[test]
fn maximum_depth_and_large_values_survive_cli_daemon_sqlite_and_restart() {
    let queue = Queue::new();
    let file = queue.dir.path().join("deep.json");
    let deep = format!("{{\"nested\":{}null{}}}", "[".repeat(62), "]".repeat(62));
    fs::write(&file, &deep).unwrap();

    queue.call(&[
        "task",
        "create",
        "deep",
        "--metafile",
        file.to_str().unwrap(),
    ]);
    assert_eq!(queue.call(&["task", "deep", "meta"]), deep);
    let large = format!("{{\"payload\":\"{}\"}}", "x".repeat(1024 * 1024 - 40));
    fs::write(&file, &large).unwrap();
    queue.call(&["task", "deep", "metafile", file.to_str().unwrap()]);
    queue.call(&["daemon", "stop"]);
    assert_eq!(queue.call(&["task", "deep", "meta"]), large);

    fs::write(&file, "x".repeat(1024 * 1024 + 1)).unwrap();
    assert_eq!(
        queue
            .raw(&["task", "deep", "metafile", file.to_str().unwrap()])
            .status
            .code(),
        Some(2)
    );
    assert_eq!(queue.call(&["task", "deep", "meta"]), large);
}
