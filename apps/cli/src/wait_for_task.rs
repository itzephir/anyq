use crate::{checkpoint::checkpoint, error::Result};
use crate::{output, presenters};
use anyq_queue_api::{CheckTaskRunning, parse_name};
use std::{sync::atomic::AtomicBool, thread, time::Duration};

pub fn wait_for_task(
    name: &str,
    status: &mut dyn CheckTaskRunning,
    cancelled: &AtomicBool,
) -> Result<String> {
    let display_name = parse_name(name)?;
    let mut announced = false;

    loop {
        checkpoint("cli.before-admission", cancelled)?;
        match status.check_running(name)? {
            true => {
                checkpoint("cli.after-admission", cancelled)?;
                return Ok(presenters::started(&display_name));
            }
            false => {
                if !announced {
                    output::emit_error(&presenters::waiting(&display_name), cancelled)?;
                    announced = true;
                }
            }
        }

        checkpoint("cli.waiting", cancelled)?;
        // Waiting is an outer CLI concern; no transaction or partial reservation is retained.
        thread::sleep(Duration::from_millis(100));
    }
}
