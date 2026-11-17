# Spec Delta

## ADDED Requirements

### Requirement: Independent shell executable
The first-party shell plugin SHALL ship as an independent native binary in the common user plugin directory. `plugin enable shell` SHALL select it without requiring a CLI executable path or runtime-mode arguments. It SHALL use the public plugin protocol and retain existing task metadata, native containment, receipts and completion behavior.

#### Scenario: Enable packaged shell
- **WHEN** the user installs anyq and enables shell by name
- **THEN** a separate shell binary executes valid future running task plans without using the CLI as its plugin process

#### Scenario: Same plan in two databases
- **WHEN** two explicitly active queues use shell metadata with the same task name
- **THEN** each queue executes its own committed plan and retains its own outcomes and resource ownership
