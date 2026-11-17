# Spec Delta

## MODIFIED Requirements

### Requirement: One local owner per database
The system SHALL allow one daemon owner per canonical database path. Concurrent clients of the current default database SHALL discover or start that owner without competing owners. Custom database clients SHALL only discover an explicitly started owner. Distinct database paths SHALL remain independent. Queue CLI clients SHALL NOT open or mutate SQLite directly.

#### Scenario: Concurrent first commands
- **WHEN** several clients address the current default database before a daemon exists
- **THEN** one owner starts and every successful request uses it without duplicate registration or excess resource usage

#### Scenario: Equivalent paths
- **WHEN** clients use relative, absolute or symbolic-link paths to the same database
- **THEN** they resolve to the same owner and apply the same default-versus-custom startup policy

#### Scenario: Equivalent new database names
- **WHEN** a foreground owner exists before SQLite creation and a client uses an equivalent spelling on a case-insensitive filesystem
- **THEN** discovery uses the same owner, status remains read-only and the first queue operation succeeds without another owner

### Requirement: Automatic startup and explicit lifecycle
The current default queue SHALL maintain its owner automatically during queue use and application startup. A non-default queue SHALL require explicit `anyq --db PATH daemon start`; ordinary CLI requests, observation, plugin changes and login launch SHALL NOT start it. Explicit lifecycle SHALL retain foreground serving, non-starting status and stop without task completion. Ordinary client exit SHALL leave active owners alive.

#### Scenario: Client exits
- **WHEN** task registration finishes and its CLI exits
- **THEN** daemon status still reports running and subsequent commands reuse the owner

#### Scenario: Stop and restart
- **WHEN** the default owner stops and a later default queue command restarts it
- **THEN** task states, links, order and running ownership are preserved

#### Scenario: Custom owner is stopped
- **WHEN** a queue command addresses a non-default database without an owner
- **THEN** it reports the need for daemon start without creating or modifying that database

#### Scenario: Explicit custom start
- **WHEN** the user runs daemon start for a custom path and then sends queue commands
- **THEN** those commands reuse that owner and globally enabled plugins resume for that queue

#### Scenario: Foreground protocol stop
- **WHEN** a foreground daemon receives a protocol stop without a native interrupt
- **THEN** it exits successfully with code 0 while committed task ownership is preserved

## ADDED Requirements

### Requirement: No remembered custom database startup
Anyq SHALL NOT discover databases by filesystem scanning or maintain a previously used database list for automatic owner startup. Resolving the current default SHALL retain existing environment, shared configuration and platform precedence independently of explicit path overrides.

#### Scenario: Previously used custom database
- **WHEN** anyq restarts after a custom owner was previously used
- **THEN** that owner remains stopped unless the user explicitly starts it again

#### Scenario: Explicit alias of the default
- **WHEN** an explicit database path resolves to the same canonical identity as the current default
- **THEN** it uses the default owner policy rather than creating a second owner
