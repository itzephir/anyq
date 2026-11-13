"""Read the real AT-SPI D-Bus tree without libatspi's process-wide cache."""

import sys
import time
from pathlib import Path

from gi.repository import Gio, GLib

ACCESSIBLE = "org.a11y.atspi.Accessible"
PROPERTIES = "org.freedesktop.DBus.Properties"
ROOT = "/org/a11y/atspi/accessible/root"


class Accessibility:
    def __enter__(self):
        self.deadline = time.monotonic() + 12
        session = Gio.bus_get_sync(Gio.BusType.SESSION, None)
        address = self.request(
            session,
            ("org.a11y.Bus", "/org/a11y/bus"),
            "org.a11y.Bus",
            "GetAddress",
        )
        self.bus = Gio.DBusConnection.new_for_address_sync(
            address,
            Gio.DBusConnectionFlags.AUTHENTICATION_CLIENT
            | Gio.DBusConnectionFlags.MESSAGE_BUS_CONNECTION,
            None,
            None,
        )
        self.names = {}
        self.applications_seen = []
        return self

    def __exit__(self, *_error):
        self.bus.close_sync(None)

    def request(self, bus, item, interface, method, parameters=None):
        assert time.monotonic() < self.deadline, (
            "AT-SPI traversal deadline",
            item,
            interface,
            method,
        )
        try:
            return bus.call_sync(
                *item,
                interface,
                method,
                parameters,
                None,
                Gio.DBusCallFlags.NONE,
                1000,
                None,
            ).unpack()[0]
        except GLib.Error as error:
            raise AssertionError(
                ("AT-SPI request failed", item, interface, method, error.message)
            ) from error

    def call(self, item, method, parameters=None, interface=ACCESSIBLE):
        return self.request(self.bus, item, interface, method, parameters)

    def applications(self, pid=None):
        for reference in self.call(("org.a11y.atspi.Registry", ROOT), "GetChildren"):
            item = tuple(reference)
            process = self.call(
                ("org.freedesktop.DBus", "/org/freedesktop/DBus"),
                "GetConnectionUnixProcessID",
                GLib.Variant("(s)", (item[0],)),
                interface="org.freedesktop.DBus",
            )
            name = Path(f"/proc/{process}/comm")
            process_name = name.read_text().strip()
            self.applications_seen.append((process, process_name, item))
            if process == pid or process_name in (
                "xfce4-panel",
                "wrapper-2.0",
                "panel-1-systray",
                "gnome-shell",
                "plasmashell",
            ):
                yield item

    def diagnose(self):
        print(
            f"Native accessibility applications: {self.applications_seen}",
            file=sys.stderr,
            flush=True,
        )

    def descendants(self, roots):
        pending, visited = list(roots), set()
        while pending:
            item = pending.pop()
            if item in visited or item[1] == "/org/a11y/atspi/null":
                continue
            visited.add(item)
            assert len(visited) <= 4096, "AT-SPI tree exceeds fixture bounds"
            yield item
            pending.extend(tuple(child) for child in self.call(item, "GetChildren"))

    def name(self, item):
        if item not in self.names:
            self.names[item] = self.call(
                item,
                "Get",
                GLib.Variant("(ss)", (ACCESSIBLE, "Name")),
                interface=PROPERTIES,
            )
        return self.names[item]

    def label(self, item):
        for child in self.descendants([item]):
            if name := self.name(child):
                return name
            if "org.a11y.atspi.Text" in self.call(child, "GetInterfaces"):
                if text := self.call(
                    child,
                    "GetText",
                    GLib.Variant("(ii)", (0, -1)),
                    interface="org.a11y.atspi.Text",
                ):
                    return text
        return ""

    def showing(self, item, state):
        words = self.call(item, "GetState")
        bit = int(state)
        return bool(words[bit // 32] & (1 << (bit % 32)))

    def bounds(self, item):
        values = self.call(
            item,
            "GetExtents",
            GLib.Variant("(u)", (0,)),
            interface="org.a11y.atspi.Component",
        )
        return dict(zip(("x", "y", "width", "height"), values, strict=True))
