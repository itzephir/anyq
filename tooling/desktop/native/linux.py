"""Inspect a real GTK session and operate its native windows and status host."""

import json
import subprocess
import sys
from pathlib import Path

import gi


def desktop_types():
    gi.require_version("Gdk", "3.0")
    gi.require_version("Atspi", "2.0")
    from gi.repository import Atspi, Gdk

    return Atspi, Gdk


Atspi, Gdk = desktop_types()


def run(*arguments):
    return subprocess.check_output(arguments, text=True, timeout=10).strip()


def windows(pid):
    found = subprocess.run(
        [
            "xdotool",
            "search",
            "--all",
            "--onlyvisible",
            "--pid",
            str(pid),
            "--name",
            "^anyq$",
        ],
        capture_output=True,
        text=True,
        timeout=10,
    )
    assert found.returncode in (0, 1), found.stderr
    result = []
    for handle in found.stdout.splitlines():
        values = dict(
            line.split("=", 1)
            for line in run(
                "xdotool", "getwindowgeometry", "--shell", handle
            ).splitlines()
        )
        result.append(
            {key: int(values[key.upper()]) for key in ["x", "y", "width", "height"]}
            | {"handle": handle, "layer": 0}
        )
    return result


def tray():
    root = Atspi.get_desktop(0)
    pending = []
    for index in range(root.get_child_count()):
        app = root.get_child_at_index(index)
        if app is None:
            continue
        name = Path(f"/proc/{app.get_process_id()}/comm")
        if name.exists() and name.read_text().strip() in (
            "xfce4-panel",
            "wrapper-2.0",
            "gnome-shell",
            "plasmashell",
        ):
            pending.append(app)
    while pending:
        item = pending.pop()
        name = (item.get_name() or "").lower()
        if "anyq" in name and item.get_role() != Atspi.Role.APPLICATION:
            component = item.get_component_iface()
            if component:
                bounds = component.get_extents(Atspi.CoordType.SCREEN)
                if 0 < bounds.width < 100 and 0 < bounds.height < 100:
                    return {
                        key: getattr(bounds, key)
                        for key in ["x", "y", "width", "height"]
                    }
        pending.extend(
            child
            for index in range(item.get_child_count())
            if (child := item.get_child_at_index(index)) is not None
        )
    panels = subprocess.run(
        ["xdotool", "search", "--onlyvisible", "--class", "xfce4-panel"],
        capture_output=True,
        text=True,
        timeout=10,
    )
    for handle in panels.stdout.splitlines():
        values = dict(
            line.split("=", 1)
            for line in run(
                "xdotool", "getwindowgeometry", "--shell", handle
            ).splitlines()
        )
        if 24 <= int(values["HEIGHT"]) <= 64:
            return {
                key: int(values[key.upper()]) for key in ["x", "y", "width", "height"]
            }
    return None


def perform(command):
    action = command["action"]
    if action == "monitors":
        display = Gdk.Display.get_default()
        result = []
        for index in range(display.get_n_monitors()):
            monitor = display.get_monitor(index)
            bounds = monitor.get_workarea()
            result.append(
                {key: getattr(bounds, key) for key in ["x", "y", "width", "height"]}
                | {"scale": monitor.get_scale_factor()}
            )
        return result
    if action == "windows":
        return windows(command["pid"])
    if action == "tray":
        return tray()
    if action == "click":
        run(
            "xdotool",
            "mousemove",
            "--sync",
            str(round(command["x"])),
            str(round(command["y"])),
        )
        run("xdotool", "click", "3" if command.get("right") else "1")
    elif action == "key":
        run("xdotool", "key", "--clearmodifiers", command["code"])
    elif action == "type":
        run("xdotool", "type", "--clearmodifiers", "--", command["text"])
    elif action == "close":
        target = max(windows(command["pid"]), key=lambda item: item["height"])
        run("xdotool", "windowactivate", "--sync", target["handle"])
        run("xdotool", "key", "--clearmodifiers", "alt+F4")
    else:
        raise AssertionError(f"Unknown native action: {action}")
    return True


for line in sys.stdin:
    try:
        reply = {"ok": True, "value": perform(json.loads(line))}
    except Exception as error:
        reply = {"ok": False, "error": str(error)}
    print(json.dumps(reply), flush=True)
