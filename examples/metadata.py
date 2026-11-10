"""Exercise metadata without installing or executing plugins."""

import json
import os
import subprocess
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

    with tempfile.TemporaryDirectory(prefix="anyq-metadata-") as directory:
        path = Path(directory)
        environment = {
            **os.environ,
            "ANYQ_DB": str(path / "queue.db"),
            "HOME": directory,
            "USERPROFILE": directory,
        }

        def queue(*args):
            return subprocess.check_output(
                [str(binary), *args], env=environment, text=True, timeout=20
            ).strip()

        source = path / "metadata.yaml"
        source.write_text("timeout: 1h\nunknown: {tags: [local, example]}\n")

        try:
            queue("topic", "create", "cpu", "--meta", "timeout", "30m")
            queue(
                "task",
                "create",
                "build",
                "-t",
                "cpu",
                "--stop",
                "--metafile",
                str(source),
                "--meta",
                "timeout",
                "2h",
            )
            assert json.loads(queue("task", "build", "meta")) == {
                "timeout": "2h",
                "unknown": {"tags": ["local", "example"]},
            }
            queue("task", "meta", "build", "unknown.enabled", "true")
            queue("task", "build", "meta", "--unset", "unknown.tags")
            queue("task", "start", "build")
            assert json.loads(queue("topic", "cpu", "meta")) == {"timeout": "30m"}
            queue("task", "complete", "build")
            print(queue("task", "build", "meta"))
        finally:
            queue("daemon", "stop")


if __name__ == "__main__":
    main()
