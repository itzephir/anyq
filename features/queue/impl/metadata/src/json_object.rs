use crate::json::parse_raw;
use anyq_queue_api::{MetadataBudget, MetadataValue};
use serde::de::{Error, MapAccess, Visitor};
use serde_json::value::RawValue;
use std::{collections::BTreeMap, fmt};

pub(crate) struct JsonObject<'a> {
    pub depth: usize,
    pub budget: &'a mut MetadataBudget,
}

impl<'de> Visitor<'de> for JsonObject<'_> {
    type Value = BTreeMap<String, MetadataValue>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an object with unique string keys")
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let mut object = BTreeMap::new();

        while let Some(key) = map.next_key::<String>()? {
            if object.contains_key(&key) {
                return Err(M::Error::custom(format!("duplicate metadata key '{key}'")));
            }
            self.budget.add_text(&key).map_err(M::Error::custom)?;
            let raw: Box<RawValue> = map.next_value()?;
            let value = parse_raw(&raw, self.depth + 1, self.budget).map_err(M::Error::custom)?;
            object.insert(key, value);
        }

        Ok(object)
    }
}
