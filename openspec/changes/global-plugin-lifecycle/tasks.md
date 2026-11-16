# Tasks

## 1. Common plugin catalog

- [x] 1.1 Add narrow catalog list/enable/disable contracts, policy and a locked atomic file adapter; verify real filesystem tests cover discovery, invalid/missing executables, concurrent updates and preserved old state after failed writes.
- [x] 1.2 Expose global name-only CLI catalog commands and document global versus selected-queue scope; verify real CLI operations work without a queue and do not start a custom owner.

## 2. Independent plugin binaries and installation

- [x] 2.1 Build standalone shell and timeout executables with explicit composition roots and private shell-anchor support; verify real protocol handshakes, shell execution and timeout completion through those binaries.
- [ ] 2.2 Package and provision first-party binaries into the common user directory for CLI and desktop distributions without implicit enablement or overwriting user binaries; verify portable and installer install/remove/reinstall scenarios on native targets and update installation documentation.

## 3. Global policy with isolated runtime progress

- [x] 3.1 Add an additive plugin-storage migration and revision-aware catalog reconciliation before initialized sessions and during live host polling; verify migration/history preservation and live enable/disable adoption without obsolete wire registration bypass.
- [x] 3.2 Add real multi-database plugin integration scenarios covering equal task names, arbitrary task/topic metadata, independent state/timers/receipts, first enable versus recovery, stopped custom owners and catalog failures; verify no cross-queue effects or duplicate execution.
- [x] 3.3 Update plugin/shell/timeout examples and recovery documentation for name-only global enablement and migration; run documented real-binary examples and existing managed cancellation, recovery and simultaneous-exit integration suites.

## 4. Explicit custom owners

- [x] 4.1 Introduce default-versus-existing-only connection selection using canonical identity and the current default precedence; verify real default auto-start, equivalent paths, stopped custom requests, explicit start/restart and preserved ownership after crashes.
- [x] 4.2 Apply the same policy to desktop observation and default application startup without creating or migrating selected custom storage; verify real GUI switching/disconnection/reconnection and existing-only legacy/corrupt/missing database scenarios.
- [x] 4.3 Update shared CLI/desktop fixtures and lifecycle documentation for intentional default and custom startup; verify native interruption, contention and package-lifetime suites still exercise their intended boundaries.

## 5. System autolaunch

- [x] 5.1 Add separate typed autolaunch contracts, dispatch and native per-user OS registration adapters with retained launcher handling; verify real isolated native registration read/on/off, failed updates and preserved previous registration on Linux, macOS and Windows.
- [x] 5.2 Add config-independent `anyq autolaunch [on/off]` and the default-only login launcher path for CLI and desktop executables; verify the registered command launches the current default with enabled plugins, leaves custom queues stopped and disabling leaves active work unchanged, including with malformed anyq config.
- [x] 5.3 Document native autolaunch scope, platform registration and recovery boundaries; verify command help and examples and assert no autolaunch key is written to shared or panel preferences.

## 6. Desktop system control

- [x] 6.1 Add accessible EN/RU desktop autolaunch status/control as an immediate OS action separate from preference drafts; verify real native CLI-to-GUI state updates, success/error handling and unchanged queue/config state on all three operating systems.
- [x] 6.2 Add deterministic autolaunch settings screenshot scenarios, explicitly generate and review affected/new platform PNG goldens, and verify independent clean renders and read-only screenshot comparison on Linux, macOS and Windows.

## 7. Integrated verification

- [x] 7.1 Run all mandatory repository checks: Rustfmt, workspace Clippy with all targets/features and warnings denied, workspace tests, feature-enabled native interruptions, both architecture checks, Ruff lint/format, strict OpenSpec and diff whitespace; retain exact results and resolve failures without weakening checks.
- [ ] 7.2 Verify current-head native CLI/plugin/desktop/system-registration, screenshot and distribution CI on Linux/macOS/Windows, including macOS Intel and Wayland jobs; only mark complete after required checks pass and report local versus remote coverage accurately.

## Workflow follow-up

- Sync and archive verified behavior only after implementation and required review; do not mark unfinished behavior as baseline.
- Follow repository PR protection and merge only when explicitly requested by the user.
