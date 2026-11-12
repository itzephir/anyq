# Design

## Context

See [proposal.md](proposal.md). The daemon and metadata stages are prerequisites.
Automatic admission is shared across registration/link/completion/topic changes;
running events must be recorded wherever admission occurs, not only explicit
start or wait. Queue dispatch owns transaction ports and SQLite implements IO.

## Goals / Non-Goals

**Goals:** durable lifecycle observation, isolated executable extensions, replay-safe
queue actions, persistent timers and a shared managed-execution completion barrier.

**Non-Goals:** dynamic Rust libraries, a package registry, remote plugins, a
message broker, exactly-once external effects or third-party code sandboxing.
Enabled executables run with the user's permissions and are explicitly trusted.

## Decisions

### Queue-owned events and transactional outbox

Define lifecycle event identity/kinds/snapshots and narrow event access contracts
in queue API. Dispatch supplies event records and an injected timestamp port
through the same UnitOfWork as task/topic changes. The domain reports actual
transitions; shared admission emits one running event per new transition. SQLite
serializes the supplied event without deciding when a task started. Persist the
occurrence timestamp once, even if later deliveries retry.

Use an append-only outbox with per-plugin delivery records, not event sourcing:
tasks/topics remain current-state authority. A failed transaction emits nothing.
Bootstrap migrations do not synthesize historical running events. Initial plugin
enable records the current high-water mark; restart resumes the existing cursor.
Do not delete events needed by a delivery or timer; retention beyond that is
bounded and configurable in the runtime, with conservative defaults.

### External executable protocol

Use `plugin enable NAME EXECUTABLE [ARG...]`, list/status/disable commands and a
versioned bounded JSON-lines protocol on stdio. The daemon starts and supervises
one instance per enabled registration. A handshake declares subscriptions and
required capabilities. Separate stdout protocol from bounded stderr diagnostics.
Apply response deadlines, exponential retry backoff and a visible failed state.
First-party timeout/shell expose executable modes in the anyq binary using this
same protocol; no privileged shortcut gives them direct SQLite access.

Each event delivery has a stable ID. A plugin replies with an acknowledgement and
namespaced state/timer updates applied atomically. Deliver in event order per
plugin while allowing asynchronous external work after acknowledgement. A crashed
consumer retries unacknowledged events; one failed plugin does not block others.

### Narrow capabilities and atomic receipts

Implement individual read, completion, state/timer and managed-execution capability
contracts. Wire envelopes route to these contracts only in outer adapters. For a
queue action, store the plugin/action ID, request digest and typed result in the
same transaction as its queue mutation. Matching repeats return the saved result;
conflicting reuse fails. A separate receipt commit after completion would leave
a crash gap and is rejected. Cross-feature dependencies target queue/plugin API;
only daemon/plugin composition imports implementations.

Persist timers with absolute deadlines and stable IDs in plugin storage. An
injected clock supplies instants to pure timer policy. The daemon scans eligible
timers with bounded work and wakes plugins; downtime delays action, not deadlines.
Disable pauses delivery/timers and retains state; re-enable resumes rather than
resetting. A first enable starts at the current event watermark.

### Managed-execution barrier before completion

Introduce typed registration, cancellation and stopped/uncertain reporting for
managed work. The daemon serializes launch registration and completion intent for
each task. Completion first prevents new launch, then requests cancellation of
active work; it invokes the domain completion only after all registered managed
work is confirmed stopped. This applies equally to manual, timeout and automatic
completion. Unmanaged tasks use today's direct completion path.

The barrier is runtime orchestration; the queue domain retains the final task
transition and validates any persisted completion permit. A plugin cannot assert
another plugin's stopped receipt. Unknown execution keeps slots held and is
inspectable. Explicit operator resolution can record confirmed external cleanup;
it must not silently relaunch work or treat a missing PID as proof of termination.
The runtime supports this contract with a test plugin before shell is introduced.

## Risks / Trade-offs

Background queue/plugin stores retry only failed lock acquisition before invoking
their transaction closure. Their wait is cancellable by owner shutdown; ordinary
client stores retain bounded lock waits. A busy live queue transaction therefore
delays background delivery instead of terminating the owner. Operations and
commits are never replayed. When shutdown ends a blocked background wait, daemon
composition treats the resulting interruption as graceful cancellation; committed
ownership and uncertain managed receipts remain intact.

- [Replay repeats external effects] -> stable IDs and receipts; uncertain
  non-idempotent actions require explicit recovery instead of automatic retry.
