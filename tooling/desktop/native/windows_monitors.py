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
    return configured
