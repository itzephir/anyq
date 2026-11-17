"""Inspect the application's real, isolated native login registration in tests."""

import os
import plistlib
import re
import sys
from pathlib import Path


def entry(home, config_directory=None):
    home = Path(home)
    if sys.platform == "darwin":
        path = home / "Library/LaunchAgents/com.itzephir.anyq.autolaunch.plist"
        if not path.exists():
            return None
        with path.open("rb") as source:
            value = plistlib.load(source)
        assert value["RunAtLoad"] and value["AbandonProcessGroup"]
        assert value["ProgramArguments"][1:] == ["daemon", "start"]
        return path, value["ProgramArguments"]
    if os.name == "nt":
        import winreg

        with winreg.OpenKey(
            winreg.HKEY_CURRENT_USER, r"Software\Microsoft\Windows\CurrentVersion\Run"
        ) as key:
            for index in range(winreg.QueryInfoKey(key)[1]):
                name, command, kind = winreg.EnumValue(key, index)
                if not name.startswith("anyq-") or kind != winreg.REG_SZ:
                    continue
                match = re.fullmatch(r'"(.+)" daemon start', command)
                if match:
                    launcher = Path(match[1])
                    try:
                        if len(launcher.parents) > 3 and launcher.parents[3].samefile(
                            home
                        ):
                            return name, command
                    except OSError:
                        continue
        return None
    directory = (
        Path(config_directory) if config_directory is not None else home / "xdg-config"
    )
    path = directory / "autostart/anyq.desktop"
    return (path, ["gio", "launch", str(path)]) if path.exists() else None


def corrupt(home):
    registration = entry(home)
    assert registration is not None
    if os.name == "nt":
        import winreg

        with winreg.OpenKey(
            winreg.HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\Run",
            0,
            winreg.KEY_SET_VALUE,
        ) as key:
            winreg.SetValueEx(
                key,
                registration[0],
                0,
                winreg.REG_SZ,
                '"missing-launcher.exe" daemon start',
            )
    else:
        registration[0].write_bytes(b"invalid native registration")
