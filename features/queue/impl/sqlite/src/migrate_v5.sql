ALTER TABLE managed_work ADD COLUMN details TEXT NOT NULL DEFAULT '{"object":[]}';
PRAGMA user_version = 6;
