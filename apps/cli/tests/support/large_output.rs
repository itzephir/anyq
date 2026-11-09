use super::queue::Queue;
use rusqlite::{Connection, params};

pub fn seed_tasks(queue: &Queue) -> String {
    queue.call(&["task", "list"]);
    let mut connection = Connection::open(&queue.db).unwrap();
    let transaction = connection.transaction().unwrap();
    let suffix = "x".repeat(4096);
    let mut expected = String::new();

    for index in 0..2048 {
        let name = format!("large-{index:04}-{suffix}-🔥");
        transaction
            .execute(
                "INSERT INTO tasks(name,state) VALUES (?1,'registered')",
                params![name],
            )
            .unwrap();
        expected.push_str(&format!("- {name}\n"));
    }

    transaction.commit().unwrap();

    expected
}
