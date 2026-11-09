use super::config_args::ConfigArgs;
use super::{task_args::TaskArgs, topic_args::TopicArgs};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Read and update ~/.anyq/config.toml")]
    Config(ConfigArgs),
    #[command(about = "List topics, configure capacity or link a named task")]
    Topic(TopicArgs),
    #[command(about = "List, register, link, wait for or complete tasks")]
    Task(TaskArgs),
}
