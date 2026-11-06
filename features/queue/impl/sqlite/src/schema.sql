CREATE TABLE topics (name TEXT PRIMARY KEY, capacity INTEGER NOT NULL CHECK(capacity > 0));
CREATE TABLE consumer_groups (
    topic TEXT NOT NULL REFERENCES topics(name), name TEXT NOT NULL,
    PRIMARY KEY(topic,name)
);
CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT, topic TEXT NOT NULL REFERENCES topics(name),
    payload TEXT NOT NULL, dedup_key TEXT, max_attempts INTEGER NOT NULL CHECK(max_attempts > 0),
    created_at INTEGER NOT NULL, UNIQUE(topic,dedup_key)
);
CREATE INDEX messages_topic_id ON messages(topic,id);
CREATE TABLE deliveries (
    message_id INTEGER NOT NULL REFERENCES messages(id), group_name TEXT NOT NULL,
    state TEXT NOT NULL CHECK(state IN ('pending','running','succeeded','failed')),
    attempts INTEGER NOT NULL CHECK(attempts >= 0), available_at INTEGER NOT NULL,
    consumer TEXT, token TEXT, lease_until INTEGER, finished_at INTEGER, result TEXT, last_error TEXT,
    PRIMARY KEY(message_id,group_name),
    CHECK((state='running' AND token IS NOT NULL AND lease_until IS NOT NULL) OR
          (state<>'running' AND token IS NULL AND lease_until IS NULL))
);
CREATE INDEX deliveries_ready ON deliveries(group_name,state,message_id,available_at);
CREATE INDEX deliveries_expired ON deliveries(state,lease_until);
CREATE UNIQUE INDEX deliveries_token ON deliveries(token) WHERE token IS NOT NULL;
