# Design

## Context

See [proposal.md](proposal.md) for motivation and delivery scope. This change
depends on the verified implementation of
[`introduce-queue-daemon`](../introduce-queue-daemon/design.md); its planning
artifacts alone do not satisfy that prerequisite.

Today, ListTopics returns topics, and GetTopicQueue reads queued/running tasks for
one topic in its own transaction. Repeating those operations cannot guarantee
that all displayed counters and memberships describe the same committed state.
GetTopicQueue can also use autocreate policy, which is unsuitable for a strictly
observational desktop view. A task may hold several topics, and running describes
admission rather than external process liveness.

CLI resolves --db/ANYQ_DB before shared db.path and platform defaults. Shared
configuration validates a fixed set of keys; desktop-only preferences must not
be inserted into that file. The reference screenshots establish two interaction
surfaces: a normal desktop window and a compact panel opened from a system icon.

## Goals / Non-Goals

**Goals:** use one owner-served model for both surfaces, preserve queue/domain
boundaries, and make data freshness and the active database unambiguous. Native
window chrome and system-panel behavior belong to platform adapters.

**Non-Goals:** direct database browsing, admission diagnostics inferred by the UI,
an event subscription framework, a browser server, or queue/execution management.
The panel's preference writes and startup of an owner for an existing database
are distinct from queue business mutations.

## Decisions

### Separate optional desktop client

Add an optional desktop application target with its own composition root. Keep
the existing CLI/daemon install usable without GUI dependencies or a graphical
session. Desktop packaging may distribute a separate application bundle without
changing the existing CLI command forms or requiring a Node/Python runtime.

Use one shared Rust application model/controller layer and a cross-platform
renderer for in-window content. Put menu-bar/tray integration, file selection,
window activation, accessibility integration, and preference IO in external
adapters. The exact toolkit is an adapter-level dependency selection, not a new
domain or daemon coupling. Before implementation, record the chosen toolkit,
supported OS versions, packaging requirements, and evidence that it supports
the specified native interactions.

Create api/impl boundaries for desktop application behavior and consumer-owned
ports for preferences and shell integration. Cross-feature business dependencies
are limited to observation and configuration-read contracts; panel-local ports
provide preference writes and native shell actions. Wire implementations explicitly
in the desktop composition root and extend architecture checks to recognize it.
Inner queue/config code gains no UI, serialization, threading, or platform SDK
dependencies. Do not introduce a generic business command dispatcher.

```text
main window -----+
                 +--> desktop model --> queue observation API
compact summary -+          |                     |
                            v                     v
                    local preferences       daemon client adapter
                                                  |
                                                  v
                                           existing queue owner
                                                  |
                                                  v
                                      observation use case --> SQLite
```

Separate native applications for every OS would duplicate selection/refresh
behavior. A browser-hosted dashboard would add a server and would not itself
provide the requested desktop lifecycle. Shared content with native shell
adapters keeps those concerns separate.

### One consistent current snapshot

Add a separate typed ObserveQueue contract and result in queue/api, an interactor
in queue/impl/dispatch, and a consumer-owned observation persistence port. Keep
the existing ListTopics/GetTopicQueue contracts and CLI output intact. The daemon
client implements ObserveQueue, and the owner exposes it as one additional
versioned wire operation.

The interactor requests all topics and the exact domain-owned current states
(queued/running) in one short transaction. It builds a normalized snapshot with
each task stored once, full topic membership, registration sequence, saved task
autostart default, explicit stop, topic capacity, and nullable topic autostart.
Topic queues and usage are derived from those same entities. Global totals count
distinct tasks rather than summing per-topic totals. Do not maintain new mutable
slot counters or reproduce admission rules in SQL or the UI.

The external response envelope carries canonical database identity and request
correlation. Client freshness timestamps belong to the desktop adapter, not the
queue domain. Snapshot restoration preserves canonical names without parsing
them as CLI input, and the renderer treats names as plain text.

