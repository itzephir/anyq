use anyq_queue_api::*;
use serde_json::{Value, json};

fn opaque(raw: &str) -> Value {
    serde_json::from_str(raw).unwrap_or_else(|_| json!(raw))
}

pub fn topic(t: &Topic) -> Value {
    json!({"name":t.name,"capacity":t.capacity})
}

pub fn group(g: &ConsumerGroup) -> Value {
    json!({"topic":g.topic,"name":g.name})
}

pub fn message(m: &Message) -> Value {
    json!({"id":m.id,"topic":m.topic,"payload":opaque(&m.payload),"key":m.key,"max_attempts":m.max_attempts,"created_at":m.created_at})
}

pub fn delivery(d: &Delivery) -> Value {
    json!({"message_id":d.message_id,"group":d.group,"state":d.state.as_str(),"attempts":d.attempts,"max_attempts":d.max_attempts,"available_at":d.available_at,"consumer":d.consumer,"token":d.token,"lease_until":d.lease_until,"finished_at":d.finished_at,"result":d.result.as_deref().map(opaque),"last_error":d.last_error})
}

pub fn view(v: &MessageView) -> Value {
    json!({"message":message(&v.message),"deliveries":v.deliveries.iter().map(delivery).collect::<Vec<_>>()})
}

pub fn error(e: &Error) -> Value {
    let kind = match e {
        Error::Invalid(_) => "invalid",
        Error::NotFound(_) => "not_found",
        Error::Conflict(_) => "conflict",
        Error::LeaseLost => "lease_lost",
        Error::Storage(_) => "storage",
    };

    json!({"error":{"kind":kind,"message":e.to_string()}})
}

pub fn run(run: anyq_worker_api::Run) -> Value {
    use anyq_worker_api::Run;

    match run {
        Run::Idle => json!({"state":"idle"}),
        Run::Stopped => json!({"state":"stopped"}),
        Run::Succeeded(id) => json!({"state":"succeeded","message_id":id}),
        Run::Failed(id) => json!({"state":"failed","message_id":id}),
    }
}

pub fn assignment(value: &Option<Assignment>) -> Value {
    value
        .as_ref()
        .map(|a| json!({"message":message(&a.message),"delivery":delivery(&a.delivery)}))
        .unwrap_or(Value::Null)
}

pub fn completion(value: &Completion) -> Value {
    json!({"delivery":delivery(&value.delivery),"next":value.next.as_ref().map(message)})
}
