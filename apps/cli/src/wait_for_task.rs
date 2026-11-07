use crate::presenters;
use anyq_queue_api::{Error, Result, StartOutcome, TryStartTask};
use std::{io::Write, thread, time::Duration};

pub fn wait_for_task(name: &str, start: &mut dyn TryStartTask) -> Result<String> {
    let mut announced = false;

    loop {
        match start.try_start(name)? {
            StartOutcome::Started => return Ok(presenters::started(name)),
            StartOutcome::Busy => {
                if !announced {
                    writeln!(std::io::stderr().lock(), "{}", presenters::waiting(name))
                        .map_err(|error| Error::Storage(error.to_string()))?;
                    announced = true;
                }
            }
        }

        // Waiting is an outer CLI concern; no transaction or partial reservation is retained.
        thread::sleep(Duration::from_millis(100));
    }
}
