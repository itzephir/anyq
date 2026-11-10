# task-queue Specification

## Purpose

A durable local admission queue coordinates named tasks belonging to people, agents, and processes that need slots from several resource topics simultaneously.

## Requirements

### Requirement: Named task registry
`anyq task create NAME` SHALL create a durable task with a database-unique name and registered state. Registration SHALL NOT launch external work or acquire slots. Registering an existing name SHALL fail, including the name of a completed task.

#### Scenario: Registration
- **WHEN** a new name my-task is registered
- **THEN** CLI reports `successfully registered task 'my-task'` and the task exists without topics or acquired slots

#### Scenario: Duplicate name
- **WHEN** task create is called for an existing name
- **THEN** CLI returns an error and preserves the original task unchanged

### Requirement: Atomic registration with topics
`anyq task create NAME --topic TOPIC` and short form `-t TOPIC` SHALL register a task and link it to an existing topic in one transaction, with automatic admission after saving all links. The flag SHALL allow repetition for several topics; duplicate links SHALL be idempotent. A result with topics SHALL be running when automatic admission is enabled and full admission is available, otherwise queued. Validation or persistence errors SHALL leave neither a task nor partial links or metadata.

#### Scenario: Several initial topics
- **WHEN** task create my-task --topic cpu -t repo -t cpu is called
- **THEN** one task is created with two links, cpu and repo; with free resources it is immediately running and holds one slot from each topic

#### Scenario: Unknown initial topic
- **WHEN** at least one topic specified at registration is absent and topic.autocreate is off
- **THEN** the command returns an error without saving the new task name or links

#### Scenario: Metadata before first admission
- **WHEN** creation supplies metadata and initial topics
- **THEN** metadata, registration and all links are saved in the same transaction before automatic admission

### Requirement: Topics as bounded resources
A topic SHALL have a unique name and positive capacity, defaulting to one slot. `anyq topic list`, `anyq topic --list`, and `anyq topic -l` SHALL list topic names. `topic create TOPIC [--capacity N]` SHALL support short flag `-c N`. Capacity SHALL be changed through `topic capacity TOPIC N` with a positional number; reducing it below the number of running tasks SHALL be rejected.

#### Scenario: Topic list
- **WHEN** itzdesign-commands, pixel-10-emulator, and itzdesign-repo-lock exist
- **THEN** topic --list lists these names

#### Scenario: Changing occupied capacity
- **WHEN** reducing capacity would leave more running tasks than slots
- **THEN** the change is rejected and existing admissions are preserved

### Requirement: List all tasks
`anyq task list`, `anyq task --list`, and `anyq task -l` SHALL return all registered names, including completed tasks, alphabetically in the topic-list format without changing states or slots.

#### Scenario: Completed name in list
- **WHEN** task -l is called after a task completes
- **THEN** the name remains listed and resource ownership is unchanged

### Requirement: Task state lookup
`anyq task NAME` SHALL show task state and stored topics without changing state or slots. An unknown name SHALL produce an error with autocreate off; with it enabled, a registered task without slots SHALL be created.

#### Scenario: Completed task state
- **WHEN** task NAME is called for a completed task
- **THEN** the lookup shows completed and stored topics without acquiring slots

### Requirement: Current queue lookup
`anyq topic TOPIC` SHALL show capacity and current queued/running tasks in registration order without changing states or slots. An unknown topic SHALL produce an error with autocreate off.

#### Scenario: Current queue
- **WHEN** a topic contains running, queued, and completed tasks
- **THEN** the lookup shows only running and queued in registration order, preserving their states and ownership

### Requirement: Domain naming for tasks and topics
The shared domain policy SHALL require an ASCII Latin initial for bare names and allow any tail. Bare list/create/capacity/link/add/start/autostart/wait/complete/meta/metafile/config/help SHALL be forbidden. One literal outer pair of backticks SHALL allow any content, including empty strings, commands, and leading hyphens, and be removed exactly once. CLI and SQL SHALL NOT duplicate these rules.

