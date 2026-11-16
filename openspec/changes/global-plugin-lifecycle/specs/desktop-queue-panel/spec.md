# Spec Delta

## ADDED Requirements

### Requirement: System autolaunch control
Desktop settings SHALL expose a localized accessible login-launch control backed by the same OS registration as `anyq autolaunch on/off`. The control SHALL query system state, report failures and remain separate from panel and shared configuration persistence. Changing it SHALL NOT stop active owners, processes or tasks.

#### Scenario: CLI changes system registration
- **WHEN** the user changes autolaunch in CLI and opens desktop settings
- **THEN** the control reflects the operating system's current registration

#### Scenario: System write fails
- **WHEN** a user enables login launch and the OS registration fails
- **THEN** settings report the failure and do not display a successful enabled state

#### Scenario: Unsaved panel preferences
- **WHEN** the user changes system autolaunch with an unsaved database/theme draft
- **THEN** the system action neither saves that draft nor writes an autolaunch key into panel.toml

### Requirement: Default startup and custom connection
Application startup SHALL ensure the current default owner is available. Selecting, refreshing or observing a different database SHALL only connect to its explicitly started owner. A stopped custom owner SHALL produce a disconnected or stale state without starting it or falling back to another database.

#### Scenario: Select a stopped custom queue
- **WHEN** the user selects a valid custom database whose daemon is stopped
- **THEN** the application shows the unavailable owner and the database remains stopped

#### Scenario: Start selected custom owner externally
- **WHEN** the selected custom queue is disconnected and the user explicitly starts its owner through CLI
- **THEN** a subsequent refresh shows that queue without changing the selection
