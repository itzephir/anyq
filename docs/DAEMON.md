# Local queue daemon

The same native `anyq` executable provides CLI and daemon modes. Each canonical
database has one owner, protected by a lifetime OS file lock. Ordinary queue
commands start it automatically and communicate through a private local endpoint.
The owner continues running after clients exit. Config, help and version remain
local and do not start it or open SQLite.

```sh
anyq --db ./queue.db daemon status
anyq --db ./queue.db daemon start
anyq --db ./queue.db topic create cpu
anyq --db ./queue.db task create build -t cpu
anyq --db ./queue.db daemon stop
anyq --db ./queue.db task build
# The owner restarts; build still holds cpu.
anyq --db ./queue.db task complete build
anyq --db ./queue.db daemon stop
```

Use `anyq daemon run --db PATH` to serve in the foreground under a process
supervisor. The foreground owner handles native Ctrl+C. A background Windows
owner has no console; use `daemon stop` for graceful shutdown. Login services and
remote clients are not installed. CLI messages and help are English-only, with
no language setting. `status` and `stop` accept an explicit --db/ANYQ_DB
even when config is damaged, allowing cleanup without reading that file.

An ordinary protocol stop lets a foreground owner return 0; native Ctrl+C returns
130. Both preserve committed task ownership.

After the first successful session, one initialized queue-service context is
retained until owner shutdown. Its disconnected client releases its stream and
quota immediately. This keeps SQLite's storage lifetime independent of client
connection churn, without holding a transaction or repeating any operation.

Background Windows startup uses an explicit handle allowlist through the
`windows-spawn` adapter. Only its null standard streams are transferred; the
caller's output pipes and other inherited handles are excluded, so capturing
a CLI command's output does not wait for the daemon to exit.

## Identity and access

Existing symlinks and relative paths resolve to a canonical database identity.
Before SQLite exists, a regular owner sidecar provides canonical filename
spelling on case-insensitive filesystems. The owner refreshes its endpoint after
acquiring the lock, and clients refresh discovery when that anchor appears, so
foreground and competing first clients share the same owner.
Status does not create either file; symlink sidecars do not redirect the database.
The owner lock is a persistent `.anyq-owner.lock` sidecar; its existence alone
does not mean an owner runs. The OS releases the actual lock when the process
dies. Stale endpoint replacement occurs only after obtaining that lock.

Unix endpoints are sockets in `/tmp/anyq-UID`, a checked owner-only directory,
with socket mode 0600 and peer effective-user verification on both sides.
Windows uses native named pipes with a protected owner-rights ACL. Endpoint names
contain a digest of the canonical path; the full identity is verified during
session opening. No IP listener, access token or network service is exposed.

Windows listeners explicitly assign their owner to the current process's user
SID. Before sending Open, clients read the connected pipe handle's owner and
require that same SID. A mismatch, missing owner or unreadable security descriptor
fails closed. The descriptor check uses the connected handle, not a process ID
or a second lookup by pipe name; the listener ACL separately restricts callers.

Do not concurrently use pre-daemon anyq binaries on the same database: those
binaries do not participate in the owner lock. Stop the daemon before rolling
back to an older compatible binary. Schema 4 includes metadata and atomic
schema-2/3 upgrades, and never resets running ownership.

Before upgrading or replacing the executable, stop the daemon for each active
database. Windows keeps the running executable in use, and a replacement CLI
may use a different protocol. The next queue command starts the installed
version and retains the stored tasks and ownership.

## Protocol and failure boundaries

Protocol version 2 uses a four-byte big-endian payload length followed by UTF-8
JSON bytes, with each chunk bounded to 16 MiB. The header's high bit indicates
continuation; the remaining bits give the chunk length. Incoming requests are
limited to 16 MiB in total. Large query responses span chunks and are reassembled
by the client so existing list/inspection output is not truncated. Every request/response has version and numeric
request ID. A queue connection begins with Open (database identity and policy
snapshot); ready is returned only after its SQLite session initializes. Further
requests invoke individual typed queue use cases. Ping and Stop are independent
control sessions. Test gate fields exist only in integration-test-hooks builds.

Open also carries the caller's absolute directory in lossless native encoding
for later execution context. Metadata values use typed envelopes with exact
decimal text and duplicate-checked object pairs. Physical JSON nesting is
bounded before deserialization while admitting 64 logical metadata levels.

The owner has separate quotas of 64 command sessions and 64 wait/start-poll
sessions. Wait moves to waiting capacity after a not-running result; start moves
there after Busy, so immediate wait/start success remains available. Other queue
operations acquire command capacity before executing. Connections move between
these quotas, so waiters do not occupy capacity needed to complete their owner.
Eight additional connections are reserved for Ping/Stop when command capacity is
full; Open must obtain command capacity before initializing SQLite. Exhausted
quotas return an explicit capacity error without releasing task ownership.
Permits are released when sessions close, bounding the owner to 136 sessions.
Initial empty input has a 30-second idle deadline. After a complete request is
queued, the session stays alive until disconnect, cancellation or shutdown;
established queue sessions survive long client output pauses between chain
operations. Invalid/control sessions close after handling, and session quotas
still apply. The first byte starts a separate 30-second whole-frame deadline,
including continuation chunks, even in an established session.
EOF, Cancel and shutdown remain observable during execution. Response writes have a five-second
inactivity deadline and a two-minute
total transfer limit. Progress renews the inactivity deadline so a consuming
client can receive a large result under contention. IO is nonblocking with
cancellation checks. Native asynchronous IO drives both Unix sockets and Windows
overlapped named pipes behind the synchronous queue adapters. A blocked stream
cannot hold up other clients or shutdown.
Graceful shutdown cancels sessions and allows up to ten seconds for draining.
If a worker cannot finish within that bound, the owner terminates the process
before releasing its lifetime lock; SQLite recovers unfinished transactions.
CLI chains retain separate requests/commits; wait/start-w poll narrow contracts
without holding a transaction while awaiting resources.

A successful mutation response follows commit. If its response is lost, the CLI
reports that the outcome is uncertain and never replays the sent request. Inspect
task/topic state before deciding to retry. Client cancellation reaches only its
own session; before commit it rolls back, afterward ownership remains durable.
See [INTERRUPTIONS.md](INTERRUPTIONS.md) for native signal boundaries and tests.

Metadata is described in [METADATA.md](METADATA.md). Lifecycle event delivery
and plugins are separate later stages of the [roadmap](PLUGIN-ROADMAP.md);
starting the daemon does not execute external work.

The bundled SQLite engine is 3.51.3 through rusqlite 0.39. This includes the
[upstream Unix VFS deadlock fix](https://www.sqlite.org/releaselog/3_51_2.html)
needed for concurrent session connection open/close. Native CI exercises that
connection churn while a task retains its resource ownership.

Stop waits for the lifetime database lock to be released after the endpoint closes.
Initial discovery can retry an owner launch after a competing owner exits; it does
not replay a sent queue operation after response loss.

SQLite WAL lock acquisition and initial schema reads retry bounded BUSY/PROTOCOL
contention before business mutations. A failed or uncertain queue mutation is
never retried through this path. Concurrent first clients are covered natively.
