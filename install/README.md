# Installing prebuilt anyq

The scripts use GitHub Releases from itzephir/anyq. The first complete release must
be published before public installation can be verified.

macOS/Linux:

```sh
curl -fsSL https://anyq.itzephir.com/install.sh -o install-anyq && chmod +x install-anyq && ./install-anyq
./install-anyq --user --version v0.1.0 --bin-dir "$HOME/bin"
```

Windows x64 (PowerShell 5.1+):

```powershell
curl.exe -fL https://anyq.itzephir.com/install.ps1 -o install-anyq.ps1; if ($?) { powershell -NoProfile -ExecutionPolicy Bypass -File .\install-anyq.ps1 }
powershell -NoProfile -ExecutionPolicy Bypass -File .\install-anyq.ps1 -Scope User -Version v0.1.0 -BinDir "$env:USERPROFILE\bin"
```

ExecutionPolicy applies only to the new process; the persistent Windows policy
is unchanged. You can read the downloaded script before running it.
Supported platforms: macOS Intel/Apple Silicon, Linux x86_64/ARM64 (static musl), and
Windows x64 MSVC with a static C runtime, without WSL or a separate VC++ installation.
Rust, a C toolchain, Python, and Node are not required at runtime.

The archive and SHA256SUMS come from the same version. Before replacing the binary,
the installer verifies SHA-256, extraction, and --version. Failure preserves the old file. A running Windows
binary may block replacement: stop it and retry installation.

Installation is system-wide for all users by default:

| OS | Directory | Permissions and PATH |
| --- | --- | --- |
| macOS/Linux | /usr/local/bin | sudo requested only without write access; shell config unchanged |
| Windows | %ProgramFiles%\anyq | UAC when needed; system PATH updated |

Download, checksum verification, and --version run before elevation. Only
placement and system PATH updates use sudo/UAC. The copy is verified again
before atomic replacement. Declining elevation preserves the previous version.
A Windows PATH write failure rolls back binary replacement.

For installation for one user without elevation:

```sh
./install-anyq --user
```

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\install-anyq.ps1 -Scope User
```

--user selects ~/.local/bin; -Scope User selects %LOCALAPPDATA%\Programs\anyq and
the user PATH. --system / -Scope System explicitly select system mode.
--bin-dir / -BinDir override the selected mode's directory; ANYQ_BIN_DIR provides
an additional Unix default. User mode never invokes sudo/UAC, even for an
inaccessible directory: installation fails in that case.

Unix reports when the directory is absent from PATH. Windows PATH is extended without
duplicates within the selected scope; restart the terminal after installation.
-NoPathUpdate disables PATH changes. Queues, settings, and history are retained.
Archives contain `anyq[.exe]`, `plugins/shell[.exe]` and `plugins/timeout[.exe]`.
The published v0.1.0/v0.1.1 archives containing only the CLI remain installable,
including through latest resolution. Legacy installation leaves existing adjacent
plugin files untouched. An archive containing only one plugin is rejected, and
versions after v0.1.1 require both plugins before any installed file is replaced.
Keep this layout when unpacking a portable release. Installers place all three
verified files together and restore previous files if placement fails. On first
ordinary CLI or plugin-catalog use, missing first-party executables are copied to
`~/.anyq/plugins/`. Desktop bundles carry the same independent executables and
provision them on startup. No plugin is enabled implicitly; existing user files
are never overwritten. Use `anyq plugin enable shell` / `timeout` to opt in.

To uninstall, first use `anyq autolaunch off` if login launch is no longer wanted,
then delete the installed anyq executable and its adjacent `plugins` payload.
Optionally remove the added PATH directory. Per-user queues, plugin catalog and
retained launchers survive removal. To update a user-managed plugin, stop affected
owners and replace its file explicitly; on Windows an active binary is locked.

## Releases

After reviewing/merging the commit with the Cargo.toml version:

```sh
git tag v0.1.0 MAIN_COMMIT
git push origin v0.1.0
```

The workflow rejects tags outside main and mismatched Cargo versions. Five native
builds check real latest/pinned installation and download/checksum errors.
The complete set is uploaded to a draft before publication. Reruns resume the draft;
published releases are not overwritten. Apple/Windows signing is not configured.
SHA-256 verifies integrity and does not replace an independent publisher signature.

```sh
python3 tooling/package_release.py --target aarch64-apple-darwin
python3 tooling/test_installers.py --target aarch64-apple-darwin
```

CI adds --system to tests on disposable runners: it checks the actual
system directory, updates, and PATH, then removes only its own binary and
restores PATH. Local tests use temporary user paths;
they also check sudo refusal, UAC argument forwarding, and Windows PATH rollback.
Installer regressions download the actual published v0.1.0/v0.1.1 native archives
with the development-only GitHub CLI and verify SHA256SUMS before serving them
through an isolated HTTP fixture. Set GH_TOKEN on CI or use an authenticated gh
session locally. --published-archive-dir can reuse checksum-verified downloads
organized by release tag. These checks cover latest/pinned legacy installation,
preserved payloads/data, upgrade to the current layout and incomplete archives.

ANYQ_RELEASE_BASE_URL (Unix) / -ReleaseBaseUrl (Windows) specify a trusted mirror
for tests. HTTP is allowed only on loopback. The checksum and archive source
remains a trusted release source.
