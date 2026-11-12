use crate::{Language, Theme};
use anyq_queue_api::{Error, Result};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PanelPreferences {
    pub database_override: Option<String>,
    pub theme: Theme,
    pub language: Language,
}

impl PanelPreferences {
    pub fn validate(&self) -> Result<()> {
        if self
            .database_override
            .as_ref()
            .is_some_and(String::is_empty)
        {
            return Err(Error::Invalid(
                "panel database override must not be empty".into(),
            ));
        }

        Ok(())
    }
}
