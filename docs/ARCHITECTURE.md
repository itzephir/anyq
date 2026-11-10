# anyq architecture

anyq coordinates named tasks and admission to local resources. Clean Architecture is organized by queue, config and daemon features, then by layer; api and impl have separate Cargo crates. Feature implementations depend on other features' APIs only. Designated composition roots assemble concrete implementations; daemon composition wires queue implementation behind injected runtime ports.

| Module | Responsibility |
| --- | --- |
| queue/api | Task, TaskState, Topic, TopicQueue, ResourceAvailability, AutoCreatePolicy, name parsing and formatting; separate use-case contracts |
| queue/impl/dispatch | Registration, linking, admission, completion, reads, and topic management; consumer-owned persistence ports |
| queue/impl/sqlite | Schema, exact queries, IMMEDIATE transaction boundary |
| queue/impl/metadata | Stateless outer JSON/YAML/TOML conversion and exact decimal serialization |
| queue/impl/composition | Explicit assembly of individual queue interactors |
| config/api | Config, typed keys/values, defaults, separate read/write contracts |
| config/impl/dispatch | GetConfig, GetConfigValue, SetConfigValue, UnsetConfigValue, and consumer-owned Store |
| config/impl/file | TOML, file locking, atomic replacement |
| config/impl/composition | Explicit assembly of individual config interactors |
| daemon/api | Versioned wire DTOs and explicit exports of narrow queue service handles |
| daemon/impl/transport | Private native endpoints, canonical database identity, lifetime locks and bounded framing |
| daemon/impl/client | Separate remote adapter for each queue use-case contract |
| daemon/impl/runtime | Concurrent cancellable sessions and outer transport routing through injected QueueFactory |
| daemon/impl/composition | Explicit client assembly and server queue factory; only this root imports queue impl |
| apps/cli | Syntax, text responses, path resolution, blocking CheckTaskRunning polling, and signals |
| tooling/architecture-lint | Rust structure checks |
| tooling/integration-test-hooks | Optional external integration-test checkpoints only |

## Domain

Task has a unique name, an immutable monotonic registration sequence, a state, and a set of topics. Registered has no topics; adding a link makes it queued. Full admission makes it running; complete makes it completed. A name cannot be reused after completion.

Task owns transitions, resource-set completeness, and the prohibition on changing running/completed membership. ResourceAvailability checks free slots and queued predecessors. The same registration order applies across all topics. A queued task keeps its place without an active wait; remove an unwanted task with complete --force.

parse_name and parse_topic_name define shared rules: a bare name starts with an ASCII Latin letter and is not list/create/capacity/link/add/start/autostart/wait/complete/meta/metafile/config/help; its tail is unrestricted. One literal outer pair of backticks allows any content, including an empty string, reserved words, leading hyphens, and nested backticks. It is removed exactly once. build and `build` are the same identity.

Storage and ports use canonical strings. Task::restore and Topic::restore restore an already parsed name without stripping backticks again and validate the aggregate's structural invariants. format_name produces a reversible representation of a canonical name. CLI and SQL do not duplicate naming rules.

Config owns task.autocreate/topic.autocreate (bool, default false), task.autostart (bool, default true), and db.path (a nonempty string or absent). AutoCreatePolicy belongs to queue and separately allows creation of each entity kind. CLI passes values from a Config snapshot into QueuePolicy; queue does not depend on config impl. On creation, a task stores task.autostart as its default. Topic.autostart is an optional override; every topic must allow automatic admission. Explicit --stop is stored separately and prevents it until manual start. StartPreference defines default/start/stop/toggle; the domain computes the effective value using the complete topic set.

ConfigKey also defines valid containers as strict prefixes of known keys. The TOML adapter validates a container before traversing its table: empty task/topic/db are valid; unknown sections and tables in place of terminal values are rejected. The adapter does not duplicate the business-key list.

## Use cases and atomicity

Every use case has its own contract and interactor. There is no generic execute(Command) -> Output. QueueServices in queue API and ConfigModule contain explicitly constructed handles without business behavior, string lookup, or a service locator. QueueModule remains an alias of QueueServices for local composition. Both local and remote composition use the same passive typed handle inventory.

RegisterTask accepts RegisterTaskRequest with a name and initial topics. Topic validation, registration, and all links are saved atomically. AddTaskToTopic uses the same boundary for one link. Autocreate runs inside the main transaction: failed wait/complete/link operations also roll back new entities. A successful lookup may create a missing entity when a setting explicitly allows it.

Registration, linking, completion, and topic capacity/autostart changes invoke a shared internal admission pass over queued tasks before commit. The pass obtains all queued tasks using an exact filter without a topic and iterates by sequence. For each task it reads the complete current snapshot of topics, running tasks, and queued predecessors; Task and ResourceAvailability decide whether the full transition is valid. A candidate is admitted automatically only when Task.permits_autostart allows it; a stopped task keeps its queue position without starting. After running is saved, later candidates see the new usage. No partial acquisition occurs. Errors roll back both the original change and all subsequent admissions. Usage is derived from running tasks and links; there are no mutable slot counters. Capacity reductions below usage are rejected.

AddTaskToTopics and AddTasksToTopic have separate typed inputs: one task and several topics, or several tasks and one topic. A shared internal helper saves the entire batch before the admission pass. Duplicate task names are deduplicated; returned tasks are ordered by registration. The older single AddTaskToTopic uses the same helper. All Task results reflect state after automatic admission.

