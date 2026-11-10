# Spec Delta

## ADDED Requirements

### Requirement: Named task registry
`anyq task create NAME` SHALL create a durable task with a database-unique name and registered state. Registration SHALL NOT launch external work or acquire slots. Registering an existing name SHALL fail, including the name of a completed task.

#### Scenario: Registration
- **WHEN** a new name my-task is registered
- **THEN** CLI reports `successfully registered task 'my-task'` and the task exists without topics or acquired slots

#### Scenario: Duplicate name
- **WHEN** task create is called for an existing name
- **THEN** CLI returns an error and preserves the original task unchanged

### Requirement: Atomic registration with topics
`anyq task create NAME --topic TOPIC` and short form `-t TOPIC` SHALL register a task and link it to an existing topic in one transaction without acquiring slots. The flag SHALL allow repetition for multiple topics; duplicate links SHALL be idempotent. A result with topics SHALL be queued. Validation/persistence errors SHALL leave neither a task nor partial links.

#### Scenario: Several initial topics
- **WHEN** task create my-task --topic cpu -t repo -t cpu is called
- **THEN** one queued task is created with cpu/repo links and no acquired slots

#### Scenario: Unknown initial topic
- **WHEN** at least one topic specified at registration is absent and topic.autocreate is off
- **THEN** the command returns an error without saving the new task name or links

### Requirement: Topics as bounded resources
A topic SHALL have a unique name and positive capacity, default one slot. `anyq topic list`, `anyq topic --list`, and `anyq topic -l` SHALL list names. `topic create TOPIC [--capacity N]` SHALL support `-c N`. Capacity SHALL change through positional `topic capacity TOPIC N`; reductions below running usage SHALL fail. Old `topic capacity TOPIC --capacity N` SHALL be rejected.

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
Shared domain policy SHALL require an ASCII Latin initial for bare names and allow any tail. Bare list/create/capacity/link/add/wait/complete/config/help SHALL be forbidden. One literal outer backtick pair SHALL allow any content, including empty strings, commands, and leading hyphens, and be stripped exactly once. CLI/SQL SHALL NOT duplicate these rules.

#### Scenario: Invalid bare name
- **WHEN** Topic or Task is created with bare create, --cpu, or a non-Latin initial
- **THEN** the domain rejects the name without changing the aggregate

#### Scenario: Arbitrary quoted name
- **WHEN** the entire name is wrapped in literal backticks
- **THEN** any inner characters, commands, and empty content are allowed

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
`anyq topic link TOPIC NAME` and `anyq task link NAME TOPIC` SHALL equivalently add an existing task to an existing topic through AddTaskToTopic, transitioning registered to queued. A task SHALL support multiple distinct topics. Duplicate links SHALL be idempotent; running/completed membership SHALL NOT change. Old `topic TOPIC add NAME` SHALL be rejected.

#### Scenario: Equivalent link forms
- **WHEN** a link is created through topic link cpu my-task and repeated through task link my-task cpu
- **THEN** one link is stored without acquiring a slot, and both commands report `successfully added 'my-task' to topic 'cpu'`

#### Scenario: Two resources
- **WHEN** my-task is added to itzdesign-commands and itzdesign-repo-lock
- **THEN** both links are stored without acquiring slots, and CLI reports `successfully added 'my-task' to topic 'TOPIC'` for each

#### Scenario: Unknown task or topic
- **WHEN** an add command references a missing task or topic name
- **THEN** CLI returns an error without implicitly creating a task or topic

#### Scenario: Adding during execution
- **WHEN** another topic is added to a running task
- **THEN** the command is rejected and the acquired resource set is preserved

### Requirement: Blocking wait for own admission
`anyq task wait NAME` SHALL wait for the specified queued task's admission and return 0 only after running. Unknown tasks, no topics, running, and completed SHALL fail without waiting. Waiting SHALL NOT launch commands or a separate task executor.

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
- **THEN** exactly one obtains new admission and the other reports an already-running error

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
A running task SHALL hold one slot from every attached topic until explicit complete. Wait return, caller exit/crash, CLI restart, and elapsed time SHALL NOT release ownership. Heartbeat, leases, and automatic timeout-based slot recovery SHALL be absent.

#### Scenario: Wait process exited
- **WHEN** wait successfully returned admission and its process exited
- **THEN** the task remains running and holds all its slots

