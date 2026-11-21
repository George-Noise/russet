#!/usr/bin/env bash
# The HISTORY-TREE ROW probe (owner request 2026-11-21: the pencil,
# brush and eraser rows draw the TOOL'S ICON from the small second atlas
# at 16 pt — the size pass grew it from the first 12 pt — and every row
# drops the node id; the eraser row paints NO color chip: it erases, it
# does not paint).
# Replays two pencil strokes, a brush stroke (hotkey B) and an eraser
# stroke crossing the first pencil line (hotkey E) under Xvfb with xdotool
# (the drag-probe template per SANDBOX.md), screenshots the default History
# Tree window (first-open spot: right edge, just below the menu bar) and
# measures PIXEL EVIDENCE on the SAME binary:
#   - the tree rendered (the branch rails' GitHub-graph blue is present),
#   - the pencil/brush ICONS are drawn in the rows (their emblems'
#     warm-gold body pixels — the hue window is calibrated wide and the
#     threshold low; nothing else on the screen's right side is warm-gold:
#     the rails are blue/green/purple, the row text light gray, the swatch
#     black, the canvas white with black strokes — the Tools window's own
#     yellow pencil button sits far left, out of the x >= 900 scan),
#   - the eraser ICON is drawn (the pink eraser body — r>200 with b>g, a
#     hue nothing else in the scan region has).
# The chipless eraser row, the tightened icon↔chip gap (ICON_CHIP_GAP
# 6 pt) and the node id's absence are code-level facts (paint/tests.rs)
# plus the human pass — no pixel assertion could be cheaper than looking.
# Requires the SANDBOX.md fix (/tmp/xkbfix, /tmp/xdotool) and a built
# binary. Exit 0 = all assertions pass.
# Usage: bash scripts/gui_tree_icon_check.sh <path-to-russet-binary> [extra launch args...]
#       e.g. `... target/debug/russet --atlas-test` — the atlas diagnostic's
#       speckle rings mark every icon picture's exact edge, which is how the
#       size pass of 2026-11-21 (12 pt → 16 pt) was judged.
set -u
BIN="$1"
shift 2>/dev/null || true
LIB=/tmp/xkbfix/ext/usr/lib/x86_64-linux-gnu:/tmp/xdotool/ext/usr/lib/x86_64-linux-gnu
XD=/tmp/xdotool/ext/usr/bin/xdotool
OUT=/tmp/tree_icon_shots
rm -rf "$OUT" && mkdir -p "$OUT"

pkill -x Xvfb 2>/dev/null; pkill -x russet 2>/dev/null; sleep 1

setsid Xvfb :99 -screen 0 1280x800x24 >/tmp/xvfb.log 2>&1 </dev/null &
sleep 2
DISPLAY=:99 LIBGL_ALWAYS_SOFTWARE=1 LD_LIBRARY_PATH="$LIB" \
  setsid "$BIN" "$@" >/tmp/russet_run.log 2>&1 </dev/null &
sleep 8

# Raw Xvfb has NO window manager — focus the window explicitly (the drag
# template's fix; the canvas drag needs the key/pointer stream delivered).
WID="$(DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" search --name "Russet" | head -1)"
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" windowfocus "$WID"
sleep 0.5

shot() { # shot <name>
  ffmpeg -y -loglevel error -f x11grab -video_size 1280x800 -i :99 -frames:v 1 "$OUT/$1.png"
}

shot 1_tree_fresh_start_row

# Two pencil strokes on the canvas (the default tool IS the pencil): each
# drag records ONE pencil node — the tree gains two pencil rows under the
# Start root. Both drags stay well clear of the tree window (x >= 1032).
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove 300 300 mousedown 1
for t in 1 2 3 4 5 6 7 8; do
  DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove $((300 + t * 15)) $((300 + t * 5)) sleep 0.05
done
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mouseup 1
sleep 1
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove 500 420 mousedown 1
for t in 1 2 3 4 5 6 7 8; do
  DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove $((500 + t * 12)) $((420 - t * 6)) sleep 0.05
done
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mouseup 1
sleep 1

# The brush stroke (hotkey B — input.rs) records a brush row with its
# icon; the eraser stroke (hotkey E) crosses the FIRST pencil line (the
# erase visibly changes pixels, so the node is certain) — its row draws
# the eraser icon and NO color chip. All drags stay clear of the tree
# window (x >= 900).
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" key b
sleep 0.5
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove 650 300 mousedown 1
for t in 1 2 3 4 5 6; do
  DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove $((650 + t * 14)) $((300 + t * 4)) sleep 0.05
done
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mouseup 1
sleep 1
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" key e
sleep 0.5
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove 340 300 mousedown 1
for t in 1 2 3 4 5 6 7 8; do
  DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove $((340 + t * 11)) $((300 + t * 11)) sleep 0.05
done
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mouseup 1
sleep 1

shot 2_tree_after_pencil_brush_eraser
pkill -x russet
pkill -x Xvfb

python3 - "$OUT" <<'PYEOF'
import sys
from PIL import Image

out = sys.argv[1]
im = Image.open(f"{out}/2_tree_after_pencil_brush_eraser.png").convert("RGB")
w, h = im.size
px = im.load()

# The History Tree window's first-open spot: the RIGHT edge below the menu
# bar — but the app window is not the full screen width under raw Xvfb, so
# the scan bounds itself to the right SIDE of the screen (x >= 900): the
# tree window hugs the right edge, the Tools window's own yellow pencil
# button sits far left, the canvas is white with black strokes.
X0, X1, Y0, Y1 = 900, 1280, 0, 400
blue = yellow = pink = 0
for y in range(Y0, min(Y1, h)):
    for x in range(X0, min(X1, w)):
        r, g, b = px[x, y]
        if abs(r - 0x4C) < 36 and abs(g - 0x8D) < 36 and abs(b - 0xFF) < 36:
            blue += 1
        elif r > 150 and g > 100 and b < 150 and r > b + 60 and g > b + 30:
            yellow += 1
        elif r > 200 and 120 < g < 210 and b > 150 and b > g:
            pink += 1
print(f"blue={blue} yellow={yellow} pink={pink}")
assert blue > 20, "the branch rails did not render (no GitHub-graph blue)"
assert yellow >= 6, "no pencil/brush-icon gold in the tree rows — the small atlas did not draw"
assert pink >= 3, "no eraser-icon pink in the tree rows — the eraser icon did not draw"
print(
    "PROBE PASS: the pencil/brush rows carry their icons (warm-gold), the "
    "eraser row its pink emblem, and the rails render"
)
PYEOF
