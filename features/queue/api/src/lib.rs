//! Named tasks, resource admission and focused use-case contracts. No IO.
mod error;
pub use error::{Error, Result};

mod auto_create_policy;
pub use auto_create_policy::AutoCreatePolicy;

mod parse_name;
pub use parse_name::{parse_name, parse_name as parse_topic_name};

mod format_name;
pub use format_name::format_name;

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

mod check_task_running;
pub use check_task_running::CheckTaskRunning;

mod complete_task;
pub use complete_task::CompleteTask;

#[cfg(test)]
mod tests;

mod add_task_to_topics;
pub use add_task_to_topics::AddTaskToTopics;

mod add_task_to_topics_request;
pub use add_task_to_topics_request::AddTaskToTopicsRequest;

mod add_tasks_to_topic;
pub use add_tasks_to_topic::AddTasksToTopic;

mod add_tasks_to_topic_request;
pub use add_tasks_to_topic_request::AddTasksToTopicRequest;

mod queue_policy;
pub use queue_policy::QueuePolicy;

mod start_preference;
pub use start_preference::StartPreference;

mod start_task_outcome;
pub use start_task_outcome::StartTaskOutcome;

mod start_task;
pub use start_task::StartTask;

mod set_topic_autostart;
pub use set_topic_autostart::SetTopicAutostart;
mod queue_services;
pub use queue_services::QueueServices;

mod observe_queue;
pub use observe_queue::ObserveQueue;

mod observe_queue_request;
pub use observe_queue_request::ObserveQueueRequest;

mod queue_snapshot;
pub use queue_snapshot::QueueSnapshot;

mod observed_task;
pub use observed_task::ObservedTask;

mod observed_topic;
pub use observed_topic::ObservedTopic;
