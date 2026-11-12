"""Independent native screenshot states, each owned by repository configuration."""

import json
import tempfile
from contextlib import ExitStack
from pathlib import Path

from driver import Driver
from fixtures import Fixtures, same_source_path, unavailable_observation
from screenshots import Screenshots


def run(cli, panel, output, goldens, configuration, mode):
    screenshots = Screenshots(output, goldens, mode, configuration["comparison"])
    for configured in configuration["visual_cases"]:
        case = configuration["visual_defaults"] | configured
        assert case["repetitions"] == 2, (
            "Every configured state must prove repeatability"
        )
        images = []
        for repetition in range(case["repetitions"]):
            with tempfile.TemporaryDirectory(
                prefix=f"anyq-visual-{case['id']}-"
            ) as temporary:
                home = Path(temporary)
                fixtures = Fixtures(cli, home, configuration)
                observed = fixtures.create(case["fixture"])
                automatic = fixtures.create(
                    case.get("environment_fixture", case["fixture"])
                )
                preferences = {"theme": case["theme"], "language": case["language"]}
                if case["source"] == "override":
                    preferences["database_override"] = str(observed)
                (home / ".anyq/panel.toml").write_text(
                    "\n".join(
                        f"{key} = {json.dumps(value)}"
                        for key, value in preferences.items()
                    )
                    + "\n",
                    encoding="utf-8",
                )
                driver = Driver(
                    panel,
                    home,
                    automatic,
                    configuration["clock_ms"],
                    case["windows_browser_arguments"],
                )
                faults = ExitStack()
                try:
                    driver.start(
                        status="live" if case.get("after_live") else case["status"],
                        viewport=case["viewport"]
                        if case["surface"] == "main"
                        else configuration["visual_defaults"]["viewport"],
                        decorations=case["decorations"],
                    )
                    if case.get("after_live"):
                        assert case["after_live"] == "queue-storage-unavailable"
                        faults.enter_context(unavailable_observation(observed))
                        driver.wait(
                            "return (await window.__TAURI__.core.invoke('get_panel_state')).status==='stale';"
                        )
                    driver.wait(
                        f"return Boolean(document.querySelector('.status.{case['status']}'));"
                    )
                    state = driver.state()
                    assert state["status"] == case["status"]
                    assert (
                        state["snapshot"]
                        == configuration["fixtures"][case["fixture"]][
                            "expected_snapshot"
                        ]
                    ), case["id"]
                    assert state["preferences"]["theme"] == case["theme"]
                    assert state["preferences"]["language"] == case["language"]
                    assert same_source_path(state["path"], observed), (
                        state["path"],
                        str(observed),
                    )
                    if case["status"] != "error":
                        assert state["last_success"] == configuration["clock_ms"]
                    if "error_kind" in case:
                        assert state["error_kind"] == case["error_kind"]
                    driver.wait(
                        f"return document.documentElement.lang === {json.dumps(case['language'])} && document.documentElement.dataset.theme === {json.dumps(case['theme'])};"
                    )
                    if case["topic"] is not None:
                        if case.get("topic_from_compact_settings"):
                            driver.click("#settings")
                            driver.call(window="compact", native="compact")
                            driver.call(
                                f"[...document.querySelectorAll('[data-open-topic]')].find(el=>el.querySelector('.mono').textContent === {json.dumps(case['topic'])}).click();",
                                "compact",
                            )
                            driver.wait(
                                f"return !document.querySelector('#save') && document.querySelector('.content-header h1')?.textContent === {json.dumps(case['topic'])};"
                            )
                        else:
                            driver.call(
                                f"[...document.querySelectorAll('[data-topic]')].find(el=>el.querySelector('.topic-name').textContent === {json.dumps(case['topic'])}).click();"
                            )
                    if case["task"] is not None:
                        driver.click(f'[data-task="{case["task"]}"]')
                    if case["settings"]:
                        driver.click("#settings")
                    if case["surface"] == "compact":
                        driver.call(window="compact", native="compact")
                        driver.resize(*case["viewport"], window="compact")
                        driver.wait(
                            f"return document.documentElement.lang === {json.dumps(case['language'])};",
                            "compact",
                        )
                    images.append(screenshots.capture(driver, case, repetition))
                finally:
                    faults.close()
                    if driver.process and driver.process.poll() is None:
                        driver.stop()
                    if driver.log:
                        driver.log.close()
                    fixtures.stop()
        screenshots.compare(case, *images)
    screenshots.finish()
