use crate::{PolicyDto, StartMode, TopicDto};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(
    tag = "operation",
    content = "input",
    rename_all = "kebab-case",
    deny_unknown_fields
)]
pub enum QueueRequest {
    Open {
        database_id: String,
        policy: PolicyDto,
        #[cfg(feature = "integration-test-hooks")]
        gate: crate::GateDto,
    },
    Ping,
    Stop,
    Cancel,
    ListTasks,
    ListTopics,
    GetTask {
        name: String,
    },
    GetTopic {
        name: String,
    },
    GetTopicQueue {
        name: String,
    },
    CreateTopic {
        topic: TopicDto,
    },
    SetTopicCapacity {
        topic: TopicDto,
    },
    SetTopicAutostart {
        name: String,
        enabled: bool,
    },
    RegisterTask {
        name: String,
        topics: Vec<String>,
        start: StartMode,
    },
    AddTaskToTopic {
        task: String,
        topic: String,
    },
    AddTaskToTopics {
        task: String,
        topics: Vec<String>,
    },
    AddTasksToTopic {
        tasks: Vec<String>,
        topic: String,
    },
    CheckTaskRunning {
        name: String,
    },
    StartTask {
        name: String,
    },
    CompleteTask {
        name: String,
        force: bool,
    },
}
