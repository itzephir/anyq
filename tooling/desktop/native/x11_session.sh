#!/usr/bin/env bash
set -euo pipefail

test "${GITHUB_ACTIONS:-}" = true
mkdir -p desktop-shell-results/linux
configuration="$RUNNER_TEMP/anyq-xorg.conf"
cat >"$configuration" <<'XORG'
Section "ServerFlags"
    Option "AutoAddDevices" "false"
    Option "DontVTSwitch" "true"
EndSection
Section "Device"
    Identifier "AnyqDummy"
    Driver "dummy"
    VideoRam 256000
EndSection
Section "Monitor"
    Identifier "AnyqMonitor"
    HorizSync 5.0-1000.0
    VertRefresh 5.0-200.0
    Modeline "1920x1080" 148.50 1920 2008 2052 2200 1080 1084 1089 1125 +HSync +VSync
    Modeline "1280x960" 108.00 1280 1376 1488 1800 960 961 964 1000 +HSync +VSync
EndSection
Section "Screen"
    Identifier "AnyqScreen"
    Device "AnyqDummy"
    Monitor "AnyqMonitor"
    DefaultDepth 24
    SubSection "Display"
        Depth 24
        Modes "1920x1080" "1280x960"
        Virtual 3840 1440
    EndSubSection
EndSection
XORG
export DISPLAY=:98
sudo Xorg "$DISPLAY" -config "$configuration" -logfile "$PWD/desktop-shell-results/linux/xorg.log" -noreset -nolisten tcp -ac >desktop-shell-results/linux/xorg-output.log 2>&1 &
xorg_pid=$!
trap 'sudo kill "$xorg_pid" 2>/dev/null || true' EXIT
python - <<'PY'
import subprocess
import time
deadline = time.monotonic() + 30
while subprocess.run(['xrandr', '--query'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode:
    assert time.monotonic() < deadline, 'Native Xorg display did not become available'
    time.sleep(0.05)
PY
xrandr --output DUMMY0 --mode 1920x1080 --primary --pos 0x0
xrandr --addmode DUMMY1 1280x960
xrandr --output DUMMY1 --mode 1280x960 --pos 1920x0
xrandr --listmonitors | tee desktop-shell-results/linux/xrandr-monitors.txt
openbox >desktop-shell-results/linux/openbox.log 2>&1 &
python tooling/test_desktop_shell.py --cli target/debug/anyq --panel target/debug/anyq-panel --profile linux-x11
