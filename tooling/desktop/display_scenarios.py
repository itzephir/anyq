"""Move actual native windows between configured displays and validate tray anchoring."""

import sys


def run(driver, native, monitors, profile):
    results = []
    if profile.get("backend") == "wayland":
        for direction in ["monitor-right", "monitor-left"]:
            before = driver.shell_state()["windows"]["main"]["monitor"]
            native.key(direction)
            after = driver.wait_shell(
                lambda state: state["windows"]["main"]["monitor"] != before
            )
            results.append(
                {
                    "scenario": "compositor-monitor-move",
                    "direction": direction,
                    "before": before,
                    "after": after["windows"]["main"]["monitor"],
                    "result": "passed",
                }
            )
        return results

    for monitor in monitors:
        target = "main" if sys.platform.startswith("linux") else "compact"
        scale = monitor["scale"] if sys.platform.startswith("linux") else 1
        driver.call(
            window=target,
            native="position",
            x=(monitor["x"] + 30) * scale,
            y=(monitor["y"] + 60) * scale,
        )

        def moved(state):
            window = state["windows"][target]
            divisor = window["scale"] if sys.platform == "darwin" else scale
            return (
                abs(window["position"]["x"] / divisor - monitor["x"] - 30) <= 4
                and abs(window["position"]["y"] / divisor - monitor["y"] - 60) <= 4
            )

        driver.wait_shell(moved)
        if target == "compact":
            icon = native.click_tray(driver.process.pid)
            state = driver.wait_shell(
                lambda state: state["windows"]["compact"]["visible"]
                and state["windows"]["compact"]["focused"]
            )
            rect = state["windows"]["compact"]
            x, y = rect["position"]["x"], rect["position"]["y"]
            width, height = rect["size"]["width"], rect["size"]["height"]
            if sys.platform == "darwin":
                x, y, width, height = (
                    value / rect["scale"] for value in (x, y, width, height)
                )
            center_x, center_y = (
                icon["x"] + icon["width"] / 2,
                icon["y"] + icon["height"] / 2,
            )
            anchors = [
                area
                for area in monitors
                if area["bounds"]["x"]
                <= center_x
                < area["bounds"]["x"] + area["bounds"]["width"]
                and area["bounds"]["y"]
                <= center_y
                < area["bounds"]["y"] + area["bounds"]["height"]
            ]
            assert len(anchors) == 1, (icon, monitors)
            area = anchors[0]
            assert (
                area["x"] <= x
                and area["y"] <= y
                and x + width <= area["x"] + area["width"]
                and y + height <= area["y"] + area["height"]
            ), state
            native.key("escape")
            driver.wait_shell(lambda state: not state["windows"]["compact"]["visible"])
            native.click_tray(driver.process.pid)
            driver.wait_shell(lambda state: state["windows"]["compact"]["visible"])
            main = driver.shell_state()["windows"]["main"]
            divisor = main["scale"] if sys.platform == "darwin" else 1
            native.call(
                "click",
                x=main["position"]["x"] / divisor + 50,
                y=main["position"]["y"] / divisor + 100,
            )
            driver.wait_shell(lambda state: not state["windows"]["compact"]["visible"])
        results.append(
            {
                "scenario": "native-display-roundtrip",
                "monitor": monitor,
                "result": "passed",
            }
        )
    return results
