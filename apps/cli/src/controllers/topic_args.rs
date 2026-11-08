use super::topic_command::TopicCommand;
use clap::Args;

#[derive(Args)]
#[command(
    after_help = "Examples:\n  anyq topic --list\n  anyq topic create cpu -c 2\n  anyq topic capacity cpu 3\n  anyq topic link cpu my-task"
)]
pub struct TopicArgs {
    #[arg(short = 'l', long, help = "List topic names")]
    pub list: bool,
    #[arg(
        value_name = "TOPIC",
        help = "Show queued and running tasks in a topic"
    )]
    pub name: Option<String>,
    #[command(subcommand)]
    pub command: Option<TopicCommand>,
}
