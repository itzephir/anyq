# Proposal

## Why

Inspecting current resource usage requires repeatedly running topic and task CLI
commands. A small desktop window and a system-panel entry would make the same
queue observable while other work continues.

## What Changes

- Add an optional, read-only desktop panel for macOS, Windows, and Linux, with
  topic search, current task queues, task details, slot usage, and live refresh.
- Provide a main window and a compact macOS menu-bar popover. Provide a Windows
  tray flyout and Linux StatusNotifierItem/AppIndicator integration, with an
  ordinary application window available when the desktop has no usable tray.
- Add panel-local database selection in settings. Resolve the panel override,
  `ANYQ_DB`, shared `db.path`, and platform default in that order; show the
  effective path and its source, and observe one database at a time.
- Add a narrowly typed, consistent queue observation snapshot through the daemon,
  keeping admission rules and persistence in their existing owners.
- Keep both desktop surfaces synchronized, identify stale data or connection
  errors, and discard responses from a previously selected database.
- Support English and Russian, light and dark themes, keyboard access, and small
  windows, adapting the approved personal-project design foundation to native
  desktop window and system-panel behavior.
- Keep queue management, external-work logs/progress, system resource monitoring,
  login autostart, and simultaneous multi-database dashboards outside this change.

### Dependency and delivery boundary

This change depends on
[`introduce-queue-daemon`](../introduce-queue-daemon/proposal.md). Implementation
starts after that change supplies a verified local owner, discovery, typed client
transport, cancellation, and recovery. The panel never falls back to direct
SQLite access. Metadata, hooks, timeout, and shell-plugin changes are not
prerequisites, and their delivery order is unchanged.

Implementation is stacked as daemon PR #14, observation PR #16, and desktop
PR #15. The daemon prerequisite is implemented and verified; desktop native
acceptance is tracked separately. Planning artifacts do not establish completed
behavior, and unfinished desktop capabilities are not synced to baseline specs.

## Capabilities

### New Capabilities

- `desktop-queue-panel`: desktop browsing, native system-panel access, panel
  lifecycle, local settings, database switching, and presentation states.
- `queue-observation`: consistent, bounded, read-only snapshots of current
  topics, task membership, queue order, and resource usage through the owner.

### Modified Capabilities

None. Existing CLI path selection, configuration keys, admission, task ownership,
and daemon guarantees remain intact; these capabilities add a desktop consumer
and its observation contract.

## Impact

Implementation adds an optional desktop application and panel settings,
extends queue API/dispatch/persistence ports with one observation use case, and
extends daemon protocol/client/runtime adapters with its typed operation. Desktop
composition, architecture checks, UI resources, packaging, installation guidance,
and native verification receive corresponding changes. GUI dependencies stay
outside inner crates and the ordinary CLI installation; the selected toolkit and platform
adapters are documented in [design.md](design.md) and docs/DESKTOP.md.