#### Scenario: Invalid bare name
- **WHEN** Topic or Task is created with bare create, --cpu, or a non-Latin initial
- **THEN** the domain rejects the name without changing the aggregate

#### Scenario: Arbitrary quoted name
- **WHEN** the entire name is wrapped in literal backticks
- **THEN** any inner characters, commands, and empty content are allowed

#### Scenario: Previously valid command-like name
- **WHEN** an existing canonical name is meta or metafile
- **THEN** its backtick-quoted spelling still addresses it and restore does not reinterpret its identity

### Requirement: Canonical name identity
Bare build and backtick-wrapped build SHALL identify one entity. Persistence and restore SHALL use canonical strings without stripping backticks again. Display SHALL return a reversible spelling. Shell escaping of spaces SHALL NOT restrict the allowed name tail; inner backslashes SHALL remain literal.

#### Scenario: Nested backticks
- **WHEN** a name containing inner backticks is saved and read by a new process
- **THEN** its content and identity remain unchanged during restore

### Requirement: Read capacity
`anyq topic capacity TOPIC` without a number SHALL show the topic's current capacity without changing it. An unknown topic SHALL produce an error with autocreate off.

#### Scenario: Read after change
- **WHEN** topic capacity cpu is called after topic capacity cpu 2
- **THEN** the command shows 2 without changing the queue or ownership

### Requirement: Shorthand registration sequence
`anyq task create NAME --topic TOPIC --wait --complete` SHALL sequentially register the task with links, wait for admission, and complete it. Short flags SHALL be -t, -w, and -c; -twc TOPIC SHALL produce the same result. Wait and complete steps SHALL be optional. A wait error SHALL stop the chain without reporting successful admission or completion.

#### Scenario: Full chain
- **WHEN** task create my-task -twc cpu obtains a free slot
- **THEN** the task becomes queued, running, and completed in sequence, and the slot is released

#### Scenario: Wait without automatic completion
- **WHEN** task create my-task -tw cpu obtains admission
- **THEN** the command returns 0 and the task remains running until a separate complete

#### Scenario: Waiting chain completed by another process
- **WHEN** a queued task is force-completed while a chain with --complete is waiting
- **THEN** the chain returns an error, reports neither admission nor its own completion, and leaves the busy resource's owner untouched

### Requirement: Registration flag dependencies
For task create, --wait/-w SHALL require at least one --topic/-t, and --complete/-c SHALL require --wait/-w. Invalid combinations SHALL be rejected before opening the database or registering a task.

#### Scenario: Required preceding flag missing
- **WHEN** wait is specified without topic or complete without wait
- **THEN** CLI returns an error without registering a task or opening the database

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

### Requirement: Atomic admission to all resources
A task SHALL become running only when one slot from every attached topic is available. Availability checks, topic membership, and the state transition SHALL form one atomic operation. A task without full admission SHALL NOT hold partial resources; no successful admission SHALL exceed topic capacity.

#### Scenario: One of two topics busy
- **WHEN** itzdesign-commands is free but itzdesign-repo-lock is busy
- **THEN** the task stays queued without holding a slot in either topic

#### Scenario: Competing processes
- **WHEN** two processes compete for a shared topic's last slot through different tasks
- **THEN** only one task becomes running

#### Scenario: Persistence failure
- **WHEN** saving admission fails
- **THEN** task state and availability of all resources remain unchanged

### Requirement: Global task order
Queued order across all of a task's topics SHALL be determined by one monotonic task registration sequence. At admission, the task SHALL be among the first queued tasks within the number of free slots in every topic. Adding a link later SHALL NOT create opposite ordering of the same task pair in different topics.

#### Scenario: Crossed linking
- **WHEN** A is registered before B and both tasks are added to two topics in different sequences
- **THEN** A precedes B in both queues and no mutual partial acquisition occurs

### Requirement: Persistent ownership until completion
A running task SHALL hold one slot from every attached topic until explicit complete. Returning from wait, caller exit or crash, CLI restart, and elapsed time SHALL NOT release ownership.

