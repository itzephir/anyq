# Design

## Context

The user explicitly requested one standalone type per file, separation of responsibilities, SOLID, DRY, and KISS. The main defect is a single execute entry point with command/result enums, rather than merely file size.

## Goals / Non-Goals

**Goals:** compile-time result types, narrow consumer dependencies, one owner per use case, unambiguous type files, and enforcement of these rules.

**Non-Goals:** CLI protocol, queue semantics, database schema changes, or new frameworks.

## Decisions

- Each queue api use case has a separate minimal trait and concrete result: Topic, Message, Assignment, Delivery, Completion, or a collection. Remove Command/Output and the public Queue dispatcher.
- A separate dispatch interactor implements each contract. Shared transactional publication, lease recovery, and reads become named internal functions without a DI locator or universal router.
- Persistence is split into TopicRecords, GroupRecords, MessageRecords, and DeliveryRecords. UnitOfWork only defines the shared transaction and explicit access to these ports for atomic handoff/fan-out; it contains no business use cases. SQL implementations follow the same data split.
- The composition root returns explicitly typed dependencies without execution/name-resolution methods. Worker receives only ClaimDelivery, RenewLease, CompleteDelivery, RejectDelivery, and GetTopic. CLI invokes the appropriate contract directly; CLI enums remain argument syntax in the external adapter only.
- Every top-level struct/enum/trait has its own file; related impls stay with the type. Rust syntax tokens support checks, with negative fixtures for multiple types and reintroduced generic APIs.
- Tests move to concrete use cases and narrow doubles; end-to-end CLI tests confirm preserved behavior.
- Blank lines separate declarations, methods, and logical use-case phases; tests visibly separate arrange, act, and assert. Group related operations manually rather than inserting a blank line after every expression. Rustfmt does not replace reading and organizing the code.

## Risks / Trade-offs

- [Many files] → meaningful types/use cases, with lib.rs defining public exports only; do not add interfaces without a consumer or rule.
- [Lost atomicity during separation] → shared consumer-owned UnitOfWork and SQLite rollback/concurrency suites preserve verifiable transactions.
