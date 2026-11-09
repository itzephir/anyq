use crate::support::{
    process::TestProcess,
    queue::{Queue, wait_for_file},
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{
    fs,
    io::Read,
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
            .stderr(Stdio::piped());

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

    pub fn interrupt(mut self) -> Output {
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
        self.process.wait_for_exit();
        let output = self.process.output();
        assert_eq!(output.status.code(), Some(130));

        output
    }

    pub fn release(self) -> Output {
        self.resume();
        self.process.output()
    }

    pub fn resume(&self) {
        fs::write(self.gate.join("release"), "continue").unwrap();
    }

    pub fn read_stdout_byte(&mut self) -> u8 {
        let mut byte = [0];
        self.process
            .child
            .as_mut()
            .unwrap()
            .stdout
            .as_mut()
            .unwrap()
            .read_exact(&mut byte)
            .unwrap();

        byte[0]
    }

    pub fn read_stderr_byte(&mut self) -> u8 {
        let mut byte = [0];
        self.process
            .child
            .as_mut()
            .unwrap()
            .stderr
            .as_mut()
            .unwrap()
            .read_exact(&mut byte)
            .unwrap();

        byte[0]
    }
}
