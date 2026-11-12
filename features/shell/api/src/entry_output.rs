#[derive(Debug, Default, Clone)]
pub struct EntryOutput {
    pub stdout: String,
    pub stderr: String,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
}
