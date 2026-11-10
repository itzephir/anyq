use std::{
    fs, io::Write, net::TcpListener, path::PathBuf, process::Command, thread, time::Duration,
};

pub fn run() -> bool {
    let arguments: Vec<_> = std::env::args().collect();
    let child = arguments
        .get(1)
        .is_some_and(|mode| mode == "worker-child" || mode == "worker-chain");
    let configured = arguments.get(1).is_some_and(|mode| mode == "worker");
    if arguments.len() > 1 && !child && !configured {
        return false;
    }
    let path = arguments
        .get(2)
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("WORKER_CONFIG").map(PathBuf::from))
        .expect("worker configuration");
    let value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let marker = PathBuf::from(value["marker"].as_str().unwrap());
    let release = marker.with_extension("release");
    if child {
        let remaining = arguments
            .get(3)
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0);
        if remaining > 0 {
            let mut next = Command::new(std::env::current_exe().unwrap())
                .args([
                    "worker-chain",
                    path.to_str().unwrap(),
                    &(remaining - 1).to_string(),
                ])
                .spawn()
                .unwrap();
            thread::spawn(move || {
                let _ = next.wait();
            });
            return true;
        }
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        fs::write(
            marker.with_extension("port"),
            listener.local_addr().unwrap().port().to_string(),
        )
        .unwrap();
        while !release.exists() {
            thread::sleep(Duration::from_millis(10));
        }
        drop(listener);
        fs::write(marker.with_extension("child-exited"), b"exited").unwrap();
        return true;
    }
    fs::write(&marker, b"started").unwrap();
    fs::write(
        marker.with_extension("cwd"),
        std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .as_bytes(),
    )
    .unwrap();
    if value["descendant"] == true {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "worker-chain",
                path.to_str().unwrap(),
                &value["chain_depth"].as_u64().unwrap_or(0).to_string(),
            ])
            .spawn()
            .unwrap();
        std::thread::spawn(move || {
            let _ = child.wait();
        });
        if value["chain_depth"].as_u64().unwrap_or(0) == 0 {
            while !marker.with_extension("port").exists() {
                thread::sleep(Duration::from_millis(10));
            }
        }
        if value["leader_exits"] != true {
            while !release.exists() {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
    if value["hold"] == true {
        while !release.exists() {
            thread::sleep(Duration::from_millis(10));
        }
    }
    let bytes = value["output_bytes"].as_u64().unwrap_or(0) as usize;
    let message = vec![b'o'; bytes];
    std::io::stdout().write_all(&message).unwrap();
    std::io::stderr().write_all(&message).unwrap();
    fs::write(marker.with_extension("exited"), b"exited").unwrap();
    std::process::exit(value["code"].as_i64().unwrap_or(0) as i32);
}
