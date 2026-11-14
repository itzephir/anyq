"""Read actual compositor output-enter/leave events for the mapped main surface."""

import re


def current(trace):
    outputs, logical, surfaces, toplevels, titles, entered = {}, {}, {}, {}, {}, {}
    for line in trace.splitlines():
        if match := re.search(r"xdg_toplevel[@#](\d+)\.destroy\(\)", line):
            entered.pop(toplevels.pop(match[1], None), None)
            titles.pop(match[1], None)
        elif match := re.search(r"wl_surface[@#](\d+)\.destroy\(\)", line):
            entered.pop(match[1], None)
            for top, surface in list(toplevels.items()):
                if surface == match[1]:
                    toplevels.pop(top)
                    titles.pop(top, None)
        elif match := re.search(
            r"get_xdg_surface\(new id xdg_surface[@#](\d+), wl_surface[@#](\d+)\)", line
        ):
            surfaces[match[1]] = match[2]
        elif match := re.search(
            r"xdg_surface[@#](\d+)\.get_toplevel\(new id xdg_toplevel[@#](\d+)\)", line
        ):
            toplevels[match[2]] = surfaces.get(match[1])
        elif match := re.search(r'xdg_toplevel[@#](\d+)\.set_title\("([^"]*)"\)', line):
            titles[match[1]] = match[2]
        elif match := re.search(
            r"get_xdg_output\(new id zxdg_output_v1[@#](\d+), wl_output[@#](\d+)\)",
            line,
        ):
            logical[match[1]] = match[2]
        elif match := re.search(
            r"wl_output[@#](\d+)\.(geometry|scale)\(([-\d, ]+)", line
        ):
            output = outputs.setdefault(match[1], {})
            values = [int(value) for value in match[3].strip().strip(",").split(",")]
            output["scale" if match[2] == "scale" else "position"] = (
                values[0] if match[2] == "scale" else values[:2]
            )
        elif match := re.search(
            r"zxdg_output_v1[@#](\d+)\.logical_position\((-?\d+), (-?\d+)\)", line
        ):
            if output := logical.get(match[1]):
                outputs.setdefault(output, {})["logical_position"] = [
                    int(match[2]),
                    int(match[3]),
                ]
        elif match := re.search(
            r"wl_surface[@#](\d+)\.(enter|leave)\(wl_output[@#](\d+)\)", line
        ):
            selected = entered.setdefault(match[1], set())
            if match[2] == "enter":
                selected.add(match[3])
            else:
                selected.discard(match[3])
    main = next(
        (
            surface
            for surface in entered
            if any(
                surface == value and titles.get(top) == "anyq"
                for top, value in toplevels.items()
            )
        ),
        None,
    )
    return {
        "surface": main,
        "outputs": [
            {
                "id": identifier,
                "position": outputs[identifier].get(
                    "logical_position", outputs[identifier].get("position")
                ),
                "scale": outputs[identifier].get("scale"),
            }
            for identifier in sorted(entered.get(main, set()))
            if identifier in outputs
        ],
    }
