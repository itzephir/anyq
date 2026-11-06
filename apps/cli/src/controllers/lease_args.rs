use anyq_queue_api::Lease;
use clap::Args;

#[derive(Args)]
pub struct LeaseArgs {
    pub message_id: i64,
    #[arg(long)]
    pub group: String,
    #[arg(long)]
    pub token: String,
}

impl LeaseArgs {
    pub fn domain(self) -> Lease {
        Lease {
            message_id: self.message_id,
            group: self.group,
            token: self.token,
        }
    }
}
