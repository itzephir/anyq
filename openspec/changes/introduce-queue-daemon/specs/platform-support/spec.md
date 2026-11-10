# Spec Delta

## MODIFIED Requirements

### Requirement: Native coordination CLI
Task registration, topic linking, wait, and complete SHALL run natively on macOS, Windows, and Linux through a local background queue daemon with identical states, atomicity, and errors. Runtime SHALL NOT require Python, bash, or WSL; the caller SHALL remain responsible for external execution.

#### Scenario: Admission on Windows
- **WHEN** Windows CLI obtains task admission to two topics and exits
- **THEN** the task remains running in both topics until a separate complete

### Requirement: Native CLI interruption
The first Ctrl+C SHALL return 130. Client cancellation SHALL reach its daemon request's safe persistence boundary. Before commit, SQLite SHALL roll back the current transaction; after commit it SHALL preserve the result and ownership. Chains SHALL leave the last committed stage. Native integration tests SHALL send real SIGINT/CTRL_C_EVENT on all three operating systems without skip/ignore and verify states, slots, and subsequent operations.

#### Scenario: Signal after admission
- **WHEN** Ctrl+C arrives after running commit but before the CLI message
- **THEN** the task stays running and holds every slot until explicit complete

#### Scenario: Signal before admission
- **WHEN** Ctrl+C arrives before running commit or while waiting for a resource
- **THEN** the task is queued without partial ownership

#### Scenario: Independent clients survive cancellation
- **WHEN** a CLI request is cancelled while another client uses the daemon
- **THEN** other requests and the daemon survive client cancellation
