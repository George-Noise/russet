#!/usr/bin/env bash
# The --atlas-test SHOT (owner request 2026-11-21): runs the binary WITH
# the atlas diagnostic under Xvfb and screenshots the full window — the
# evidence image for the human (the 1 px speckle ring on every icon
# picture: the Tools grid, the panel buttons, the History Tree rows and
# the pipette cursor's source raster). No pixel assertions on the LAYOUT —
# the automated signal is the count of ADJACENT pure-black/pure-white
# pixel pairs (the ring's signature; the ordinary UI has none: the canvas
# is uniform white, the dark theme's surfaces are grays, the AA edges are
# blends). The human decides what the visible insets mean.
# Usage: bash scripts/gui_atlas_test_shot.sh <path-to-russet-binary>
set -u
BIN="$1"
LIB=/tmp/xkbfix/ext/usr/lib/x86_64-linux-gnu:/tmp/xdotool/ext/usr/lib/x86_64-linux-gnu
OUT=/tmp/atlas_test_shots
rm -rf "$OUT" && mkdir -p "$OUT"

pkill -x Xvfb 2>/dev/null; pkill -x russet 2>/dev/null; sleep 1

setsid Xvfb :99 -screen 0 1280x800x24 >/tmp/xvfb.log 2>&1 </dev/null &
sleep 2
DISPLAY=:99 LIBGL_ALWAYS_SOFTWARE=1 LD_LIBRARY_PATH="$LIB" \
  setsid "$BIN" --atlas-test >/tmp/russet_run.log 2>&1 </dev/null &
sleep 8

ffmpeg -y -loglevel error -f x11grab -video_size 1280x800 -i :99 -frames:v 1 "$OUT/atlas_test.png"
pkill -x russet
pkill -x Xvfb

python3 - "$OUT/atlas_test.png" <<'PYEOF'
import sys
from PIL import Image

im = Image.open(sys.argv[1]).convert("RGB")
w, h = im.size
px = im.load()
pairs = 0
for y in range(h):
    for x in range(1, w):
        a, b = px[x - 1, y], px[x, y]
        if {a, b} == {(0, 0, 0), (255, 255, 255)}:
            pairs += 1
for y in range(1, h):
    for x in range(w):
        a, b = px[x, y - 1], px[x, y]
        if {a, b} == {(0, 0, 0), (255, 255, 255)}:
            pairs += 1
print(f"adjacent pure-black/pure-white pairs: {pairs}")
assert pairs > 200, "no speckle rings on screen — the flag did not reach the atlas"
print("ATLAS-TEST SHOT PASS: the speckle rings render")
PYEOF
