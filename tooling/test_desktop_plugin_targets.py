"""Build real cross-target and universal plugins through the production hook."""

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

from prepare_desktop_plugins import MACOS_TARGETS, ROOT, host_target, verify_macos


def handshake(path, name):
    frames = [
        {
            "type": "hello",
            "version": 2,
            "plugin": name,
            "max_frame_bytes": 16777216,
            "state": {"object": []},
        },
        {"type": "stop"},
    ]
    result = subprocess.run(
        [str(path)],
        input="".join(json.dumps(frame) + "\n" for frame in frames),
        capture_output=True,
        text=True,
        timeout=20,
        check=True,
    )
    hello = json.loads(result.stdout.splitlines()[0])
    assert hello["type"] == "hello" and hello["version"] == 2, hello
    assert "task.running" in hello["subscriptions"], hello


def main():
    assert sys.platform == "darwin", "Cross-target Mach-O verification requires macOS"
    host = host_target()
    target = next(triple for triple in MACOS_TARGETS if triple != host)
    with tempfile.TemporaryDirectory(prefix="anyq-plugin-targets-") as temporary:
        directory = Path(temporary)
        for selected, architectures in [
            (target, (target,)),
            ("universal-apple-darwin", MACOS_TARGETS),
        ]:
            destination = directory / selected
            environment = {
                **os.environ,
                "TAURI_ENV_TARGET_TRIPLE": selected,
                "CARGO_BUILD_TARGET": host,
            }
            subprocess.run(
                [
                    sys.executable,
                    str(ROOT / "tooling/prepare_desktop_plugins.py"),
                    "--output",
                    str(destination),
                ],
                cwd=ROOT,
                env=environment,
                check=True,
            )
            for name in ("shell", "timeout"):
                binary = destination / name
                verify_macos(binary, architectures)
                if len(architectures) == 2:
                    handshake(binary, name)
    print("Cross-target and universal plugin architectures and native protocol passed")


if __name__ == "__main__":
    main()
