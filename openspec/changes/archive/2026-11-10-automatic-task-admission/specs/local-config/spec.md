## MODIFIED Requirements

### Requirement: Typed configuration keys
Configuration SHALL support task.autocreate and topic.autocreate with true/false and default false, task.autostart with true/false and default true, and optional db.path. Unknown keys, invalid types, and empty paths SHALL fail without changing settings. Validation and defaults SHALL belong to the domain.

#### Scenario: No config yet
- **WHEN** the settings file is absent
- **THEN** both autocreate settings are off, task.autostart is on, and the platform default database path is used

#### Scenario: Invalid value
- **WHEN** a non-boolean value is supplied for task.autocreate
- **THEN** the operation fails and preserves previous settings

#### Scenario: Table instead of terminal value
- **WHEN** task.autocreate, task.autostart, topic.autocreate, or db.path contains a table, including an empty one
- **THEN** reads and mutations fail without rewriting the original file

#### Scenario: Unknown empty section
- **WHEN** the file contains an unknown section, including list = {} or [list]
- **THEN** the domain schema rejects it without changing settings or the database

#### Scenario: Valid empty section
- **WHEN** the file contains an empty task, topic, or db container without terminal values
- **THEN** reads use defaults without changing the file

### Requirement: Transactional autocreate
With task.autocreate/topic.autocreate enabled, missing named entities SHALL be created inside the main operation; a new topic SHALL have capacity 1. Operation errors SHALL roll back its automatic creations. Autocreate SHALL NOT bypass valid transitions, ordering, or naming restrictions.

#### Scenario: Linking missing objects
- **WHEN** both autocreate settings are on and link references a missing task and topic
- **THEN** they are created with their link; the task is automatically admitted when autostart and resources allow, otherwise stays queued

#### Scenario: Completing an unstarted task
- **WHEN** task.autocreate is on and complete without force references a missing name
- **THEN** the command reports was not run and leaves no new task
