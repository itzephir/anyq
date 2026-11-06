use std::{
    thread,
    time::{Duration, Instant},
};

#[cfg(unix)]
fn alive(pid: u32) -> bool {
    let output = std::process::Command::new("ps")
        .args(["-o", "stat=", "-p", &pid.to_string()])
        .output()
        .expect("failed to inspect process state with ps");
    let state = std::str::from_utf8(&output.stdout)
        .expect("ps returned a non-UTF-8 process state")
        .trim();

    // Both procps (Linux) and BSD ps (macOS) exit with 1 for an absent PID.
    let absent = output.status.code() == Some(1) && state.is_empty() && output.stderr.is_empty();

    assert!(
        output.status.success() || absent,
        "failed to inspect process {pid}: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // A zombie retains its PID until reaped, but no longer executes any code.
    !state.is_empty() && !state.starts_with(['Z', 'X'])
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

#[cfg(test)]
mod tests {
    #[test]
    fn current_process_is_alive() {
        assert!(super::alive(std::process::id()));
    }
}
