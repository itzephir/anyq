"""Join a fixture's real Windows owner before deleting its retained executable."""

import ctypes
import re
import sys
from contextlib import contextmanager
from ctypes import wintypes


@contextmanager
def owner_exit(status):
    handle = None
    if sys.platform == "win32" and (
        match := re.fullmatch(r"daemon is running \(pid (\d+)\)", status)
    ):
        kernel = ctypes.WinDLL("kernel32", use_last_error=True)
        kernel.OpenProcess.restype = wintypes.HANDLE
        handle = kernel.OpenProcess(0x100000, False, int(match[1]))
        if not handle:
            assert ctypes.get_last_error() == 87, ctypes.WinError(
                ctypes.get_last_error()
            )
    try:
        yield
        if handle:
            assert kernel.WaitForSingleObject(wintypes.HANDLE(handle), 10_000) == 0, (
                "Fixture owner process did not exit"
            )
    finally:
        if handle:
            kernel.CloseHandle(wintypes.HANDLE(handle))
