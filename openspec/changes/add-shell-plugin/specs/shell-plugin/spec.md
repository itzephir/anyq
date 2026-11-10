# Spec Delta

## Purpose

Execute explicitly configured task commands or scripts after admission, preserving durable outcomes and coordinating process lifetime with resource ownership.

## ADDED Requirements

### Requirement: Explicit shell plan
An enabled shell plugin SHALL interpret only the running task snapshot's shell object. Exactly one nonempty array of nonempty strings, sh or src, SHALL be required; autocomplete SHALL be a boolean defaulting to false. Topic metadata SHALL NOT inject commands. Invalid plans SHALL fail visibly without launching work or releasing slots.

#### Scenario: Conflicting execution sources
- **WHEN** shell metadata contains both sh and src
- **THEN** the plugin records a validation failure and executes neither

### Requirement: Sequential native execution
The plugin SHALL run sh entries through an explicitly resolved native interpreter or execute src paths as native programs. Entries SHALL run in order and stop after the first launch failure or nonzero exit. Interpreter, arguments and working directory SHALL be recorded before launch. Paths SHALL NOT be implicitly shell-expanded.

#### Scenario: Second command fails
- **WHEN** the second of three sh entries returns nonzero
- **THEN** the first two outcomes are recorded and the third entry is not launched

### Requirement: Stable execution context
Task creation with shell metadata SHALL capture an absolute working directory, defaulting to the creating client's directory. Relative src paths SHALL resolve against it. Execution SHALL NOT depend on the daemon's own current directory. Environment inheritance and explicit overrides SHALL be documented and credentials SHALL NOT enter ordinary diagnostics.

#### Scenario: Client exits from another directory
- **WHEN** a task created in a project directory starts after its client exits
- **THEN** its relative script paths and commands use the captured project directory

### Requirement: Managed launch and recovery receipts
The plugin SHALL register managed work and persist launch intent before spawning. Duplicate event delivery SHALL NOT relaunch a recorded execution. If recovery cannot establish whether work launched or remains alive, it SHALL expose an uncertain state, retain slots, and require explicit resolution before another launch or completion.

#### Scenario: Crash around spawn
- **WHEN** the plugin crashes after launch intent but before a durable launch result
- **THEN** recovery does not blindly rerun the script or release the task's resources

### Requirement: Outcome and output inspection
Execution inspection SHALL show planned/running/exited/cancelled/uncertain status, entry outcomes and bounded stdout/stderr logs. A nonzero exit SHALL remain distinguishable from successful execution even if the task becomes completed. Task completed SHALL continue to mean released resource ownership.

#### Scenario: Failed command inspected
- **WHEN** a command exits with code 7
- **THEN** inspection retains that code and output independently of the queue task state

### Requirement: Automatic completion policy
With autocomplete true, final confirmed exit, including a nonzero exit or launch failure with no remaining process, SHALL request ordinary task completion exactly once logically. With autocomplete false, final exit SHALL leave the task running until another completion request. Cancellation SHALL use the common completion barrier.

#### Scenario: Failed work with autocomplete
- **WHEN** a command exits nonzero with autocomplete true
- **THEN** its failure outcome is retained and a validated completion request releases resources

### Requirement: Process-tree cancellation barrier
Timeout, manual completion, plugin disable and daemon shutdown SHALL coordinate termination of managed work and descendants. Resource release SHALL wait for confirmed termination; failure or uncertainty SHALL keep ownership and expose a recoverable error. Cancellation SHALL prevent later queued entries from launching.

#### Scenario: Script starts a descendant
- **WHEN** timeout expires while a command and its descendant use task resources
- **THEN** both are confirmed stopped before any successor receives those resources

### Requirement: Native optional execution
The plugin SHALL use native process supervision on Linux, macOS and Windows without requiring WSL. The core queue SHALL not require an interpreter. An unavailable configured interpreter or non-executable src SHALL produce a visible launch failure rather than silently choosing another execution mode.

#### Scenario: Missing interpreter
- **WHEN** sh is configured but its selected interpreter is unavailable
- **THEN** no command executes and the launch failure follows the configured autocomplete policy
