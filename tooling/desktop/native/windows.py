"""Native Win32 input, windows and display inspection in an isolated CI desktop."""

import ctypes as c
import json
import subprocess
import sys
from ctypes import wintypes as w

user = c.windll.user32
user.SetProcessDpiAwarenessContext(c.c_void_p(-4))


class MonitorInfo(c.Structure):
    _fields_ = [
        ("size", w.DWORD),
        ("bounds", w.RECT),
        ("work", w.RECT),
        ("flags", w.DWORD),
        ("device", w.WCHAR * 32),
    ]


def rect(bounds):
    return dict(
        x=bounds.left,
        y=bounds.top,
        width=bounds.right - bounds.left,
        height=bounds.bottom - bounds.top,
    )


def monitors():
    result = []

    @c.WINFUNCTYPE(w.BOOL, w.HMONITOR, w.HDC, c.POINTER(w.RECT), w.LPARAM)
    def visit(handle, _dc, _bounds, _data):
        info = MonitorInfo()
        info.size = c.sizeof(info)
        assert user.GetMonitorInfoW(w.HMONITOR(handle), c.byref(info))
        x, y = w.UINT(), w.UINT()
        assert (
            c.windll.shcore.GetDpiForMonitor(
                w.HMONITOR(handle), 0, c.byref(x), c.byref(y)
            )
            == 0
        )
        result.append(
            rect(info.work)
            | {
                "scale": x.value / 96,
                "device": info.device,
                "bounds": rect(info.bounds),
            }
        )
        return True

    assert user.EnumDisplayMonitors(None, None, visit, 0)
    return result


def windows(pid):
    result = []

    @c.WINFUNCTYPE(w.BOOL, w.HWND, w.LPARAM)
    def visit(handle, _data):
        owner = w.DWORD()
        user.GetWindowThreadProcessId(w.HWND(handle), c.byref(owner))
        if owner.value == pid and user.IsWindowVisible(w.HWND(handle)):
            bounds = w.RECT()
            assert user.GetWindowRect(w.HWND(handle), c.byref(bounds))
            name = c.create_unicode_buffer(256)
            user.GetClassNameW(w.HWND(handle), name, len(name))
            result.append(
                rect(bounds) | {"handle": handle, "layer": 0, "class": name.value}
            )
        return True

    assert user.EnumWindows(visit, 0)
    return result


def tray():
    script = """
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$root = [System.Windows.Automation.AutomationElement]::RootElement
$scope = [System.Windows.Automation.TreeScope]::Descendants
$property = [System.Windows.Automation.AutomationElement]::NameProperty
$buttons = [System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Button)
function Is-NotificationArea($element) {
    $walker = [System.Windows.Automation.TreeWalker]::ControlViewWalker
    for ($depth = 0; $element -and $depth -lt 10; $depth++) {
        $class = $element.Current.ClassName
        if ($class -match 'TaskList') { return $false }
        if ($class -match 'SystemTray|TrayNotifyWnd|NotifyIconOverflowWindow' -or $element.Current.Name -match 'Notification Area') { return $true }
        $element = $walker.GetParent($element)
    }
    return $false
}
function Find-TrayButton {
    foreach ($candidate in $root.FindAll($scope, $buttons)) {
        if ($candidate.Current.Name -match '(?i)anyq' -and !$candidate.Current.IsOffscreen -and (Is-NotificationArea $candidate)) { return $candidate }
    }
    return $null
}
$item = Find-TrayButton
if (!$item) {
    $chevron = $root.FindAll($scope, $buttons) | Where-Object { $_.Current.Name -eq 'Show Hidden Icons' -and !$_.Current.IsOffscreen } | Select-Object -First 1
    if ($chevron) {
        $chevron.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
        $deadline = [DateTime]::UtcNow.AddSeconds(5)
        do { $item = Find-TrayButton } while (!$item -and [DateTime]::UtcNow -lt $deadline)
    }
}
if ($item) {
    $r = $item.Current.BoundingRectangle
    @{x=$r.X;y=$r.Y;width=$r.Width;height=$r.Height} | ConvertTo-Json -Compress
} else {
    [Console]::Error.WriteLine('Tray button names: ' + (($root.FindAll($scope, $buttons) | ForEach-Object {$_.Current.Name}) -join ', '))
    'null'
}
"""
    output = subprocess.check_output(
        ["powershell", "-NoProfile", "-Command", script], text=True, timeout=15
    )
    return json.loads(output)


def perform(command):
    action = command["action"]
    if action == "monitors":
        return monitors()
    if action == "configure-displays":
        from windows_monitors import configure

        return configure()
    if action == "windows":
        return windows(command["pid"])
    if action == "menu-open":
        return any(window["class"] == "#32768" for window in windows(command["pid"]))
    if action == "tray":
        return tray()
    if action == "click":
        assert user.SetCursorPos(round(command["x"]), round(command["y"]))
        down, up = (8, 16) if command.get("right") else (2, 4)
        user.mouse_event(down, 0, 0, 0, 0)
        user.mouse_event(up, 0, 0, 0, 0)
    elif action == "key":
        modifiers = command.get("modifiers", [])
        for code in modifiers:
            user.keybd_event(code, 0, 0, 0)
        user.keybd_event(command["code"], 0, 0, 0)
        user.keybd_event(command["code"], 0, 2, 0)
        for code in reversed(modifiers):
            user.keybd_event(code, 0, 2, 0)
    elif action == "close":
        target = max(windows(command["pid"]), key=lambda item: item["height"])
        user.PostMessageW(w.HWND(target["handle"]), 0x0010, 0, 0)
    else:
        raise AssertionError(f"Unknown native action: {action}")
    return True


for line in sys.stdin:
    try:
        reply = {"ok": True, "value": perform(json.loads(line))}
    except Exception as error:
        reply = {"ok": False, "error": str(error)}
    print(json.dumps(reply), flush=True)
