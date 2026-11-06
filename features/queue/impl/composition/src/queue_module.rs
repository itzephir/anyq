//! Explicitly constructed use-case handles; contains no dispatch or lookup behavior.
use anyq_queue_api::*;

pub struct QueueModule {
    pub create_topic: Box<dyn CreateTopic>,
    pub set_topic_capacity: Box<dyn SetTopicCapacity>,
    pub list_topics: Box<dyn ListTopics>,
    pub get_topic: Box<dyn GetTopic>,
    pub create_consumer_group: Box<dyn CreateConsumerGroup>,
    pub list_consumer_groups: Box<dyn ListConsumerGroups>,
    pub publish_message: Box<dyn PublishMessage>,
    pub claim_delivery: Box<dyn ClaimDelivery>,
    pub renew_lease: Box<dyn RenewLease>,
    pub complete_delivery: Box<dyn CompleteDelivery>,
    pub reject_delivery: Box<dyn RejectDelivery>,
    pub retry_delivery: Box<dyn RetryDelivery>,
    pub inspect_message: Box<dyn InspectMessage>,
    pub list_messages: Box<dyn ListMessages>,
}
