use crate::support::{
    process::TestProcess,
    queue::{Queue, wait_for_file},
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output, Stdio},
};

static NEXT_GATE: AtomicUsize = AtomicUsize::new(0);

pub struct InterruptProcess {
    process: TestProcess,
    gate: PathBuf,
}

impl InterruptProcess {
    pub fn spawn(queue: &Queue, args: &[&str], checkpoint: &str, hit: usize) -> Self {
        let number = NEXT_GATE.fetch_add(1, Ordering::SeqCst);
        let gate = queue
            .dir
            .path()
            .join(format!("gate-{number}-{checkpoint}-{hit}"));
        let mut command = queue.cmd(args);
        command
            .env("ANYQ_TEST_CHECKPOINT", checkpoint)
            .env("ANYQ_TEST_CHECKPOINT_HIT", hit.to_string())
            .env("ANYQ_TEST_CHECKPOINT_DIR", &gate)
            .stdout(Stdio::piped())
            .stderr(fs::File::create(queue.log(checkpoint)).unwrap());

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x00000010);
        }

        let process = TestProcess::spawn(&mut command);
        let mut child = Self { process, gate };
        wait_for_file(&child.gate.join("ready"), Some(checkpoint));
        child.process.assert_running();

        child
    }

    pub fn interrupt(self) -> Output {
        let pid = self.process.child.as_ref().unwrap().id();
        let python = if cfg!(windows) { "python" } else { "python3" };
        let helper =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tooling/send_interrupt.py");
        let signal = Command::new(python)
            .arg(helper)
            .arg(pid.to_string())
            .output()
            .unwrap();

        assert!(
            signal.status.success(),
            "interrupt sender: {}",
            String::from_utf8_lossy(&signal.stderr)
        );
        let output = self.process.output();
        assert_eq!(output.status.code(), Some(130));

        output
    }

    pub fn release(self) -> Output {
        fs::write(self.gate.join("release"), "continue").unwrap();
        self.process.output()
    }
}
