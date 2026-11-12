CREATE TABLE IF NOT EXISTS plugin_schema (id INTEGER PRIMARY KEY CHECK(id=1), version INTEGER NOT NULL);
INSERT OR IGNORE INTO plugin_schema VALUES (1,1);
CREATE TABLE IF NOT EXISTS plugins (
    name TEXT PRIMARY KEY,
    executable TEXT NOT NULL,
    arguments TEXT NOT NULL CHECK(json_valid(arguments)),
    enabled INTEGER NOT NULL CHECK(enabled IN (0,1)),
    running INTEGER NOT NULL CHECK(running IN (0,1)),
    cursor INTEGER NOT NULL CHECK(cursor>=0),
    failures INTEGER NOT NULL,
    retry_at INTEGER NOT NULL,
    last_failure TEXT,
    subscriptions TEXT NOT NULL CHECK(json_valid(subscriptions)),
    capabilities TEXT NOT NULL CHECK(json_valid(capabilities)),
    state TEXT NOT NULL CHECK(json_valid(state))
);
CREATE TABLE IF NOT EXISTS plugin_timers (
    plugin TEXT NOT NULL REFERENCES plugins(name),
    id TEXT NOT NULL,
    deadline INTEGER NOT NULL CHECK(deadline>=0),
    origin_sequence INTEGER NOT NULL,
    payload TEXT NOT NULL CHECK(json_valid(payload)),
    PRIMARY KEY(plugin,id)
);
CREATE INDEX IF NOT EXISTS plugin_timers_by_deadline ON plugin_timers(plugin,deadline,id);
