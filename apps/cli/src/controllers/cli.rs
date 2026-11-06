use super::Commands;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "anyq",
    version,
    about = "Durable local topic queues with independent consumer groups"
)]
pub struct Cli {
    #[arg(
        long,
        env = "ANYQ_DB",
        global = true,
        help = "Local SQLite file (default: LOCALAPPDATA/anyq on Windows; XDG_DATA_HOME/anyq or ~/.local/share/anyq on Unix)"
    )]
    pub db: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Commands,
}
