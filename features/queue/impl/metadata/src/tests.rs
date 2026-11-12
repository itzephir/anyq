use crate::{parse_json, parse_value, to_json};
use anyq_queue_api::{Metadata, MetadataValue};

#[test]
fn json_round_trips_exact_numbers_and_rejects_duplicates_at_every_depth() {
    let text = r#"{"n":9007199254740993,"array":[null,true,1.230000000000000000001],"literal.dot":"line\n\u0000"}"#;

    let value = parse_json(text).unwrap();
    let encoded = to_json(&value).unwrap();

    assert!(encoded.contains("9007199254740993"));
    assert!(encoded.contains("1.230000000000000000001"));
    assert_eq!(parse_json(&encoded).unwrap(), value);
    for text in [
        r#"{"x":1,"x":2}"#,
        r#"{"nested":[{"x":1,"x":2}]}"#,
        r#"{"x":1e999}"#,
    ] {
        assert!(parse_json(text).is_err(), "{text}");
        assert!(parse_value(text).is_err(), "{text}");
    }
}

#[test]
fn cli_literal_strings_and_json_values_have_distinct_types() {
    assert_eq!(
        parse_value("2h").unwrap(),
        MetadataValue::String("2h".into())
    );
    assert_eq!(parse_value("true").unwrap(), MetadataValue::Bool(true));
    assert_eq!(parse_value("null").unwrap(), MetadataValue::Null);
    assert_eq!(
        parse_value(r#""true""#).unwrap(),
        MetadataValue::String("true".into())
    );
    assert!(Metadata::new(parse_json("[]").unwrap()).is_err());
}

#[test]
fn depth_and_decoded_size_limits_apply_before_objects_are_completed() {
    let valid = format!("{}null{}", "[".repeat(63), "]".repeat(63));
    let invalid = format!("[{valid}]");

    assert!(parse_json(&valid).is_ok());
    assert!(parse_json(&invalid).is_err());
    assert!(parse_json(&format!("\"{}\"", "x".repeat(Metadata::MAX_BYTES))).is_err());
}

#[cfg(feature = "files")]
#[test]
fn every_file_format_preserves_the_same_exact_values() {
    let json = r#"{"n":9007199254740993,"precise":0.123456789012345678901,"timeout":"2h","shell":{"autocomplete":false,"sh":["printf ok"]},"literal.dot":{"x":true}}"#;
    let yaml = "n: 9007199254740993\nprecise: 0.123456789012345678901\ntimeout: 2h\nshell:\n  autocomplete: false\n  sh: [printf ok]\nliteral.dot: {x: true}\n";
    let toml = "n = 9007199254740993\nprecise = 0.123456789012345678901\ntimeout = '2h'\n'literal.dot' = {x = true}\n[shell]\nautocomplete = false\nsh = ['printf ok']\n";
    let expected = Metadata::new(parse_json(json).unwrap()).unwrap();

    assert_eq!(crate::parse_yaml(yaml).unwrap(), expected);
    assert_eq!(crate::parse_toml(toml).unwrap(), expected);
}

#[cfg(feature = "files")]
#[test]
fn yaml_aliases_are_bounded_and_tags_non_string_keys_and_duplicates_are_rejected() {
    let metadata = crate::parse_yaml("original: &a [true, ok]\ncopy: *a\n").unwrap();

    assert_eq!(
        metadata.get(Some("original")).unwrap(),
        metadata.get(Some("copy")).unwrap()
    );
    for yaml in [
        "x: 1\nx: 2",
        "x: {y: 1, y: 2}",
        "true: 1",
        "x: !run command",
        "x: !!str text",
        "x: .nan",
        "x: +.inf",
        "x: &a [*a]",
        "[one, two]",
        "x: 1\n---\nx: 2",
    ] {
        assert!(crate::parse_yaml(yaml).is_err(), "{yaml}");
    }
    let mut bomb = "a0: &a0 [true]\n".to_owned();
    for index in 1..24 {
        bomb.push_str(&format!(
            "a{index}: &a{index} [*a{}, *a{}]\n",
            index - 1,
            index - 1
        ));
    }

    assert!(crate::parse_yaml(&bomb).is_err());
    let valid = format!("x: {}null{}", "[".repeat(62), "]".repeat(62));
    let invalid = format!("x: {}null{}", "[".repeat(63), "]".repeat(63));
    assert!(crate::parse_yaml(&valid).is_ok());
    assert!(crate::parse_yaml(&invalid).is_err());
    assert!(crate::parse_yaml(&"#".repeat(Metadata::MAX_BYTES + 1)).is_err());
}

#[cfg(feature = "files")]
#[test]
fn toml_rejects_dates_duplicates_and_non_finite_values() {
    for toml in [
        "x=1\nx=2",
        "x=2026-11-10",
        "x=nan",
        "x=inf",
        "x=[{y=1,y=2}]",
    ] {
        assert!(crate::parse_toml(toml).is_err(), "{toml}");
    }

    let valid = format!("x = {}true{}", "[".repeat(62), "]".repeat(62));
    let invalid = format!("x = {}true{}", "[".repeat(63), "]".repeat(63));
    assert!(crate::parse_toml(&valid).is_ok());
    assert!(crate::parse_toml(&invalid).is_err());
    assert!(crate::parse_toml(&"#".repeat(Metadata::MAX_BYTES + 1)).is_err());
}
