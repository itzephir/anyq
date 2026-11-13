"""Verify queue data copied from an installed application's real native document."""

import re
import time


def verify(text, expected):
    normalized = re.sub(r"\s+", " ", text).strip()
    comparable = normalized.casefold()
    missing = [
        value for value in expected["required"] if value.casefold() not in comparable
    ]
    unexpected = [
        value for value in expected.get("absent", []) if value.casefold() in comparable
    ]
    assert not missing and not unexpected, {
        "missing": missing,
        "unexpected": unexpected,
        "document": normalized,
    }
    return normalized


def wait(native, pid, expected):
    deadline = time.monotonic() + 30
    while True:
        text = native.document_text(pid)
        try:
            return verify(text, expected)
        except AssertionError:
            if time.monotonic() >= deadline:
                raise
        time.sleep(0.05)
