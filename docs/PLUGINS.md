# Local executable plugins

Plugins are optional trusted programs supervised by the queue daemon. Metadata
alone never enables an executable. CLI help, errors and plugin management output
are English-only. Timeout and shell are separate implementation stages.

```sh
anyq plugin enable observer python3 /absolute/path/to/examples/observer.py
anyq plugin list
anyq plugin status observer
anyq plugin disable observer
anyq plugin inspect observer
anyq plugin resolve observer ACTION_ID --confirmed-cleanup
```

The CLI resolves the executable locally to an absolute path. Arguments are passed
as separate arguments; use absolute paths for script arguments. An enabled
program has the invoking user's permissions. The host supplies neither database
handles nor database paths through its protocol. It is not a security sandbox.

First enable captures the current event watermark: existing running tasks do not
produce synthetic starts. Disable pauses events and timers, preserving state,
receipts, cursor and deadlines. Re-enable the same name and command to resume.
Changing a registered command requires another name. At most 16 plugins may be
enabled. Each has its own ordered delivery position and supervised process. Plugins run
in a separate process group (and detached console on Windows), so terminal
Ctrl+C reaches the daemon and cancellation reaches plugins through the protocol.

Status reports enabled/running state, event lag, active timer and unresolved
managed-action counts, consecutive failures and a bounded failure description.
Ordinary status does not include metadata, plugin state, stderr or script output.
`inspect` lists managed action identities and states for explicit recovery.

## Owner startup and existing databases

Starting the daemon resumes enabled plugins from a complete schema-6 queue,
including saved deliveries and overdue timers, without a queue command. This
also applies when an observation client starts the owner. First enable still
uses the future-event watermark; startup never synthesizes historical starts.
Disabled registrations retain their state and deadlines without launching.
Completion receipts remain reusable, and uncertain managed work keeps its slots
until explicit confirmed cleanup. Restart does not authorize relaunching it.

The existing-only runtime opener distinguishes these storage outcomes:

| Outcome | Background behavior |
| --- | --- |
| Ready | Resume persisted plugin supervision and delivery. |
| Missing | Keep the owner available; do not create the database or its parent. |
| Uninitialized | Leave empty or unversioned storage unchanged. |
| Upgrade required | Leave schema 2/3/4/5 unchanged until a normal queue request upgrades it. |
| Incompatible | Leave unsupported, malformed or incomplete storage unchanged. |

Non-ready outcomes do not stop lifecycle or observation sessions. A successful
normal queue request can initialize or upgrade storage and release the waiting
background runtime. Startup probes once, rather than copying or retrying an
incompatible database on every polling interval. Replacing or repairing storage
externally requires an owner restart or an ordinary queue request; observation
does not trigger initialization or upgrades.

Validation checks every required queue, work and plugin table and column before
runtime configuration or recovery. It uses a private temporary copy of the main
database and any committed WAL: SQLite read-only opening can create auxiliary
files, while immutable opening can omit committed WAL data. No SHM file is copied.
The real handle opens without `CREATE` and validates again in one read snapshot
before changing connection settings or journal mode. Daemon composition serializes
this with normal initialization. Copying uses bounded memory and cancellable IO,
but requires temporary disk space proportional to the database plus WAL. The
single-owner contract excludes concurrent direct writers or external replacement
while the runtime is active.

## Events

Queue schema 5 adds a persistent random database identity, monotonically ordered
outbox, task caller directories, completion receipts and managed-work records.
Schemas 2/3/4 upgrade without admitting queued tasks or creating historical events.
The daemon injects plugin storage initialization into that same schema transaction;
a failure or interruption cannot leave a partial feature migration.

Version 1 events have a database identity, sequence, kind and Unix millisecond
occurrence timestamp. The stable event identity is `DATABASE_ID:SEQUENCE`.
Registration, new links, running, completion, topic creation, capacity/autostart
changes and metadata changes produce events next to their successful domain
mutation. Failed transactions and no-op changes publish none. A completion and
its automatically admitted successors commit together in that order.

Task events contain a task snapshot and separate snapshots of its linked topics.
A `task.running` timestamp is the committed start instant, independent of plugin
launch and exit. Later edits cannot rewrite the snapshot. The native caller
working directory is saved at registration and reused for later running events.

Events become visible after commit without an active CLI wait. Delivery failures
leave admission committed. The default runtime retains 10,000 recent events;
`HostOptions.retained_events` configures this window. Consumer positions, timer
origins and unresolved managed work pin needed history, including while disabled.
Completed queue action receipts retain their results independently of event
retention. This outbox is not an event-sourced replacement for tasks and topics.

## Protocol version 1

Each stdio line is one UTF-8 JSON object. Stdout is protocol-only; stderr is bounded
diagnostics and is never interpreted as a frame. Both peers use the public types
in `anyq-plugin-wire`, also usable by first-party implementations. Numbers in
metadata are textual decimal literals to preserve precision:

```json
{"object":[["timeout",{"string":"2h"}],["enabled",{"bool":true}]]}
```

The metadata variants are `"null"`, `{"bool":true}`, `{"number":"1.25"}`,
`{"string":"text"}`, `{"array":[...]}` and `{"object":[["key",VALUE],...]}`.
Object pairs allow duplicate-key detection. Ordinary CLI metadata output uses
natural JSON; the typed envelope belongs only to transport.

The host begins with:

```json
{"type":"hello","version":1,"plugin":"observer","max_frame_bytes":16777216,"state":{"object":[]}}
```

The plugin responds once with its subscriptions and required capabilities:

```json
{"type":"hello","version":1,"subscriptions":["task.running"],"capabilities":["read","complete","state","timers","managed"]}
```

