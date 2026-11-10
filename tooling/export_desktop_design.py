#!/usr/bin/env python3
"""Export editable desktop previews and licensed assets for OpenDesign."""

import argparse
import base64
import re
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "apps/desktop/ui"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    output = parser.parse_args().output
    output.mkdir(parents=True, exist_ok=True)
    fonts = {
        f"assets/{name}.ttf": "data:font/ttf;base64,"
        + base64.b64encode((SOURCE / "assets" / f"{name}.ttf").read_bytes()).decode()
        for name in ("ibm-plex-sans", "ibm-plex-mono")
    }
    for path in SOURCE.rglob("*"):
        if not path.is_file() or path.suffix not in (
            ".html",
            ".css",
            ".js",
            ".txt",
            ".ttf",
            ".svg",
            ".md",
        ):
            continue
        target = output / path.relative_to(SOURCE)
        target.parent.mkdir(parents=True, exist_ok=True)
        if path.suffix == ".ttf":
            shutil.copyfile(path, target)
            continue
        content = path.read_text()
        for font, data in fonts.items():
            content = content.replace(font, data)
        if path.suffix == ".html":
            content = content.replace("<body", '<body data-preview="true"', 1)
            content = content.replace(
                'type="module" src="panel.js"', 'defer src="preview-bundle.js"'
            )
        target.write_text(content)

    # OpenDesign's opaque preview sandbox cannot use authenticated ES modules.
    # Keep the native source modular and export a classic script for that sandbox.
    bundle = []
    bundle.append(
        (SOURCE / "icons.js").read_text().replace("export default", "const icons =", 1)
    )
    for language in ("en", "ru"):
        content = (SOURCE / "locales" / f"{language}.js").read_text()
        bundle.append(content.replace("export default", f"const {language} =", 1))
    bundle.append(
        (SOURCE / "preview.js").read_text().replace("export function", "function")
    )
    bundle.append(
        re.sub(r"^import[^\n]*\n", "", (SOURCE / "panel.js").read_text(), flags=re.M)
    )
    (output / "preview-bundle.js").write_text("\n".join(bundle))

    template = (output / "index.html").read_text()
    for surface, state, language in (
        ("main", "live", "en"),
        ("compact", "live", "en"),
        ("settings", "live", "en"),
        ("settings", "live", "ru"),
        ("main", "stale", "en"),
        ("main", "empty", "en"),
        ("main", "error", "en"),
    ):
        for theme in ("dark", "light"):
            attributes = (
                f'data-surface="{surface}" data-state="{state}" '
                f'data-theme="{theme}" data-language="{language}"'
            )
            target = output / f"{surface}-{state}-{language}-{theme}.html"
            target.write_text(template.replace("<body>", f"<body {attributes}>"))
    print(f"Exported desktop design to {output}")


if __name__ == "__main__":
    main()
