use anyq_queue_api::Claim;
use clap::Args;

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
