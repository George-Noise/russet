#!/usr/bin/env bash
# The WET-JOINT PROBE (owner bug report 2026-11-23: with a semi-transparent
# brush color the stroke's joint circles turned less transparent — each
# frame's smooth-piece stroke carries round caps, the next frame's start
# cap (and the press dot) overlapped the previous frame's cap, and
# premultiplied source-over stacked the alpha at every overlap).
# End-to-end evidence under Xvfb: the Brush tool launches with a
# translucent red (`--primary FF000080`), one slow horizontal glide drag
# paints a stroke whose EVERY piece boundary is a frame joint (one
# mousemove per frame at the glide's 30 ms cadence). The evidence is pixel
# UNIFORMITY on the screenshot: the stroke's centerline row is sampled and
# every stroke pixel must match the mode color within a small tolerance —
# the wet layer composites the whole gesture ONCE, so joints, the press
# dot and the body are one color (before the fix the joint circles read
# tens of units darker: the same deviation the headless regression
# `app/input/tests.rs::translucent_brush_joints_match_the_stroke_body`
# pins at the pixel level). The fixed screen row is safe from the floating
# windows: the Tools window sits at the LEFT edge, the Palette at the
# left-bottom, and the drag runs y=300, x 300..600 — mid-canvas at either
# canvas anchoring, clear of both. The gesture mechanics (piece chaining,
# cap geometry, one-shot composite math) are pinned by the headless tests
# (`tools/wet/tests.rs`, `app/input/tests.rs`).
# Requires the SANDBOX.md fix (/tmp/xkbfix, /tmp/xdotool). Exit 0 = the
# stroke is uniform along its whole length.
# Usage: bash scripts/gui_wet_joint_check.sh <path-to-russet-binary>
set -u
BIN="$1"
LIB=/tmp/xkbfix/ext/usr/lib/x86_64-linux-gnu:/tmp/xdotool/ext/usr/lib/x86_64-linux-gnu
XD=/tmp/xdotool/ext/usr/bin/xdotool
OUT=/tmp/wet_joint_shots
rm -rf "$OUT" && mkdir -p "$OUT"

pkill -x Xvfb 2>/dev/null; pkill -x russet 2>/dev/null; sleep 1

setsid Xvfb :99 -screen 0 1280x800x24 >/tmp/xvfb.log 2>&1 </dev/null &
sleep 2

DISPLAY=:99 LIBGL_ALWAYS_SOFTWARE=1 LD_LIBRARY_PATH="$LIB" \
  setsid "$BIN" --tool brush --primary FF000080 >/tmp/russet_run.log 2>&1 </dev/null &
sleep 8

# Raw Xvfb has NO window manager — focus explicitly or the gestures go
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

shot shot0_before
drag 300 300 600 300
sleep 1
shot shot1_stroke
pkill -x russet 2>/dev/null; pkill -x Xvfb 2>/dev/null

python3 - "$OUT/shot0_before.png" "$OUT/shot1_stroke.png" <<'PY'
import sys
from collections import Counter
from PIL import Image

before = Image.open(sys.argv[1]).convert("RGB")
after = Image.open(sys.argv[2]).convert("RGB")
ROW, X0, X1 = 300, 312, 588

b = list(before.crop((X0, ROW, X1, ROW + 1)).getdata())
a = list(after.crop((X0, ROW, X1, ROW + 1)).getdata())

stroke = [p for p in a if p[0] - p[2] > 30]  # translucent red over white
print(f"stroke pixels on the row: {len(stroke)}")
assert len(stroke) >= 180, "the drag must paint a long stroke"

mode = Counter(stroke).most_common(1)[0]
print(f"mode color {mode[0]} x{mode[1]}")
dev = max(max(abs(p[c] - mode[0][c]) for c in range(3)) for p in stroke)
print(f"max per-channel deviation from the mode: {dev}")

before_red = sum(1 for p in b if p[0] - p[2] > 30)
print(f"red pixels on the row BEFORE the drag: {before_red}")
assert before_red == 0, "the row must start clean"

# The wet layer composites the whole gesture ONCE: uniform color along the
# stroke (tolerance covers the PNG/AA noise only). Before the fix the
# joint circles deviated by tens of units per channel.
assert dev <= 8, f"joints must not darken (deviation {dev} > 8)"
print("WET-JOINT PROBE: PASS")
PY
