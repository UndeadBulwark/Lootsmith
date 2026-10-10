#!/usr/bin/env bash
# Launch the Lootsmith GUI on X11 (Xwayland), screenshot its window, and leave
# it running so it can be driven with xdotool.
#
# Usage: scripts/gui-drive.sh [--open save.sav] [--dumps "Asset Library Manager.json"]
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${LOOTSMITH_GUI_BIN:-$ROOT/target/release/lootsmith-gui}"
[ -x "$BIN" ] || BIN="$ROOT/target/debug/lootsmith-gui"
OUT="${SHOT:-/tmp/opencode/lootsmith-shot.png}"
LOG="${LOG:-/tmp/opencode/lootsmith-gui.log}"
mkdir -p "$(dirname "$OUT")"

# Force X11 so xdotool/import can drive and capture the window.
env -u WAYLAND_DISPLAY "$BIN" "$@" >"$LOG" 2>&1 &
PID=$!

WID=""
for _ in $(seq 1 60); do
  WID=$(xdotool search --name '^Lootsmith$' 2>/dev/null | head -1 || true)
  [ -n "$WID" ] && break
  sleep 0.25
done
if [ -z "$WID" ]; then
  echo "window not found; see $LOG" >&2
  exit 1
fi
sleep 0.6
import -window "$WID" "$OUT"
echo "pid=$PID window=$WID shot=$OUT log=$LOG"
