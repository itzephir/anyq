"""Exercise current installers with real published and incomplete archives."""

import hashlib
import shutil
import subprocess
import tarfile
import zipfile

from package_release import asset_name
from published_release import LEGACY_VERSIONS, published_archive


def archive_files(path):
    if path.suffix == ".zip":
        with zipfile.ZipFile(path) as archive:
            return {
                name: archive.read(name)
                for name in archive.namelist()
                if not name.endswith("/")
            }
    with tarfile.open(path, "r:gz") as archive:
        return {
            member.name: archive.extractfile(member).read()
            for member in archive.getmembers()
            if member.isfile()
        }


def without_member(source, destination, excluded):
    if source.suffix == ".zip":
        with (
            zipfile.ZipFile(source) as archive,
            zipfile.ZipFile(destination, "w", zipfile.ZIP_DEFLATED) as output,
        ):
            for member in archive.infolist():
                if member.filename != excluded:
                    output.writestr(member, archive.read(member))
    else:
        with (
            tarfile.open(source, "r:gz") as archive,
            tarfile.open(destination, "w:gz") as output,
        ):
            for member in archive.getmembers():
                if member.name != excluded:
                    output.addfile(
                        member, archive.extractfile(member) if member.isfile() else None
                    )


def publish(root, version, archive):
    directory = root / f"releases/download/{version}"
    directory.mkdir(parents=True, exist_ok=True)
    destination = directory / archive.name
    shutil.copy2(archive, destination)
    checksum = hashlib.sha256(destination.read_bytes()).hexdigest()
    (directory / "SHA256SUMS").write_text(
        f"{checksum}  {destination.name}\n", encoding="ascii"
    )


def check_published_installation(
    install, server, root, current, version, target, published
):
    suffix = ".exe" if "windows" in target else ""
    executable = f"anyq{suffix}"
    plugins = [f"plugins/{name}{suffix}" for name in ("shell", "timeout")]
    modern_files = archive_files(current)
    preserved = {
        path: path.read_bytes()
        for path in (root / ".anyq/plugins").iterdir()
        if path.is_file()
    }
    database = root / "smoke queue.db"
    committed = database.read_bytes()

    try:
        for legacy_version in LEGACY_VERSIONS:
            legacy = published_archive(target, legacy_version, published)
            legacy_files = archive_files(legacy)
            assert set(legacy_files) == {executable}, legacy_files.keys()
            publish(root, legacy_version, legacy)
            server.release_version = legacy_version
            destination = root / f"published {legacy_version}"

            for selected in ("latest", legacy_version):
                install(selected, directory=destination)
                assert (destination / executable).read_bytes() == legacy_files[
                    executable
                ]
                reported = subprocess.check_output(
                    [str(destination / executable), "--version"], text=True
                ).strip()
                assert reported == f"anyq {legacy_version[1:]}", reported
                assert not (destination / "plugins").exists()

            for name in plugins:
                path = destination / name
                path.parent.mkdir(exist_ok=True)
                path.write_bytes(b"adjacent-user-payload")
            install(legacy_version, directory=destination)
            for name in plugins:
                assert (destination / name).read_bytes() == b"adjacent-user-payload"

            publish(root, version, current)
            install(version, directory=destination)
            for name, contents in modern_files.items():
                assert (destination / name).read_bytes() == contents

        for name in plugins:
            damaged = root / "partial" / asset_name(target)
            damaged.parent.mkdir(exist_ok=True)
            without_member(current, damaged, name)
            publish(root, version, damaged)
            failure = install(version, success=False)
            assert "incomplete plugin payload" in failure.stderr, failure.stderr

        publish(root, "v9.9.9", legacy)
        failure = install("v9.9.9", success=False)
        assert "missing required plugin executables" in failure.stderr, failure.stderr

        assert database.read_bytes() == committed
        assert all(
            path.read_bytes() == contents for path, contents in preserved.items()
        )
    finally:
        publish(root, version, current)
        server.release_version = version
