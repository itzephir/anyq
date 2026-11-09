use crate::presenters;
use crate::{checkpoint::checkpoint, error::Result};
use anyq_queue_api::{StartOutcome, TryStartTask, parse_name};
use std::{io::Write, sync::atomic::AtomicBool, thread, time::Duration};

pub fn wait_for_task(
    name: &str,
    start: &mut dyn TryStartTask,
    cancelled: &AtomicBool,
) -> Result<String> {
    let display_name = parse_name(name)?;
    let mut announced = false;

    loop {
        checkpoint("cli.before-admission", cancelled)?;
        match start.try_start(name)? {
            StartOutcome::Started => {
                checkpoint("cli.after-admission", cancelled)?;
                return Ok(presenters::started(&display_name));
            }
            StartOutcome::Busy => {
                if !announced {
                    writeln!(
                        std::io::stderr().lock(),
                        "{}",
                        presenters::waiting(&display_name)
                    )?;
                    announced = true;
                }
            }
        }

        checkpoint("cli.waiting", cancelled)?;
        // Waiting is an outer CLI concern; no transaction or partial reservation is retained.
        thread::sleep(Duration::from_millis(100));
    }
}
