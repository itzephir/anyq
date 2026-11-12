# Code rules

These rules are mandatory for anyq implementation and review. Automated checks supplement review of actual responsibilities and boundaries.

## One type per file

One standalone struct, enum, or trait per file. This includes entities, DTOs, interfaces, interactors, adapters, CLI arguments, and test doubles. Methods and related impls stay with their owner. Function-local nested types are allowed; a namespace/inline module does not make independent types local. lib.rs/mod.rs define explicit modules and exports.

## Readability

Use one blank line between declarations, impl blocks, methods, and logical phases: input validation, state reads, domain decisions, persistence, and return. Keep related actions together. In tests, visually separate arrange, act, and assert. Rustfmt maintains formatting but does not replace meaningful grouping.

## SOLID and responsibility

- SRP: RegisterTaskInteractor registers a name; CheckTaskRunningInteractor observes running; queue mutations invoke a shared admission pass; CompleteTaskInteractor releases ownership. Do not combine use cases into a manager or dispatcher.
- OCP: a new use case gets its own contract and implementation. CLI syntax parsing stays in the external adapter.
- LSP: adapters honor the full port contract, including errors and rollback. Verify fakes and SQLite with meaningful scenarios.
- ISP: consumers receive only the use case they need; blocking CLI wait depends on CheckTaskRunning.
- DIP: the domain and use cases depend on their own contracts. SQLite, CLI, and IO stay outside; implementations are injected explicitly.

A generic business API execute(Command) -> Output is forbidden. Use cases return concrete results: RegisterTask -> Task, CheckTaskRunning -> bool. Force is an explicit completion parameter. Domain-owned AutoCreatePolicy separately authorizes creation of a missing task; force does not bypass errors or other invariants.

Task owns state transitions and completeness of the resource set. ResourceAvailability owns free-slot and predecessor-queue checks. CLI and SQL must not duplicate these rules.

RegisterTaskRequest combines the name and initial topics for one registration use case; it does not replace separate linking, admission, and completion contracts. CLI chains only invoke these use cases sequentially. ListTasks, GetTask, and GetTopicQueue have their own narrow interfaces; TopicQueue defines current queue states and order.

## DRY, KISS, and atomicity

Every domain rule has one owner. Do not store a separate mutable slot counter and independently repair it on completion: derive usage from running tasks and links.

UnitOfWork combines a technical transaction boundary with consumer-owned persistence ports. It is not a business router. QueueModule contains explicitly constructed handles without behavior, string lookup, or a service locator.

Do not add an abstraction without a concrete rule or consumer. The caller owns external work, result handling, and retries. Anyq preserves ownership until explicit task completion.

## Boundaries

Api and inner use cases do not import impl, IO, or infrastructure SDKs. Actual Cargo dependencies point inward. Controllers translate input; presenters format responses. The blocking wait pause belongs to CLI; no transaction or partial ownership is held during it.

Composition roots are listed in [ARCHITECTURE.md](ARCHITECTURE.md): queue/impl/composition, config/impl/composition, daemon/impl/composition, plugins/impl/composition, desktop/impl/composition, apps/cli/src/main.rs, and apps/desktop/src/main.rs. Only they assemble implementations. The CLI composes remote queue adapters; only daemon composition imports the queue implementation for serving sessions.

Names and their reversible representation belong to the queue domain; restore receives canonical strings without parsing them again. The config domain owns keys and defaults; the file adapter only handles TOML/IO. Autocreate belongs to the main use-case transaction and rolls back on its error.

## Checks

Linux lint includes Rustfmt, Clippy with -D warnings, the Rust AST linter, the Cargo/API boundary checker, Ruff, and strict OpenSpec validation. Architecture checks have negative fixtures.

Builds and unit/integration tests run separately on Linux, macOS, and Windows. They verify atomic admission, rollback, real concurrency, persistent ownership after crashes, all CLI forms, config, and autocreate. The native Ctrl+C suite with integration-test-hooks is required on all three operating systems; it sends signals to real processes without skip/ignore. Checkpoints are absent from the normal binary and inner layers.

Do not disable checks or add blanket allow directives to make CI pass. Review additionally checks SRP, SOLID, DRY, KISS, ownership of domain rules, and test quality.

Optional first-party plugin modes use the same typed public protocol as external
plugins. Only the CLI root assembles their protocol adapters and native process
ports. Prepared shell policy and queue admission never spawn processes.


Behavior changes require integration tests against their real external boundary.
Graphical interfaces additionally require native screenshot regression scenarios
on Linux, macOS and Windows, with reviewed PNG goldens committed to the repository.
Missing goldens and visual differences fail CI and retain actual/diff artifacts.
Explicit golden updates are reviewed as code; comparison jobs never rewrite or
auto-accept the baseline. Package builds, unit/controller tests and manually saved
screenshots supplement these required checks rather than replace them.

Screenshot runners provide separate `verify` and `generate` modes. Verification
never modifies goldens and fails on missing or changed images; generation creates
reviewable candidates. A visual state's configuration owns its complete fixture,
clock/timezone, locale, theme, viewport, selection, scroll and focus. Independent
clean runs must produce deterministic renders under a documented visual comparison
before a candidate is accepted. Document any handling of native antialiasing;
never use broad image masks or automatic acceptance to hide state/layout changes.
All tests should isolate their inputs and wait for observable conditions, with
minimal dependence on the host environment and arbitrary elapsed delays.
