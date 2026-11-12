use crate::*;

/// Passive composition handles. Every consumer receives a focused use-case contract.
pub struct QueueServices {
    pub get_task_metadata: Box<dyn GetTaskMetadata>,
    pub set_task_metadata: Box<dyn SetTaskMetadata>,
    pub unset_task_metadata: Box<dyn UnsetTaskMetadata>,
    pub replace_task_metadata: Box<dyn ReplaceTaskMetadata>,
    pub get_topic_metadata: Box<dyn GetTopicMetadata>,
    pub set_topic_metadata: Box<dyn SetTopicMetadata>,
    pub unset_topic_metadata: Box<dyn UnsetTopicMetadata>,
    pub replace_topic_metadata: Box<dyn ReplaceTopicMetadata>,
    pub create_topic: Box<dyn CreateTopic>,
    pub set_topic_capacity: Box<dyn SetTopicCapacity>,
    pub list_topics: Box<dyn ListTopics>,
    pub list_tasks: Box<dyn ListTasks>,
    pub get_topic: Box<dyn GetTopic>,
    pub get_task: Box<dyn GetTask>,
    pub get_topic_queue: Box<dyn GetTopicQueue>,
    pub register_task: Box<dyn RegisterTask>,
    pub add_task_to_topics: Box<dyn AddTaskToTopics>,
    pub add_tasks_to_topic: Box<dyn AddTasksToTopic>,
    pub add_task_to_topic: Box<dyn AddTaskToTopic>,
    pub check_task_running: Box<dyn CheckTaskRunning>,
    pub start_task: Box<dyn StartTask>,
    pub set_topic_autostart: Box<dyn SetTopicAutostart>,
    pub complete_task: Box<dyn CompleteTask>,
}
