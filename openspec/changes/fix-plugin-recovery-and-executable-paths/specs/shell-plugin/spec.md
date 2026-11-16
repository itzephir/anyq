## MODIFIED Requirements

### Requirement: Automatic completion policy
With autocomplete true, final confirmed exit, including a nonzero exit or launch failure with no remaining process, SHALL request ordinary task completion exactly once logically. With autocomplete false, final exit SHALL leave the task running until another completion request. Cancellation SHALL use the common completion barrier. Recovery SHALL retry pending completion with the original action identity without relaunching work. Live execution and recovery SHALL retire pending completion after confirming that another caller already completed the task. Outgoing capability requests SHALL use bounded fair flow control without overflowing or relaxing host protocol limits.

#### Scenario: Failed work with autocomplete
- **WHEN** a command exits nonzero with autocomplete true
- **THEN** its failure outcome is retained and a validated completion request releases resources

#### Scenario: Recovered completion waits for another worker
- **WHEN** recovery finds confirmed stopped autocomplete work and another managed action temporarily blocks its completion
- **THEN** the task retains all slots until that action confirms stopped, and the plugin retries the same completion action until completion succeeds and successors are admitted

#### Scenario: Recovered work was already completed
- **WHEN** a recovered autocomplete action refers to a task that another caller already completed
- **THEN** recovery recognizes the terminal state without relaunching work or creating another logical completion

#### Scenario: Manual completion wins a live autocomplete race
- **WHEN** a live shell plugin confirms command exit and another caller completes its task before autocomplete succeeds
- **THEN** the live plugin confirms the terminal task state and retires that execution without restarting the plugin, repeating completion indefinitely, relaunching work or creating another completion event

#### Scenario: Sixteen commands finish together
- **WHEN** 16 admitted shell tasks exit after a shared release signal with autocomplete enabled
- **THEN** bounded fair protocol output completes each task once, preserves the plugin process without protocol failure or uncertain work, and makes all released slots reusable
