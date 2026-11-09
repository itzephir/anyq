"""Native installer checks for privilege boundaries and machine-wide placement."""

import hashlib
import json
import os
import subprocess
from pathlib import Path


def powershell(command, environment=None):
    return subprocess.check_output(
        [
            "powershell.exe",
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            command,
        ],
        env=environment,
        text=True,
    )


def path_value(target):
    return json.loads(
        powershell(
            "[Environment]::GetEnvironmentVariable('Path', $env:ANYQ_PATH_TARGET) | ConvertTo-Json -Compress",
            {**os.environ, "ANYQ_PATH_TARGET": target},
        )
    )


def restore_path(target, value):
    powershell(
        "[Environment]::SetEnvironmentVariable('Path', $(if ($env:ANYQ_RESTORE_NULL -eq '1') { $null } else { $env:ANYQ_RESTORE_PATH }), $env:ANYQ_PATH_TARGET)",
        {
            **os.environ,
            "ANYQ_PATH_TARGET": target,
            "ANYQ_RESTORE_PATH": value or "",
            "ANYQ_RESTORE_NULL": str(int(value is None)),
        },
    )


def check_sudo_denial(install, root, original, checksum_file, valid_checksum):
    assert os.geteuid() != 0, "The sudo boundary fixture requires a non-root runner"
    protected = root / "protected installation"
    protected.mkdir()
    destination = protected / "anyq"
    destination.write_bytes(original)
    protected.chmod(0o555)
    fake_bin = root / "fake sudo"
    fake_bin.mkdir()
    sudo = fake_bin / "sudo"
    sudo.write_text('#!/bin/sh\nprintf "%s\\n" "$@" > "$ANYQ_SUDO_LOG"\nexit 1\n')
    sudo.chmod(0o755)
    log = root / "sudo-request"
    environment = {
        "PATH": str(fake_bin) + os.pathsep + os.environ["PATH"],
        "ANYQ_SUDO_LOG": str(log),
    }

    try:
        checksum_file.write_text("invalid checksum")
        install(
            scope=None,
            directory=protected,
            success=False,
            extra_environment=environment,
        )
        assert not log.exists(), "Bad downloads must not request sudo"
        checksum_file.write_text(valid_checksum)
        install(
            scope="User",
            directory=protected,
            success=False,
            extra_environment=environment,
        )
        assert not log.exists(), "User mode must not request sudo"
        install(
            scope=None,
            directory=protected,
            success=False,
            extra_environment=environment,
        )
        assert "/bin/sh" in log.read_text(), (
            "System mode did not request its placement worker"
        )
        assert destination.read_bytes() == original
        assert not list(protected.glob(".anyq*"))
    finally:
        protected.chmod(0o755)
        checksum_file.write_text(valid_checksum)


def check_windows_worker(installer, source, root):
    before = {target: path_value(target) for target in ("User", "Machine")}
    command = r"""
$ErrorActionPreference = 'Stop'
. $env:ANYQ_WORKER_INSTALLER
$source = $env:ANYQ_WORKER_SOURCE
$directory = Join-Path $env:ANYQ_WORKER_ROOT "worker O'Brien & spaces"
$request = [pscustomobject]@{
    Source = $source; BinDir = $directory; Scope = 'User'; NoPathUpdate = $true
    Hash = (Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant()
}
& powershell.exe -NoProfile -ExecutionPolicy Bypass -EncodedCommand (New-AnyqPlacementCommand $request)
if ($LASTEXITCODE -ne 0) { throw 'Encoded placement failed' }
$destination = Join-Path $directory 'anyq.exe'
if ((Get-FileHash -LiteralPath $destination -Algorithm SHA256).Hash.ToLowerInvariant() -cne $request.Hash) { throw 'Transport changed the binary' }
[IO.File]::WriteAllText($destination, 'previous binary')
$request.NoPathUpdate = $false
$denied = $false
try { Publish-Anyq $request -WritePath { throw 'PATH write fixture denied' } } catch { $denied = $true }
if (-not $denied -or [IO.File]::ReadAllText($destination) -cne 'previous binary') { throw 'PATH failure did not restore the binary' }
$request.Scope = 'System'
$denied = $false
try { Invoke-AnyqPlacement $request -Administrator:$false -Elevate { return 1 } } catch { $denied = $true }
if (-not $denied -or [IO.File]::ReadAllText($destination) -cne 'previous binary') { throw 'Elevation failure changed the binary' }
$request.Hash = '0' * 64
$denied = $false
try { Publish-Anyq $request } catch { $denied = $true }
if (-not $denied -or [IO.File]::ReadAllText($destination) -cne 'previous binary') { throw 'Placement hash failure changed the binary' }
if (Get-ChildItem -LiteralPath $directory -Force | Where-Object Name -like '.anyq*') { throw 'Worker temporary files leaked' }
"""
    powershell(
        command,
        {
            **os.environ,
            "ANYQ_WORKER_INSTALLER": str(installer),
            "ANYQ_WORKER_SOURCE": str(source),
            "ANYQ_WORKER_ROOT": str(root),
        },
    )
    assert {target: path_value(target) for target in before} == before


def check_system_install(install, windows, original):
    assert os.environ.get("GITHUB_ACTIONS") == "true", (
        "System installation tests run only on disposable CI runners"
    )
    before = {}
    if windows:
        administrator = json.loads(
            powershell(
                "([Security.Principal.WindowsPrincipal]::new([Security.Principal.WindowsIdentity]::GetCurrent())).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator) | ConvertTo-Json"
            )
        )
        assert administrator, "The Windows system fixture needs an administrator runner"
        prefix = json.loads(
            powershell(
                "$(if ($env:ProgramW6432) { $env:ProgramW6432 } else { [Environment]::GetFolderPath('ProgramFiles') }) | ConvertTo-Json"
            )
        )
        directory = Path(prefix) / "anyq"
        before = {target: path_value(target) for target in ("User", "Machine")}
    else:
        directory = Path("/usr/local/bin")
    destination = directory / ("anyq.exe" if windows else "anyq")
    assert not destination.exists() and not destination.is_symlink(), (
        "Refusing to overwrite a pre-existing system anyq"
    )
    created_directory = not directory.exists()
    previous = b"previous system binary"

    try:
        install(scope=None, directory=None, update_path=True)
        assert destination.read_bytes() == original
        try:
            destination.write_bytes(previous)
        except PermissionError:
            subprocess.run(
                ["sudo", "-n", "--", "/usr/bin/tee", str(destination)],
                input=previous,
                stdout=subprocess.DEVNULL,
                check=True,
            )
        install(scope=None, directory=None, update_path=True)
        assert destination.read_bytes() == original
        if windows:
            assert path_value("User") == before["User"]
            entries = path_value("Machine").split(";")
            assert (
                sum(
                    entry.rstrip("\\").casefold() == str(directory).casefold()
                    for entry in entries
                )
                == 1
            )
        else:
            assert destination.stat().st_mode & 0o777 == 0o755
    finally:
        if windows:
            for target, value in before.items():
                restore_path(target, value)
        if destination.exists():
            assert hashlib.sha256(destination.read_bytes()).digest() in {
                hashlib.sha256(original).digest(),
                hashlib.sha256(previous).digest(),
            }, "Refusing to remove an unexpected system binary"
            try:
                destination.unlink()
            except PermissionError:
                subprocess.run(
                    ["sudo", "-n", "--", "/bin/rm", "-f", "--", str(destination)],
                    check=True,
                )
        if windows and created_directory and directory.exists():
            directory.rmdir()
    assert not destination.exists()
