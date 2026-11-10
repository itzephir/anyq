use anyq_queue_api::{Error, Metadata, MetadataBudget, MetadataNumber, MetadataValue, Result};
use std::collections::BTreeMap;
use toml_edit::{DocumentMut, Item, Table, Value};

fn object(table: &Table, depth: usize, budget: &mut MetadataBudget) -> Result<MetadataValue> {
    budget.add_value(depth)?;
    let mut object = BTreeMap::new();
    for (key, child) in table.iter() {
        budget.add_text(key)?;
        object.insert(key.into(), item(child, depth + 1, budget)?);
    }

    Ok(MetadataValue::Object(object))
}

fn value(value: &Value, depth: usize, budget: &mut MetadataBudget) -> Result<MetadataValue> {
    budget.add_value(depth)?;

    Ok(match value {
        Value::String(value) => {
            budget.add_text(value.value())?;
            MetadataValue::String(value.value().clone())
        }
        Value::Boolean(value) => MetadataValue::Bool(*value.value()),
        Value::Integer(value) => {
            let number = MetadataNumber::parse(&value.value().to_string())?;
            budget.add_text(number.as_str())?;
            MetadataValue::Number(number)
        }
        Value::Float(value) => {
            let text = value.display_repr().replace('_', "");
            let number = MetadataNumber::parse(text.strip_prefix('+').unwrap_or(&text))?;
            budget.add_text(number.as_str())?;
            MetadataValue::Number(number)
        }
        Value::Datetime(_) => {
            return Err(Error::Invalid(
                "TOML dates are not JSON-compatible metadata".into(),
            ));
        }
        Value::Array(values) => MetadataValue::Array(
            values
                .iter()
                .map(|child| self::value(child, depth + 1, budget))
                .collect::<Result<_>>()?,
        ),
        Value::InlineTable(table) => {
            let mut object = BTreeMap::new();
            for (key, child) in table.iter() {
                budget.add_text(key)?;
                object.insert(key.into(), self::value(child, depth + 1, budget)?);
            }
            MetadataValue::Object(object)
        }
    })
}

fn item(item: &Item, depth: usize, budget: &mut MetadataBudget) -> Result<MetadataValue> {
    match item {
        Item::None => Err(Error::Invalid("unsupported empty TOML value".into())),
        Item::Value(child) => value(child, depth, budget),
        Item::Table(table) => object(table, depth, budget),
        Item::ArrayOfTables(tables) => {
            budget.add_value(depth)?;

            tables
                .iter()
                .map(|table| object(table, depth + 1, budget))
                .collect::<Result<Vec<_>>>()
                .map(MetadataValue::Array)
        }
    }
}

pub fn parse_toml(input: &str) -> Result<Metadata> {
    if input.len() > Metadata::MAX_BYTES {
        return Err(Error::Invalid("metadata TOML file exceeds 1 MiB".into()));
    }
    let document = input
        .parse::<DocumentMut>()
        .map_err(|error| Error::Invalid(format!("invalid metadata TOML: {error}")))?;

    Metadata::new(object(
        document.as_table(),
        1,
        &mut MetadataBudget::default(),
    )?)
}
