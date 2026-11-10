## MODIFIED Requirements

### Requirement: Atomic registration with topics
`anyq task create NAME --topic TOPIC` and short `-t TOPIC` SHALL register/link an existing topic in one transaction with automatic admission after all links are saved. The flag SHALL allow multiple topics; duplicate links SHALL be idempotent. Tasks with topics SHALL be running when autostart and full admission allow, otherwise queued. Validation/persistence errors SHALL leave neither a task nor partial links.

#### Scenario: Several initial topics
- **WHEN** task create my-task --topic cpu -t repo -t cpu is called
- **THEN** one task is created with two links, cpu and repo; with free resources it is immediately running and holds one slot from each topic

#### Scenario: Unknown initial topic
- **WHEN** at least one topic specified at registration is absent and topic.autocreate is off
- **THEN** the command returns an error without saving the new task name or links

### Requirement: Task membership in several topics
`anyq topic link TOPIC NAME` and `anyq task link NAME TOPIC` SHALL equivalently add an existing task to an existing topic, transition registered to queued, and automatically check admission of all queued tasks through one AddTaskToTopic use case. One task SHALL support several distinct topics. Repeating a queued task's link SHALL be idempotent; running/completed membership SHALL NOT change.

#### Scenario: Equivalent link forms
- **WHEN** cpu is busy and a link is created through topic link cpu my-task then repeated through task link my-task cpu
- **THEN** one queued link is stored without acquiring a slot, and both commands report `successfully added 'my-task' to topic 'cpu'`

#### Scenario: Two resources
- **WHEN** my-task is added to itzdesign-commands and itzdesign-repo-lock while resources are busy
- **THEN** both links are stored without acquiring slots, and CLI reports `successfully added 'my-task' to topic 'TOPIC'` for each

#### Scenario: Unknown task or topic
- **WHEN** an add command references a missing task or topic name
- **THEN** CLI returns an error without implicitly creating a task or topic

#### Scenario: Adding during execution
- **WHEN** another topic is added to a running task
- **THEN** the command is rejected and the acquired resource set is preserved

### Requirement: Blocking wait for own admission
`anyq task wait NAME` SHALL wait for the specified queued task's admission and return 0 only after it becomes running. Running SHALL succeed immediately; unknown tasks, missing topics, and completed SHALL fail without waiting. Wait SHALL only observe state, without performing admission or changing ownership. Waiting SHALL NOT launch commands or a separate task executor.

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

### Requirement: Ordinary completion of a running task
`anyq task complete NAME` SHALL complete only a running task, atomically transitioning it to completed, releasing all its slots, and automatically admitting eligible successors in the same transaction. Registered or queued tasks SHALL report `'NAME' was not run`. Unknown or already completed tasks SHALL produce an error without state changes.

#### Scenario: Completion
- **WHEN** complete is called for running task my-task
- **THEN** CLI reports `'my-task' successfully completed`, previous ownership is released, and eligible queued successors are already running without wait

#### Scenario: Task never ran
- **WHEN** complete is called for registered or queued task my-task without --force
- **THEN** CLI reports `'my-task' was not run`, returns nonzero, and preserves state

### Requirement: Forced completion
`anyq task complete --force NAME` and short form `-f` SHALL allow registered, queued, and running tasks to become completed. For an unstarted task, the operation SHALL remove it from queues without external execution; for running it SHALL release all slots. A missing name SHALL be created only with task.autocreate enabled; completed SHALL NOT restart. Removing a predecessor SHALL automatically admit newly eligible tasks in the same transaction.

#### Scenario: Unstarted task
- **WHEN** complete --force is called for queued task my-task
- **THEN** CLI reports `'my-task' successfully completed`, the task becomes completed, and it no longer blocks queues

#### Scenario: Force during wait
- **WHEN** a task is completed with --force before the waiting wait obtains admission
- **THEN** the waiting wait returns an error without authorizing external work

### Requirement: Domain naming for tasks and topics
The shared domain policy SHALL require an ASCII Latin initial for bare names and allow any tail. Bare list/create/capacity/link/add/start/autostart/wait/complete/config/help SHALL be forbidden. One literal outer pair of backticks SHALL allow any content, including empty strings, commands, and leading hyphens, and be removed exactly once. CLI and SQL SHALL NOT duplicate these rules.

#### Scenario: Invalid bare name
- **WHEN** Topic or Task is created with bare create, --cpu, or a non-Latin initial
- **THEN** the domain rejects the name without changing the aggregate

