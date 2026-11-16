"""Build and stage independent plugins as native desktop bundle resources."""

import os
import shutil
import subprocess
from pathlib import Path

root = Path(__file__).resolve().parents[1]
target = os.environ.get("CARGO_BUILD_TARGET")
command = [
    "cargo",
    "build",
    "--locked",
    "--release",
    "-p",
    "anyq",
    "--bin",
    "anyq-shell",
    "--bin",
    "anyq-timeout",
]
if target:
    command.extend(["--target", target])
subprocess.run(command, cwd=root, check=True)
source = root / "target"
if target:
    source /= target
source /= "release"
destination = root / "apps/desktop/resources/plugins"
destination.mkdir(parents=True, exist_ok=True)
suffix = ".exe" if os.name == "nt" else ""
for name in ("shell", "timeout"):
    shutil.copy2(source / f"anyq-{name}{suffix}", destination / f"{name}{suffix}")
