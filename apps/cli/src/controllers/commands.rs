use super::{task_command::TaskCommand, topic_args::TopicArgs};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "List topics, configure capacity or add a named task to a topic")]
    Topic(TopicArgs),
    #[command(subcommand)]
    Task(TaskCommand),
}
