use super::*;
use anyq_queue_api::{Metadata, MetadataValue};
use std::collections::BTreeMap;

fn metadata(entries: impl IntoIterator<Item = (&'static str, MetadataValue)>) -> Metadata {
    let shell = MetadataValue::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key.into(), value))
            .collect(),
    );
    Metadata::new(MetadataValue::Object(BTreeMap::from([(
        "shell".into(),
        shell,
    )])))
    .unwrap()
}

fn array() -> MetadataValue {
    MetadataValue::Array(vec![MetadataValue::String("echo hello".into())])
}

#[test]
fn source_is_exclusive_and_optional_autocomplete_defaults_false() {
    assert!(Plan::from_task(&Metadata::default()).unwrap().is_none());
    let plan = Plan::from_task(&metadata([("sh", array())]))
        .unwrap()
        .unwrap();

    assert_eq!(plan.source, Source::Shell);
    assert!(!plan.autocomplete);
    for invalid in [
        metadata([]),
        metadata([("sh", array()), ("src", array())]),
        metadata([("sh", MetadataValue::Array(vec![]))]),
        metadata([("sh", MetadataValue::String("ls".into()))]),
        metadata([
            ("sh", array()),
            ("autocomplete", MetadataValue::String("true".into())),
        ]),
        metadata([
            ("sh", array()),
            ("cwd", MetadataValue::String(String::new())),
        ]),
    ] {
        assert!(Plan::from_task(&invalid).is_err());
    }
}

#[test]
fn only_explicit_string_environment_overrides_are_accepted() {
    let value = metadata([
        ("src", array()),
        (
            "env",
            MetadataValue::Object(BTreeMap::from([(
                "TOKEN".into(),
                MetadataValue::String("private".into()),
            )])),
        ),
    ]);
    assert_eq!(
        Plan::from_task(&value).unwrap().unwrap().environment["TOKEN"],
        "private"
    );
    let value = metadata([("src", array()), ("env", MetadataValue::Bool(true))]);

    assert!(Plan::from_task(&value).is_err());
}
