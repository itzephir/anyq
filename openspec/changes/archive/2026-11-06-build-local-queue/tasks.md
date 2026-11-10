# Tasks

## 1. Domain boundaries

- [x] 1.1 Create a queue/worker Cargo workspace with separate api/impl; document rules in AGENTS.md/docs/ARCHITECTURE.md; verify graph/negative fixtures with the architecture checker.
- [x] 1.2 Implement pure queue/lease/retry models and interactors with injectable Store/Clock/Tokens; test domain scenarios without SQLite.

## 2. Durability

- [x] 2.1 Implement SQLite port/DI factory; verify independent groups, late subscription, deduplication, handoff rollback, expiry/fencing, and real-connection concurrent claims.

## 3. Usage

- [x] 3.1 Implement worker through queue api and process/wait ports; unit-test heartbeat, lease loss, and failure without real processes.
- [x] 3.2 Implement CLI, JSON presenters, and POSIX process adapter; document/test manual and automatic build → emulator scenarios, stdin, and exit codes.

## 4. Integration

- [x] 4.1 Pass fmt, clippy, cargo tests, architecture checker, strict OpenSpec validation, and end-to-end checks; create a private GitHub repository, push bootstrap, and verify CI.

## Workflow follow-up

- Archive through OpenSpec after review; do not move unfinished requirements into baseline specs.
