mod controllers;
mod presenters;
mod wait_for_task;

use anyq_queue_api::{Error, Result};
use clap::Parser;
use controllers::{Cli, Request, TaskCommand, TopicCommand};
use std::{io::Write, path::PathBuf, process::ExitCode};

fn default_db() -> Result<PathBuf> {
    #[cfg(windows)]
    let data = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| Error::Invalid("set ANYQ_DB or --db: LOCALAPPDATA is unavailable".into()))?;
    #[cfg(not(windows))]
    let data = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|path| PathBuf::from(path).join(".local/share")))
        .ok_or_else(|| {
            Error::Invalid("set ANYQ_DB or --db: HOME and XDG_DATA_HOME are unavailable".into())
        })?;

    Ok(data.join("anyq/queue.db"))
}

fn emit(message: &str) -> Result<()> {
    writeln!(std::io::stdout().lock(), "{message}")
        .map_err(|error| Error::Storage(error.to_string()))
}

fn run(cli: Cli) -> Result<()> {
    // Parse nested topic syntax before opening or creating any database.
    let request = Request::try_from(cli.command)?;
    let path = match cli.db {
        Some(path) => path,
        None => default_db()?,
    };

    if let Some(parent) = path.parent().filter(|path| !path.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|error| Error::Storage(error.to_string()))?;
    }

    let mut cases = anyq_queue_impl::open(&path)?;

    let message = match request {
        Request::Topic(command) => match command {
            TopicCommand::List => cases
                .list_topics
                .list()?
                .iter()
                .map(|topic| format!("- {}", topic.name))
                .collect::<Vec<_>>()
                .join("\n"),
            TopicCommand::Create(topic) => {
                let topic = cases.create_topic.create(topic)?;
                format!("successfully registered topic '{}'", topic.name)
            }
            TopicCommand::Capacity(topic) => {
                let topic = cases.set_topic_capacity.set_capacity(topic)?;
                format!("capacity of '{}' set to {}", topic.name, topic.capacity)
            }
            TopicCommand::Add { topic, task } => {
                cases.add_task_to_topic.add(&task, &topic)?;
                presenters::added(&task, &topic)
            }
        },
        Request::Task(command) => match command {
            TaskCommand::Add { name } => {
                cases.register_task.register(name.clone())?;
                presenters::registered(&name)
            }
            TaskCommand::Wait { name } => {
                wait_for_task::wait_for_task(&name, cases.try_start_task.as_mut())?
            }
            TaskCommand::Complete { name, force } => {
                cases.complete_task.complete(&name, force)?;
                presenters::completed(&name)
            }
        },
    };

    if !message.is_empty() {
        emit(&message)?;
    }

    Ok(())
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(2)
        }
    }
}
