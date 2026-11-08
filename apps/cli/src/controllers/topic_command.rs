use clap::Subcommand;

#[derive(Subcommand)]
pub enum TopicCommand {
    #[command(about = "Create a resource topic")]
    Create {
        #[arg(value_name = "TOPIC")]
        name: String,
        #[arg(short = 'c', long, default_value_t = 1)]
        capacity: u32,
    },
    #[command(about = "Show capacity, or set it when N is provided")]
    Capacity {
        #[arg(value_name = "TOPIC")]
        name: String,
        #[arg(value_name = "N")]
        capacity: Option<u32>,
    },
    #[command(about = "Link an existing task to a topic")]
    Link { topic: String, name: String },
}
