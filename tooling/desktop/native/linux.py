"""Inspect a real GTK session and operate its native windows and status host."""

import faulthandler
import json
import os
import subprocess
import sys

import gi
from accessibility import Accessibility
from gi.repository import Gio, GLib


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


def tray():
    with Accessibility() as accessibility:
        for item in accessibility.descendants(accessibility.applications()):
            if (
                "anyq" in accessibility.name(item).lower()
                and accessibility.call(item, "GetRole") != Atspi.Role.APPLICATION
            ):
                if "org.a11y.atspi.Component" in accessibility.call(
                    item, "GetInterfaces"
                ):
                    bounds = accessibility.bounds(item)
                    if 0 < bounds["width"] < 100 and 0 < bounds["height"] < 100:
                        return bounds
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
    with Accessibility() as accessibility:
        for item in accessibility.descendants(accessibility.applications(pid)):
            if accessibility.call(item, "GetRole") not in (
                Atspi.Role.MENU_ITEM,
                Atspi.Role.CHECK_MENU_ITEM,
            ):
                continue
            showing = accessibility.showing(item, Atspi.StateType.SHOWING)
            name = accessibility.label(item).replace("_", "")
            candidates.append({"name": name, "showing": showing})
            if showing and name == label:
                return accessibility.bounds(item)
        accessibility.diagnose()
    print(f"Native menu items: {candidates}", file=sys.stderr, flush=True)
    return None


def perform(command):
    action = command["action"]
    if action in ("extension", "extension-state"):
        method = (
            "GetExtensionInfo"
            if action == "extension-state"
            else "EnableExtension"
            if command["enabled"]
            else "DisableExtension"
        )
        return (
            Gio.bus_get_sync(Gio.BusType.SESSION, None)
            .call_sync(
                "org.gnome.Shell",
                "/org/gnome/Shell",
                "org.gnome.Shell.Extensions",
                method,
                GLib.Variant("(s)", (command["uuid"],)),
                None,
                Gio.DBusCallFlags.NONE,
                1000,
                None,
            )
            .unpack()[0]
        )
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
