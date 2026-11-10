# Design

## Context

The user contract is a named task, several resource topics, and explicit wait/complete. Messages, consumer-group copies, leases, and worker are replaced completely. The caller executes external commands. Behavior is implemented in Rust with the same native CLI on Linux/macOS/Windows.

## Goals / Non-Goals

Goals: atomic admission across all topics, one registration order, durable ownership, exact CLI syntax, typed settings, and real-process interruption tests. External commands, outcomes, heartbeat, and automatic retries are outside anyq's responsibility.

## Decisions

### Domain model and order

Task stores a canonical unique name, monotonic registration sequence, state, and topic set. Registered has no topics; queued has links without ownership; running holds one slot from each topic; completed retains history. Names are not reused. Running/completed membership is immutable; queued links are idempotent.

Each topic has capacity minus running free slots. The domain admits a task when the number of queued predecessors with smaller registration sequences is less than free slots in every topic. One order applies across all shared resources; queued tasks retain position without active wait. This permits later tasks to be delayed when the head task waits for another resource.

### Names and identity

Shared task/topic rules live in queue/api. Bare names start with ASCII Latin and have an arbitrary tail. Bare list/create/capacity/link/add/wait/complete/config/help are reserved. One literal outer backtick pair permits any content, including empty strings and commands, and is stripped once. build and `build` are identical. Escapes inside the wrapper are not interpreted.

Ports store canonical strings. Restore does not parse them again; otherwise nested backticks would change identity on read. Domain formatting produces a reversible CLI representation. The shell escapes spaces before producing argv.

### Narrow use cases and transactions

Queue api/dispatch/sqlite/composition retain inward dependencies. RegisterTask, AddTaskToTopic, TryStartTask, CompleteTask, ListTasks, GetTask, GetTopicQueue, and topic management have separate typed contracts. UnitOfWork/TaskRecords/TopicRecords belong to dispatch. SQL executes exact TaskFilter queries; the domain decides admission.

RegisterTaskRequest combines a name and initial topics: one transaction creates the task and all links. TryStartTask reads state, links, and usage across all topics and saves running atomically. CompleteTask saves completed. Usage is derived from running; no separate counters exist. Errors roll back all changes; capacity cannot fall below usage.

### Config and autocreate

Config is a separate api/dispatch/file/composition feature; queue does not depend on its impl. The domain defines task.autocreate/topic.autocreate bool defaults false and optional nonempty db.path; separate use cases read/change typed values. Git-style config stores ~/.anyq/config.toml, using USERPROFILE as Windows home.

Under an OS file lock, the TOML adapter rereads current state, applies the operation, syncs a temporary file in the same directory, and atomically replaces the main file. Errors before replacement preserve old bytes. Concurrent successful changes to different keys are not lost.

ConfigKey defines valid sections through known-key prefixes. The adapter checks tables before recursion, including empty ones: unknown sections or tables instead of terminal values fail. Empty task/topic/db containers use defaults. Read errors prevent set/unset and preserve original bytes.

CLI selects --db > ANYQ_DB > db.path > platform default; ~/ in config expands relative to home. A config snapshot becomes an explicitly injected queue-domain AutoCreatePolicy. Missing entities are created in the main transaction; new topics have cap1. Wait without topics or ordinary complete errors roll back new names. Successful reads with autocreate may create entities without ownership. Existing-state rules are not bypassed.

### CLI and external execution

README describes list flags, task/topic lookups, capacity read/write, create/link/wait/complete, and name-first/command-first forms. Config list/--list/-l are equivalent; list is a subcommand and fails ConfigKey validation. CLI normalizes syntax only, not business rules. Task create replaces add; both config settings are named autocreate. Help/version do not open config/DB; config does not open DB.

Task create -t -w -c and link -w -c invoke separate use cases sequentially; wait -c invokes admission then completion. Create wait requires topic, complete requires wait; link complete requires wait. Parsing -twc TOPIC reorders value-taking -t. Unknown/incomplete combinations are rejected before IO.

Busy causes a short CLI pause without holding a transaction. Wait returns 0 after running commit. Wait exit, crashes, and time do not release ownership. Two waits for one name do not acquire twice; forcing a waiting task makes its next poll fail.

### Interruptions and verifiable boundaries

The first Ctrl+C sets a token in external adapters; IO/lock loops/commit boundaries check it. Before commit the open transaction rolls back; afterward persisted state is retained. CLI returns 130. Chains have separate queued/running/completed commits. A second Ctrl+C terminates forcibly.

Stdout/stderr and help use a separate writer thread without business state. The main thread waits on a channel with a short timeout; the first Ctrl+C stops waiting and releases resources without joining the writer. Successful writes are confirmed before the next stage. Native tests fill real pipes, read one byte to confirm output began, and await exit after one signal without draining the rest.

Only integration-test-hooks adds external checkpoints. Integration tests launch the real CLI and send SIGINT on Unix or CTRL_C_EVENT in a separate Windows console. They verify registration, link, admission, complete, schema, config replacement, autocreate, read, and real lock-wait boundaries. Normal binaries have no test IO. docs/INTERRUPTIONS.md has the outcome table; the suite is required in every CI integration job.

### Storage and build

SQLite schema2: topics/tasks/task_topics with unique names, monotonic ordering keys, and FKs. WAL, synchronous FULL, short IMMEDIATE transactions. Old schema1 and unknown versions are rejected before mutation without automatic conversion.

Four worker crates and the old CLI are removed. Workspace: four queue crates, four config crates, CLI, and two tooling crates. One standalone type per file; explicit composition roots without execute-dispatchers or service locators.

## Risks / Trade-offs

- Crashes after admission leave resources held: explicit complete follows inspection of external work.
- A head queued task may reduce utilization: selected global FIFO without partial acquisition.
- SQLite polling needs short transactions and pauses; no daemon is added.
- The API is cooperative: knowing a name allows complete/force; this is not an OS authorization boundary.
- A post-commit signal may precede the success message: inspect state through a separate read.
- Forced termination by a second signal may leave a temporary config file; the main file stays intact.

## Migration Plan

1. Implement domain, typed use cases, schema2, native CLI, and config; remove worker.
2. Verify unit and real SQLite/file/CLI/Ctrl+C suites, both architecture linters, and three operating systems.
3. Do not import old data: use a separate database and retain the pilot's old binary.
4. Test itzdesign build/lint on a separate database and share the native protocol with the linked chat.
5. Wait for green CI and PR review. Merge only on user request, then sync/archive OpenSpec. Removing command-worker includes its obsolete Purpose; preserve other baseline Purposes.
