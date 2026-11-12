use super::expand_task_create_flags::expand_task_create_flags;
use std::ffi::OsString;

pub fn normalize_arguments(arguments: impl IntoIterator<Item = OsString>) -> Vec<OsString> {
    let mut arguments: Vec<_> = arguments.into_iter().collect();

    if let Some(resource) = next_word(&arguments, 1) {
        let kind = arguments[resource].to_str().unwrap_or("");
        let actions: &[&str] = match kind {
            "topic" => &[
                "list",
                "create",
                "capacity",
                "autostart",
                "link",
                "meta",
                "metafile",
                "help",
            ],
            "task" => &[
                "list", "create", "link", "start", "wait", "complete", "meta", "metafile", "help",
            ],
            _ => &[],
        };

        if !actions.is_empty() {
            if let Some(name) = next_word(&arguments, resource + 1) {
                let spelling = arguments[name].to_str().unwrap_or("");

                if !spelling.starts_with('-') && !actions.contains(&spelling) {
                    if let Some(action) = next_word(&arguments, name + 1) {
                        let spelling = arguments[action].to_str().unwrap_or("");

                        if actions.contains(&spelling) && !matches!(spelling, "list" | "help") {
                            arguments.swap(name, action);
                        }
                    }
                }
            }
        }
    }

    expand_task_create_flags(arguments)
}

fn next_word(arguments: &[OsString], mut position: usize) -> Option<usize> {
    while let Some(argument) = arguments.get(position) {
        if argument == "--db" {
            position += 2;
        } else if argument
            .to_str()
            .is_some_and(|value| value.starts_with("--db="))
        {
            position += 1;
        } else {
            return Some(position);
        }
    }

    None
}
