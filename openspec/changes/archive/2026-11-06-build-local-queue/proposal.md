# Proposal

## Why

Heavy local builds/tests compete for one machine's resources. Agents, people, and bash processes need a shared durable queue where finishing a build reliably schedules the next emulator stage.

## What Changes

- Rust `anyq` CLI: topics, consumer groups, JSON publication, manual claim/ack, and bash workers.
- Durable local queue with per-topic concurrency, leases, heartbeat, retries, and status history.
- Atomic acknowledgment plus publication of the next task to another topic.
- Domain features with separate Rust `api`/`impl` crates, internal ports, explicit DI, and CI architecture checks.
- OpenSpec and a private GitHub repository with Linux/macOS checks.

The user chose a separate copy of the topic stream for every consumer group; within a group one consumer executes a message. New groups read retained history from the beginning. Kafka wire protocol, network broker, web UI, and distributed coordination are outside v1.

## Capabilities

### New Capabilities

- `task-queue`: topics, durable jobs, leases, retries, statuses, and atomic follow-up handoff.
- `command-worker`: explicit local command execution with JSON input and lease renewal.
- `architecture`: domain feature boundaries, Dependency Rule, DI, and automated checks.

### Modified Capabilities

None: new project.

## Impact

New Cargo workspace, CLI, and local SQLite database. Commands run as the user; payload is data, and the worker owner explicitly selects commands. Node is used only for the pinned OpenSpec CLI.
