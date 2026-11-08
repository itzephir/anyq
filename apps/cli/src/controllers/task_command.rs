use clap::Subcommand;

#[derive(Subcommand)]
pub enum TaskCommand {
    #[command(about = "Register a task, optionally linked to existing topics")]
    Add {
        name: String,
        #[arg(short = 't', long = "topic", value_name = "TOPIC")]
        topics: Vec<String>,
        #[arg(short = 'w', long, requires = "topics")]
        wait: bool,
        #[arg(short = 'c', long, requires = "wait")]
        complete: bool,
    },
    #[command(about = "Link an existing task to a topic")]
    Link { name: String, topic: String },
    #[command(about = "Wait for admission to every linked topic")]
    Wait { name: String },
    #[command(about = "Complete a task and release its resource slots")]
    Complete {
        name: String,
        #[arg(short = 'f', long)]
        force: bool,
    },
}
