use super::Commands;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "anyq",
    version,
    about = "Coordinate named tasks across local resource queues"
)]
pub struct Cli {
    #[arg(
        long,
        env = "ANYQ_DB",
        global = true,
        help = "Shared local SQLite database (schema 2)"
    )]
    pub db: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Commands,
}
