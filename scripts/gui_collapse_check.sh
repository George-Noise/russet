#!/usr/bin/env bash
# The CLICK-PROBE for the magnetized tool-settings panel's collapse rule
# (owner request 2026-11-19): while the Tools window is COLLAPSED to its
# title bar (the strip's ▼ button), the panel hides with it and comes back
# the frame the window is expanded again. Adapted from the drag-probe
# template `scripts/gui_magnet_check.sh` (clicks instead of drags); the
# button positions are located FROM THE SCREENSHOTS with PIL, so the probe
# survives window repositioning (a collapse re-anchors a pivot-anchored
# window around its pivot — the strip lands lower). Requires the SANDBOX.md
# fix (/tmp/xkbfix, /tmp/xdotool with libxdo3 + libxtst6) and a built
# binary (`cargo build --features mcp`).
# Usage: bash scripts/gui_collapse_check.sh <path-to-russet-binary>
set -u
BIN="$1"
LIB=/tmp/xkbfix/ext/usr/lib/x86_64-linux-gnu:/tmp/xdotool/ext/usr/lib/x86_64-linux-gnu
OUT=/tmp/collapse_check_shots
XD=/tmp/xdotool/ext/usr/bin/xdotool
rm -rf "$OUT" && mkdir -p "$OUT"

pkill -x Xvfb 2>/dev/null; pkill -x russet 2>/dev/null; sleep 1

setsid Xvfb :99 -screen 0 1280x800x24 >/tmp/xvfb.log 2>&1 </dev/null &
sleep 2
DISPLAY=:99 LIBGL_ALWAYS_SOFTWARE=1 LD_LIBRARY_PATH="$LIB" \
  setsid "$BIN" --tool line >/tmp/russet_run.log 2>&1 </dev/null &
sleep 7

shot() { ffmpeg -y -loglevel error -f x11grab -video_size 1280x800 -i :99 -frames:v 1 "$OUT/$1.png"; }
click() { DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove "$1" "$2" sleep 0.3 click 1 sleep 0.5; }

# Left-icon centroid of the title strip: the strip is the first run of rows
# where x=8 (the first-open dropped gap) is non-canvas BELOW the menu bar
# (the menu bar spans the full width at y 0..~26 and would be matched
# otherwise); the collapse/expand icon is the light cluster in the run's
# first 40 rows, x < 48 (the ✕ sits further right). Prints "X Y" or nothing.
# The run must be >= 10 rows: the X11 crosshair cursor (the Line tool over
# the canvas) is ~30 px wide, so it ALSO crosses x=8 wherever the mouse
# happens to idle — a ~6-row run — and clicking its centroid would click the
# cursor's own position (a no-op) instead of the strip icon (caught by the
# 2026-11-19 header round: the re-expand click landed on the idle cursor).
locate_strip_icon() {
  python3 - "$1" <<'PYEOF'
import sys
from PIL import Image
im = Image.open(sys.argv[1]).convert('RGB')
canvas = (64, 64, 64)
rows = [y for y in range(40, im.height) if im.getpixel((8, y)) != canvas]
if not rows:
    sys.exit(0)
runs = []
start = prev = None
for y in rows:
    if start is None:
        start = prev = y
    elif y == prev + 1:
        prev = y
    else:
        runs.append((start, prev))
        start = prev = y
if start is not None:
    runs.append((start, prev))
top = next((a for a, b in runs if b - a + 1 >= 10), None)
if top is None:
    sys.exit(0)
pts = [(x, y) for y in range(top, min(top + 36, im.height))
       for x in range(10, 48) if im.getpixel((x, y))[0] > 100]
if not pts:
    sys.exit(0)
print(round(sum(p[0] for p in pts) / len(pts)), round(sum(p[1] for p in pts) / len(pts)))
PYEOF
}

# Whether the settings panel is drawn BELOW the tool grid in this shot: the
# panel region is scanned at the grid's center column, just below the grid
# rows (the button-fill bounding box); no buttons (a collapsed window) means
# no panel. Prints "present" / "absent".
panel_state() {
  python3 - "$1" <<'PYEOF'
import sys
from PIL import Image
im = Image.open(sys.argv[1]).convert('RGB')
canvas = (64, 64, 64)
fill = (216, 216, 216)
xs = []
ys = []
for y in range(0, im.height, 1):
    for x in range(0, 300, 1):
        if im.getpixel((x, y)) == fill:
            xs.append(x)
            ys.append(y)
if not xs:
    print("absent")
    sys.exit(0)
cx = (min(xs) + max(xs)) // 2
count = sum(1 for y in range(max(ys) + 4, min(max(ys) + 70, im.height))
            if im.getpixel((cx, y)) != canvas)
print("present" if count > 10 else "absent")
PYEOF
}

shot 1_expanded
PANEL_1=$(panel_state "$OUT/1_expanded.png")
POS=$(locate_strip_icon "$OUT/1_expanded.png")
if [ -z "$POS" ]; then echo "FAIL: the collapse icon was not found in shot 1"; exit 1; fi
click $POS
sleep 2
shot 2_collapsed
# Escape closes any popup a stray click may have opened, so the next locate
# sees the plain screen (harmless otherwise — no gesture is active).
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" key Escape
PANEL_2=$(panel_state "$OUT/2_collapsed.png")
POS=$(locate_strip_icon "$OUT/2_collapsed.png")
if [ -z "$POS" ]; then echo "FAIL: the expand icon was not found in shot 2"; exit 1; fi
click $POS
sleep 2
shot 3_reexpanded
PANEL_3=$(panel_state "$OUT/3_reexpanded.png")

pkill -x russet
pkill -x Xvfb

echo "expanded: panel $PANEL_1 (want present)"
echo "collapsed: panel $PANEL_2 (want absent)"
echo "re-expanded: panel $PANEL_3 (want present)"
if [ "$PANEL_1" = present ] && [ "$PANEL_2" = absent ] && [ "$PANEL_3" = present ]; then
  echo "COLLAPSE CHECK OK"
  exit 0
fi
echo "COLLAPSE CHECK FAILED"
exit 1
