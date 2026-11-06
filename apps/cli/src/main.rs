mod controllers;
mod presenters;
use anyq_queue_api::{Error, Result};
use anyq_worker_api::Run;
use clap::Parser;
use controllers::{Cli, Commands};
use std::{io::Write, path::PathBuf, process::ExitCode};

fn emit(value: serde_json::Value) -> Result<()> {
    writeln!(std::io::stdout().lock(), "{value}").map_err(|e| Error::Storage(e.to_string()))
}
fn default_db() -> Result<PathBuf> {
    let data = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/share")))
        .ok_or_else(|| {
            Error::Invalid("set ANYQ_DB or --db: HOME and XDG_DATA_HOME are unavailable".into())
        })?;
    Ok(data.join("anyq/queue.db"))
}
fn run(cli: Cli) -> Result<u8> {
    let path = match cli.db {
        Some(p) => p,
        None => default_db()?,
    };
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| Error::Storage(e.to_string()))?;
    }
    let mut queue = anyq_queue_impl::open(&path)?;
    match cli.command {
        Commands::Worker(args) => {
            let (settings, command, once) = args.domain()?;
            let mut worker = anyq_worker_impl::create(queue, command, settings)?;
            loop {
                let result = worker.run_one()?;
                if once || result != Run::Idle {
                    emit(presenters::run(result))?;
                }
                if result == Run::Stopped || worker.stopped() {
                    return Ok(130);
                }
                if once {
                    return Ok(if matches!(result, Run::Failed(_)) {
                        1
                    } else {
                        0
                    });
                }
                if result == Run::Idle {
                    worker.idle();
                }
            }
        }
        command => {
            emit(presenters::output(
                &queue.execute(controllers::command(command)?)?,
            ))?;
            Ok(0)
        }
    }
}
fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("{}", presenters::error(&error));
            ExitCode::from(2)
        }
    }
}
