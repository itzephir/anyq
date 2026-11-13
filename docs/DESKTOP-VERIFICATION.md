# Desktop acceptance evidence

The delivery sequence is [daemon #14](https://github.com/itzephir/anyq/pull/14)
→ [metadata #17](https://github.com/itzephir/anyq/pull/17)
→ [hooks/plugins #18](https://github.com/itzephir/anyq/pull/18)
→ [timeout #19](https://github.com/itzephir/anyq/pull/19)
→ [shell #20](https://github.com/itzephir/anyq/pull/20)
→ [observation #16](https://github.com/itzephir/anyq/pull/16)
→ [desktop #15](https://github.com/itzephir/anyq/pull/15). Each PR follows its
predecessor into main after current-head CI. Combined observation and panel use
protocol 7. [PR #21](https://github.com/itzephir/anyq/pull/21) supplies the
remaining native shell, monitor and installed-package acceptance below.

Daemon prerequisite: commit `2ab8cb6` passed [CI run 42910987965](https://github.com/itzephir/anyq/actions/runs/42910987965), including native process and interruption matrices. The observation and desktop branches merge this daemon head, including request-timeout and Windows pipe-ownership corrections.

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

## Review regressions

Real owner-process cases reject both zero-byte files and empty unversioned SQLite
files before WAL changes or schema initialization, preserve their bytes and leave
no SQLite sidecars. They also verify that ordinary CLI startup can initialize the
same files and make observation available afterward.

Native application integration covers vertical and horizontal topic-list scroll
positions across CLI updates, and selecting a compact-view topic while the main
window has an unsaved settings draft. Navigation opens the requested queue and
discards the draft without saving it. The `main-dark-en` screenshot now reaches
its reviewed state through this navigation; all existing PNGs remain the verify
baseline. These regressions run in the required Linux/macOS/Windows matrices.

## Combined plugin runtime

Observation protocol 7 passed [CI run 43200921266](https://github.com/itzephir/anyq/actions/runs/43200921266)
on Linux, macOS and Windows before #16 merged. This includes real schema-compatible
observation, existing-only plugin recovery, bounded responses and interruptions.

Local macOS process and native application integration pass with protocol 7:
113 feature workflows, 35 interruption scenarios, canonical alias retargeting,
and panel-first plugin recovery. The configured recovery fixture uses daemon
clock 1000 → 2001, two independent panel starts, and 40 tasks containing about
20 MiB of metadata. It verifies exactly one timeout completion, successor
admission, unchanged metadata, no historical running events, and an owner that
survives panel exit. The complete panel snapshot remains below 16 KiB.

Combined panel commit `a6e0600` passed [CI run 43210294059](https://github.com/itzephir/anyq/actions/runs/43210294059).
Linux, macOS and Windows passed both native application scenarios, including
canonical alias retargeting and panel-first plugin recovery, then verified all
45 existing PNGs twice from independent clean fixtures. The same run passed the
native CLI/plugin and Ctrl+C matrices, all four GUI package builds and CLI
distribution checks. Current-head CI remains mandatory for later commits.
This verifies the existing-only bootstrap requirement synchronized into the
plugin-runtime baseline. The native host, monitor and installed-package
acceptance is recorded below; change archival remains separate.

Windows fixtures compare actual file identity, including canonical paths with an
extended-length prefix. The disappearance scenario removes the target after
client discovery and before owner startup: current-schema runtime recovery
legitimately retains a SQLite handle, so deletion after startup is not a valid
session-free fixture on Windows. The stale-state fixture temporarily renames
the isolated queue's topics table, waits for the real owner's read error, and
restores it to verify recovery. It preserves task data and source identity
without renaming a file held open by the runtime. All scenarios remain mandatory
on every OS, with the same reviewed stale-state PNG.

## Native shell and installed-package acceptance

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

The macOS shell profile passed on `67c01d7` in
[CI run 43236739454](https://github.com/itzephir/anyq/actions/runs/43236739454).
The `desktop-shell-macos` artifact records actual menu-bar input, main-window
close/reopen, compact placement after moves across three WindowServer displays
(1x/2x scales and negative coordinates), Escape and focus-loss dismissal, the
native file picker, SystemUIServer restart and native-menu Quit. The same daemon
PID and exact task/topic/slot rows survive these transitions. These are direct
native checks, independent of the successful window and OpenDesign preview.

The Windows shell profile passed on `f4ef115` in
[CI run 43356127012](https://github.com/itzephir/anyq/actions/runs/43356127012).
The `desktop-shell-windows` artifact records actual notification-icon/menu input,
flyout placement and focus/dismissal after moves across three native outputs:
1024×768 at 100%, plus two 1920×1080 outputs at 150%, including a negative origin.
Explorer restart restores a reachable main window; the icon/menu recover and
native Quit preserves the same daemon PID and exact queue ownership. This shell
profile pass does not waive the run's remaining package/Linux/core failures.

Production package acceptance passed on `73a18bf` in
[CI run 43395626654](https://github.com/itzephir/anyq/actions/runs/43395626654)
for macOS ARM64/Intel app bundles, the Windows x64 NSIS installer and the Ubuntu
24.04 Debian package. Each `desktop-package-evidence-*` artifact records two
actual installation, native-window launch, removal and reinstallation cycles.
The same daemon PID, exact task/topic/slot rows and hashes of shared configuration
and panel preferences survive both cycles. Windows additionally verifies that
the live owner executable is the immutable, digest-matched copy outside the
installation directory; macOS verifies the installed bundle's resource seal.
The ordinary CLI is built independently. This run also passed all three native
CLI/interruption and screenshot matrices, plus macOS and Windows shell profiles;
the two Linux host profiles still failed, so that run's overall CI was red.

### Completed native acceptance

All five native shell profiles passed on `f728ca5` in
[CI run 43425023028](https://github.com/itzephir/anyq/actions/runs/43425023028):
macOS menu bar, Windows notification area, Xfce StatusNotifier on X11, GNOME
AppIndicator on Wayland, and X11 without a status host. Each
`desktop-shell-*` artifact contains the actual native-operation log and
`shell-integration.json`; only a completed scenario writes a passed report.
The X11 and GNOME reports verify both monitor roundtrips, menu Open window and
Quit, host loss, closing the restored last window while the host is absent,
and recovery through a restarted host/client. Windows exercises the equivalent
Explorer loss/restart. Every profile preserves the same daemon PID and exact
queue ownership. macOS exercises its automatically restarted SystemUIServer
and retains its separate native file-picker check.

The verified platform/package baseline is listed in
[DESKTOP.md](DESKTOP.md#verified-platform-baseline). X11 runs two real Xorg
outputs with the Xfce StatusNotifier host. GNOME 46.2 runs a real nested Wayland
compositor with 1x/2x outputs and its packaged AppIndicator extension; moves
wait for the specific target monitor and scale. Windows configures three
native outputs at 100%/150% through a signed, digest-pinned virtual display
driver. macOS uses three WindowServer outputs at 1x/2x. These are operating-system
outputs and actual input, rather than injected application geometry.

Native acceptance found and fixed macOS global/logical coordinate conversion,
compact-window sizing/clamping, the installed macOS bundle seal, and Windows
browser profiles and a live owner executable held inside the installation
directory. Windows now runs a digest-verified owner copy outside that directory,
so NSIS removal/reinstallation preserves the live owner. The native runners
also own their desktop configuration and accessibility bridge, wait for actual
cursor/window/menu/compositor state, and retain failure logs and screenshots.
The existing 45 reviewed screenshot goldens remain unchanged.

The same run passed the aggregate `CI passed` gate: all five shell profiles,
all four production-package install/removal profiles, native UI/daemon integration
and verification of the 45 tracked PNGs from two independent clean fixtures,
plus the Linux/macOS/Windows CLI/plugin and native Ctrl+C suites. Each package
report confirms two installation/removal cycles with one unchanged owner PID,
queue fingerprint and preference/configuration hashes. Later commits still
require their own current-head CI pass.

The original adapter capability check was planned before feature work. It was
completed late in PR #21 after features already existed. This is an explicit
sequencing deviation, not evidence that the original prerequisite happened
on time. The remaining capability, Linux-host, lifecycle and integrated native
acceptance tasks are closed against the actual results above. Other OS versions
and desktop hosts, including KDE Plasma, are not inferred to be verified.

The earlier local macOS menu-bar follow-up could not proceed because the screen was
locked. The main window was accessible before the lock; no additional tray or
file-picker acceptance is inferred from that partial session. Its isolated panel
and owner processes were stopped afterward. The hosted native shell evidence
above supersedes that local input limitation.

The initial desktop implementation passed [CI run 42829086558](https://github.com/itzephir/anyq/actions/runs/42829086558)
on `563a74a`: native GUI packages for macOS ARM64/Intel, Windows and Linux;
controller tests; all existing process/Ctrl+C suites and CLI installers; lint and
website checks. Later changes require a new current-head CI pass before review.
