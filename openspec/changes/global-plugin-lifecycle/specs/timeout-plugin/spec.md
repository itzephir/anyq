# Spec Delta

## ADDED Requirements

### Requirement: Independent timeout executable
The first-party timeout plugin SHALL ship as an independent native binary in the common user plugin directory. `plugin enable timeout` SHALL select it without a CLI path or runtime-mode arguments. Deadline parsing SHALL remain plugin-owned, using arbitrary task/topic metadata from public event snapshots.

#### Scenario: Enable packaged timeout
- **WHEN** the user enables the installed timeout binary by name
- **THEN** all active queues apply their own metadata deadlines without a per-database enable command

#### Scenario: Resume after explicit owner restart
- **WHEN** an owner restarts with timeout globally enabled and a persisted overdue timer
- **THEN** the separate timeout binary receives the original due timer and preserves managed-work cancellation before release