Finish the read transaction before writing to a client. Enforce collection and
encoding bounds consistent with the daemon frame budget and fail explicitly if
the complete current snapshot cannot fit. Version one does not add pagination,
held database cursors, event sourcing, or hooks. A bounded full snapshot is a
deliberate simplicity trade-off for the small local panel; an oversized queue is
a visible error rather than a silently incomplete view.

### No mutation authority in the panel

The controller receives no create/link/start/complete/capacity/autostart or
daemon-stop handles. ObserveQueue has no AutoCreatePolicy input and never invokes
the admission pass. Missing selections are presentation changes, not entity
lookups that can create objects. Pure snapshot projection handles topic/task
search and selection without changing domain ordering or identities.

Reuse/start the daemon through the prerequisite's private discovery adapter only
for an existing selected database. Add an existing-database startup guard at the
owner boundary so a missing path, including a disappearance during startup,
cannot create a replacement database. The owner retains compatibility checking
and any supported schema migration; the desktop does not inspect SQLite itself.
An absent target produces an error with access to settings. Unsupported startup
or observation capabilities produce an incompatibility state, without direct IO
fallback or an attempt to stop/replace another owner.

### One model, two presentations

The main window has a searchable topic list and a selected-topic area containing
the current queue and expandable task details. Preserve global registration order
inside each topic; completed and registered tasks are outside the current view.
A selected task that leaves the current snapshot loses its current-detail view.
An explicit stopped flag is shown without inventing another persisted TaskState.

The compact panel uses the same accepted snapshot for connection state, distinct
running/queued totals, and a short topic list with slot usage. Limit the visible
topic list to the available space and provide Open Window for the complete list.
Opening a topic focuses the one main window and applies that topic selection.
The compact limit is a presentation choice, not a truncated data response.

Inherit the approved personal-project design foundation referenced by AGENTS.md:
IBM Plex Sans/Mono for custom content, restrained neutral surfaces, a muted blue
accent, and light/dark semantic tokens. Use native window chrome and native tray
menus where the OS draws them. Adapt the layout to a narrow desktop window with
reachable panes/details, keyboard focus, accessible labels, and localized en/ru
states. Mobile application delivery is outside this desktop change. Document
these adaptations in the panel's own design guidance during implementation.

### Platform-specific entry points

| Platform | Adapter behavior |
| --- | --- |
| macOS | Menu-bar icon opens a popover-style compact panel; Open Window focuses a normal resizable window. Native popover dismissal leaves the client running. |
| Windows | Notification-area icon opens a compact flyout near its icon, constrained to the available monitor work area; provide a native context menu with Open Window and Quit. |
| Linux | Use an available StatusNotifierItem/AppIndicator host. Export a concise status menu and Open Window/Quit; use activation to focus the main window where supported. A custom anchored popover is not a universal Linux requirement. |

Close Window retains the client while it has a usable system-panel entry. Quit
closes the desktop client only. If no usable tray host exists, launch the main
window and let its last-window close exit the client instead of leaving an
inaccessible background process. If the host disappears while every window is
closed, make the main window accessible again. Detect support at runtime rather
than inferring it from a distribution name, and test Linux with and without a
host. Login autostart and OS-service installation are deferred.

Platform references for implementation:

