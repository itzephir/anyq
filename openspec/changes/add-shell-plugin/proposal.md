# Proposal

## Why

Callers should be able to opt into command/script execution when a task obtains
all its resources, with completion and timeout coordinated by the background
owner. Stage 5 of the [shared roadmap](../../../docs/PLUGIN-ROADMAP.md) depends on
all preceding stages and retains external-caller workflows by default.

## What Changes

- Add an explicitly enabled first-party shell plugin handling `shell` metadata
  after committed running events; accept exactly one nonempty `sh` or `src` array.
- Execute entries in order in an explicit captured working directory, stop on the
  first failure, and record launch/exit/cancellation outcomes and bounded logs.
- Support `autocomplete`, default false; true requests completion after final exit,
  including nonzero exit, while retaining execution outcome separately.
- Integrate process-tree cancellation with timeout and manual completion so managed
  work is confirmed stopped before resource release.
- Persist launch intent/receipts; show uncertain recovery states for explicit
  resolution instead of repeating scripts after ambiguous failures.
- **BREAKING (opt-in capability)**: revise the absolute no-command-execution rule
  to permit the shell plugin; queue domain/use cases still never spawn processes.

## Capabilities

### New Capabilities

- `shell-plugin`: metadata validation, managed execution, outcomes and cancellation.

### Modified Capabilities

- `task-queue`: optional plugin-managed work alongside caller-managed execution.
- `platform-support`: native process-tree supervision and optional interpreters.

## Impact

An external process adapter and pure shell-plan policy use the plugin protocol.
Native shell selection, source paths, cwd/environment, child output and process
tree control belong to this adapter. Unix and Windows use their native execution
facilities; core runtime never requires bash, Python, Node or WSL. Architecture,
interruption documentation, examples, packaging and native CI cover this behavior.
