"""Wait for this fixture's native browser processes before removing its profile."""

import os
import subprocess
import sys
from pathlib import Path


def wait_for_renderers(home):
    if sys.platform != "win32":
        return
    script = """
$ErrorActionPreference = 'Stop'
$deadline = [DateTime]::UtcNow.AddSeconds(25)
do {
    $owned = @(Get-CimInstance Win32_Process -Filter "Name = 'msedgewebview2.exe'" | Where-Object {
        $_.CommandLine -and $_.CommandLine.IndexOf($env:ANYQ_RENDERER_HOME, [StringComparison]::OrdinalIgnoreCase) -ge 0
    })
    if (!$owned.Count) { exit 0 }
    if ([DateTime]::UtcNow -ge $deadline) { throw "Fixture WebView2 processes did not exit: $($owned.ProcessId -join ', ')" }
    Start-Sleep -Milliseconds 100
} while ($true)
"""
    subprocess.run(
        ["powershell", "-NoProfile", "-Command", script],
        env=os.environ | {"ANYQ_RENDERER_HOME": str(Path(home).resolve()) + os.sep},
        check=True,
        timeout=35,
    )
