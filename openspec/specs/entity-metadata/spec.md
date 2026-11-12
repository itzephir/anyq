# entity-metadata Specification

## Purpose

Attach durable structured context to tasks and topics so callers and optional plugins can describe behavior without changing resource admission rules.

## Requirements

### Requirement: JSON-compatible object metadata
Every task and topic SHALL have metadata defaulting to an empty object. Objects, arrays, strings, finite numbers, booleans and null SHALL be supported. Unknown keys SHALL be preserved without requiring an installed plugin. Non-object roots, duplicate keys, unsupported values and configured size/depth limits SHALL fail before mutation.

#### Scenario: Arbitrary extension data
- **WHEN** metadata contains an unknown plugin namespace with nested arrays and booleans
- **THEN** a later process reads the same values without executing or interpreting them

### Requirement: Metadata read and key mutation
`task NAME meta` and `topic TOPIC meta` SHALL print the object as JSON. A dotted key SHALL read its value, KEY VALUE SHALL replace that value, and `--unset KEY` SHALL remove it. Command-first forms SHALL be equivalent. Non-JSON scalar input SHALL be a string; valid JSON input SHALL retain its type. Missing reads and invalid traversal SHALL fail without mutation.

#### Scenario: Timeout string and boolean
- **WHEN** a caller sets timeout to `2h` and shell.autocomplete to `true`
- **THEN** metadata stores a string duration and a boolean respectively, preserving unrelated keys

#### Scenario: Concurrent independent patches
- **WHEN** clients set different metadata keys concurrently
- **THEN** both successful patches remain stored without a lost update

### Requirement: Atomic file replacement
`task NAME metafile FILE` and the equivalent topic command SHALL parse JSON, YAML or TOML selected by the file extension and replace the complete metadata object atomically. File reads SHALL occur on the CLI's machine before sending data. YAML custom tags, non-string keys and alias expansion beyond limits SHALL be rejected. File import SHALL NOT execute content.

#### Scenario: Broken replacement
- **WHEN** an import contains invalid syntax, duplicate keys, unsupported types or a non-object root
- **THEN** the command fails and the previous metadata and queue state are preserved

### Requirement: Creation-time metadata
Task/topic creation SHALL accept one `--metafile FILE` and repeated `--meta KEY VALUE` patches. The file object SHALL be loaded first and patches applied in argument order. Metadata, registration and all initial links SHALL commit together before automatic admission. Failure SHALL leave no partial entity, links or metadata.

#### Scenario: Immediate admission with metadata
- **WHEN** a task is created with free topics and timeout metadata
- **THEN** its first running state already includes that metadata in the same commit

### Requirement: Execution snapshot stability
Task metadata SHALL be editable only before running. Completed and running task mutations SHALL fail without changes. Topic metadata SHALL remain editable; edits SHALL NOT modify an already captured running task's metadata inputs. Generic inheritance or merging across task/topic metadata SHALL NOT occur.

#### Scenario: Running task edit
- **WHEN** a caller attempts to change a running task's timeout
- **THEN** the edit is rejected and the original execution inputs remain unchanged

### Requirement: Queue-independent metadata semantics
Metadata operations SHALL use the existing canonical entity identity and autocreate policy. Failed operations SHALL roll back automatic creations. Metadata SHALL NOT allocate resources, alter task membership, or interpret plugin-specific rules. Metadata keys SHALL NOT be restricted to the typed configuration key inventory.

#### Scenario: Invalid patch with autocreate
- **WHEN** autocreate is enabled but a missing entity's metadata patch is invalid
- **THEN** neither an entity nor a partial metadata object is saved

### Requirement: Ownership-preserving migration
Existing compatible databases SHALL upgrade metadata storage atomically with empty metadata for existing entities. Names, registration order, links, states and ownership SHALL survive. Incompatible schemas SHALL be rejected before modifying data or journal mode.

#### Scenario: Upgrade with owner
- **WHEN** a database with a running task and queued successor is upgraded
- **THEN** both have empty metadata, the owner retains every slot, and the successor remains queued
