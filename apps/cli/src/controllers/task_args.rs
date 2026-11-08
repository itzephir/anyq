use super::task_command::TaskCommand;
use clap::Args;

#[derive(Args)]
pub struct TaskArgs {
    #[arg(
        short = 'l',
        long,
        help = "List all registered task names, including completed tasks"
    )]
    pub list: bool,
    #[arg(value_name = "NAME", help = "Show a task's status and linked topics")]
    pub name: Option<String>,
    #[command(subcommand)]
    pub command: Option<TaskCommand>,
}
