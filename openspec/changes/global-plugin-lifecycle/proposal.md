# Proposal

## Why

Plugin installation and enablement currently belong to an individual queue and require executable arguments, including internal modes of the CLI for first-party plugins. Users need one catalog of independent plugin binaries, consistent enablement across running queues, explicit custom-queue lifecycle, and an optional operating-system login launch.

## What Changes

- **BREAKING**: Use `plugin enable NAME` and `plugin disable NAME` to control a user-wide catalog in `~/.anyq/plugins/`; plugins receive task/topic metadata through the existing public protocol and own its interpretation. No framework-level plugin configuration schema is introduced.
- Ship shell and timeout as independent native binaries installed in the same plugin directory as third-party binaries. Preserve explicit enablement and managed-work recovery.
- Keep delivery cursors, timers, receipts and managed work isolated per queue. Global enablement applies to every active owner and owners started later; it never discovers or starts other databases.
- **BREAKING**: Automatically maintain the current default queue owner; custom database requests and desktop observations only connect to an explicitly started owner. No remembered-database registry or filesystem scan is added.
- Add `anyq autolaunch on/off` and a localized desktop control backed by the operating system's per-user login registration, independently of `config.toml` and `panel.toml`.
- Restore globally enabled plugins when the default owner or an explicitly started custom owner restarts. OS login launch, when enabled, starts only the default queue runtime.

## Capabilities

### New Capabilities

- `plugin-catalog`: User-wide binary discovery, installation identity and persistent explicit enablement.
- `system-autolaunch`: Native per-user login registration shared by CLI and desktop.

### Modified Capabilities

- `plugin-runtime`: Global registration authority with queue-local delivery and recovery.
- `queue-daemon`: Automatic default ownership and explicit custom ownership.
- `desktop-queue-panel`: System autolaunch control and non-starting custom observation.
- `shell-plugin`: Standalone native binary distribution and startup.
- `timeout-plugin`: Standalone native binary distribution and startup.

## Impact

Plugin API/dispatch/file/composition and host adapters; daemon connection selection and startup; CLI syntax and first-party binary roots; desktop settings/presentation; native system integration; installers and release/desktop bundles; real CLI/plugin/lifecycle tests and reviewed screenshots on Linux, macOS and Windows. Existing queue data, receipts, metadata and managed-work invariants must remain intact. Legacy queue-local registrations retain history but do not authorize new user-wide execution implicitly.
