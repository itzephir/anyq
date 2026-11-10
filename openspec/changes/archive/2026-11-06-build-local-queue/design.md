# Design

## Context

New empty repository; request and boundaries are in proposal. Rules were read in itzmoney/docs/architecture/FEATURES.md and itzdesign/docs/ARCHITECTURE.md. The user chose independent Kafka-like consumer groups.

## Goals / Non-Goals

**Goals:** one local file, atomic multiprocess operations, clear human/agent CLI, IO-free domain.

**Non-Goals:** Kafka network protocol, partitions, offsets API, strict completion order, exactly-once external commands, daemon, or UI.

## Decisions

- SQLite WAL with IMMEDIATE transactions and busy timeout replaces a server. Database stays on a local filesystem. Messages are immutable; processing is stored per message/group. Publishing creates deliveries for current groups; creating a group copies the retained stream from its beginning in one transaction.
- A topic owns the shared limit on active leases across all groups (default 1), representing a machine resource. Groups acknowledge independent copies while sharing the resource limit. Use one topic per physical emulator. Ready deliveries follow message ID; delayed retries wait until their ready time.
- At-least-once: time is stored as Unix milliseconds; each claim gets a new UUID token. The domain checks status/token/expiry before ack/nack/renew. Expired leases recover before mutations/lookups. Attempts belong to group deliveries. Failed deliveries can be retried explicitly with attempts reset.
- Ack and optional follow-up share one transaction, avoiding loss of the next stage. Repeated ack is rejected; producers can republish with a topic-unique idempotency key. Independent group acknowledgments may intentionally publish different follow-up messages.
- Queue owns Topic, ConsumerGroup, Message, Delivery: capacity, order, deduplication, retries, fencing, handoff. `api` contains pure models/contracts; `impl/dispatch` contains interactors and Records/Store/Clock/Tokens ports; `impl/sqlite` maps queries/transactions only; `impl/composition` is a factory. Interactors supply domain query filters; SQL executes them without its own policy.
- Worker: `api` has configuration/Worker contract; `impl/execution` handles claim, heartbeat, completion, follow-up, and Runner/Process/Wait ports; `impl/process` adapts POSIX commands/process groups; `impl/composition` is a factory. It depends only on queue api.
- `apps/cli` is the explicit composition root, controllers, and JSON presenters. Only main joins both feature factories. Cargo crates separate layers; Python checks the allowed graph, negative fixtures, and prohibited IO in inner crates. Inner crates use only std/public APIs.
- Worker reads JSON stdin and supplies ANYQ_MESSAGE_ID/ANYQ_GROUP/ANYQ_CONSUMER/ANYQ_ATTEMPT; command arguments follow `--`. Child stdout/stderr go to worker stderr. The interactor owns heartbeat; the process adapter only launches/polls/stops the group. Unexpected worker death may leave an external command; leases do not physically fence emulators.

## Risks / Trade-offs

- [External action repeated after failure] → idempotent commands/producer keys; no exactly-once promise.
- [Clock jump / worker SIGKILL] → document limitations, adequate leases, manual resource inspection. On detected lease loss, worker kills the process group.
- [Growing local stream] → retain history; retention/compaction after v1. CLI uses bounded pages; claims select only needed records.
- [Next-topic errors] → roll back ack; worker verifies the topic before launching commands.

## Migration Plan

Schema v1 is created on first open; unknown versions are rejected. For backups, stop workers and use SQLite backup. Release one binary through cargo install; Node is needed only for OpenSpec development.
