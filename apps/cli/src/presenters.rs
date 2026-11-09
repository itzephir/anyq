use anyq_queue_api::{Task, TopicQueue, format_name};

pub fn registered(name: &str) -> String {
    format!("successfully registered task '{}'", format_name(name))
}

pub fn added(name: &str, topic: &str) -> String {
    format!(
        "successfully added '{}' to topic '{}'",
        format_name(name),
        format_name(topic)
    )
}

pub fn waiting(name: &str) -> String {
    format!("waiting for '{}' to start", format_name(name))
}

pub fn started(name: &str) -> String {
    format!("'{}' started", format_name(name))
}

pub fn completed(name: &str) -> String {
    format!("'{}' successfully completed", format_name(name))
}

pub fn task(task: &Task) -> String {
    let topics = if task.topics().is_empty() {
        "topics: (none)".into()
    } else {
        format!(
            "topics:\n{}",
            task.topics()
                .iter()
                .map(|topic| format!("- {}", format_name(topic)))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };

    format!(
        "task: {}\nstatus: {}\n{topics}",
        format_name(task.name()),
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
                .map(|task| format!("- {} [{}]", format_name(task.name()), task.state().as_str()))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };

    format!(
        "topic: {}\ncapacity: {}\n{tasks}",
        format_name(&queue.topic.name),
        queue.topic.capacity
    )
}
