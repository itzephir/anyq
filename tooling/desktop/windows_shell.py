"""Hold the real Windows shell absent while testing host loss on disposable CI."""

import os
import subprocess
from contextlib import contextmanager
from pathlib import Path


@contextmanager
def stopped(home):
    assert os.environ.get("GITHUB_ACTIONS") == "true", (
        "Use a disposable Windows CI session"
    )
    release = Path(home) / "resume-explorer"
    release.unlink(missing_ok=True)
    script = """
$ErrorActionPreference = 'Stop'
while (-not (Test-Path -LiteralPath $env:ANYQ_EXPLORER_RELEASE)) {
    Get-Process -Name explorer -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 50
}
"""
    killer = subprocess.Popen(
        ["powershell", "-NoProfile", "-Command", script],
        env={**os.environ, "ANYQ_EXPLORER_RELEASE": str(release)},
    )
    try:
        yield
    finally:
        release.touch()
        try:
            assert killer.wait(timeout=15) == 0
        finally:
            subprocess.run(
                ["powershell", "-NoProfile", "-Command", "Start-Process explorer.exe"],
                check=True,
                timeout=15,
            )
