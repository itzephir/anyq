# System launch at login

```sh
anyq autolaunch        # Read current system registration: on or off.
anyq autolaunch on
anyq autolaunch off
```

These operations read/change OS registration directly, without opening a queue or
reading shared configuration. They remain available with damaged config.toml.
They never write an autolaunch preference to config.toml or panel.toml.
The desktop Settings control performs the same immediate system action.

The launcher runs `daemon start`, without a stored database path. At login it
resolves the current default using ANYQ_DB, then db.path, then the platform
location. Login environments commonly omit shell-specific environment variables;
use an absolute db.path for a stable default. Enabled executables in
`~/.anyq/plugins/` resume for that owner. Other databases require explicit
`anyq --db PATH daemon start`; there is no discovery or remembered-database list.
The desktop launcher handles this command without opening a window.

Native per-user registrations:

- macOS: `~/Library/LaunchAgents/com.itzephir.anyq.autolaunch.plist`, RunAtLoad
  with ProgramArguments and AbandonProcessGroup. The registration takes effect at
  the next login; the command does not bootstrap a launchd job immediately.
- Linux: `$XDG_CONFIG_HOME/autostart/anyq.desktop`, falling back to
  `~/.config/autostart/anyq.desktop`, using the XDG desktop autostart protocol.
  A fixed `/usr/bin/env --` executable passes the retained launcher as a quoted
  argument, so percent signs in profile paths survive GLib's executable check
  before desktop-field expansion. No shell interprets the command.
- Windows: an anyq profile-specific REG_SZ value in
  `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.

The launcher is an immutable verified copy under `~/.anyq/owners/BUILD_HASH/`.
This permits launching after an app bundle or CLI installation has been removed.
Enabling again registers the current build. Existing retained images are never
overwritten, including images in use on Windows. The launcher cache is not a
plugin directory and contains no plugin enablement or queue settings.

Updates stage and verify the retained image before publishing registration.
A failed update preserves the previous registration; a cancelled operation before
publication does too. A successful response reflects a fresh native read.
Malformed entries or missing launchers are reported as errors; `off` can remove
an owned broken registration. Disabling affects future login launches and does
not stop any daemon, plugin, task or GUI already running.

The status reflects the owned registration. OS policy, login/session restrictions,
permissions and external startup controls can prevent login execution. Native
integration tests inspect real isolated registrations and execute their recorded
commands; they do not simulate an actual machine reboot or user login.
