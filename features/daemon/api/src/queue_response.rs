use crate::{MetadataDto, QueueError, TaskDto, TopicDto, TopicQueueDto};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(
    tag = "result",
    content = "value",
    rename_all = "kebab-case",
    deny_unknown_fields
)]
pub enum QueueResponse {
    PluginStatus(anyq_plugin_wire::PluginStatusDto),
    Plugins(Vec<anyq_plugin_wire::PluginStatusDto>),
    ManagedWork(anyq_plugin_wire::ManagedWorkDto),
    ManagedList(Vec<anyq_plugin_wire::ManagedWorkDto>),
    Ready {
        pid: u32,
        database_id: String,
    },
    Observation {
        database_id: String,
        snapshot: crate::QueueSnapshotDto,
    },
    Stopped,
    Metadata(MetadataDto),
    Task(TaskDto),
    Tasks(Vec<TaskDto>),
    Topic(TopicDto),
    Topics(Vec<TopicDto>),
    Names(Vec<String>),
    TopicQueue(TopicQueueDto),
    Running(bool),
    Started(String),
    Error(QueueError),
}
