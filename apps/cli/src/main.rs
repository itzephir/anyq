mod checkpoint;
mod controllers;
mod daemon_presenters;
mod error;
mod interrupt;
mod metadata_input;
mod output;
mod paths;
mod plugin_executable;
mod presenters;
mod start_task;
mod task_flow;
mod wait_for_task;

use anyq_config_api::ConfigKey;
use anyq_queue_api::{
    AddTaskToTopicsRequest, AddTasksToTopicRequest, AutoCreatePolicy, QueuePolicy, format_name,
    parse_topic_name,
};
use checkpoint::checkpoint;
use clap::Parser;
use controllers::{Cli, ConfigCommand, DaemonCommand, Request};
use error::{Error, Result};
use std::{
    process::ExitCode,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

fn run(cli: Cli, cancelled: Arc<AtomicBool>) -> Result<()> {
    let metadata_input = metadata_input::MetadataInput {
        parse_value: anyq_queue_metadata::parse_value,
        parse_json: anyq_queue_metadata::parse_json,
        parse_yaml: anyq_queue_metadata::parse_yaml,
        parse_toml: anyq_queue_metadata::parse_toml,
    };
    let mut request = Request::parse(cli.command, &metadata_input)?;
    if let Request::PluginRuntime(mode) = request {
        match mode {
            controllers::PluginRuntimeMode::Timeout => {
                anyq_timeout_impl::run(&mut std::io::stdin().lock(), &mut std::io::stdout().lock())?
            }
            controllers::PluginRuntimeMode::Shell => anyq_shell_runtime::run(
                std::io::BufReader::new(std::io::stdin()),
                &mut std::io::stdout().lock(),
                &mut anyq_shell_process::NativeFactory,
            )?,
        }
        return Ok(());
    }
    if let Request::Plugin(controllers::PluginCommand::Enable {
        name, executable, ..
    }) = &mut request
    {
        anyq_plugin_api::validate_plugin_name(name)?;
        *executable = plugin_executable::resolve(executable)?;
    }
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
            output::emit(&message, &cancelled)?;
        }

        return checkpoint("cli.after-command", &cancelled);
    }

    if let Request::Daemon(DaemonCommand::Status | DaemonCommand::Stop) = &request {
        if let Some(path) = &cli.db {
            let pid = if matches!(request, Request::Daemon(DaemonCommand::Stop)) {
                anyq_daemon_impl::daemon_stop(path, cancelled.clone())?;
                None
            } else {
                anyq_daemon_impl::daemon_status(path, cancelled.clone())?
            };
            output::emit(&daemon_presenters::status(pid), &cancelled)?;

            return Ok(());
        }
    }

    let config = configuration.get_config.get()?;
    let path = match cli.db {
        Some(path) => path,
        None => match config.db_path() {
            Some(value) => paths::configured_db(value)?,
            None => paths::default_db()?,
        },
    };

    let policy = QueuePolicy {
        autocreate: AutoCreatePolicy {
            tasks: config.task_autocreate(),
            topics: config.topic_autocreate(),
        },
        task_autostart: config.task_autostart(),
    };
    if let Request::Daemon(command) = request {
        let pid = match command {
            DaemonCommand::Run => return Ok(anyq_daemon_impl::run(&path, cancelled)?),
            DaemonCommand::Start => Some(anyq_daemon_impl::daemon_start(
                &path,
                policy,
                cancelled.clone(),
            )?),
            DaemonCommand::Status => anyq_daemon_impl::daemon_status(&path, cancelled.clone())?,
            DaemonCommand::Stop => {
                anyq_daemon_impl::daemon_stop(&path, cancelled.clone())?;
                None
            }
        };
        output::emit(&daemon_presenters::status(pid), &cancelled)?;

        return Ok(());
    }

    if let Request::Plugin(command) = request {
        let mut plugins = anyq_daemon_impl::connect_plugins(&path, policy, cancelled.clone())?;
        use controllers::PluginCommand;
        let message = match command {
            PluginCommand::Enable {
                name,
                executable,
                arguments,
            } => serde_json::to_string_pretty(&anyq_plugin_wire::PluginStatusDto::from(
                plugins
                    .enable
                    .enable(anyq_plugin_api::EnablePluginRequest {
                        name,
                        executable,
                        arguments,
                    })?,
            )),
            PluginCommand::List => serde_json::to_string_pretty(
                &plugins
                    .list
                    .list()?
                    .into_iter()
                    .map(anyq_plugin_wire::PluginStatusDto::from)
                    .collect::<Vec<_>>(),
            ),
            PluginCommand::Status { name } => serde_json::to_string_pretty(
                &anyq_plugin_wire::PluginStatusDto::from(plugins.status.status(&name)?),
            ),
            PluginCommand::Disable { name } => serde_json::to_string_pretty(
                &anyq_plugin_wire::PluginStatusDto::from(plugins.disable.disable(&name)?),
            ),
            PluginCommand::Inspect { name } => serde_json::to_string_pretty(
                &anyq_daemon_impl::inspect_managed(&path, policy, cancelled.clone(), name)?,
            ),
            PluginCommand::Resolve {
                name,
                action_id,
                confirmed_cleanup,
            } => serde_json::to_string_pretty(&anyq_daemon_impl::resolve_managed(
                &path,
                policy,
                cancelled.clone(),
                name,
                action_id,
                confirmed_cleanup,
            )?),
        }
        .map_err(|error| Error::Invalid(error.to_string()))?;
        output::emit(&message, &cancelled)?;

        return Ok(());
    }

    let mut cases = anyq_daemon_impl::connect(&path, policy, cancelled.clone())?;
    checkpoint("cli.before-operation", &cancelled)?;

    let message = match request {
        Request::GetTaskMetadata(request) => Some(anyq_queue_metadata::to_json(
            &cases.get_task_metadata.get(request)?,
        )?),
        Request::SetTaskMetadata(request) => {
            cases.set_task_metadata.set(request)?;
            None
        }
        Request::UnsetTaskMetadata(request) => {
            cases.unset_task_metadata.unset(request)?;
            None
        }
        Request::ReplaceTaskMetadata(request) => {
            cases.replace_task_metadata.replace(request)?;
            None
        }
        Request::GetTopicMetadata(request) => Some(anyq_queue_metadata::to_json(
            &cases.get_topic_metadata.get(request)?,
        )?),
        Request::SetTopicMetadata(request) => {
            cases.set_topic_metadata.set(request)?;
            None
        }
        Request::UnsetTopicMetadata(request) => {
            cases.unset_topic_metadata.unset(request)?;
            None
        }
        Request::ReplaceTopicMetadata(request) => {
            cases.replace_topic_metadata.replace(request)?;
            None
        }
        Request::PluginRuntime(_) => unreachable!("plugin runtime handled before queue operations"),
        Request::Plugin(_) => unreachable!("plugin operations handled before queue operations"),
        Request::Config(_) => unreachable!("configuration handled before opening the queue"),
        Request::Daemon(_) => unreachable!("daemon lifecycle handled before queue operations"),
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
        Request::SetTopicAutostart { name, enabled } => {
            let topic = cases.set_topic_autostart.set_autostart(&name, enabled)?;
            Some(format!(
                "autostart of '{}' set to {}",
                format_name(&topic.name),
                if enabled { "on" } else { "off" }
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
            output::emit(&presenters::registered(task.name()), &cancelled)?;

            if wait {
                let completion = if complete {
                    Some(cases.complete_task.as_mut())
                } else {
                    None
                };
                task_flow::wait_then_complete(
                    &input_name,
                    cases.check_task_running.as_mut(),
                    completion,
                    &cancelled,
                )?;
            }

            None
        }
        Request::LinkTask {
            name,
            topics,
            wait,
            complete,
        } => {
            let task = cases.add_task_to_topics.add(AddTaskToTopicsRequest {
                task: name.clone(),
                topics: topics.clone(),
            })?;
            checkpoint("cli.after-link", &cancelled)?;
            for topic in topics {
                output::emit(
                    &presenters::added(task.name(), &parse_topic_name(&topic)?),
                    &cancelled,
                )?;
            }

            if wait {
                let completion = if complete {
                    Some(cases.complete_task.as_mut())
                } else {
                    None
                };
                task_flow::wait_then_complete(
                    &name,
                    cases.check_task_running.as_mut(),
                    completion,
                    &cancelled,
                )?;
            }

            None
        }
        Request::LinkTasks {
            names,
            topic,
            wait,
            complete,
        } => {
            let tasks = cases.add_tasks_to_topic.add(AddTasksToTopicRequest {
                tasks: names,
                topic: topic.clone(),
            })?;
            checkpoint("cli.after-link", &cancelled)?;

            for task in &tasks {
                output::emit(
                    &presenters::added(task.name(), &parse_topic_name(&topic)?),
                    &cancelled,
                )?;
            }

            if wait {
                for task in tasks {
                    let completion = if complete {
                        Some(cases.complete_task.as_mut())
                    } else {
                        None
                    };
                    task_flow::wait_then_complete(
                        &format_name(task.name()),
                        cases.check_task_running.as_mut(),
                        completion,
                        &cancelled,
                    )?;
                }
            }

            None
        }
        Request::StartTask {
            name,
            wait,
            complete,
        } => {
            let completion = if complete {
                Some(cases.complete_task.as_mut())
            } else {
                None
            };
            task_flow::start_then_complete(
                &name,
                wait,
                cases.start_task.as_mut(),
                completion,
                &cancelled,
            )?;
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
                cases.check_task_running.as_mut(),
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
        output::emit(&message, &cancelled)?;
    }

    checkpoint("cli.after-command", &cancelled)
}

fn main() -> ExitCode {
    let cancelled = match interrupt::install() {
        Ok(cancelled) => cancelled,
        Err(error) => {
            let _ = output::emit_error(&format!("error: {error}"), &AtomicBool::new(false));
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
            let code = error.exit_code() as u8;
            let _ = output::write(move || error.print(), &cancelled);
            return ExitCode::from(if cancelled.load(Ordering::SeqCst) {
                130
            } else {
                code
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
            let _ = output::emit_error(&format!("error: {error}"), &cancelled);
            ExitCode::from(if cancelled.load(Ordering::SeqCst) {
                130
            } else {
                error.exit_code()
            })
        }
    }
}
