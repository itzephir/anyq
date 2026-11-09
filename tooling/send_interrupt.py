"""Send an actual Ctrl+C to one isolated CLI process used by integration tests."""

import os
import signal
import sys


def main():
    pid = int(sys.argv[1])

    if pid <= 0:
        raise ValueError("an owned positive process id is required")

    if os.name != "nt":
        os.kill(pid, signal.SIGINT)
        return

    import ctypes
    from ctypes import wintypes

    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    kernel.OpenProcess.restype = wintypes.HANDLE
    kernel.WaitForSingleObject.argtypes = [wintypes.HANDLE, wintypes.DWORD]
    kernel.WaitForSingleObject.restype = wintypes.DWORD
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    kernel.FreeConsole()
    handle = kernel.OpenProcess(0x00100000, False, pid)

    if not handle:
        raise ctypes.WinError(ctypes.get_last_error())

    try:
        if not kernel.AttachConsole(pid):
            raise ctypes.WinError(ctypes.get_last_error())

        # CTRL_C_EVENT targets the attached console, so the CLI owns a new console.
        if not kernel.SetConsoleCtrlHandler(None, True):
            raise ctypes.WinError(ctypes.get_last_error())

        if not kernel.GenerateConsoleCtrlEvent(0, 0):
            raise ctypes.WinError(ctypes.get_last_error())

        if kernel.WaitForSingleObject(handle, 15000) != 0:
            raise RuntimeError("CLI did not exit after CTRL_C_EVENT")
    finally:
        kernel.FreeConsole()
        kernel.CloseHandle(handle)


if __name__ == "__main__":
    main()
