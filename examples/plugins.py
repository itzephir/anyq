#!/usr/bin/env python3
"""Run the public observer protocol against an isolated local queue."""

import json
import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BINARY = ROOT / "target" / "debug" / ("anyq.exe" if os.name == "nt" else "anyq")


def main():
    with tempfile.TemporaryDirectory(prefix="anyq-plugins-") as directory:
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
            run(
                "plugin",
                "enable",
                "observer",
                sys.executable,
                str(ROOT / "examples" / "observer.py"),
            )
            deadline = time.monotonic() + 10
            while not json.loads(run("plugin", "status", "observer"))["running"]:
                if time.monotonic() > deadline:
                    raise RuntimeError("observer handshake timed out")
                time.sleep(0.02)
            run("task", "create", "example", "-t", "cpu")
            while json.loads(run("plugin", "status", "observer"))["lag"]:
                if time.monotonic() > deadline:
                    raise RuntimeError("observer acknowledgement timed out")
                time.sleep(0.02)
            status = json.loads(run("plugin", "status", "observer"))
            assert status["enabled"] and status["running"]
            run("task", "complete", "example")
            run("plugin", "disable", "observer")
            print("Observer acknowledged a committed running event.")
        finally:
            run("daemon", "stop")


if __name__ == "__main__":
    main()
