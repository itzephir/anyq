# Spec Delta

## Purpose

Let the user control native operating-system login launch from either the CLI or desktop without storing an anyq preference that can disagree with the system.

## ADDED Requirements

### Requirement: System-backed login registration
`anyq autolaunch on/off` SHALL register or remove native per-user login launch on Linux, macOS and Windows without administrator access. Reading autolaunch status SHALL query that system registration. Neither operation SHALL use a config.toml or panel.toml autolaunch key.

#### Scenario: Enable and disable
- **WHEN** a user enables autolaunch, starts another client, and disables it
- **THEN** the clients observe the same OS-backed enabled state and the final registration is absent

#### Scenario: Invalid shared configuration
- **WHEN** anyq configuration is malformed but the user requests autolaunch off
- **THEN** the OS registration can still be removed without opening a queue

### Requirement: Default runtime login launch
An enabled login registration SHALL start the current default queue runtime and its globally enabled plugins after user login. It SHALL resolve the then-current default database and SHALL NOT retain or start custom database paths. Removing autolaunch SHALL affect future login launch without completing tasks or stopping active managed work.

#### Scenario: Default path changes
- **WHEN** the user changes db.path after registering login launch
- **THEN** the next login starts the new default owner without starting the old database

#### Scenario: Custom queue remains manual
- **WHEN** the computer restarts with autolaunch enabled and a previously used custom database
- **THEN** only the current default owner starts automatically and the custom database requires daemon start

#### Scenario: Disable while commands execute
- **WHEN** the user switches autolaunch off while shell tasks hold resources
- **THEN** the current tasks, processes and owners continue unchanged

### Requirement: Visible system integration errors
Registration failures SHALL be reported and SHALL NOT be presented as successful enabled state. Unsupported or externally changed registrations SHALL remain observable. Tests SHALL isolate OS registration and exercise the actual native registration boundary on every supported platform.

#### Scenario: Registration cannot be written
- **WHEN** the native registration target is unwritable
- **THEN** enabling fails visibly and the previous registration remains intact
