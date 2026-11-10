use anyq_queue_api::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "message",
    rename_all = "kebab-case",
    deny_unknown_fields
)]
pub enum QueueError {
    Invalid(String),
    NotFound(String),
    Conflict(String),
    Storage(String),
    Interrupted,
}

impl From<Error> for QueueError {
    fn from(value: Error) -> Self {
        match value {
            Error::Invalid(value) => Self::Invalid(value),
            Error::NotFound(value) => Self::NotFound(value),
            Error::Conflict(value) => Self::Conflict(value),
            Error::Storage(value) => Self::Storage(value),
            Error::Interrupted => Self::Interrupted,
        }
    }
}

impl From<QueueError> for Error {
    fn from(value: QueueError) -> Self {
        match value {
            QueueError::Invalid(value) => Self::Invalid(value),
            QueueError::NotFound(value) => Self::NotFound(value),
            QueueError::Conflict(value) => Self::Conflict(value),
            QueueError::Storage(value) => Self::Storage(value),
            QueueError::Interrupted => Self::Interrupted,
        }
    }
}
