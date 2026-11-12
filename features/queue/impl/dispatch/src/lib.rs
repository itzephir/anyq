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

mod get_task_metadata;
pub use get_task_metadata::GetTaskMetadataInteractor;

mod set_task_metadata;
pub use set_task_metadata::SetTaskMetadataInteractor;

mod unset_task_metadata;
pub use unset_task_metadata::UnsetTaskMetadataInteractor;

mod replace_task_metadata;
pub use replace_task_metadata::ReplaceTaskMetadataInteractor;

mod get_topic_metadata;
pub use get_topic_metadata::GetTopicMetadataInteractor;

mod set_topic_metadata;
pub use set_topic_metadata::SetTopicMetadataInteractor;

mod unset_topic_metadata;
pub use unset_topic_metadata::UnsetTopicMetadataInteractor;

mod replace_topic_metadata;
pub use replace_topic_metadata::ReplaceTopicMetadataInteractor;

mod record_event;

mod complete_task_once;
pub use complete_task_once::CompleteTaskOnceInteractor;

mod begin_completion;
pub use begin_completion::BeginCompletionInteractor;

mod register_managed_work;
pub use register_managed_work::RegisterManagedWorkInteractor;

mod report_managed_work;
pub use report_managed_work::ReportManagedWorkInteractor;

mod list_managed_work;
pub use list_managed_work::ListManagedWorkInteractor;

mod resolve_managed_work;
pub use resolve_managed_work::ResolveManagedWorkInteractor;

mod cancel_managed_work;
pub use cancel_managed_work::CancelManagedWorkInteractor;

mod observe_queue;
pub use observe_queue::ObserveQueueInteractor;
