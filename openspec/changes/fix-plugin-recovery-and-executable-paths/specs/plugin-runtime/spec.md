## MODIFIED Requirements

### Requirement: Explicit plugin registration
`plugin enable NAME EXECUTABLE [ARG...]` SHALL register and start a local executable with declared event subscriptions. `plugin list`, `plugin status NAME` and `plugin disable NAME` SHALL expose lifecycle and pending/error state. Registration SHALL persist across daemon restart. Unknown metadata SHALL NOT enable plugins implicitly. Resolution SHALL retain the selected executable's absolute invocation path, preserving symbolic links that determine its runtime environment.

#### Scenario: Metadata without plugin
- **WHEN** a task contains shell metadata but shell has not been enabled
- **THEN** no command runs and queue ownership follows ordinary rules

#### Scenario: Virtual environment executable
- **WHEN** an explicitly enabled plugin uses a virtual environment interpreter through its symbolic link, either explicitly or through PATH
- **THEN** the plugin runs with that interpreter's environment before and after daemon restart
