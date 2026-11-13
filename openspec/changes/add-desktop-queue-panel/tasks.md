# Tasks

## 1. Prerequisite and desktop boundaries

- [x] 1.1 Verify that introduce-queue-daemon supplies the owner, private discovery, typed transport, cancellation, and recovery required by this change; record its implementation/CI evidence and keep dependent work blocked if only its planning exists.
- [ ] 1.2 Select the renderer and native shell adapters within the design's fixed Rust-controller architecture; document dependency versions, supported OS/desktop environments, runtime and packaging requirements, and verify native window/popover/tray capability with adapter-level evidence before building features.
- [x] 1.3 Add optional desktop api/impl modules and an explicit application composition root; update architecture documentation and negative fixtures, and verify inner crates cannot import GUI/IO adapters and the ordinary CLI builds without desktop dependencies.

## 2. Owner-served queue observation

- [x] 2.1 Add the separate typed observation contract and pure snapshot projection; verify exact current-state inclusion, empty topics, registration order, inherited autostart, stopped tasks, full membership, and distinct global totals without infrastructure.
- [x] 2.2 Implement the consumer-owned observation port and SQLite adapter under one short snapshot transaction; verify completion/admission races return a complete old or new state and observation leaves business state unchanged with autocreate enabled.
- [x] 2.3 Add the versioned daemon observation operation and narrow client adapter; verify canonical identities including empty/Unicode/backtick names, canonical database identity, unsupported versions, bounded collection/encoding, oversized snapshots, cancellation, and slow clients with real owner processes.
- [x] 2.4 Add existing-database-only owner attachment/startup for the desktop client; verify missing paths, a target disappearing during startup, incompatible files, and owner recovery never create a replacement database, alter ownership, or change ordinary CLI startup behavior.
- [x] 2.5 Document the observation contract, complete-response limit, process ownership, and compatibility errors alongside the daemon API; verify the documented examples against an isolated database without enabling metadata or plugins.

## 3. Panel settings and application state

- [x] 3.1 Share database path-selection policy between external clients and add the panel override ahead of ANYQ_DB/shared db.path/default; verify all precedence combinations, CLI behavior preservation, source reporting, canonical aliases, and stable absolute panel overrides.
- [x] 3.2 Add typed panel-local preferences with atomic coordinated persistence; verify save/restart/reset behavior, persistence failure preserving the old active selection, and unchanged shared config/environment/CLI selection.
- [x] 3.3 Implement one observation controller with bounded nonoverlapping polling, refresh coalescing, request deadlines, and stale/recovery states; verify both surfaces consume the same accepted snapshot and connection errors cannot look like an empty live queue.
- [x] 3.4 Implement source generations and selection invalidation; verify switching A to B discards a late A response even when both contain the same names, clears old rows/details, and leaves the old owner running.
- [x] 3.5 Document Automatic, file override, effective source/path, reset, preference location, inherited environment, and refresh behavior; verify the examples reproduce the intended CLI/panel selection and failure states.

## 4. Main window and shared presentation

- [x] 4.1 Build searchable topic browsing, current task queues, and expandable task details from the accepted snapshot; verify empty topics, multi-topic tasks, registration order, stopped status, names rendered as text, and removal of selected completed tasks.
- [x] 4.2 Build the compact summary projection with distinct task totals, bounded visible topics, and Open Window/topic navigation; verify the summary and main window stay consistent and additional topics remain reachable.
- [x] 4.3 Add database settings, source identity, loading/empty/stale/error states, and explicit refresh to the visual adapter; verify switching to missing/unavailable/incompatible targets reports the selected source without fallback or database creation.
- [x] 4.4 Apply the personal-project design foundation to custom content and implement en/ru resources, light/dark themes, keyboard focus, accessible labels, and narrow-window layouts; verify representative live/empty/error/settings states visually in both languages and themes and document native desktop adaptations.
- [x] 4.5 Verify the desktop dependency graph and visible actions expose observation and panel preferences only; exercise search, refresh, window closure, and source switching with autocreate enabled and confirm no task/topic mutation or resource release occurs.

## 5. Native shell adapters and lifecycle

- [x] 5.1 Implement macOS menu-bar popover, window activation, dismissal, and Quit behavior; retain native smoke evidence that closing/reopening the window preserves the client and quitting it preserves the daemon and running slots, and document macOS usage.
- [x] 5.2 Implement Windows notification-area icon, nearby flyout, context menu, focus, and Quit behavior; verify native placement at representative scale/monitor configurations and unchanged daemon ownership after exit, and document Windows usage.
- [ ] 5.3 Implement Linux StatusNotifierItem/AppIndicator status menu and activation with ordinary-window fallback; verify a compatible host, GNOME with its required integration where available, and a session with no host, including supported Wayland/X11 cases, and document prerequisites and behavior differences.
- [ ] 5.4 Handle runtime tray-host loss and last-window closure; verify no inaccessible background client remains without a usable host and owner/task state is preserved during every desktop lifecycle transition.

## 6. Optional packaging and integration acceptance

- [ ] 6.1 Add optional native desktop packaging and installation/removal guidance without changing ordinary CLI installation; verify each supported native package launches and connects to a compatible existing queue and removal preserves queue data and shared configuration.
- [ ] 6.2 Run real-daemon desktop integration scenarios for concurrent CLI changes, source switching, owner restart, oversized responses, and client exit; retain native window/tray smoke evidence on macOS, Windows, and the documented Linux configurations without substituting mocks for OS behavior.
- [x] 6.3 Run the mandatory repository formatting, Clippy, architecture, Ruff, workspace, native interruption, strict OpenSpec, and diff checks from AGENTS.md; retain results and leave failed or unverified implementation tasks unchecked.
- [x] 6.4 Open the implementation PR and verify current GitHub Actions CI passed, including native Linux/macOS/Windows process and Ctrl+C suites and applicable desktop packaging checks; preserve reviewable evidence for GUI checks that require a desktop session.

- [x] 6.5 Add required real native UI/daemon integration and screenshot regression scenarios on Linux/macOS/Windows; inspect and commit platform PNG goldens, verify missing/changed goldens fail with actual/diff artifacts, and retain a current CI pass for every applicable PR.

## 7. Cross-stack compatibility

- [x] 7.1 Keep observation read models, SQL, and wire DTOs independent of metadata/history; verify read-only schemas 3–6, rejected schemas unchanged, and the whole encoded response limit through real native owners on Linux/macOS/Windows.
- [x] 7.2 Canonicalize each refreshed source through daemon discovery and verify alias retargeting clears details and rejects old generations in the real native panel on all three systems.
- [x] 7.3 Form the sequential stack #14 → #17 → #18 → #19 → #20 → #16 → #15 with combined protocol 7 and require applicable native CLI/plugin/panel and tracked screenshot verification on all three systems before each sequential merge.
- [x] 7.4 Recover current-schema plugin runtime when the panel is the first owner client, without creating or migrating databases, admission, or historical events; verify deadlines, receipts, disabled plugins, and uncertain-work barriers through native processes and panel integration.

## Workflow follow-up

- The implementation is tracked in PR #15. A task is checked only after its stated implementation and verification exist; pending native desktop sessions remain explicit in docs/DESKTOP-VERIFICATION.md, and this change stays active until that acceptance exists.
- Delivery follows the unified sequential stack; the panel's observation contract remains read-only when plugins are configured.
- Do not merge without the user's instruction. Sync only verified behavior to baseline specifications and archive after implementation merge and the project's verification/review requirements are satisfied.
