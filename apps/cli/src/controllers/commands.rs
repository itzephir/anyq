use super::{ClaimArgs, Data, Groups, Jobs, LeaseArgs, Next, Topics, WorkerArgs};
use clap::Subcommand;
#[derive(Subcommand)]
pub enum Commands {
    #[command(subcommand)]
    Topic(Topics),
    #[command(subcommand)]
    Group(Groups),
    Publish {
        topic: String,
        #[command(flatten)]
        data: Data,
        #[arg(long)]
        key: Option<String>,
        #[arg(long, default_value_t = 3)]
        max_attempts: u32,
    },
    Claim(ClaimArgs),
    Ack {
        #[command(flatten)]
        lease: LeaseArgs,
        #[arg(long)]
        result: Option<String>,
        #[command(flatten)]
        next: Next,
    },
    Nack {
        #[command(flatten)]
        lease: LeaseArgs,
        #[arg(long)]
        reason: String,
        #[arg(long, default_value_t = 1000)]
        delay_ms: i64,
    },
    Renew {
        #[command(flatten)]
        lease: LeaseArgs,
        #[arg(long, default_value_t = 60000)]
        lease_ms: i64,
    },
    Retry {
        message_id: i64,
        #[arg(long)]
        group: String,
    },
    #[command(subcommand)]
    Jobs(Jobs),
    #[command(
        about = "Run an explicit command; JSON arrives on stdin, command output goes to stderr"
    )]
    Worker(WorkerArgs),
}
