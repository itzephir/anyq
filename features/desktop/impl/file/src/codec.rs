use anyq_desktop_api::{Error, Language, PanelPreferences, Result, Theme};
use toml::{Value, map::Map};

fn invalid(message: impl ToString) -> Error {
    Error::Invalid(message.to_string())
}

pub(crate) fn decode(text: &str) -> Result<PanelPreferences> {
    let table: Map<String, Value> = toml::from_str(text).map_err(invalid)?;

    for (key, value) in &table {
        if !["database_override", "theme", "language"].contains(&key.as_str()) || !value.is_str() {
            return Err(invalid(format!("invalid panel preference '{key}'")));
        }
    }

    let mut preferences = PanelPreferences {
        database_override: table
            .get("database_override")
            .and_then(Value::as_str)
            .map(str::to_owned),
        ..PanelPreferences::default()
    };
    if let Some(theme) = table.get("theme").and_then(Value::as_str) {
        preferences.theme = Theme::parse(theme)?;
    }
    if let Some(language) = table.get("language").and_then(Value::as_str) {
        preferences.language = Language::parse(language)?;
    }
    preferences.validate()?;

    Ok(preferences)
}

pub(crate) fn encode(preferences: &PanelPreferences) -> Result<String> {
    preferences.validate()?;
    let mut table = Map::new();
    if let Some(path) = &preferences.database_override {
        table.insert("database_override".into(), Value::String(path.clone()));
    }
    table.insert(
        "theme".into(),
        Value::String(preferences.theme.as_str().into()),
    );
    table.insert(
        "language".into(),
        Value::String(preferences.language.as_str().into()),
    );

    toml::to_string(&table).map_err(invalid)
}
