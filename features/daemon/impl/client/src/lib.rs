//! Focused remote adapters for queue contracts.
mod channel;
mod client_session;
mod start_owner;

pub use channel::Channel;

mod remote_observe_queue;
pub use remote_observe_queue::RemoteObserveQueue;

mod lifecycle;
pub use lifecycle::{status, stop};

mod remote_create_topic;
pub use remote_create_topic::RemoteCreateTopic;

mod remote_set_topic_capacity;
pub use remote_set_topic_capacity::RemoteSetTopicCapacity;

mod remote_list_topics;
pub use remote_list_topics::RemoteListTopics;

mod remote_list_tasks;
pub use remote_list_tasks::RemoteListTasks;

mod remote_get_topic;
pub use remote_get_topic::RemoteGetTopic;

mod remote_get_task;
pub use remote_get_task::RemoteGetTask;

mod remote_get_topic_queue;
pub use remote_get_topic_queue::RemoteGetTopicQueue;

mod remote_register_task;
pub use remote_register_task::RemoteRegisterTask;

mod remote_add_task_to_topics;
pub use remote_add_task_to_topics::RemoteAddTaskToTopics;

mod remote_add_tasks_to_topic;
pub use remote_add_tasks_to_topic::RemoteAddTasksToTopic;

mod remote_add_task_to_topic;
pub use remote_add_task_to_topic::RemoteAddTaskToTopic;

mod remote_check_task_running;
pub use remote_check_task_running::RemoteCheckTaskRunning;

mod remote_set_topic_autostart;
pub use remote_set_topic_autostart::RemoteSetTopicAutostart;

mod remote_complete_task;
pub use remote_complete_task::RemoteCompleteTask;

mod remote_start_task;
pub use remote_start_task::RemoteStartTask;

mod remote_get_task_metadata;
pub use remote_get_task_metadata::RemoteGetTaskMetadata;

mod remote_set_task_metadata;
pub use remote_set_task_metadata::RemoteSetTaskMetadata;

mod remote_unset_task_metadata;
pub use remote_unset_task_metadata::RemoteUnsetTaskMetadata;

mod remote_replace_task_metadata;
pub use remote_replace_task_metadata::RemoteReplaceTaskMetadata;

mod remote_get_topic_metadata;
pub use remote_get_topic_metadata::RemoteGetTopicMetadata;

mod remote_set_topic_metadata;
pub use remote_set_topic_metadata::RemoteSetTopicMetadata;

mod remote_unset_topic_metadata;
pub use remote_unset_topic_metadata::RemoteUnsetTopicMetadata;

mod remote_replace_topic_metadata;
pub use remote_replace_topic_metadata::RemoteReplaceTopicMetadata;

mod remote_enable_plugin;
pub use remote_enable_plugin::RemoteEnablePlugin;

mod remote_list_plugins;
pub use remote_list_plugins::RemoteListPlugins;

mod remote_get_plugin_status;
pub use remote_get_plugin_status::RemoteGetPluginStatus;

mod remote_disable_plugin;
pub use remote_disable_plugin::RemoteDisablePlugin;
