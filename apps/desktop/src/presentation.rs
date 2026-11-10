use crate::controller::Controller;
use anyq_queue_api::TaskState;
use serde_json::{Value, json};

pub fn state(controller: &Controller) -> Value {
    let session = controller.session();
    let preferences = &session.preferences;
    let snapshot = session.model.snapshot.as_ref().map(|snapshot| json!({
        "topics": snapshot.topics.iter().map(|topic| json!({
            "name": topic.name, "capacity": topic.capacity, "autostart": topic.autostart,
            "running": snapshot.topic_count(&topic.name, TaskState::Running),
            "queued": snapshot.topic_count(&topic.name, TaskState::Queued),
        })).collect::<Vec<_>>(),
        "tasks": snapshot.tasks.iter().map(|task| json!({
            "name": task.name(), "sequence": task.sequence(), "state": task.state().as_str(),
            "topics": task.topics(), "autostart_default": task.autostart_default(), "stopped": task.stopped(),
        })).collect::<Vec<_>>(),
        "running": snapshot.count(TaskState::Running), "queued": snapshot.count(TaskState::Queued),
    }));

    let error = session
        .model
        .error
        .as_deref()
        .unwrap_or_default()
        .to_lowercase();
    let error_kind = if session.preference_error.is_some() {
        "preferences"
    } else if error.contains("does not exist") || error.contains("no such file") {
        "missing"
    } else if error.contains("incompatible")
        || error.contains("schema")
        || error.contains("not a database")
    {
        "incompatible"
    } else if error.contains("limit") || error.contains("too large") || error.contains("exceeds") {
        "limit"
    } else {
        "unavailable"
    };

    json!({
        "error_kind": error_kind,
        "generation": session.model.generation, "status": session.model.status.as_str(),
        "path": session.selection.as_ref().map(|selection| selection.path.to_string_lossy().into_owned()).or_else(|| preferences.database_override.clone()).unwrap_or_default(),
        "source": session.selection.as_ref().map(|selection| selection.source.as_str()).unwrap_or(if preferences.database_override.is_some() {"override"} else {"default"}),
        "preferences": { "database_override": preferences.database_override, "theme": preferences.theme.as_str(), "language": preferences.language.as_str() },
        "last_success": session.last_success, "snapshot": snapshot,
    })
}
