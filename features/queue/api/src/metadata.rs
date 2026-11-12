use crate::{Error, MetadataPath, MetadataValue, Result};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata(MetadataValue);

impl Default for Metadata {
    fn default() -> Self {
        Self(MetadataValue::Object(BTreeMap::new()))
    }
}

impl Metadata {
    pub const MAX_BYTES: usize = 1024 * 1024;
    pub const MAX_DEPTH: usize = 64;
    pub const VALUE_BYTES: usize = 8;

    pub fn new(value: MetadataValue) -> Result<Self> {
        if !matches!(value, MetadataValue::Object(_)) {
            return Err(Error::Invalid("metadata root must be an object".into()));
        }
        value.validate()?;

        Ok(Self(value))
    }

    pub fn value(&self) -> &MetadataValue {
        &self.0
    }

    pub fn into_value(self) -> MetadataValue {
        self.0
    }

    pub fn get(&self, key: Option<&str>) -> Result<&MetadataValue> {
        let Some(key) = key else {
            return Ok(&self.0);
        };
        let path = MetadataPath::parse(key)?;
        let mut value = &self.0;

        for part in path.segments() {
            let MetadataValue::Object(object) = value else {
                return Err(Error::Invalid(
                    "metadata path crosses a scalar or array".into(),
                ));
            };
            value = object
                .get(part)
                .ok_or_else(|| Error::Invalid(format!("metadata key '{key}' is missing")))?;
        }

        Ok(value)
    }

    pub fn set(&mut self, key: &str, value: MetadataValue) -> Result<()> {
        value.validate()?;
        let path = MetadataPath::parse(key)?;
        let mut candidate = self.clone();
        let mut parent = &mut candidate.0;
        let (last, preceding) = path.segments().split_last().expect("nonempty path");

        for part in preceding {
            let MetadataValue::Object(object) = parent else {
                return Err(Error::Invalid(
                    "metadata path crosses a scalar or array".into(),
                ));
            };
            parent = object
                .entry(part.clone())
                .or_insert_with(|| MetadataValue::Object(BTreeMap::new()));
        }
        let MetadataValue::Object(object) = parent else {
            return Err(Error::Invalid(
                "metadata path crosses a scalar or array".into(),
            ));
        };
        object.insert(last.clone(), value);
        candidate.0.validate()?;
        *self = candidate;

        Ok(())
    }

    pub fn unset(&mut self, key: &str) -> Result<()> {
        let path = MetadataPath::parse(key)?;
        let mut candidate = self.clone();
        let mut parent = &mut candidate.0;
        let (last, preceding) = path.segments().split_last().expect("nonempty path");

        for part in preceding {
            let MetadataValue::Object(object) = parent else {
                return Err(Error::Invalid(
                    "metadata path crosses a scalar or array".into(),
                ));
            };
            let Some(value) = object.get_mut(part) else {
                return Ok(());
            };
            parent = value;
        }
        let MetadataValue::Object(object) = parent else {
            return Err(Error::Invalid(
                "metadata path crosses a scalar or array".into(),
            ));
        };
        object.remove(last);
        *self = candidate;

        Ok(())
    }
}
