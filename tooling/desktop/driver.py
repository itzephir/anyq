"""Drive real native webviews through the private test-only transport."""

import json
import os
import subprocess
import time
from pathlib import Path


class Driver:
    def __init__(self, executable, home, database):
        self.executable = Path(executable).resolve()
        self.home = Path(home)
        self.database = Path(database)
        self.directory = self.home / "driver"
        self.directory.mkdir(parents=True, exist_ok=True)
        self.environment = os.environ | {
            "HOME": str(self.home),
            "USERPROFILE": str(self.home),
            "ANYQ_DB": str(self.database),
            "ANYQ_PANEL_TEST_DIR": str(self.directory),
        }
        self.process = None
        self.sequence = 0
        self.log = None

    def start(self):
        (self.directory / "command.json").unlink(missing_ok=True)
        self.log = (self.home / "panel.log").open("ab")
        self.process = subprocess.Popen(
            [str(self.executable)],
            env=self.environment,
            stdout=self.log,
            stderr=self.log,
        )
        self.wait("return Boolean(window.__TAURI__?.core);")
        self.call(native="resize", width=1000, height=700)
        self.wait(
            "return (await window.__TAURI__.core.invoke('get_panel_state')).status === 'live';"
        )

    def call(self, script="return null;", window="main", **native):
        self.sequence += 1
        command = {"id": self.sequence, "window": window, "script": script} | native
        temporary = self.directory / "command.tmp"
        temporary.write_text(json.dumps(command), encoding="utf-8")
        temporary.replace(self.directory / "command.json")
        response = self.directory / f"response-{self.sequence}.json"
        deadline = time.monotonic() + 25
        while not response.exists():
            if self.process.poll() is not None:
                raise AssertionError(
                    f"Panel exited: {(self.home / 'panel.log').read_text(errors='replace')}"
                )
            if time.monotonic() >= deadline:
                raise AssertionError(f"Native UI command timed out: {command}")
            time.sleep(0.02)
        result = json.loads(response.read_text(encoding="utf-8"))
        if not result["payload"]["ok"]:
            raise AssertionError(result["payload"])
        return result

    def wait(self, script, window="main", timeout=20):
        deadline = time.monotonic() + timeout
        while True:
            result = self.call(script, window)
            if result["payload"]["value"]:
                return result
            if time.monotonic() >= deadline:
                raise AssertionError(f"Native UI condition timed out: {script}")
            time.sleep(0.05)

    def state(self):
        return self.call(
            "return await window.__TAURI__.core.invoke('get_panel_state');"
        )["payload"]["value"]

    def click(self, selector, window="main"):
        return self.call(
            f"document.querySelector({json.dumps(selector)}).click();", window
        )

    def fill(self, selector, value):
        return self.call(
            f"const element=document.querySelector({json.dumps(selector)});"
            f"element.value={json.dumps(value)};element.dispatchEvent(new Event('input',{{bubbles:true}}));"
        )

    def select(self, selector, value):
        return self.call(
            f"const element=document.querySelector({json.dumps(selector)});"
            f"element.value={json.dumps(value)};element.dispatchEvent(new Event('change',{{bubbles:true}}));"
        )

    def stop(self):
        if self.process and self.process.poll() is None:
            self.call(native="quit")
            self.process.wait(timeout=10)
        if self.log:
            self.log.close()
