#!/usr/bin/env python3
"""Exercise real desktop hosts, native menus and display configurations."""

import argparse
import json
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent / "desktop"))


def main():
    from shell_scenarios import run

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", type=Path, required=True)
    parser.add_argument("--panel", type=Path, required=True)
    parser.add_argument("--profile", required=True)
    parser.add_argument("--output", type=Path, default=Path("desktop-shell-results"))
    arguments = parser.parse_args()
    configuration = json.loads(
        Path("tests/desktop/config.json").read_text(encoding="utf-8")
    )
    profiles = json.loads(Path("tests/desktop/native.json").read_text(encoding="utf-8"))
    with tempfile.TemporaryDirectory(prefix="anyq-native-shell-") as home:
        run(
            arguments.cli.resolve(),
            arguments.panel.resolve(),
            home,
            arguments.output.resolve(),
            configuration,
            profiles["profiles"][arguments.profile],
        )
    print("Real native tray/menu, display and owner-lifetime scenarios passed")


if __name__ == "__main__":
    main()
