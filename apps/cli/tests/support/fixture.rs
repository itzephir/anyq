use std::{
    io::{Read, Write},
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
pub fn arguments() -> Vec<String> {
    vec![
        std::env::current_exe()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned(),
        "--exact".into(),
        "fixture_process".into(),
        "--nocapture".into(),
    ]
}
pub fn wait_for_file(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !path.exists() {
        assert!(Instant::now() < deadline, "fixture did not start");
        thread::sleep(Duration::from_millis(10));
    }
}
pub fn run() {
    let Ok(mode) = std::env::var("ANYQ_FIXTURE_MODE") else {
        return;
    };
    match mode.as_str() {
        "echo" => {
            let mut input = String::new();
            std::io::stdin().read_to_string(&mut input).unwrap();
            assert!(std::env::var("ANYQ_MESSAGE_ID").is_ok());
            print!("{input}command-output");
            std::io::stdout().flush().unwrap();
        }
        "fail" => std::process::exit(7),
        "sleep" => {
            let mut input = String::new();
            std::io::stdin().read_to_string(&mut input).unwrap();
            std::fs::write(std::env::var_os("ANYQ_FIXTURE_MARKER").unwrap(), "ready").unwrap();
            thread::sleep(Duration::from_secs(4));
        }
        "leaf" => {
            std::fs::write(
                std::env::var_os("ANYQ_FIXTURE_PID").unwrap(),
                std::process::id().to_string(),
            )
            .unwrap();
            thread::sleep(Duration::from_secs(30));
        }
        "tree" | "orphan" => {
            let args = arguments();
            let mut child = Command::new(&args[0])
                .args(&args[1..])
                .env("ANYQ_FIXTURE_MODE", "leaf")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            let pid = std::env::var_os("ANYQ_FIXTURE_PID").unwrap();
            wait_for_file(Path::new(&pid));
            std::fs::write(std::env::var_os("ANYQ_FIXTURE_MARKER").unwrap(), "ready").unwrap();
            if mode == "orphan" {
                std::process::exit(0);
            }
            child.wait().unwrap();
        }
        #[cfg(windows)]
        "console-event" => send_console_event(),
        _ => panic!("unknown fixture mode {mode}"),
    }
    std::process::exit(0);
}
#[cfg(windows)]
#[allow(unsafe_code)]
fn send_console_event() {
    use windows_sys::Win32::System::Console::{
        AttachConsole, FreeConsole, GenerateConsoleCtrlEvent, SetConsoleCtrlHandler,
    };
    unsafe extern "system" fn ignore(_: u32) -> i32 {
        1
    }
    let pid = std::env::var("ANYQ_FIXTURE_WORKER")
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let event = std::env::var("ANYQ_FIXTURE_EVENT")
        .unwrap()
        .parse::<u32>()
        .unwrap();
    // The worker has a dedicated console; this helper never signals the test runner's console.
    unsafe {
        FreeConsole();
        assert_ne!(AttachConsole(pid), 0);
        assert_ne!(SetConsoleCtrlHandler(Some(ignore), 1), 0);
        assert_ne!(GenerateConsoleCtrlEvent(event, 0), 0);
    }
    thread::sleep(Duration::from_millis(200));
}