#### Scenario: Arbitrary quoted name
- **WHEN** the entire name is wrapped in literal backticks
- **THEN** any inner characters, commands, and empty content are allowed

## ADDED Requirements

### Requirement: Automatic queue advancement
Successful registration with topics, link, complete (including force), and topic capacity/autostart changes SHALL admit all eligible queued tasks with autostart enabled in global registration order before commit. Admission SHALL be saved atomically with the original change. Neither active wait nor a background process SHALL be required; persistence failure SHALL roll back the entire operation.

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

### Requirement: Atomic batch linking
`anyq topic link TOPIC NAME...` SHALL link one or more tasks to a topic, and `anyq task link NAME TOPIC...` SHALL link a task to one or more topics. The entire batch SHALL be saved before automatic admission. Any name, transition, or persistence error SHALL roll back all links and autocreate. Duplicates SHALL NOT create repeated links. Name-first forms SHALL be equivalent.

#### Scenario: Several resources for one task
- **WHEN** task link build cpu repo runs with free cpu and repo
- **THEN** both links are saved together and build obtains full admission to both resources

#### Scenario: Several tasks for one topic
- **WHEN** topic link cpu third first second links registered first, second, third to single-slot cpu
- **THEN** all links are saved, first becomes running in registration order, and the others stay queued

#### Scenario: Error at end of batch
- **WHEN** the last batch participant is absent with autocreate off or is already running
- **THEN** no new batch links are saved and no task obtains new admission

#### Scenario: Full batch chain
- **WHEN** topic link cpu second first -wc runs for tasks first and second registered in that order
- **THEN** the entire batch is saved first, then first and second are awaited and completed in registration order

### Requirement: Automatic start precedence
A task SHALL store the task.autostart default at creation. An unconfigured topic SHALL inherit this default; `topic TOPIC autostart [on/off]` SHALL set an override, defaulting to on when omitted. Autostart SHALL require on for all task topics and no explicit stop. Topic setting changes SHALL atomically reconsider queued tasks without stopping running tasks.

#### Scenario: Topic overrides default
- **WHEN** the task default is false but its only topic has autostart on
- **THEN** the task automatically becomes running when resources are available

#### Scenario: One topic disallows autostart
- **WHEN** at least one task topic has autostart off
- **THEN** automatic admission does not occur; explicit start remains possible under the shared resource and queue rules

### Requirement: Start control during registration
Create SHALL accept mutually exclusive --start, --stop, and -s. Stop SHALL persist the automatic-start prohibition until explicit start. Start with autostart enabled SHALL do nothing extra; otherwise it SHALL require immediate full admission and roll back registration on failure. -s SHALL invert effective permission. Cluster -tswc TOPIC SHALL retain the meaning of individual flags.

#### Scenario: Persistent stop
- **WHEN** a task is created with --stop and resources are later released or topic autostart is enabled
- **THEN** it stays queued until explicit task start

#### Scenario: Cluster with autostart enabled
- **WHEN** create NAME -tswc TOPIC runs with autostart on
- **THEN** the task is saved stopped/queued, the chain waits for a separate start, and it completes the task after observing running

#### Scenario: Cluster with autostart disabled
- **WHEN** create NAME -tswc TOPIC runs with autostart off and a free slot
- **THEN** the task starts immediately, wait observes running, and complete finishes it

### Requirement: Explicit start and chains
`task start NAME` and `task NAME start` SHALL immediately attempt full admission regardless of autostart; with unavailable resources or queue position, CLI SHALL return an error without changes. -w/--wait SHALL retry until admission; -c/--complete SHALL require wait and then complete the task. Running SHALL report already running and succeed; registered without topics and completed SHALL produce errors.

#### Scenario: Busy resource
- **WHEN** start without -w is called for a queued task with an unavailable topic
- **THEN** the command returns an error, preserving queued with no partial ownership

#### Scenario: Waiting explicit start
- **WHEN** task NAME start -wc waits for a resource and the owner releases it
- **THEN** the task obtains all slots and then completes; resources are released or passed to eligible successors

### Requirement: Supported schema upgrade
A version 2 database SHALL upgrade to 3 atomically, preserving names, registration, links, states, and ownership. Existing tasks SHALL receive default autostart=true and stopped=false; topics SHALL inherit the default. Database open SHALL NOT perform admission. Interruption before commit SHALL preserve the previous schema, and afterward the complete new schema.

#### Scenario: Existing ownership
- **WHEN** a version 2 database with a running owner and queued successor is opened
- **THEN** after upgrade the owner holds the same slots, and the successor remains queued until the owner's explicit complete
