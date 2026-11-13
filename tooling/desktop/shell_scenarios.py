"""Exercise actual status icons, native menus, display topology and owner lifetime."""

import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

import display_scenarios
from driver import Driver
from fixtures import Fixtures, queue_fingerprint, same_source_path
from gnome_status_host import GnomeStatusHost
from native_session import NativeSession
from PIL import ImageGrab
from screenshots import platform_name
from status_host import StatusHost


def run(cli, panel, home, output, configuration, profile):
    home = Path(home)
    fixtures = Fixtures(cli, home, configuration)
    native_environment = (
        {"GTK_MODULES": "atk-bridge", "NO_AT_BRIDGE": "0"}
        if sys.platform.startswith("linux")
        else {}
    )
    fixtures.environment.update(native_environment)
    database = fixtures.create("primary")
    alternate = fixtures.create("alternate")
    (home / ".anyq/panel.toml").write_text(
        'theme = "dark"\nlanguage = "en"\n', encoding="utf-8"
    )
    driver = Driver(panel, home, database, configuration["clock_ms"])
    driver.environment.update(native_environment)
    native = NativeSession(home, fixtures.environment)
    host = StatusHost(home, fixtures.environment) if profile["host"] == "xfce" else None
    if profile["host"] == "gnome":
        host = GnomeStatusHost(home, fixtures.environment)
    report = []

    def menu_open():
        native.select_menu(
            driver.process.pid, "Open window", right=profile["host"] != "xfce"
        )
        driver.wait_shell(
            lambda state: state["windows"]["main"]["visible"]
            and state["windows"]["main"]["focused"]
        )

    def exit_without_host():
        native.close_window(driver.process.pid)
        assert driver.process.wait(timeout=15) == 0
        driver.stop()
        assert fixtures.call(["daemon", "status"], database) == owner
        assert queue_fingerprint(database) == before
        report.append({"scenario": "host-loss-last-window-exits", "result": "passed"})

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
        assert (
            driver.state()["snapshot"]
            == configuration["fixtures"]["primary"]["expected_snapshot"]
        )
        before = queue_fingerprint(database)
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

            report.extend(display_scenarios.run(driver, native, monitors, profile))

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
                native.wait(
                    "windows",
                    predicate=lambda items: len(items) > prior + 1,
                    pid=driver.process.pid,
                )
                native.call("type", text=str(alternate))
                native.key("enter")
                native.wait(
                    "windows",
                    predicate=lambda items: len(items) <= prior + 1,
                    pid=driver.process.pid,
                )
                native.key("enter")
                driver.wait(
                    f"return document.querySelector('#database-path')?.value.endsWith({json.dumps('/' + alternate.name)});"
                )
                chosen = driver.call(
                    "return document.querySelector('#database-path').value;"
                )["payload"]["value"]
                assert same_source_path(chosen, alternate), (chosen, str(alternate))
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
                    exit_without_host()
                    host.start()
                    driver.start(viewport=configuration["visual_defaults"]["viewport"])
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
                    exit_without_host()
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
                    driver.start(viewport=configuration["visual_defaults"]["viewport"])
                else:
                    subprocess.run(
                        ["killall", "SystemUIServer"], check=True, timeout=15
                    )
                native.wait("tray", pid=driver.process.pid)
                driver.wait_shell(lambda state: state["tray"]["usable"])
                menu_open()
                report.append({"scenario": "real-host-restart", "result": "passed"})

            native.select_menu(
                driver.process.pid, "Quit anyq", right=profile["host"] != "xfce"
            )
            assert driver.process.wait(timeout=15) == 0
            report.append({"scenario": "native-menu-quit", "result": "passed"})

        assert fixtures.call(["daemon", "status"], database) == owner
        assert queue_fingerprint(database) == before
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
        destination = Path(output) / platform_name()
        destination.mkdir(parents=True, exist_ok=True)
        if sys.exc_info()[0] is not None:
            try:
                ImageGrab.grab(all_screens=True).save(
                    destination / "native-failure.png"
                )
            except OSError as error:
                print(f"Native failure capture unavailable: {error}", file=sys.stderr)
        try:
            if driver.process and driver.process.poll() is None:
                driver.process.terminate()
                driver.process.wait(timeout=10)
            if driver.log:
                driver.log.close()
            if host:
                host.close()
            native.stop()
            fixtures.stop()
        finally:
            for log in home.glob("*.log"):
                shutil.copyfile(log, destination / log.name)
