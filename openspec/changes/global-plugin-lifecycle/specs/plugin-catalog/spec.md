# Spec Delta

## Purpose

Give one user a common set of explicitly enabled plugin binaries for every running anyq queue, independently of queue storage and plugin-specific metadata policy.

## ADDED Requirements

### Requirement: Common binary directory
Installed plugins SHALL be independent native binaries in `~/.anyq/plugins/`, including shell and timeout. A plugin SHALL be selected by its validated name, with the platform executable suffix. Discovering a file SHALL NOT enable execution implicitly.

#### Scenario: Discover a third-party binary
- **WHEN** a user places an executable named observer in the plugin directory
- **THEN** plugin list includes observer as disabled until explicitly enabled

#### Scenario: First-party installation
- **WHEN** CLI or desktop installation provisions first-party plugins for a new user
- **THEN** shell and timeout are separate native binaries in that user's plugin directory and neither is enabled implicitly

#### Scenario: Previously published CLI archive
- **WHEN** a current installer selects a published v0.1.0 or v0.1.1 archive containing only the CLI
- **THEN** installation succeeds without modifying adjacent plugin files, user plugins or queue data

#### Scenario: Incomplete plugin archive
- **WHEN** a downloaded archive contains only one first-party plugin, or a later-version archive contains neither
- **THEN** installation fails before replacing any installed executable

#### Scenario: Cross-target desktop distribution
- **WHEN** a desktop package targets another supported architecture or universal macOS
- **THEN** its first-party plugin binaries contain the same target architecture or architectures as the application

### Requirement: User-wide explicit enablement
`plugin enable NAME` and `plugin disable NAME` SHALL persist one user-wide enabled set. Every active owner and subsequently started owner SHALL use that set without per-database enable commands. Catalog operations SHALL NOT start custom database owners or require a queue database.

#### Scenario: Two active databases
- **WHEN** shell is enabled while two independent queues have explicitly started owners
- **THEN** both owners apply shell to future running events without enabling it again for either database

#### Scenario: Stopped custom database
- **WHEN** a plugin is globally enabled and a custom database has no owner
- **THEN** that database remains stopped until an explicit daemon start

#### Scenario: Concurrent catalog updates
- **WHEN** independent clients enable different installed plugins concurrently
- **THEN** both changes survive and readers observe complete old or new catalog snapshots

### Requirement: Plugin-owned metadata policy
Anyq SHALL transport arbitrary bounded task and topic metadata through the existing public protocol. Each binary SHALL own interpretation and validation of its parameters. The catalog SHALL NOT define plugin-specific configuration keys or translate metadata into executable arguments.

#### Scenario: Unknown plugin parameters
- **WHEN** a task contains metadata keys understood only by a third-party plugin
- **THEN** the plugin receives them unchanged without a framework schema for those keys

### Requirement: Safe catalog failures
Invalid names, absent or non-executable binaries and malformed enablement storage SHALL produce visible errors without partial updates or implicit trust. Removing or disabling a binary SHALL retain queue-local history and follow the managed cancellation barrier.

#### Scenario: Missing enabled binary
- **WHEN** an enabled plugin binary is removed before owner restart
- **THEN** the plugin does not launch, status exposes the unavailable executable and unresolved work retains resource ownership

#### Scenario: Legacy queue-local registration
- **WHEN** a database contains an old executable registration absent from the global enabled catalog
- **THEN** its history is preserved without launching or promoting that executable to user-wide enablement
