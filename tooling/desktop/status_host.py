"""Start and stop a real Xfce StatusNotifier host inside the test's private session."""

import subprocess
from pathlib import Path


class StatusHost:
    def __init__(self, home, environment):
        self.environment = environment
        root = Path(environment["XDG_CONFIG_HOME"]) / "xfce4/xfconf/xfce-perchannel-xml"
        root.mkdir(parents=True, exist_ok=True)
        (root / "xfce4-panel.xml").write_text(
            """<?xml version="1.0" encoding="UTF-8"?>
<channel name="xfce4-panel" version="1.0">
  <property name="configver" type="int" value="2"/>
  <property name="panels" type="array">
    <value type="int" value="1"/>
    <property name="panel-1" type="empty">
      <property name="position" type="string" value="p=6;x=640;y=0"/>
      <property name="position-locked" type="bool" value="true"/>
      <property name="size" type="uint" value="40"/>
      <property name="length" type="uint" value="10"/>
      <property name="plugin-ids" type="array"><value type="int" value="1"/></property>
    </property>
  </property>
  <property name="plugins" type="empty">
    <property name="plugin-1" type="string" value="systray"/>
  </property>
</channel>
""",
            encoding="utf-8",
        )
        self.log = (Path(home) / "status-host.log").open("ab")
        self.process = None

    def start(self):
        self.process = subprocess.Popen(
            ["xfce4-panel", "--disable-wm-check"],
            env=self.environment,
            stdout=self.log,
            stderr=self.log,
        )

    def stop(self):
        if self.process and self.process.poll() is None:
            self.process.terminate()
            self.process.wait(timeout=10)
        self.process = None

    def close(self):
        self.stop()
        self.log.close()
