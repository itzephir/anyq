use super::Commands;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "anyq",
    version,
    disable_version_flag = true,
    about = "Coordinate named tasks across local resource queues"
)]
pub struct Cli {
    #[arg(
        long,
        env = "ANYQ_LANG",
        global = true,
        value_enum,
        default_value = "en",
        help = "Presentation language (en or ru)"
    )]
    pub lang: crate::language::Language,
    #[arg(short = 'v', long, action = clap::ArgAction::Version)]
    version: Option<bool>,
    #[arg(
        long,
        env = "ANYQ_DB",
        global = true,
        help = "Shared local SQLite database"
    )]
    pub db: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Commands,
}
