# Desktop acceptance evidence

The implementation is stacked as [daemon #14](https://github.com/itzephir/anyq/pull/14)
→ [observation #16](https://github.com/itzephir/anyq/pull/16)
→ [desktop #15](https://github.com/itzephir/anyq/pull/15). Each PR targets its
immediate predecessor; no baseline desktop behavior is declared complete yet.

Daemon prerequisite: commit `068ac48` passed [CI run 42804237207](https://github.com/itzephir/anyq/actions/runs/42804237207), including native process and interruption matrices.

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
  active source on failed persistence, and recover a stale snapshot.
- Architecture checks reject desktop SDKs in inner layers, cross-feature
  implementation imports and factories outside the desktop composition root.
- CI builds/packages the GUI on Linux, both macOS architectures and Windows,
  and runs its controller tests. The ordinary native process/Ctrl+C matrices
  remain mandatory and unchanged. Packaging checks do not prove GUI behavior.

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


The initial desktop implementation passed [CI run 42829086558](https://github.com/itzephir/anyq/actions/runs/42829086558)
on `563a74a`: native GUI packages for macOS ARM64/Intel, Windows and Linux;
controller tests; all existing process/Ctrl+C suites and CLI installers; lint and
website checks. Later changes require a new current-head CI pass before review.
