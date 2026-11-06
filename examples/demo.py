"""Run a self-contained build -> emulator simulation on every supported OS."""

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path


def main():
    root = Path(__file__).resolve().parents[1]
    binary = Path(
        os.environ.get(
            "ANYQ_BIN",
            root / "target/debug" / ("anyq.exe" if os.name == "nt" else "anyq"),
        )
    )
    with tempfile.TemporaryDirectory(prefix="anyq-demo-") as directory:
        environment = {**os.environ, "ANYQ_DB": str(Path(directory) / "queue.db")}

        def queue(*args):
            subprocess.run([str(binary), *args], env=environment, check=True)

        queue("topic", "create", "large-builds")
        queue("topic", "create", "emulator-use")
        queue("group", "create", "large-builds", "builders")
        queue("group", "create", "emulator-use", "emulator-tests")
        queue(
            "publish",
            "large-builds",
            "--payload",
            json.dumps({"product": "demo", "revision": "abc123"}),
            "--key",
            "demo-abc123",
        )
        echo = "import sys; print(sys.stdin.read()); print('Simulated stage succeeded')"
        queue(
            "worker",
            "large-builds",
            "--group",
            "builders",
            "--consumer",
            "native-build",
            "--once",
            "--next-topic",
            "emulator-use",
            "--next-payload",
            json.dumps({"artifact": "demo.apk", "test": "smoke"}),
            "--",
            sys.executable,
            "-c",
            echo,
        )
        queue(
            "worker",
            "emulator-use",
            "--group",
            "emulator-tests",
            "--consumer",
            "emulator-1",
            "--once",
            "--",
            sys.executable,
            "-c",
            echo,
        )
        queue("jobs", "list")


if __name__ == "__main__":
    main()
