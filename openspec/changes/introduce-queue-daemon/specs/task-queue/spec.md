# Spec Delta

## MODIFIED Requirements

### Requirement: Native coordination interface
Task and topic commands SHALL work natively through the local background queue daemon. The caller SHALL remain responsible for external work. Successful operations SHALL return 0 with a clear message; state and input errors SHALL return nonzero.

#### Scenario: External build
- **WHEN** the caller registers a task, links it to topics, obtains wait, performs a build, and calls complete
- **THEN** anyq coordinates slots while the caller launches and manages the build process

### Requirement: Blocking wait for own admission
`anyq task wait NAME` SHALL wait for the specified queued task's admission and return 0 only after it becomes running. Running SHALL succeed immediately; unknown tasks, missing topics, and completed SHALL fail without waiting. Wait SHALL only observe state, without performing admission or changing ownership. Waiting SHALL NOT launch commands or a task executor; the persistent queue owner SHALL serve observation requests independently of external work.

#### Scenario: No topics
- **WHEN** wait is called for a registered task without topics
- **THEN** CLI immediately reports `'my-task' is not registered in any topic` and returns nonzero

#### Scenario: Busy resource
- **WHEN** a queued task has not yet been admitted across all its topics
- **THEN** CLI reports `waiting for 'my-task' to start` and continues waiting

#### Scenario: Admission obtained
- **WHEN** a task atomically obtains admission in every topic
- **THEN** wait returns 0, after which the caller performs external work itself

#### Scenario: Two waits for one name
- **WHEN** two processes concurrently call wait for the same task
- **THEN** both succeed after the same transition to running; repeated wait also succeeds
