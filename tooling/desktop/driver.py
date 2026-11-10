"""Drive real native webviews through the private test-only transport."""

import json
import subprocess
import time
from pathlib import Path

from fixtures import environment


class Driver:
    def __init__(self, executable, home, database, clock_ms):
        self.executable = Path(executable).resolve()
        self.home = Path(home)
        self.database = Path(database)
        self.directory = self.home / "driver"
        self.directory.mkdir(parents=True, exist_ok=True)
        self.environment = environment(self.home, self.database) | {
            "ANYQ_PANEL_TEST_DIR": str(self.directory),
            "ANYQ_PANEL_TEST_CLOCK": str(clock_ms),
        }
        self.process = None
        self.sequence = 0
        self.log = None

    def start(self, status="live", viewport=(1000, 700)):
        (self.directory / "command.json").unlink(missing_ok=True)
        self.log = (self.home / "panel.log").open("ab")
        self.process = subprocess.Popen(
            [str(self.executable)],
            env=self.environment,
            stdout=self.log,
            stderr=self.log,
        )
        self.wait("return Boolean(window.__TAURI__?.core);")
        self.resize(*viewport)
        self.wait(
            f"return (await window.__TAURI__.core.invoke('get_panel_state')).status === {json.dumps(status)};"
        )
        self.wait(f"return Boolean(document.querySelector('.status.{status}'));")

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

    def resize(self, width, height, window="main"):
        requested_width, requested_height = width, height
        for _ in range(3):
            self.call(
                window=window,
                native="resize",
                width=requested_width,
                height=requested_height,
            )
            result = self.wait(
                "return innerWidth > 0 && innerHeight > 0 ? [innerWidth,innerHeight] : false;",
                window,
            )["payload"]["value"]
            if result == [width, height]:
                return
            requested_width += width - result[0]
            requested_height += height - result[1]
        raise AssertionError(
            f"Native viewport did not reach {[width, height]}: {result}"
        )

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
