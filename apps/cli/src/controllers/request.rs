use super::{Commands, TaskCommand, TopicCommand};
use anyq_queue_api::{Error, RegisterTaskRequest, Topic};

/// Parsed CLI syntax, routed only in the outer composition root.
pub enum Request {
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
    },
    WaitTask {
        name: String,
    },
    CompleteTask {
        name: String,
        force: bool,
    },
}

impl TryFrom<Commands> for Request {
    type Error = Error;

    fn try_from(command: Commands) -> Result<Self, Self::Error> {
        match command {
            Commands::Topic(args) => match (args.list, args.name, args.command) {
                (true, None, None) => Ok(Self::ListTopics),
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
                (false, None, Some(TopicCommand::Link { topic, name })) => {
                    Ok(Self::LinkTask { name, topic })
                }
                _ => Err(Error::Invalid(
                    "use topic --list, topic TOPIC, or one of topic create/capacity/link".into(),
                )),
            },
            Commands::Task(args) => match (args.list, args.name, args.command) {
                (true, None, None) => Ok(Self::ListTasks),
                (false, Some(name), None) => Ok(Self::GetTask { name }),
                (
                    false,
                    None,
                    Some(TaskCommand::Add {
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
                (false, None, Some(TaskCommand::Link { name, topic })) => {
                    Ok(Self::LinkTask { name, topic })
                }
                (false, None, Some(TaskCommand::Wait { name })) => Ok(Self::WaitTask { name }),
                (false, None, Some(TaskCommand::Complete { name, force })) => {
                    Ok(Self::CompleteTask { name, force })
                }
                _ => Err(Error::Invalid(
                    "use task --list, task NAME, or one of task add/link/wait/complete".into(),
                )),
            },
        }
    }
}
