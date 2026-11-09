use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
    Queue(anyq_queue_api::Error),
    Config(anyq_config_api::Error),
    Io(std::io::Error),
    Invalid(String),
    MissingConfigValue,
    Interrupted,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Queue(error) => Display::fmt(error, formatter),
            Self::Config(error) => Display::fmt(error, formatter),
            Self::Io(error) => Display::fmt(error, formatter),
            Self::Invalid(message) => formatter.write_str(message),
            Self::MissingConfigValue => formatter.write_str("configuration value is not set"),
            Self::Interrupted => formatter.write_str("interrupted"),
        }
    }
}

impl Error {
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Interrupted
            | Self::Queue(anyq_queue_api::Error::Interrupted)
            | Self::Config(anyq_config_api::Error::Interrupted) => 130,
            Self::MissingConfigValue => 1,
            _ => 2,
        }
    }
}

impl From<anyq_queue_api::Error> for Error {
    fn from(error: anyq_queue_api::Error) -> Self {
        Self::Queue(error)
    }
}

impl From<anyq_config_api::Error> for Error {
    fn from(error: anyq_config_api::Error) -> Self {
        Self::Config(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}
