# Spec Delta

## MODIFIED Requirements

### Requirement: Supported schema upgrade
Versions 2/3/4/5 SHALL upgrade to schema 6 atomically, preserving names, registration, links, states, metadata and ownership. Queue events, receipts, managed work and plugin storage SHALL share one schema transaction, without admission or historical running events. Versions 2/3 SHALL receive empty metadata; version 2 SHALL receive task autostart=true/stopped=false with inheriting topics. Interruption SHALL leave the old schema before commit or the complete new schema afterward.


#### Scenario: Existing ownership
- **WHEN** a version 2 database with a running owner and queued successor is opened
- **THEN** after upgrade the owner holds the same slots, and the successor remains queued until the owner's explicit complete

#### Scenario: Existing schema 3
- **WHEN** a schema 3 database with running and queued tasks is upgraded
- **THEN** metadata defaults to empty objects and all states, order and ownership survive

#### Scenario: Metadata schema with active ownership
- **WHEN** a schema 4 database containing running tasks and metadata is upgraded
- **THEN** schema 6 preserves the metadata and all ownership without synthetic running events

#### Scenario: Interrupted feature migration
- **WHEN** initialization fails or is interrupted before schema commit
- **THEN** outbox, receipts, managed work and plugin storage are all rolled back with the queue schema

#### Scenario: Existing managed receipts
- **WHEN** schema 5 contains managed actions and active owners
- **THEN** schema 6 adds empty bounded execution details atomically while preserving action identities, states, ownership and completion receipts
