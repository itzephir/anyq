# Tasks

## 1. Domain and use cases

- [x] 1.1 Replace Message/Delivery with Task, states, and topic membership, retaining one standalone type per file; verify transitions and unique names. Implement shared domain task/topic naming: Latin initial, arbitrary tail, reserved bare commands, literal backticks allowing any content and stripped once; quoted/bare forms share identity. Test symbols, Unicode, spaces, nested backticks, canonical restore, and update docs/ARCHITECTURE.md.
- [x] 1.2 Implement pure admission policy using all topic capacities and one registration sequence; unit-test capacities 1/2, crossed linking order, predecessor blocking, and no partial acquisition.
- [x] 1.3 Extract typed RegisterTask, AddTaskToTopic, TryStartTask, and CompleteTask contracts/interactors with consumer-owned ports; RegisterTaskRequest supports atomic registration with initial topics. Use doubles to test ordinary completion before admission, force for registered/queued, concurrent reacquisition, unknown-topic registration atomicity, and persistence rollback. Document policy and transaction-boundary responsibilities.
- [x] 1.4 Implement separate ListTasks, GetTask, GetTopicQueue, exact TaskFilter reads, and capacity lookup through GetTopic; doubles and SQLite/CLI tests verify all listed states, exclusion of completed/registered from current queues, registration order, unknown names, and unchanged lookup ownership.

## 2. Storage

- [x] 2.1 Introduce SQLite schema 2 for topics/tasks/task_topics and read the version before changes; test new database opening, registered/queued/running/completed persistence on reopen, and rejection of old/future schemas without deleting existing data. Document separate database selection in README.
- [x] 2.2 Implement TaskRecords, TopicRecords, and shared UnitOfWork for registration, links, slot/predecessor counts, admission, and completion; SQLite integration tests verify two-topic atomicity, rollback, and rejection of capacity below usage.
- [x] 2.3 Use truly independent connections/processes to test contention for the last shared slot, two waits for one name, add concurrent with admission, and force concurrent with admission/wait; confirm exactly one running transition and no partial ownership.

## 3. Native CLI and replacement of the previous model

- [x] 3.1 Implement topic/task list/--list/-l, topic create TOPIC --capacity/-c N, topic capacity TOPIC [N], topic link TOPIC NAME and equivalent task link NAME TOPIC, topic TOPIC, and task NAME. Support name-first actions and help [command]. Task create accepts repeated --topic/-t and dependent --wait/-w, --complete/-c with -twc TOPIC; link supports --wait/--complete, wait supports --complete/-c, complete supports --force/-f. CLI tests verify equivalent forms, chains, quoted names, errors; update README.
- [x] 3.2 Implement blocking task wait retrying separate TryStartTask without holding a transaction during pauses; CLI tests verify immediate no-topic error, waiting for all slots, exit 0 only after running, and errors after forcing an unstarted task. Document caller-owned execution after admission.
- [x] 3.3 Remove old CLI commands, Message/Delivery/lease contracts, four worker crates, process adapters/fixtures, and unused dependencies; verify the current workspace build, Cargo graph, and negative architecture fixtures. Update AGENTS.md, OpenSpec context, docs/ARCHITECTURE.md, and docs/CODE-RULES.md for the agreed model replacement.

## 4. End-to-end verification and pilot

- [x] 4.1 Replace demo and old worker workflow tests with the native scenario; verify commands, aliases, flags, and boundaries on three operating systems. Real processes and Ctrl+C deterministically check before/after registration, link, admission, complete, database initialization, waiting, chains, autocreate, reads, config writes before/after replacement, and lock waits; verify exit/state/slots, no partial changes, and subsequent operations. No skip/ignore; document each boundary and add a separate required CI integration check.
- [x] 4.2 On a separate new-format database, run itzdesign build/lint through direct task commands; record admission across shared topics, no simultaneous execution, and release after complete. Share the new usage with the linked chat, retaining the old temporary binary and pilot database for history/rollback.
- [x] 4.3 Run required fmt/clippy/tests, both architecture linters, Ruff, strict OpenSpec validation, and git diff --check; submit a PR, wait for all eight CI jobs, and review the final contract against user examples. Do not mark unverified scenarios complete.

## 5. Configuration and autocreate

- [x] 5.1 Extract config api/dispatch/file/composition crates with narrow GetConfig, GetConfigValue, SetConfigValue, and UnsetConfigValue. Define three domain keys, types, defaults, and validation; IO-free unit tests cover unknown keys, bool, path, and unset.
- [x] 5.2 Implement TOML storage for ~/.anyq/config.toml with atomic replacement and concurrent-write locking; integration tests cover absence, reads/writes, preserving the old file on failure, and independent concurrent mutations. Update Cargo graph, architecture linters, and CI without reducing matrices.
- [x] 5.3 Implement git-style config CLI: key/value reads/writes, --get, --unset, --list/-l. Test --db > ANYQ_DB > db.path > default and Unix/Windows home; config/help/version do not open the database; all tests isolate user home.
- [x] 5.4 Introduce domain AutoCreatePolicy explicitly injected into queue use cases. task.autocreate/topic.autocreate create missing entities inside the main transaction; errors roll back creation and retain other invariants. Unit/SQLite/CLI tests cover defaults false, read/link/wait/complete/capacity, rollback, and independent processes.
- [x] 5.5 Update OpenSpec artifacts, README, docs/ARCHITECTURE.md, CODE-RULES, and demo for final syntax, quoted/bare identity, config, and autocreate; verify the complete contract after green CI and do not sync baseline before an authorized merge.

- [x] 5.6 Add config list as a subcommand equivalent to --list/-l; list is not a valid ConfigKey. Integration-test defaults/persisted settings, no file writes/database opens, help, invalid combinations, and real Ctrl+C at read boundaries; update README and delta spec.

## 6. Review fixes

- [x] 6.1 Domain rules define valid config sections; the TOML adapter rejects tables instead of terminal values and unknown sections, including empty ones. Integration-test reads, set/unset, byte preservation on errors, valid empty sections, and successful writes after file repair.
- [x] 6.2 Make stdout/stderr and help interruptible by the first Ctrl+C with a full pipe, without changing persisted ownership. Add real native integration tests with unread pipes, verified write start, exit 130, and subsequent operations; also test complete output with successful reads and write errors.

## Workflow follow-up

- Merge the PR only on an explicit user request, preserving main protection.
- After implementation, review, and authorized merge, sync delta specs and archive the change. Removing command-worker includes its obsolete Purpose; preserve other existing Purposes.
