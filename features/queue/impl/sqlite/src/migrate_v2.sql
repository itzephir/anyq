ALTER TABLE topics ADD COLUMN autostart INTEGER CHECK(autostart IN (0,1));
ALTER TABLE tasks ADD COLUMN autostart_default INTEGER NOT NULL DEFAULT 1 CHECK(autostart_default IN (0,1));
ALTER TABLE tasks ADD COLUMN stopped INTEGER NOT NULL DEFAULT 0 CHECK(stopped IN (0,1));
