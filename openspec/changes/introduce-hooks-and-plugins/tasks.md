# Tasks

## 1. Queue events

- [x] 1.1 Define queue-owned event IDs/kinds/snapshots and injected timestamp/record ports; verify pure tests produce events for real transitions and none for failed/no-op operations.
- [x] 1.2 Record events in registration/link/start/completion/topic/metadata transactions, including shared automatic admission; verify a completion and every admitted successor commit or roll back together.
- [x] 1.3 Add SQLite outbox migration and event reading; verify stable ordered IDs, immutable separate task/topic snapshots, no synthetic historical starts and preserved ownership after interrupted upgrade.
- [x] 1.4 Document event payloads, order and post-commit semantics; verify examples against adapter fixtures and fake-clock tests.

## 2. Executable host and delivery

- [x] 2.1 Add plugin API/dispatch/process/persistence/composition crates and update graph checks; verify negative fixtures reject cross-feature impl dependencies and plugin database access.
- [x] 2.2 Implement versioned bounded stdio handshake/subscriptions/acks with isolated supervision; verify incompatible, hanging, crashing and output-flooding fixture plugins do not block queue clients.
- [x] 2.3 Persist registrations, future-event enable watermark and per-plugin delivery positions; verify independent consumers, retry backoff, lost ack, disable/re-enable and restart resume without changed event identity.
- [x] 2.4 Add plugin enable/list/status/disable CLI and English-only diagnostics; verify explicit enable is required and document executable trust, delivery behavior and error recovery with a runnable fixture plugin.

## 3. Capabilities, timers and managed work

- [x] 3.1 Implement narrow read/completion capabilities and transactional action receipts; verify conflicting ID reuse fails and a crash after queue commit cannot duplicate completion or successor events.
- [x] 3.2 Implement namespaced state/timers atomically with acknowledgement and injected clock policy; verify duplicate delivery, overdue restart, paused disable and unchanged original deadlines.
- [x] 3.3 Implement per-task managed-launch/completion serialization and typed cancellation/stopped/uncertain reports; verify a fixture worker prevents resource release until confirmed stopped, including manual completion.
- [x] 3.4 Add operator inspection and explicit confirmed-cleanup resolution; verify uncertain actions cannot auto-relaunch, spoof another plugin's receipt or release slots during disable/shutdown failure.
- [x] 3.5 Document protocol/capabilities/receipt and cancellation contracts; verify first-party examples use the public protocol and production frames contain no integration-test gates.

## 4. Integration acceptance

- [x] 4.1 Run mandatory local checks and native daemon/plugin crash, cancellation and interruption suites; record results without marking untested platform behavior complete.
- [x] 4.2 Open the hooks/plugins-only implementation PR and verify current CI passed for all supported OSes and required categories.
- [x] 4.3 Sync verified queue-events/plugin-runtime specs and verify strict validation while timeout and shell remain planned changes.

## 5. Storage contention review corrections

- [x] 5.1 Keep background pre-mutation lock acquisition retryable until shutdown; verify a 31-second transaction commits once, lifecycle remains responsive and stopping under contention cancels safely without replay or slot release.
- [x] 5.2 Verify current native Linux/macOS/Windows CI and synchronize the verified background-contention scenario without merging PRs.

## Workflow follow-up

- This is stage 3 and depends on the daemon and metadata implementations.
- Do not merge without the user's instruction. Archive after merge and verified baseline sync.
