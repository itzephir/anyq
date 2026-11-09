CREATE TABLE topics (name TEXT PRIMARY KEY, capacity INTEGER NOT NULL CHECK(capacity > 0));
CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    state TEXT NOT NULL CHECK(state IN ('registered','queued','running','completed'))
);
CREATE TABLE task_topics (
    task_id INTEGER NOT NULL REFERENCES tasks(id),
    topic_name TEXT NOT NULL REFERENCES topics(name),
    PRIMARY KEY(task_id, topic_name)
);
CREATE INDEX task_topics_by_topic ON task_topics(topic_name, task_id);
CREATE INDEX tasks_by_state ON tasks(state, id);
INSERT INTO topics VALUES ('cpu', 1), ('repo', 1);
INSERT INTO tasks VALUES (1, 'owner', 'running'), (2, 'next', 'queued'), (3, 'done', 'completed');
INSERT INTO task_topics VALUES (1, 'cpu'), (2, 'cpu'), (2, 'repo'), (3, 'repo');
PRAGMA user_version=2;
