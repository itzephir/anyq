# Spec Delta

## Purpose

Keep a local queue owner alive independently of CLI clients and provide a reliable native request interface for queue operations and later background extensions.

## ADDED Requirements

### Requirement: One local owner per database
The system SHALL allow one daemon owner per canonical database path. Concurrent clients SHALL discover or start that owner without creating competing owners. Distinct database paths SHALL remain independent. Queue CLI clients SHALL NOT open or mutate SQLite directly.

#### Scenario: Concurrent first commands
- **WHEN** several clients address one database before a daemon exists
- **THEN** one owner starts and every successful request uses it without duplicate registration or excess resource usage

#### Scenario: Equivalent paths
- **WHEN** clients use relative, absolute or symbolic-link paths to the same database
- **THEN** they resolve to the same owner

### Requirement: Automatic startup and explicit lifecycle
Queue commands SHALL ensure a daemon is available. `daemon start` SHALL start or reuse it, `daemon run` SHALL serve in the foreground, `daemon status` SHALL report running/stopped without starting it, and `daemon stop` SHALL stop it without releasing task ownership. Ordinary client exit SHALL leave the daemon alive.

#### Scenario: Client exits
- **WHEN** task registration finishes and its CLI exits
- **THEN** daemon status still reports running and subsequent commands reuse the owner

#### Scenario: Stop and restart
- **WHEN** the daemon stops and a later queue command restarts it
- **THEN** existing task states, links, order and running ownership are preserved

### Requirement: Private bounded local protocol
Daemon communication SHALL be restricted to the invoking account and local machine, use an explicit protocol version and bounded messages, and reject invalid or incompatible requests without executing queue operations. Endpoint discovery SHALL NOT expose access secrets through ordinary output or logs.

#### Scenario: Invalid peer input
- **WHEN** an unauthorised, oversized, malformed or incompatible request reaches the endpoint
- **THEN** it is rejected without database changes and valid clients can continue

### Requirement: Request results reflect persistence
A successful mutation response SHALL be sent only after its commit. Loss of a response SHALL NOT imply rollback. The client SHALL report an uncertain outcome after losing an in-flight mutation response and SHALL NOT automatically replay that mutation. Explicit queries SHALL allow the caller to inspect durable state.

#### Scenario: Response lost after admission
- **WHEN** a task obtains running ownership but its connection closes before the response
- **THEN** ownership remains durable and a later state lookup reports running

### Requirement: Client cancellation is request scoped
Client cancellation or disconnect SHALL cancel only its active request at a safe boundary. Work before commit SHALL roll back; committed work SHALL survive. Cancelling a wait SHALL NOT stop the daemon, cancel another client's request, or release any task's resources.

#### Scenario: Cancel one of two clients
- **WHEN** one waiting client receives Ctrl+C while another client queries the queue
- **THEN** the first exits 130, the other succeeds, and existing ownership is unchanged

### Requirement: Nonblocking owner lifecycle
A blocked client input/output stream or long wait SHALL NOT prevent other clients from querying or mutating the queue. Graceful daemon shutdown SHALL stop accepting requests, cancel unfinished work at persistence boundaries and preserve committed ownership.

#### Scenario: Abandoned slow client
- **WHEN** a client stops reading a large response
- **THEN** unrelated clients remain usable and daemon shutdown can finish without draining that client

### Requirement: Recovery and endpoint replacement
After a daemon crash, the next start SHALL recover persistent state and replace only stale discovery data while holding ownership. It SHALL NOT infer task completion from process death, replay unknown mutations, or perform new admission merely by opening the database.

#### Scenario: Owner crash
- **WHEN** an owner is forcibly terminated with running and queued tasks
- **THEN** a replacement sees the last committed states and does not release or restart work

### Requirement: Client compatibility and independent config
Existing task/topic command forms, output, errors, path precedence and separately committed chains SHALL remain compatible. Help, version and config SHALL neither start a daemon nor open the queue database. A client SHALL pass a validated policy snapshot for its typed operation.

#### Scenario: Config without owner
- **WHEN** config is changed or nested help is requested while no daemon runs
- **THEN** the operation succeeds independently and creates no queue database or endpoint

### Requirement: Native daemon verification
Native tests SHALL exercise client cancellation, owner interruption, singleton contention, stale endpoint recovery and blocked output on Linux, macOS and Windows without skip/ignore. Daemon and client SHALL require no Python, Node, bash or WSL at runtime.

#### Scenario: Owner interruption at commit
- **WHEN** native CI interrupts an owner before or after a queue commit
- **THEN** recovery observes the complete old or new state and no partial resource acquisition
