use super::json::json;
use anyq_queue_api::{Error, Publish, Result};
use clap::Args;
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
    pub fn domain(self) -> Result<Option<Publish>> {
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
