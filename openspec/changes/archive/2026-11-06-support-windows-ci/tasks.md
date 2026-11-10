# Tasks

## 1. Platform adapter

- [x] 1.1 Separate Unix/Windows worker details; implement Job Object with safe suspended launch, cleanup, and console cancellation; verify compilation, native process-tree/interruption fixtures on relevant operating systems; update architecture documentation.
- [x] 1.2 Add LOCALAPPDATA as the Windows database default; verify with an isolated CLI test and document explicit-path precedence.
- [x] 1.3 Fix Unix cleanup assertions: exited but unreaped children count as terminated; do not mask observation errors; test a live process and zombie regression before wait().

## 2. Portable checks

- [x] 2.1 Move shared CLI integration tests/demo to portable fixtures, retaining stdin, handoff, heartbeat, and failure checks; run cargo tests/demo and add a Windows README example.
- [x] 2.2 Add Windows CI, normalize architecture-checker paths, run fmt/clippy/tests/OpenSpec/architecture, and wait for green macOS/Windows/Linux PR matrices.

## Workflow follow-up

- Archive the completed change after review and merge. Do not merge without a user request.