#### Scenario: External work crashed
- **WHEN** the caller process dies after admission
- **THEN** slots remain acquired until explicit task completion

### Requirement: Ordinary completion of a running task
`anyq task complete NAME` SHALL complete only running tasks, atomically transitioning to completed and releasing all slots. Registered/queued SHALL report `'NAME' was not run`. Unknown/already completed tasks SHALL fail without state changes.

#### Scenario: Completion
- **WHEN** complete is called for running task my-task
- **THEN** CLI reports `'my-task' successfully completed` and all its resources are released

#### Scenario: Task never ran
- **WHEN** complete is called for registered or queued task my-task without --force
- **THEN** CLI reports `'my-task' was not run`, returns nonzero, and preserves state

### Requirement: Forced completion
`anyq task complete --force NAME` and short `-f` SHALL allow registered/queued/running to become completed. Unstarted tasks SHALL leave queues without external execution; running tasks SHALL release all slots. Missing names SHALL be created only with task.autocreate enabled; completed SHALL NOT restart.

#### Scenario: Unstarted task
- **WHEN** complete --force is called for queued task my-task
- **THEN** CLI reports `'my-task' successfully completed`, the task becomes completed, and it no longer blocks queues

#### Scenario: Force during wait
- **WHEN** a task is completed with --force before the waiting wait obtains admission
- **THEN** the waiting wait returns an error without authorizing external work

### Requirement: Durability and transition from the old database
Names, topic links, order, and states SHALL survive opening in a new process. The former message/group database SHALL be rejected with a clear incompatible-schema message recommending a separate file through --db. Its data SHALL NOT be deleted or converted automatically.

#### Scenario: Restart
- **WHEN** a new process opens a database containing queued, running, and completed tasks
- **THEN** it sees the previous states, order, topic membership, and acquired slots

#### Scenario: Old database
- **WHEN** the new CLI opens a message database with schema version 1
- **THEN** the operation fails, recommends a separate database, and preserves previous data

### Requirement: Native coordination interface
Task/topic commands SHALL work natively without a Python launcher, daemon, command-list payload, or consumer groups. Success SHALL return 0 with a clear message; state/input errors return nonzero. Old publish/claim/ack/nack/renew/retry/jobs/group/worker SHALL NOT remain a parallel API.

#### Scenario: External build
- **WHEN** the caller registers a task, links it to topics, obtains wait, performs a build, and calls complete
- **THEN** anyq coordinates slots while the caller launches and manages the build process

### Requirement: Help and version without IO
Flags -v and -h SHALL be equivalent to --version and --help; help [command] SHALL show help, including nested commands. These forms SHALL NOT open config or the database.

#### Scenario: Nested help
- **WHEN** help task link is called with damaged config
- **THEN** output equals task link --help and the database is not opened

### Requirement: Equivalent action ordering
Named-object commands SHALL support command-first/name-first: topic link TOPIC NAME = topic TOPIC link NAME, task link NAME TOPIC = task NAME link TOPIC, topic TOPIC create/capacity, task NAME create/wait/complete. Names, --db, and -- SHALL be preserved. Task add SHALL be rejected.

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

## REMOVED Requirements

### Requirement: Topics and durable jobs
**Reason**: Named-task registration replaces JSON publication and idempotency keys.
**Migration**: Create a separate new-model database, register through task create, and link topics.

### Requirement: Independent consumer groups
**Reason**: A task has one lifecycle and shared admission, without group-specific processing copies.
**Migration**: Use a named task and task wait; old database groups are not imported automatically.

### Requirement: Competing consumers and ordering
**Reason**: Admission of one task across all topics replaces claiming one group's delivery.
**Migration**: Attach every required topic and use the global registration order.

### Requirement: Lease and fencing
**Reason**: The user chose persistent ownership until explicit complete instead of lease expiry.
**Migration**: Complete tasks explicitly; after a crash, inspect external work before complete.

### Requirement: Retries and observability
**Reason**: anyq no longer executes commands, evaluates exit codes, or retries automatically.
**Migration**: The caller handles external work outcomes; register new execution under a new name.

### Requirement: Atomic follow-up stage
**Reason**: The core contract is registration, admission, and completion of a named task.
**Migration**: The caller registers follow-up stages rather than worker publishing with ack.
