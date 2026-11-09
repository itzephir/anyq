"""Exercise the native admission CLI; the caller explicitly runs its own work."""

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
        environment = {
            **os.environ,
            "ANYQ_DB": str(Path(directory) / "tasks.db"),
            "HOME": directory,
            "USERPROFILE": directory,
        }

        def queue(*args):
            subprocess.run([str(binary), *args], env=environment, check=True)

        for topic in ["large-builds", "itzdesign-repo-lock", "pixel-10-emulator"]:
            queue("topic", "create", topic)

        queue("topic", "--list")

        for name, topics, description in [
            ("build-revision-001", ["large-builds", "itzdesign-repo-lock"], "Build"),
            (
                "smoke-revision-001",
                ["pixel-10-emulator", "itzdesign-repo-lock"],
                "Test",
            ),
        ]:
            queue("task", "create", name)

            queue("task", "link", name, *topics)

            queue("task", name)
            queue("topic", topics[0])

            queue("task", "wait", name)

            try:
                # This process belongs to the caller. anyq only granted resource admission.
                subprocess.run(
                    [
                        sys.executable,
                        "-c",
                        f"print('{description} completed by the caller')",
                    ],
                    check=True,
                )
            finally:
                queue("task", "complete", name)

        queue("task", "create", "probe-001", "-twc", "large-builds")
        queue("task", "-l")
        queue("topic", "capacity", "large-builds")


if __name__ == "__main__":
    main()
