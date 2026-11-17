use anyq_queue_api::{Error, Result};
use std::process::ExitCode;

fn run() -> Result<()> {
    if std::env::args_os().len() != 1 {
        return Err(Error::Invalid("timeout plugin takes no arguments".into()));
    }

    anyq_timeout_impl::run(&mut std::io::stdin().lock(), &mut std::io::stdout().lock())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("timeout plugin: {error}");
            ExitCode::from(2)
        }
    }
}
