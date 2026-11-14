"""Native Win32 input, windows and display inspection in an isolated CI desktop."""

import ctypes as c
import json
import subprocess
import sys
import traceback
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
                "primary": bool(info.flags & 1),
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
$scope = [System.Windows.Automation.TreeScope]::Descendants
Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Text;
using System.Runtime.InteropServices;
public static class ShellWindows {
    [DllImport("user32.dll")]
    public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
    [StructLayout(LayoutKind.Sequential)]
    public struct Point { public int X; public int Y; }
    public delegate bool Visitor(IntPtr handle, IntPtr data);
    [DllImport("user32.dll")]
    public static extern bool EnumWindows(Visitor callback, IntPtr data);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern int GetClassName(IntPtr handle, StringBuilder name, int length);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern int GetWindowText(IntPtr handle, StringBuilder name, int length);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern IntPtr FindWindow(string type, string name);
    [DllImport("user32.dll")]
    public static extern uint GetWindowThreadProcessId(IntPtr handle, out uint process);
    [DllImport("user32.dll")]
    public static extern bool IsWindowVisible(IntPtr handle);
    [DllImport("user32.dll")]
    public static extern IntPtr WindowFromPoint(Point point);
    [DllImport("user32.dll")]
    public static extern IntPtr GetAncestor(IntPtr handle, uint flags);
    public static bool ContainsPoint(IntPtr handle, int x, int y) {
        return GetAncestor(handle, 2) == GetAncestor(WindowFromPoint(new Point { X = x, Y = y }), 2);
    }
    public static IntPtr[] GetWindows() {
        var handles = new List<IntPtr>();
        uint shellProcess;
        GetWindowThreadProcessId(FindWindow("Shell_TrayWnd", null), out shellProcess);
        EnumWindows((handle, data) => {
            if (!IsWindowVisible(handle)) return true;
            uint process;
            GetWindowThreadProcessId(handle, out process);
            var name = new StringBuilder(256);
            GetClassName(handle, name, name.Capacity);
            var type = name.ToString();
            var title = new StringBuilder(256);
            GetWindowText(handle, title, title.Capacity);
            if (process == shellProcess || type == "NotifyIconOverflowWindow" || title.ToString().IndexOf("System tray overflow", StringComparison.OrdinalIgnoreCase) >= 0) handles.Add(handle);
            return true;
        }, IntPtr.Zero);
        return handles.ToArray();
    }
}
'@
[ShellWindows]::SetThreadDpiAwarenessContext([IntPtr](-4)) | Out-Null
$buttons = [System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::Button)
function Get-TrayButtons {
    $handles = [ShellWindows]::GetWindows()
    foreach ($handle in $handles) {
        if ($handle -ne [IntPtr]::Zero) {
            $shellWindow = [System.Windows.Automation.AutomationElement]::FromHandle($handle)
            $shellWindow.FindAll($scope, $buttons)
        }
    }
}
function Is-NotificationArea($element) {
    $walker = [System.Windows.Automation.TreeWalker]::ControlViewWalker
    for ($depth = 0; $element -and $depth -lt 10; $depth++) {
        $class = $element.Current.ClassName
        if ($class -match 'TaskList') { return $false }
        if ($class -match 'SystemTray|TrayNotifyWnd|NotifyIconOverflowWindow' -or $element.Current.Name -match 'Notification Area|System tray overflow') { return $true }
        $element = $walker.GetParent($element)
    }
    return $false
}
function Find-TrayButton {
    foreach ($candidate in (Get-TrayButtons)) {
        if ($candidate.Current.Name -notmatch '(?i)anyq' -or $candidate.Current.IsOffscreen -or !(Is-NotificationArea $candidate)) { continue }
        $rect = $candidate.Current.BoundingRectangle
        $owner = $candidate
        $walker = [System.Windows.Automation.TreeWalker]::ControlViewWalker
        for ($depth = 0; $owner -and $depth -lt 10; $depth++) {
            $handle = $owner.Current.NativeWindowHandle
            if ($handle -ne 0) {
                if ([ShellWindows]::ContainsPoint([IntPtr]$handle, [int]($rect.X + $rect.Width / 2), [int]($rect.Y + $rect.Height / 2))) { return $candidate }
                break
            }
            $owner = $walker.GetParent($owner)
        }
        [Console]::Error.WriteLine('Tray button is occluded: ' + $candidate.Current.Name)
    }
    return $null
}
$item = Find-TrayButton
if (!$item) {
    $chevron = Get-TrayButtons | Where-Object { $_.Current.Name -eq 'Show Hidden Icons' -and !$_.Current.IsOffscreen } | Select-Object -First 1
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
    [Console]::Error.WriteLine('Tray button names: ' + ((Get-TrayButtons | ForEach-Object {$_.Current.Name}) -join ', '))
    'null'
}
"""
    output = subprocess.check_output(
        ["powershell", "-NoProfile", "-Command", script], text=True, timeout=15
    )
    return json.loads(output)


def menu_item(pid, label):
    menus = [window for window in windows(pid) if window["class"] == "#32768"]
    if not menus:
        return None
    handle = c.c_size_t()
    assert user.SendMessageTimeoutW(
        w.HWND(menus[0]["handle"]), 0x01E1, 0, 0, 2, 1000, c.byref(handle)
    )
    menu = w.HMENU(handle.value)
    count = user.GetMenuItemCount(menu)
    assert count >= 0, "The visible popup has no readable native menu"
    labels = []
    for index in range(count):
        name = c.create_unicode_buffer(512)
        user.GetMenuStringW(menu, index, name, len(name), 0x400)
        labels.append(name.value)
        if name.value.replace("&", "").split("\t")[0] == label:
            bounds = w.RECT()
            assert user.GetMenuItemRect(None, menu, index, c.byref(bounds))
            return rect(bounds)
    print(f"Native menu items: {labels}", file=sys.stderr, flush=True)
    return None


def perform(command):
    action = command["action"]
    if action == "tray-monitor":
        from windows_monitors import set_primary

        return set_primary(command["monitor"], monitors())
    if action == "focus":
        target = max(windows(command["pid"]), key=lambda item: item["height"])
        user.ShowWindow(w.HWND(target["handle"]), 9)
        user.SetForegroundWindow(w.HWND(target["handle"]))
        owner = w.DWORD()
        user.GetForegroundWindow.restype = w.HWND
        user.GetWindowThreadProcessId(
            w.HWND(user.GetForegroundWindow()), c.byref(owner)
        )
        return owner.value == command["pid"]
    if action in ("clipboard-clear", "clipboard-text"):
        if not user.OpenClipboard(None):
            return "" if action == "clipboard-text" else False
        try:
            if action == "clipboard-clear":
                return bool(user.EmptyClipboard())
            user.GetClipboardData.restype = w.HANDLE
            handle = user.GetClipboardData(13)
            if not handle:
                return ""
            kernel = c.windll.kernel32
            kernel.GlobalLock.restype = c.c_void_p
            pointer = kernel.GlobalLock(w.HANDLE(handle))
            assert pointer, "Unicode clipboard data is not readable"
            try:
                return c.wstring_at(pointer)
            finally:
                kernel.GlobalUnlock(w.HANDLE(handle))
        finally:
            user.CloseClipboard()
    if action == "process-path":
        kernel = c.WinDLL("kernel32", use_last_error=True)
        kernel.OpenProcess.restype = w.HANDLE
        handle = kernel.OpenProcess(0x1000, False, command["pid"])
        assert handle, c.WinError(c.get_last_error())
        try:
            path = c.create_unicode_buffer(32768)
            length = w.DWORD(len(path))
            assert kernel.QueryFullProcessImageNameW(
                w.HANDLE(handle), 0, path, c.byref(length)
            )
            return path.value
        finally:
            kernel.CloseHandle(w.HANDLE(handle))
    if action == "monitors":
        return monitors()
    if action == "configure-displays":
        from windows_monitors import configure

        return configure()
    if action == "windows":
        return windows(command["pid"])
    if action == "menu-open":
        return any(window["class"] == "#32768" for window in windows(command["pid"]))
    if action == "menu-item":
        return menu_item(command["pid"], command["label"])
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
        traceback.print_exc(file=sys.stderr)
        reply = {"ok": False, "error": str(error)}
    print(json.dumps(reply), flush=True)
