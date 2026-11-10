# Tasks

## 1. Shell plan and execution context

- [x] 1.1 Add pure shell-plan validation for exclusive sh/src, nonempty entries, autocomplete, cwd and interpreter options; verify invalid plans and topic metadata cannot launch commands.
- [x] 1.2 Capture/persist absolute task-origin cwd and migrate unknown origins explicitly; verify a delayed start resolves relative scripts against the creating client directory rather than daemon cwd.
- [x] 1.3 Document native interpreter defaults, executable src semantics and environment policy; verify examples use explicit supported commands on each OS and never imply arbitrary cross-platform shell syntax.

## 2. Managed process adapter

- [x] 2.1 Implement native process-group/job supervision behind consumer-owned ports; verify ordinary exit, launch failure and descendants outliving the leader with real native processes.
- [x] 2.2 Implement sequential entries and bounded concurrent stdout/stderr drains; verify first-error stopping, output saturation, truncation and cancellation preventing later launches.
- [x] 2.3 Persist managed registration/launch intent and durable launch/exit receipts; verify duplicate running deliveries do not relaunch and crashes around spawn become uncertain with slots retained.
- [x] 2.4 Implement confirmed process-tree cancellation for timeout/manual completion/disable/shutdown; verify every managed descendant stops before successors acquire resources and failure keeps all slots held.
- [x] 2.5 Document adapter containment, uncertain recovery and explicit resolution; verify PID reuse cannot kill unrelated work or falsely certify cleanup.

## 3. Outcomes and plugin integration

- [x] 3.1 Expose first-party shell through the public executable protocol and inspection API/CLI; verify English-only status, exit codes and bounded logs remain available after daemon restart.
- [x] 3.2 Implement autocomplete false/true through replay-safe completion requests; verify success, nonzero exit and no-process launch failure retain outcomes while applying the chosen completion policy.
- [x] 3.3 Add end-to-end timeout+shell tests with descendants and manual completion races; verify resource capacity is never reused before confirmed managed termination.
- [x] 3.4 Update architecture/project context, ownership/interruption docs and runnable examples for optional execution; verify ordinary caller-owned workflows and disabled shell still run no commands.

## 4. Integration acceptance

- [x] 4.1 Run all mandatory local checks plus native process-tree/crash/autocomplete tests; retain results and leave unsupported or untested behavior unchecked.
- [ ] 4.2 Open the shell-only implementation PR and verify current CI passed on Linux/macOS/Windows and all required categories, including native interruption and distribution.
- [ ] 4.3 Sync only verified shell and platform deltas into baseline specs and verify strict OpenSpec validation without losing prior daemon or timeout wording.

## Workflow follow-up

- This is stage 5 and depends on all preceding roadmap stages.
- Do not merge without the user's instruction. Archive after merge and verified baseline sync.
