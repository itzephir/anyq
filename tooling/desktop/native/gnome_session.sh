#!/usr/bin/env bash
set -euo pipefail

test "${GITHUB_ACTIONS:-}" = true
mkdir -p desktop-shell-results/linux
export XDG_RUNTIME_DIR="$RUNNER_TEMP/anyq-gnome-runtime"
export XDG_CONFIG_HOME="$RUNNER_TEMP/anyq-gnome-config"
mkdir -p "$XDG_RUNTIME_DIR" "$XDG_CONFIG_HOME"
chmod 700 "$XDG_RUNTIME_DIR"
export MUTTER_DEBUG_NUM_DUMMY_MONITORS=2
export MUTTER_DEBUG_DUMMY_MODE_SPECS=1920x1080:2560x1920
export MUTTER_DEBUG_DUMMY_MONITOR_SCALES=1,2
export XDG_CURRENT_DESKTOP=GNOME
export XDG_SESSION_TYPE=wayland
dbus-update-activation-environment XDG_RUNTIME_DIR XDG_CONFIG_HOME DISPLAY
gsettings set org.gnome.desktop.interface toolkit-accessibility true
gsettings set org.gnome.shell disable-user-extensions false
gsettings set org.gnome.desktop.wm.keybindings move-to-monitor-right "['<Super><Shift>Right']"
gsettings set org.gnome.desktop.wm.keybindings move-to-monitor-left "['<Super><Shift>Left']"
gnome-shell --nested --wayland --wayland-display=anyq-wayland >desktop-shell-results/linux/gnome-shell.log 2>&1 &
gnome_pid=$!
trap 'kill "$gnome_pid" 2>/dev/null || true' EXIT
python - <<'PY'
import os
import time
from pathlib import Path
socket = Path(os.environ['XDG_RUNTIME_DIR']) / 'anyq-wayland'
deadline = time.monotonic() + 30
while not socket.exists():
    assert time.monotonic() < deadline, 'GNOME Wayland socket did not become available'
    time.sleep(0.05)
PY
export WAYLAND_DISPLAY=anyq-wayland
export GDK_BACKEND=wayland
dbus-update-activation-environment WAYLAND_DISPLAY
gdbus wait --session --timeout=30 org.gnome.Shell
stage_ids=$(timeout 30 xdotool search --sync --onlyvisible --pid "$gnome_pid")
mapfile -t stage_windows <<< "$stage_ids"
xdotool windowfocus --sync "${stage_windows[0]}"
xdotool key --clearmodifiers Escape
python tooling/test_desktop_shell.py --cli target/debug/anyq --panel target/debug/anyq-panel --profile linux-gnome-wayland
