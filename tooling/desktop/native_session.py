"""Drive operating-system input independently from the application's test transport."""

import json
import os
import queue
import subprocess
import sys
import threading
import time
from pathlib import Path


class NativeSession:
    def __init__(self, directory, environment=None):
        source = Path(__file__).parent / "native"
        self.log = (Path(directory) / "native.log").open("ab")
        if sys.platform == "darwin":
            executable = Path(directory) / "native-macos"
            subprocess.run(
                [
                    "clang",
                    "-fobjc-arc",
                    "-framework",
                    "AppKit",
                    "-framework",
                    "ApplicationServices",
                    str(source / "macos.m"),
                    "-o",
                    str(executable),
                ],
                check=True,
                timeout=60,
                stdout=self.log,
                stderr=self.log,
            )
            command = [str(executable)]
        elif sys.platform == "win32":
            command = [sys.executable, str(source / "windows.py")]
        else:
            command = ["/usr/bin/python3", str(source / "linux.py")]
        self.process = subprocess.Popen(
            command,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=self.log,
            text=True,
            encoding="utf-8",
            env=environment,
        )
        self.responses = queue.Queue()
        threading.Thread(target=self._read, daemon=True).start()
        if sys.platform == "darwin":
            self.log.write(
                (
                    json.dumps({"native_capabilities": self.call("capabilities")})
                    + "\n"
                ).encode()
            )
            self.log.flush()

    def _read(self):
        for line in self.process.stdout:
            self.responses.put(line)
        self.responses.put(
            json.dumps(
                {"ok": False, "error": "Native helper exited; inspect native.log"}
            )
        )

    def call(self, action, **values):
        self.process.stdin.write(json.dumps({"action": action} | values) + "\n")
        self.process.stdin.flush()
        try:
            reply = json.loads(self.responses.get(timeout=20))
        except queue.Empty as error:
            raise AssertionError(f"Native operation timed out: {action}") from error
        assert reply["ok"], reply
        self.log.write(
            (
                json.dumps(
                    {"action": action, "arguments": values, "result": reply["value"]}
                )
                + "\n"
            ).encode()
        )
        self.log.flush()
        return reply["value"]

    def wait(self, action, predicate=bool, **values):
        deadline = time.monotonic() + 20
        while True:
            result = self.call(action, **values)
            if predicate(result):
                return result
            assert time.monotonic() < deadline, (action, result)
            time.sleep(0.05)

    def click_tray(self, pid, right=False, monitor=None):
        def matches(rect):
            return rect and (
                monitor is None
                or all(
                    monitor["bounds"][axis]
                    <= rect[axis] + rect[size] / 2
                    < monitor["bounds"][axis] + monitor["bounds"][size]
                    for axis, size in (("x", "width"), ("y", "height"))
                )
            )

        rect = self.wait(
            "tray",
            predicate=matches,
            pid=pid,
            **({"monitor": monitor["bounds"]} if monitor else {}),
        )
        self.call(
            "click",
            x=rect["x"] + rect["width"] / 2,
            y=rect["y"] + rect["height"] / 2,
            right=right,
        )
        return rect

    def key(self, name):
        if sys.platform == "darwin":
            code, flags = {
                "escape": (53, 0),
                "enter": (36, 0),
                "down": (125, 0),
                "up": (126, 0),
                "end": (119, 0),
                "home": (115, 0),
                "close": (13, 1 << 20),
                "go": (5, (1 << 20) | (1 << 17)),
                "select-all": (0, 1 << 20),
                "copy": (8, 1 << 20),
            }[name]
            return self.call("key", code=code, flags=flags)
        if sys.platform == "win32":
            code = {
                "escape": 0x1B,
                "enter": 0x0D,
                "down": 0x28,
                "up": 0x26,
                "end": 0x23,
                "home": 0x24,
                "select-all": 0x41,
                "copy": 0x43,
            }[name]
            return self.call(
                "key",
                code=code,
                modifiers=[0x11] if name in ("select-all", "copy") else [],
            )
        else:
            code = {
                "escape": "Escape",
                "enter": "Return",
                "down": "Down",
                "up": "Up",
                "end": "End",
                "home": "Home",
                "go": "ctrl+l",
                "monitor-right": "super+shift+Right",
                "monitor-left": "super+shift+Left",
                "move-window": "alt+F7",
                "right": "Right",
                "select-all": "ctrl+a",
                "copy": "ctrl+c",
            }[name]
        return self.call("key", code=code)

    def document_text(self, pid):
        assert os.environ.get("GITHUB_ACTIONS") == "true", (
            "Clipboard fixtures require a disposable CI session"
        )
        self.wait("focus", pid=pid)
        windows = self.call("windows", pid=pid)
        window = max(windows, key=lambda item: item["width"] * item["height"])
        self.call(
            "click",
            x=window["x"] + window["width"] - 50,
            y=window["y"] + window["height"] - 80,
        )
        self.wait("clipboard-clear")
        self.key("select-all")
        self.key("copy")
        deadline = time.monotonic() + 1
        while not (text := self.call("clipboard-text")):
            if time.monotonic() >= deadline:
                break
            time.sleep(0.02)
        self.wait("clipboard-clear")
        return text

    def select_menu(self, pid, label, *, right=True):
        self.click_tray(pid, right=right)
        self.wait("menu-open", pid=pid)
        if sys.platform == "darwin" or (
            sys.platform.startswith("linux") and os.environ.get("GDK_BACKEND") == "x11"
        ):
            self.key("end" if label == "Quit anyq" else "home")
            self.key("enter")
        else:
            rect = self.wait("menu-item", pid=pid, label=label)
            self.call(
                "click",
                x=rect["x"] + rect["width"] / 2,
                y=rect["y"] + rect["height"] / 2,
            )
        self.wait("menu-open", pid=pid, predicate=lambda opened: not opened)

    def close_window(self, pid):
        if sys.platform == "darwin":
            self.key("close")
        else:
            self.call("close", pid=pid)

    def stop(self):
        failing = sys.exc_info()[0] is not None
        try:
            self.process.stdin.close()
            try:
                self.process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait(timeout=5)
                if not failing:
                    raise
            if not failing:
                assert self.process.returncode == 0
        finally:
            self.log.close()
