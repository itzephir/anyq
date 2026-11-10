# Tasks

## 1. Timeout policy

- [x] 1.1 Add pure timeout duration/precedence/deadline types; verify missing values, minimum across task/topics, supported suffixes, overflow, zero, negatives and invalid types without clocks or IO.
- [x] 1.2 Implement running/completed/timer handlers through plugin contracts; verify the same event always produces one logical timer anchored to its original occurrence timestamp.
- [x] 1.3 Document opt-in metadata and unmanaged-work resource-release semantics; verify 2h/30m examples and disabled-plugin behavior with fixture events.

## 2. Runtime integration and recovery

- [x] 2.1 Expose the first-party executable plugin mode through the ordinary versioned protocol; verify registration, subscriptions and English-only status work without direct queue/database access.
- [x] 2.2 Persist deadline/state acknowledgements and replay-safe completion actions; verify delayed delivery, restart-after-deadline and disable/re-enable never extend the deadline.
- [x] 2.3 Integrate the common managed-execution completion barrier; verify a fixture worker's failed cancellation retains slots and confirmed stop permits atomic release/admission.
- [x] 2.4 Add real-client tests for manual completion/expiry races and completion timer retirement; verify one logical completion and no forced removal of queued/registered tasks.
- [x] 2.5 Add runnable daemon/metadata/timeout examples and recovery instructions; verify commands against an isolated database with short real deadlines.

## 3. Integration acceptance

- [x] 3.1 Run mandatory local checks plus native expiry/restart/cancellation integration tests and record actual results.
- [x] 3.2 Open the timeout-only implementation PR and verify current CI passed on Linux/macOS/Windows and all required categories.
- [x] 3.3 Sync verified timeout ownership policy and capability specs; verify strict validation and unchanged no-timeout caller workflows.

## Workflow follow-up

- This is stage 4 and depends on introduce-hooks-and-plugins.
- Do not merge without the user's instruction. Archive after merge and verified baseline sync.
