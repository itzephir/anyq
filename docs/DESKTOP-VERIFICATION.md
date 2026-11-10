# Desktop acceptance evidence

The implementation is stacked as [daemon #14](https://github.com/itzephir/anyq/pull/14)
→ [observation #16](https://github.com/itzephir/anyq/pull/16)
→ [desktop #15](https://github.com/itzephir/anyq/pull/15). Each PR targets its
immediate predecessor; no baseline desktop behavior is declared complete yet.

Daemon prerequisite: commit `b6fb8b7` passed [CI run 42896501151](https://github.com/itzephir/anyq/actions/runs/42896501151), including native process and interruption matrices. The observation and desktop branches merge this daemon head rather than relying on the earlier implementation.

## Automated coverage

- Queue/SQLite/real-owner observation suites cover current-state projection,
  consistent concurrent reads, distinct totals, complete membership, bounded
  collection, denied writes and existing-only opening.
- The shared config policy tests all eight precedence combinations; the existing
  real CLI path/environment/config fixtures preserve CLI behavior.
- Preference file tests cover save/restart/reset, unchanged shared config,
  invalid values, malformed future files and atomic failure preservation.
- Controller tests block an old source response, coalesce 100 refresh requests,
  switch sources with the same topic identity, reject stale data, preserve the
  active source on failed persistence, and recover a stale snapshot. A blocked
  read also reaches the real controller deadline, observes cancellation without
  overlapping refreshes, and recovers to a live snapshot with a fixed display clock.
- Architecture checks reject desktop SDKs in inner layers, cross-feature
  implementation imports and factories outside the desktop composition root.
- CI builds/packages the GUI on Linux, both macOS architectures and Windows,
  and runs its controller tests. A separate required Linux/macOS/Windows matrix
  drives the real app and daemon through integration and native screenshot
  scenarios, comparing tracked platform PNGs; see [desktop tests](../tests/desktop/README.md). The ordinary native process/Ctrl+C matrices
  remain mandatory and unchanged. Packaging checks do not prove GUI behavior.

## Native screenshot acceptance

The Linux, macOS and Windows jobs in [generator run 42888007910](https://github.com/itzephir/anyq/actions/runs/42888007910)
passed real native application/daemon integration and 15 configured visual states,
each repeated in two independent clean processes and databases. All 45 reviewed
PNG files are recorded in [the golden directories](../tests/desktop/goldens);
downloaded ZIP digests matched GitHub's artifact digests. Full [PR CI run 42891577193](https://github.com/itzephir/anyq/actions/runs/42891577193)
passed on `afd232f`; the restacked `088007e` also passed [CI run 42900003085](https://github.com/itzephir/anyq/actions/runs/42900003085), including native integration and verification against the
committed goldens on all three operating systems, four GUI package builds and
the ordinary native process/Ctrl+C matrices. Later commits must pass the same
required PR workflow; this evidence does not waive current-head checks.
Candidate generation does not satisfy the required `CI passed` gate.

Native checks found and fixed Windows file-sharing races in test transport and
fixture cleanup, GTK's incompatible ephemeral-context protocol registration,
macOS native scrollbar animation, and WebKit's inconsistent word-boundary kerning.
The kerning failure changed a line's advance by 13 font units despite identical
text and loaded fonts. The product now sets its text policy explicitly; tests
assert that policy and retain measured line rectangles and font metadata.

Observation prerequisite: `09ec5ed` passed [CI run 42900909651](https://github.com/itzephir/anyq/actions/runs/42900909651)
on Linux/macOS/Windows, including native process and interruption suites. Its
existing-only disappearance fixture joins the old owner before launching a fresh
owner without a SQLite session and also passed 20 independent local native runs.

Dedicated observation process tests cover a foreign database identity,
unsupported versions, cancellation, a snapshot with over 1 MiB of encoded
Unicode/escaped names below the raw-text limit, oversized-text rejection and a
slow reader alongside concurrent commands and owner shutdown. The capacity
scenario verifies that 64 read-only sessions cannot consume the reserved
lifecycle quota. Native Linux/macOS/Windows CI passed these cases and all 73 CLI
workflows and 27 interruption scenarios; stopping the owner preserves slots.

## Manual native acceptance

The local macOS application bundle was launched on an isolated queue with two
running tasks, two queued tasks and three topics. Native accessibility inspection
and screenshots verified task details/full membership, task search, settings,
persisted English/Russian and dark/light choices, selection of a missing file,
explicit missing-source error, source-generation clearing, reset to ANYQ_DB,
main-window close/reopen, owner restart/reconnection and Cmd+Q. The original
owner PID remained alive across window closure; after stopping it deliberately,
the panel launched its own hidden owner mode and reconnected. Task states and
slot usage were unchanged after client exit; the missing target was not created.

A lock-ordering issue found during this smoke run was fixed: native menu updates
now happen after releasing the model lock. Unchanged polling results preserve
DOM nodes and keyboard focus. [Dark native window](images/desktop-macos-dark.jpg)
and [Russian missing-source state](images/desktop-macos-missing-ru.jpg) retain
reviewable screenshot evidence using synthetic queue data.

The menu-bar icon click, compact panel dismissal/placement, and native file
picker still require their own direct native smoke evidence. They are not
inferred from the successful window or OpenDesign preview.

Pending native configurations remain explicit:

- Windows: real notification-area activation, right-click menu, flyout focus,
  work-area placement at different scale factors and monitor/taskbar positions,
  Explorer restart, last-window closure, installer launch/removal and owner
  survival after client exit.
- Linux: a real compatible StatusNotifier host, GNOME with AppIndicator
  integration, host disappearance, no-host window fallback, supported X11 and
  Wayland sessions, package launch/removal and owner survival after client exit.

These rows require actual desktop sessions. A build, mock or headless controller
test is not a substitute. Corresponding OpenSpec tasks remain unchecked until
that evidence exists; the implementation PR remains a draft meanwhile.

The local macOS menu-bar follow-up could not proceed because the screen was
locked. The main window was accessible before the lock; no additional tray or
file-picker acceptance is inferred from that partial session. Its isolated panel
and owner processes were stopped afterward.

The initial desktop implementation passed [CI run 42829086558](https://github.com/itzephir/anyq/actions/runs/42829086558)
on `563a74a`: native GUI packages for macOS ARM64/Intel, Windows and Linux;
controller tests; all existing process/Ctrl+C suites and CLI installers; lint and
website checks. Later changes require a new current-head CI pass before review.
