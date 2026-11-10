# Spec Delta

## ADDED Requirements

### Requirement: Independent verification of tasks and resources
Task, atomic admission, and completion invariants SHALL be tested without a database or processes. Adapters SHALL be tested separately, including real contention for several topics, ownership durability, and rollback.

#### Scenario: One resource unavailable
- **WHEN** a unit test supplies a task use case with two topics, one busy
- **THEN** pure domain policy rejects admission without reserving the other resource or using real infrastructure

## REMOVED Requirements

### Requirement: Independent rule verification
**Reason**: Lease/delivery checks are replaced by named-task/admission invariants.
**Migration**: Retain isolated domain/use-case and separate adapter tests, replacing message/group/lease fixtures with tasks/resources.
