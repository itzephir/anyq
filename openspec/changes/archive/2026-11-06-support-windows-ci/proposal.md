# Proposal

## Why

CI already checks macOS/Linux, but the Unix-dependent worker adapter does not build on Windows. Native Windows builds/tests must preserve process-tree management and an OS-independent domain.

## What Changes

- Add Windows to the required CI matrix alongside macOS/Linux.
- Implement Windows process-tree launch/termination and console interruption adapter.
- Make end-to-end tests/demo portable and verify descendant termination on Windows.
- Select a default Windows database location and document native commands.

## Capabilities

### New Capabilities

- `platform-support`: native CLI/worker, local database selection, and macOS/Windows/Linux CI.

### Modified Capabilities

No baseline specs: initial queue requirements remain in the completed, unarchived build-local-queue change. New platform guarantees supplement it without changing queue semantics.

## Impact

Changes are limited to the external process adapter, CLI database-path controller, tests, CI, and documentation. Message/group/lease models and SQLite schema remain unchanged.
