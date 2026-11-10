# Spec Delta

## Purpose

Clean Architecture centered on domain rules keeps the queue independent of SQLite, CLI, and process execution, with build-verifiable boundaries.

## ADDED Requirements

### Requirement: Domain boundaries and DI
Features SHALL have separate api/impl modules. Cross-feature dependencies SHALL target api only; the domain and interactors SHALL NOT depend on IO or infrastructure. Implementations SHALL be injected explicitly through ports, and impl factories SHALL be called only by designated composition roots.

#### Scenario: Forbidden dependency
- **WHEN** an architecture check receives a dependency from an inner use case to SQLite or another feature's impl
- **THEN** the check fails

### Requirement: Independent rule verification
Invariants/use cases SHALL be tested without SQLite or processes. Adapters SHALL be tested separately, including real concurrent claims and rollback.

#### Scenario: Lease without database
- **WHEN** a unit test checks an expired token with injected time
- **THEN** the domain decides rejection without infrastructure
