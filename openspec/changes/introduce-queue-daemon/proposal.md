# Proposal

## Why

Automatic admission can happen without a waiting CLI, while future plugins need
an owner that continues working after callers exit. This is stage 1 of the
[shared background queue and plugin roadmap](../../../docs/PLUGIN-ROADMAP.md).

## What Changes

- **BREAKING**: move queue database ownership from individual CLI processes to one
  background daemon per canonical database path; CLI sends local requests.
- Add automatic startup and explicit `daemon start`, `daemon run`, `daemon status`
  and `daemon stop`, with singleton discovery, compatibility checks and recovery.
- Preserve existing task/topic syntax, text, typed use cases, admission invariants,
  path precedence, chain commits and ownership after client interruption.
- Keep help/version/config independent of daemon/database access.
- Carry request cancellation to safe persistence boundaries and retain real native
  interruption coverage for both client and owner on all supported platforms.
- Establish the runtime foundation only; metadata, hooks and plugins follow later.

## Capabilities

### New Capabilities

- `queue-daemon`: local owner lifecycle, discovery, request protocol and recovery.

### Modified Capabilities

- `task-queue`: native coordination through the daemon and observation without a
  separate executor.
- `platform-support`: native daemon/client operation and interruption boundaries.

## Impact

CLI queue composition moves behind a daemon composition root. Local transport and
client adapters implement narrow queue API contracts; framing/routing stays in
external adapters. Queue domain and SQLite semantics remain authoritative.
Architecture inventory/checks, interruption fixtures, examples, distribution and
documentation must be updated. The single executable can serve CLI and daemon
modes, preserving installation without Python/Node or an extra server package.
