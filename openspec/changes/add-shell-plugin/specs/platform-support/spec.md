# Spec Delta

## MODIFIED Requirements

### Requirement: Native coordination CLI
Task registration, topic linking, wait, and complete SHALL run natively on macOS, Windows, and Linux through a local background queue daemon with identical states, atomicity, and errors. Runtime SHALL NOT require Python, bash, or WSL; the caller SHALL remain responsible for external execution unless it explicitly selects the optional shell plugin. That plugin SHALL use native process supervision; a shell interpreter SHALL be required only for shell-command mode.

#### Scenario: Admission on Windows
- **WHEN** Windows CLI obtains task admission to two topics and exits
- **THEN** the task remains running in both topics until a separate complete
