# Proposal

## Why

Optional automation must observe committed transitions even when no CLI is
waiting, without making plugin IO part of admission transactions. Stage 3 of the
[shared roadmap](../../../docs/PLUGIN-ROADMAP.md) depends on the daemon and metadata stages.

## What Changes

- Record lifecycle events for task registration, topic linking, running and
  completion, topic creation/settings changes and metadata changes.
- Save events and task/topic snapshots atomically with the originating mutation;
  deliver only after commit through durable per-plugin delivery records.
- Add explicitly enabled executable plugins, a versioned JSON-lines handshake,
  subscriptions, acknowledgements, bounded retries and observable failures.
- Give plugins narrow typed queue read/completion and persistent state/timer
  capabilities, including a managed-execution cancellation handshake.
- Preserve event IDs across retries and require explicit recovery for ambiguous
  non-idempotent actions; do not promise exactly-once script execution.
- Add plugin list/enable/disable/status commands. Timeout and shell follow later.

## Capabilities

### New Capabilities

- `queue-events`: committed lifecycle event identity, snapshots and ordering.
- `plugin-runtime`: executable plugin registration, delivery, requests and recovery.

### Modified Capabilities

None. Existing domain admission and ownership rules remain in force.

## Impact

Queue API/dispatch and SQLite gain explicit event/outbox contracts. A plugin
feature has separate API, dispatch, persistence/process adapters and composition.
The daemon hosts delivery and capabilities; CLI manages registration. Cross-feature
dependencies target API only, and plugins never receive a database handle.
Architecture checks must cover the new graph and process-recovery tests.
