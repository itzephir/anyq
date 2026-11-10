# Background queue and plugin roadmap

## Shared motivation

Tasks can become running when another operation releases resources, even when no
caller is waiting. Timeouts and scripts need a persistent owner that observes
these transitions after the CLI exits. Make anyq a background local queue with a
message-based CLI, attach structured task/topic metadata, and let optional plugins
react to committed events without moving admission rules into infrastructure.

The queue continues to own registration order, all-or-nothing admission and
resource ownership. A plugin requests a typed queue operation; it never edits
SQLite, allocates slots, or bypasses task transitions. Running means admission was
obtained; it does not mean that a script has already started or succeeded.

The CLI is English-only throughout all five stages, without translations or
language selection. Website language support remains a separate concern.

## Delivery order

| Stage | OpenSpec change | Prerequisite | Independently reviewable result |
| --- | --- | --- | --- |
| 1 | [introduce-queue-daemon](../openspec/changes/introduce-queue-daemon/proposal.md) | Existing queue | A persistent local owner and CLI request/response transport |
| 2 | [add-task-topic-metadata](../openspec/changes/add-task-topic-metadata/proposal.md) | Stage 1 | Durable metadata, CLI editing and JSON/YAML/TOML import |
| 3 | [introduce-hooks-and-plugins](../openspec/changes/introduce-hooks-and-plugins/proposal.md) | Stages 1–2 | Durable lifecycle events and an optional plugin host |
| 4 | [add-timeout-plugin](../openspec/changes/add-timeout-plugin/proposal.md) | Stages 1–3 | Recoverable deadlines and completion requests |
| 5 | [add-shell-plugin](../openspec/changes/add-shell-plugin/proposal.md) | Stages 1–4 | Managed commands/scripts and coordinated cancellation |

The planning PR contains all five proposals, delta specifications, designs, and
unchecked implementation tasks. Each implementation gets its own subsequent PR,
in the order above. Before prerequisites merge, PRs may be stacked against their
predecessor. Opening a PR does not authorize merging it. Completed behavior is
synced into baseline specifications only after verification; unfinished stages
remain changes. Changes are archived after their implementation is merged.

## Architectural direction

```text
CLI ---- local requests ----> daemon ---- typed queue use cases ----> SQLite
                                |
                                +---- committed event deliveries ----> plugins
                                ^                                         |
                                +------------- typed requests -------------+
```

One daemon owns each canonical database path. Ordinary CLI queue commands ensure
that owner is available, send bounded versioned messages, and preserve existing
command forms, output and exit codes. Help/version/config remain independent of
the database. Daemon start/status/stop are explicit lifecycle controls; automatic
startup preserves the existing one-command workflow. OS login services and
remote/network clients are outside this roadmap.

Task/topic metadata is an arbitrary JSON-compatible object. Plugins own the
meaning of their namespaces, such as `timeout` and `shell`. Topic and task metadata
are separate inputs; the queue does not impose a generic inheritance order across
several topics. Metadata can be supplied atomically during creation, before
automatic admission. Event snapshots prevent later edits from changing an
already scheduled action.

Plugins are explicitly enabled local executables with a versioned JSON-lines
protocol; no dynamic Rust library ABI or package marketplace is required. The
host delivers events after commit and persists delivery state. Delivery can be
repeated after failure; event IDs and durable plugin receipts support deduplication.
External side effects do not acquire an exactly-once guarantee. Ambiguous script
launches must be shown for explicit recovery rather than automatically repeated.

Timeout and shell are optional, first-party consumers of the same plugin contract.
Without an enabled plugin and matching metadata, current ownership and external
caller behavior remain unchanged. The baseline statements that anyq never runs
commands and needs no server are intentionally revised in their owning stages;
they must not silently constrain the newly authorized behavior.

## Proposed defaults

These choices make the proposals concrete and are reviewable before implementation:

- Daemons run locally under the invoking account, one per canonical database,
  and are discovered through a private endpoint. CLI disconnects do not stop them.
- File import replaces the complete metadata object; individual dotted-key writes
  patch one value. Imports are atomic, preserve unrelated entity state, and never
  execute content. Unknown plugin keys remain valid stored data.
- Task execution metadata can change before running; running/completed task
  metadata is immutable. Topic edits affect future running snapshots only.
- Timeout uses the smallest configured positive duration among the task and all
  its topics. Its deadline starts at the committed running timestamp. Missed
  deadlines are processed after recovery; no action is promised while stopped.
- Shell accepts exactly one of `sh` or `src`, executes entries in order, and stops
  on the first failure. `autocomplete` defaults to false; when true, normal final
  exit, including nonzero exit, requests completion with the outcome retained.
- Managed execution must confirm process-tree termination before timeout releases
  slots. Unmanaged external work remains the caller's responsibility; its opt-in
  timeout is a resource-release policy and cannot prove that work stopped.

## Verification boundaries

Every stage keeps pure domain/use-case tests and adapter/process tests separate.
Run the mandatory repository checks before commits. Native CI must cover Linux,
macOS and Windows, including actual Ctrl+C, daemon survival, rollback and recovery.
Do not turn native interruption tests into client-only mocks or skipped tests.
The shell stage additionally verifies cancellation of descendants on each OS.
No implementation task is complete merely because its planning artifact exists.
