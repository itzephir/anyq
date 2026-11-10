# repository-delivery Specification

## Purpose

Changes pass separate verifiable CI stages; the default branch accepts them only through a PR with a current successful result for all required checks.

## Requirements

### Requirement: Separate CI categories
CI SHALL run a separate lint job only on Linux, and build-and-unit-tests and integration-tests natively on Linux, macOS, and Windows. Failure on one OS SHALL NOT cancel the others.

#### Scenario: One linter run
- **WHEN** a PR is created or updated
- **THEN** lint runs once on Linux while builds and tests run on three operating systems

#### Scenario: Integration failure
- **WHEN** an integration test fails on Windows
- **THEN** Windows integration fails while the other platforms continue their checks

### Requirement: Required aggregate status
The final CI passed SHALL run after all categories and pass only when every category succeeds. Failure, cancelled, and skipped SHALL NOT produce a successful final status.

#### Scenario: Skipped job
- **WHEN** any required job is skipped or cancelled
- **THEN** the final CI passed does not permit merging

### Requirement: Default branch protection
GitHub SHALL require a PR and current successful CI passed from GitHub Actions before merging into the default branch, including for administrators. Force pushes and default branch deletion SHALL be forbidden; PR discussions SHALL be resolved before merge.

#### Scenario: CI did not pass
- **WHEN** a PR lacks a current successful CI passed
- **THEN** GitHub blocks merging into the default branch

#### Scenario: Direct push
- **WHEN** an administrator attempts to push directly to the default branch or force-push
- **THEN** GitHub rejects bypassing the configured PR/CI workflow
