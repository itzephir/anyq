mod queue_module;
mod system_clock;
mod uuid_tokens;
use anyq_queue_api::Result;
use anyq_queue_dispatch::*;
use anyq_queue_sqlite::Sqlite;
pub use queue_module::QueueModule;
use std::path::Path;
use system_clock::SystemClock;
use uuid_tokens::UuidTokens;
pub fn open(path: &Path) -> Result<QueueModule> {
    let store = Sqlite::open(path)?;
    Ok(QueueModule {
        create_topic: Box::new(CreateTopicInteractor::new(store.clone())),
        set_topic_capacity: Box::new(SetTopicCapacityInteractor::new(store.clone())),
        list_topics: Box::new(ListTopicsInteractor::new(store.clone())),
        get_topic: Box::new(GetTopicInteractor::new(store.clone())),
        create_consumer_group: Box::new(CreateConsumerGroupInteractor::new(store.clone())),
        list_consumer_groups: Box::new(ListConsumerGroupsInteractor::new(store.clone())),
        publish_message: Box::new(PublishMessageInteractor::new(store.clone(), SystemClock)),
        claim_delivery: Box::new(ClaimDeliveryInteractor::new(
            store.clone(),
            SystemClock,
            UuidTokens,
        )),
        renew_lease: Box::new(RenewLeaseInteractor::new(store.clone(), SystemClock)),
        complete_delivery: Box::new(CompleteDeliveryInteractor::new(store.clone(), SystemClock)),
        reject_delivery: Box::new(RejectDeliveryInteractor::new(store.clone(), SystemClock)),
        retry_delivery: Box::new(RetryDeliveryInteractor::new(store.clone(), SystemClock)),
        inspect_message: Box::new(InspectMessageInteractor::new(store.clone(), SystemClock)),
        list_messages: Box::new(ListMessagesInteractor::new(store.clone(), SystemClock)),
    })
}
