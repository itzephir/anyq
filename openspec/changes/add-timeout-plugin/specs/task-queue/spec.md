# Spec Delta

## MODIFIED Requirements

### Requirement: Persistent ownership until completion
A running task SHALL hold one slot from every attached topic until an explicit completion request from a caller or an enabled plugin. A metadata-driven timeout SHALL act only through such a validated request; absent that policy, elapsed time SHALL NOT change ownership. Returning from wait, caller exit or crash, CLI restart, and elapsed time without an enabled timeout policy SHALL NOT release ownership.

#### Scenario: Wait process exited
- **WHEN** wait successfully returned admission and its process exited
- **THEN** the task remains running and holds all its slots

#### Scenario: External work crashed
- **WHEN** the caller process dies after admission without an enabled timeout policy
- **THEN** slots remain acquired until explicit task completion
