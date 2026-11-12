"""Operate the real GNOME AppIndicator extension without restarting its compositor."""

import json
import subprocess
from pathlib import Path


class GnomeStatusHost:
    def __init__(self, home, environment):
        self.environment = environment
        extensions = list(
            Path("/usr/share/gnome-shell/extensions").glob("*appindicator*")
        )
        assert len(extensions) == 1, extensions
        self.uuid = json.loads((extensions[0] / "metadata.json").read_text())["uuid"]
        self.log = (Path(home) / "gnome-extension.log").open("ab")

    def start(self):
        subprocess.run(
            ["gnome-extensions", "enable", self.uuid],
            env=self.environment,
            check=True,
            timeout=15,
            stdout=self.log,
            stderr=self.log,
        )

    def stop(self):
        subprocess.run(
            ["gnome-extensions", "disable", self.uuid],
            env=self.environment,
            check=True,
            timeout=15,
            stdout=self.log,
            stderr=self.log,
        )

    def close(self):
        self.stop()
        self.log.close()
