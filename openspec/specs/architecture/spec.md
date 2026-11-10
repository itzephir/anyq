# architecture Specification

## Purpose

Clean Architecture centered on domain rules keeps the queue independent of SQLite, CLI, and process execution, with build-verifiable boundaries.

## Requirements

### Requirement: Domain boundaries and DI
Features SHALL have separate api/impl modules. Cross-feature dependencies SHALL target api only; the domain and interactors SHALL NOT depend on IO or infrastructure. Implementations SHALL be injected explicitly through ports, and impl factories SHALL be called only by designated composition roots.

#### Scenario: Forbidden dependency
- **WHEN** an architecture check receives a dependency from an inner use case to SQLite or another feature's impl
- **THEN** the check fails

### Requirement: Independent verification of tasks and resources
Task, atomic admission, and completion invariants SHALL be tested without a database or processes. Adapters SHALL be tested separately, including real contention for several topics, ownership durability, and rollback.

#### Scenario: One resource unavailable
- **WHEN** a unit test supplies a task use case with two topics, one busy
- **THEN** pure domain policy rejects admission without reserving the other resource or using real infrastructure

### Requirement: Separate admission from observation
Admission decisions SHALL belong to the domain and be invoked by queue mutations inside their transactions. Blocking wait SHALL depend only on the narrow running-check contract. Storage SHALL execute exact use-case filters, including queued tasks unrestricted by a single topic, without deciding admission.

#### Scenario: Observe without mutation
- **WHEN** a running check is performed for an existing queued or running task
- **THEN** it returns the result without writing a state transition or changing ownership
