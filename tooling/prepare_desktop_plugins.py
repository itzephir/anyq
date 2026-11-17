"""Build and stage independent plugins for the desktop application's target."""

import argparse
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
NAMES = ("shell", "timeout")
MACOS_TARGETS = ("aarch64-apple-darwin", "x86_64-apple-darwin")


def host_target():
    version = subprocess.check_output(["rustc", "-vV"], text=True)
    return next(
        line.removeprefix("host: ")
        for line in version.splitlines()
        if line.startswith("host: ")
    )


def build(target):
    command = ["cargo", "build", "--locked", "--release", "-p", "anyq"]
    for name in NAMES:
        command.extend(["--bin", f"anyq-{name}"])
    command.extend(["--target", target, "--message-format=json-render-diagnostics"])
    result = subprocess.run(
        command, cwd=ROOT, check=True, text=True, stdout=subprocess.PIPE
    )
    artifacts = {}
    for line in result.stdout.splitlines():
        message = json.loads(line)
        if message["reason"] == "compiler-artifact" and message.get("executable"):
            for name in NAMES:
                if message["target"]["name"] == f"anyq-{name}":
                    artifacts[name] = Path(message["executable"])
    if set(artifacts) != set(NAMES):
        raise RuntimeError(f"Cargo did not report both plugin executables for {target}")
    return artifacts


def verify_macos(path, targets):
    expected = {
        "arm64" if target.startswith("aarch64-") else target.split("-")[0]
        for target in targets
    }
    actual = set(
        subprocess.check_output(["lipo", "-archs", str(path)], text=True).split()
    )
    if actual != expected:
        raise RuntimeError(
            f"Wrong plugin architectures in {path}: {actual}, expected {expected}"
        )


def prepare(target, destination):
    targets = MACOS_TARGETS if target == "universal-apple-darwin" else (target,)
    builds = {triple: build(triple) for triple in targets}
    destination.mkdir(parents=True, exist_ok=True)
    suffix = ".exe" if "windows" in target else ""
    with tempfile.TemporaryDirectory(prefix="anyq-desktop-plugins-") as temporary:
        for name in NAMES:
            source = builds[targets[0]][name]
            if len(targets) == 2:
                source = Path(temporary) / name
                subprocess.run(
                    [
                        "lipo",
                        "-create",
                        *(str(builds[triple][name]) for triple in targets),
                        "-output",
                        str(source),
                    ],
                    check=True,
                )
            if target.endswith("apple-darwin"):
                verify_macos(source, targets)
            shutil.copy2(source, destination / f"{name}{suffix}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output", type=Path, default=ROOT / "apps/desktop/resources/plugins"
    )
    arguments = parser.parse_args()
    target = (
        os.environ.get("TAURI_ENV_TARGET_TRIPLE")
        or os.environ.get("CARGO_BUILD_TARGET")
        or host_target()
    )
    prepare(target, arguments.output)


if __name__ == "__main__":
    main()
