use clap::Subcommand;
#[derive(Subcommand)]
pub enum Jobs {
    List {
        #[arg(long)]
        topic: Option<String>,
        #[arg(long)]
        group: Option<String>,
        #[arg(long, default_value_t = 0)]
        after: i64,
        #[arg(long, default_value_t = 100)]
        limit: u32,
    },
    Show {
        message_id: i64,
    },
}
