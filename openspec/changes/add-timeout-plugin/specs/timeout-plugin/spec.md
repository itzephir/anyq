# Spec Delta

## Purpose

Apply explicit metadata-driven ownership deadlines through the plugin interface without adding elapsed-time behavior to queue admission.

## ADDED Requirements

### Requirement: Explicit timeout policy
The enabled timeout plugin SHALL consume timeout strings from a running event's task and separate topic metadata snapshots. It SHALL choose the shortest configured positive duration. Missing values SHALL add no deadline. Invalid configured values SHALL produce a visible plugin error without releasing ownership.

#### Scenario: Several topics
- **WHEN** the task timeout is 2h and its topics specify 30m and no timeout
- **THEN** the effective timeout is 30 minutes

### Requirement: Unambiguous bounded durations
Timeout values SHALL be positive integer strings with one suffix from ms, s, m, h or d. Zero, negative, fractional, missing-unit and overflowing values SHALL be rejected. Conversion SHALL use checked arithmetic and preserve the original value for diagnostics without changing metadata.

#### Scenario: Overflow
- **WHEN** a configured duration cannot be represented as a persisted deadline
- **THEN** status reports invalid timeout and the task remains running with its resources

### Requirement: Deadline anchored to admission
The deadline SHALL equal the original task.running timestamp plus the effective duration. Event delivery delay, retries, topic edits and daemon restart SHALL NOT extend it. One event identity SHALL produce at most one logical persisted timer.

#### Scenario: Delayed delivery
- **WHEN** a 2h timeout event is first processed three hours after admission
- **THEN** the persisted deadline remains two hours after admission and expiry is immediately eligible

### Requirement: Expiry through validated completion
At expiry the plugin SHALL request ordinary completion of the same running task through the daemon. Successful completion SHALL release all slots and admit eligible successors atomically. Already completed tasks SHALL be treated as resolved; queued/registered tasks SHALL NOT be force-completed by timeout.

#### Scenario: Manual completion wins
- **WHEN** a user completes the task before its timeout action arrives
- **THEN** timeout records a resolved action without error, new transitions or effects on successors

### Requirement: Recovery and disable semantics
Saved deadlines SHALL survive daemon/plugin restart and be processed when overdue after recovery. Completion events SHALL retire corresponding timers. Disabling timeout SHALL pause its actions; re-enabling SHALL retain the original deadline. Time alone SHALL NOT act while the daemon is stopped or the plugin is disabled.

#### Scenario: Disabled past deadline
- **WHEN** timeout is re-enabled after a saved deadline has passed
- **THEN** the original timer becomes due rather than starting a fresh duration

### Requirement: Managed work cancellation before release
Timeout completion SHALL use the managed-execution barrier. Slots SHALL remain held until managed work is confirmed stopped. Unmanaged external work SHALL not be claimed stopped; timeout documentation SHALL identify its policy as resource release for that case.

#### Scenario: Failed cancellation
- **WHEN** a managed worker cannot confirm termination at expiry
- **THEN** timeout remains visibly unresolved and all task resources remain held
