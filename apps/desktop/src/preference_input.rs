use anyq_desktop_api::{Language, PanelPreferences, Theme};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreferenceInput {
    database_override: Option<String>,
    theme: String,
    language: String,
}

impl PreferenceInput {
    pub fn parse(self) -> Result<PanelPreferences, String> {
        let database_override = self
            .database_override
            .map(|path| {
                if path.is_empty() {
                    return Err("empty database override".to_owned());
                }
                std::path::absolute(path)
                    .map(|path| path.to_string_lossy().into_owned())
                    .map_err(|error| error.to_string())
            })
            .transpose()?;

        Ok(PanelPreferences {
            database_override,
            theme: Theme::parse(&self.theme).map_err(|error| error.to_string())?,
            language: Language::parse(&self.language).map_err(|error| error.to_string())?,
        })
    }
}
