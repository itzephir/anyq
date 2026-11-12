use anyq_queue_api::{Error, Metadata, MetadataNumber, MetadataValue, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Wire values keep decimals as text so no intermediary rounds them to floats.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum MetadataDto {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array(Vec<MetadataDto>),
    Object(Vec<(String, MetadataDto)>),
}

impl From<MetadataValue> for MetadataDto {
    fn from(value: MetadataValue) -> Self {
        match value {
            MetadataValue::Null => Self::Null,
            MetadataValue::Bool(value) => Self::Bool(value),
            MetadataValue::Number(value) => Self::Number(value.as_str().into()),
            MetadataValue::String(value) => Self::String(value),
            MetadataValue::Array(values) => {
                Self::Array(values.into_iter().map(Into::into).collect())
            }
            MetadataValue::Object(values) => Self::Object(
                values
                    .into_iter()
                    .map(|(key, value)| (key, value.into()))
                    .collect(),
            ),
        }
    }
}

impl From<Metadata> for MetadataDto {
    fn from(metadata: Metadata) -> Self {
        metadata.into_value().into()
    }
}

impl TryFrom<MetadataDto> for MetadataValue {
    type Error = Error;

    fn try_from(value: MetadataDto) -> Result<Self> {
        let value = match value {
            MetadataDto::Null => Self::Null,
            MetadataDto::Bool(value) => Self::Bool(value),
            MetadataDto::Number(value) => Self::Number(MetadataNumber::parse(&value)?),
            MetadataDto::String(value) => Self::String(value),
            MetadataDto::Array(values) => Self::Array(
                values
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_>>()?,
            ),
            MetadataDto::Object(values) => {
                let mut object = BTreeMap::new();
                for (key, value) in values {
                    if object.insert(key.clone(), value.try_into()?).is_some() {
                        return Err(Error::Invalid(format!("duplicate metadata key '{key}'")));
                    }
                }

                Self::Object(object)
            }
        };
        value.validate()?;

        Ok(value)
    }
}

impl TryFrom<MetadataDto> for Metadata {
    type Error = Error;

    fn try_from(value: MetadataDto) -> Result<Self> {
        Self::new(value.try_into()?)
    }
}
