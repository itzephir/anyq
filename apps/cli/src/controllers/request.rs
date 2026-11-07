use super::{Commands, TaskCommand, TopicCommand};
use anyq_queue_api::Error;

/// Parsed CLI syntax, routed only in the outer composition root.
pub enum Request {
    Topic(TopicCommand),
    Task(TaskCommand),
}

impl TryFrom<Commands> for Request {
    type Error = Error;

    fn try_from(command: Commands) -> Result<Self, Self::Error> {
        match command {
            Commands::Topic(args) => Ok(Self::Topic(args.command()?)),
            Commands::Task(command) => Ok(Self::Task(command)),
        }
    }
}
