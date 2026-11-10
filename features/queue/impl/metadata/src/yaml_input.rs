use crate::yaml_scalar::scalar;
use anyq_queue_api::{Error, Metadata, MetadataBudget, MetadataValue, Result};
use std::{collections::BTreeMap, str::Chars};
use yaml_rust2::parser::{Event, Parser};

pub(crate) struct YamlInput<'a> {
    parser: Parser<Chars<'a>>,
    anchors: BTreeMap<usize, MetadataValue>,
    budget: MetadataBudget,
    anchor_budget: MetadataBudget,
    aliases: usize,
}

fn invalid(error: impl std::fmt::Display) -> Error {
    Error::Invalid(format!("invalid metadata YAML: {error}"))
}

impl<'a> YamlInput<'a> {
    pub fn parse(input: &'a str) -> Result<Metadata> {
        if input.len() > Metadata::MAX_BYTES {
            return Err(invalid("file exceeds 1 MiB"));
        }
        let mut reader = Self {
            parser: Parser::new_from_str(input),
            anchors: BTreeMap::new(),
            budget: MetadataBudget::default(),
            anchor_budget: MetadataBudget::default(),
            aliases: 0,
        };
        if reader.event()? != Event::StreamStart || reader.event()? != Event::DocumentStart {
            return Err(invalid("expected one object document"));
        }
        let value = reader.node(1)?;
        if reader.event()? != Event::DocumentEnd || reader.event()? != Event::StreamEnd {
            return Err(invalid("expected exactly one document"));
        }

        Metadata::new(value)
    }

    fn event(&mut self) -> Result<Event> {
        self.parser
            .next_token()
            .map(|(event, _)| event)
            .map_err(invalid)
    }

    fn anchor(&mut self, id: usize, value: &MetadataValue) -> Result<()> {
        if id != 0 {
            if self.anchors.len() >= 1024 {
                return Err(invalid("too many anchors"));
            }
            self.anchor_budget.include(value, 1)?;
            self.anchors.insert(id, value.clone());
        }

        Ok(())
    }

    fn alias(&mut self, id: usize) -> Result<&MetadataValue> {
        self.aliases += 1;
        if self.aliases > 1024 {
            return Err(invalid("too many aliases"));
        }

        self.anchors
            .get(&id)
            .ok_or_else(|| invalid("recursive or undefined alias"))
    }

    fn key(&mut self) -> Result<String> {
        let value = match self.event()? {
            Event::Scalar(text, style, id, None) => {
                let value = scalar(text, style)?;
                self.anchor(id, &value)?;

                value
            }
            Event::Alias(id) => self.alias(id)?.clone(),
            _ => return Err(invalid("mapping keys must be untagged strings")),
        };
        let MetadataValue::String(key) = value else {
            return Err(invalid("mapping keys must be strings"));
        };
        self.budget.add_text(&key)?;

        Ok(key)
    }

    fn node(&mut self, depth: usize) -> Result<MetadataValue> {
        let event = self.event()?;
        if let Event::Alias(id) = event {
            // Charge the expanded subtree before allocating its copy.
            let _ = self.alias(id)?;
            let value = self.anchors.get(&id).expect("alias exists");
            self.budget.include(value, depth)?;

            return Ok(value.clone());
        }
        self.budget.add_value(depth)?;

        let (id, value) = match event {
            Event::Scalar(text, style, id, None) => {
                let value = scalar(text, style)?;
                match &value {
                    MetadataValue::String(text) => self.budget.add_text(text)?,
                    MetadataValue::Number(number) => self.budget.add_text(number.as_str())?,
                    _ => {}
                }

                (id, value)
            }
            Event::SequenceStart(id, None) => {
                let mut items = Vec::new();
                while !matches!(self.parser.peek().map_err(invalid)?.0, Event::SequenceEnd) {
                    items.push(self.node(depth + 1)?);
                }
                self.event()?;

                (id, MetadataValue::Array(items))
            }
            Event::MappingStart(id, None) => {
                let mut object = BTreeMap::new();
                while !matches!(self.parser.peek().map_err(invalid)?.0, Event::MappingEnd) {
                    let key = self.key()?;
                    if object.contains_key(&key) {
                        return Err(invalid(format!("duplicate key '{key}'")));
                    }
                    object.insert(key, self.node(depth + 1)?);
                }
                self.event()?;

                (id, MetadataValue::Object(object))
            }
            _ => return Err(invalid("tags and non-JSON values are unsupported")),
        };
        self.anchor(id, &value)?;

        Ok(value)
    }
}
