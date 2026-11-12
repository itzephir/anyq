#!/usr/bin/env python3
"""Observe real opt-in timeout completion in an isolated queue."""

import json
import os
import subprocess
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BINARY = ROOT / "target" / "debug" / ("anyq.exe" if os.name == "nt" else "anyq")


def main():
    with tempfile.TemporaryDirectory(prefix="anyq-timeout-") as directory:
        environment = dict(os.environ, HOME=directory, USERPROFILE=directory)
        database = str(Path(directory) / "queue.db")

        def run(*arguments):
            return subprocess.check_output(
                [str(BINARY), "--db", database, *arguments],
                env=environment,
                text=True,
                timeout=15,
            )

        try:
            run("topic", "create", "cpu")
            run("plugin", "enable", "timeout", str(BINARY), "plugin-runtime", "timeout")
            end = time.monotonic() + 15
            while not json.loads(run("plugin", "status", "timeout"))["running"]:
                if time.monotonic() > end:
                    raise RuntimeError("timeout plugin did not handshake")
                time.sleep(0.02)
            run("task", "create", "example", "-t", "cpu", "--meta", "timeout", "100ms")
            while "completed" not in run("task", "example"):
                if time.monotonic() > end:
                    raise RuntimeError("timeout did not complete the task")
                time.sleep(0.02)
            print("Timeout completed the task through the public plugin protocol.")
        finally:
            run("daemon", "stop")


if __name__ == "__main__":
    main()
