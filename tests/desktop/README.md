# Native desktop integration and screenshot tests

Every UI behavior change requires real native integration and screenshot checks
on Linux, macOS and Windows. The required CI matrix is in
`.github/workflows/desktop.yml`, included in the stable `CI passed` gate.

`tooling/test_desktop_package.py` installs the actual production app bundle,
NSIS installer or Debian package, launches the installed executable, quits the
client, removes the package and repeats installation/removal. It checks a real
visible native window, the same live daemon, unchanged queue ownership and
byte-identical preferences/shared configuration. Windows/Linux installation
requires a disposable GitHub Actions machine; macOS uses an isolated per-user
Applications directory. Shipping packages do not contain the UI test transport.

`tooling/test_desktop_shell.py` uses `native.json` to configure real desktop-host
and display scenarios. Operating-system helpers inspect native windows and
post real input independently of the private application driver. Native shell
state is read without evaluating a hidden webview. Platform helpers are test
tools only and are never included in app packages. macOS display setup uses
the private CoreGraphics interfaces also exercised by
[Chromium's display tests](https://chromium.googlesource.com/chromium/src/+/HEAD/ui/display/mac/test/virtual_display_util_mac.mm);
the production app uses public AppKit screen APIs. Native input and capture
permissions are environmental requirements, and a denied operation fails the
scenario rather than simulating a click or accepting incomplete evidence.

Windows display CI installs only the checksum-pinned driver-only archive from
[Virtual Display Driver 25.7.23](https://github.com/VirtualDrivers/Virtual-Display-Driver/releases/tag/25.7.23)
on its disposable runner, verifies the signed catalog, and registers the UMDF
device through native SetupAPI. It never disables driver-signature enforcement
or enables test signing. Display setup logs and signer/digest information are
retained. Per-display DPI is configured through native DisplayConfig packets,
following the interfaces exercised by [SetDPI](https://github.com/imniko/SetDPI).
No driver, native input helper or test transport is shipped with the panel.

The Linux profiles use two real Xorg dummy outputs with the Xfce StatusNotifier
host, a no-host X11 session, and a nested GNOME Wayland compositor with two
virtual outputs and the packaged AppIndicator extension. The Wayland profile
uses compositor monitor-move shortcuts and toggles the extension to test host
loss without destroying the compositor connection. Absolute window placement is
not assumed on Wayland. Monitor mode/scale controls follow
[Mutter's dummy-output implementation](https://github.com/GNOME/mutter/blob/gnome-46/src/backends/meta-monitor-manager-dummy.c).
Native menu input waits for an actual visible menu. Windows reads the real
Win32 popup handle, item labels and screen rectangles, following
[pywinauto's popup adapter](https://github.com/pywinauto/pywinauto/blob/master/pywinauto/controls/win32_controls.py).
Linux reads native AT-SPI menu item bounds. Both use real pointer input;
macOS uses Home/End and Enter. Xfce uses the primary button; GNOME uses its
AppIndicator extension's secondary-button context menu. Notification-icon lookup excludes ordinary application/taskbar buttons
and restricts Windows accessibility searches to actual shell windows.

Windows fixture teardown waits for its own WebView2 processes to exit before
removing profile files. Test transport retries transient Windows sharing denials
within the existing response deadline; it never replays a request or accepts
an incomplete response. Linux D-Bus activation receives the private display,
configuration and Xauthority settings before starting desktop services.
AT-SPI queries dispatch accessibility events within a bounded interval, inspect
only the application and status-host processes without recursively invalidating
the desktop cache, and read descendant label text when a native menu container
has no own name. Windows enumerates visible Explorer windows, including the
separate overflow popup, before querying their accessibility trees.
Host-loss scenarios close the restored last window while the host is still
absent, verify client exit with unchanged owner/slots, then reconnect through
a restarted client and host.

The NSIS fixture passes its final `/D=` argument without quotes, including for
the configured installation path with spaces, as required by the
[NSIS command-line contract](https://nsis.sourceforge.io/Docs/Chapter3.html#installerusage).
This is a direct CreateProcess call with an explicit executable, without a shell.
macOS review bundles are ad-hoc sealed as complete bundles and checked again
after installation; Developer ID signing/notarization remains release work.

The modes are explicit: `verify` compares with committed goldens and fails on a
regression; `generate` writes new goldens for review. Both modes launch every
visual case twice from independent clean processes and databases, and reject
any visual instability under the configured comparison before accepting candidates.

`config.json` owns each state's CLI seed commands, expected complete snapshot,
fixed clock/timezone, source, preferences, viewport/decorations, selected topic/task,
scroll, focus, cursor, renderer arguments, masks and comparison thresholds. Cases
do not inherit earlier scenarios.
The configured capture deadline waits for three consecutive visually stable
native frames after fonts/layout are ready. It rejects an unsettled renderer;
it does not replace the separate repeatability check across clean processes.
Only machine-specific source-path text is masked and its actual identity is
asserted separately. The display clock and timezone are injected only into the
feature-gated test build. Timers still use real bounded deadlines; tests wait for
observable states rather than asserting arbitrary elapsed delays.

`tooling/test_desktop.py` launches the real CLI, owner and Tauri webviews using an
isolated home/database. `integration-test-hooks` adds a private file transport
for driving the actual renderer. macOS/Windows use incognito stores; Linux uses
an independent XDG data/cache/config directory for each fixture, preserving GTK's
shared URI-scheme registration for both windows. These exclude previous renderer
sessions. Inherited WebView2 settings are removed before applying configured
renderer arguments. These hooks are absent from shipping packages and do not
replace the daemon or its queue. Screenshots capture native window pixels,
not a browser preview or a DOM re-render. Queue fingerprints prove that viewing,
searching, refresh, settings and client exit do not change business state.

Scenarios cover current/empty queues, full membership, text escaping, search,
shared window/compact state, en/ru, both themes, narrow layout, missing/incompatible
sources, stale recovery, owner restart, concurrent CLI completion, oversized
results, source generations, preference restart/reset and close/reopen/quit.
Regressions also cover unchanged zero-byte files, vertical/horizontal topic-list
scroll positions across real CLI updates, and compact-topic navigation out of
settings without saving the abandoned draft. The `main-dark-en` screenshot is
reached through that settings-to-compact navigation before comparing its existing
reviewed PNG, rather than selecting the topic directly.
The stale fixture temporarily renames the isolated queue's topics table and
restores it after the real owner reports a read failure. It preserves task data
and source identity, works with Windows SQLite sharing, and retains the existing
reviewed stale-state PNG. It does not replace the owner or its observation adapter.

The canonical-alias scenario retargets a Unix directory symlink or Windows
junction between two configured databases with the same task sequence, checking
generation changes, cleared details, shared surfaces, and the old owner's lifetime.
A separate configured plugin scenario seeds a fixed daemon clock and persisted
timeout, then starts the owner through the actual panel before any queue RPC.
Expiry admits the successor exactly once over two panel-first starts, preserves
40 large metadata records, and creates no historical running events. The injected
daemon clock is enabled only in the integration build; native screenshots retain
their separate fixed display clock and existing reviewed PNGs.

PNG goldens are tracked in `goldens/linux`, `goldens/macos` and `goldens/windows`.
Only configured source-path regions are masked; the fixed update time remains
visible and comparable. Other content remains visible and comparable. The
product uses an explicit kerning policy to keep word-boundary glyph advances
stable across native sessions; capture metadata retains measured line rectangles
and loaded font faces as well as asserting the configured policy.
The visual configuration disables native window decorations to exclude the macOS
translucent rounded frame and its dependency on desktop pixels. Integration
scenarios retain ordinary decorated windows; the production window is unchanged.
The cursor is moved outside the client to exclude ambient hover state. Windows
visual cases configure software rendering, grayscale text and sRGB explicitly;
ordinary integration cases retain default WebView2 rendering. Native desktop
capture avoids PrintWindow's partially blank WebView2 surfaces. Configured
viewports fit the runner's visible work area without taskbar occlusion.
The product styles its scrollbars explicitly, avoiding macOS's asynchronous
native scrollbar expansion/fading during updates without masking scrollable content.
The pinned Pixelmatch comparator uses a strict 0.01 perceptual threshold and
antialias-edge detection, allowing zero differing visual pixels. This handles
measured native rasterization noise (single-channel changes at SVG/rounded edges),
not missing content, moved geometry or changed state. Both independent renders
must pass this comparison, and verify compares both against the golden. Raw pixel
difference counts remain in the JSON report; diff PNGs show antialiasing in yellow
and regressions in red. Missing goldens and dimension changes fail. Actual,
difference and JSON evidence are uploaded even when CI fails. No comparison job
writes or accepts golden changes.

To run locally after installing `tooling/requirements-desktop.txt`:

```sh
cargo build --locked -p anyq -p anyq-desktop --features anyq-desktop/integration-test-hooks,anyq-desktop/custom-protocol
python tooling/test_desktop.py --cli target/debug/anyq --panel target/debug/anyq-panel --mode verify
```

Use `.exe` binaries on Windows. Linux needs a graphical X11 session; CI uses
Xvfb, a real window manager and D-Bus. The ordinary-window/no-tray fallback is
exercised when no compatible host exists. Real GNOME/KDE/Wayland host behavior
remains additional native acceptance rather than being inferred from Xvfb.
macOS capture requires the test runner's screen-capture permission.

To update goldens intentionally, use `--mode generate` on the corresponding
native platform, inspect every changed PNG and commit the result in the PR.
The native UI workflow also exposes `verify`/`generate` through workflow_dispatch.
Before that workflow reaches main, invoke the existing CI workflow on the PR
branch with `desktop_goldens=true`; this runs only the native generator matrix.
It deliberately cannot satisfy `CI passed`, whose gate still rejects skipped
normal checks. Every PR always runs the complete required verify matrix and other
categories. Generation never
commits files or grants a CI pass for missing/unreviewed goldens.
The initial hosted-runner images may be collected from CI artifacts, inspected,
and committed. A missing initial baseline deliberately fails comparison until
that review exists. Never fabricate one platform's images from another platform.
