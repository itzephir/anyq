//! Named tasks, resource admission and focused use-case contracts. No IO.
mod error;
pub use error::{Error, Result};

mod validate_name;
pub use validate_name::validate_name;

mod validate_topic_name;
pub use validate_topic_name::validate_topic_name;

mod topic;
pub use topic::Topic;

mod task_state;
pub use task_state::TaskState;

mod start_outcome;
pub use start_outcome::StartOutcome;

mod resource_availability;
pub use resource_availability::ResourceAvailability;

mod task;
pub use task::Task;

mod create_topic;
pub use create_topic::CreateTopic;

mod set_topic_capacity;
pub use set_topic_capacity::SetTopicCapacity;

mod list_topics;
pub use list_topics::ListTopics;

mod list_tasks;
pub use list_tasks::ListTasks;

mod get_topic;
pub use get_topic::GetTopic;

mod get_task;
pub use get_task::GetTask;

mod get_topic_queue;
pub use get_topic_queue::GetTopicQueue;

mod topic_queue;
pub use topic_queue::TopicQueue;

mod register_task;
pub use register_task::RegisterTask;

mod register_task_request;
pub use register_task_request::RegisterTaskRequest;

mod add_task_to_topic;
pub use add_task_to_topic::AddTaskToTopic;

mod try_start_task;
pub use try_start_task::TryStartTask;

mod complete_task;
pub use complete_task::CompleteTask;

#[cfg(test)]
mod tests;
