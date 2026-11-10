# platform-support Specification

## Purpose

Users can run the same local queue natively on macOS, Windows, and Linux, with CI checking real processes and durability on every platform.

## Requirements

### Requirement: Windows database path
Without --db, ANYQ_DB, or db.path in config, Windows CLI SHALL use LOCALAPPDATA/anyq/queue.db. Explicit paths SHALL override the default location.

#### Scenario: Local database
- **WHEN** Windows CLI runs with LOCALAPPDATA and no database path in arguments, environment, or config
- **THEN** the database is created in the anyq subdirectory of LOCALAPPDATA

### Requirement: CI matrix
CI SHALL check formatting, architecture, OpenSpec, and lint on Linux, and builds, domain/end-to-end tests, and demos natively on macOS, Windows, and Linux. Failure on one platform SHALL NOT cancel the others.

#### Scenario: Implementation change
- **WHEN** a commit is pushed or a PR opened
- **THEN** lint passes on Linux and builds and tests pass on all three native runners

### Requirement: Native coordination CLI
Task registration, topic linking, wait, and complete SHALL run natively on macOS, Windows, and Linux with identical states, atomicity, and errors. Runtime SHALL NOT require Python, bash, or WSL; the caller SHALL remain responsible for external execution.

#### Scenario: Admission on Windows
- **WHEN** Windows CLI obtains task admission to two topics and exits
- **THEN** the task remains running in both topics until a separate complete

### Requirement: Native CLI interruption
The first Ctrl+C SHALL return 130. Before commit, SQLite SHALL roll back the current transaction; after commit it SHALL preserve the result and ownership. Chains SHALL leave the last committed stage. Native integration tests SHALL send real SIGINT/CTRL_C_EVENT on all three operating systems without skip/ignore and verify states, slots, and subsequent operations.

#### Scenario: Signal after admission
- **WHEN** Ctrl+C arrives after running commit but before the CLI message
- **THEN** the task stays running and holds every slot until explicit complete

#### Scenario: Signal before admission
- **WHEN** Ctrl+C arrives before running commit or while waiting for a resource
- **THEN** the task is queued without partial ownership

### Requirement: Interrupt blocked output
The first Ctrl+C SHALL exit CLI with 130 even when stdout or stderr is full and the consumer is not reading. Output cancellation SHALL NOT change persisted states, ownership, or settings. Native integration tests SHALL reproduce actual pipe saturation and send one signal on each OS.

#### Scenario: Consumer stopped reading
- **WHEN** CLI is blocked writing to full stdout or stderr and receives the first Ctrl+C
- **THEN** the process exits with 130 without a second signal or waiting for pipe reads

#### Scenario: Output fully read
- **WHEN** the consumer reads stdout without interruption
- **THEN** CLI returns the full result with its ordinary exit code
