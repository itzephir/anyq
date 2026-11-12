CREATE TABLE queue_identity (id INTEGER PRIMARY KEY CHECK(id=1), identity TEXT NOT NULL);
INSERT INTO queue_identity VALUES (1, lower(hex(randomblob(16))));
CREATE TABLE queue_events (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    payload TEXT NOT NULL CHECK(json_valid(payload))
);
CREATE TABLE task_directories (
    task_name TEXT PRIMARY KEY REFERENCES tasks(name),
    directory TEXT NOT NULL CHECK(json_valid(directory))
);

CREATE TABLE managed_work (
    owner TEXT NOT NULL,
    action_id TEXT NOT NULL,
    task_name TEXT NOT NULL REFERENCES tasks(name),
    state TEXT NOT NULL CHECK(state IN ('launch-pending','active','stopped','uncertain')),
    cancellation_requested INTEGER NOT NULL DEFAULT 0 CHECK(cancellation_requested IN (0,1)),
    PRIMARY KEY(owner,action_id)
);
CREATE INDEX managed_work_by_task ON managed_work(task_name,state);
CREATE TABLE completion_intents (task_name TEXT PRIMARY KEY REFERENCES tasks(name));
CREATE TABLE completion_receipts (
    owner TEXT NOT NULL,
    action_id TEXT NOT NULL,
    task_name TEXT NOT NULL,
    force INTEGER NOT NULL,
    result TEXT NOT NULL CHECK(json_valid(result)),
    PRIMARY KEY(owner,action_id)
);
