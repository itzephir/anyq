use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn call(home: &Path, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_anyq"))
        .args(arguments)
        .env("HOME", home)
        .env("USERPROFILE", home)
        .env_remove("ANYQ_DB")
        .current_dir(home)
        .output()
        .unwrap()
}

fn json(home: &Path, arguments: &[&str]) -> Value {
    let output = call(home, arguments);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn global_enablement_needs_no_queue_or_valid_shared_config() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path();
    let directory = home.join(".anyq/plugins");
    fs::create_dir_all(&directory).unwrap();
    fs::copy(
        env!("CARGO_BIN_EXE_anyq"),
        directory.join(format!("observer{}", std::env::consts::EXE_SUFFIX)),
    )
    .unwrap();
    let config = home.join(".anyq/config.toml");
    fs::write(&config, "invalid = [").unwrap();
    let custom = home.join("unused.db");
    let path = custom.to_str().unwrap();

    let listed = json(home, &["--db", path, "plugin", "list"]);
    let enabled = json(home, &["--db", path, "plugin", "enable", "observer"]);
    let restored = json(home, &["plugin", "list"]);
    let disabled = json(home, &["plugin", "disable", "observer"]);

    assert_eq!(listed[0]["enabled"], false);
    assert_eq!(enabled["enabled"], true);
    assert_eq!(restored[0]["enabled"], true);
    assert_eq!(disabled["enabled"], false);
    assert!(!custom.exists());
    assert_eq!(fs::read_to_string(&config).unwrap(), "invalid = [");
    let status = call(home, &["--db", path, "daemon", "status"]);
    assert!(status.status.success());
    assert_eq!(
        String::from_utf8(status.stdout).unwrap().trim(),
        "daemon is stopped"
    );
}

#[test]
fn enable_is_name_only_and_invalid_requests_preserve_the_installed_binary() {
    let temp = tempfile::tempdir().unwrap();
    let home = temp.path();
    let directory = home.join(".anyq/plugins");
    fs::create_dir_all(&directory).unwrap();
    let executable = directory.join(format!("observer{}", std::env::consts::EXE_SUFFIX));
    fs::copy(env!("CARGO_BIN_EXE_anyq"), &executable).unwrap();

    assert!(
        !call(
            home,
            &[
                "plugin",
                "enable",
                "observer",
                "untrusted-program",
                "argument"
            ]
        )
        .status
        .success()
    );
    assert!(!call(home, &["plugin", "enable", ".."]).status.success());
    assert!(
        !call(home, &["plugin", "enable", "missing"])
            .status
            .success()
    );

    assert!(executable.is_file());
    assert!(!directory.join(".catalog.toml").exists());
    assert_eq!(json(home, &["plugin", "list"])[0]["enabled"], false);
}
