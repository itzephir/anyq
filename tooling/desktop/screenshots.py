"""Capture actual native client pixels; verify or explicitly generate PNG goldens."""

import json
import sys
import time
from pathlib import Path

from PIL import Image, ImageChops, ImageGrab


def platform_name():
    return {"darwin": "macos", "win32": "windows"}.get(sys.platform, "linux")


def changed_pixels(expected, actual, threshold):
    if expected.size != actual.size:
        return None, None
    difference = ImageChops.difference(expected.convert("RGB"), actual.convert("RGB"))
    red, green, blue = difference.split()
    maximum = ImageChops.lighter(ImageChops.lighter(red, green), blue)
    return sum(maximum.histogram()[threshold + 1 :]), difference


class Screenshots:
    def __init__(self, output, golden_root, mode, comparison):
        assert mode in ("verify", "generate")
        self.platform = platform_name()
        self.output = Path(output) / self.platform
        self.goldens = Path(golden_root) / self.platform
        self.output.mkdir(parents=True, exist_ok=True)
        self.mode = mode
        self.comparison = comparison
        self.failures = []
        self.results = []

    def capture(self, driver, case, repetition):
        name, window = case["id"], case["surface"]
        window = "compact" if window == "compact" else "main"
        print(
            f"Native screenshot: {self.platform}/{name}, independent run {repetition + 1}",
            flush=True,
        )
        rules = ",".join(case["mask_selectors"])
        style = "*,*::before,*::after{animation:none!important;transition:none!important;caret-color:transparent!important}"
        if rules:
            style += rules + "{visibility:hidden!important}"
        style += ".source-card p{height:20px;overflow:hidden;white-space:nowrap}"
        result = driver.call(
            f"const style=document.createElement('style');style.textContent={json.dumps(style)};document.head.append(style);"
            "document.activeElement?.blur();"
            f"document.querySelector('.content')?.scrollTo(0,{case['scroll']['content']});"
            f"document.querySelector('.topic-list')?.scrollTo(0,{case['scroll']['topics']});"
            f"scrollTo(0,{case['scroll']['body']});"
            + (
                f"document.querySelector({json.dumps(case['focus'])}).focus();"
                if case["focus"]
                else ""
            )
            + "return {width:innerWidth,height:innerHeight,text:document.body.innerText};",
            window,
        )
        viewport = result["payload"]["value"]
        assert [viewport["width"], viewport["height"]] == case["viewport"]
        outer, scale = result["outer"], result["rect"]["scale"]
        width, height = (int(value * scale) for value in case["viewport"])
        border = max(0, (outer["width"] - width) // 2)
        inset_top = max(0, outer["height"] - height - border)
        time.sleep(0.15)
        if sys.platform == "win32":
            # PrintWindow captures the real window even when the taskbar overlaps it.
            captured = ImageGrab.grab(window=result["native_handle"]).convert("RGB")
            if captured.size == (width, height):
                actual = captured
            else:
                assert captured.width >= width and captured.height >= height, (
                    captured.size,
                    result,
                )
                actual = captured.crop(
                    (border, inset_top, border + width, inset_top + height)
                )
        else:
            divisor = scale if sys.platform == "darwin" else 1
            x = (outer["x"] + border) / divisor
            y = (outer["y"] + inset_top) / divisor
            actual = ImageGrab.grab(
                bbox=(
                    round(x),
                    round(y),
                    round(x + width / divisor),
                    round(y + height / divisor),
                )
            ).convert("RGB")
        if actual.size != tuple(case["viewport"]):
            actual = actual.resize(tuple(case["viewport"]), Image.Resampling.LANCZOS)
        colors = actual.getcolors(maxcolors=32)
        assert colors is None or len(colors) > 1, f"Blank native capture: {name}"
        suffix = "actual" if repetition == 0 else "repeat"
        actual.save(self.output / f"{name}.{suffix}.png")
        (self.output / f"{name}.{suffix}.json").write_text(
            json.dumps(result, indent=2), encoding="utf-8"
        )
        return actual

    def compare(self, case, first, repeat):
        name = case["id"]
        deterministic, difference = changed_pixels(
            first, repeat, self.comparison["determinism_channel_delta"]
        )
        if (
            deterministic is None
            or deterministic > self.comparison["determinism_max_changed_pixels"]
        ):
            if difference is not None:
                difference.save(self.output / f"{name}.determinism.diff.png")
            self.failures.append(
                f"Nondeterministic native screenshot: {name}: {deterministic} pixels"
            )
            self.results.append(
                {
                    "name": name,
                    "result": "nondeterministic",
                    "changed_pixels": deterministic,
                }
            )
            return
        target = self.goldens / f"{name}.png"
        if self.mode == "generate":
            self.goldens.mkdir(parents=True, exist_ok=True)
            first.save(target)
            self.results.append(
                {"name": name, "result": "generated", "deterministic": True}
            )
            return
        if not target.exists():
            self.failures.append(f"Missing reviewed golden: {target}")
            self.results.append(
                {"name": name, "result": "missing", "deterministic": True}
            )
            return
        with Image.open(target) as expected:
            changed, difference = changed_pixels(
                expected, first, self.comparison["channel_delta"]
            )
        if difference is not None:
            difference.save(self.output / f"{name}.diff.png")
        self.results.append(
            {"name": name, "changed_pixels": changed, "deterministic": True}
        )
        if changed is None or changed > self.comparison["max_changed_pixels"]:
            self.failures.append(
                f"Visual regression: {name}: {changed} pixels; comparison {self.comparison}"
            )

    def finish(self):
        (self.output / "report.json").write_text(
            json.dumps({"mode": self.mode, "cases": self.results}, indent=2) + "\n",
            encoding="utf-8",
        )
        if self.failures:
            raise AssertionError("\n".join(self.failures))
