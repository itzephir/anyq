"""Exercise actual status icons, native menus, display topology and owner lifetime."""

import json
import os
import shutil
import sqlite3
import subprocess
import sys
from contextlib import closing
from pathlib import Path

from driver import Driver
from fixtures import Fixtures
from native_session import NativeSession
from screenshots import platform_name
from status_host import StatusHost


def run(cli, panel, home, output, configuration, profile):
    home = Path(home)
    fixtures = Fixtures(cli, home, configuration)
    database = fixtures.create("primary")
    alternate = fixtures.create("alternate")
    (home / ".anyq/panel.toml").write_text(
        'theme = "dark"\nlanguage = "en"\n', encoding="utf-8"
    )
    driver = Driver(panel, home, database, configuration["clock_ms"])
    native = NativeSession(home, fixtures.environment)
    host = StatusHost(home, fixtures.environment) if profile["host"] == "xfce" else None
    report = []

    def fingerprint():
        with closing(sqlite3.connect(database)) as connection:
            return {
                table: connection.execute(
                    f"SELECT * FROM {table} ORDER BY 1"
                ).fetchall()
                for table in ("topics", "tasks", "task_topics")
            }

    def menu_open():
        native.click_tray(driver.process.pid, right=True)
        native.key("down")
        native.key("enter")
        driver.wait_shell(
            lambda state: state["windows"]["main"]["visible"]
            and state["windows"]["main"]["focused"]
        )

    try:
        if profile["displays"]:
            native.call("displays", displays=profile["displays"])
        if sys.platform == "win32":
            native.wait(
                "monitors",
                predicate=lambda items: len(items) >= profile["minimum_monitors"],
            )
            native.call("configure-displays")
        monitors = native.wait(
            "monitors",
            predicate=lambda items: len(items) >= profile["minimum_monitors"],
        )
        if sys.platform == "win32":
            monitors = native.wait(
                "monitors",
                predicate=lambda items: {1, 1.5}.issubset(
                    {item["scale"] for item in items}
                ),
            )
        if host:
            host.start()
        driver.start(viewport=configuration["visual_defaults"]["viewport"])
        before = fingerprint()
        owner = fixtures.call(["daemon", "status"], database)
        assert "running" in owner

        if profile["host"] == "none":
            driver.wait_shell(lambda state: not state["tray"]["usable"])
            native.close_window(driver.process.pid)
            assert driver.process.wait(timeout=10) == 0
            report.append({"scenario": "no-host-last-window-exits", "result": "passed"})
        else:
            driver.wait_shell(lambda state: state["tray"]["usable"])
            native.close_window(driver.process.pid)
            driver.wait_shell(lambda state: not state["windows"]["main"]["visible"])
            menu_open()
            report.append({"scenario": "native-menu-reopens-main", "result": "passed"})

            for monitor in monitors:
                target = "main" if sys.platform.startswith("linux") else "compact"
                scale = monitor["scale"] if sys.platform.startswith("linux") else 1
                driver.call(
                    window=target,
                    native="position",
                    x=(monitor["x"] + 30) * scale,
                    y=(monitor["y"] + 60) * scale,
                )

                def moved(state):
                    window = state["windows"][target]
                    divisor = window["scale"] if sys.platform == "darwin" else scale
                    return (
                        abs(window["position"]["x"] / divisor - monitor["x"] - 30) <= 4
                        and abs(window["position"]["y"] / divisor - monitor["y"] - 60)
                        <= 4
                    )

                driver.wait_shell(moved)
                if target == "compact":
                    native.click_tray(driver.process.pid)
                    state = driver.wait_shell(
                        lambda state: state["windows"]["compact"]["visible"]
                        and state["windows"]["compact"]["focused"]
                    )
                    rect = state["windows"]["compact"]
                    x, y = rect["position"]["x"], rect["position"]["y"]
                    width, height = rect["size"]["width"], rect["size"]["height"]
                    if sys.platform == "darwin":
                        x, y, width, height = (
                            value / rect["scale"] for value in (x, y, width, height)
                        )
                    assert any(
                        area["x"] <= x
                        and area["y"] <= y
                        and x + width <= area["x"] + area["width"]
                        and y + height <= area["y"] + area["height"]
                        for area in monitors
                    ), state
                    native.key("escape")
                    driver.wait_shell(
                        lambda state: not state["windows"]["compact"]["visible"]
                    )
                    native.click_tray(driver.process.pid)
                    driver.wait_shell(
                        lambda state: state["windows"]["compact"]["visible"]
                    )
                    main = driver.shell_state()["windows"]["main"]
                    divisor = main["scale"] if sys.platform == "darwin" else 1
                    native.call(
                        "click",
                        x=main["position"]["x"] / divisor + 50,
                        y=main["position"]["y"] / divisor + 100,
                    )
                    driver.wait_shell(
                        lambda state: not state["windows"]["compact"]["visible"]
                    )
                report.append(
                    {
                        "scenario": "native-display-roundtrip",
                        "monitor": monitor,
                        "result": "passed",
                    }
                )

            if profile["file_picker"]:
                driver.click("#settings")
                driver.click("#automatic")
                prior = len(native.call("windows", pid=driver.process.pid))
                driver.click("#choose")
                native.wait(
                    "windows",
                    predicate=lambda items: len(items) > prior,
                    pid=driver.process.pid,
                )
                native.key("go")
                native.call("type", text=str(alternate))
                native.key("enter")
                native.wait(
                    "windows",
                    predicate=lambda items: len(items) <= prior + 1,
                    pid=driver.process.pid,
                )
                native.key("enter")
                driver.wait(
                    f"return document.querySelector('#database-path')?.value === {json.dumps(str(alternate))};"
                )
                driver.click("#cancel")
                report.append(
                    {"scenario": "real-native-file-picker", "result": "passed"}
                )

            if profile["restart_host"]:
                assert os.environ.get("GITHUB_ACTIONS") == "true", (
                    "Restart desktop hosts only in disposable CI sessions"
                )
                native.close_window(driver.process.pid)
                driver.wait_shell(lambda state: not state["windows"]["main"]["visible"])
                if host:
                    host.stop()
                    driver.wait_shell(
                        lambda state: not state["tray"]["usable"]
                        and state["windows"]["main"]["visible"]
                    )
                    host.start()
                elif sys.platform == "win32":
                    subprocess.run(
                        [
                            "powershell",
                            "-NoProfile",
                            "-Command",
                            "Stop-Process -Name explorer -Force",
                        ],
                        check=True,
                        timeout=15,
                    )
                    driver.wait_shell(
                        lambda state: not state["tray"]["usable"]
                        and state["windows"]["main"]["visible"]
                    )
                    subprocess.run(
                        [
                            "powershell",
                            "-NoProfile",
                            "-Command",
                            "Start-Process explorer.exe",
                        ],
                        check=True,
                        timeout=15,
                    )
                else:
                    subprocess.run(
                        ["killall", "SystemUIServer"], check=True, timeout=15
                    )
                native.wait("tray", pid=driver.process.pid)
                driver.wait_shell(lambda state: state["tray"]["usable"])
                menu_open()
                report.append({"scenario": "real-host-restart", "result": "passed"})

            native.click_tray(driver.process.pid, right=True)
            native.key("up")
            native.key("enter")
            assert driver.process.wait(timeout=15) == 0
            report.append({"scenario": "native-menu-quit", "result": "passed"})

        assert fixtures.call(["daemon", "status"], database) == owner
        assert fingerprint() == before
        report.append(
            {
                "scenario": "same-owner-and-slot-ownership",
                "owner": owner,
                "result": "passed",
            }
        )
        destination = Path(output) / platform_name() / "shell-integration.json"
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(
            json.dumps(
                {
                    "result": "passed",
                    "profile": profile,
                    "monitors": monitors,
                    "scenarios": report,
                },
                indent=2,
            )
            + "\n",
            encoding="utf-8",
        )
    finally:
        if driver.process and driver.process.poll() is None:
            driver.process.terminate()
            driver.process.wait(timeout=10)
        if driver.log:
            driver.log.close()
        if host:
            host.close()
        native.stop()
        fixtures.stop()
        destination = Path(output) / platform_name()
        destination.mkdir(parents=True, exist_ok=True)
        for log in home.glob("*.log"):
            shutil.copyfile(log, destination / log.name)