- [Apple popover-style menu-bar content](https://developer.apple.com/documentation/swiftui/menubarextrastyle/window).
- [Microsoft notification icon and rich flyout sample](https://learn.microsoft.com/en-us/windows/win32/shell/samples-notificationicon).
- [StatusNotifierItem specification](https://specifications.freedesktop.org/status-notifier-item/latest/status-notifier-item.html).
- [GNOME AppIndicator integration](https://github.com/ubuntu/gnome-shell-extension-appindicator).

### Database settings mirror CLI selection

Use a typed panel preference with an optional database override. Store desktop
preferences separately in `~/.anyq/panel.toml` (under USERPROFILE on Windows),
using atomic replacement and coordinated writes. The shared config schema and
`~/.anyq/config.toml` remain unchanged. Theme/language preferences can use the same
panel-local storage with validated values.

Resolve sources through shared path-selection policy rather than copying CLI
precedence into UI handlers:

```text
panel override --> inherited ANYQ_DB --> shared db.path --> platform default
```

The override is the panel equivalent of CLI --db. Use the same source-specific
path validation, expansion, and platform defaults as CLI, then canonicalize via
daemon discovery. The file picker saves an absolute override; normalize a manually
entered relative override when it is applied so later launches do not reinterpret
it under a different working directory. Display the effective absolute path and
source in settings, keeping database identity visible in the main window.

Read inherited ANYQ_DB from the desktop process environment. Launching from a
terminal and from the desktop may provide different environments; do not source
shell startup files to simulate another process's environment. Re-evaluate
automatic selection on launch and when automatic mode is explicitly applied;
ordinary queue polling stays attached to the selected database. Environment
changes require relaunching the client.

Validate and atomically save a changed preference before making it active. A
failed save retains the previous saved and active selection. A saved but
unreachable database becomes the selected error state; do not revert to another
path. Reset removes only the panel override and re-runs automatic resolution.

### Refresh, switching, and recovery

One controller owns the active database, local selection, latest accepted
snapshot, last-success time, and connection state for both surfaces. Start with
polling through ObserveQueue, without subscribing to future hook deliveries.
Use one in-flight observation per active source, with a finite request deadline.
A manual refresh or opening a surface coalesces with the next read. A one-second
visible interval and a slower background interval are initial tuning defaults;
keep request deadlines and retries bounded and avoid overlapping read loops.

Each source change increments a local generation. Cancel the old request, clear
old selections and cached values from the active view, and enter loading for the
new target. Apply a response only if its generation, request correlation, and
canonical source identity still match. A late A response cannot update B even if
both have a topic with the same name. Switching never stops the old owner.

A failed or overdue refresh retains an explicitly stale snapshot only for the
same source, with its last successful update time. The absence of a previous
snapshot produces an error/disconnected state, not an empty queue. Retry reads
with bounded backoff and reuse/start the selected existing owner under the same
guard. A successful snapshot replaces all counters and rows together and clears
staleness. No persisted queue revision or business-schema change is needed.

## Risks / Trade-offs

- [Linux host differences] -> require a working native status menu and ordinary
  window fallback; verify KDE-compatible hosting, GNOME integration where
  available, and a session without a tray, including supported Wayland/X11 cases.
- [Toolkit limitations] -> keep it behind external adapters and verify native
  focus, accessibility, scale, flyout placement, and release packaging before
  committing to the dependency; document any adapter-specific implementation.
- [Polling cost and response size] -> use short snapshots, bounded materialization,
  nonoverlapping requests, slower hidden refresh, and explicit oversize errors.
- [Wrong queue after a source change] -> visible source identity, stable saved
  overrides, generation checks, and late-response integration scenarios.
- [Read accidentally invokes autocreate/admission] -> a dedicated observation
  contract, no mutation handles in desktop controllers, and business-state
  equality checks with autocreate enabled.
- [GUI launch environment differs from CLI] -> show resolved path/source and
  provide an explicit local override; never guess a terminal's environment.

## Migration Plan

1. Verify the daemon prerequisite, then select the external UI/tray adapters and
   supported desktop environments without weakening the specified behavior.
2. Add and verify the observation use case and compatible wire operation while
   preserving existing CLI contracts and native interruption coverage.
3. Add desktop composition, local preferences, controller, main window, and native
   system-panel adapters as an optional installation.
4. Verify real-daemon observation/switching and native window/tray behavior, then
   document packaging, settings, Linux prerequisites, and design inheritance.
5. Release the optional desktop package only after the relevant native checks.
   Rollback removes the desktop client and its optional preferences without
   resetting queue data, CLI configuration, or running ownership.

## Open Questions

- Which maintained renderer and native integration libraries best satisfy the
  fixed adapter contracts, and which exact OS/desktop versions will the package
  support? Record dependency/version and packaging evidence in task 1.2 before
  feature implementation. These choices must preserve the Rust controller,
  daemon-only data access, and platform behaviors described above.
