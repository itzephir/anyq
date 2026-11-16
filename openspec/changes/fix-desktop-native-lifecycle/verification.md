# Verification

## Regression coverage

- The GNOME/Wayland scenario waits for observable keyboard focus before sending real Alt+F4 after host loss. The native closure and independent queue-owner assertions remain required.
- macOS retains the panel's verified executable outside the removable app bundle. Native package acceptance inspects the actual owner path, checks its content hash, verifies the same daemon after client quit and package removal, and queries the queue while the installation is absent. Both install/remove/reinstall cycles remain required.
- A retained-copy regression executes the cached binary after deleting its source installation and checks private executable permissions on Unix. Existing concurrent-publication and altered-cache checks remain enabled.
- Windows menu acceptance reacquires the visible notification button after a missed click during Explorer recovery and requires an actual popup owned by the target process within the existing deadline. It continues selecting real native menu items.

## Local validation

- Required workspace tests, native Ctrl+C tests, Clippy with all targets/features, Rustfmt, both architecture checkers, Ruff, strict OpenSpec validation and whitespace checks pass.
- All 115 real CLI/plugin workflow scenarios pass together on macOS, including both large-output cases. Before the IO fix, both large-output cases fail under `taskpolicy -b` after exceeding their unchanged 20-second process deadline; the fixture finishes with both failures in 26.71 seconds. With readiness wakeups, the same two background-policy cases pass in 4.59 seconds.
- The native macOS helper compiles and reads a live process's executable path. An isolated production-binary probe confirms that the retained owner serves the same running task after its source app directory is removed.

## Cross-platform native acceptance

[CI run 43760693335](https://github.com/itzephir/anyq/actions/runs/43760693335) passes for implementation commit `c643ed4d0d1ba2307fd51f6942b763bf9abafaef`, including the [required aggregate gate](https://github.com/itzephir/anyq/actions/runs/43760693335/job/132957574393) and development website deployment.

- [GNOME/Wayland](https://github.com/itzephir/anyq/actions/runs/43760693335/job/132954608665), X11, Linux without a tray host, [Windows shell](https://github.com/itzephir/anyq/actions/runs/43760693335/job/132954608613) and macOS shell scenarios pass.
- Production desktop package scenarios pass on [macOS Intel](https://github.com/itzephir/anyq/actions/runs/43760693335/job/132954608544), macOS ARM64, Linux and Windows, including the stronger owner checks.
- All native screenshot comparisons pass on [Linux](https://github.com/itzephir/anyq/actions/runs/43760693335/job/132954608670), [macOS](https://github.com/itzephir/anyq/actions/runs/43760693335/job/132954608655) and [Windows](https://github.com/itzephir/anyq/actions/runs/43760693335/job/132954608718) against the existing reviewed PNG baselines. No goldens or comparison thresholds changed.
- The ordinary macOS CLI integration job previously timed out repeatedly in two large-output cases while the owner was sending a response. Both cases also fail locally under `taskpolicy -b`, establishing that timer-only polling can exceed the existing deadline under background scheduling. Native IO now wakes the waiting thread on readiness. The [macOS integration job](https://github.com/itzephir/anyq/actions/runs/43760693335/job/132954608450) passes all 115 workflow cases, both background-policy cases in 1.94 seconds, and all 35 native Ctrl+C cases. Linux and Windows integration jobs also pass. Test deadlines and coverage remain unchanged.
