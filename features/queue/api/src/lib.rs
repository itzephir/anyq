//! Named tasks, resource admission and focused use-case contracts. No IO.
mod error;
pub use error::{Error, Result};

mod metadata;
pub use metadata::Metadata;

mod metadata_budget;
pub use metadata_budget::MetadataBudget;

mod metadata_number;
pub use metadata_number::MetadataNumber;

mod metadata_path;
pub use metadata_path::MetadataPath;

mod metadata_value;
pub use metadata_value::MetadataValue;

#[cfg(test)]
mod metadata_tests;

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

mod get_task_metadata;
pub use get_task_metadata::GetTaskMetadata;

mod get_task_metadata_request;
pub use get_task_metadata_request::GetTaskMetadataRequest;

mod set_task_metadata;
pub use set_task_metadata::SetTaskMetadata;

mod set_task_metadata_request;
pub use set_task_metadata_request::SetTaskMetadataRequest;

mod unset_task_metadata;
pub use unset_task_metadata::UnsetTaskMetadata;

mod unset_task_metadata_request;
pub use unset_task_metadata_request::UnsetTaskMetadataRequest;

mod replace_task_metadata;
pub use replace_task_metadata::ReplaceTaskMetadata;

mod replace_task_metadata_request;
pub use replace_task_metadata_request::ReplaceTaskMetadataRequest;

mod get_topic_metadata;
pub use get_topic_metadata::GetTopicMetadata;

mod get_topic_metadata_request;
pub use get_topic_metadata_request::GetTopicMetadataRequest;

mod set_topic_metadata;
pub use set_topic_metadata::SetTopicMetadata;

mod set_topic_metadata_request;
pub use set_topic_metadata_request::SetTopicMetadataRequest;

mod unset_topic_metadata;
pub use unset_topic_metadata::UnsetTopicMetadata;

mod unset_topic_metadata_request;
pub use unset_topic_metadata_request::UnsetTopicMetadataRequest;

mod replace_topic_metadata;
pub use replace_topic_metadata::ReplaceTopicMetadata;

mod replace_topic_metadata_request;
pub use replace_topic_metadata_request::ReplaceTopicMetadataRequest;

mod execution_directory;
pub use execution_directory::ExecutionDirectory;

mod queue_event_kind;
pub use queue_event_kind::QueueEventKind;

mod event_snapshot;
pub use event_snapshot::EventSnapshot;

mod queue_event;
pub use queue_event::QueueEvent;

mod read_queue_events;
pub use read_queue_events::ReadQueueEvents;

mod managed_work_state;
pub use managed_work_state::ManagedWorkState;

mod managed_work;
pub use managed_work::{ManagedWork, validate_action_identity};

mod managed_launch;
pub use managed_launch::ManagedLaunch;

mod completion_receipt;
pub use completion_receipt::CompletionReceipt;

mod complete_task_once_request;
pub use complete_task_once_request::CompleteTaskOnceRequest;

mod complete_task_once;
pub use complete_task_once::CompleteTaskOnce;

mod begin_completion_request;
pub use begin_completion_request::BeginCompletionRequest;

mod begin_completion;
pub use begin_completion::BeginCompletion;

mod register_managed_work_request;
pub use register_managed_work_request::RegisterManagedWorkRequest;

mod register_managed_work;
pub use register_managed_work::RegisterManagedWork;

mod report_managed_work_request;
pub use report_managed_work_request::ReportManagedWorkRequest;

mod report_managed_work;
pub use report_managed_work::ReportManagedWork;

mod list_managed_work_request;
pub use list_managed_work_request::ListManagedWorkRequest;

mod list_managed_work;
pub use list_managed_work::ListManagedWork;

mod resolve_managed_work_request;
pub use resolve_managed_work_request::ResolveManagedWorkRequest;

mod resolve_managed_work;
pub use resolve_managed_work::ResolveManagedWork;

mod work_services;
pub use work_services::WorkServices;

mod cancel_managed_work;
pub use cancel_managed_work::CancelManagedWork;
