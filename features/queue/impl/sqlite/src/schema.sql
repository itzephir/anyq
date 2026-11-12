CREATE TABLE topics (
    name TEXT PRIMARY KEY,
    capacity INTEGER NOT NULL CHECK(capacity > 0),
    autostart INTEGER CHECK(autostart IN (0,1)),
    metadata TEXT NOT NULL DEFAULT '{}' CHECK(CASE WHEN json_valid(metadata) THEN json_type(metadata)='object' ELSE 0 END)
);
CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    state TEXT NOT NULL CHECK(state IN ('registered','queued','running','completed')),
    autostart_default INTEGER NOT NULL DEFAULT 1 CHECK(autostart_default IN (0,1)),
    stopped INTEGER NOT NULL DEFAULT 0 CHECK(stopped IN (0,1)),
    metadata TEXT NOT NULL DEFAULT '{}' CHECK(CASE WHEN json_valid(metadata) THEN json_type(metadata)='object' ELSE 0 END)
);
CREATE TABLE task_topics (
    task_id INTEGER NOT NULL REFERENCES tasks(id),
    topic_name TEXT NOT NULL REFERENCES topics(name),
    PRIMARY KEY(task_id, topic_name)
);
CREATE INDEX task_topics_by_topic ON task_topics(topic_name, task_id);
CREATE INDEX tasks_by_state ON tasks(state, id);
