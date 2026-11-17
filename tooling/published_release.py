"""Fetch and verify immutable published archives for installer regressions."""

import hashlib
import re
import subprocess
import time
from pathlib import Path

from package_release import asset_name

LEGACY_VERSIONS = ("v0.1.0", "v0.1.1")


def published_archive(target, version, directory):
    directory = Path(directory) / version
    directory.mkdir(parents=True, exist_ok=True)
    archive = directory / asset_name(target)
    checksums = directory / "SHA256SUMS"
    if not archive.is_file() or not checksums.is_file():
        for attempt in range(3):
            result = subprocess.run(
                [
                    "gh",
                    "release",
                    "download",
                    version,
                    "--repo",
                    "itzephir/anyq",
                    "--pattern",
                    archive.name,
                    "--pattern",
                    "SHA256SUMS",
                    "--dir",
                    str(directory),
                    "--clobber",
                ],
                text=True,
                capture_output=True,
                timeout=90,
            )
            if result.returncode == 0:
                break
            if attempt == 2:
                raise RuntimeError(result.stderr)
            time.sleep(1)
    hashes = [
        fields[0]
        for line in checksums.read_text(encoding="ascii").splitlines()
        if len(fields := line.split()) == 2 and fields[1] == archive.name
    ]
    assert len(hashes) == 1 and re.fullmatch("[a-f0-9]{64}", hashes[0]), hashes
    assert hashlib.sha256(archive.read_bytes()).hexdigest() == hashes[0], archive
    return archive
