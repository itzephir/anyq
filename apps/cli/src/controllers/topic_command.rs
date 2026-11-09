use clap::Subcommand;

#[derive(Subcommand)]
pub enum TopicCommand {
    #[command(about = "List topic names")]
    List,
    #[command(about = "Create a resource topic")]
    Create {
        #[arg(value_name = "TOPIC")]
        name: String,
        #[arg(short = 'c', long, default_value_t = anyq_queue_api::Topic::DEFAULT_CAPACITY)]
        capacity: u32,
    },
    #[command(about = "Show capacity, or set it when N is provided")]
    Capacity {
        #[arg(value_name = "TOPIC")]
        name: String,
        #[arg(value_name = "N")]
        capacity: Option<u32>,
    },
    #[command(about = "Set topic autostart (overrides task.autostart)")]
    Autostart {
        #[arg(value_name = "TOPIC")]
        name: String,
        #[arg(value_name = "on/off", value_parser = ["on", "off"], default_value = "on")]
        value: String,
    },
    #[command(about = "Atomically link one or more tasks to a topic")]
    Link {
        topic: String,
        #[arg(required = true, num_args = 1.., value_name = "NAME")]
        names: Vec<String>,
        #[arg(short = 'w', long)]
        wait: bool,
        #[arg(short = 'c', long, requires = "wait")]
        complete: bool,
    },
}
