"""Capture actual native client pixels; verify or explicitly generate PNG goldens."""

import json
import sys
import time
from pathlib import Path

from PIL import Image, ImageChops, ImageGrab
from pixelmatch.contrib.PIL import pixelmatch


def platform_name():
    return {"darwin": "macos", "win32": "windows"}.get(sys.platform, "linux")


def changed_pixels(expected, actual, comparison):
    if expected.size != actual.size:
        return None, None
    difference = Image.new("RGBA", expected.size)
    changed = pixelmatch(
        expected,
        actual,
        output=difference,
        threshold=comparison["perceptual_threshold"],
        includeAA=comparison["include_antialiasing"],
    )
    return changed, difference.convert("RGB")


def raw_changed_pixels(first, second):
    if first.size != second.size:
        return None
    red, green, blue = ImageChops.difference(
        first.convert("RGB"), second.convert("RGB")
    ).split()
    maximum = ImageChops.lighter(ImageChops.lighter(red, green), blue)
    return sum(maximum.histogram()[1:])


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
            + "const typography=[...document.querySelectorAll('.empty p')].map(element=>{"
            "const css=getComputedStyle(element);const range=document.createRange();range.selectNodeContents(element);"
            "return {font:css.font,smoothing:css.webkitFontSmoothing,lines:[...range.getClientRects()].map(rect=>[rect.x,rect.y,rect.width,rect.height])};});"
            "return {width:innerWidth,height:innerHeight,text:document.body.innerText,typography,"
            "fonts:[...document.fonts].map(font=>({family:font.family,weight:font.weight,status:font.status}))};",
            window,
            native="cursor",
            x=case["cursor"][0],
            y=case["cursor"][1],
        )
        viewport = result["payload"]["value"]
        assert [viewport["width"], viewport["height"]] == case["viewport"]
        outer, scale = result["outer"], result["rect"]["scale"]
        width, height = (int(value * scale) for value in case["viewport"])
        border = max(0, (outer["width"] - width) // 2)
        inset_top = max(0, outer["height"] - height - border)
        work = result["work_area"]
        assert work is not None, "Native capture requires a visible monitor"
        assert (
            work["x"] <= outer["x"] + border
            and work["y"] <= outer["y"] + inset_top
            and outer["x"] + border + width <= work["x"] + work["width"]
            and outer["y"] + inset_top + height <= work["y"] + work["height"]
        ), f"Configured client viewport is outside the visible work area: {result}"
        divisor = scale if sys.platform == "darwin" else 1
        x = (outer["x"] + border) / divisor
        y = (outer["y"] + inset_top) / divisor

        # Capture displayed pixels. PrintWindow can return partially blank
        # WebView2 surfaces despite a fully loaded DOM and a visible window.
        def grab():
            frame = ImageGrab.grab(
                bbox=(
                    round(x),
                    round(y),
                    round(x + width / divisor),
                    round(y + height / divisor),
                )
            ).convert("RGB")
            if frame.size != tuple(case["viewport"]):
                frame = frame.resize(tuple(case["viewport"]), Image.Resampling.LANCZOS)
            return frame

        readiness = case["capture_stability"]
        deadline = time.monotonic() + readiness["timeout_seconds"]
        previous, stable, frames = grab(), 0, 1
        while True:
            time.sleep(readiness["interval_seconds"])
            actual = grab()
            frames += 1
            changed, difference = changed_pixels(previous, actual, self.comparison)
            stable = stable + 1 if changed == 0 else 0
            if stable >= readiness["consecutive_frames"]:
                break
            if time.monotonic() >= deadline:
                actual.save(self.output / f"{name}.unstable.png")
                if difference is not None:
                    difference.save(self.output / f"{name}.unstable.diff.png")
                raise AssertionError(f"Native renderer did not settle: {name}")
            previous = actual
        result["capture_frames"] = frames
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
        deterministic, difference = changed_pixels(first, repeat, self.comparison)
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
        raw_repeated = raw_changed_pixels(first, repeat)
        target = self.goldens / f"{name}.png"
        if self.mode == "generate":
            self.goldens.mkdir(parents=True, exist_ok=True)
            first.save(target)
            self.results.append(
                {
                    "name": name,
                    "result": "generated",
                    "deterministic": True,
                    "raw_repeat_pixels": raw_repeated,
                }
            )
            return
        if not target.exists():
            self.failures.append(f"Missing reviewed golden: {target}")
            self.results.append(
                {"name": name, "result": "missing", "deterministic": True}
            )
            return
        with Image.open(target) as expected:
            changed, difference = changed_pixels(expected, first, self.comparison)
            raw_golden = raw_changed_pixels(expected, first)
            repeated, repeat_difference = changed_pixels(
                expected, repeat, self.comparison
            )
        if repeat_difference is not None:
            repeat_difference.save(self.output / f"{name}.repeat.diff.png")
        if difference is not None:
            difference.save(self.output / f"{name}.diff.png")
        self.results.append(
            {
                "name": name,
                "changed_pixels": changed,
                "repeat_changed_pixels": repeated,
                "raw_golden_pixels": raw_golden,
                "raw_repeat_pixels": raw_repeated,
                "deterministic": True,
            }
        )
        if (
            changed is None
            or repeated is None
            or max(changed, repeated) > self.comparison["max_changed_pixels"]
        ):
            self.failures.append(
                f"Visual regression: {name}: {changed} pixels; comparison {self.comparison}"
            )

    def finish(self):
        (self.output / "report.json").write_text(
            json.dumps(
                {
                    "mode": self.mode,
                    "comparison": self.comparison,
                    "cases": self.results,
                },
                indent=2,
            )
            + "\n",
            encoding="utf-8",
        )
        if self.failures:
            raise AssertionError("\n".join(self.failures))
