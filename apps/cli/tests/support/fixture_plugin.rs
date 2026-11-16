use crate::support::queue::Queue;
use std::{fs, path::Path};

pub fn enable(queue: &Queue, name: &str, mode: &str, log: &Path) {
    let directory = queue.dir.path().join(".anyq/plugins");
    fs::create_dir_all(&directory).unwrap();
    let executable = directory.join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
    if !executable.exists() {
        fs::copy(env!("CARGO_BIN_EXE_anyq-plugin-fixture"), &executable).unwrap();
    }
    fs::write(
        executable.with_extension("plugin.json"),
        serde_json::to_vec(&serde_json::json!({"mode":mode,"log":log})).unwrap(),
    )
    .unwrap();
    queue.call(&["plugin", "enable", name]);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    loop {
        let status: serde_json::Value =
            serde_json::from_str(&queue.call(&["plugin", "status", name])).unwrap();
        if status["running"] == true || status["failures"].as_u64().unwrap() > 0 {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "fixture plugin did not start"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
