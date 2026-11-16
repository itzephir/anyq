# Design

## Context

See proposal.md for motivation and compatibility changes. Today every canonical queue has a daemon, its own plugin registrations and host, and the client starts any missing owner. Plugins already consume opaque task/topic metadata through bounded JSONL and perform managed work through durable queue capabilities. The desktop observes only one queue and stores view preferences separately. The implementation has no OS login registration.

## Goals / Non-Goals

**Goals:**
- Separate user installation/enablement from queue-local runtime progress.
- Preserve singleton ownership, all-topic admission, output bounds, replay receipts and uncertain-work barriers.
- Share actual OS autolaunch state across CLI and desktop without a second saved boolean.
- Exercise actual binaries, multiple real owners, system registration and native screenshots.

**Non-Goals:**
- No database discovery, remembered custom-database startup list, remote service or plugin-specific settings schema.
- No shared mutable plugin state across queue identities, automatic replay of external work, or implicit plugin trust.
- No GUI redesign or new queue mutation controls.

## Decisions

### Global catalog, queue-local hosts

Keep one host instance per active queue. A common binary is instantiated for each owner, preserving narrow capabilities and all existing per-queue receipts and timers. A single multiplexed process would require a new cross-database capability protocol and introduce routing/state confusion without a user requirement for one process.

Add a plugin file adapter and separate typed catalog list/enable/disable contracts and interactors. The catalog directory contains named executable files (`shell`, `timeout`, third-party names; `.exe` on Windows) and a hidden, versioned enablement record. File locking and same-directory atomic replacement serialize concurrent updates. Installed binaries begin disabled. The catalog stores framework lifecycle fields only, including per-plugin revisions; arbitrary plugin parameters remain in task/topic metadata and are not interpreted here.

Each host reconciles that catalog with queue-local registrations. Synchronize before exposing a newly initialized queue session and poll for subsequent global changes. Global enablement is durable immediately; per-owner readiness is observable through status and bounded reconciliation, not a cross-database transaction. First registration uses that queue's current watermark; restarting a materialized registration preserves cursor/state/timers. Repeating enable advances that plugin's revision to reset its bounded failure policy without resetting other plugins.

Persist the applied catalog revision in plugin storage through an additive versioned migration. Preserve existing execution history and receipts. Commands no longer accept arbitrary executable arguments at enable time. Legacy registrations absent from the catalog are not promoted; they retain history and remain disabled. Executable changes are reconciled through a stopped instance before replacement, with the normal cancellation barrier. Missing files and invalid catalog data must be visible without accepting new trust or discarding ownership.

`plugin list/enable/disable` operate on the common catalog without requiring or starting a selected custom queue. Runtime status/state/inspect/resolve remain explicitly scoped to the selected queue. Existing wire enable/disable operations cannot override the catalog authority.

### Independent first-party binaries

Build separate shell and timeout binary targets alongside the CLI, using their existing protocol/runtime/process crates. Only their small binary composition roots assemble implementations; shell retains a private anchor entrypoint in its own executable. Bundles and installers carry these binaries. Provision missing first-party files in `~/.anyq/plugins/` from packaged resources without overwriting a user-supplied installed executable or enabling it implicitly. Portable development builds resolve sibling built binaries.

### Startup is selected by canonical database identity

Resolve the current default using existing ANYQ_DB, shared db.path and platform precedence without an explicit CLI/panel override. Compare canonical identity, including equivalent paths, before choosing connection policy. Default queue/application startup can ensure its owner; all other ordinary connections and observations are existing-only. Explicit daemon start remains the custom-queue activation boundary, with the internal foreground owner entrypoint used by startup and native signal tests.

Do not infer startup permission from a saved path list or plugin enablement. Custom stop/crash remains stopped until explicit start. A default connection can recover the default owner; neither mode replays an in-flight operation. Default desktop startup ensures the default runtime independently of its observation-only selected custom source, without routing observation through a queue-initializing Open request. Existing-only observation and background validation continue to preserve legacy, malformed and missing storage.

### System autolaunch is its own feature

Use separate API contracts, dispatch ports, native system adapters and composition. CLI `autolaunch on/off` is handled independently of queue configuration parsing; bare autolaunch reports status. Desktop exposes an immediate system action independently of the preference draft and reflects errors rather than persisting a guessed boolean.

Register per-user login launch through a macOS LaunchAgent, Linux XDG autostart desktop entry, or Windows HKCU Run entry. The registered command uses a retained executable with `daemon start` and resolves the default at login; the GUI executable supports that non-GUI launcher mode as well. Registration does not snapshot custom paths. Disabling removes future login launch without stopping the detached owner or managed processes. Native integration tests use isolated profile locations/registry identities and execute the registered launch command against a real default owner.

References: [Apple launchd jobs](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPSystemStartup/Chapters/CreatingLaunchdJobs.html), [XDG autostart](https://specifications.freedesktop.org/autostart-spec/latest/), [Windows Run registration](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys).

### Tests and delivery

Adapt shared real-process fixture factories to the explicit-owner policy instead of adding starts throughout test bodies. Keep tests that intentionally exercise default auto-start distinct from custom existing-only tests. Add multi-database plugin and recovery scenarios, metadata pass-through, binary placement and installer checks, explicit custom lifecycle and system registration tests.

Extend desktop EN/RU settings with the existing styles and a native autolaunch scenario. Verify status/error/system effects outside preference persistence. Generate and review changed/new PNG goldens in separate generation mode on Linux, macOS and Windows; comparison remains read-only. Run all repository-mandated local checks and require current-head multi-OS integration and screenshot checks before claiming completion.

## Risks / Trade-offs

- Global enablement propagates asynchronously to existing hosts → expose queue-local readiness, synchronize before new sessions, and test concurrent catalog updates and live adoption.
- A shared name can exist in several queues → keep work/cursor/timer/receipt storage and authenticated capability routing per queue.
- Old queue-local commands may contain interpreter arguments → preserve history but require explicit installation/enablement under the new binary contract; do not broaden old trust silently.
- Windows can lock running executables → provision missing binaries without replacing active/user-owned files and keep updates explicit and packaged.
- External OS settings can change registration → query native state and report unsupported/malformed registration rather than treating anyq config as authoritative.
- Changing owner startup affects many integration fixtures → centralize fixture changes and retain real automatic/default and manual/custom acceptance tests on all platforms.

## Migration Plan

1. Add catalog and OS integration contracts/adapters and their boundary tests.
2. Ship standalone plugin binaries and package/provision them without enabling by default.
3. Add revision-aware catalog reconciliation, preserving queue-local progress and legacy history.
4. Apply default/existing-only startup consistently to CLI, observation and explicit lifecycle.
5. Add CLI/desktop autolaunch, native fixtures and reviewed goldens; update migration/installation documentation.
6. Validate the entire change before syncing baseline specs. A rollback must preserve databases, catalog state and OS registrations for explicit inspection rather than deleting user state.

The CLI ensures the current default transport before classifying ordinary queue requests. This establishes the existing owner sidecar identity for concurrent first clients using case aliases on case-insensitive filesystems. This Ping-only step does not create SQLite storage, and never starts a custom owner. Explicit lifecycle and config/catalog/system commands retain their own boundaries.
