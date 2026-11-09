"""Exercise the project ingress and mounted releases with real nginx containers."""

import re
import subprocess
import tempfile
import time
from pathlib import Path

from package_website import main as package
from receive_website import publish


def docker(*args, check=True):
    return subprocess.run(
        ["docker", *args], check=check, capture_output=True, text=True
    )


def main():
    names = ["anyq-site-dev", "anyq-site-prod", "anyq-ingress-ci"]
    for name in names:
        if docker("container", "inspect", name, check=False).returncode == 0:
            raise RuntimeError("Refusing to replace an existing hosting test container")
    network = "anyq-hosting-ci"
    if docker("network", "inspect", network, check=False).returncode == 0:
        raise RuntimeError("Refusing to replace an existing hosting test network")
    package()
    payload = Path("website/site.zip").read_bytes()
    docker(
        "build",
        "-q",
        "-f",
        "website/ingress.Dockerfile",
        "-t",
        "anyq-ingress:test",
        ".",
    )
    created = []
    docker("network", "create", network)
    try:
        with tempfile.TemporaryDirectory(prefix="anyq-hosting-") as directory:
            root = Path(directory).resolve()
            for stand in ("dev", "prod"):
                (root / stand).mkdir()
                docker(
                    "run",
                    "-d",
                    "--name",
                    f"anyq-site-{stand}",
                    "--network",
                    network,
                    "--mount",
                    f"type=bind,source={root / stand},target=/srv/site,readonly",
                    "anyq-site:test",
                )
                created.append(f"anyq-site-{stand}")
            docker(
                "run",
                "-d",
                "--name",
                "anyq-ingress-ci",
                "--network",
                network,
                "anyq-ingress:test",
            )
            created.append("anyq-ingress-ci")

            def request(host, path="/"):
                result = docker(
                    "exec",
                    "anyq-ingress-ci",
                    "wget",
                    "-S",
                    "-O",
                    "-",
                    "--header",
                    "Host: " + host,
                    "http://127.0.0.1:8080" + path,
                    check=False,
                )
                statuses = re.findall(r"HTTP/1\.1 (\d+)", result.stderr)
                return int(statuses[-1]) if statuses else 0, result.stdout

            for _attempt in range(40):
                if request("dev.anyq.itzephir.com")[0] == 503:
                    break
                time.sleep(0.25)
            assert request("anyq.itzephir.com")[0] == 503
            publish("dev", "a" * 40, payload, root)
            assert request("dev.anyq.itzephir.com")[0] == 200
            assert request("dev.anyq.itzephir.com", "/revision.txt?q=preserved") == (
                200,
                "a" * 40 + "\n",
            )
            assert request("anyq.itzephir.com")[0] == 503, (
                "Dev deployment touched production"
            )
            assert request("unknown.anyq.itzephir.com")[0] == 404
            assert request("dev.anyq.itzephir.com", "/.manifest.json")[0] == 404
            publish("prod", "b" * 40, payload, root)
            assert request("anyq.itzephir.com", "/revision.txt") == (
                200,
                "b" * 40 + "\n",
            )
            assert request("dev.anyq.itzephir.com", "/revision.txt") == (
                200,
                "a" * 40 + "\n",
            )
    finally:
        for name in reversed(created):
            docker("rm", "-f", name, check=False)
        docker("network", "rm", network, check=False)
    print("Project ingress, dev/prod isolation and mounted release publication passed")


if __name__ == "__main__":
    main()
