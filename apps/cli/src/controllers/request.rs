use super::{Commands, ConfigCommand, TaskCommand, TopicCommand, config_action::ConfigAction};
use crate::error::{Error, Result};
use crate::metadata_input::MetadataInput;
use anyq_config_api::{ConfigKey, ConfigValue, SetConfigRequest};
use anyq_queue_api::{RegisterTaskRequest, StartPreference, Topic};

/// Parsed CLI syntax, routed only in the outer composition root.
pub enum Request {
    Plugin(super::PluginCommand),
    GetTaskMetadata(anyq_queue_api::GetTaskMetadataRequest),
    SetTaskMetadata(anyq_queue_api::SetTaskMetadataRequest),
    UnsetTaskMetadata(anyq_queue_api::UnsetTaskMetadataRequest),
    ReplaceTaskMetadata(anyq_queue_api::ReplaceTaskMetadataRequest),
    GetTopicMetadata(anyq_queue_api::GetTopicMetadataRequest),
    SetTopicMetadata(anyq_queue_api::SetTopicMetadataRequest),
    UnsetTopicMetadata(anyq_queue_api::UnsetTopicMetadataRequest),
    ReplaceTopicMetadata(anyq_queue_api::ReplaceTopicMetadataRequest),
    Daemon(super::DaemonCommand),
    Config(ConfigCommand),
    ListTopics,
    ListTasks,
    GetTask {
        name: String,
    },
    GetTopicQueue {
        name: String,
    },
    GetTopicCapacity {
        name: String,
    },
    CreateTopic(Topic),
    SetTopicCapacity(Topic),
    SetTopicAutostart {
        name: String,
        enabled: bool,
    },
    RegisterTask {
        request: RegisterTaskRequest,
        wait: bool,
        complete: bool,
    },
    LinkTask {
        name: String,
        topics: Vec<String>,
        wait: bool,
        complete: bool,
    },
    LinkTasks {
        names: Vec<String>,
        topic: String,
        wait: bool,
        complete: bool,
    },
    StartTask {
        name: String,
        wait: bool,
        complete: bool,
    },
    WaitTask {
        name: String,
        complete: bool,
    },
    CompleteTask {
        name: String,
        force: bool,
    },
}

impl Request {
    fn task_metadata(
        args: super::metadata_args::MetadataArgs,
        input: &MetadataInput,
    ) -> Result<Self> {
        let name = args.name;
        if let Some(key) = args.unset.as_deref().or(args.key.as_deref()) {
            anyq_queue_api::MetadataPath::parse(key)?;
        }

        Ok(match (args.unset, args.key, args.value) {
            (Some(key), None, None) => {
                Self::UnsetTaskMetadata(anyq_queue_api::UnsetTaskMetadataRequest {
                    task: name,
                    key,
                })
            }
            (None, Some(key), Some(value)) => {
                Self::SetTaskMetadata(anyq_queue_api::SetTaskMetadataRequest {
                    task: name,
                    key,
                    value: (input.parse_value)(&value)?,
                })
            }
            (None, key, None) => {
                Self::GetTaskMetadata(anyq_queue_api::GetTaskMetadataRequest { task: name, key })
            }
            _ => {
                return Err(Error::Invalid(
                    "use meta [KEY [VALUE]] or meta --unset KEY".into(),
                ));
            }
        })
    }

