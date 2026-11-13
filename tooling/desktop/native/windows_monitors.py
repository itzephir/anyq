"""Configure actual per-display DPI on disposable Windows test desktops."""

import ctypes as c
import os
from ctypes import wintypes as w


class Luid(c.Structure):
    _fields_ = [("low", w.DWORD), ("high", w.LONG)]


class Rational(c.Structure):
    _fields_ = [("numerator", w.UINT), ("denominator", w.UINT)]


class Source(c.Structure):
    _fields_ = [("adapter", Luid), ("id", w.UINT), ("mode", w.UINT), ("flags", w.UINT)]


class Target(c.Structure):
    _fields_ = [
        ("adapter", Luid),
        ("id", w.UINT),
        ("mode", w.UINT),
        ("technology", w.UINT),
        ("rotation", w.UINT),
        ("scaling", w.UINT),
        ("refresh", Rational),
        ("scanline", w.UINT),
        ("available", w.BOOL),
        ("flags", w.UINT),
    ]


class DisplayPath(c.Structure):
    _fields_ = [("source", Source), ("target", Target), ("flags", w.UINT)]


class Header(c.Structure):
    _fields_ = [("type", w.UINT), ("size", w.UINT), ("adapter", Luid), ("id", w.UINT)]


class DpiInfo(c.Structure):
    _fields_ = [
        ("header", Header),
        ("minimum", w.INT),
        ("current", w.INT),
        ("maximum", w.INT),
    ]


class DpiSetting(c.Structure):
    _fields_ = [("header", Header), ("relative", w.INT)]


class SourceName(c.Structure):
    _fields_ = [("header", Header), ("name", w.WCHAR * 32)]


class DisplayMode(c.Structure):
    _fields_ = [
        ("device", w.WCHAR * 32),
        ("spec", w.WORD),
        ("driver", w.WORD),
        ("size", w.WORD),
        ("extra", w.WORD),
        ("fields", w.DWORD),
        ("x", w.LONG),
        ("y", w.LONG),
        ("orientation", w.DWORD),
        ("fixed_output", w.DWORD),
        ("color", w.SHORT),
        ("duplex", w.SHORT),
        ("y_resolution", w.SHORT),
        ("tt_option", w.SHORT),
        ("collate", w.SHORT),
        ("form", w.WCHAR * 32),
        ("log_pixels", w.WORD),
        ("bits", w.DWORD),
        ("width", w.DWORD),
        ("height", w.DWORD),
        ("display_flags", w.DWORD),
        ("frequency", w.DWORD),
        ("icm_method", w.DWORD),
        ("icm_intent", w.DWORD),
        ("media", w.DWORD),
        ("dither", w.DWORD),
        ("reserved1", w.DWORD),
        ("reserved2", w.DWORD),
        ("panning_width", w.DWORD),
        ("panning_height", w.DWORD),
    ]


def set_primary(device, outputs):
    assert os.environ.get("GITHUB_ACTIONS") == "true", (
        "Change the primary display only in disposable CI"
    )
    user = c.windll.user32
    target = next(item for item in outputs if item["device"] == device)
    plans = []
    for output in sorted(outputs, key=lambda item: item["device"] != device):
        mode = DisplayMode()
        mode.size = c.sizeof(mode)
        assert user.EnumDisplaySettingsW(output["device"], -1, c.byref(mode)), (
            "Read display mode",
            output,
        )
        mode.fields = 0x20
        mode.x -= target["bounds"]["x"]
        mode.y -= target["bounds"]["y"]
        flags = 0x10000001 | (0x10 if output["device"] == device else 0)
        plans.append((output, mode, flags))
    for output, mode, flags in plans:
        result = user.ChangeDisplaySettingsExW(
            output["device"], c.byref(mode), None, flags, None
        )
        assert result == 0, (
            "Prepare primary display",
            output,
            mode.x,
            mode.y,
            flags,
            result,
        )
    result = user.ChangeDisplaySettingsExW(None, None, None, 0, None)
    assert result == 0, ("Apply primary display", device, result)
    return True


def configure():
    assert os.environ.get("GITHUB_ACTIONS") == "true", (
        "Configure native DPI only in disposable CI"
    )
    user = c.windll.user32
    paths, modes = w.UINT(), w.UINT()
    assert user.GetDisplayConfigBufferSizes(2, c.byref(paths), c.byref(modes)) == 0
    assert c.sizeof(DisplayPath) == 72
    active = (DisplayPath * paths.value)()
    mode_data = c.create_string_buffer(modes.value * 64)
    assert (
        user.QueryDisplayConfig(
            2, c.byref(paths), active, c.byref(modes), mode_data, None
        )
        == 0
    )
    assert paths.value >= 2, "Windows needs at least two real active display paths"
    configured = []
    for index, path in enumerate(active[: paths.value]):
        source = SourceName()
        source.header = Header(
            1, c.sizeof(SourceName), path.source.adapter, path.source.id
        )
        assert user.DisplayConfigGetDeviceInfo(c.byref(source)) == 0
        if index > 0:
            mode = DisplayMode()
            mode.size = c.sizeof(mode)
            assert mode.size == 220
            assert user.EnumDisplaySettingsW(source.name, -1, c.byref(mode))
            mode.fields = 0x20 | 0x80000 | 0x100000 | 0x400000
            mode.x = -1920 if index == 1 else (index - 1) * 1920
            mode.y = -120 if index == 1 else 0
            mode.width, mode.height, mode.frequency = 1920, 1080, 60
            assert (
                user.ChangeDisplaySettingsExW(
                    source.name, c.byref(mode), None, 0x10000001, None
                )
                == 0
            )
        info = DpiInfo()
        info.header = Header(
            0xFFFFFFFD, c.sizeof(DpiInfo), path.source.adapter, path.source.id
        )
        assert user.DisplayConfigGetDeviceInfo(c.byref(info)) == 0
        target_index = 0 if index == 0 else 2
        relative = target_index + info.minimum
        assert info.minimum <= relative <= info.maximum, (
            info.minimum,
            info.maximum,
            relative,
        )
        setting = DpiSetting(
            Header(
                0xFFFFFFFC, c.sizeof(DpiSetting), path.source.adapter, path.source.id
            ),
            relative,
        )
        assert user.DisplayConfigSetDeviceInfo(c.byref(setting)) == 0
        configured.append(
            {
                "source": path.source.id,
                "relative": relative,
                "scale": 1 if index == 0 else 1.5,
            }
        )
    assert user.ChangeDisplaySettingsExW(None, None, None, 0, None) == 0
    return configured
