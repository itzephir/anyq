"""Package one target, or verify and checksum the complete release set."""

import argparse
import hashlib
import shutil
import tarfile
import zipfile
from pathlib import Path

TARGETS = (
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-unknown-linux-musl",
    "x86_64-unknown-linux-musl",
    "x86_64-pc-windows-msvc",
)


def asset_name(target):
    suffix = "zip" if "windows" in target else "tar.gz"
    return f"anyq-{target}.{suffix}"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", choices=TARGETS)
    parser.add_argument("--output", type=Path, default=Path("release-dist"))
    parser.add_argument("--checksums", action="store_true")
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)

    if args.checksums:
        names = [asset_name(target) for target in TARGETS]
        missing = [name for name in names if not (args.output / name).is_file()]
        if missing:
            parser.error(f"Incomplete release: {', '.join(missing)}")

        lines = []
        for name in names:
            digest = hashlib.sha256((args.output / name).read_bytes()).hexdigest()
            lines.append(f"{digest}  {name}\n")

        (args.output / "SHA256SUMS").write_text("".join(lines), encoding="ascii")
        for name in ("install.sh", "install.ps1"):
            shutil.copy2(Path("install") / name, args.output / name)
        return

    if not args.target:
        parser.error("--target or --checksums is required")

    suffix = ".exe" if "windows" in args.target else ""
    directory = Path("target") / args.target / "release"
    files = {f"anyq{suffix}": directory / f"anyq{suffix}"}
    for name in ("shell", "timeout"):
        files[f"plugins/{name}{suffix}"] = directory / f"anyq-{name}{suffix}"
    for binary in files.values():
        if not binary.is_file():
            parser.error(f"Missing release executable: {binary}")
    archive = args.output / asset_name(args.target)
    if archive.suffix == ".zip":
        with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as output:
            for name, binary in files.items():
                output.write(binary, name)
    else:
        with tarfile.open(archive, "w:gz") as output:
            for name, binary in files.items():
                output.add(binary, name)

    print(archive)


if __name__ == "__main__":
    main()
