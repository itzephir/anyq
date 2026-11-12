use std::{
    io::Read,
    process::{Child, Command, Output},
    thread,
    time::{Duration, Instant},
};

/// Keep test failures from leaking waiting CLI or external-caller processes.
pub struct TestProcess {
    pub child: Option<Child>,
    label: String,
}

impl TestProcess {
    pub fn spawn(command: &mut Command) -> Self {
        let label = format!(
            "{:?} {:?}",
            command.get_program(),
            command.get_args().collect::<Vec<_>>()
        );

        Self {
            child: Some(super::process_spawn::spawn(command).unwrap()),
            label,
        }
    }

    pub fn assert_running(&mut self) {
        assert!(self.child.as_mut().unwrap().try_wait().unwrap().is_none());
    }

    pub fn output(mut self) -> Output {
        let child = self.child.as_mut().unwrap();
        let stdout = drain(child.stdout.take());
        let stderr = drain(child.stderr.take());

        self.wait_for_exit();
        let status = self.child.take().unwrap().wait().unwrap();

        Output {
            status,
            stdout: collect(stdout, "stdout"),
            stderr: collect(stderr, "stderr"),
        }
    }

    pub fn wait_for_exit(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(20);

        while self.child.as_mut().unwrap().try_wait().unwrap().is_none() {
            #[cfg(target_os = "macos")]
            if Instant::now() >= deadline {
                report_timeout(self.child.as_ref().unwrap());
            }

            assert!(
                Instant::now() < deadline,
                "CLI did not finish within the test deadline: {}",
                self.label
            );
            thread::sleep(Duration::from_millis(10));
        }
    }

    pub fn stop(mut self) {
        if let Some(mut child) = self.child.take() {
            if child.try_wait().unwrap().is_none() {
                child.kill().unwrap();
            }

            child.wait().unwrap();
        }
    }
}

#[cfg(target_os = "macos")]
fn report_timeout(child: &Child) {
    let mut command = Command::new("/usr/bin/sample");
    command
        .arg(child.id().to_string())
        .args(["1", "1"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    match super::process_spawn::spawn(&mut command).and_then(|child| child.wait_with_output()) {
        Ok(output) => eprintln!(
            "CLI timeout sample:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
        Err(error) => eprintln!("Could not sample timed-out CLI: {error}"),
    }
}

fn collect(reader: Option<thread::JoinHandle<Vec<u8>>>, label: &str) -> Vec<u8> {
    let Some(reader) = reader else {
        return Vec::new();
    };
    let deadline = Instant::now() + Duration::from_secs(10);

    while !reader.is_finished() {
        assert!(
            Instant::now() < deadline,
            "{label} stayed open after the CLI process exited"
        );
        thread::sleep(Duration::from_millis(10));
    }

    reader.join().unwrap()
}

fn drain<R: Read + Send + 'static>(reader: Option<R>) -> Option<thread::JoinHandle<Vec<u8>>> {
    reader.map(|mut reader| {
        thread::spawn(move || {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).unwrap();

            bytes
        })
    })
}

impl Drop for TestProcess {
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
