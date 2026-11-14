"""Check native popup geometry against its visible icon and monitor contract."""


def verify(window, icon, monitors, *, logical_coordinates):
    center = (icon["x"] + icon["width"] / 2, icon["y"] + icon["height"] / 2)
    candidates = [
        area
        for area in monitors
        if all(
            area["bounds"][axis] <= point < area["bounds"][axis] + area["bounds"][size]
            for axis, size, point in zip(("x", "y"), ("width", "height"), center)
        )
    ]
    assert len(candidates) == 1, ("Icon monitor", icon, monitors)
    area = candidates[0]
    assert window["scale"] == area["scale"], ("Popup DPI", window, area)
    divisor = window["scale"] if logical_coordinates else 1
    actual = {
        "x": window["client_position"]["x"] / divisor,
        "y": window["client_position"]["y"] / divisor,
        "width": window["client_size"]["width"] / divisor,
        "height": window["client_size"]["height"] / divisor,
    }
    unit = 1 if logical_coordinates else area["scale"]
    margin = 8 * unit
    inset_x = min(margin, (area["width"] - 1) / 2)
    inset_y = min(margin, (area["height"] - 1) / 2)
    width = min(380 * unit, area["width"] - 2 * inset_x)
    height = min(355 * unit, area["height"] - 2 * inset_y)
    left, top = area["x"] + inset_x, area["y"] + inset_y
    right, bottom = (
        area["x"] + area["width"] - inset_x,
        area["y"] + area["height"] - inset_y,
    )
    below = icon["y"] + icon["height"] + margin
    y = below if below + height <= bottom else icon["y"] - margin - height
    expected = {
        "x": max(left, min(center[0] - width / 2, right - width)),
        "y": max(top, min(y, bottom - height)),
        "width": width,
        "height": height,
    }
    assert all(abs(actual[key] - value) <= 2 for key, value in expected.items()), (
        "Popup size and icon anchoring",
        actual,
        expected,
        icon,
        area,
    )
    return {"actual": actual, "expected": expected, "icon": icon, "monitor": area}
