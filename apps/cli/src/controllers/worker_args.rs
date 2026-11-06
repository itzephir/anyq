use super::{ClaimArgs, Next};
use anyq_queue_api::Result;
use anyq_worker_api::Settings;
use clap::Args;

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
