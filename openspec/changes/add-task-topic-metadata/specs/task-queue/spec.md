# Spec Delta

## MODIFIED Requirements

### Requirement: Atomic registration with topics
`anyq task create NAME --topic TOPIC` and short form `-t TOPIC` SHALL register a task and link it to an existing topic in one transaction, with automatic admission after saving all links. The flag SHALL allow repetition for several topics; duplicate links SHALL be idempotent. A result with topics SHALL be running when automatic admission is enabled and full admission is available, otherwise queued. Creation-time metadata SHALL be saved with registration and links before admission. Validation or persistence errors SHALL leave neither a task nor partial links or metadata.

#### Scenario: Several initial topics
- **WHEN** task create my-task --topic cpu -t repo -t cpu is called
- **THEN** one task is created with two links, cpu and repo; with free resources it is immediately running and holds one slot from each topic

#### Scenario: Unknown initial topic
- **WHEN** at least one topic specified at registration is absent and topic.autocreate is off
- **THEN** the command returns an error without saving the new task name or links

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
