# Tasks

## 1. Domain and use cases

- [x] 1.1 Implement automatic admission, typed batch-link use cases, and observing CheckTaskRunning; run domain/dispatch tests covering multiple resources, force, capacity, and rollback; update architecture documentation.

- [x] 1.2 Add domain autostart defaults/precedence, persistent stop, and explicit StartTask; verify domain/dispatch/config tests, rollback, and stored settings.

## 2. SQLite and CLI

- [x] 2.1 Support all-queued filtering in SQLite and observing wait in CLI; test autonomous advancement, both atomic batch-link forms, concurrency, and repeated waits in SQLite/CLI; update README/help.

- [x] 2.2 Add config task.autostart, topic autostart, create --start/--stop/-s/-tswc, and start -w/-c; test CLI matrix and SQLite migration 2 → 3; update README/help.

## 3. Persistence boundaries

- [x] 3.1 Update native Ctrl+C tests and docs/INTERRUPTIONS.md for atomic advancement in registration/link/complete/capacity/start/autostart and schema upgrade; run the suite with integration-test-hooks.

## 4. Final verification

- [x] 4.1 Run all mandatory AGENTS.md checks, create a PR, and wait for standard Linux/macOS/Windows CI results.

## Workflow follow-up

- Sync baseline specs and archive the change after review and authorized merge.
