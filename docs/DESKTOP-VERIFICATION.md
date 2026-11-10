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

macOS is exercised locally on an isolated queue with multi-topic running and
queued tasks. Keep evidence for window launch, details, search, settings/reset,
missing source, refresh, menu-bar opening/dismissal, close/reopen and Quit.
Any discovered native event-loop blocking must be fixed before considering the
smoke run successful; never hold model locks while calling native menu APIs.

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
