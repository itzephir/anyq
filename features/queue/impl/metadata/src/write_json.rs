use anyq_queue_api::{Error, MetadataValue, Result};

fn string(value: &str) -> Result<String> {
    serde_json::to_string(value).map_err(|error| Error::Invalid(error.to_string()))
}

fn write(value: &MetadataValue, output: &mut String) -> Result<()> {
    match value {
        MetadataValue::Null => output.push_str("null"),
        MetadataValue::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
        MetadataValue::Number(number) => output.push_str(number.as_str()),
        MetadataValue::String(value) => output.push_str(&string(value)?),
        MetadataValue::Array(items) => {
            output.push('[');
            for (index, item) in items.iter().enumerate() {
                if index != 0 {
                    output.push(',');
                }
                write(item, output)?;
            }
            output.push(']');
        }
        MetadataValue::Object(items) => {
            output.push('{');
            for (index, (key, item)) in items.iter().enumerate() {
                if index != 0 {
                    output.push(',');
                }
                output.push_str(&string(key)?);
                output.push(':');
                write(item, output)?;
            }
            output.push('}');
        }
    }

    Ok(())
}

pub fn to_json(value: &MetadataValue) -> Result<String> {
    value.validate()?;
    let mut output = String::new();
    write(value, &mut output)?;

    Ok(output)
}
