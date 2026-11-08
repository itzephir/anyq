use anyq_queue_api::*;

/// Passive composition handles. Every consumer receives a focused use-case contract.
pub struct QueueModule {
    pub create_topic: Box<dyn CreateTopic>,
    pub set_topic_capacity: Box<dyn SetTopicCapacity>,
    pub list_topics: Box<dyn ListTopics>,
    pub list_tasks: Box<dyn ListTasks>,
    pub get_topic: Box<dyn GetTopic>,
    pub get_task: Box<dyn GetTask>,
    pub get_topic_queue: Box<dyn GetTopicQueue>,
    pub register_task: Box<dyn RegisterTask>,
    pub add_task_to_topic: Box<dyn AddTaskToTopic>,
    pub try_start_task: Box<dyn TryStartTask>,
    pub complete_task: Box<dyn CompleteTask>,
}
