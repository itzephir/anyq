use crate::json_object::JsonObject;
use anyq_queue_api::{Error, Metadata, MetadataBudget, MetadataNumber, MetadataValue, Result};
use serde::Deserializer;
use serde_json::value::RawValue;

fn invalid(error: impl std::fmt::Display) -> Error {
    Error::Invalid(format!("invalid metadata JSON: {error}"))
}

pub fn parse_json(input: &str) -> Result<MetadataValue> {
    // Domain size counts decoded UTF-8 bytes. JSON escaping can expand them 6x.
    if input.len() > Metadata::MAX_BYTES * 6 {
        return Err(Error::Invalid(
            "metadata JSON exceeds encoded size limit".into(),
        ));
    }
    let raw: &RawValue = serde_json::from_str(input).map_err(invalid)?;

    parse_raw(raw, 1, &mut MetadataBudget::default())
}

pub fn parse_value(input: &str) -> Result<MetadataValue> {
    if input.len() > Metadata::MAX_BYTES * 6 {
        return Err(Error::Invalid(
            "metadata value exceeds encoded size limit".into(),
        ));
    }

    if serde_json::from_str::<&RawValue>(input).is_ok() {
        parse_json(input)
    } else {
        let value = MetadataValue::String(input.into());
        value.validate()?;

        Ok(value)
    }
}

pub(crate) fn parse_raw(
    raw: &RawValue,
    depth: usize,
    budget: &mut MetadataBudget,
) -> Result<MetadataValue> {
    budget.add_value(depth)?;
    let input = raw.get();

    Ok(match input.as_bytes()[0] {
        b'{' => {
            let mut deserializer = serde_json::Deserializer::from_str(input);
            let object = deserializer
                .deserialize_map(JsonObject { depth, budget })
                .map_err(invalid)?;

            MetadataValue::Object(object)
        }
        b'[' => {
            let items: Vec<Box<RawValue>> = serde_json::from_str(input).map_err(invalid)?;
            MetadataValue::Array(
                items
                    .iter()
                    .map(|value| parse_raw(value, depth + 1, budget))
                    .collect::<Result<_>>()?,
            )
        }
        b'"' => {
            let text: String = serde_json::from_str(input).map_err(invalid)?;
            budget.add_text(&text)?;

            MetadataValue::String(text)
        }
        b't' => MetadataValue::Bool(true),
        b'f' => MetadataValue::Bool(false),
        b'n' => MetadataValue::Null,
        _ => {
            let number = MetadataNumber::parse(input)?;
            budget.add_text(number.as_str())?;

            MetadataValue::Number(number)
        }
    })
}
