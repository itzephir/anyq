use super::{Commands, ConfigCommand, TaskCommand, TopicCommand};
use crate::error::{Error, Result};
use anyq_config_api::{ConfigKey, ConfigValue, SetConfigRequest};
use anyq_queue_api::{RegisterTaskRequest, Topic};

/// Parsed CLI syntax, routed only in the outer composition root.
pub enum Request {
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
    RegisterTask {
        request: RegisterTaskRequest,
        wait: bool,
        complete: bool,
    },
    LinkTask {
        name: String,
        topic: String,
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

impl TryFrom<Commands> for Request {
    type Error = Error;

    fn try_from(command: Commands) -> Result<Self> {
        match command {
            Commands::Config(args) => {
                let command = match (args.list, args.get, args.unset, args.words.as_slice()) {
                    (true, false, false, []) => ConfigCommand::List,
                    (false, _, false, [key]) => ConfigCommand::Get(ConfigKey::parse(key)?),
                    (false, false, true, [key]) => ConfigCommand::Unset(ConfigKey::parse(key)?),
                    (false, false, false, [key, value]) => {
                        let key = ConfigKey::parse(key)?;
                        ConfigCommand::Set(SetConfigRequest {
                            key,
                            value: ConfigValue::parse(key, value)?,
                        })
                    }
                    _ => {
                        return Err(Error::Invalid(
                            "use config KEY [VALUE], --get KEY, --unset KEY, or --list".into(),
                        ));
                    }
                };

                Ok(Self::Config(command))
            }
            Commands::Topic(args) => match (args.list, args.name, args.command) {
                (true, None, None) | (false, None, Some(TopicCommand::List)) => {
                    Ok(Self::ListTopics)
                }
                (false, Some(name), None) => Ok(Self::GetTopicQueue { name }),
                (false, None, Some(TopicCommand::Create { name, capacity })) => {
                    Ok(Self::CreateTopic(Topic::new(name, capacity)?))
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
                (
                    false,
                    None,
                    Some(TopicCommand::Link {
                        topic,
                        name,
                        wait,
                        complete,
                    }),
                ) => Ok(Self::LinkTask {
                    name,
                    topic,
                    wait,
                    complete,
                }),
                _ => Err(Error::Invalid(
                    "use topic list, topic TOPIC, or one of topic create/capacity/link".into(),
                )),
            },
            Commands::Task(args) => match (args.list, args.name, args.command) {
                (true, None, None) | (false, None, Some(TaskCommand::List)) => Ok(Self::ListTasks),
                (false, Some(name), None) => Ok(Self::GetTask { name }),
                (
                    false,
                    None,
                    Some(TaskCommand::Create {
                        name,
                        topics,
                        wait,
                        complete,
                    }),
                ) => Ok(Self::RegisterTask {
                    request: RegisterTaskRequest { name, topics },
                    wait,
                    complete,
                }),
                (
                    false,
                    None,
                    Some(TaskCommand::Link {
                        name,
                        topic,
                        wait,
                        complete,
                    }),
                ) => Ok(Self::LinkTask {
                    name,
                    topic,
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
