use anyq_queue_api::{ExecutionDirectory, Metadata, MetadataNumber, MetadataValue, Result};
use anyq_shell_api::{EntryOutcome, EntryOutput, PreparedPlan};
use std::collections::BTreeMap;

pub(crate) fn initial(plan: &PreparedPlan, event: &str) -> Result<Metadata> {
    let mut details = Metadata::default();
    details.set("status", MetadataValue::String("planned".into()))?;
    details.set("event_id", MetadataValue::String(event.into()))?;
    details.set("autocomplete", MetadataValue::Bool(plan.autocomplete))?;
    details.set("outcomes", MetadataValue::Array(Vec::new()))?;
    let commands = plan
        .commands
        .iter()
        .map(|command| {
            let mut record = BTreeMap::from([
                (
                    "program".into(),
                    MetadataValue::String(command.program.clone()),
                ),
                (
                    "arguments".into(),
                    MetadataValue::Array(
                        command
                            .arguments
                            .iter()
                            .cloned()
                            .map(MetadataValue::String)
                            .collect(),
                    ),
                ),
                (
                    "environment_keys".into(),
                    MetadataValue::Array(
                        command
                            .environment
                            .keys()
                            .cloned()
                            .map(MetadataValue::String)
                            .collect(),
                    ),
                ),
            ]);
            let (platform, units): (&str, Vec<u64>) = match &command.directory {
                ExecutionDirectory::Unix(bytes) => (
                    "unix",
                    bytes.iter().map(|value| u64::from(*value)).collect(),
                ),
                ExecutionDirectory::Windows(units) => (
                    "windows",
                    units.iter().map(|value| u64::from(*value)).collect(),
                ),
            };
            record.insert(
                "directory_platform".into(),
                MetadataValue::String(platform.into()),
            );
            record.insert(
                "directory_units".into(),
                MetadataValue::Array(
                    units
                        .into_iter()
                        .map(|number| {
                            MetadataValue::Number(
                                MetadataNumber::parse(&number.to_string()).expect("integer"),
                            )
                        })
                        .collect(),
                ),
            );
            if let Some(raw) = &command.raw_shell {
                record.insert("shell_text".into(), MetadataValue::String(raw.clone()));
            }
            MetadataValue::Object(record)
        })
        .collect();
    details.set("commands", MetadataValue::Array(commands))?;
    // Reserve output space before authorizing any native launch.
    let mut budget = details.clone();
    budget.set(
        "reserved_output",
        MetadataValue::String(" ".repeat(256 * 1024)),
    )?;

    Ok(details)
}

pub(crate) fn output(value: EntryOutput) -> MetadataValue {
    MetadataValue::Object(BTreeMap::from([
        ("stdout".into(), MetadataValue::String(value.stdout)),
        ("stderr".into(), MetadataValue::String(value.stderr)),
        (
            "stdout_truncated".into(),
            MetadataValue::Bool(value.stdout_truncated),
        ),
        (
            "stderr_truncated".into(),
            MetadataValue::Bool(value.stderr_truncated),
        ),
    ]))
}

pub(crate) fn outcome(details: &mut Metadata, index: usize, mut value: EntryOutcome) -> Result<()> {
    let mut outcomes = match details.get(Some("outcomes"))? {
        MetadataValue::Array(values) => values.clone(),
        _ => unreachable!(),
    };
    // One bounded aggregate output budget per action, regardless of entry count.
    let used = format!("{outcomes:?}").len();
    let budget = (128 * 1024_usize).saturating_sub(used) / 2;
    for (text, truncated) in [
        (&mut value.output.stdout, &mut value.output.stdout_truncated),
        (&mut value.output.stderr, &mut value.output.stderr_truncated),
    ] {
        if text.len() > budget {
            let mut end = budget;
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            text.truncate(end);
            *truncated = true;
        }
    }
    let mut record = BTreeMap::from([
        (
            "index".into(),
            MetadataValue::Number(MetadataNumber::parse(&index.to_string())?),
        ),
        ("success".into(), MetadataValue::Bool(value.success)),
        ("output".into(), output(value.output)),
    ]);
    record.insert(
        "code".into(),
        match value.code {
            Some(code) => MetadataValue::Number(MetadataNumber::parse(&code.to_string())?),
            None => MetadataValue::Null,
        },
    );
    outcomes.push(MetadataValue::Object(record));
    details.unset("live_output")?;
    details.set("outcomes", MetadataValue::Array(outcomes))
}