- [Plugin IO stalls queue work] -> separate bounded delivery/process supervision,
  post-commit scheduling and per-plugin failure isolation.
- [Completion races launch] -> persisted managed state, per-task sequencing and
  tests with real concurrent requests at both sides of the barrier.
- [Clock changes/downtime] -> absolute persisted deadlines and documented delayed
  recovery; wall-clock policy remains distinct from active-process duration.
- [Unbounded history/output] -> bounded frames/logs and retention pinned by
  unresolved deliveries, receipts and timers.

## Migration Plan

Upgrade outbox, delivery, receipt, timer and managed-work tables atomically while
preserving task ownership. Update architecture allowlists and negative fixtures
for each new crate. Prove delivery rollback, duplicate acknowledgement, request
deduplication and daemon/plugin crash recovery using fixture executables before
enabling first-party behavior. Document that downgrades require a pre-upgrade
backup and that disabling a plugin does not implicitly complete its tasks.

## Implemented boundaries and limits

The queue domain API remains free of serialization. A separate queue wire API is
shared by daemon, outbox and plugin representations. There are 25 workspace crates;
plugin API/wire/dispatch/SQLite/process/runtime/composition have separate ownership.
The daemon root injects plugin storage initialization into the queue SQLite schema-5
transaction. Versions 2/3/4 preserve states, metadata and ownership without events
for historical running tasks. Plugin progress, state and timers use a separate
consumer-owned transaction port; completion receipts and managed barriers remain
inside queue transactions because they protect resource ownership.

Protocol v1 limits are 16 MiB/frame, physical depth 256, 64 KiB stderr/instance,
16 enabled plugins, 128 timers/plugin, five-second response deadlines and a
three-second cancellation grace. Eight consecutive failures require explicit
enable after exponential backoff capped at 60 seconds. Runtime options expose
response/cancellation/polling and event-retention bounds; default retention is
10,000 recent events, with disabled cursors, timer origins and unresolved work
pinning needed history. Daemon IPC advances to version 3.

### Following shell stage

The stacked shell stage extends the shared migration to schema 6 with bounded
managed execution details and paged protocol-v2 history. Its verified schema
contract carries forward all schema-5 ownership and receipt scenarios in this
change's task-queue delta so later archival cannot replace the newer baseline
with an older migration contract. The hooks implementation PR itself remains
the schema-5 and protocol-v1 foundation.

### Observation integration: existing-only bootstrap

The shell-stage schema-6 runtime must recover independently of queue RPCs, so an
observation-first owner starts enabled plugin supervision immediately. Queue
SQLite exposes `open_existing_for_runtime`, returning `RuntimeOpen` with ready,
missing, uninitialized, upgrade-required or incompatible outcomes. The daemon
injects the plugin schema validator; the queue adapter validates queue/work
storage without importing plugin implementations. No initializer runs on this
path. Plugin store handles also open without `CREATE`.

SQLite can create WAL/SHM files even on read-only opening; immutable opening can
miss a committed crash-persisted WAL. Preflight therefore streams a private copy
of the main file plus WAL, never SHM, and validates it in a read snapshot. Only
complete current storage permits a real non-creating handle, whose own read
snapshot rechecks version and mandatory columns before configuration. This costs
temporary disk proportional to database plus WAL, uses bounded memory and checks
shutdown during copying. Direct external writers are outside the owner contract.

A cancellable factory initialization mutex serializes this one-time preflight
with ordinary queue opening. Non-ready storage leaves the owner available and
waits for successful normal initialization; it is not repeatedly copied or
migrated in the polling loop. Existing host recovery owns cursor/timer replay,
receipt reuse and conversion of unresolved managed work to uncertain state.

This follow-up is prepared on the shell-stage base for integration into the
observation stack. Its combined panel-first and three-platform CI acceptance
remains tracked separately; it does not amend verified baseline specs yet.

Local bootstrap acceptance passed on macOS: all mandatory formatting, Clippy,
workspace tests, architecture checks, Ruff and strict OpenSpec validation;
106 feature-enabled CLI/plugin workflows and all 35 native interruption tests.
Seven new queue adapter tests and two plugin storage tests cover preflight,
committed WAL, missing tables/columns, replacement revalidation and non-creating
handles. Five new native workflows exercise owner-only recovery and preservation.
Combined observation/panel behavior and Linux/Windows execution remain pending
integration CI; task 6.4 stays unchecked until that evidence exists.
