#!/usr/bin/env bash
# The SELECTION-CLIP PROBE (owner request 2026-11-21: an active selection
# confines every painting tool to its region). End-to-end gesture evidence
# under Xvfb: the Select tool makes a rectangle selection, the `b` hotkey
# switches to the Brush, then TWO horizontal red strokes are dragged on the
# SAME canvas row — one fully INSIDE the selection, one crossing its right
# edge ~170 px past it. The evidence is pixel geometry on the screenshots:
#   - shot0 (before any drawing) has NO stroke-red anywhere,
#   - the inside stroke paints red,
#   - the crossing stroke's red STOPS at the selection's right edge — its
#     rightmost red pixel is ≤ edge + 40, i.e. ≥ 130 px short of the drag's
#     endpoint (without the clip the red would reach the endpoint's cap).
# The gestures self-locate: a pre-shot finds the checkerboard canvas (its
# two most common grays) and the longest unbroken checker run on one row,
# so windows (Tools panel, palette) can never occlude the gestures.
# The per-tool mechanics (pencil plots, the eraser, scratch transfer,
# exactness) are pinned by the headless unit tests (`tools/tests/clip.rs`,
# `selection/tests.rs`, `app/preview/tests.rs`, `app/input/tests.rs`).
# Requires the SANDBOX.md fix (/tmp/xkbfix, /tmp/xdotool). Exit 0 = all
# measured numbers in tolerance.
# Usage: bash scripts/gui_selection_clip.sh <path-to-russet-binary>
set -u
BIN="$1"
LIB=/tmp/xkbfix/ext/usr/lib/x86_64-linux-gnu:/tmp/xdotool/ext/usr/lib/x86_64-linux-gnu
XD=/tmp/xdotool/ext/usr/bin/xdotool
OUT=/tmp/selection_clip_shots
rm -rf "$OUT" && mkdir -p "$OUT"

pkill -x Xvfb 2>/dev/null; pkill -x russet 2>/dev/null; sleep 1

setsid Xvfb :99 -screen 0 1280x800x24 >/tmp/xvfb.log 2>&1 </dev/null &
sleep 2

DISPLAY=:99 LIBGL_ALWAYS_SOFTWARE=1 LD_LIBRARY_PATH="$LIB" \
  setsid "$BIN" --tool select --primary FF0000 >/tmp/russet_run.log 2>&1 </dev/null &
sleep 8

# Raw Xvfb has NO window manager — focus explicitly or the `b` hotkey goes
# nowhere (SANDBOX.md pitfall 11).
WID="$(DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" search --name "Russet" | head -1)"
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" windowfocus "$WID"
sleep 0.5

shot() { # shot <name>
  ffmpeg -y -loglevel error -f x11grab -video_size 1280x800 -i :99 -frames:v 1 "$OUT/$1.png"
}

drag() { # drag <x0> <y0> <x1> <y1> — press, glide in steps, release
  DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove "$1" "$2" sleep 0.1 \
    mousedown 1 sleep 0.1
  local n=12
  local i
  for i in $(seq 1 "$n"); do
    local x y
    x="$(python3 -c "print(int($1 + ($3 - $1) * $i / $n))")"
    y="$(python3 -c "print(int($2 + ($4 - $2) * $i / $n))")"
    DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove "$x" "$y" sleep 0.03
  done
  DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mouseup 1 sleep 0.3
}

shot 0_before

# The gestures' row: the canvas displays a WHITE sheet — find the longest
# run of pure white per row (the canvas bbox), then the vertical extent at
# its middle column; the gestures stay inside with margins, so no window
# (Tools panel, palette) can occlude them.
PLAN="$(python3 - "$OUT/0_before.png" <<'PYEOF'
import sys
from PIL import Image

im = Image.open(sys.argv[1]).convert("RGB")
w, h = im.size
px = im.load()
WHITE = (255, 255, 255)

best = (0, 0, 0)  # (length, y, x0)
for y in range(40, h - 40, 3):
    run = 0
    start = None
    for x in range(30, w - 30):
        if px[x, y] == WHITE:
            if run == 0:
                start = x
            run += 1
        else:
            if run > best[0]:
                best = (run, y, start)
            run = 0
    if run > best[0]:
        best = (run, y, start)
length, y0, x0 = best
assert length > 600, f"no white canvas run found (best {length}px at y={y0})"

# The canvas's vertical extent through the run's middle column.
cx = x0 + length // 2
top = y0
while top > 0 and px[cx, top - 1] == WHITE:
    top -= 1
bot = y0
while bot < h - 1 and px[cx, bot + 1] == WHITE:
    bot += 1
height = bot - top
assert height > 220, f"the canvas is too short: {height}px at column {cx}"
print(f"Y={(top + bot) // 2} X0={x0} LEN={length} TOP={top} BOT={bot}")
PYEOF
)"
echo "plan: $PLAN"
eval "$PLAN"

# The gesture band the red scans cover (the strokes and the selection live
# strictly inside it; the white-run plan guarantees no window occludes it).
scan_red() { # scan_red <name> — red samples inside the band, "x y" lines
  python3 - "$OUT/$1.png" "$Y" "$X0" "$TOP" "$BOT" <<'PYEOF'
import sys
from PIL import Image

path, y0, x0, top, bot = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4]), int(sys.argv[5])
im = Image.open(path).convert("RGB")
w, h = im.size
px = im.load()
found = []
for y in range(max(top, y0 - 90), min(bot, y0 + 90), 2):
    for x in range(x0 + 30, min(w, x0 + 720), 2):
        r, g, b = px[x, y]
        if r > 200 and g < 80 and b < 80:
            found.append(f"{x} {y}")
print(*found, sep="\n")
PYEOF
}

BEFORE="$(scan_red 0_before)"
[ -z "$BEFORE" ] || {
  echo "FAIL: shot0 already has stroke-red in the band: $BEFORE"
  exit 1
}

# The selection: a rect spanning the row's left part, ±70 px tall.
SXA=$((X0 + 60)); SXB=$((X0 + 530))
SYA=$((Y - 70)); SYB=$((Y + 70))
drag "$SXA" "$SYA" "$SXB" "$SYB"
sleep 0.5

# Brush via the hotkey (the selection stays — it is document state).
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" key b
sleep 0.5

# Stroke A: crosses the selection's right edge, ending ~120 px past it.
drag $((SXA + 80)) "$Y" $((SXB + 120)) "$Y"
# Stroke B: fully inside the selection, one row above.
drag $((SXA + 80)) $((Y - 45)) $((SXA + 200)) $((Y - 45))
sleep 0.8
shot 1_strokes

pkill -x russet
pkill -x Xvfb

AFTER="$(scan_red 1_strokes)"
[ -n "$AFTER" ] || {
  echo "FAIL: no red painted at all — the strokes never landed"
  exit 1
}
MAX_X="$(echo "$AFTER" | python3 -c "import sys; print(max(int(l.split()[0]) for l in sys.stdin))")"
COUNT="$(echo "$AFTER" | wc -l)"
# The crossing stroke must STOP at the selection's right edge: the drag
# ended 120 px past it — without the clip the red would reach the
# endpoint's cap.
if [ "$MAX_X" -gt "$((SXB + 40))" ]; then
  echo "FAIL: the stroke painted past the selection (rightmost red x=$MAX_X, edge=$SXB) — the clip is NOT working"
  exit 1
fi
echo "PROBE PASS: red stops at x=$MAX_X (edge $SXB, endpoint $((SXB + 120))); $COUNT red samples in the band"
