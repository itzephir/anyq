# Spec Delta

## MODIFIED Requirements

### Requirement: Explicit plugin registration
`plugin enable NAME` SHALL enable an installed binary from the user-wide plugin catalog for every active queue owner. `plugin list` SHALL show common installation and enablement; `plugin status NAME` and `plugin disable NAME` SHALL expose lifecycle and pending/error state in their documented scope. Enablement SHALL persist across daemon restart. Unknown metadata SHALL NOT enable plugins implicitly.

#### Scenario: Metadata without plugin
- **WHEN** a task contains shell metadata but shell has not been enabled
- **THEN** no command runs and queue ownership follows ordinary rules

#### Scenario: New owner after global enablement
- **WHEN** a custom queue is explicitly started after a plugin was globally enabled
- **THEN** that owner uses the enabled binary without another enable command

### Requirement: Durable per-plugin delivery
Each subscribed plugin SHALL have independent persisted delivery state for each queue identity. First enable for a queue SHALL begin with future committed events; restart SHALL resume its previous position. An event SHALL remain pending until acknowledged. Unacknowledged events SHALL be retried with bounded backoff and visible failure state, preserving identity.

#### Scenario: Independent consumers
- **WHEN** one plugin fails to acknowledge a running event and another succeeds
- **THEN** only the failed consumer retries that event and unrelated queue requests remain usable

#### Scenario: Equal task names in two databases
- **WHEN** two queues contain a task named build and use the same globally enabled plugin
- **THEN** their deliveries, state, timers, managed work and completion receipts remain independent

### Requirement: Existing-only background recovery
Starting a default owner automatically or a custom owner explicitly on a complete current queue SHALL resume globally enabled plugin delivery and timers without another queue request. Background recovery SHALL NOT create or migrate missing, uninitialized, legacy or incompatible custom storage. Such storage SHALL leave lifecycle and observation sessions available until ordinary queue initialization succeeds.

#### Scenario: Observation starts the owner after a deadline
- **WHEN** an observation client starts the current default owner of a complete schema-6 queue with a globally enabled plugin and overdue persisted timer
- **THEN** the original timer becomes eligible without another queue RPC and completion receipts and uncertain-work barriers still apply

#### Scenario: Observation of legacy or incomplete storage
- **WHEN** an explicitly started custom owner addresses missing, empty, schema-2/3/4/5 or incomplete schema-6 storage
- **THEN** background startup leaves selected storage unchanged and lifecycle and observation remain available without triggering creation, upgrade or repair

#### Scenario: Disabled recovery preserves state
- **WHEN** an owner starts after a globally disabled plugin's saved deadline
- **THEN** that plugin does not launch and its cursor, state, receipts and original deadline remain saved
