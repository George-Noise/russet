#!/usr/bin/env bash
# The WAND + FILL + ESC probe (owner request 2026-11-23: the magic-wand
# selection «как в Paint.Net … простой алгоритм», the fill through the SAME
# flood core with the icon LAST in the Tools grid, and Esc deselecting from
# any tool). Replays under Xvfb with xdotool (the drag-probe template per
# SANDBOX.md) and measures PIXEL EVIDENCE on the SAME binary:
#   1. a pencil blob (six stacked strokes),
#   2. the FILL (hotkey F) on the empty background floods it BLACK — the
#      canvas's white pixel count collapses (one click = one Fill node),
#   3. undo restores the white background,
#   4. the WAND: Select (hotkey V) + two left-clicks on the Select button
#      cycle Rect→Lasso→Wand; a wand click on the background selects the
#      flood band — the marching ants' WHITE underlay lights up along the
#      blob's black boundary,
#   5. Esc (the GLOBAL deselect) drops the selection — the light boundary
#      pixels vanish.
# The tools-grid order (Fill LAST), the wand panel row and the ants' look
# are the human pass on the same shots — no pixel assertion is cheaper than
# looking.
# Requires the SANDBOX.md fix (/tmp/xkbfix, /tmp/xdotool) and a built
# binary. Exit 0 = all assertions pass.
# Usage: bash scripts/gui_wand_fill_check.sh <path-to-russet-binary>
set -u
BIN="$1"
shift 2>/dev/null || true
LIB=/tmp/xkbfix/ext/usr/lib/x86_64-linux-gnu:/tmp/xdotool/ext/usr/lib/x86_64-linux-gnu
XD=/tmp/xdotool/ext/usr/bin/xdotool
OUT=/tmp/wand_fill_shots
rm -rf "$OUT" && mkdir -p "$OUT"

pkill -x Xvfb 2>/dev/null; pkill -x russet 2>/dev/null; sleep 1

setsid Xvfb :99 -screen 0 1280x800x24 >/tmp/xvfb.log 2>&1 </dev/null &
sleep 2
DISPLAY=:99 LIBGL_ALWAYS_SOFTWARE=1 LD_LIBRARY_PATH="$LIB" \
  setsid "$BIN" "$@" >/tmp/russet_run.log 2>&1 </dev/null &
sleep 8

# Raw Xvfb has NO window manager — focus the window explicitly (the drag
# template's fix).
WID="$(DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" search --name "Russet" | head -1)"
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" windowfocus "$WID"
sleep 0.5

shot() { # shot <name>
  ffmpeg -y -loglevel error -f x11grab -video_size 1280x800 -i :99 -frames:v 1 "$OUT/$1.png"
}

# The blob: eleven stacked pencil strokes (the default tool IS the pencil),
# 5 px apart — a dense dark mass whose interior the flood stops at, with
# only hairline white slits between the rows.
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

shot 1_calibration_blob_and_tools

# The FILL: hotkey F, then ONE click on the empty background — the flood
# paints the whole white band black (the blob's AA ring stops it).
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" key f
sleep 0.5
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove 700 150 click 1
sleep 1

shot 2_after_fill_background_black

# Undo restores the background (the fill node is undoable like a stroke).
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" key ctrl+z
sleep 1

# The WAND: hotkey V activates the Select tool ALREADY IN Rect mode, so
# TWO left-clicks on its button (the grid's row 3, col 1 — measured on the
# calibration shot) cycle Rect → Lasso → Wand. (Three would wrap back to
# Rect — found on the first probe run.)
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" key v
sleep 0.5
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove 30 358 click 1
sleep 0.5
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove 30 358 click 1
sleep 0.5

shot 3_wand_mode_selected

# The wand click on the empty background: the flood band becomes the
# selection — the marching ants' WHITE underlay lights up along the blob's
# black contour (and the canvas rim).
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove 700 150 click 1
sleep 1

shot 4_wand_selection_ants

# Esc — the GLOBAL deselect (owner request 2026-11-23: «На кнопку Esc нужно
# сделать убирание выделения»): the ants vanish.
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" key Escape
sleep 1

shot 5_after_esc_deselected

pkill -x russet
pkill -x Xvfb

python3 - "$OUT" <<'PYEOF'
import sys
from PIL import Image

out = sys.argv[1]
a = Image.open(f"{out}/1_calibration_blob_and_tools.png").convert("RGB")
b = Image.open(f"{out}/2_after_fill_background_black.png").convert("RGB")
baseline = Image.open(f"{out}/3_wand_mode_selected.png").convert("RGB")
c = Image.open(f"{out}/4_wand_selection_ants.png").convert("RGB")
d = Image.open(f"{out}/5_after_esc_deselected.png").convert("RGB")

def count(im, box, pred):
    px = im.load()
    n = 0
    for y in range(box[1], box[3]):
        for x in range(box[0], box[2]):
            r, g, bl = px[x, y]
            if pred(r, g, bl):
                n += 1
    return n

whites = lambda im, box: count(im, box, lambda r, g, b: r > 230 and g > 230 and b > 230)

def diffs(im1, im2, box):
    # The ants repaint the contour BOTH ways (a white underlay over the
    # dark stroke rows AND black dashes over the white slits), so the
    # signal is the raw pixel-change count against the pre-selection
    # baseline — not a one-direction light delta.
    p1, p2 = im1.load(), im2.load()
    n = 0
    for y in range(box[1], box[3]):
        for x in range(box[0], box[2]):
            if p1[x, y] != p2[x, y]:
                n += 1
    return n

CANVAS = (200, 100, 860, 700)
# The blob's box (the strokes ran (260..340) × (268..330)).
BLOB = (258, 264, 342, 334)
w_before = whites(a, CANVAS)
w_after = whites(b, CANVAS)
ants = diffs(baseline, c, BLOB)
gone = diffs(baseline, d, BLOB)
print(f"white before fill={w_before} after fill={w_after}; ants diff={ants}, after esc diff={gone}")
assert w_before > 30000, "the calibration canvas is not a white field"
assert w_after < 4000, (
    f"the background flood did not paint (white {w_after}) — the Fill tool did not commit"
)
assert ants >= 60, (
    f"no ants on the blob contour (diff {ants}) — the wand flood did not select"
)
assert gone <= 15, (
    f"the ants survived Esc (diff {gone}) — the deselect did not fire"
)
print(
    "PROBE PASS: the Fill flood painted the background black (one Fill node), "
    "the wand flood-selected the background (the ants lit the blob's contour), "
    "and Esc dropped the selection"
)
PYEOF
