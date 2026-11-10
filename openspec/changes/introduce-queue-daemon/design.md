# Design

## Context

See [proposal.md](proposal.md) and [the roadmap](../../../docs/PLUGIN-ROADMAP.md).
The current CLI main composes queue/config implementations and invokes narrow
queue traits. SQLite uses an IMMEDIATE transaction per operation, and
CheckTaskRunning is observational. Existing integration tests send real native
signals at adapter checkpoints and include lock contention and pipe saturation.
The schema is version 3. Daemon introduction itself needs no business-schema change.

## Goals / Non-Goals

**Goals:** retain the typed use-case boundary and transactional behavior while
moving queue IO behind an independently running owner; make interruption and
ambiguous response loss testable across processes.

**Non-Goals:** OS login services, remote access, a message broker, event sourcing,
metadata or plugin execution. Automatic startup runs a detached native process;
`daemon run` also supports foreground supervision.

## Decisions

### Single executable, separate runtime boundary

Keep `anyq` as the distributed binary, with daemon subcommands selecting server
mode. Add a daemon feature with API/protocol, client adapter, runtime adapter and
composition crates; designate CLI main and daemon composition as roots. The
client implements each existing queue API trait individually. The transport
router dispatches wire variants to typed use cases; it is not a generic business
`execute(Command) -> Output` API. Queue/config inner crates acquire no transport,
serialization, filesystem, threading or clock dependencies.

An extra distributed binary would require installer/release changes before any
behavioral benefit. An in-process client fallback would violate owner isolation.

### Private local transport and singleton

Use native local sockets (Unix-domain sockets on Unix, named pipes on Windows)
through a reviewed cross-platform adapter. [Interprocess's local socket API](https://docs.rs/interprocess/latest/interprocess/local_socket/)
is the initial candidate; enforce private directory/socket permissions and a
current-user pipe ACL rather than relying on endpoint-name secrecy. Use an OS
file lock for the canonical database identity, held for the owner's lifetime.
Resolve existing database symlinks or the nearest existing parent plus filename
before discovering the owner. Atomically publish discovery only after bind and
compatible database initialization. Remove stale discovery under the lock only.

Use a per-user private runtime directory and a stable digest of the canonical
path to avoid socket length limits. Verify the canonical identity in the
handshake, preventing digest collisions from connecting to the wrong queue.
Do not expose an IP listener; it would add authentication and firewall concerns
without a remote-client requirement.

### Bounded typed request/response frames

Use length-delimited versioned JSON frames with typed request/response variants,
canonical operation payloads, request IDs and typed queue errors. Set explicit
frame, connection and response-write limits. Preserve canonical names without
reparsing during DTO restoration. A successful response follows commit. Retry
connection/start discovery before sending, but never replay a mutation whose
response was lost; return an inspectable uncertain-result error.

The CLI reads config and resolves db.path/ANYQ_DB/--db as today, passing the
validated QueuePolicy snapshot per request. Config/help/version remain local.
Client wait/start-w continue polling the corresponding narrow remote contract,
so no server request holds a transaction while waiting for admission. Separate
chain stages remain separate requests and commits.

### Request-scoped cancellation and native test gates

The daemon accepts bounded concurrent sessions. A request worker creates its
own queue composition and SQLite connection inside the daemon, with its own
cancellation token. A connection monitor detects cancellation/EOF and requests
rollback at the existing adapter boundaries. A slow request or locked database
must not block cancellation, status or other connections. Owner shutdown cancels
workers and closes clients with bounded draining; committed ownership survives.

Integration gates move to a feature-only request context with per-session hit
counts and selected gate directory. A CLI process forwards that context only in
integration-test-hooks builds; production accepts no test-gate fields. Tests
interrupt the real client while its real owner is gated before/after commits,
and separately interrupt the foreground owner. Do not use process-global mutable
environment overrides inside a concurrent daemon. Normal client tests terminate
their isolated owner during fixture cleanup, without adding production idle exit.

### Lifecycle and presentation

`start` waits for readiness, `status` probes without starting, `stop` requests
bounded graceful shutdown, and `run` owns the foreground signal handler. Queue
commands auto-start and wait for readiness without inheriting CLI stdout/stderr
pipes. New lifecycle messages are localized in en/ru at the presentation boundary;
wire error kinds remain language independent. Preserve existing default-English
text and exit codes, including first Ctrl+C = 130 and forced second interruption.

## Risks / Trade-offs

- [Client cancellation races commit] -> inspect durable state after uncertain
  results; never interpret cancellation as releasing already acquired slots.
- [Stale endpoints and startup races] -> lifetime lock, canonical identity,
  atomic readiness publication, native contention/crash tests.
- [Native pipe access and shutdown differ] -> use explicit platform adapters,
  bounded IO and required Windows/macOS/Linux process tests.
- [Old binaries still write schema 3 directly] -> document that mixed-version
  concurrent use is unsupported; singleton enforcement covers supported clients.
- [More processes in test suites] -> fixture-owned daemons, explicit cleanup and
  checks that blocked responses cannot keep a test or owner alive indefinitely.

## Migration Plan

1. Add transport/runtime and typed remote adapters behind daemon composition.
2. Switch every CLI queue operation to the client; remove direct queue impl access
   from CLI business paths and update architectural allowlists/negative fixtures.
3. Keep the compatible database unchanged, preserving version 2 upgrade behavior.
4. Run mandatory local checks and native CI, including interruption and installers.
5. Update architecture/config context and examples to explain the new owner.
   Rollback requires stopping the daemon before using an older binary; this stage
   performs no business-schema migration and does not reset ownership.