#### Scenario: Wait process exited
- **WHEN** wait successfully returned admission and its process exited
- **THEN** the task remains running and holds all its slots

#### Scenario: External work crashed
- **WHEN** the caller process dies after admission
- **THEN** slots remain acquired until explicit task completion

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

### Requirement: Database durability and compatibility
Names, topic links, order, and states SHALL survive opening the database in a new process. A database with an unsupported schema version, or a nonempty unversioned database, SHALL be rejected before changing its data or journal mode, with an incompatibility message recommending a separate file through --db.

#### Scenario: Restart
- **WHEN** a new process opens a database containing queued, running, and completed tasks
- **THEN** it sees the previous states, order, topic membership, and acquired slots

#### Scenario: Incompatible database
- **WHEN** CLI opens a database with an unsupported schema version or a nonempty unversioned database
- **THEN** the operation fails and recommends a separate database, original bytes are preserved, and journal files are not created

### Requirement: Native coordination interface
Task and topic commands SHALL work natively through the local background queue daemon. The caller SHALL remain responsible for external work. Successful operations SHALL return 0 with a clear message; state and input errors SHALL return nonzero.

#### Scenario: External build
- **WHEN** the caller registers a task, links it to topics, obtains wait, performs a build, and calls complete
- **THEN** anyq coordinates slots while the caller launches and manages the build process

### Requirement: Help and version without IO
Flags -v and -h SHALL be equivalent to --version and --help; help [command] SHALL show help, including nested commands. These forms SHALL NOT open config or the database.

#### Scenario: Nested help
- **WHEN** help task link is called with damaged config
- **THEN** output equals task link --help and the database is not opened

### Requirement: Equivalent action ordering
Commands with named objects SHALL support command-first and name-first forms: topic link TOPIC NAME = topic TOPIC link NAME, task link NAME TOPIC = task NAME link TOPIC, topic TOPIC create/capacity, task NAME create/wait/complete. Name values, --db, and -- SHALL be preserved.

#### Scenario: Name-first creation
- **WHEN** task job create -twc cpu is called
- **THEN** the result equals task create job -twc cpu

### Requirement: Link and wait chains
Both link forms SHALL accept --wait/-w and --complete/-c; complete SHALL require wait. Task wait SHALL accept --complete/-c. Steps SHALL invoke separate use cases sequentially and stop on error. Without complete, successful admission SHALL leave running until explicit completion.

#### Scenario: Full link
- **WHEN** task job link cpu -wc obtains a free resource
- **THEN** the link is saved, the task starts, and it immediately completes

#### Scenario: Invalid link
- **WHEN** complete is specified for link without wait
- **THEN** CLI returns an error before opening the database

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
Versions 2/3/4 SHALL upgrade to schema 5 atomically, preserving names, registration, links, states, metadata and ownership. Queue events, receipts, managed work and plugin storage SHALL share one schema transaction, without admission or historical running events. Versions 2/3 SHALL receive empty metadata; version 2 SHALL receive task autostart=true/stopped=false with inheriting topics. Interruption SHALL leave the old schema before commit or the complete new schema afterward.


#### Scenario: Existing ownership
- **WHEN** a version 2 database with a running owner and queued successor is opened
- **THEN** after upgrade the owner holds the same slots, and the successor remains queued until the owner's explicit complete

#### Scenario: Existing schema 3
- **WHEN** a schema 3 database with running and queued tasks is upgraded
- **THEN** metadata defaults to empty objects and all states, order and ownership survive

#### Scenario: Metadata schema with active ownership
- **WHEN** a schema 4 database containing running tasks and metadata is upgraded
- **THEN** schema 5 preserves the metadata and all ownership without synthetic running events

#### Scenario: Interrupted feature migration
- **WHEN** initialization fails or is interrupted before schema commit
- **THEN** outbox, receipts, managed work and plugin storage are all rolled back with the queue schema
