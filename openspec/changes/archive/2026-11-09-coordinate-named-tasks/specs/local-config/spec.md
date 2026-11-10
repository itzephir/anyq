# Spec Delta

## Purpose

Provide local typed anyq settings with durable TOML storage, explicit database selection, and controlled automatic creation of tasks and topics.

## ADDED Requirements

### Requirement: Typed configuration keys
Configuration SHALL support task.autocreate/topic.autocreate true/false, default false, and optional db.path. Unknown keys, invalid types, and empty paths SHALL fail without changing settings. Validation/defaults SHALL belong to the domain.

#### Scenario: No config yet
- **WHEN** the settings file is absent
- **THEN** both autocreate settings are off and the platform database path is used

#### Scenario: Invalid value
- **WHEN** a non-boolean value is supplied for task.autocreate
- **THEN** the operation fails and preserves previous settings

#### Scenario: Table instead of terminal value
- **WHEN** task.autocreate, topic.autocreate, or db.path contains a table, including empty
- **THEN** reads and mutations fail without rewriting the original file

#### Scenario: Unknown empty section
- **WHEN** the file contains an unknown section, including list = {} or [list]
- **THEN** the domain schema rejects it without changing settings or the database

#### Scenario: Valid empty section
- **WHEN** the file contains an empty task, topic, or db container without terminal values
- **THEN** reads use defaults without changing the file

### Requirement: Git-style config CLI
The config command SHALL support KEY for reading, KEY VALUE for writing, --get KEY, --unset KEY, and equivalent list/--list/-l. List SHALL be a subcommand, not a valid key. Settings SHALL be stored in ~/.anyq/config.toml on Unix and Windows. Config and help SHALL NOT open the queue database.

#### Scenario: Write and read
- **WHEN** a new config task.autocreate process runs after config task.autocreate true
- **THEN** it shows true without opening the database

#### Scenario: Reset
- **WHEN** a configured key is reset through --unset
- **THEN** the default or absent path override is used

#### Scenario: Equivalent lists
- **WHEN** config list, config --list, and config -l are called
- **THEN** output matches and settings and database remain unchanged

#### Scenario: List is not a key
- **WHEN** config list VALUE, config --get list, or config --unset list is supplied
- **THEN** the command fails and preserves previous settings

### Requirement: Database path precedence
The database path SHALL be selected in the order --db, ANYQ_DB, db.path, platform default. The supplied path SHALL survive command-first/name-first normalization.

#### Scenario: Explicit path
- **WHEN** all four path sources are present
- **THEN** only the --db value is used

### Requirement: Atomic configuration persistence
Writes SHALL atomically replace the entire file and coordinate concurrent changes. Errors or Ctrl+C before replacement SHALL preserve the previous file; afterward they SHALL leave the complete new file. Interruption SHALL NOT leave a held lock.

#### Scenario: Two processes
- **WHEN** two processes change different keys concurrently
- **THEN** both successful changes are retained without a lost update

#### Scenario: Interrupted write
- **WHEN** a process receives real Ctrl+C before or after atomic replacement
- **THEN** a new process reads the complete old or complete new configuration and can write settings again

### Requirement: Transactional autocreate
With task.autocreate/topic.autocreate enabled, missing named entities SHALL be created inside the main operation; a new topic SHALL have capacity 1. Operation errors SHALL roll back its automatic creations. Autocreate SHALL NOT bypass valid transitions, ordering, or naming restrictions.

#### Scenario: Linking missing objects
- **WHEN** both autocreate settings are on and link references a missing task and topic
- **THEN** both are created with the link, the task is queued, and no slots are acquired

#### Scenario: Completing an unstarted task
- **WHEN** task.autocreate is on and complete without force references a missing name
- **THEN** the command reports was not run and leaves no new task
