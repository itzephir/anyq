"""Create isolated, explicitly configured real queues without inherited settings."""

import os
import subprocess
import time
from pathlib import Path


def environment(home, database):
    inherited = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith(("ANYQ_", "WEBVIEW2_"))
    }
    return inherited | {
        "HOME": str(home),
        "USERPROFILE": str(home),
        "ANYQ_DB": str(database),
        "TZ": "UTC",
        "PYTHONUTF8": "1",
    }


class Fixtures:
    def __init__(self, cli, home, configuration):
        self.cli = Path(cli).resolve()
        self.home = Path(home)
        self.home.mkdir(parents=True, exist_ok=True)
        self.configuration = configuration
        self.paths = set()
        self.environment = environment(self.home, self.home / "queue.db")
        for command in configuration["shared_config_commands"]:
            self.call(command, self.home / "queue.db")

    def call(self, command, database):
        result = subprocess.run(
            [str(self.cli), "--db", str(database), *command],
            env=self.environment,
            capture_output=True,
            encoding="utf-8",
            timeout=20,
        )
        assert result.returncode == 0, (command, result.stderr)
        return result.stdout.strip()

    def create(self, name):
        fixture = self.configuration["fixtures"][name]
        path = self.home / fixture["file"]
        assert path.parent == self.home, (
            "Fixture files must stay inside the isolated home"
        )
        if path in self.paths:
            return path
        self.paths.add(path)
        if fixture["kind"] == "queue":
            self.call(["daemon", "start"], path)
            for command in fixture["commands"]:
                self.call(command, path)
            self.call(["daemon", "stop"], path)
        elif fixture["kind"] == "bytes":
            path.write_bytes(fixture["content"].encode("utf-8"))
        else:
            assert fixture["kind"] == "missing"
            assert not path.exists()
        return path

    def stop(self):
        for path in self.paths:
            self.call(["daemon", "stop"], path)
            deadline = time.monotonic() + 10
            while "running" in self.call(["daemon", "status"], path):
                assert time.monotonic() < deadline, "Fixture owner did not stop"
                time.sleep(0.02)
