# Tasks

## 1. Protocol and ownership boundary

- [x] 1.1 Add daemon API/protocol and client/runtime/composition crates with explicit roots; verify Cargo graph checks and negative fixtures reject inner IO or CLI access to queue impl.
- [x] 1.2 Define bounded versioned request/response and typed error DTOs; verify round trips preserve canonical names and malformed/oversized/version-mismatched frames execute no operation.
- [x] 1.3 Implement private canonical-path discovery and lifetime singleton locking; verify concurrent first starts, symlink aliases, independent databases and stale endpoint replacement with real processes.
- [x] 1.4 Document protocol, private endpoint access and mixed-version limitations in docs/ARCHITECTURE.md and daemon usage docs; verify documented foreground/status commands against a temporary database.

## 2. Runtime and CLI migration

- [x] 2.1 Implement foreground serving, automatic detached startup, start/status/stop and bounded shutdown; verify a daemon survives client exit and stopped/restarted owners retain every task state and slot.
- [x] 2.2 Implement individual remote queue trait adapters and switch every task/topic CLI path to them; verify existing workflow/name/alias/config/path-precedence tests and chain commit behavior without direct CLI SQLite access.
- [x] 2.3 Preserve config/help/version independence and English-only CLI presentation; verify those commands create no database/owner and no CLI language option or translation resources are added.
- [x] 2.4 Report response-loss ambiguity without replaying mutations; verify a severed post-commit response preserves ownership and a later query resolves the durable result.
- [x] 2.5 Update README/examples and OpenSpec project context for daemon ownership; verify examples run as documented and do not imply plugin execution exists yet.

## 3. Cancellation and recovery

- [x] 3.1 Add per-request cancellation and safe-boundary propagation without global token sharing; verify cancelling one real client rolls back uncommitted work while other clients and owner remain usable.
- [x] 3.2 Refactor feature-only integration gates to request/session scope with per-session hit counts; verify all existing native Ctrl+C cases still exercise real owner persistence boundaries without skip/ignore.
- [x] 3.3 Add foreground-owner signal/crash tests and blocked-client framing/output tests; verify committed states survive and shutdown remains bounded on native Linux/macOS/Windows CI.
- [x] 3.4 Add fixture-owned daemon cleanup and update docs/INTERRUPTIONS.md; verify process tests leave no owners and first/second Ctrl+C behavior matches documented boundaries.

## 4. Integration acceptance

- [x] 4.1 Run all mandatory formatting, lint, architecture, Ruff, OpenSpec, workspace and native interruption commands; retain their results and leave failures unchecked.
- [x] 4.2 Open the daemon-only implementation PR and verify current GitHub CI passed, including all native matrices, distribution and website categories, before marking cross-platform acceptance complete.
- [x] 4.3 Sync only verified daemon behavior into baseline specs; verify strict OpenSpec validation and review the diff for unchanged admission/ownership semantics.

## 5. Review corrections

- [x] 5.1 Separate bounded waiting/command session quotas and reserve lifecycle capacity; verify 64 real wait/start clients, chained completion, quota exhaustion and lifecycle access at full command capacity.
- [x] 5.2 Clarify selected anyq database ownership in README and document the session limits; verify mandatory local checks and synchronize only verified lifecycle scenarios.
- [x] 5.3 Publish the corrections in PR #14 and verify current native Linux/macOS/Windows CI without merging PRs.

## Workflow follow-up

- This is stage 1; metadata implementation starts from its branch only after the daemon implementation is reviewable.
- Do not merge without the user's instruction. Archive after merge and verified baseline sync.
