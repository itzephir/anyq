#!/usr/bin/env python3
"""Run integration and reviewed native screenshot scenarios on every desktop OS."""

import argparse
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent / "desktop"))

from importlib import import_module

run = import_module("scenarios").run


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", type=Path, required=True)
    parser.add_argument("--panel", type=Path, required=True)
    parser.add_argument("--output", type=Path, default=Path("desktop-test-results"))
    parser.add_argument("--goldens", type=Path, default=Path("tests/desktop/goldens"))
    parser.add_argument("--update-goldens", action="store_true")
    arguments = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="anyq-desktop-e2e-") as home:
        run(
            arguments.cli.resolve(),
            arguments.panel.resolve(),
            home,
            arguments.output.resolve(),
            arguments.goldens.resolve(),
            arguments.update_goldens,
        )
    print("Native integration and screenshot scenarios passed")


if __name__ == "__main__":
    main()
