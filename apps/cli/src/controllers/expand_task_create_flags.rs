use std::ffi::OsString;

/// Move the value-taking -t to the end of task-create clusters such as -twc TOPIC.
pub fn expand_task_create_flags(arguments: impl IntoIterator<Item = OsString>) -> Vec<OsString> {
    enum Position {
        Root,
        Task,
        Create,
        Other,
    }

    let mut arguments = arguments.into_iter();
    let mut expanded: Vec<_> = arguments.next().into_iter().collect();
    let mut position = Position::Root;
    let mut values_expected: usize = 0;
    let mut positional_only = false;

    for argument in arguments {
        if values_expected != 0 || positional_only {
            values_expected = values_expected.saturating_sub(1);
            expanded.push(argument);
            continue;
        }

        if argument == "--" {
            positional_only = true;
        } else if argument == "--db" {
            values_expected = 1;
        } else {
            let text = argument.to_str().unwrap_or("");

            position = match position {
                Position::Root if text == "task" => Position::Task,
                Position::Task if text == "create" => Position::Create,
                Position::Root | Position::Task if !text.starts_with('-') => Position::Other,
                other => other,
            };

            if matches!(position, Position::Create) {
                if let Some(flags) = topic_cluster(text) {
                    expanded.push(format!("-{flags}").into());
                    expanded.push("-t".into());
                    values_expected = 1;
                    continue;
                }

                values_expected = match text {
                    "--meta" => 2,
                    "-t" | "--topic" | "--metafile" => 1,
                    text if text.starts_with("--meta=") => 1,
                    _ => 0,
                };
            }
        }

        expanded.push(argument);
    }

    expanded
}

fn topic_cluster(argument: &str) -> Option<String> {
    let flags = argument.strip_prefix('-')?;

    if flags.len() < 2
        || flags.bytes().filter(|flag| *flag == b't').count() != 1
        || !flags.bytes().all(|flag| b"twcs".contains(&flag))
    {
        return None;
    }

    Some(flags.chars().filter(|flag| *flag != 't').collect())
}
