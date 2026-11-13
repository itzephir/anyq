"""Inspect a real GTK session and operate its native windows and status host."""

import faulthandler
import json
import os
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


def applications(pid=None):
    root = Atspi.get_desktop(0)
    if root is None:
        return []
    root.set_cache_mask(Atspi.Cache.NONE)
    result = []
    for index in range(root.get_child_count()):
        app = root.get_child_at_index(index)
        if app is None:
            continue
        process = app.get_process_id()
        name = Path(f"/proc/{process}/comm")
        if (
            process == pid
            or name.exists()
            and name.read_text().strip()
            in (
                "xfce4-panel",
                "wrapper-2.0",
                "gnome-shell",
                "plasmashell",
            )
        ):
            app.set_cache_mask(Atspi.Cache.NONE)
            result.append(app)
    return result


def descendants(roots):
    pending = list(roots)
    visited = set()
    while pending:
        item = pending.pop()
        if item in visited:
            continue
        visited.add(item)
        yield item
        pending.extend(
            child
            for index in range(item.get_child_count())
            if (child := item.get_child_at_index(index)) is not None
        )


def tray():
    for item in descendants(applications()):
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


def menu_item(pid, label):
    candidates = []
    for item in descendants(applications(pid)):
        if item.get_role() in (Atspi.Role.MENU_ITEM, Atspi.Role.CHECK_MENU_ITEM):
            candidates.append(
                {
                    "name": item.get_name(),
                    "showing": item.get_state_set().contains(Atspi.StateType.SHOWING),
                }
            )
        if (
            item.get_role() in (Atspi.Role.MENU_ITEM, Atspi.Role.CHECK_MENU_ITEM)
            and accessible_label(item).replace("_", "") == label
            and item.get_state_set().contains(Atspi.StateType.SHOWING)
        ):
            bounds = item.get_component_iface().get_extents(Atspi.CoordType.SCREEN)
            return {key: getattr(bounds, key) for key in ["x", "y", "width", "height"]}
    print(f"Native menu items: {candidates}", file=sys.stderr, flush=True)
    return None


def accessible_label(item):
    if name := item.get_name():
        return name
    for child in descendants([item]):
        if name := child.get_name():
            return name
        if text := child.get_text_iface():
            if value := text.get_text(0, -1):
                return value
    return ""


def perform(command):
    action = command["action"]
    if action == "monitors":
        display = Gdk.Display.get_default()
        result = []
        for index in range(display.get_n_monitors()):
            monitor = display.get_monitor(index)
            bounds = monitor.get_workarea()
            frame = monitor.get_geometry()
            result.append(
                {key: getattr(bounds, key) for key in ["x", "y", "width", "height"]}
                | {
                    "scale": monitor.get_scale_factor(),
                    "bounds": {
                        key: getattr(frame, key)
                        for key in ["x", "y", "width", "height"]
                    },
                }
            )
        return result
    if action == "windows":
        return windows(command["pid"])
    if action == "tray":
        return tray()
    if action == "menu-open":
        return bool(menu_item(command["pid"], "Open window"))
    if action == "menu-item":
        return menu_item(command["pid"], command["label"])
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
        if os.environ.get("GDK_BACKEND") != "wayland":
            target = max(windows(command["pid"]), key=lambda item: item["height"])
            run("xdotool", "windowactivate", "--sync", target["handle"])
        run("xdotool", "key", "--clearmodifiers", "alt+F4")
    else:
        raise AssertionError(f"Unknown native action: {action}")
    return True


for line in sys.stdin:
    faulthandler.dump_traceback_later(15)
    try:
        reply = {"ok": True, "value": perform(json.loads(line))}
    except Exception as error:
        reply = {"ok": False, "error": str(error)}
    finally:
        faulthandler.cancel_dump_traceback_later()
    print(json.dumps(reply), flush=True)
