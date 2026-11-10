use anyq_daemon_api::{QueueRequest, QueueResponse, QueueServices};
use anyq_queue_api::{
    AddTaskToTopicsRequest, AddTasksToTopicRequest, Error, RegisterTaskRequest, Result,
    StartTaskOutcome,
};

pub(crate) fn route(request: QueueRequest, services: &mut QueueServices) -> Result<QueueResponse> {
    Ok(match request {
        QueueRequest::GetTaskMetadata { task, key } => QueueResponse::Metadata(
            services
                .get_task_metadata
                .get(anyq_queue_api::GetTaskMetadataRequest { task, key })?
                .into(),
        ),
        QueueRequest::SetTaskMetadata { task, key, value } => QueueResponse::Metadata(
            services
                .set_task_metadata
                .set(anyq_queue_api::SetTaskMetadataRequest {
                    task,
                    key,
                    value: value.try_into()?,
                })?
                .into(),
        ),
        QueueRequest::UnsetTaskMetadata { task, key } => QueueResponse::Metadata(
            services
                .unset_task_metadata
                .unset(anyq_queue_api::UnsetTaskMetadataRequest { task, key })?
                .into(),
        ),
        QueueRequest::ReplaceTaskMetadata { task, metadata } => QueueResponse::Metadata(
            services
                .replace_task_metadata
                .replace(anyq_queue_api::ReplaceTaskMetadataRequest {
                    task,
                    metadata: metadata.try_into()?,
                })?
                .into(),
        ),
        QueueRequest::GetTopicMetadata { topic, key } => QueueResponse::Metadata(
            services
                .get_topic_metadata
                .get(anyq_queue_api::GetTopicMetadataRequest { topic, key })?
                .into(),
        ),
        QueueRequest::SetTopicMetadata { topic, key, value } => QueueResponse::Metadata(
            services
                .set_topic_metadata
                .set(anyq_queue_api::SetTopicMetadataRequest {
                    topic,
                    key,
                    value: value.try_into()?,
                })?
                .into(),
        ),
        QueueRequest::UnsetTopicMetadata { topic, key } => QueueResponse::Metadata(
            services
                .unset_topic_metadata
                .unset(anyq_queue_api::UnsetTopicMetadataRequest { topic, key })?
                .into(),
        ),
        QueueRequest::ReplaceTopicMetadata { topic, metadata } => QueueResponse::Metadata(
            services
                .replace_topic_metadata
                .replace(anyq_queue_api::ReplaceTopicMetadataRequest {
                    topic,
                    metadata: metadata.try_into()?,
                })?
                .into(),
        ),
        QueueRequest::ListTasks => QueueResponse::Names(services.list_tasks.list()?),
        QueueRequest::ListTopics => QueueResponse::Topics(
            services
                .list_topics
                .list()?
                .into_iter()
                .map(Into::into)
                .collect(),
        ),
        QueueRequest::GetTask { name } => QueueResponse::Task(services.get_task.get(&name)?.into()),
        QueueRequest::GetTopic { name } => {
            QueueResponse::Topic(services.get_topic.get(&name)?.into())
        }
        QueueRequest::GetTopicQueue { name } => {
            QueueResponse::TopicQueue(services.get_topic_queue.get(&name)?.into())
        }
        QueueRequest::CreateTopic { topic } => {
            QueueResponse::Topic(services.create_topic.create(topic.try_into()?)?.into())
        }
        QueueRequest::SetTopicCapacity { topic } => QueueResponse::Topic(
            services
                .set_topic_capacity
                .set_capacity(topic.try_into()?)?
                .into(),
        ),
        QueueRequest::SetTopicAutostart { name, enabled } => QueueResponse::Topic(
            services
                .set_topic_autostart
                .set_autostart(&name, enabled)?
                .into(),
        ),
        QueueRequest::RegisterTask {
            name,
            topics,
            start,
            metadata,
        } => QueueResponse::Task(
            services
                .register_task
                .register(RegisterTaskRequest {
                    metadata: metadata.try_into()?,
                    name,
                    topics,
                    start: start.into(),
                })?
                .into(),
        ),
        QueueRequest::AddTaskToTopic { task, topic } => {
            QueueResponse::Task(services.add_task_to_topic.add(&task, &topic)?.into())
        }
        QueueRequest::AddTaskToTopics { task, topics } => QueueResponse::Task(
            services
                .add_task_to_topics
                .add(AddTaskToTopicsRequest { task, topics })?
                .into(),
        ),
        QueueRequest::AddTasksToTopic { tasks, topic } => QueueResponse::Tasks(
            services
                .add_tasks_to_topic
                .add(AddTasksToTopicRequest { tasks, topic })?
                .into_iter()
                .map(Into::into)
                .collect(),
        ),
        QueueRequest::CheckTaskRunning { name } => {
            QueueResponse::Running(services.check_task_running.check_running(&name)?)
        }
        QueueRequest::StartTask { name } => QueueResponse::Started(
            match services.start_task.start(&name)? {
                StartTaskOutcome::Started => "started",
                StartTaskOutcome::AlreadyRunning => "already-running",
                StartTaskOutcome::Busy => "busy",
            }
            .into(),
        ),
        QueueRequest::CompleteTask { name, force } => {
            QueueResponse::Task(services.complete_task.complete(&name, force)?.into())
        }
        _ => return Err(Error::Invalid("invalid queue session operation".into())),
    })
}
