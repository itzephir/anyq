"""Drive operating-system input independently from the application's test transport."""

import json
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

    def _read(self):
        for line in self.process.stdout:
            self.responses.put(line)

    def call(self, action, **values):
        self.process.stdin.write(json.dumps({"action": action} | values) + "\n")
        self.process.stdin.flush()
        try:
            reply = json.loads(self.responses.get(timeout=20))
        except queue.Empty as error:
            raise AssertionError(f"Native operation timed out: {action}") from error
        assert reply["ok"], reply
        return reply["value"]

    def wait(self, action, predicate=bool, **values):
        deadline = time.monotonic() + 20
        while True:
            result = self.call(action, **values)
            if predicate(result):
                return result
            assert time.monotonic() < deadline, (action, result)
            time.sleep(0.05)

    def click_tray(self, pid, right=False):
        rect = self.wait("tray", pid=pid)
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
                "close": (13, 1 << 20),
                "go": (5, (1 << 20) | (1 << 17)),
            }[name]
            return self.call("key", code=code, flags=flags)
        if sys.platform == "win32":
            code = {
                "escape": 0x1B,
                "enter": 0x0D,
                "down": 0x28,
                "up": 0x26,
                "end": 0x23,
            }[name]
        else:
            code = {
                "escape": "Escape",
                "enter": "Return",
                "down": "Down",
                "up": "Up",
                "end": "End",
                "go": "ctrl+l",
            }[name]
        return self.call("key", code=code)

    def close_window(self, pid):
        if sys.platform == "darwin":
            self.key("close")
        else:
            self.call("close", pid=pid)

    def stop(self):
        self.process.stdin.close()
        self.process.wait(timeout=10)
        assert self.process.returncode == 0
        self.log.close()
