# Proposal

## Why

Plugins and callers need durable structured task/topic context without adding a
queue field for every extension. Stage 2 of the [shared roadmap](../../../docs/PLUGIN-ROADMAP.md)
depends on `introduce-queue-daemon` and prepares data for lifecycle event snapshots.

## What Changes

- Add JSON-compatible object metadata to tasks and topics, defaulting to `{}`.
- Add name-first and command-first `meta` reads, dotted-key writes and unsets,
  plus `metafile FILE` to replace metadata from JSON, YAML or TOML.
- Add creation-time `--meta KEY VALUE` and `--metafile FILE` so metadata and initial
  topic links are saved before automatic admission in the same transaction.
- Keep task/topic metadata separate, allow unknown plugin namespaces, and validate
  structural/type bounds without interpreting timeout or shell settings.
- Freeze task metadata once running; topic changes affect future snapshots only.
- Upgrade the database atomically while preserving ownership and history.

## Capabilities

### New Capabilities

- `entity-metadata`: metadata value semantics, persistence, editing and file import.

### Modified Capabilities

- `task-queue`: metadata-aware atomic registration and shared command-name rules.

## Impact

Task/Topic aggregates, typed metadata use cases, SQLite schema/migration and daemon
protocol gain metadata. JSON/YAML/TOML parsing stays in outer adapters. Existing
configuration retains its typed known-key policy; metadata is not config storage.
UI text/help supports English and Russian. Parser dependencies are limited to
the adapters that need them; domain values remain independent of file formats.
