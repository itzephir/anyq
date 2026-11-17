use anyq_queue_api::{Error, Result};
use std::process::ExitCode;

fn run() -> Result<()> {
    let arguments: Vec<_> = std::env::args_os().skip(1).collect();
    if arguments.len() == 2 && arguments[0] == "plugin-runtime" && arguments[1] == "shell-anchor" {
        return anyq_shell_process::run_anchor();
    }
    if !arguments.is_empty() {
        return Err(Error::Invalid("shell plugin takes no arguments".into()));
    }

    anyq_shell_runtime::run(
        std::io::BufReader::new(std::io::stdin()),
        &mut std::io::stdout().lock(),
        &mut anyq_shell_process::NativeFactory::new(
            std::env::current_exe().map_err(|error| Error::Storage(error.to_string()))?,
        ),
    )
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("shell plugin: {error}");
            ExitCode::from(2)
        }
    }
}
