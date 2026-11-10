pub fn status(pid: Option<u32>) -> String {
    match pid {
        Some(pid) => format!("daemon is running (pid {pid})"),
        None => "daemon is stopped".into(),
    }
}
