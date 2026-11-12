"""Recover durable plugin work when the native panel starts the owner first."""

import json
import sqlite3
from contextlib import closing
from pathlib import Path

from driver import Driver
from fixtures import Fixtures
from screenshots import platform_name


def run(cli, panel, home, output, configuration):
    home = Path(home)
    setup = configuration["plugin_recovery"]
    fixtures = Fixtures(cli, home, configuration)
    clock = home / "plugin-clock"
    clock.write_text(str(setup["initial_clock_ms"]), encoding="utf-8")
    fixtures.environment["ANYQ_TEST_CLOCK_FILE"] = str(clock)
    driver = None
    try:
        database = fixtures.create(setup["fixture"])
        metadata = json.dumps({"payload": "x" * setup["metadata_bytes_per_task"]})
        with closing(sqlite3.connect(database)) as connection, connection:
            for index in range(setup["metadata_tasks"]):
                task = connection.execute(
                    "INSERT INTO tasks(name,state,stopped,metadata) VALUES (?,'queued',1,?)",
                    (f"large-metadata-{index:02}", metadata),
                )
                connection.execute(
                    "INSERT INTO task_topics(task_id,topic_name) VALUES (?,'cpu')",
                    (task.lastrowid,),
                )
            initial_events = connection.execute(
                "SELECT count(*) FROM queue_events"
            ).fetchone()[0]
        clock.write_text(str(setup["overdue_clock_ms"]), encoding="utf-8")
        (home / ".anyq/panel.toml").write_text(
            'theme = "dark"\nlanguage = "en"\n', encoding="utf-8"
        )
        driver = Driver(panel, home, database, configuration["clock_ms"])
        driver.environment["ANYQ_TEST_CLOCK_FILE"] = str(clock)

        for _ in range(setup["panel_first_starts"]):
            driver.start(viewport=configuration["visual_defaults"]["viewport"])
            driver.wait(
                "const state=await window.__TAURI__.core.invoke('get_panel_state');"
                f"return state.status==='live' && state.snapshot.tasks.some(task=>task.name==={json.dumps(setup['successor'])} && task.state==='running')"
                f" && !state.snapshot.tasks.some(task=>task.name==={json.dumps(setup['expired_task'])});"
            )
            snapshot = driver.state()["snapshot"]
            assert snapshot["running"] == 1
            assert snapshot["queued"] == setup["metadata_tasks"]
            assert all(
                "metadata" not in record
                for record in snapshot["tasks"] + snapshot["topics"]
            )
            assert len(json.dumps(snapshot).encode()) < 16 * 1024
            with closing(sqlite3.connect(database)) as connection:
                assert connection.execute("PRAGMA user_version").fetchone()[0] == 6
                assert (
                    connection.execute(
                        "SELECT count(*) FROM tasks WHERE metadata=?", (metadata,)
                    ).fetchone()[0]
                    == setup["metadata_tasks"]
                )
                assert (
                    connection.execute(
                        "SELECT count(*) FROM queue_events WHERE json_extract(payload,'$.kind')='task.completed' AND json_extract(payload,'$.task.name')=?",
                        (setup["expired_task"],),
                    ).fetchone()[0]
                    == 1
                )
                assert (
                    connection.execute("SELECT count(*) FROM plugin_timers").fetchone()[
                        0
                    ]
                    == 0
                )
                assert (
                    connection.execute("SELECT count(*) FROM queue_events").fetchone()[
                        0
                    ]
                    == initial_events + 2
                )
            driver.stop()
            assert "running" in fixtures.call(["daemon", "status"], database)
            fixtures.call(["daemon", "stop"], database)

        destination = Path(output) / platform_name() / "plugin-integration.json"
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(
            json.dumps(
                {
                    "result": "passed",
                    "scenarios": [
                        "panel-first-plugin-recovery",
                        "exactly-once-expiry-after-restart",
                        "no-historical-running-events",
                        "large-metadata-excluded",
                        "owner-survives-panel-quit",
                    ],
                },
                indent=2,
            )
            + "\n",
            encoding="utf-8",
        )
    finally:
        if driver and driver.process and driver.process.poll() is None:
            driver.process.terminate()
            driver.process.wait(timeout=10)
        if driver and driver.log:
            driver.log.close()
        fixtures.stop()
