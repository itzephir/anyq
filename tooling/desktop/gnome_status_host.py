"""Operate the real GNOME AppIndicator extension without restarting its compositor."""

import json
from pathlib import Path


class GnomeStatusHost:
    def __init__(self, native):
        self.native = native
        extensions = list(
            Path("/usr/share/gnome-shell/extensions").glob("*appindicator*")
        )
        assert len(extensions) == 1, extensions
        self.uuid = json.loads((extensions[0] / "metadata.json").read_text())["uuid"]

    def start(self):
        assert self.native.call("extension", uuid=self.uuid, enabled=True)
        self.native.wait(
            "extension-state", uuid=self.uuid, predicate=lambda info: info["state"] == 1
        )

    def stop(self):
        assert self.native.call("extension", uuid=self.uuid, enabled=False)
        self.native.wait(
            "extension-state", uuid=self.uuid, predicate=lambda info: info["state"] == 2
        )

    def close(self):
        self.stop()
