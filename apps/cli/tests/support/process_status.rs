use std::{
    thread,
    time::{Duration, Instant},
};
#[cfg(unix)]
fn alive(pid: u32) -> bool {
    std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stderr(std::process::Stdio::null())
        .status()
        .unwrap()
        .success()
}
#[cfg(windows)]
#[allow(unsafe_code)]
fn alive(pid: u32) -> bool {
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        System::Threading::{GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
    };
    // Handles are closed on every path; an exited process may still have a live handle.
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return false;
        }
        let mut code = 0;
        let queried = GetExitCodeProcess(handle, &mut code);
        CloseHandle(handle);
        queried != 0 && code == 259
    }
}
pub fn assert_exited(pid: u32) {
    let end = Instant::now() + Duration::from_secs(5);
    while alive(pid) {
        assert!(
            Instant::now() < end,
            "process {pid} survived worker cleanup"
        );
        thread::sleep(Duration::from_millis(10));
    }
}
