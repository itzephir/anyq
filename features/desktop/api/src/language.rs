use anyq_queue_api::{Error, Result};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Language {
    #[default]
    English,
    Russian,
}

impl Language {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "en" => Ok(Self::English),
            "ru" => Ok(Self::Russian),
            _ => Err(Error::Invalid("unsupported panel language".into())),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Russian => "ru",
        }
    }
}
