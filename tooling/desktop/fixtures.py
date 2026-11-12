"""Create isolated, explicitly configured real queues without inherited settings."""

import json
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
        "XDG_DATA_HOME": str(Path(home) / "xdg-data"),
        "XDG_CONFIG_HOME": str(Path(home) / "xdg-config"),
        "XDG_CACHE_HOME": str(Path(home) / "xdg-cache"),
        "ANYQ_DB": str(database),
        "TZ": "UTC",
        "PYTHONUTF8": "1",
    }


def directory_alias(path, target):
    if path.is_symlink():
        path.unlink()
    elif path.exists():
        assert os.name == "nt", "Only a Windows junction may be replaced"
        path.rmdir()

    if os.name == "nt":
        subprocess.run(
            ["cmd", "/c", "mklink", "/J", str(path), str(target)],
            check=True,
            capture_output=True,
        )
    else:
        path.symlink_to(target, target_is_directory=True)


def same_source_path(actual, expected):
    actual, expected = Path(actual), Path(expected)
    if actual.exists() and expected.exists():
        return actual.samefile(expected)

    def normalized(path):
        value = str(path.resolve())
        if value.startswith("\\\\?\\UNC\\"):
            value = "\\\\" + value[8:]
        else:
            value = value.removeprefix("\\\\?\\")
        return os.path.normcase(value)

    return normalized(actual) == normalized(expected)


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
        assert self.home.resolve() in path.resolve().parents, (
            "Fixture files must stay inside the isolated home"
        )
        path.parent.mkdir(parents=True, exist_ok=True)
        if path in self.paths:
            return path
        self.paths.add(path)
        if fixture["kind"] == "queue":
            self.call(["daemon", "start"], path)
            for command in fixture["commands"]:
                self.call(
                    [str(self.cli) if value == "@cli" else value for value in command],
                    path,
                )
            if plugin := fixture.get("ready_plugin"):
                deadline = time.monotonic() + 15
                while True:
                    status = json.loads(
                        self.call(["plugin", "status", plugin["name"]], path)
                    )
                    if (
                        status["running"]
                        and status["active_timers"] == plugin["active_timers"]
                    ):
                        break
                    assert time.monotonic() < deadline, status
                    time.sleep(0.02)
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
