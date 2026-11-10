# task-queue Specification

## Purpose

A durable local queue coordinates people, agents, and processes sharing machine resources through named topics and explicit job ownership.

## Requirements

### Requirement: Topics and durable jobs
The system SHALL create named topics with positive concurrency limits, publish JSON jobs, and persist them across restarts. A repeated idempotency key within a topic SHALL return the original job without republishing.

#### Scenario: Restart and deduplication
- **WHEN** a producer repeats publication with the same key after a new process opens the database
- **THEN** one job exists with the original payload and identity

### Requirement: Independent consumer groups
Each consumer group SHALL receive a separate delivery for every topic message. One group's acknowledgments/attempts SHALL NOT change another. A new group SHALL receive the complete retained stream from the beginning.

#### Scenario: Two groups
- **WHEN** build and audit groups read one message
- **THEN** both groups can acknowledge processing independently

#### Scenario: Late subscription
- **WHEN** a group is created after another group published and processed a message
- **THEN** the new group receives that message as pending

### Requirement: Competing consumers and ordering
The system SHALL issue ready jobs within a group in publication order to one consumer, respecting the topic-wide limit. Delayed jobs SHALL NOT block ready jobs. Different topics SHALL have independent limits.

#### Scenario: Shared resource
- **WHEN** two processes concurrently request the only slot of a busy topic
- **THEN** no additional job is issued until the slot is released

### Requirement: Lease and fencing
Claiming SHALL create a unique lease token and expiry for a message/group pair. Only a valid token SHALL allow renewal, acknowledgment, or rejection. Expired ownership SHALL requeue the group delivery or mark it failed when attempts are exhausted.

#### Scenario: Stale consumer
- **WHEN** a new consumer claims a job after lease expiry and the old consumer acknowledges its result
- **THEN** the old acknowledgment is rejected and new ownership is retained

### Requirement: Retries and observability
The system SHALL support bounded attempts, explicit delayed rejection, terminal failed state, and manual retry of failed jobs. CLI SHALL return JSON with status, attempts, consumer, lease, result, or last error.

#### Scenario: Attempts exhausted
- **WHEN** a consumer rejects a failed job on its final attempt
- **THEN** the job becomes failed and is not issued automatically

### Requirement: Atomic follow-up stage
A consumer SHALL atomically acknowledge a job with optional publication of the next JSON job to one topic. On follow-up errors, the original job SHALL remain unacknowledged.

#### Scenario: Build and emulator
- **WHEN** a consumer acknowledges large-builds with existing emulator-use as the next stage
- **THEN** the completed build and one emulator job are saved together

#### Scenario: Missing follow-up topic
- **WHEN** the next stage references a missing topic
- **THEN** acknowledgment is rejected and the current lease remains valid
