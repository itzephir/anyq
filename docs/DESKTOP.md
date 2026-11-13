# Desktop queue panel

The optional `anyq-panel` application observes one existing local queue through
its daemon. It provides topics, current tasks, occupied/free slots, full task
memberships, and saved start preferences. Running means that slots are owned;
it does not describe an external process. Search, refresh, source selection,
and appearance settings do not change queue state.

## Selection and preferences

Database selection is panel override, inherited `ANYQ_DB`, shared `db.path`, then
the same platform default as the CLI. The config domain owns that precedence;
the config file adapter supplies the shared path resolver used by both clients.
The CLI's explicit `--db` still has first priority. Configured `~/` paths expand
as before; shell/environment paths keep the CLI's existing interpretation.

Preferences live in `~/.anyq/panel.toml`, independently of `~/.anyq/config.toml`:

```toml
database_override = "/absolute/path/to/queue.db"
theme = "system"
language = "en"
```

Settings accepts an existing database selected with the native file picker or a
manually entered path. Manual relative overrides are stored as absolute paths
before saving. Automatic selection removes the override. Saving uses an
exclusive preference-file lock and atomic replacement; a failed save preserves
the old active source. Malformed/unknown preferences are reported rather than
silently overwritten. The panel does not change the shell environment, shared
configuration, CLI selection, or install login items.

The settings screen displays the effective path and source. A GUI launched from
Finder, the Start menu, or a Linux launcher inherits that launcher's environment,
which can differ from a terminal's. For example, a terminal's `ANYQ_DB` export
usually is not inherited by an app opened separately from Finder. An explicit
panel override makes the selection independent of its launcher.

Only existing databases are opened. Missing, incompatible, inaccessible and
oversized targets report errors without falling back or creating a new queue.
Changing source clears previous rows/details immediately. Responses from an old
source generation cannot overwrite the current view. Both surfaces consume one
accepted snapshot. Visible polling is approximately once per second, hidden
polling once per five seconds; refresh requests coalesce behind one active
request. A ten-second watchdog cancels slow requests through the daemon's
bounded cancellation cleanup. Stale data is labeled and retained only for its
own source. Refresh/recovery never releases resource ownership.

## Native interaction

| Platform | System access | Window lifecycle |
| --- | --- | --- |
| macOS | Menu-bar icon opens a compact borderless panel; right-click opens its native menu | Clicking away dismisses the compact panel. Closing the main window keeps the icon; Open window or Dock activation restores it. Quit exits the client. |
| Windows | Notification-area icon opens a nearby compact flyout; right-click menu | Placement uses the icon's monitor, physical coordinates and work area, clamped to its bounds. Closing the main window keeps the tray client. Quit exits it. |
| Linux | AppIndicator/StatusNotifierItem native status menu with counts, Open window, Refresh, Settings and Quit | A compatible host is detected through the StatusNotifierWatcher service. Without a host, use the ordinary window; closing it exits. Host loss restores a reachable window. |

Linux's tray event adapter does not provide portable icon-coordinate callbacks,
so its compact access is a native menu rather than a macOS-style popup. KDE
Plasma and desktops with a StatusNotifier host are intended targets. GNOME needs
its AppIndicator integration/extension. X11/Wayland support depends on that host
and GTK/WebKit availability. Unsupported sessions retain the normal window.
Windows monitor/scale combinations and Linux desktop-host combinations still
need the manual acceptance runs listed in [DESKTOP-VERIFICATION.md](DESKTOP-VERIFICATION.md).

The client and owner have separate lifetimes. Quitting/uninstalling the panel
does not stop the owner, complete tasks, release slots, or delete queue data.
When no owner exists for an existing database, the bundled executable starts
its hidden `daemon run --db PATH` mode before initializing any GUI. The daemon
protocol must match the CLI/panel build. See [OBSERVATION.md](OBSERVATION.md).

## Build and packages

The renderer uses Tauri **2.11.6**, tauri-build **2.6.3**, dialog plugin **2.7.3**,
and the pinned npm CLI **2.12.4**. Cargo/npm lock files pin transitive versions.
It uses system WebKit on macOS, WebView2 on Windows, and WebKit2GTK 4.1 plus
AppIndicator on Linux. Node, Python and a local HTTP server are not application
runtime dependencies. Build prerequisites follow the official
[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) and
[system tray adapter](https://v2.tauri.app/learn/system-tray/) documentation.

The current package matrix builds macOS ARM64/Intel app bundles, a Windows x64
NSIS installer, and an Ubuntu 24.04 x64 Debian package. Review artifacts are
unsigned; signing/notarization and public desktop release are separate work.
The existing CLI installers and static Linux CLI distribution remain independent.

macOS review bundles use an explicit ad-hoc identity to seal the complete bundle;
package acceptance verifies the seal again after installation. Windows webview
profiles live in `~/.anyq/panel-webview/main` and `compact`, outside the installation
directory, so package removal and replacement do not own browser-profile files.
On Windows, a panel-started owner runs an immutable, hash-verified copy of the
panel executable at `~/.anyq/owners/<sha256>/anyq-owner.exe`. NSIS may remove or
replace the GUI executable without killing the queue owner or deleting an
executable that Windows still has mapped. Different panel builds retain separate
copies; removing the panel never removes a live owner's copy. Ordinary CLI
startup continues to launch its own executable.
The macOS shell uses the lock-file-pinned objc2 0.6.5 and AppKit bindings 0.3.2
for public screen/pointer coordinates. Private virtual-display interfaces and
the Windows virtual-display driver are CI tools only.

```sh
npm ci --ignore-scripts
# On Debian/Ubuntu, before compiling GUI features:
sudo apt-get install libwebkit2gtk-4.1-dev libayatana-appindicator3-dev libxdo-dev librsvg2-dev patchelf

cd apps/desktop
npx tauri build --features native --bundles app -- --locked   # macOS
# Use --bundles nsis on Windows, or --bundles deb on Linux.
```

Move `anyq.app` to Applications, run the NSIS installer, or install the generated
Debian package with the platform package manager. Remove the app/installer
package through the same platform mechanism. Preferences and databases are
preserved; remove panel.toml separately only when intentionally resetting panel
preferences. Package installation/removal on Windows/Linux is not yet manually
verified. CLI-only builds remain `cargo build --locked -p anyq` and do not
compile desktop dependencies.

## Design

[Editable OpenDesign project](https://opendesign.itzephir.com/projects/989a9d56-7944-4f7f-b2ae-3301735dcbd4/conversations/e6451dc1-603a-47df-ba5c-732f2739ff7d/files/design.html)
contains the window, compact view, settings, and connection states. Its source is
`apps/desktop/ui`, shared with the native application. Custom content follows
the personal product foundation: IBM Plex Sans/Mono, restrained neutral surfaces,
muted blue accents, light/dark/system themes, en/ru, keyboard focus and a narrow
window layout. Native window controls, menus and file pickers retain OS styling.
Icons use the licensed Lucide source pinned in the asset README; the app/tray
mark derives from the existing anyq favicon.

Rebuild the editable preview package with:

```sh
python3 tooling/export_desktop_design.py --output /tmp/anyq-desktop-design
```

Open `apps/desktop/ui/design.html` with a local static server for development.
OpenDesign's opaque authenticated iframe needs classic scripts and inline font
URLs; the exporter supplies those without changing the modular native source.
Preview fixtures contain synthetic queue data; the packaged application always
uses its Rust controller and IPC state.
