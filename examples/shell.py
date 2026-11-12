#!/usr/bin/env python3
"""Run a platform-native shell plan using only the ordinary anyq binary."""

import json
import os
import subprocess
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BINARY = ROOT / "target" / "debug" / ("anyq.exe" if os.name == "nt" else "anyq")


def main():
    with tempfile.TemporaryDirectory(prefix="anyq-shell-") as directory:
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
            run("plugin", "enable", "shell", str(BINARY), "plugin-runtime", "shell")
            end = time.monotonic() + 15
            while not json.loads(run("plugin", "status", "shell"))["running"]:
                if time.monotonic() > end:
                    raise RuntimeError("shell plugin did not handshake")
                time.sleep(0.02)
            metadata = Path(directory) / "metadata.json"
            command = (
                "echo hello>result.txt"
                if os.name == "nt"
                else "printf hello > result.txt"
            )
            metadata.write_text(
                json.dumps(
                    {"shell": {"sh": [command], "cwd": directory, "autocomplete": True}}
                ),
                encoding="utf-8",
            )
            run("task", "create", "example", "-t", "cpu", "--metafile", str(metadata))
            while "completed" not in run("task", "example"):
                if time.monotonic() > end:
                    raise RuntimeError("shell did not complete the task")
                time.sleep(0.02)
            assert (Path(directory) / "result.txt").read_text().strip() == "hello"
            work = json.loads(run("plugin", "inspect", "shell"))
            assert len(work) == 1 and work[0]["state"] == "stopped"
            print(
                "Shell retained its outcome and completed after confirmed native exit."
            )
        finally:
            run("daemon", "stop")


if __name__ == "__main__":
    main()
