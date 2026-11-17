"""Verify immediate native system control through the actual desktop application."""

import json
import subprocess
import sys
from pathlib import Path

from autolaunch_registration import corrupt, entry
from driver import Driver
from fixtures import Fixtures, queue_fingerprint


def run(cli, panel, home, output, configuration):
    home = Path(home)
    fixtures = Fixtures(cli, home, configuration)
    database = fixtures.create("primary")
    driver = Driver(panel, home, database, configuration["clock_ms"])
    preferences = home / ".anyq/panel.toml"
    preferences.write_text('theme = "dark"\nlanguage = "en"\n', encoding="utf-8")
    original_preferences = preferences.read_bytes()
    config = home / ".anyq/config.toml"
    original_config = config.read_bytes()
    fingerprint = queue_fingerprint(database)
    try:
        if sys.platform == "linux":
            driver.environment["XDG_CONFIG_HOME"] = ""
        driver.start()
        if sys.platform == "linux":
            driver.click("#settings")
            driver.wait(
                "return document.querySelector('#autolaunch-status')?.dataset.enabled==='false';"
            )
            driver.click("#autolaunch")
            driver.wait(
                "return document.querySelector('#autolaunch-status')?.dataset.enabled==='true' && document.querySelector('#autolaunch-status')?.dataset.busy==='false';"
            )
            assert entry(home, home / ".config") is not None
            driver.click("#autolaunch")
            driver.wait(
                "return document.querySelector('#autolaunch-status')?.dataset.enabled==='false' && document.querySelector('#autolaunch-status')?.dataset.busy==='false';"
            )
            driver.stop()
            driver.environment["XDG_CONFIG_HOME"] = str(home / "xdg-config")
            driver.start()
        owner = fixtures.call(["daemon", "status"], database)
        driver.click("#settings")
        driver.wait(
            "return document.querySelector('#autolaunch-status')?.dataset.enabled==='false';"
        )
        driver.click("#autolaunch")
        driver.wait(
            "return document.querySelector('#autolaunch-status')?.dataset.enabled==='true' && document.querySelector('#autolaunch-status')?.dataset.busy==='false';"
        )
        assert fixtures.call(["autolaunch"], database) == "on"
        registration = entry(home)
        assert registration is not None
        driver.click("#cancel")
        assert fixtures.call(["autolaunch"], database) == "on"
        assert preferences.read_bytes() == original_preferences
        assert config.read_bytes() == original_config
        driver.click("#settings")
        fixtures.call(["autolaunch", "off"], database)
        driver.wait(
            "return document.querySelector('#autolaunch-status')?.dataset.enabled==='false';"
        )
        fixtures.call(["autolaunch", "on"], database)
        driver.wait(
            "return document.querySelector('#autolaunch-status')?.dataset.enabled==='true';"
        )
        corrupt(home)
        driver.wait("return Boolean(document.querySelector('#autolaunch-retry'));")
        driver.click("#autolaunch-disable")
        driver.wait(
            "return document.querySelector('#autolaunch-status')?.dataset.enabled==='false' && !document.querySelector('#autolaunch-retry');"
        )
        assert fixtures.call(["autolaunch"], database) == "off"
        fixtures.call(["autolaunch", "on"], database)
        driver.wait(
            "return document.querySelector('#autolaunch-status')?.dataset.enabled==='true';"
        )
        # The GUI remains able to change native state with malformed shared config.
        config.write_text("broken = [", encoding="utf-8")
        driver.click("#autolaunch")
        driver.wait(
            "return document.querySelector('#autolaunch-status')?.dataset.enabled==='false';"
        )
        assert fixtures.call(["autolaunch"], database) == "off"
        assert config.read_text(encoding="utf-8") == "broken = ["
        config.write_bytes(original_config)
        driver.click("#autolaunch")
        driver.wait(
            "return document.querySelector('#autolaunch-status')?.dataset.enabled==='true';"
        )
        registration = entry(home)
        driver.stop()
        assert fixtures.call(["daemon", "status"], database) == owner
        fixtures.call(["daemon", "stop"], database)
        # Execute the real registration written by the GUI. It must remain headless.
        subprocess.run(registration[1], env=driver.environment, check=True, timeout=30)
        fixtures.wait_owner(database)
        assert "running" in fixtures.call(["daemon", "status"], database)
        assert queue_fingerprint(database) == fingerprint
        assert preferences.read_bytes() == original_preferences
        assert config.read_bytes() == original_config
        destination = Path(output) / "autolaunch-integration.json"
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(
            json.dumps(
                {
                    "result": "passed",
                    "scenarios": [
                        "immediate-system-toggle",
                        "cancel-does-not-revert-system-action",
                        "cli-to-gui-live-status",
                        "malformed-config-independent",
                        "gui-retained-headless-login-launcher",
                        "queue-and-preferences-preserved",
                        *(
                            ["empty-xdg-gui-startup-and-native-control"]
                            if sys.platform == "linux"
                            else []
                        ),
                    ],
                },
                indent=2,
            )
            + "\n",
            encoding="utf-8",
        )
    finally:
        if driver.process and driver.process.poll() is None:
            driver.stop()
        if driver.log:
            driver.log.close()
        config.write_bytes(original_config)
        fixtures.stop()
