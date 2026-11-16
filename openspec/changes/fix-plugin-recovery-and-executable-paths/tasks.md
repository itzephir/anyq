# Tasks

## 1. Recovered shell completion

- [x] 1.1 Track and retry recovered autocomplete with stable receipts, recognize already completed tasks, document recovery, and verify real CLI/daemon/plugin regressions fail before the fix and pass afterward.

## 2. Plugin executable invocation

- [x] 2.1 Preserve explicit and PATH-selected absolute invocation paths, document virtual environment registration, and verify isolated Python environment integration before/after daemon restart fails before the fix and passes afterward.

## 3. Combined verification and delivery

- [x] 3.1 Run the required local checks, verify both regressions are wired into the Linux/macOS/Windows integration matrix, and record the independent main desktop CI failures with linked evidence.

## 4. Live autocomplete race follow-up

- [x] 4.1 Share terminal-state reconciliation for live and recovered completion, add a real CLI/daemon/live-shell regression that fails before the fix and passes afterward, document the behavior, run required local checks, and wire the scenario into the existing Linux/macOS/Windows workflow suite.

## 5. Parallel shell completion follow-up

- [x] 5.1 Bound first-party shell protocol output and poll it fairly without relaxing host flood protection; add a real native integration regression for 16 simultaneous exits, prove failure before and success after, document behavior, run required local checks, and include the case in the existing Linux/macOS/Windows workflow suite.

## Workflow follow-up

- Open the requested PR and inspect its Linux/macOS/Windows integration results.
- Sync and archive only after the complete change is verified under the repository's review requirements.
