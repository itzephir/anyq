use crate::support::process::TestProcess;
use std::process::{Command, Output, Stdio};

impl TestProcess {
    pub fn interrupt(mut self) -> Output {
        let pid = self.child.as_ref().unwrap().id();
        let python = if cfg!(windows) { "python" } else { "python3" };
        let helper = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tooling/send_interrupt.py");
        let mut command = Command::new(python);
        command
            .arg(helper)
            .arg(pid.to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let signal = crate::support::process_spawn::spawn(&mut command)
            .unwrap()
            .wait_with_output()
            .unwrap();
        assert!(
            signal.status.success(),
            "interrupt sender: {}",
            String::from_utf8_lossy(&signal.stderr)
        );
        self.wait_for_exit();
        let output = self.output();
        assert_eq!(output.status.code(), Some(130));

        output
    }
}
