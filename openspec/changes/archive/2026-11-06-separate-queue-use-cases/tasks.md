# Tasks

## 1. Contracts and use cases

- [x] 1.1 Remove Queue/Command/Output/Dispatch; extract separate contracts, interactors, and persistence ports; verify concrete unit/SQLite integration scenarios without behavior changes.
- [x] 1.2 Move worker and CLI to minimal typed dependencies; split models, adapters, arguments, and test doubles into files; verify CLI handoff, retries, heartbeat, and errors.
- [x] 1.3 Separate declarations, methods, and logical phases with blank lines, including arrange/act/assert in tests; document readability rules and verify the Rust diff changes whitespace only; string literals remain unchanged, formatting and local checks pass.

## 2. Enforcing rules

- [x] 2.1 Add one-top-level-type-per-file and generic-business-dispatcher checks, negative fixtures, and architecture documentation; pass fmt/clippy/tests/architecture/OpenSpec and CI.

## Workflow follow-up

- Archive the completed change after review; do not merge the PR without a user request.
