"""Exercise native installers against an HTTP fixture and a real anyq binary."""

import argparse
import functools
import hashlib
import os
import platform
import shutil
import subprocess
import tempfile
import threading
import tomllib
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

from check_system_install import (
    check_sudo_denial,
    check_system_install,
    check_windows_worker,
    path_value,
    restore_path,
)
from package_release import asset_name

ROOT = Path(__file__).resolve().parents[1]
VERSION = (
    "v"
    + tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["package"][
        "version"
    ]
)


class ReleaseHandler(SimpleHTTPRequestHandler):
    def do_GET(self):
        if self.path == "/releases/latest":
            self.send_response(302)
            self.send_header("Location", f"/releases/tag/{VERSION}")
            self.end_headers()
        elif self.path == f"/releases/tag/{VERSION}":
            self.send_response(200)
            self.end_headers()
        else:
            super().do_GET()

    def log_message(self, *_args):
        pass


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", required=True)
    parser.add_argument("--archive-dir", type=Path, default=Path("release-dist"))
    parser.add_argument(
        "--system",
        action="store_true",
        help="Exercise the real system directory on disposable CI runners",
    )
    args = parser.parse_args()
    asset = asset_name(args.target)
    archive = args.archive_dir.resolve() / asset
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    windows = platform.system() == "Windows"
    old_user_path = path_value("User") if windows else None

    with tempfile.TemporaryDirectory(prefix="anyq installer ") as directory:
        root = Path(directory)
        releases = root / f"releases/download/{VERSION}"
        releases.mkdir(parents=True)
        served_archive = releases / asset
        shutil.copy2(archive, served_archive)
        checksum_file = releases / "SHA256SUMS"
        valid_checksum = f"{digest}  {asset}\n"
        checksum_file.write_text(valid_checksum)
        handler = functools.partial(ReleaseHandler, directory=root)
        with ThreadingHTTPServer(("127.0.0.1", 0), handler) as server:
            thread = threading.Thread(target=server.serve_forever, daemon=True)
            thread.start()
            base_url = f"http://127.0.0.1:{server.server_port}/releases"
            bin_dir = root / "install O'Brien & spaces"
            destination = bin_dir / ("anyq.exe" if windows else "anyq")

            def install(
                version="latest",
                success=True,
                update_path=False,
                scope="User",
                directory=bin_dir,
                extra_environment=None,
            ):
                if windows:
                    command = [
                        "powershell.exe",
                        "-NoProfile",
                        "-ExecutionPolicy",
                        "Bypass",
                        "-File",
                        str(ROOT / "install/install.ps1"),
                        "-Version",
                        version,
                        "-ReleaseBaseUrl",
                        base_url,
                    ]
                    if scope is not None:
                        command.extend(["-Scope", scope])
                    if directory is not None:
                        command.extend(["-BinDir", str(directory)])
                    if not update_path:
                        command.append("-NoPathUpdate")
                else:
                    command = [
                        "sh",
                        str(ROOT / "install/install.sh"),
                        "--version",
                        version,
                    ]
                    if scope is not None:
                        command.append("--user" if scope == "User" else "--system")
                    if directory is not None:
                        command.extend(["--bin-dir", str(directory)])
                environment = {
                    **os.environ,
                    "HOME": str(root),
                    "USERPROFILE": str(root),
                    "LOCALAPPDATA": str(root / "appdata"),
                    "ANYQ_RELEASE_BASE_URL": base_url,
                    **(extra_environment or {}),
                }

                result = subprocess.run(
                    command, env=environment, capture_output=True, text=True, timeout=60
                )
                assert (result.returncode == 0) == success, (
                    result.stdout + result.stderr
                )
                return result

            try:
                install()
                assert (
                    subprocess.check_output(
                        [destination, "--version"], text=True
                    ).strip()
                    == f"anyq {VERSION[1:]}"
                )
                original = destination.read_bytes()
                smoke_environment = {
                    **os.environ,
                    "HOME": str(root),
                    "USERPROFILE": str(root),
                    "LOCALAPPDATA": str(root / "appdata"),
                }
                for command in (
                    ["topic", "create", "installer-resource"],
                    [
                        "task",
                        "create",
                        "installer-probe",
                        "-t",
                        "installer-resource",
                        "-w",
                    ],
                    ["task", "complete", "installer-probe"],
                ):
                    subprocess.run(
                        [destination, "--db", str(root / "smoke queue.db"), *command],
                        env=smoke_environment,
                        capture_output=True,
                        check=True,
                        timeout=15,
                    )

                install(VERSION)
                assert destination.read_bytes() == original

                for invalid in (f"{'0' * 64}  {asset}\n", "", valid_checksum * 2):
                    checksum_file.write_text(invalid)
                    install(success=False)
                    assert destination.read_bytes() == original

                checksum_file.write_text(valid_checksum)
                served_archive.write_bytes(b"corrupted download")
                install(success=False)
                assert destination.read_bytes() == original

                checksum_file.write_text(
                    f"{hashlib.sha256(served_archive.read_bytes()).hexdigest()}  {asset}\n"
                )
                install(success=False)
                assert destination.read_bytes() == original

                wrong_release = root / "releases/download/v9.9.9"
                wrong_release.mkdir()
                shutil.copy2(archive, wrong_release / asset)
                (wrong_release / "SHA256SUMS").write_text(valid_checksum)
                install("v9.9.9", success=False)
                assert destination.read_bytes() == original

                served_archive.unlink()
                install(success=False)
                assert destination.read_bytes() == original
                install("../../unexpected", success=False)
                assert destination.read_bytes() == original
                assert not list(bin_dir.glob(".anyq*")), "Staged files leaked"

                shutil.copy2(archive, served_archive)
                checksum_file.write_text(valid_checksum)
                install(directory=None)
                user_directory = root / (
                    "appdata/Programs/anyq" if windows else ".local/bin"
                )
                assert (user_directory / destination.name).read_bytes() == original

                if windows:
                    install(update_path=True)
                    install(update_path=True)
                    updated = path_value("User")
                    assert (
                        sum(
                            entry.casefold() == str(bin_dir).casefold()
                            for entry in updated.split(";")
                        )
                        == 1
                    )
                    check_windows_worker(
                        ROOT / "install/install.ps1", destination, root
                    )
                else:
                    check_sudo_denial(
                        install, root, original, checksum_file, valid_checksum
                    )
                if args.system:
                    check_system_install(install, windows, original)
            finally:
                if windows:
                    restore_path("User", old_user_path)
                server.shutdown()
                thread.join()

    print(f"Native installer checks passed: {args.target}")


if __name__ == "__main__":
    main()
