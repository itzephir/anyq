use crate::{Metadata, MetadataNumber, MetadataValue};
use std::collections::BTreeMap;

#[test]
fn decimals_are_exact_normalized_and_finite() {
    for (input, expected) in [
        ("9007199254740993", "9007199254740993"),
        ("18446744073709551616", "18446744073709551616"),
        (
            "1.230000000000000000000000001",
            "1.230000000000000000000000001",
        ),
        ("1.00e2", "100"),
        ("0.0100", "0.01"),
        ("-0.00e5", "0"),
        ("1e-7", "1e-7"),
        ("1e23", "1e23"),
    ] {
        let number = MetadataNumber::parse(input).unwrap();

        assert_eq!(number.as_str(), expected);
        assert_eq!(MetadataNumber::parse(number.as_str()).unwrap(), number);
    }

    for input in [
        "", "NaN", "inf", "1e309", "01", "+1", "1.", ".1", "1e", "1e+", "--1",
    ] {
        assert!(MetadataNumber::parse(input).is_err(), "{input}");
    }
}

#[test]
fn dotted_patches_preserve_unrelated_data_and_reject_scalar_traversal_atomically() {
    let mut metadata = Metadata::default();
    metadata
        .set("timeout", MetadataValue::String("2h".into()))
        .unwrap();
    metadata
        .set("shell.autocomplete", MetadataValue::Bool(true))
        .unwrap();
    let before = metadata.clone();

    assert!(metadata.set("timeout.unit", MetadataValue::Null).is_err());
    assert!(metadata.unset("timeout.unit").is_err());
    assert!(metadata.set("a..b", MetadataValue::Null).is_err());
    assert_eq!(metadata, before);
    assert_eq!(
        metadata.get(Some("shell.autocomplete")).unwrap(),
        &MetadataValue::Bool(true)
    );

    metadata.unset("absent.nested").unwrap();
    assert_eq!(metadata, before);
    metadata.unset("shell.autocomplete").unwrap();

    assert!(metadata.get(Some("shell.autocomplete")).is_err());
    assert_eq!(
        metadata.get(Some("timeout")).unwrap(),
        &MetadataValue::String("2h".into())
    );
}

#[test]
fn non_objects_and_limits_are_rejected_without_mutation() {
    assert!(Metadata::new(MetadataValue::Null).is_err());
    let mut metadata = Metadata::default();
    let before = metadata.clone();
    let mut value = MetadataValue::Null;

    for _ in 0..Metadata::MAX_DEPTH {
        value = MetadataValue::Array(vec![value]);
    }

    assert!(metadata.set("deep", value).is_err());
    assert!(
        metadata
            .set(
                "large",
                MetadataValue::String("x".repeat(Metadata::MAX_BYTES))
            )
            .is_err()
    );
    assert_eq!(metadata, before);
}

#[test]
fn imported_literal_dots_and_unknown_namespaces_remain_independent() {
    let metadata = Metadata::new(MetadataValue::Object(BTreeMap::from([
        (
            "literal.dot".into(),
            MetadataValue::Array(vec![MetadataValue::Null]),
        ),
        ("unknown".into(), MetadataValue::Bool(false)),
    ])))
    .unwrap();

    assert!(metadata.get(Some("literal.dot")).is_err());
    assert_eq!(
        metadata.get(Some("unknown")).unwrap(),
        &MetadataValue::Bool(false)
    );
    assert_eq!(Metadata::new(metadata.value().clone()).unwrap(), metadata);
}
