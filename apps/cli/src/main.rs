mod controllers;
mod presenters;
mod wait_for_task;

use anyq_queue_api::{Error, Result};
use clap::Parser;
use controllers::{Cli, Request};
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
        Request::GetTask { name } => presenters::task(&cases.get_task.get(&name)?),
        Request::GetTopicQueue { name } => {
            presenters::topic_queue(&cases.get_topic_queue.get(&name)?)
        }
        Request::GetTopicCapacity { name } => cases.get_topic.get(&name)?.capacity.to_string(),
        Request::ListTasks => cases
            .list_tasks
            .list()?
            .iter()
            .map(|name| format!("- {name}"))
            .collect::<Vec<_>>()
            .join("\n"),
        Request::ListTopics => cases
            .list_topics
            .list()?
            .iter()
            .map(|topic| format!("- {}", topic.name))
            .collect::<Vec<_>>()
            .join("\n"),
        Request::CreateTopic(topic) => {
            let topic = cases.create_topic.create(topic)?;
            format!("successfully registered topic '{}'", topic.name)
        }
        Request::SetTopicCapacity(topic) => {
            let topic = cases.set_topic_capacity.set_capacity(topic)?;
            format!("capacity of '{}' set to {}", topic.name, topic.capacity)
        }
        Request::RegisterTask {
            request,
            wait,
            complete,
        } => {
            let task = cases.register_task.register(request)?;
            emit(&presenters::registered(task.name()))?;

            if wait {
                let message =
                    wait_for_task::wait_for_task(task.name(), cases.try_start_task.as_mut())?;
                emit(&message)?;
            }

            if complete {
                cases.complete_task.complete(task.name(), false)?;
                emit(&presenters::completed(task.name()))?;
            }

            return Ok(());
        }
        Request::LinkTask { name, topic } => {
            cases.add_task_to_topic.add(&name, &topic)?;
            presenters::added(&name, &topic)
        }
        Request::WaitTask { name } => {
            wait_for_task::wait_for_task(&name, cases.try_start_task.as_mut())?
        }
        Request::CompleteTask { name, force } => {
            cases.complete_task.complete(&name, force)?;
            presenters::completed(&name)
        }
    };

    if !message.is_empty() {
        emit(&message)?;
    }

    Ok(())
}

fn main() -> ExitCode {
    let arguments = controllers::expand_task_add_flags(std::env::args_os());

    match run(Cli::parse_from(arguments)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(2)
        }
    }
}
