# plugin-runtime Specification

## Purpose

Run explicitly enabled local extensions against committed events while retaining queue validation, durable progress and visible recovery behavior.

## Requirements

### Requirement: Explicit plugin registration
`plugin enable NAME EXECUTABLE [ARG...]` SHALL register and start a local executable with declared event subscriptions. `plugin list`, `plugin status NAME` and `plugin disable NAME` SHALL expose lifecycle and pending/error state. Registration SHALL persist across daemon restart. Unknown metadata SHALL NOT enable plugins implicitly.

#### Scenario: Metadata without plugin
- **WHEN** a task contains shell metadata but shell has not been enabled
- **THEN** no command runs and queue ownership follows ordinary rules

### Requirement: Versioned executable protocol
Plugins SHALL use a bounded versioned JSON-lines protocol with handshake, subscriptions, event deliveries, acknowledgements, capability requests and results. Incompatible, malformed or unresponsive plugins SHALL fail visibly without blocking the queue. Plugin stderr SHALL be diagnostic output, not protocol messages.

#### Scenario: Incompatible plugin
- **WHEN** a plugin announces an unsupported protocol version
- **THEN** status reports incompatibility and no event is treated as acknowledged

### Requirement: Durable per-plugin delivery
Each subscribed plugin SHALL have independent persisted delivery state. First enable SHALL begin with future committed events; restart SHALL resume its previous position. An event SHALL remain pending until acknowledged. Unacknowledged events SHALL be retried with bounded backoff and visible failure state, preserving identity.

#### Scenario: Independent consumers
- **WHEN** one plugin fails to acknowledge a running event and another succeeds
- **THEN** only the failed consumer retries that event and unrelated queue requests remain usable

### Requirement: Replay-safe plugin actions
Plugins SHALL receive event/action IDs usable as durable receipt keys. Queue capability requests SHALL deduplicate repeated action IDs and reject conflicting reuse. The runtime SHALL NOT claim exactly-once external side effects or automatically repeat an ambiguously started non-idempotent action.

#### Scenario: Lost completion response
- **WHEN** a plugin repeats the same completion action after losing its response
- **THEN** the original durable result is returned without duplicate completion or successor transitions

### Requirement: Narrow queue capabilities
Plugins SHALL access queue state and request completion through validated typed operations. They SHALL NOT receive database handles or bypass admission, canonical identity or completion invariants. Capability access SHALL be limited to the enabled plugin's declared needs.

#### Scenario: Invalid completion
- **WHEN** a plugin requests ordinary completion of a queued task
- **THEN** it receives a domain error and no task or resource changes

### Requirement: Persistent plugin state and timers
Enabled plugins SHALL have namespaced durable state and timers addressed by stable action IDs. Timer deadlines SHALL survive restart; overdue timers SHALL become eligible after recovery. No timer execution SHALL be promised while the daemon is stopped. Acknowledgement, state and requested timer changes SHALL commit together.

#### Scenario: Restart after deadline
- **WHEN** the daemon restarts after a saved plugin deadline
- **THEN** the plugin receives the original due timer without resetting its deadline

### Requirement: Existing-only background recovery
Starting an owner on a complete current queue SHALL resume enabled plugin delivery and timers without a queue request. Background recovery SHALL NOT create or migrate missing, uninitialized, legacy or incompatible storage. Such storage SHALL leave lifecycle and observation sessions available until ordinary queue initialization succeeds.

#### Scenario: Observation starts the owner after a deadline
- **WHEN** an observation client starts the owner of a complete schema-6 queue with an enabled plugin and an overdue persisted timer
- **THEN** the original timer becomes eligible without any queue RPC, and existing completion receipts and uncertain-work barriers still apply

#### Scenario: Observation of legacy or incomplete storage
- **WHEN** an owner starts on missing, empty, schema-2/3/4/5 or incomplete schema-6 storage
- **THEN** background startup leaves selected storage unchanged and lifecycle and observation remain available without triggering creation, upgrade or repair

#### Scenario: Disabled recovery preserves state
- **WHEN** an owner starts after a disabled plugin's saved deadline
- **THEN** that plugin does not launch and its cursor, state, receipts and original deadline remain saved

### Requirement: Managed execution completion barrier
A plugin SHALL be able to register managed execution against a running task before launching work. Completion requests SHALL require that execution to report stopped before releasing slots. Cancellation failure or uncertain execution state SHALL retain ownership and expose a recoverable error. Pending launch and completion SHALL be serialized for the same task.

#### Scenario: Completion races launch
- **WHEN** completion arrives while managed launch is pending
- **THEN** launch is prevented or execution is confirmed stopped before resource release

### Requirement: Disable and daemon shutdown
Disabling a plugin SHALL pause its deliveries and timers without deleting receipts or silently completing tasks. Disabling or stopping a plugin with managed execution SHALL first resolve cancellation; failure SHALL retain ownership and show the blocked execution. Daemon shutdown SHALL follow the same managed-work barrier.

#### Scenario: Disable active worker
- **WHEN** a managed worker cannot confirm process termination during disable
- **THEN** the task retains all slots and status explains why shutdown or disable remains unresolved

#### Scenario: Client transaction blocks background storage
- **WHEN** a live queue transaction holds the database beyond the ordinary lock timeout
- **THEN** the background host waits without terminating the daemon, retries only pre-mutation lock acquisition and stops waiting when owner shutdown requires cancellation without releasing committed task ownership

### Requirement: Observable bounded failures
Plugin status SHALL expose enabled/running state, delivery lag, last failure, active timers and unresolved actions. Payloads, response times and retained diagnostics SHALL be bounded. Stored metadata, credentials and script output SHALL not be copied into ordinary status or logs by default.

#### Scenario: Plugin floods output
- **WHEN** a plugin exceeds protocol or diagnostic limits
- **THEN** it is isolated with a visible error and other clients and plugins remain responsive
