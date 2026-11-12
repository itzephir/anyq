ALTER TABLE topics ADD COLUMN metadata TEXT NOT NULL DEFAULT '{}' CHECK(CASE WHEN json_valid(metadata) THEN json_type(metadata)='object' ELSE 0 END);
ALTER TABLE tasks ADD COLUMN metadata TEXT NOT NULL DEFAULT '{}' CHECK(CASE WHEN json_valid(metadata) THEN json_type(metadata)='object' ELSE 0 END);
