"""Native application scenarios backed by real CLI/daemon/SQLite processes."""

import json
import os
import sqlite3
import time
from contextlib import closing
from pathlib import Path

from driver import Driver
from fixtures import Fixtures
from screenshots import platform_name


def run(cli, panel, home, output, configuration):
    home = Path(home)
    fixtures = Fixtures(cli, home, configuration)
    database = fixtures.create("primary")
    alternate = fixtures.create("alternate")
    empty = fixtures.create("empty")
    missing = fixtures.create("missing")
    incompatible = fixtures.create("incompatible")

    def call(*arguments, db=database):
        return fixtures.call(list(arguments), db)

    def fingerprint(db):
        with closing(sqlite3.connect(db)) as connection:
            return {
                table: connection.execute(
                    f"SELECT * FROM {table} ORDER BY 1"
                ).fetchall()
                for table in ("topics", "tasks", "task_topics")
            }

    hostile = configuration["fixtures"]["primary"]["commands"][5][2][1:-1]
    (home / ".anyq/panel.toml").write_text(
        'theme = "dark"\nlanguage = "en"\n', encoding="utf-8"
    )
    original = fingerprint(database)
    driver = Driver(panel, home, database, configuration["clock_ms"])
    platform = platform_name()
    (Path(output) / platform).mkdir(parents=True, exist_ok=True)

    def ui(script):
        return driver.call(script)["payload"].get("value")

    def settings():
        driver.click("#settings")
        driver.wait("return Boolean(document.querySelector('#save'));")

    def save():
        driver.click("#save")
        driver.wait("return !document.querySelector('#save');")

    def source(path, status="live", language=None, theme=None):
        settings()
        automatic = ui("return document.querySelector('#automatic').checked;")
        if (path is None) != automatic:
            driver.click("#automatic")
        if path is not None:
            driver.fill("#database-path", str(path))
        if language:
            driver.select("#language", language)
        if theme:
            driver.select("#theme", theme)
        save()
        driver.wait(
            f"return (await window.__TAURI__.core.invoke('get_panel_state')).status === {json.dumps(status)};"
        )

    try:
        driver.start()
        state = driver.state()
        original_frame = state
        assert state["source"] == "environment"
        assert state["snapshot"]["running"] == 2
        assert state["snapshot"]["queued"] == 3
        assert len(state["snapshot"]["topics"]) == 6
        assert ui(
            f"return document.body.innerText.includes({json.dumps(hostile)}) && !window.injected;"
        )
        assert ui(
            "return document.querySelector('.topic-name').textContent.includes('<img');"
        )
        # Select a topic by canonical text instead of depending on sorted indexes.
        ui(
            "[...document.querySelectorAll('[data-topic]')].find(el=>el.querySelector('.topic-name').textContent==='cpu').click();"
        )
        driver.click('[data-task="1"]')
        driver.wait(
            "return JSON.stringify([...document.querySelectorAll('.resource-tags > span')].map(el=>el.textContent)) === '[\"cpu\",\"repo\"]';"
        )
        driver.fill("#task-search", "release")
        assert ui("return document.querySelectorAll('[data-task]').length;") == 1
        driver.fill("#task-search", "")
        assert ui("return document.querySelectorAll('[data-task]').length;") == 3
        driver.call(window="compact", native="compact")
        driver.wait(
            "return document.querySelector('.compact-counts')?.innerText.includes('running');",
            "compact",
        )
        compact = driver.call(
            "return await window.__TAURI__.core.invoke('get_panel_state');", "compact"
        )["payload"]["value"]
        assert compact["snapshot"] == driver.state()["snapshot"], (
            "Both surfaces must consume one accepted snapshot"
        )
        assert driver.call(
            "return [...document.querySelectorAll('.compact-counts strong')].map(el=>Number(el.textContent));",
            "compact",
        )["payload"]["value"] == [2, 3]
        driver.click("#open-main", "compact")
        source(None, language="ru", theme="light")
        driver.wait(
            "return document.documentElement.lang==='ru' && document.documentElement.dataset.theme==='light';"
        )
        settings()
        driver.click("#cancel")
        source(empty)
        assert driver.state()["snapshot"]["topics"] == []
        source(missing, "error")
        assert driver.state()["snapshot"] is None
        assert driver.state()["error_kind"] == "missing"
        assert not missing.exists()
        source(incompatible, "error")
        assert driver.state()["error_kind"] == "incompatible"
        assert incompatible.read_bytes() == b"not an anyq SQLite database\n"
        source(None, language="en", theme="dark")
        assert fingerprint(database) == original, (
            "Read-only UI actions changed queue state"
        )
        # Observe a genuine source failure after a successful snapshot.
        paused = home / "paused.db"
        deadline = time.monotonic() + 10
        while True:
            try:
                database.rename(paused)
                break
            except PermissionError:
                assert time.monotonic() < deadline
                time.sleep(0.05)
        try:
            driver.wait(
                "return (await window.__TAURI__.core.invoke('get_panel_state')).status==='stale';"
            )
            assert driver.state()["snapshot"]["running"] == 2
        finally:
            paused.rename(database)
        driver.wait(
            "return (await window.__TAURI__.core.invoke('get_panel_state')).status==='live';"
        )
        owner = call("daemon", "status")
        call("daemon", "stop")
        driver.click("#refresh")
        deadline = time.monotonic() + 15
        while (
            restarted := call("daemon", "status")
        ) == owner or "stopped" in restarted:
            assert time.monotonic() < deadline, (
                "Panel did not restart its bundled owner"
            )
            time.sleep(0.05)
        driver.wait(
            "return (await window.__TAURI__.core.invoke('get_panel_state')).status==='live';"
        )
        driver.resize(360, 650)
        driver.resize(1000, 700)
        ui(
            "[...document.querySelectorAll('[data-topic]')].find(el=>el.querySelector('.topic-name').textContent==='cpu').click();"
        )
        driver.click('[data-task="1"]')
        call("task", "complete", "build-ui")
        driver.wait(
            "return !(await window.__TAURI__.core.invoke('get_panel_state')).snapshot.tasks.some(task=>task.sequence===1);"
        )
        driver.wait(
            "return !document.querySelector('[data-task=\"1\"]') && !document.querySelector('.detail');"
        )
        after_completion = fingerprint(database)
        with closing(sqlite3.connect(database)) as connection, connection:
            connection.executemany(
                "INSERT INTO topics(name,capacity) VALUES (?,1)",
                [(f"limit-{index}",) for index in range(10_001)],
            )
        try:
            driver.click("#refresh")
            driver.wait(
                "return (await window.__TAURI__.core.invoke('get_panel_state')).error_kind==='limit' && (await window.__TAURI__.core.invoke('get_panel_state')).status==='stale';"
            )
            assert len(driver.state()["snapshot"]["topics"]) == 6
        finally:
            with closing(sqlite3.connect(database)) as connection, connection:
                connection.execute("DELETE FROM topics WHERE name LIKE 'limit-%'")
        driver.wait(
            "return (await window.__TAURI__.core.invoke('get_panel_state')).status==='live';"
        )
        assert fingerprint(database) == after_completion
        source(alternate)
        assert driver.state()["snapshot"]["running"] == 0
        assert driver.state()["snapshot"]["queued"] == 1
        source_path = driver.state()["path"]
        driver.call(native="publish-state", payload=original_frame)
        assert (
            ui("return document.querySelector('.source-path').textContent;")
            == source_path
        )
        assert (
            ui(
                "return document.querySelector('.stat strong').textContent.replace(/\\s/g,'');"
            )
            == "0/2"
        )

        alternate_before = fingerprint(alternate)
        driver.stop()
        assert "running" in call("daemon", "status", db=alternate)
        driver.start()
        assert driver.state()["path"] == source_path
        assert driver.state()["source"] == "override"
        assert fingerprint(alternate) == alternate_before
        source(None)
        assert "database_override" not in (home / ".anyq/panel.toml").read_text()
        assert driver.state()["source"] == "environment"
        assert fingerprint(database) == after_completion
        tray = driver.call()["tray"]
        driver.call(native="close-main")
        if tray:
            assert driver.process.poll() is None
            driver.call(native="open-main")
            driver.stop()
        else:
            assert os.name != "nt" and os.uname().sysname == "Linux", (
                "A native tray should be available"
            )
            driver.process.wait(timeout=10)
        assert "running" in call("daemon", "status")
        assert fingerprint(database) == after_completion
        (Path(output) / platform / "integration.json").write_text(
            json.dumps(
                {
                    "result": "passed",
                    "native_tray": tray,
                    "scenarios": [
                        "current-snapshot",
                        "text-safety",
                        "details",
                        "search",
                        "shared-surfaces",
                        "localization",
                        "themes",
                        "missing-no-create",
                        "incompatible-no-rewrite",
                        "stale-recovery",
                        "owner-restart",
                        "narrow-layout",
                        "concurrent-completion",
                        "oversized-response",
                        "source-switch",
                        "late-native-frame-rejected",
                        "preference-restart-reset",
                        "close-reopen-quit",
                        "ownership-preserved",
                    ],
                },
                indent=2,
            )
            + "\n"
        )
    finally:
        if driver.process and driver.process.poll() is None:
            driver.process.terminate()
            driver.process.wait(timeout=10)
        if driver.log:
            driver.log.close()
        Path(output).mkdir(parents=True, exist_ok=True)
        (Path(output) / "panel.log").write_bytes(
            (home / "panel.log").read_bytes() if (home / "panel.log").exists() else b""
        )
        fixtures.stop()
