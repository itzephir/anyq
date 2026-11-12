"""Install real shipping packages and preserve the owner across removal and reinstall."""

import hashlib
import json
import os
import shutil
import sqlite3
import subprocess
import sys
import time
from contextlib import closing
from pathlib import Path

from fixtures import Fixtures
from native_session import NativeSession
from screenshots import platform_name


def fingerprint(database):
    with closing(sqlite3.connect(database)) as connection:
        return {
            table: connection.execute(f"SELECT * FROM {table} ORDER BY 1").fetchall()
            for table in ("topics", "tasks", "task_topics")
        }


class Package:
    def __init__(self, bundles, home):
        self.home = Path(home)
        if sys.platform == "darwin":
            self.source = next(Path(bundles).glob("macos/*.app"))
            self.installed = self.home / "Applications" / self.source.name
            self.executable = self.installed / "Contents/MacOS/anyq-panel"
        elif sys.platform == "win32":
            assert os.environ.get("GITHUB_ACTIONS") == "true", (
                "Use a disposable Windows CI machine"
            )
            self.source = next(Path(bundles).glob("nsis/*-setup.exe"))
            self.installed = self.home / "Installed panel with spaces"
            self.executable = self.installed / "anyq-panel.exe"
        else:
            assert os.environ.get("GITHUB_ACTIONS") == "true", (
                "Use a disposable Linux CI machine"
            )
            self.source = next(Path(bundles).glob("deb/*.deb"))
            self.name = subprocess.check_output(
                ["dpkg-deb", "--field", str(self.source), "Package"], text=True
            ).strip()
            self.executable = Path("/usr/bin/anyq-panel")

    def install(self):
        if sys.platform == "darwin":
            self.installed.parent.mkdir(parents=True, exist_ok=True)
            subprocess.run(
                ["ditto", str(self.source), str(self.installed)], check=True, timeout=60
            )
            subprocess.run(
                ["codesign", "--verify", "--deep", "--strict", str(self.installed)],
                check=True,
                timeout=30,
            )
        elif sys.platform == "win32":
            subprocess.run(
                subprocess.list2cmdline([str(self.source)])
                + f" /S /D={self.installed}",
                executable=str(self.source),
                check=True,
                timeout=90,
            )
        else:
            subprocess.run(
                ["sudo", "dpkg", "--install", str(self.source)], check=True, timeout=60
            )
            files = subprocess.check_output(
                ["dpkg-query", "--listfiles", self.name], text=True
            )
            assert str(self.executable) in files.splitlines()
            assert any(path.endswith(".desktop") for path in files.splitlines())
        deadline = time.monotonic() + 30
        while not self.executable.is_file():
            assert time.monotonic() < deadline, (
                self.executable,
                list(self.executable.parent.glob("*")),
            )
            time.sleep(0.05)

    def remove(self):
        if sys.platform == "darwin":
            shutil.rmtree(self.installed)
        elif sys.platform == "win32":
            uninstallers = list(self.installed.glob("uninstall*.exe"))
            assert len(uninstallers) == 1, uninstallers
            subprocess.run([str(uninstallers[0]), "/S"], check=True, timeout=90)
        else:
            subprocess.run(
                ["sudo", "dpkg", "--remove", self.name], check=True, timeout=60
            )
        deadline = time.monotonic() + 30
        while self.executable.exists():
            assert time.monotonic() < deadline, (
                f"Installed executable survived removal: {self.executable}"
            )
            time.sleep(0.05)


def run(cli, bundles, home, output, configuration):
    home = Path(home)
    fixtures = Fixtures(cli, home, configuration)
    database = fixtures.create("primary")
    preferences = home / ".anyq/panel.toml"
    preferences.write_text(
        f'database_override = {json.dumps(str(database))}\ntheme = "dark"\nlanguage = "en"\n',
        encoding="utf-8",
    )
    shared = home / ".anyq/config.toml"
    saved = {
        path: hashlib.sha256(path.read_bytes()).hexdigest()
        for path in [preferences, shared]
    }
    before = fingerprint(database)
    package = Package(bundles, home)
    native = NativeSession(home, fixtures.environment)
    client = None
    owner = None
    report = []
    try:
        for cycle in range(2):
            package.install()
            log = (home / f"installed-panel-{cycle}.log").open("ab")
            try:
                client = subprocess.Popen(
                    [str(package.executable)],
                    env=fixtures.environment,
                    stdout=log,
                    stderr=log,
                )
                windows = native.wait("windows", pid=client.pid)
                assert any(
                    window["width"] >= 360 and window["height"] >= 500
                    for window in windows
                ), windows
                deadline = time.monotonic() + 20
                while "running" not in (
                    current := fixtures.call(["daemon", "status"], database)
                ):
                    assert client.poll() is None and time.monotonic() < deadline, (
                        current
                    )
                    time.sleep(0.05)
                if owner is None:
                    owner = current
                assert current == owner
                if sys.platform in ("darwin", "win32"):
                    native.click_tray(client.pid, right=True)
                    native.key("up")
                    native.key("enter")
                else:
                    native.close_window(client.pid)
                assert client.wait(timeout=20) == 0
                client = None
            finally:
                log.close()
            package.remove()
            assert fixtures.call(["daemon", "status"], database) == owner
            assert fingerprint(database) == before
            assert {
                path: hashlib.sha256(path.read_bytes()).hexdigest() for path in saved
            } == saved
            report.append(
                {
                    "cycle": cycle + 1,
                    "installed_window": windows,
                    "owner": owner,
                    "removal": "passed",
                    "queue_and_preferences": "unchanged",
                }
            )
        destination = Path(output) / platform_name() / "package-integration.json"
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(
            json.dumps(
                {"result": "passed", "package": package.source.name, "cycles": report},
                indent=2,
            )
            + "\n",
            encoding="utf-8",
        )
    finally:
        if client and client.poll() is None:
            client.terminate()
            client.wait(timeout=10)
        fixtures.stop()
        if package.executable.exists():
            package.remove()
        native.stop()
        destination = Path(output) / platform_name()
        destination.mkdir(parents=True, exist_ok=True)
        for log in home.glob("*.log"):
            shutil.copyfile(log, destination / log.name)
