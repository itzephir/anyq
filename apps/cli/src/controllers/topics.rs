use clap::Subcommand;

#[derive(Subcommand)]
pub enum Topics {
    Create {
        name: String,
        #[arg(long, default_value_t = 1)]
        capacity: u32,
    },
    Capacity {
        name: String,
        #[arg(long)]
        capacity: u32,
    },
    List,
}