StartTask attempts immediate full admission regardless of autostart; Busy leaves the queue unchanged, while running returns AlreadyRunning. Ordinary CLI start turns Busy into an error; start -w retries this narrow use case, and start -wc then invokes CompleteTask. --start on create requires immediate admission only when effective autostart is off and rolls back the entire registration on Busy.

CheckTaskRunning only checks domain state: queued is false, running is true, registered/completed are errors. Checking inside a transaction preserves rollback for failed autocreate; it does not write existing task states.

ListTasks reads all names; GetTask returns the aggregate. GetTopicQueue requests domain-owned TopicQueue.STATES and builds the queue in registration order. Store/UnitOfWork, TopicRecords, TaskRecords, and TaskFilter belong to dispatch. The interactor supplies the exact filter; SQLite executes it without deciding admission. Lookups do not change existing ownership.

Config Store combines reading/changing one Config. Under a file lock, FileConfig rereads current state, applies the typed use case, writes and syncs a temporary file in the same directory, then atomically replaces config.toml. Errors before replacement preserve the old file. The lock is released on exit; a persistent .lock file does not indicate a held lock.

## External details

The domain and use cases do not use IO, SQL, clocks, CLI, or SDKs. Composition roots are features/queue/impl/composition, features/config/impl/composition, features/daemon/impl/composition, and apps/cli/src/main.rs; only they assemble implementations. The workspace has 17 crates, including two development tools. CLI no longer imports queue composition or SQLite; daemon composition creates each session's local SQLite-backed services. Protocol serialization belongs to the technical daemon boundary, not queue/config domain APIs.

Metadata/MetadataValue, exact normalized MetadataNumber, dotted paths and the incremental size/depth budget belong to queue API. Separate task/topic read, set, unset and replace interactors use the existing transaction. Task owns its edit-state restriction; metadata mutations never invoke admission. Creation saves initial metadata before links and admission. Topic capacity/autostart changes preserve its metadata.

Stateless format conversion lives in queue/impl/metadata. SQLite uses its JSON adapter; only the CLI composition root imports the format implementation and injects parsing functions into the local MetadataInput adapter. Controllers receive those functions without depending on an implementation crate. MetadataInput performs bounded client-local UTF-8 file reads. YAML/TOML dependencies are enabled by its files feature. Queue domain and interactors import no serialization libraries. Format and transport limits are documented in [METADATA.md](METADATA.md).

One daemon holds the lifetime owner lock for each canonical database. CLI sessions
send an Open frame with the canonical identity and validated policy snapshot;
the owner initializes that session's SQLite connection before returning ready.
Each subsequent wire operation routes to one typed use case. Initial connection
discovery can start a detached owner, but no sent operation is automatically
replayed after response loss. Private native IPC, framing and shutdown limits
are documented in [DAEMON.md](DAEMON.md).

The native transport uses an IO reactor behind polling Read/Write adapters;
queue interactors remain synchronous and independent of that reactor. Each wire
chunk and complete incoming request is bounded, while query results can span
chunks to preserve large-list compatibility.

CLI normalizes command-first/name-first forms, link directions, and short flags while preserving arguments. Flag dependencies are checked before opening the database. create/link → wait → complete chains invoke separate use cases sequentially. Stages have separate commits; they are not one transaction. Complete does not execute an external command.

CLI retries CheckTaskRunning after false with a short pause without holding a transaction. It returns 0 when it observes running, including repeated and concurrent waits. For batch topic link -wc, tasks are awaited and completed in registration order. The caller launches external work. CLI exit, crashes, and elapsed time do not release slots.

The first Ctrl+C sets an external cancellation token. A client sends cancellation to its own daemon session and waits for a safe-boundary response; EOF also cancels that session. Adapters check its separate token at IO/commit boundaries and while waiting for locks; RAII rolls back the open transaction and removes the temporary file. Committed state survives even if CLI returns 130. A second Ctrl+C forcibly terminates the process. The foreground owner's first signal cancels and drains sessions without releasing ownership. Boundary semantics and checks are described in [INTERRUPTIONS.md](INTERRUPTIONS.md).

stdout/stderr writes, including help, run in a separate thread owning only its message. The main thread waits for the result through a channel with a short timeout and checks the cancellation token. On cancellation it releases its resources and returns 130 without joining a blocked writer; the writer terminates with CLI. During normal execution, the next step begins only after the write result. Standard Rust streams retain their platform behavior; no business state is passed to the writer.

SQLite stores topics/tasks/task_topics. Schema 4 adds checked task/topic metadata objects; schema 2 receives autostart fields and metadata, while schema 3 receives metadata, in one IMMEDIATE transaction preserving all states and ownership. Version 1 remains incompatible. Opening a database does not perform domain admission. Existing database compatibility is checked before modifying its data or journal mode. An incompatible file is rejected without changing or deleting data.

## Checks

One standalone type per file; see [CODE-RULES.md](CODE-RULES.md). The Rust AST linter checks structure and prohibits a generic business dispatcher. The Python checker verifies the Cargo graph, inward dependencies, CLI factories only in main, and no direct CLI queue implementation access. Wire routing is an external adapter and never replaces typed use cases.

Domain/use-case tests verify rules without infrastructure. SQLite and file integration suites use real data, failures, and independent writers. CLI workflow and native Ctrl+C suites run real clients and owners on Linux/macOS/Windows. Test hooks are enabled by a separate feature in external adapters only; they are absent from the normal binary and production protocol. Daemon gates use thread-scoped session context and hit counts without changing global environment. Fixtures stop every isolated owner before removing its database directory.
