# Spec Delta

## ADDED Requirements

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

## REMOVED Requirements

### Requirement: Native CLI and worker
**Reason**: The coordination CLI replaces command workers with payloads/group identities.
**Migration**: Use task create, topic link TOPIC NAME or task link NAME TOPIC, task wait, and task complete on the same OS; run external commands yourself.

### Requirement: Process-tree termination
**Reason**: anyq does not launch external work or own its process tree. Caller crashes do not release slots automatically.
**Migration**: The caller manages external processes; complete tasks explicitly after inspecting remaining work.
