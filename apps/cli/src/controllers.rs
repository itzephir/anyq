use anyq_queue_api::*;
use anyq_worker_api::Settings;
use clap::{Args, Parser, Subcommand};
use std::{io::Read, path::PathBuf};

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
        help = "Local SQLite file (default: $XDG_DATA_HOME/anyq/queue.db or ~/.local/share/anyq/queue.db)"
    )]
    pub db: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Commands,
}
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
#[derive(Subcommand)]
pub enum Groups {
    Create { topic: String, name: String },
    List { topic: String },
}
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
#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct Data {
    #[arg(long, help = "JSON payload")]
    pub payload: Option<String>,
    #[arg(long, help = "Read JSON from a file; '-' reads stdin")]
    pub file: Option<PathBuf>,
}
#[derive(Args)]
pub struct ClaimArgs {
    pub topic: String,
    #[arg(long)]
    pub group: String,
    #[arg(long)]
    pub consumer: String,
    #[arg(long, default_value_t = 60000)]
    pub lease_ms: i64,
}
impl ClaimArgs {
    pub fn domain(self) -> Claim {
        Claim {
            topic: self.topic,
            group: self.group,
            consumer: self.consumer,
            lease_ms: self.lease_ms,
        }
    }
}
#[derive(Args)]
pub struct LeaseArgs {
    pub message_id: i64,
    #[arg(long)]
    pub group: String,
    #[arg(long)]
    pub token: String,
}
impl LeaseArgs {
    fn domain(self) -> Lease {
        Lease {
            message_id: self.message_id,
            group: self.group,
            token: self.token,
        }
    }
}
#[derive(Args)]
pub struct Next {
    #[arg(long, requires = "next_payload")]
    pub next_topic: Option<String>,
    #[arg(long, requires = "next_topic")]
    pub next_payload: Option<String>,
    #[arg(long, requires = "next_topic")]
    pub next_key: Option<String>,
}
impl Next {
    fn domain(self) -> Result<Option<Publish>> {
        match (self.next_topic, self.next_payload) {
            (Some(topic), Some(payload)) => Ok(Some(Publish {
                topic,
                payload: json(&payload)?,
                key: self.next_key,
                max_attempts: 3,
            })),
            (None, None) => Ok(None),
            _ => Err(Error::Invalid(
                "next-topic and next-payload must be specified together".into(),
            )),
        }
    }
}
#[derive(Args)]
pub struct WorkerArgs {
    #[command(flatten)]
    pub claim: ClaimArgs,
    #[arg(long, default_value_t = 1000)]
    pub retry_delay_ms: i64,
    #[arg(long, default_value_t = 1000)]
    pub poll_ms: u64,
    #[arg(long)]
    pub once: bool,
    #[command(flatten)]
    pub next: Next,
    #[arg(last=true, required=true, num_args=1..)]
    pub command: Vec<String>,
}
impl WorkerArgs {
    pub fn domain(self) -> Result<(Settings, Vec<String>, bool)> {
        Ok((
            Settings {
                claim: self.claim.domain(),
                retry_delay_ms: self.retry_delay_ms,
                idle_ms: self.poll_ms,
                next: self.next.domain()?,
            },
            self.command,
            self.once,
        ))
    }
}
fn invalid(error: impl std::fmt::Display) -> Error {
    Error::Invalid(error.to_string())
}
pub fn json(raw: &str) -> Result<String> {
    serde_json::from_str::<serde_json::Value>(raw)
        .map(|v| v.to_string())
        .map_err(invalid)
}
fn payload(data: Data) -> Result<String> {
    let raw = if let Some(s) = data.payload {
        s
    } else {
        let file = data
            .file
            .ok_or_else(|| Error::Invalid("payload or file required".into()))?;
        let mut source: Box<dyn Read> = if file.as_os_str() == "-" {
            Box::new(std::io::stdin())
        } else {
            Box::new(std::fs::File::open(file).map_err(invalid)?)
        };
        let mut raw = String::new();
        source
            .by_ref()
            .take(1_048_577)
            .read_to_string(&mut raw)
            .map_err(invalid)?;
        raw
    };
    if raw.len() > 1_048_576 {
        return Err(Error::Invalid(
            "payload exceeds 1 MiB; put large artifacts in files and publish paths".into(),
        ));
    }
    json(&raw)
}
pub fn command(command: Commands) -> Result<Command> {
    Ok(match command {
        Commands::Topic(Topics::Create { name, capacity }) => {
            Command::CreateTopic(Topic::new(name, capacity)?)
        }
        Commands::Topic(Topics::Capacity { name, capacity }) => {
            Command::SetCapacity(Topic::new(name, capacity)?)
        }
        Commands::Topic(Topics::List) => Command::Topics,
        Commands::Group(Groups::Create { topic, name }) => {
            Command::CreateGroup(ConsumerGroup { topic, name })
        }
        Commands::Group(Groups::List { topic }) => Command::Groups { topic },
        Commands::Publish {
            topic,
            data,
            key,
            max_attempts,
        } => Command::Publish(Publish {
            topic,
            payload: payload(data)?,
            key,
            max_attempts,
        }),
        Commands::Claim(args) => Command::Claim(args.domain()),
        Commands::Ack {
            lease,
            result,
            next,
        } => Command::Ack {
            lease: lease.domain(),
            result: result.map(|v| json(&v)).transpose()?,
            next: next.domain()?,
        },
        Commands::Nack {
            lease,
            reason,
            delay_ms,
        } => Command::Nack {
            lease: lease.domain(),
            reason,
            delay_ms,
        },
        Commands::Renew { lease, lease_ms } => Command::Renew {
            lease: lease.domain(),
            lease_ms,
        },
        Commands::Retry { message_id, group } => Command::Retry { message_id, group },
        Commands::Jobs(Jobs::List {
            topic,
            group,
            after,
            limit,
        }) => Command::List {
            topic,
            group,
            after,
            limit,
        },
        Commands::Jobs(Jobs::Show { message_id }) => Command::Inspect { message_id },
        Commands::Worker(_) => {
            return Err(Error::Invalid(
                "worker must be assembled by the CLI composition root".into(),
            ));
        }
    })
}
