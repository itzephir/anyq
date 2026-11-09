mod checkpoint;
mod controllers;
mod error;
mod interrupt;
mod output;
mod paths;
mod presenters;
mod task_flow;
mod wait_for_task;

use anyq_config_api::ConfigKey;
use anyq_queue_api::{AutoCreatePolicy, format_name, parse_topic_name};
use checkpoint::checkpoint;
use clap::Parser;
use controllers::{Cli, ConfigCommand, Request};
use error::{Error, Result};
use std::{
    process::ExitCode,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

fn run(cli: Cli, cancelled: Arc<AtomicBool>) -> Result<()> {
    let request = Request::try_from(cli.command)?;
    checkpoint("cli.before-command", &cancelled)?;
    let config_path = paths::home()?.join(".anyq/config.toml");
    let mut configuration = anyq_config_impl::open(&config_path, cancelled.clone());

    if let Request::Config(command) = request {
        let message = match command {
            ConfigCommand::List => {
                let config = configuration.get_config.get()?;
                Some(
                    ConfigKey::ALL
                        .into_iter()
                        .filter_map(|key| {
                            config
                                .get(key)
                                .map(|value| format!("{}={value}", key.as_str()))
                        })
                        .collect::<Vec<_>>()
                        .join("\n"),
                )
            }
            ConfigCommand::Get(key) => Some(
                configuration
                    .get_value
                    .get(key)?
                    .ok_or(Error::MissingConfigValue)?
                    .to_string(),
            ),
            ConfigCommand::Set(request) => {
                configuration.set_value.set(request)?;
                None
            }
            ConfigCommand::Unset(key) => {
                configuration.unset_value.unset(key)?;
                None
            }
        };

        if let Some(message) = message {
            output::emit(&message)?;
        }

        return checkpoint("cli.after-command", &cancelled);
    }

    let config = configuration.get_config.get()?;
    let path = match cli.db {
        Some(path) => path,
        None => match config.db_path() {
            Some(value) => paths::configured_db(value)?,
            None => paths::default_db()?,
        },
    };

    if let Some(parent) = path.parent().filter(|path| !path.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }

    let policy = AutoCreatePolicy {
        tasks: config.task_autocreate(),
        topics: config.topic_autocreate(),
    };
    let mut cases = anyq_queue_impl::open_with_options(&path, policy, cancelled.clone())?;
    checkpoint("cli.before-operation", &cancelled)?;

    let message = match request {
        Request::Config(_) => unreachable!("configuration handled before opening the queue"),
        Request::GetTask { name } => Some(presenters::task(&cases.get_task.get(&name)?)),
        Request::GetTopicQueue { name } => {
            Some(presenters::topic_queue(&cases.get_topic_queue.get(&name)?))
        }
        Request::GetTopicCapacity { name } => {
            Some(cases.get_topic.get(&name)?.capacity.to_string())
        }
        Request::ListTasks => Some(
            cases
                .list_tasks
                .list()?
                .iter()
                .map(|name| format!("- {}", format_name(name)))
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        Request::ListTopics => Some(
            cases
                .list_topics
                .list()?
                .iter()
                .map(|topic| format!("- {}", format_name(&topic.name)))
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        Request::CreateTopic(topic) => {
            let topic = cases.create_topic.create(topic)?;
            Some(format!(
                "successfully registered topic '{}'",
                format_name(&topic.name)
            ))
        }
        Request::SetTopicCapacity(topic) => {
            let topic = cases.set_topic_capacity.set_capacity(topic)?;
            Some(format!(
                "capacity of '{}' set to {}",
                format_name(&topic.name),
                topic.capacity
            ))
        }
        Request::RegisterTask {
            request,
            wait,
            complete,
        } => {
            let input_name = request.name.clone();
            let task = cases.register_task.register(request)?;
            checkpoint("cli.after-registration", &cancelled)?;
            output::emit(&presenters::registered(task.name()))?;

            if wait {
                let completion = if complete {
                    Some(cases.complete_task.as_mut())
                } else {
                    None
                };
                task_flow::wait_then_complete(
                    &input_name,
                    cases.try_start_task.as_mut(),
                    completion,
                    &cancelled,
                )?;
            }

            None
        }
        Request::LinkTask {
            name,
            topic,
            wait,
            complete,
        } => {
            let task = cases.add_task_to_topic.add(&name, &topic)?;
            checkpoint("cli.after-link", &cancelled)?;
            output::emit(&presenters::added(task.name(), &parse_topic_name(&topic)?))?;

            if wait {
                let completion = if complete {
                    Some(cases.complete_task.as_mut())
                } else {
                    None
                };
                task_flow::wait_then_complete(
                    &name,
                    cases.try_start_task.as_mut(),
                    completion,
                    &cancelled,
                )?;
            }

            None
        }
        Request::WaitTask { name, complete } => {
            let completion = if complete {
                Some(cases.complete_task.as_mut())
            } else {
                None
            };
            task_flow::wait_then_complete(
                &name,
                cases.try_start_task.as_mut(),
                completion,
                &cancelled,
            )?;
            None
        }
        Request::CompleteTask { name, force } => Some(task_flow::complete(
            &name,
            force,
            cases.complete_task.as_mut(),
            &cancelled,
        )?),
    };

    if let Some(message) = message.filter(|message| !message.is_empty()) {
        output::emit(&message)?;
    }

    checkpoint("cli.after-command", &cancelled)
}

fn main() -> ExitCode {
    let cancelled = match interrupt::install() {
        Ok(cancelled) => cancelled,
        Err(error) => {
            eprintln!("error: {error}");
            return ExitCode::from(2);
        }
    };

    if checkpoint("cli.before-parse", &cancelled).is_err() {
        return ExitCode::from(130);
    }

    let arguments = controllers::normalize_arguments(std::env::args_os());
    let cli = match Cli::try_parse_from(arguments) {
        Ok(cli) => cli,
        Err(error) => {
            let _ = error.print();
            return ExitCode::from(if cancelled.load(Ordering::SeqCst) {
                130
            } else {
                error.exit_code() as u8
            });
        }
    };
    let result = run(cli, cancelled.clone());

    if cancelled.load(Ordering::SeqCst) {
        return ExitCode::from(130);
    }

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(Error::MissingConfigValue) => ExitCode::from(1),
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(error.exit_code())
        }
    }
}
