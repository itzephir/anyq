use clap::Subcommand;

#[derive(Subcommand)]
pub enum TaskCommand {
    Add {
        name: String,
    },
    Wait {
        name: String,
    },
    Complete {
        name: String,
        #[arg(long)]
        force: bool,
    },
}
