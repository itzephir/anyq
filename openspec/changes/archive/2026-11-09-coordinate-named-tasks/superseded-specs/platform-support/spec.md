# platform-support Specification

## Purpose

Users can run the same local queue natively on macOS, Windows, and Linux, with CI checking real processes and durability on every platform.

## Requirements

### Requirement: Native CLI and worker
CLI and worker SHALL build and execute natively on macOS/Windows/Linux, preserving JSON stdin, environment variables, and independent consumer groups. Windows SHALL NOT require WSL or bash for runtime/tests.

#### Scenario: Windows command
- **WHEN** Windows worker successfully executes an explicitly specified program
- **THEN** it acknowledges delivery and atomically publishes the next stage

### Requirement: Process-tree termination
On cancellation or detected lease loss, worker SHALL terminate the launched command and descendants in its managed group. After the main command exits, remaining descendants SHALL terminate before releasing delivery. macOS/Linux SHALL handle SIGINT/SIGTERM; Windows SHALL handle Ctrl+C/Ctrl+Break.

#### Scenario: Parent exits before descendant
- **WHEN** a command exits leaving a running descendant
- **THEN** worker terminates the descendant before acknowledging the result

#### Scenario: Windows interruption
- **WHEN** Windows worker receives a console interruption during a command
- **THEN** the command tree terminates, delivery is rejected, and worker exits 130

### Requirement: Windows database path
Without --db or ANYQ_DB, Windows CLI SHALL use LOCALAPPDATA/anyq/queue.db. Explicit paths SHALL override the default location.

#### Scenario: Local database
- **WHEN** Windows CLI runs with LOCALAPPDATA and no explicit database path
- **THEN** the database is created in the anyq subdirectory of LOCALAPPDATA

### Requirement: CI matrix
CI SHALL check formatting, architecture, OpenSpec, and lint on Linux, and builds, domain/end-to-end tests, and demos natively on macOS, Windows, and Linux. Failure on one platform SHALL NOT cancel the others.

#### Scenario: Implementation change
- **WHEN** a commit is pushed or a PR opened
- **THEN** lint passes on Linux and builds and tests pass on all three native runners
