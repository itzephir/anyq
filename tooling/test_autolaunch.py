"""Exercise native registration and its actual launcher with isolated queues."""

import argparse
import json
import os
import subprocess
import tempfile
import time
from pathlib import Path

from autolaunch_registration import entry


def wait(condition):
    deadline = time.monotonic() + 20
    while not condition():
        assert time.monotonic() < deadline, "Login-launch condition timed out"
        time.sleep(0.02)


def run(cli):
    with tempfile.TemporaryDirectory(prefix="anyq login O'Brien & % $-") as temporary:
        home = Path(temporary)
        environment = {
            key: value
            for key, value in os.environ.items()
            if not key.startswith("ANYQ_")
        }
        environment.update(
            HOME=str(home),
            USERPROFILE=str(home),
            XDG_CONFIG_HOME=str(home / "xdg-config"),
            XDG_DATA_HOME=str(home / "xdg-data"),
            LOCALAPPDATA=str(home / "local"),
        )
        default = home / "current.db"
        changed = home / "changed.db"
        custom = home / "custom.db"

        def call(*args, success=True):
            result = subprocess.run(
                [str(cli), *map(str, args)],
                env=environment,
                capture_output=True,
                text=True,
                timeout=20,
            )
            assert result.returncode == (0 if success else 2), (args, result.stderr)
            return result.stdout.strip()

        try:
            assert call("autolaunch") == "off"
            assert call("autolaunch", "on") == "on"
            registration = entry(home)
            assert registration is not None
            assert not default.exists() and not custom.exists()
            assert not (home / ".anyq/config.toml").exists()
            call("config", "db.path", str(default))
            call("--db", str(custom), "daemon", "start")
            call("--db", str(custom), "topic", "create", "cpu")
            call("--db", str(custom), "task", "create", "held", "-t", "cpu")
            call("--db", str(custom), "daemon", "stop")
            call("plugin", "enable", "timeout")
            # Login resolves the current default; it does not persist a database list/path.
            call("config", "db.path", str(changed))
            subprocess.run(
                registration[1],
                env=environment,
                check=True,
                timeout=30,
            )
            wait(lambda: "running" in call("--db", str(changed), "daemon", "status"))
            assert changed.is_file()
            assert not default.exists()
            assert call("--db", str(custom), "daemon", "status") == "daemon is stopped"
            call("topic", "create", "cpu")
            wait(lambda: json.loads(call("plugin", "status", "timeout"))["running"])
            call("task", "create", "expires", "-t", "cpu", "--meta", "timeout", "1ms")
            wait(lambda: "completed" in call("task", "expires"))
            owner = call("daemon", "status")
            assert call("autolaunch", "off") == "off"
            assert entry(home) is None
            assert call("daemon", "status") == owner
            # System control remains available even when ordinary anyq config is invalid.
            config = home / ".anyq/config.toml"
            config.write_text("broken = [", encoding="utf-8")
            assert call("autolaunch", "on") == "on"
            assert call("autolaunch") == "on"
            assert call("autolaunch", "off") == "off"
            assert config.read_text(encoding="utf-8") == "broken = ["
            assert not (home / ".anyq/panel.toml").exists()
        finally:
            call("autolaunch", "off")
            for database in (default, changed, custom):
                call("--db", str(database), "daemon", "stop")
    print("Native login registration and default-only launcher passed")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--cli", required=True, type=Path)
    run(parser.parse_args().cli.resolve(strict=True))
