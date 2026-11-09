use crate::{checkpoint::checkpoint, error::Result, output, presenters};
use anyq_queue_api::{Error, StartTask, StartTaskOutcome, format_name, parse_name};
use std::{sync::atomic::AtomicBool, thread, time::Duration};

pub fn start_task(
    name: &str,
    wait: bool,
    start: &mut dyn StartTask,
    cancelled: &AtomicBool,
) -> Result<String> {
    let display_name = parse_name(name)?;
    let mut announced = false;

    loop {
        checkpoint("cli.before-admission", cancelled)?;
        let outcome = start.start(name)?;

        match outcome {
            StartTaskOutcome::Started | StartTaskOutcome::AlreadyRunning => {
                checkpoint("cli.after-admission", cancelled)?;

                return Ok(if outcome == StartTaskOutcome::AlreadyRunning {
                    format!("'{}' is already running", format_name(&display_name))
                } else {
                    presenters::started(&display_name)
                });
            }
            StartTaskOutcome::Busy if !wait => {
                return Err(Error::Conflict(format!(
                    "'{}' cannot start: resources or queue position are unavailable",
                    format_name(&display_name)
                ))
                .into());
            }
            StartTaskOutcome::Busy => {
                if !announced {
                    output::emit_error(&presenters::waiting(&display_name), cancelled)?;
                    announced = true;
                }
            }
        }

        checkpoint("cli.waiting", cancelled)?;
        thread::sleep(Duration::from_millis(100));
    }
}
