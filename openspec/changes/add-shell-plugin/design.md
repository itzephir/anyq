# Design

## Context

See [proposal.md](proposal.md). The preceding stages provide daemon ownership,
metadata snapshots, durable deliveries/receipts and a managed-work completion
barrier already used by timeout. Running means resources were admitted, not that
a command exists. Existing caller-managed task workflows remain supported.

## Goals / Non-Goals

**Goals:** explicit execution plans, reproducible working directory, durable
outcomes and process-tree supervision integrated with the common barrier.

**Non-Goals:** a workflow DAG, task retries/reuse, secret management, a terminal UI,
running code from topic metadata, or executing arbitrary plugin payloads in the
queue domain. Completed is not redefined as successful.

## Decisions

### Pure plan validation and captured context

The shell API owns plan validation; its external adapter owns process IO. Accept
exactly one nonempty sh/src string array and an optional boolean autocomplete
(default false). Permit explicit cwd and interpreter configuration. Default cwd
is the creating CLI's absolute directory, persisted as task origin context before
admission; tasks configured with shell later still use that captured origin.
Newly migrated tasks without origin must supply explicit shell.cwd before start.
Task/topic metadata are separate; topic values never contribute commands.

For sh, use an explicitly recorded interpreter (default /bin/sh -c on Unix,
native cmd.exe /D /S /C on Windows). For src, pass the exact executable path and
resolve relative paths against cwd, without shell expansion or guessing file
extensions. Scripts require executable permissions/shebang or an explicit sh
invocation as appropriate. Inherit the daemon account's environment at launch;
support explicit string overrides only when supplied, and never persist the
entire environment or expose credentials in ordinary status.

### Sequential supervised execution

Execute one entry at a time, recording the plan, entry index and outcome. Stop
after first launch/nonzero failure; cancellation prevents subsequent entries.
Capture stdout/stderr concurrently into bounded logs so full pipes cannot block
supervision. Expose execution status/results through typed inspection operations.
Queue state remains running after successful or failed exit with autocomplete
false. With true, a final confirmed exit requests idempotent ordinary completion,
and an exit failure remains in execution history.

### Launch intent and uncertain recovery

Persist managed registration and launch intent before spawn. The executable
plugin keeps a supervised process group/job and records durable launch/exit
receipts through host capabilities. Duplicate running events consult receipts.
A crash between spawn and a receipt is not solved by re-running: mark uncertain,
retain resources and expose explicit resolve-after-cleanup behavior. PID alone
cannot establish identity after restart, so recovery must never kill an unrelated
reused PID or conclude an entire process tree stopped from the leader's absence.

### Native process-tree control

Use a reviewed process-group/job-object adapter, with Unix process groups and
Windows Job Objects created before work can spawn descendants. [Command-group's
native grouping API](https://docs.rs/command-group/latest/command_group/stdlib/trait.CommandGroup.html)
is an initial candidate; its leader wait alone is insufficient proof that every
managed descendant exited. The adapter must verify managed-tree termination and
return stopped only after that check. Platform-specific code stays outside inner
crates, and unsupported supervision fails before launch.

Timeout/manual completion/disable/shutdown all enter the common barrier. Request
graceful cancellation where supported, then bounded forced tree termination and
confirmed cleanup. If confirmation fails, report uncertain/pending, keep slots and
require explicit recovery. Force-completing a queued task still never launches it;
the existing force flag is not a shortcut around active managed-work cleanup.

## Risks / Trade-offs

- [Spawn/receipt crash gap] -> visible uncertainty, no automatic replay, retained
  ownership and explicit operator resolution.
- [Leader exits before descendants] -> group/job lifecycle checks and native
  tests with a descendant retaining a resource after its parent exits.
- [Shell syntax differs across OSes] -> explicit interpreter/defaults, native
  examples and no promise that Unix command text is Windows-compatible.
- [User code intentionally escapes supervision] -> managed execution documents
  its containment contract; unconfirmed cleanup never grants a stopped receipt.
- [Log flooding] -> bounded concurrent drains, truncation markers and inspection
  that does not depend on a live CLI reader.

## Migration Plan

Ship disabled and require explicit registration. Add task origin/execution state
storage atomically if it was not already needed by the metadata implementation;
never infer historical working directories or synthesize running events. Update
the absolute no-execution documentation only with this implemented opt-in path.
Run native descendant cancellation, timeout, disable, owner/plugin crash and
autocomplete tests before declaring the stage complete. Disable does not delete
history or permit an unconfirmed worker to release resources.

## Implemented boundaries

The shell API validates task-only plans; protocol runtime consumes injected native
process ports. The CLI root assembles the three shell crates. Schema 6 adds bounded
per-action metadata details through existing managed capabilities, inside the queue
upgrade transaction. Daemon protocol 5 and plugin protocol 2 carry those details.
Unix reserves group identity with a private live anchor outside the target group
and requires the kernel empty-group result; Windows uses a
suspended windows-spawn child, kill-on-close job, retained root handle and separate win32job member query.
Both pipe drains and native termination must be confirmed before stopped reports.
