"""Move actual native windows between configured displays and validate tray anchoring."""

import json
import sys

import popup_geometry
import wayland_outputs


def run(driver, native, monitors, profile):
    results = []
    if profile.get("backend") == "wayland":
        targets = sorted(
            native.call("monitors"),
            key=lambda monitor: monitor["bounds"]["x"],
        )
        assert len(targets) == 2, targets
        for direction, target in zip(
            ["monitor-right", "monitor-left"], [targets[1], targets[0]], strict=True
        ):

            def actual():
                return wayland_outputs.current(
                    (driver.home / "panel.log").read_text(errors="replace")
                )

            before = actual()
            native.key(direction)

            def arrived(state):
                observed = actual()["outputs"]
                return (
                    len(observed) == 1
                    and observed[0]["position"]
                    == [target["bounds"]["x"], target["bounds"]["y"]]
                    and observed[0]["scale"] == target["scale"]
                    and state["windows"]["main"]["scale"] == target["scale"]
                )

            try:
                after = driver.wait_shell(arrived)
            except AssertionError as error:
                raise AssertionError(
                    {"wayland": actual(), "target": target, "gtk": driver.shell_state()}
                ) from error
            results.append(
                {
                    "scenario": "compositor-monitor-move",
                    "direction": direction,
                    "before": before,
                    "after": actual(),
                    "gtk_monitor": after["windows"]["main"]["monitor"],
                    "result": "passed",
                }
            )
            print(json.dumps(results[-1]), flush=True)
        return results

    if not sys.platform.startswith("linux"):
        return popups(driver, native, monitors)

    for monitor in monitors:
        move(driver, "main", monitor)
        results.append(
            {
                "scenario": "native-display-roundtrip",
                "monitor": monitor,
                "result": "passed",
            }
        )
    return results


def move(driver, label, monitor):
    scale = monitor["scale"] if sys.platform.startswith("linux") else 1
    driver.call(
        window=label,
        native="position",
        x=(monitor["x"] + 30) * scale,
        y=(monitor["y"] + 60) * scale,
    )

    def moved(state):
        window = state["windows"][label]
        divisor = window["scale"] if sys.platform == "darwin" else scale
        return all(
            abs(window["position"][axis] / divisor - monitor[axis] - offset) <= 4
            for axis, offset in (("x", 30), ("y", 60))
        )

    driver.wait_shell(moved)


def popups(driver, native, monitors):
    identifier = "id" if sys.platform == "darwin" else "device"
    results, scales = [], set()
    for original in monitors:
        identity = original[identifier]
        native.call("tray-monitor", monitor=identity)
        current = native.wait(
            "monitors",
            predicate=lambda items: any(
                item[identifier] == identity and item["primary"] for item in items
            ),
        )
        target = next(item for item in current if item[identifier] == identity)
        move(driver, "main", target)
        native.wait("focus", pid=driver.process.pid)
        main = driver.shell_state()["windows"]["main"]
        divisor = main["scale"] if sys.platform == "darwin" else 1
        native.call(
            "click",
            x=main["position"]["x"] / divisor + 50,
            y=main["position"]["y"] / divisor + 100,
        )
        move(driver, "compact", target)
        icon = native.click_tray(driver.process.pid, monitor=target)

        def placed(state):
            window = state["windows"]["compact"]
            if not window["visible"] or not window["focused"]:
                return False
            try:
                popup_geometry.verify(
                    window, icon, current, logical_coordinates=sys.platform == "darwin"
                )
                return True
            except AssertionError:
                return False

        try:
            state = driver.wait_shell(placed)
        except AssertionError:
            popup_geometry.verify(
                driver.shell_state()["windows"]["compact"],
                icon,
                current,
                logical_coordinates=sys.platform == "darwin",
            )
            raise
        geometry = popup_geometry.verify(
            state["windows"]["compact"],
            icon,
            current,
            logical_coordinates=sys.platform == "darwin",
        )
        assert geometry["monitor"][identifier] == identity, geometry
        scales.add(geometry["monitor"]["scale"])
        results.append(
            {
                "scenario": "native-popup-placement",
                "original_monitor": original,
                **geometry,
                "result": "passed",
            }
        )
        print(json.dumps(results[-1]), flush=True)
        native.key("escape")
        driver.wait_shell(lambda state: not state["windows"]["compact"]["visible"])
        native.click_tray(driver.process.pid, monitor=target)
        driver.wait_shell(placed)
        native.call(
            "click",
            x=main["position"]["x"] / divisor + 50,
            y=main["position"]["y"] / divisor + 100,
        )
        driver.wait_shell(lambda state: not state["windows"]["compact"]["visible"])
    assert ({1, 2} if sys.platform == "darwin" else {1, 1.5}).issubset(scales), (
        "Opened popup scales",
        scales,
    )
    original_primary = next(item for item in monitors if item["primary"])
    native.call("tray-monitor", monitor=original_primary[identifier])
    restored = native.wait(
        "monitors",
        predicate=lambda items: any(
            item[identifier] == original_primary[identifier] and item["primary"]
            for item in items
        ),
    )
    move(
        driver,
        "main",
        next(
            item
            for item in restored
            if item[identifier] == original_primary[identifier]
        ),
    )
    native.wait("focus", pid=driver.process.pid)
    return results