Available subscriptions are `task.registered`, `task.linked`, `task.running`,
`task.completed`, `task.metadata-changed`, `topic.created`,
`topic.capacity-changed`, `topic.autostart-changed` and `topic.metadata-changed`.
Unknown versions, subscriptions or capabilities fail the handshake. Frames have
a 16 MiB byte limit and a 256-level physical nesting limit. The default handshake
and event acknowledgement deadline is five seconds. Protocol queues and stderr
are bounded; over 64 KiB of stderr per instance fails that process. Raw stderr is
not copied into ordinary diagnostics. Failed consumers retry independently with
250 ms exponential backoff capped at 60 seconds. After eight consecutive failures,
explicit enable is required to try again. A successful acknowledgement resets
failures. Other queue clients keep working.

The host sends `{"type":"event","delivery_id":"event:...","event":...}` or
`{"type":"timer","delivery_id":"timer:...","timer":...}`. At most one delivery
awaits acknowledgement for a plugin. Events retain order; timers and events are
scheduled fairly. A plugin can acknowledge before asynchronous managed work exits.
The acknowledgement replaces optional namespaced state and applies optional timer
changes in the same transaction as the delivery position:

```json
{"type":"ack","delivery_id":"event:DATABASE:42","state":{"object":[]},"timers":[{"operation":"put","timer":{"id":"deadline:42","deadline":1234567890000,"payload":{"object":[]}}}]}
```

Use `state:null` to preserve state. `timers` defaults to an empty list. A remove
update is `{"operation":"remove","id":"deadline:42"}`. A plugin has at most 128
timers; state and individual timer payloads follow the metadata budget. Timer
identities and absolute deadlines survive restarts. Overdue timers are delivered
when the daemon and plugin are available, with their original deadline. Duplicate
acknowledgements cannot reapply a state change or remove a newer timer version.

## Capabilities and receipts

A plugin request has its own correlation ID and a typed operation:

```json
{"type":"request","id":"request-1","request":{"operation":"complete","action_id":"complete:DATABASE:42","task":"job","force":false}}
```

The host returns `{"type":"result","id":"request-1","result":...}`. Operation
and result schemas are defined by `CapabilityRequest` and `CapabilityResult`:

| Operation | Required capability | Result |
| --- | --- | --- |
| `read-task` with `name` | `read` | `task` with task snapshot |
| `read-state` | `state` | `state` with namespaced state |
| `complete` with `action_id`, `task`, optional `force` | `complete` | `task` or `completion-pending` |
| `managed-register` with `action_id`, `task` | `managed` | `managed-launch` with work and `launch_allowed` |
| `managed-report` with `action_id`, `state` | `managed` | `managed-work` |
| `managed-list` | `managed` | `managed-list` with this plugin's work |

All return typed errors for invalid input or unavailable capabilities. Names use
the same reversible quoting as CLI. The host supplies the authenticated plugin
identity; frames cannot choose another receipt owner. Completion receipts store
the canonical request and result in the same transaction as completion, successor
admission and events. Matching retries return the original task result. Reusing
an ID for another request or action kind fails. There is no separate receipt
commit after resource release, and no exactly-once promise for external effects.

## Managed work and recovery

Before launching external work, register a stable managed action against a running
task. Only the first `managed-launch` response with `launch_allowed:true` authorizes
launch. A repeated registration returns the durable action with false. After a
lost launch response or host failure, do not assume a launch did or did not happen.

The work states are `launch-pending`, `active`, `stopped` and `uncertain`. Plugins
report only active/stopped/uncertain; a stopped action cannot restart. Uncertain
work cannot become active again. Only its own plugin can report its state.

Manual, timeout and automatic completion share a persisted barrier. Completion
prevents new registrations for that task, sends `{"type":"cancel","work":...}`
and releases slots only after every managed action is confirmed stopped. The
manual command waits up to five seconds; on unresolved work it returns an error
and leaves the completion intent and ownership intact. Plugin completion returns
`completion-pending` and may retry its same action ID. Unmanaged completion keeps
the direct atomic path.

Disable and daemon shutdown request cancellation before stopping the executable.
The default cancellation grace is three seconds. Missing confirmation or process
failure records uncertain work, retains every slot and exposes the unresolved
action. A restarted host preserves uncertainty and does not authorize a repeated
launch. Removing a PID is not proof that all descendants stopped.

After personally confirming external cleanup, an operator can record
`plugin resolve NAME ACTION_ID --confirmed-cleanup`, then explicitly complete the
task. Resolution never launches work or implicitly completes the task. Receipts
and managed state must not be deleted to force a retry. Back up the database before
upgrade; older binaries cannot read schema 5. Stop an old daemon before replacing
its binary.

Native timer recovery tests inject an external fixture clock through the same
clock ports used by queue timestamps and plugin scheduling. Tests advance that
clock after observing persisted timers; they do not race short wall-clock sleeps.
The fixture adapter is compiled only with integration-test-hooks and adds no
production protocol fields.

Concurrent native sessions use the bundled SQLite 3.51.3 engine, including the
[upstream Unix VFS lock-order fix](https://www.sqlite.org/releaselog/3_51_2.html).
The regression suite repeatedly opens/closes real clients while ownership stays
held; it does not extend response deadlines to hide a blocked engine.

The explicit `plugin state NAME` command returns bounded namespaced metadata state. The
first-party [timeout plugin](TIMEOUT.md) runs over this same public protocol.

Cancellation is delivered once per unresolved action and live plugin session.
Reliable stdio transport either delivers it or fails that session; repeated polls
do not flood a worker while it confirms native tree termination. Responses and
cancellation have a bounded 32-frame outbound queue.

Protocol 2 stores bounded managed-action details and pages `managed-list` one
action at a time. Supply the returned `next` action ID as `after`; an empty page
ends recovery. This keeps long execution histories within the frame limit.
