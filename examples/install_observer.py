"""Build an example plugin-owned launcher; the framework receives only its name."""

import os
import subprocess
from pathlib import Path


def install(home, interpreter, script, *arguments):
    directory = Path(home) / ".anyq/plugins"
    directory.mkdir(parents=True, exist_ok=True)
    target = directory / ("observer.exe" if os.name == "nt" else "observer")
    subprocess.run(
        [
            "rustc",
            "--edition=2024",
            str(Path(__file__).with_name("plugin_launcher.rs")),
            "-o",
            str(target),
        ],
        check=True,
        timeout=60,
    )
    target.with_suffix(".launcher").write_text(
        "\n".join(map(str, [interpreter, script, *arguments])) + "\n", encoding="utf-8"
    )
    return target
