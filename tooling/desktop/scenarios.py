"""Native application scenarios backed by real CLI/daemon/SQLite processes."""

import json
import os
import sqlite3
import subprocess
import time
from pathlib import Path

from driver import Driver
from screenshots import Screenshots


def run(cli, panel, home, output, goldens, update=False):
    home = Path(home)
    home.mkdir(parents=True, exist_ok=True)
    database = home / "queue.db"
    alternate = home / "other.db"
    empty = home / "empty.db"
    missing = home / "missing.db"
    incompatible = home / "incompatible.db"
    environment = os.environ | {
        "HOME": str(home),
        "USERPROFILE": str(home),
        "ANYQ_DB": str(database),
    }

    def call(*arguments, db=database):
        result = subprocess.run(
            [str(cli), "--db", str(db), *arguments],
            env=environment,
            text=True,
            capture_output=True,
            timeout=20,
        )
        assert result.returncode == 0, (arguments, result.stderr)
        return result.stdout.strip()

    def fingerprint(db):
        with sqlite3.connect(db) as connection:
            return {
                table: connection.execute(
                    f"SELECT * FROM {table} ORDER BY 1"
                ).fetchall()
                for table in ("topics", "tasks", "task_topics")
            }

    call("config", "task.autocreate", "true")
    call("config", "topic.autocreate", "true")
    for name, capacity in [
        ("cpu", "4"),
        ("repo", "1"),
        ("emulator", "1"),
        ("empty", "2"),
        ("`данные`", "1"),
    ]:
        call("topic", "create", name, "-c", capacity)
    hostile = "</button><img src=x onerror='window.injected=1'>"
    call("topic", "create", f"`{hostile}`")
    call("task", "create", "build-ui", "-t", "cpu", "-t", "repo")
    call("task", "create", "lint-tests", "-t", "cpu")
    call("task", "create", "release-preview", "-t", "cpu", "-t", "repo")
    call("task", "create", "manual-check", "-t", "emulator", "--stop")
    call("task", "create", "``", "-t", "emulator", "--stop")
    call("topic", "create", "cpu", "-c", "2", db=alternate)
    call("topic", "create", "repo", "-c", "3", db=alternate)
    call("task", "create", "build-ui", "-t", "cpu", "--stop", db=alternate)
    call("daemon", "start", db=empty)
    incompatible.write_bytes(b"not an anyq SQLite database\n")
    (home / ".anyq/panel.toml").write_text('theme = "dark"\nlanguage = "en"\n')
    original = fingerprint(database)
    driver = Driver(panel, home, database)
    screenshots = Screenshots(output, goldens, update)

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
        screenshots.capture(driver, "main-dark-en")
        driver.click('[data-task="1"]')
        driver.wait(
            "return JSON.stringify([...document.querySelectorAll('.resource-tags > span')].map(el=>el.textContent)) === '[\"cpu\",\"repo\"]';"
        )
        screenshots.capture(driver, "details-dark-en")
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
        screenshots.capture(driver, "compact-dark-en", "compact")
        driver.click("#open-main", "compact")
        source(None, language="ru", theme="light")
        driver.wait(
            "return document.documentElement.lang==='ru' && document.documentElement.dataset.theme==='light';"
        )
        screenshots.capture(driver, "main-light-ru")
        settings()
        screenshots.capture(driver, "settings-light-ru")
        driver.click("#cancel")
        source(empty)
        assert driver.state()["snapshot"]["topics"] == []
        screenshots.capture(driver, "empty-light-ru")
        source(missing, "error")
        assert driver.state()["snapshot"] is None
        assert driver.state()["error_kind"] == "missing"
        assert not missing.exists()
        screenshots.capture(driver, "missing-light-ru")
        source(incompatible, "error")
        assert driver.state()["error_kind"] == "incompatible"
        assert incompatible.read_bytes() == b"not an anyq SQLite database\n"
        screenshots.capture(driver, "incompatible-light-ru")
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
            screenshots.capture(driver, "stale-dark-en")
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
        driver.call(native="resize", width=360, height=650)
        screenshots.capture(driver, "narrow-dark-en")
        driver.call(native="resize", width=1000, height=700)
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
        screenshots.capture(driver, "completion-dark-en")
        after_completion = fingerprint(database)
        with sqlite3.connect(database) as connection:
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
            with sqlite3.connect(database) as connection:
                connection.execute("DELETE FROM topics WHERE name LIKE 'limit-%'")
        driver.wait(
            "return (await window.__TAURI__.core.invoke('get_panel_state')).status==='live';"
        )
        assert fingerprint(database) == after_completion
        source(alternate)
        assert driver.state()["snapshot"]["running"] == 0
        assert driver.state()["snapshot"]["queued"] == 1
        screenshots.capture(driver, "source-switch-dark-en")
        source_path = driver.state()["path"]
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
        (Path(output) / screenshots.platform / "integration.json").write_text(
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
                        "preference-restart-reset",
                        "close-reopen-quit",
                        "ownership-preserved",
                    ],
                },
                indent=2,
            )
            + "\n"
        )
        screenshots.finish()
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
        for db in (database, alternate, empty):
            call("daemon", "stop", db=db)
