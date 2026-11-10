# Proposal

## Why

The current Queue::execute(Command) -> Output combines unrelated use cases, requires runtime result matching, and makes worker depend on the entire queue API. Multiple types per file and a shared Dispatch violate the user's expected SRP, ISP, and one-standalone-type-per-file rule.

## What Changes

- Remove generic Queue/Command/Output and monolithic Dispatch.
- Extract minimal typed contracts and interactors per use case; worker receives only required ports.
- Split models, ports, SQL adapters, CLI arguments, and test doubles by responsibility and file.
- Enforce rules in AGENTS.md and architecture checks with negative fixtures.

## Capabilities

### New Capabilities

None: internal Rust API refactoring without CLI or queue-semantics changes.

### Modified Capabilities

None: observable behavior is preserved. The change declares skip_specs because it changes implementation organization rather than product requirements.

## Impact

BREAKING for internal Rust API 0.1; CLI commands, SQLite schema, and data are preserved. The existing support-windows-ci change continues on the new structure.
