"""Build the allowlisted static website and its editable OpenDesign copy."""

import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "website/dist"


def main():
    if OUTPUT.exists():
        shutil.rmtree(OUTPUT)

    OUTPUT.mkdir()
    for name in ("index.html", "style.css", "theme.js", "site.js", "favicon.svg"):
        shutil.copy2(ROOT / "website" / name, OUTPUT / name)

    page = (OUTPUT / "index.html").read_text(encoding="utf-8")
    for theme in ("dark", "light"):
        preview = page.replace(
            'data-theme="dark"', f'data-theme="{theme}" data-preview-theme="{theme}"', 1
        )
        (OUTPUT / f"{theme}.html").write_text(preview, encoding="utf-8")

    shutil.copytree(ROOT / "website/assets", OUTPUT / "assets")
    for name in ("install.sh", "install.ps1"):
        shutil.copy2(ROOT / "install" / name, OUTPUT / name)

    shutil.copy2(ROOT / "website/DESIGN.md", OUTPUT / "DESIGN.md")
    print(f"Built {len(list(OUTPUT.rglob('*')))} website entries")


if __name__ == "__main__":
    main()