    fn topic_metadata(
        args: super::metadata_args::MetadataArgs,
        input: &MetadataInput,
    ) -> Result<Self> {
        let name = args.name;
        if let Some(key) = args.unset.as_deref().or(args.key.as_deref()) {
            anyq_queue_api::MetadataPath::parse(key)?;
        }

        Ok(match (args.unset, args.key, args.value) {
            (Some(key), None, None) => {
                Self::UnsetTopicMetadata(anyq_queue_api::UnsetTopicMetadataRequest {
                    topic: name,
                    key,
                })
            }
            (None, Some(key), Some(value)) => {
                Self::SetTopicMetadata(anyq_queue_api::SetTopicMetadataRequest {
                    topic: name,
                    key,
                    value: (input.parse_value)(&value)?,
                })
            }
            (None, key, None) => {
                Self::GetTopicMetadata(anyq_queue_api::GetTopicMetadataRequest { topic: name, key })
            }
            _ => {
                return Err(Error::Invalid(
                    "use meta [KEY [VALUE]] or meta --unset KEY".into(),
                ));
            }
        })
    }
    pub fn parse(command: Commands, input: &MetadataInput) -> Result<Self> {
        match command {
            Commands::Plugin(args) => Ok(Self::Plugin(args.command)),
            Commands::Daemon(args) => Ok(Self::Daemon(args.command)),
            Commands::Config(args) => {
                let command = match (
                    args.list,
                    args.get,
                    args.unset,
                    args.words.as_slice(),
                    args.action,
                ) {
                    (true, false, false, [], None)
                    | (false, false, false, [], Some(ConfigAction::List)) => ConfigCommand::List,
                    (false, _, false, [key], None) => ConfigCommand::Get(ConfigKey::parse(key)?),
                    (false, false, true, [key], None) => {
                        ConfigCommand::Unset(ConfigKey::parse(key)?)
                    }
                    (false, false, false, [key, value], None) => {
                        let key = ConfigKey::parse(key)?;
                        ConfigCommand::Set(SetConfigRequest {
                            key,
                            value: ConfigValue::parse(key, value)?,
                        })
                    }
                    _ => {
                        return Err(Error::Invalid(
                            "use config KEY [VALUE], --get KEY, --unset KEY, or list/--list/-l"
                                .into(),
                        ));
                    }
                };

                Ok(Self::Config(command))
            }
            Commands::Topic(args) => match (args.list, args.name, args.command) {
                (false, None, Some(TopicCommand::Meta(args))) => Self::topic_metadata(args, input),
                (false, None, Some(TopicCommand::Metafile { name, file })) => Ok(
                    Self::ReplaceTopicMetadata(anyq_queue_api::ReplaceTopicMetadataRequest {
                        topic: name,
                        metadata: input.read(&file)?,
                    }),
                ),
                (true, None, None) | (false, None, Some(TopicCommand::List)) => {
                    Ok(Self::ListTopics)
                }
                (false, Some(name), None) => Ok(Self::GetTopicQueue { name }),
                (
                    false,
                    None,
                    Some(TopicCommand::Create {
                        name,
                        capacity,
                        metadata,
                    }),
                ) => {
                    let mut topic = Topic::new(name, capacity)?;
                    topic.metadata =
                        input.initial(metadata.metafile.as_deref(), &metadata.patches)?;

                    Ok(Self::CreateTopic(topic))
                }
                (
                    false,
                    None,
                    Some(TopicCommand::Capacity {
                        name,
                        capacity: Some(capacity),
                    }),
                ) => Ok(Self::SetTopicCapacity(Topic::new(name, capacity)?)),
                (
                    false,
                    None,
                    Some(TopicCommand::Capacity {
                        name,
                        capacity: None,
                    }),
                ) => Ok(Self::GetTopicCapacity { name }),
                (false, None, Some(TopicCommand::Autostart { name, value })) => {
                    Ok(Self::SetTopicAutostart {
                        name,
                        enabled: value == "on",
                    })
                }
                (
                    false,
                    None,
                    Some(TopicCommand::Link {
                        topic,
                        names,
                        wait,
                        complete,
                    }),
                ) => Ok(Self::LinkTasks {
                    names,
                    topic,
                    wait,
                    complete,
                }),
                _ => Err(Error::Invalid(
                    "use topic list, topic TOPIC, or one of topic create/capacity/link".into(),
                )),
            },
            Commands::Task(args) => match (args.list, args.name, args.command) {
                (false, None, Some(TaskCommand::Meta(args))) => Self::task_metadata(args, input),
                (false, None, Some(TaskCommand::Metafile { name, file })) => Ok(
                    Self::ReplaceTaskMetadata(anyq_queue_api::ReplaceTaskMetadataRequest {
                        task: name,
                        metadata: input.read(&file)?,
                    }),
                ),
                (true, None, None) | (false, None, Some(TaskCommand::List)) => Ok(Self::ListTasks),
                (false, Some(name), None) => Ok(Self::GetTask { name }),
                (
                    false,
                    None,
                    Some(TaskCommand::Create {
                        name,
                        metadata,
                        topics,
                        start,
                        stop,
                        switch_start,
                        wait,
                        complete,
                    }),
                ) => Ok(Self::RegisterTask {
                    request: RegisterTaskRequest {
                        metadata: input.initial(metadata.metafile.as_deref(), &metadata.patches)?,
                        name,
                        topics,
                        start: if start {
                            StartPreference::Start
                        } else if stop {
                            StartPreference::Stop
                        } else if switch_start {
                            StartPreference::Toggle
                        } else {
                            StartPreference::Default
                        },
                    },
                    wait,
                    complete,
                }),
                (
                    false,
                    None,
                    Some(TaskCommand::Link {
                        name,
                        topics,
                        wait,
                        complete,
                    }),
                ) => Ok(Self::LinkTask {
                    name,
                    topics,
                    wait,
                    complete,
                }),
                (
                    false,
                    None,
                    Some(TaskCommand::Start {
                        name,
                        wait,
                        complete,
                    }),
                ) => Ok(Self::StartTask {
                    name,
                    wait,
                    complete,
                }),
                (false, None, Some(TaskCommand::Wait { name, complete })) => {
                    Ok(Self::WaitTask { name, complete })
                }
                (false, None, Some(TaskCommand::Complete { name, force })) => {
                    Ok(Self::CompleteTask { name, force })
                }
                _ => Err(Error::Invalid(
                    "use task list, task NAME, or one of task create/link/wait/complete".into(),
                )),
            },
        }
    }
}
