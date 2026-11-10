# Proposal

## Why

Tasks with an explicit timeout need predictable automatic completion even after
the creating CLI exits or the daemon restarts. Stage 4 of the [shared roadmap](../../../docs/PLUGIN-ROADMAP.md)
depends on durable events, plugin state and timers from `introduce-hooks-and-plugins`.

## What Changes

- Provide an optional first-party timeout plugin consuming `task.running` events
  and `timeout` metadata from the task and its topic snapshots.
- Resolve the shortest configured positive duration and persist one deadline
  anchored to the original committed running timestamp.
- Request completion at expiry, preserving ordinary queue validation, all-resource
  release and successor admission. Repeated delivery/completion is harmless.
- Recover overdue deadlines after restart and cancel timers for completed tasks.
- Use the common managed-execution cancellation contract before resource release;
  document that an unmanaged caller cannot be stopped by a queue timeout.

## Capabilities

### New Capabilities

- `timeout-plugin`: duration resolution, deadline persistence, expiry and recovery.

### Modified Capabilities

- `task-queue`: clarify explicit plugin completion as an opt-in ownership policy.

## Impact

A first-party plugin executable/mode and pure timeout policy are added using the
same protocol as other plugins. No timeout branches enter queue admission or
SQLite conversion. Delivery/timer receipts remain tied to event/task identity.
Documentation/examples and native end-to-end CI cover disabled operation,
concurrent manual completion, cancellation failure and restart recovery.
