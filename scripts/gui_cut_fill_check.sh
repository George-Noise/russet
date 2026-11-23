#!/usr/bin/env bash
# The CUT → FILL crash probe (owner bug report 2026-11-23): a fill click on
# fully transparent pixels (the hole a select + cut leaves) CRASHED the app.
# The flow: draw a blob → rect-select it → Cut (the ops row) → the hole →
# switch to the Fill tool → click INSIDE the hole → the app must SURVIVE and
# the hole must be painted; then the wand click on the hole must survive too.
# Usage: bash scripts/gui_cut_fill_check.sh target/debug/russet
set -u
BIN="${1:-target/debug/russet}"
LIB=/tmp/xkbfix/ext/usr/lib/x86_64-linux-gnu:/tmp/xdotool/ext/usr/lib/x86_64-linux-gnu
XD=/tmp/xdotool/ext/usr/bin/xdotool
OUT=/tmp/cut_fill_shots
rm -rf "$OUT" && mkdir -p "$OUT"
# The Cut button of the Select panel's ops row (calibrated on shot 1b —
# the second icon: Copy, Cut, Paste, Crop).
CUT_X="${CUT_X:-63}"
CUT_Y="${CUT_Y:-505}"

pkill -x Xvfb 2>/dev/null; pkill -x russet 2>/dev/null; sleep 1

setsid Xvfb :99 -screen 0 1280x800x24 >/tmp/xvfb.log 2>&1 </dev/null &
sleep 2
DISPLAY=:99 LIBGL_ALWAYS_SOFTWARE=1 LD_LIBRARY_PATH="$LIB" \
  setsid "$BIN" >"$OUT/app.log" 2>&1 </dev/null &
sleep 8

# Raw Xvfb has NO window manager — focus the window explicitly (the drag
# template's fix); without this every `key` event goes nowhere.
WID="$(DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" search --name "Russet" | head -1)"
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" windowfocus "$WID"
sleep 0.5

shot() { ffmpeg -y -loglevel error -f x11grab -video_size 1280x800 -i :99 -frames:v 1 "$OUT/$1.png"; }

shot 1_white_canvas

# The blob: eleven stacked pencil strokes (the default tool IS the pencil),
# 5 px apart — the SAME slow-stepped drag the wand probe calibrated (the
# app needs pointer-move events to register a drag; instant moves vanish).
y=268
for row in 1 2 3 4 5 6 7 8 9 10 11; do
  DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove 260 $y mousedown 1
  for t in 1 2 3 4 5 6 7 8; do
    DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove $((260 + t * 10)) $y sleep 0.04
  done
  DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mouseup 1
  sleep 0.4
  y=$((y + 5))
done
sleep 1

shot 1a_blob_drawn

# The RECT select: V activates the Select tool in the Rect mode; drag the
# region around the blob (the same slow-stepped drag).
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" key v
sleep 1
shot 1b_select_panel_rect
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove 250 258 mousedown 1
for t in 1 2 3 4 5 6 7 8; do
  DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove $((250 + t * 12)) $((258 + t * 10)) sleep 0.04
done
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mouseup 1
sleep 1

shot 2_rect_selected

# CUT via the ops row's second button (the scissors).
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove "$CUT_X" "$CUT_Y" click 1
sleep 1

shot 3_after_cut_hole

# The hole must be transparent: the blob box now shows the app background,
# not white canvas and not black strokes.
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" key f
sleep 0.5

# THE FILL CLICK INSIDE THE HOLE — the owner's crash point.
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove 300 299 click 1
sleep 1.5

# The process must be ALIVE (the owner: «вылетает приложение»).
if ! pgrep -x russet >/dev/null; then
  echo "APP CRASHED after the fill click on the cut hole; log tail:"
  tail -20 "$OUT/app.log"
  exit 1
fi
shot 4_after_fill_inside_hole

# Undo restores the hole. Then THE OWNER'S CRASH CLICK: a fill click on
# the background OUTSIDE the active selection — with the confinement the
# gate stops it before any flood (nothing paints, nothing records); the
# app must survive.
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" key ctrl+z
sleep 1
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove 700 150 click 1
sleep 1.5

if ! pgrep -x russet >/dev/null; then
  echo "APP CRASHED after the fill click outside the selection; log tail:"
  tail -20 "$OUT/app.log"
  exit 1
fi
shot 5_after_fill_outside

# The WAND click on the transparent pixels — the same flood read path —
# must survive too.
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" key v
sleep 0.3
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove 30 358 click 1
sleep 0.3
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove 30 358 click 1
sleep 0.5
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove 300 299 click 1
sleep 1.5

if ! pgrep -x russet >/dev/null; then
  echo "APP CRASHED after the wand click on the cut hole; log tail:"
  tail -20 "$OUT/app.log"
  exit 1
fi
shot 6_after_wand_on_hole

DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" key Escape
sleep 0.5

python3 - "$OUT" <<'PYEOF'
import sys, os
from PIL import Image

out = sys.argv[1]

def img(name):
    return Image.open(os.path.join(out, name + ".png")).convert("RGB")

def count(im, box, pred):
    px = im.load()
    n = 0
    for y in range(box[1], box[3]):
        for x in range(box[0], box[2]):
            if pred(*px[x, y]):
                n += 1
    return n

whites = lambda im, box: count(im, box, lambda r, g, b: r > 230 and g > 230 and b > 230)
blacks = lambda im, box: count(im, box, lambda r, g, b: r < 40 and g < 40 and b < 40)

# The blob's box (the strokes ran (260..340) × (268..330) on screen).
BLOB = (258, 264, 342, 334)
a = img("1a_blob_drawn")
sel = img("2_rect_selected")
hole = img("3_after_cut_hole")
filled = img("4_after_fill_inside_hole")
outside = img("5_after_fill_outside")

w_before = whites(a, BLOB)
b_before = blacks(a, BLOB)
b_sel = blacks(sel, BLOB)
w_hole = whites(hole, BLOB)
b_hole = blacks(hole, BLOB)
b_filled = blacks(filled, BLOB)
# The background box around the outside-click point (700, 150): with the
# confinement nothing paints there (without it the whole background would
# flood black — the owner's crash click).
BG = (690, 130, 750, 180)
b_outside = blacks(outside, BG)
print(
    f"blob before: {b_before} black strokes on {w_before} white; selected: {b_sel} black; "
    f"after cut: {w_hole} white/{b_hole} black (the hole shows the dark background); "
    f"after fill: {b_filled} black (the hole painted)"
)
assert b_before > 500, "the blob did not draw"
assert w_before > 3000, "the blob's surroundings are not white canvas"
assert b_hole < 300, (
    f"the cut left {b_hole} black pixels in the blob box — the strokes survived the cut"
)
assert b_filled > 3000, (
    f"the fill inside the hole left only {b_filled} black pixels — the fill did not paint"
)
assert b_outside < 100, (
    f"the fill click OUTSIDE the selection painted {b_outside} black pixels "
    "in the background box — the confinement is broken"
)
print(
    "PROBE PASS: the cut emptied the selected blob (transparent hole), the fill click "
    "inside the hole SURVIVED and painted the hole black, the fill click OUTSIDE "
    "the selection painted nothing, the wand click on the transparent pixels "
    "survived too"
)
PYEOF
STATUS=$?
pkill -x russet 2>/dev/null
exit $STATUS
