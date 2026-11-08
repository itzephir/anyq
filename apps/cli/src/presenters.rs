use anyq_queue_api::{Task, TopicQueue};

pub fn registered(name: &str) -> String {
    format!("successfully registered task '{name}'")
}

pub fn added(name: &str, topic: &str) -> String {
    format!("successfully added '{name}' to topic '{topic}'")
}

pub fn waiting(name: &str) -> String {
    format!("waiting for '{name}' to start")
}

pub fn started(name: &str) -> String {
    format!("'{name}' started")
}

pub fn completed(name: &str) -> String {
    format!("'{name}' successfully completed")
}
pub fn task(task: &Task) -> String {
    let topics = if task.topics().is_empty() {
        "topics: (none)".into()
    } else {
        format!(
            "topics:\n{}",
            task.topics()
                .iter()
                .map(|topic| format!("- {topic}"))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };

    format!(
        "task: {}\nstatus: {}\n{topics}",
        task.name(),
        task.state().as_str()
    )
}

pub fn topic_queue(queue: &TopicQueue) -> String {
    let tasks = if queue.tasks.is_empty() {
        "queue: (empty)".into()
    } else {
        format!(
            "queue:\n{}",
            queue
                .tasks
                .iter()
                .map(|task| format!("- {} [{}]", task.name(), task.state().as_str()))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };

    format!(
        "topic: {}\ncapacity: {}\n{tasks}",
        queue.topic.name, queue.topic.capacity
    )
}
