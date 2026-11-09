use anyq_config_api::{Config, ConfigKey, ConfigValue, Error, Result};

pub(crate) fn decode(input: &str) -> Result<Config> {
    let table = input
        .parse::<toml::Table>()
        .map_err(|error| Error::Storage(error.to_string()))?;
    let mut config = Config::default();
    decode_table("", &table, &mut config)?;

    Ok(config)
}

fn decode_table(prefix: &str, table: &toml::Table, config: &mut Config) -> Result<()> {
    for (key, value) in table {
        let name = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{prefix}.{key}")
        };

        if let toml::Value::Table(table) = value {
            decode_table(&name, table, config)?;
            continue;
        }

        let key = ConfigKey::parse(&name)?;
        let value = match value {
            toml::Value::Boolean(value) => ConfigValue::Boolean(*value),
            toml::Value::String(value) => ConfigValue::Text(value.clone()),
            _ => return Err(Error::Storage(format!("unsupported TOML value for {name}"))),
        };
        config.set(key, value)?;
    }

    Ok(())
}

pub(crate) fn encode(config: &Config) -> Result<String> {
    let mut table = toml::Table::new();

    for (key, value) in config.overrides() {
        let (section, name) = key
            .as_str()
            .split_once('.')
            .ok_or_else(|| Error::Storage("configuration key has no section".into()))?;
        let section = table
            .entry(section.to_owned())
            .or_insert_with(|| toml::Value::Table(toml::Table::new()))
            .as_table_mut()
            .ok_or_else(|| Error::Storage("invalid TOML section".into()))?;
        let value = match value {
            ConfigValue::Boolean(value) => toml::Value::Boolean(value),
            ConfigValue::Text(value) => toml::Value::String(value),
        };
        section.insert(name.into(), value);
    }

    toml::to_string_pretty(&table).map_err(|error| Error::Storage(error.to_string()))
}
