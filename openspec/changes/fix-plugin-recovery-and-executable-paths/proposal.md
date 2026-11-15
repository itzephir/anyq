# Proposal

## Why

Recovered shell autocomplete requests are lost when another managed action temporarily blocks completion, leaving stopped work holding its queue slots. Plugin executable canonicalization also bypasses symlink-based Python virtual environments and loses their dependencies.

## What Changes

- Retry recovered shell completion requests with their original receipt identity until completion is confirmed, without relaunching work.
- Retire live shell execution after a manual caller wins completion, sharing terminal-state reconciliation with recovered completion.
- Preserve the selected executable's absolute invocation path, including its final symbolic link, for explicit and PATH-based plugin registration.
- Cover both failures through real CLI, daemon and executable-plugin integration scenarios.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `shell-plugin`: retain and retry recovered autocomplete intent across a pending managed-work barrier.
- `plugin-runtime`: preserve executable invocation identity when resolving plugin commands.

## Impact

Changes affect CLI executable resolution, the shell protocol runtime, native integration fixtures, and plugin documentation. Queue domain rules, storage schema and protocol versions remain compatible. Required CI continues to run the integration scenarios on Linux, macOS and Windows.
