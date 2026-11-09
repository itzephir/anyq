use crate::support::queue::Queue;
use std::fs;

#[test]
fn invalid_config_tables_reject_reads_and_updates_without_replacing_the_file() {
    let queue = Queue::new();
    let path = queue.dir.path().join(".anyq/config.toml");
    fs::create_dir_all(path.parent().unwrap()).unwrap();

    for invalid in [
        "task.autocreate = {}",
        "[task.autocreate]",
        "topic.autocreate = {}",
        "[topic.autocreate]",
        "db.path = {}",
        "[db.path]",
        "list = {}",
        "[list]",
        "unknown = {}",
        "[unknown]",
        "task.unknown = {}",
        "[task.unknown]",
        "task = { autocreate = {} }",
        "task.autocreate = true\nlist = {}",
    ] {
        fs::write(&path, invalid).unwrap();
        let before = fs::read(&path).unwrap();

        for command in [
            vec!["config", "list"],
            vec!["config", "--list"],
            vec!["config", "-l"],
            vec!["config", "--get", "task.autocreate"],
            vec!["config", "topic.autocreate", "true"],
            vec!["config", "--unset", "task.autocreate"],
            vec!["task", "list"],
            vec!["topic", "list"],
        ] {
            let output = queue.raw(&command);

            assert_eq!(output.status.code(), Some(2), "{invalid}: {command:?}");
            assert!(!output.stderr.is_empty());
            assert_eq!(fs::read(&path).unwrap(), before, "{invalid}: {command:?}");
            assert!(!queue.db.exists());
        }

        fs::write(&path, "").unwrap();
        queue.call(&["config", "task.autocreate", "true"]);

        assert_eq!(queue.call(&["config", "task.autocreate"]), "true");
    }
}

#[test]
fn valid_empty_config_sections_keep_defaults_and_accept_later_updates() {
    let queue = Queue::new();
    let path = queue.dir.path().join(".anyq/config.toml");
    fs::create_dir_all(path.parent().unwrap()).unwrap();

    for valid in ["[task]\n[topic]\n[db]", "task = {}\ntopic = {}\ndb = {}"] {
        fs::write(&path, valid).unwrap();

        assert_eq!(
            queue.call(&["config", "list"]),
            "task.autocreate=false\ntopic.autocreate=false"
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), valid);

        queue.call(&["config", "topic.autocreate", "true"]);

        assert_eq!(queue.call(&["config", "--get", "topic.autocreate"]), "true");
        assert!(!queue.db.exists());
    }
}
