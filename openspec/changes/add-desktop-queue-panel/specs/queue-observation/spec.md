# Spec Delta

## Purpose

Provide desktop clients with a consistent view of current queue ownership and
membership through the local daemon, without performing queue mutations.

## ADDED Requirements

### Requirement: Owner-served observation
Current queue observation SHALL be available through the private local daemon interface for the selected database. Desktop clients SHALL NOT open SQLite directly, parse CLI text as their data interface, or depend on metadata, event hooks, or executable plugins to observe the queue.

#### Scenario: Observe without plugins
- **WHEN** a compatible daemon serves an existing database with no plugins enabled
- **THEN** the desktop client can obtain its current topics, tasks, and resource usage through the local interface

### Requirement: Consistent current snapshot
A successful observation SHALL describe one committed queue state and identify its canonical source database. It SHALL include all topics and all distinct queued/running tasks, with complete topic membership and registration order. Topic usage, task state, membership, and summary totals SHALL agree within the returned snapshot.

#### Scenario: Completion races observation
- **WHEN** one task completes and a successor starts while another client requests observation
- **THEN** the result describes the state before or after the complete transition without mixing the old owner with the successor's slot usage

#### Scenario: Canonical database identity
- **WHEN** a client reaches the same database through a symbolic-link alias
- **THEN** the returned source identity matches the daemon's canonical database identity

### Requirement: Current-state inclusion and ordering
Observation SHALL include empty topics and exclude registered/completed tasks from current queues and current task totals. Tasks in each topic SHALL appear in global registration order. A task's full linked-topic set, saved autostart default, and explicit stop status SHALL remain inspectable. Topic autostart overrides SHALL distinguish an explicit value from inheritance.

#### Scenario: Topic with historical and active tasks
- **WHEN** a topic has a completed task, a running task, and a later stopped queued task
- **THEN** its current queue includes only the running and queued tasks in registration order, with the stopped status retained

#### Scenario: Inherited autostart
- **WHEN** a topic has no autostart override
- **THEN** observation identifies inheritance rather than substituting one global boolean for every linked task

### Requirement: Resource and task counts
Occupied slots SHALL equal the number of running tasks linked to that topic. Queued-task counts SHALL count linked queued tasks. Global running/queued totals SHALL count distinct tasks once, irrespective of topic membership. Observation SHALL report persisted ownership rather than infer external process liveness, progress, or outcome.

#### Scenario: One owner of two resources
- **WHEN** one running task is linked to cpu and repo
- **THEN** both topics show one occupied slot and the global running total is one

#### Scenario: Queued task acquires nothing
- **WHEN** a task is queued for cpu and repo because repo is unavailable
- **THEN** it contributes to both queued counts and occupies no slot in either topic

### Requirement: Mutation-free observation
Observation SHALL NOT create entities, run admission, start or complete tasks, change capacity or start policy, or release ownership. Shared autocreate settings SHALL NOT grant observation permission to create anything. Searching, selecting absent identities, reconnecting, and cancelling a read SHALL preserve queue business state.

#### Scenario: Read with automatic creation configured
- **WHEN** autocreate is enabled and a refreshed selection refers to an identity absent from the current snapshot
- **THEN** the missing identity is reported or cleared without inserting a task or topic

#### Scenario: Cancel or disconnect
- **WHEN** a desktop observation is cancelled or its client disconnects
- **THEN** existing queue states and ownership remain intact and other daemon clients remain usable

### Requirement: Canonical identity preservation
Observation SHALL preserve stored task and topic identities, including empty names, Unicode, whitespace, and nested backticks. Receiving or displaying a snapshot SHALL NOT interpret canonical names as new command input, markup, or another round of quote normalization.

#### Scenario: Unusual canonical name
- **WHEN** an existing task name contains literal nested backticks and markup-like text
- **THEN** the client receives and displays that exact identity as text without changing it or executing content

### Requirement: Bounded complete responses
Observation SHALL obey daemon protocol message, request, and connection limits. A result that cannot fit the supported snapshot limit SHALL produce an explicit size error without a silently truncated successful snapshot. A slow or abandoned observer SHALL NOT indefinitely block other requests or owner shutdown.

#### Scenario: Snapshot exceeds the limit
- **WHEN** the current queue cannot fit within the supported response bound
- **THEN** observation reports a size error and the client does not replace its live view with a partial queue

#### Scenario: Client stops consuming
- **WHEN** an observer stops reading a response
- **THEN** unrelated queue requests and bounded daemon shutdown remain available

### Requirement: Compatible observation protocol
Clients and owners SHALL detect unsupported observation versions or operations before presenting a live snapshot. Incompatibility or owner unavailability SHALL be distinguishable from an empty queue. The desktop client SHALL NOT fall back to direct database access or reinterpret a failed response as zero tasks.

#### Scenario: Older owner
- **WHEN** a desktop client connects to an owner that does not support observation
- **THEN** the panel reports incompatibility with the selected source and does not show an empty live queue
