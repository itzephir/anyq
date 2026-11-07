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
