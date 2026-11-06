use crate::{io, managed_process::ManagedProcess, platform};
use anyq_queue_api::{Assignment, Error, Result};
use anyq_worker_execution::{Process, Runner};
use std::{
    io::Write,
    process::{Command, Stdio},
    thread,
};
pub struct CommandRunner {
    command: Vec<String>,
}
impl CommandRunner {
    pub fn new(command: Vec<String>) -> Result<Self> {
        if command.first().is_none_or(|s| s.is_empty()) {
            return Err(Error::Invalid("worker command is required after --".into()));
        }
        Ok(Self { command })
    }
}
impl Runner for CommandRunner {
    fn start(&mut self, a: &Assignment) -> Result<Box<dyn Process>> {
        let mut cmd = Command::new(&self.command[0]);
        cmd.args(&self.command[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::from(std::io::stderr()))
            .stderr(Stdio::inherit())
            .env("ANYQ_MESSAGE_ID", a.message.id.to_string())
            .env("ANYQ_TOPIC", &a.message.topic)
            .env("ANYQ_GROUP", &a.delivery.group)
            .env(
                "ANYQ_CONSUMER",
                a.delivery.consumer.as_deref().unwrap_or_default(),
            )
            .env("ANYQ_ATTEMPT", a.delivery.attempts.to_string());
        let mut tree = platform::ChildTree::spawn(&mut cmd).map_err(io)?;
        let stdin = tree.child.stdin.take();
        let payload = a.message.payload.clone();
        // A command may never read stdin; pipe backpressure must not block heartbeats.
        thread::spawn(move || {
            if let Some(mut pipe) = stdin {
                let _ = pipe.write_all(payload.as_bytes());
            }
        });
        Ok(Box::new(ManagedProcess { tree }))
    }
}
