"""Restricted SSH receiver for the public anyq website, isolated by stand."""

import hashlib
import io
import json
import os
import re
import shutil
import stat
import subprocess
import sys
import tempfile
import time
import zipfile
from pathlib import Path

ROOT = Path("/srv/anyq")
MAX_ARCHIVE = 8 * 1024 * 1024
MAX_CONTENT = 16 * 1024 * 1024
PUBLIC = {
    "index.html",
    "dark.html",
    "light.html",
    "style.css",
    "theme.js",
    "site.js",
    "favicon.svg",
    "install.sh",
    "install.ps1",
    "DESIGN.md",
}
REQUIRED = {
    "index.html",
    "style.css",
    "theme.js",
    "site.js",
    "install.sh",
    "install.ps1",
    "assets/ibm-plex-sans.ttf",
    "assets/ibm-plex-mono.ttf",
}
URLS = {"dev": "https://dev.anyq.itzephir.com/", "prod": "https://anyq.itzephir.com/"}


def unpack(payload):
    if len(payload) > MAX_ARCHIVE:
        raise ValueError("Website archive is too large")

    files = {}
    total = 0
    with zipfile.ZipFile(io.BytesIO(payload)) as archive:
        entries = archive.infolist()
        if not 1 <= len(entries) <= 64:
            raise ValueError("Unexpected website file count")

        for entry in entries:
            name = entry.filename
            valid_asset = re.fullmatch(
                r"assets/(?:[a-z0-9_-]+/)*[a-z0-9_-]+\.(?:ttf|woff2|svg|png|jpg|txt|md)",
                name,
                re.IGNORECASE,
            )
            mode = stat.S_IFMT(entry.external_attr >> 16)
            if (
                (name not in PUBLIC and not valid_asset)
                or name in files
                or mode not in (0, stat.S_IFREG)
                or entry.flag_bits & 1
            ):
                raise ValueError("Archive contains an unsupported file")

            total += entry.file_size
            if entry.file_size < 0 or total > MAX_CONTENT:
                raise ValueError("Expanded website is too large")

            with archive.open(entry) as stream:
                content = stream.read(MAX_CONTENT + 1)
            if len(content) != entry.file_size:
                raise ValueError("Invalid website file size")
            files[name] = content

    if not REQUIRED <= files.keys():
        raise ValueError("Website is incomplete")
    return files


def switch(current, target):
    if target is None:
        current.unlink(missing_ok=True)
        return

    pending = current.with_name(".current-next")
    pending.unlink(missing_ok=True)
    pending.symlink_to(target)
    pending.replace(current)


def healthy(stand, revision):
    for _attempt in range(20):
        result = subprocess.run(
            [
                "docker",
                "exec",
                f"anyq-site-{stand}",
                "wget",
                "-q",
                "-O",
                "-",
                "http://127.0.0.1:8080/revision.txt",
            ],
            capture_output=True,
            text=True,
            timeout=5,
        )
        if result.returncode == 0 and result.stdout.strip() == revision:
            return
        time.sleep(0.3)
    raise RuntimeError("Published website did not pass its HTTP revision check")


def publish(stand, revision, payload, root=ROOT, health=healthy):
    if stand not in URLS or not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise ValueError("Invalid stand or revision")

    files = unpack(payload)
    files["revision.txt"] = (revision + "\n").encode()
    manifest = {
        name: hashlib.sha256(content).hexdigest() for name, content in files.items()
    }
    directory = root / stand
    releases = directory / "releases"
    releases.mkdir(parents=True, exist_ok=True)
    current = directory / "current"
    if current.exists() and not current.is_symlink():
        raise ValueError("Refusing to replace an unmanaged current directory")

    previous = os.readlink(current) if current.is_symlink() else None
    if previous is not None and not re.fullmatch(r"releases/[0-9a-f]{40}", previous):
        raise ValueError("Refusing an unmanaged current link")

    destination = releases / revision
    if destination.is_symlink():
        raise ValueError("Release directory must not be a symlink")
    if destination.exists():
        if json.loads((destination / ".manifest.json").read_text()) != manifest:
            raise ValueError(
                "This revision was already published with different contents"
            )
    else:
        incoming = Path(tempfile.mkdtemp(prefix=".incoming-", dir=directory))
        try:
            for name, content in files.items():
                path = incoming / name
                path.parent.mkdir(parents=True, exist_ok=True)
                for parent in path.parents:
                    parent.chmod(0o755)
                    if parent == incoming:
                        break
                with path.open("wb") as stream:
                    stream.write(content)
                    stream.flush()
                    os.fsync(stream.fileno())
                path.chmod(0o644)
            (incoming / ".manifest.json").write_text(json.dumps(manifest))
            incoming.chmod(0o755)
            incoming.rename(destination)
        finally:
            if incoming.exists():
                shutil.rmtree(incoming)

    try:
        switch(current, "releases/" + revision)
        health(stand, revision)
    except Exception:
        switch(current, previous)
        raise
    return {"stand": stand, "revision": revision, "url": URLS[stand]}


def main():
    import fcntl

    os.environ["PATH"] = "/usr/sbin:/usr/bin:/sbin:/bin"
    os.umask(0o022)
    if os.geteuid() != 0 or len(sys.argv) != 2 or sys.argv[1] not in URLS:
        raise ValueError("Run through the configured project SSH key")
    stand = sys.argv[1]
    command = re.fullmatch(
        r"deploy ([0-9a-f]{40})", os.environ.get("SSH_ORIGINAL_COMMAND", "")
    )
    if command is None:
        raise ValueError("Only deploy REVISION is accepted")

    payload = sys.stdin.buffer.read(MAX_ARCHIVE + 1)
    with (ROOT / f".{stand}.lock").open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        result = publish(stand, command[1], payload)
    print(json.dumps(result))


if __name__ == "__main__":
    try:
        main()
    except Exception:
        print(
            "anyq website deployment failed; previous stand was preserved",
            file=sys.stderr,
        )
        sys.exit(1)
