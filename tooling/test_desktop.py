#!/usr/bin/env python3
"""Run configured native integration and deterministic verify/generate screenshot cases."""

import argparse
import importlib
import json
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent / "desktop"))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", type=Path, required=True)
    parser.add_argument("--panel", type=Path, required=True)
    parser.add_argument("--mode", choices=("verify", "generate"), default="verify")
    parser.add_argument(
        "--config", type=Path, default=Path("tests/desktop/config.json")
    )
    parser.add_argument("--output", type=Path, default=Path("desktop-test-results"))
    parser.add_argument("--goldens", type=Path, default=Path("tests/desktop/goldens"))
    arguments = parser.parse_args()
    configuration = json.loads(arguments.config.read_text(encoding="utf-8"))
    assert configuration["version"] == 1 and configuration["timezone"] == "UTC"
    cli, panel, output = (
        arguments.cli.resolve(),
        arguments.panel.resolve(),
        arguments.output.resolve(),
    )
    with tempfile.TemporaryDirectory(prefix="anyq-desktop-integration-") as home:
        importlib.import_module("scenarios").run(
            cli, panel, home, output, configuration
        )
    importlib.import_module("visual_cases").run(
        cli, panel, output, arguments.goldens.resolve(), configuration, arguments.mode
    )
    print(f"Native integration and deterministic screenshot {arguments.mode} passed")


if __name__ == "__main__":
    main()
