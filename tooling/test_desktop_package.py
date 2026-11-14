#!/usr/bin/env python3
"""Exercise installed production desktop packages on disposable native CI machines."""

import argparse
import json
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent / "desktop"))


def main():
    from package_scenarios import run

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cli", type=Path, required=True)
    parser.add_argument("--bundles", type=Path, default=Path("target/release/bundle"))
    parser.add_argument("--output", type=Path, default=Path("desktop-package-results"))
    arguments = parser.parse_args()
    configuration = json.loads(
        Path("tests/desktop/config.json").read_text(encoding="utf-8")
    )
    with tempfile.TemporaryDirectory(prefix="anyq-installed-desktop-") as home:
        run(
            arguments.cli.resolve(),
            arguments.bundles.resolve(),
            home,
            arguments.output.resolve(),
            configuration,
        )
    print(
        "Installed production package launch/removal/reinstall preserved the owner, queue and preferences"
    )


if __name__ == "__main__":
    main()
