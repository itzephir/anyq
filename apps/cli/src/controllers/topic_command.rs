use anyq_queue_api::Topic;

pub enum TopicCommand {
    List,
    Create(Topic),
    Capacity(Topic),
    Add { topic: String, task: String },
}
