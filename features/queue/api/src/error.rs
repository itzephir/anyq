use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Invalid(String),
    NotFound(String),
    Conflict(String),
    LeaseLost,
    Storage(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(s) | Self::NotFound(s) | Self::Conflict(s) | Self::Storage(s) => {
                f.write_str(s)
            }

            Self::LeaseLost => f.write_str("lease expired, replaced, or already completed"),
        }
    }
}

impl std::error::Error for Error {}
