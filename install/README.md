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
To uninstall, delete anyq/anyq.exe and optionally remove the added PATH directory.

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

ANYQ_RELEASE_BASE_URL (Unix) / -ReleaseBaseUrl (Windows) specify a trusted mirror
for tests. HTTP is allowed only on loopback. The checksum and archive source
remains a trusted release source.
