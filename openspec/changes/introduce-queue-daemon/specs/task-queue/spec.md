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

### Requirement: Automatic queue advancement
Successful registration with topics, link, complete (including force), and topic capacity/autostart changes SHALL admit all eligible queued tasks with autostart enabled in global registration order before commit. Admission SHALL be saved atomically with the original change. No active wait or periodic scheduling SHALL be required; persistence failure SHALL roll back the entire operation.

#### Scenario: Release without observer
- **WHEN** the owner of the only slot completes and the next queued task has no active wait
- **THEN** completion persists the owner as completed and its successor as running in one transaction

#### Scenario: Capacity increase
- **WHEN** a busy topic's capacity grows from one to three and three tasks wait for its slot
- **THEN** the two earliest eligible tasks become running and the third stays queued

#### Scenario: Remove a blocking predecessor
- **WHEN** force removes an early queued task that blocked a later task on a free topic because of another busy resource
- **THEN** the later task obtains full admission without wait

#### Scenario: Independent resources
- **WHEN** an early task waits for a busy topic and a later task needs only a different free topic
- **THEN** the later task becomes running while the earlier stays queued without partial acquisition

#### Scenario: Already running task
- **WHEN** wait is called for a running task once or repeatedly
- **THEN** every invocation immediately returns 0 without changing the task or slots
