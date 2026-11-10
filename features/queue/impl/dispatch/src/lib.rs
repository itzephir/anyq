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

mod check_task_running;
pub use check_task_running::CheckTaskRunningInteractor;

mod complete_task;
pub use complete_task::CompleteTaskInteractor;

mod admit_queued_tasks;
mod lookup_task;
mod lookup_topic;

#[cfg(test)]
mod tests;

mod add_task_to_topics;
pub use add_task_to_topics::AddTaskToTopicsInteractor;

mod add_tasks_to_topic;
pub use add_tasks_to_topic::AddTasksToTopicInteractor;

mod link_tasks_to_topics;

mod resource_availability;

mod start_task;
pub use start_task::StartTaskInteractor;

mod set_topic_autostart;
pub use set_topic_autostart::SetTopicAutostartInteractor;
