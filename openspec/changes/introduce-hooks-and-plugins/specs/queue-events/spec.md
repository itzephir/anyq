# Spec Delta

## Purpose

Expose durable committed queue lifecycle changes with stable identities and metadata snapshots for reliable background consumers.

## ADDED Requirements

### Requirement: Committed lifecycle events
The queue SHALL emit events for task registration, new topic links, running and completion, topic creation, capacity/autostart changes and metadata changes. An event SHALL be committed atomically with its originating mutation. Rolled-back and no-op mutations SHALL NOT emit transition events.

#### Scenario: Completion admits successors
- **WHEN** completing one owner admits two successors in the same transaction
- **THEN** the completion and both running events are saved together or none are saved

### Requirement: Stable ordered event identity
Each event SHALL have a durable database identity, monotonic sequence, version, kind, occurrence timestamp and canonical entity identity. Retries SHALL preserve the same identity and payload. Per-plugin delivery SHALL preserve event sequence; this SHALL NOT impose completion order on external work.

#### Scenario: Replay after failure
- **WHEN** a committed event is delivered again after an acknowledgement is lost
- **THEN** the plugin receives the same event ID and immutable payload

### Requirement: Running event snapshot
A task.running event SHALL include the admitted task, its metadata and each linked topic's metadata as separate inputs. Its timestamp SHALL be the persisted start instant used for deadlines. Later metadata or topic edits SHALL NOT rewrite the event. Running SHALL describe resource admission, independent of execution launch or outcome.

#### Scenario: Topic edited after admission
- **WHEN** a topic's timeout changes after a task obtains running ownership
- **THEN** redelivery of that task's running event still contains the original topic value and timestamp

### Requirement: Post-commit background delivery
Events SHALL become eligible for plugins only after commit and SHALL be observable without an active CLI wait. Plugin errors SHALL NOT roll back committed queue changes. Failed delivery SHALL remain inspectable and recoverable independently of queue availability.

#### Scenario: Plugin unavailable
- **WHEN** a task becomes running while its subscribed plugin process is unavailable
- **THEN** admission stays committed and delivery remains pending for recovery

### Requirement: Event recovery retains queue ownership
Daemon restart SHALL preserve committed events and plugin delivery positions without manufacturing new running transitions or releasing slots. Migration SHALL NOT create historical running events for existing tasks.

#### Scenario: Existing owner during upgrade
- **WHEN** event storage is introduced for a database containing running tasks
- **THEN** their ownership is preserved and plugins do not retroactively launch work for them
