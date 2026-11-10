# Proposal

## Why

Currently free tasks stay queued until wait, and completing an owner does not transfer slots automatically. Admission should be a queue property; wait should observe persisted state.

## What Changes

- **BREAKING**: registration with topics and link automatically admit eligible tasks; running tasks cannot add topics, so specify the entire resource set at registration.
- Completion, force, and capacity increases advance the queue in the same transaction without a background process or active waits.
- Add atomic batch topic link TOPIC NAME... and task link NAME TOPIC...; save all links before admission.
- Wait only checks state: queued waits, running succeeds immediately, and registered/completed remain errors. Several waits for one name may successfully observe one admission.
- task.autostart (default true), nullable topic override, persistent --stop, --start/-s, task start with -w/-c, and compact create -tswc.
- Preserve global order, full atomic acquisition, and ownership until explicit complete.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `task-queue`: automatic admission on queue mutations and observing wait.
- `local-config`: typed task.autostart and new-task defaults.
- `architecture`: shared internal admission pass and a separate running-check contract replacing TryStartTask.

## Impact

Queue domain/dispatch, persistence filter/SQLite, composition/CLI; domain/use-case/concurrency/Ctrl+C tests; README, architecture, help, and interruption instructions. No new runtime dependencies or daemon; SQLite schema 2 upgrades atomically to 3 while preserving tasks/ownership.
