//! Named-task scenarios. Domain policy determines eligibility; storage performs exact queries.
pub mod ports;

mod create_topic;
pub use create_topic::CreateTopicInteractor;

mod set_topic_capacity;
pub use set_topic_capacity::SetTopicCapacityInteractor;

mod list_topics;
pub use list_topics::ListTopicsInteractor;

mod list_tasks;
pub use list_tasks::ListTasksInteractor;

mod get_topic;
pub use get_topic::GetTopicInteractor;

mod get_task;
pub use get_task::GetTaskInteractor;

mod get_topic_queue;
pub use get_topic_queue::GetTopicQueueInteractor;

mod register_task;
pub use register_task::RegisterTaskInteractor;

mod add_task_to_topic;
pub use add_task_to_topic::AddTaskToTopicInteractor;

mod try_start_task;
pub use try_start_task::TryStartTaskInteractor;

mod complete_task;
pub use complete_task::CompleteTaskInteractor;

mod lookup_task;
mod lookup_topic;

#[cfg(test)]
mod tests;
