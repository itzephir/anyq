# Design

## Context

See proposal.md for motivation. Shell execution already has stable completion receipt IDs and periodic retry for live executions, but recovered stopped work has no in-memory completion tracking. CLI registration currently canonicalizes the full executable path before persisting it.

## Goals / Non-Goals

Goals are recovery of pending autocomplete and preservation of executable invocation identity. Queue ownership, managed cleanup, durable action IDs and existing command immutability remain authoritative. This change does not launch recovered work, change the wire protocol or migrate existing registrations.

## Decisions

- Track recovered completions separately from native executions. A small pending-completion type owns the action/task identity, retry deadline and response handling. Reusing an execution with a synthetic empty plan would conflate completion and launch permission. Retry through the public completion capability; recognize an already completed task through the public read capability when another caller wins completion.
- Resolve explicit paths and PATH candidates to absolute invocation paths without resolving the final executable link. Validate that the selected path points to a file. Canonicalizing the target would lose Python virtual environment semantics. Existing persisted commands remain unchanged; corrected registration applies to new registrations.
- Prove behavior at executable boundaries. Shell recovery tests use real CLI/daemon/plugin processes, a persisted stopped shell action, and a controllable managed worker. Python fixtures create their own symlink-based virtual environment and exercise explicit and PATH selection plus restart. Platform CI provides Python for this development-only test; runtime keeps no Python dependency.

## Risks / Trade-offs

- Recovered completion may remain blocked indefinitely by uncertain external work. Retry remains bounded in frequency and never bypasses the managed-work barrier.
- A competing caller may complete first. Check terminal task state rather than retrying a permanent conflict forever.
- Live and recovered completion share one protocol result handler: a completion conflict requests the task through the public read capability; only a confirmed completed task retires the pending execution. Other responses retain the existing stable-receipt retry behavior. A real live-plugin fixture pauses after confirmed exit, completes through a separate CLI, and observes execution retirement before map cleanup. Its test-only checkpoints select both boundaries explicitly, without an elapsed-time assertion or plugin restart.
- Environment fixtures use Unix interpreter links and Windows' native virtual environment launcher. Both explicit and PATH selection run on every supported platform without installing packages or depending on an activated ambient environment.
- Existing wrongly canonicalized registrations cannot change command in place. Document registering the intended interpreter under a new plugin name.

## Migration Plan

No storage or wire migration is required. Install the corrected binary after stopping the old daemon. Persisted stopped shell actions are recovered with their existing receipt IDs.
