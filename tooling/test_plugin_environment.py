"""Verify plugin invocation preserves an isolated Python environment across restart."""

import argparse
import json
import os
import subprocess
import sys
import tempfile
import time
import unittest
import venv
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "examples"))
from install_observer import install

OBSERVER = """import json, os, pathlib, sys
import anyq_environment_probe

marker = pathlib.Path(sys.argv[1])
records = json.loads(marker.read_text()) if marker.exists() else []
records.append({'prefix': sys.prefix, 'dependency': anyq_environment_probe.VALUE})
replacement = marker.with_suffix('.next')
replacement.write_text(json.dumps(records))
os.replace(replacement, marker)
hello = json.loads(sys.stdin.readline())
print(json.dumps({'type': 'hello', 'version': hello['version'],
                  'subscriptions': [], 'capabilities': []}), flush=True)
for line in sys.stdin:
    if json.loads(line)['type'] == 'stop':
        break
"""


def wait(condition):
    deadline = time.monotonic() + 15
    while not condition():
        assert time.monotonic() < deadline, "Plugin environment condition timed out"
        time.sleep(0.02)


def check_environment(cli, selection):
    with tempfile.TemporaryDirectory(prefix="anyq-plugin-environment-") as temporary:
        root = Path(temporary).resolve()
        environment = root / "isolated environment"
        # Unix venvs normally use interpreter links; Windows uses its native launcher.
        venv.EnvBuilder(with_pip=False, symlinks=os.name != "nt").create(environment)
        directory = environment / ("Scripts" if os.name == "nt" else "bin")
        interpreter = directory / ("python.exe" if os.name == "nt" else "python")
        if os.name != "nt":
            assert interpreter.is_symlink(), (
                "The fixture must exercise a linked interpreter"
            )
        library = subprocess.check_output(
            [
                str(interpreter),
                "-c",
                "import sysconfig; print(sysconfig.get_path('purelib'))",
            ],
            text=True,
            timeout=15,
        ).strip()
        Path(library, "anyq_environment_probe.py").write_text(
            "VALUE = 'only-in-the-selected-environment'\n", encoding="utf-8"
        )
        observer = root / "observer.py"
        observer.write_text(OBSERVER, encoding="utf-8")
        marker = root / "starts.json"
        child_environment = dict(os.environ, HOME=str(root), USERPROFILE=str(root))
        child_environment["ANYQ_DB"] = str(root / "queue.db")
        executable = str(interpreter)
        if selection == "path":
            child_environment["PATH"] = os.pathsep.join(
                [str(directory), child_environment.get("PATH", "")]
            )
            executable = interpreter.name

        def call(*arguments):
            completed = subprocess.run(
                [str(cli), *arguments],
                cwd=root,
                env=child_environment,
                text=True,
                capture_output=True,
                timeout=20,
            )
            assert completed.returncode == 0, (arguments, completed.stderr)
            return completed.stdout.strip()

        def status():
            return json.loads(call("plugin", "status", "observer"))

        def started(count):
            current = status()
            assert current["failures"] == 0, current
            return (
                current["running"]
                and marker.exists()
                and len(json.loads(marker.read_text())) == count
            )

        install(root, executable, observer, marker)
        try:
            call("daemon", "start")
            call("plugin", "enable", "observer")
            wait(lambda: started(1))
            call("daemon", "stop")
            call("daemon", "start")
            wait(lambda: started(2))
            # The same selected invocation remains compatible with durable registration.
            call("plugin", "enable", "observer")

            records = json.loads(marker.read_text())
            assert len(records) == 2, records
            for record in records:
                assert Path(record["prefix"]).resolve() == environment, record
                assert record["dependency"] == "only-in-the-selected-environment", (
                    record
                )
        finally:
            call("daemon", "stop")


class PluginEnvironmentTests(unittest.TestCase):
    cli = None

    def test_plugin_owned_explicit_interpreter_preserves_environment_after_restart(
        self,
    ):
        check_environment(self.cli, "explicit")

    def test_path_selected_interpreter_preserves_environment_after_restart(self):
        check_environment(self.cli, "path")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--cli", required=True, type=Path)
    arguments = parser.parse_args()
    PluginEnvironmentTests.cli = arguments.cli.resolve(strict=True)
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(PluginEnvironmentTests)
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    if not result.wasSuccessful():
        sys.exit(1)


if __name__ == "__main__":
    main()
