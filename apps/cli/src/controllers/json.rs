use anyq_queue_api::{Error, Result};

fn invalid(error: impl std::fmt::Display) -> Error {
    Error::Invalid(error.to_string())
}

pub fn json(raw: &str) -> Result<String> {
    serde_json::from_str::<serde_json::Value>(raw)
        .map(|v| v.to_string())
        .map_err(invalid)
}
