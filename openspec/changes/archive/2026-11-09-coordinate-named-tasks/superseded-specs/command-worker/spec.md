# command-worker Specification

## Purpose

A local worker connects bash, builds, and emulators to the shared queue while retaining explicit command selection and the group's ownership protocol.

## Requirements

### Requirement: Explicit command and payload
Worker SHALL launch only explicitly specified commands with JSON payload on stdin and job identity in environment variables. Command stdout SHALL go to worker stderr, preserving CLI stdout for the JSON protocol.

#### Scenario: Payload is not interpreted
- **WHEN** payload contains shell text and worker launches a stdin reader
- **THEN** text is passed as data and is not executed by a shell

### Requirement: Execution and ownership renewal
Worker SHALL renew leases during commands, acknowledge exit 0, reject failures for retry, and terminate the launched process group on lease loss. Once mode SHALL process at most one job.

#### Scenario: Long build
- **WHEN** a command lasts longer than its initial lease
- **THEN** heartbeat preserves ownership until the command finishes

#### Scenario: Ownership lost
- **WHEN** the lease cannot be renewed
- **THEN** worker stops the command without acknowledging its result

### Requirement: Worker follow-up stage
Worker SHALL support an optional next topic with explicit JSON payload and publish it atomically after successful execution.

#### Scenario: Failed build
- **WHEN** the build command exits nonzero
- **THEN** no emulator test is published
