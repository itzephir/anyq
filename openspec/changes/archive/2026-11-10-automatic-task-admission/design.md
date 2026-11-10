# Design

## Context

See proposal.md. TryStartTask currently reads resources and changes state in an IMMEDIATE transaction; other operations do not advance the queue. Slots derive from running tasks. The SQL filter requires a specific topic.

## Goals / Non-Goals

Keep the original operation and resulting admissions atomic while retaining an IO-independent domain. Do not add a daemon, timers, or command execution; do not restart completed tasks.

## Decisions

- A shared internal dispatch function selects queued tasks using an explicit topic-free filter, sorts by sequence, and passes each task's complete current resource snapshot to the domain. After every admission, the next task sees persisted running. One pass suffices: only earlier tasks are admitted, and unavailable predecessors retain their place. SQL scheduling would violate domain boundaries.
- RegisterTask, AddTaskToTopic, CompleteTask, and SetTopicCapacity invoke the pass before commit. Returned Tasks are reread after admission. Any write error rolls back the original operation and already admitted successors.
- CheckTaskRunning replaces public TryStartTask. The domain returns true for running, false for queued, and previous errors for registered/completed. Checks run inside a transaction so failed waits roll back autocreate. CLI only retries with a pause.
- TaskFilter.topic becomes optional: None means all topics, Some retains the old restriction. SQLite uses EXISTS to avoid duplicate multi-topic tasks. This is a technical filter, not a business decision.
- Separate typed AddTaskToTopics and AddTasksToTopic use cases share an internal linking helper. The whole batch is validated/saved in one transaction, followed by one admission pass. Repeated names are deduplicated; task batches return in sequence order. topic link -w/-c processes tasks in registration order, completing after each successful wait. Any participant's error rolls back the batch and autocreate.
- QueuePolicy supplies autocreate and default task.autostart from config. Task stores the default on creation; Config still does not open the database or change existing tasks. Topic.autostart overrides the default; all topics must allow admission. Stopped is stored separately and is not reset by capacity/topic changes.
- StartPreference defines default/start/stop/toggle. The domain treats --start as a no-op with effective autostart=true; otherwise it requires immediate full admission and rolls back registration on Busy. -s inverts the effective value across the complete resource set.
- StartTask returns Started/AlreadyRunning/Busy; failed attempts leave the task unchanged. CLI without -w turns Busy into an error; with -w it retries. -c requires -w and performs separate completion after success. Wait remains an observer even for stopped tasks.
- Running topic membership stays immutable. For several resources, prefer atomic registration with all -t flags or task link with the complete set; sequential links remain valid while queued.

## Risks / Trade-offs

- Automatic start after the first link → explicitly document specifying all resources at registration.
- Previously queued tasks may have depended on wait → the next successful mutation reconsiders all queued tasks; reads/wait intentionally do not start them. Opening does not rewrite the schema or old data.
- The pass covers the entire queued set → sufficiently simple for a local tool; add targeted optimization only for measured need without changing the contract.
- Completion may be interrupted between several task writes → tests verify full rollback before commit and complete persistence afterward.

## Migration Plan

Schema becomes version 3; version 2 upgrades transactionally by adding task.autostart_default/stopped and nullable topic.autostart. Version 1 and unknown versions are rejected. Update CLI/internal Rust contracts and documentation. Run required checks before committing and standard three-OS PR CI. Do not merge automatically or sync baseline specs before behavior verification.
