use super::config_args::ConfigArgs;
use super::{task_args::TaskArgs, topic_args::TopicArgs};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Run a first-party plugin over the public stdio protocol")]
    PluginRuntime(super::plugin_runtime::PluginRuntime),
    #[command(about = "Enable, inspect or disable trusted local plugins")]
    Plugin(super::plugin_args::PluginArgs),
    #[command(about = "Start, inspect or stop the background queue owner")]
    Daemon(super::daemon_args::DaemonArgs),
    #[command(about = "Read and update ~/.anyq/config.toml")]
    Config(ConfigArgs),
    #[command(about = "List topics, configure capacity or link a named task")]
    Topic(TopicArgs),
    #[command(about = "List, register, link, wait for or complete tasks")]
    Task(TaskArgs),
}
