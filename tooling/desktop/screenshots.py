"""Capture real native window pixels and compare reviewed platform PNGs."""

import json
import sys
import time
from pathlib import Path

from PIL import Image, ImageChops, ImageGrab


class Screenshots:
    def __init__(self, output, golden_root, update=False):
        self.platform = {"darwin": "macos", "win32": "windows"}.get(
            sys.platform, "linux"
        )
        self.output = Path(output) / self.platform
        self.goldens = Path(golden_root) / self.platform
        self.output.mkdir(parents=True, exist_ok=True)
        self.update = update
        self.failures = []
        self.results = []

    def capture(self, driver, name, window="main"):
        print(f"Native screenshot: {self.platform}/{name}", flush=True)
        result = driver.call(
            """if(!document.getElementById('test-screenshot-styles')) {
              const style=document.createElement('style');style.id='test-screenshot-styles';
              style.textContent='*,*::before,*::after{animation:none!important;transition:none!important;caret-color:transparent!important}.updated,.source-path,.source-card p{visibility:hidden!important}';document.head.append(style);
            }
            document.activeElement?.blur();return {width:innerWidth,height:innerHeight,text:document.body.innerText};""",
            window,
        )
        rect = result["rect"]
        # The compositor settles after the renderer's two animation frames.
        time.sleep(0.15)
        scale = rect["scale"] if sys.platform == "darwin" else 1
        x, y, width, height = (
            rect[key] / scale for key in ("x", "y", "width", "height")
        )
        actual = ImageGrab.grab(
            bbox=(int(x), int(y), int(x + width), int(y + height))
        ).convert("RGB")
        viewport = result["payload"]["value"]
        expected_size = (viewport["width"], viewport["height"])
        if actual.size != expected_size:
            actual = actual.resize(expected_size, Image.Resampling.LANCZOS)
        colors = actual.getcolors(maxcolors=32)
        if colors is not None and len(colors) <= 1:
            raise AssertionError(f"Blank native capture: {name}")
        path = self.output / f"{name}.png"
        actual.save(path)
        golden = self.goldens / path.name
        if self.update:
            self.goldens.mkdir(parents=True, exist_ok=True)
            actual.save(golden)
            self.results.append({"name": name, "result": "updated"})
            return
        if not golden.exists():
            self.failures.append(f"Missing reviewed golden: {golden}")
            self.results.append({"name": name, "result": "missing"})
            return
        with Image.open(golden) as expected:
            expected = expected.convert("RGB")
            if expected.size != actual.size:
                self.failures.append(
                    f"Size changed: {name}: {expected.size} -> {actual.size}"
                )
                return
            difference = ImageChops.difference(expected, actual)
            channels = difference.split()
            maximum = ImageChops.lighter(
                ImageChops.lighter(channels[0], channels[1]), channels[2]
            )
            changed = sum(maximum.histogram()[9:])
            difference.save(self.output / f"{name}.diff.png")
            self.results.append({"name": name, "changed_pixels": changed})
            if changed > 50:
                self.failures.append(
                    f"Visual regression: {name}: {changed} pixels (channel delta > 8; limit 50)"
                )

    def finish(self):
        (self.output / "report.json").write_text(
            json.dumps(self.results, indent=2) + "\n"
        )
        if self.failures:
            raise AssertionError("\n".join(self.failures))
