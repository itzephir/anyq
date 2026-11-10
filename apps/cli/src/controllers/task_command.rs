use clap::Subcommand;

#[derive(Subcommand)]
pub enum TaskCommand {
    #[command(about = "List all task names")]
    List,
    #[command(about = "Register a task, optionally linked to existing topics")]
    Create {
        name: String,
        #[arg(short = 't', long = "topic", value_name = "TOPIC")]
        topics: Vec<String>,
        #[arg(long, conflicts_with_all = ["stop", "switch_start"])]
        start: bool,
        #[arg(long, conflicts_with_all = ["start", "switch_start"])]
        stop: bool,
        #[arg(short = 's', conflicts_with_all = ["start", "stop"], help = "Invert effective autostart for this task")]
        switch_start: bool,
        #[arg(short = 'w', long, requires = "topics")]
        wait: bool,
        #[arg(short = 'c', long, requires = "wait")]
        complete: bool,
    },
    #[command(about = "Atomically link a task to one or more topics")]
    Link {
        name: String,
        #[arg(required = true, num_args = 1.., value_name = "TOPIC")]
        topics: Vec<String>,
        #[arg(short = 'w', long)]
        wait: bool,
        #[arg(short = 'c', long, requires = "wait")]
        complete: bool,
    },
    #[command(about = "Start a task now, or wait for resources with --wait")]
    Start {
        name: String,
        #[arg(short = 'w', long)]
        wait: bool,
        #[arg(short = 'c', long, requires = "wait")]
        complete: bool,
    },
    #[command(about = "Wait until the task is running (immediate success if already running)")]
    Wait {
        name: String,
        #[arg(short = 'c', long)]
        complete: bool,
    },
    #[command(about = "Complete a task and release its resource slots")]
    Complete {
        name: String,
        #[arg(short = 'f', long)]
        force: bool,
    },
}
