#!/usr/bin/env bash
# The MOUSE-SIDE PROBE (owner request 2026-11-23: Mouse4/Mouse5 undo/redo
# behind a first-press opt-in modal — SPEC §3.9). End-to-end evidence under
# Xvfb against the REAL binary:
#   1. one red stroke drag paints a node;
#   2. the FIRST Mouse4 press (X button 8 → winit Back → egui Extra1)
#      opens the "Mouse Side Buttons" modal AND leaves the canvas alone
#      (the stroke survives — the asking press never undoes);
#   3. clicking YES enables the binding for the session;
#   4. the NEXT Mouse4 press UNDOES (the stroke disappears from the
#      canvas), and a Mouse5 press (X button 9 → winit Forward → egui
#      Extra2) REDOES (the stroke is back).
# The Yes/No buttons are located from the modal screenshot (two bright
# button blobs in the dialog's lower strip; Yes is the RIGHTMOST one —
# the dialog_button_row convention). The state machine, the decline path
# and the gesture guard are pinned by the headless tests
# (`app/mouse_side/tests.rs`).
# Requires the SANDBOX.md fix (/tmp/xkbfix, /tmp/xdotool). Exit 0 = PASS.
# Usage: bash scripts/gui_mouse_side_check.sh <path-to-russet-binary>
set -u
BIN="$1"
LIB=/tmp/xkbfix/ext/usr/lib/x86_64-linux-gnu:/tmp/xdotool/ext/usr/lib/x86_64-linux-gnu
XD=/tmp/xdotool/ext/usr/bin/xdotool
OUT=/tmp/mouse_side_shots
rm -rf "$OUT" && mkdir -p "$OUT"

pkill -x Xvfb 2>/dev/null; pkill -x russet 2>/dev/null; sleep 1

setsid Xvfb :99 -screen 0 1280x800x24 >/tmp/xvfb.log 2>&1 </dev/null &
sleep 2

DISPLAY=:99 LIBGL_ALWAYS_SOFTWARE=1 LD_LIBRARY_PATH="$LIB" \
  setsid "$BIN" --tool brush --primary FF0000FF >/tmp/russet_run.log 2>&1 </dev/null &
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

red_on_row() { # red_on_row <png> — red stroke pixels on the drag row
  python3 - "$1" <<'PY'
import sys
from PIL import Image
im = Image.open(sys.argv[1]).convert("RGB")
row = list(im.crop((300, 300, 600, 301)).getdata())
print(sum(1 for p in row if p[0] - p[2] > 30))
PY
}

drag 300 300 600 300
sleep 0.5
shot 1_stroke
REDO_BASE="$(red_on_row "$OUT/1_stroke.png")"
echo "stroke painted: $REDO_BASE red pixels on the row"
if [ "$REDO_BASE" -lt 180 ]; then
  echo "MOUSE-SIDE PROBE: FAIL — the stroke never painted"
  exit 1
fi

# The FIRST Mouse4 press asks — and must NOT undo.
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" click 8
sleep 0.5
shot 2_modal
ASK_ROW="$(red_on_row "$OUT/2_modal.png")"
echo "red pixels after the asking press: $ASK_ROW"
if [ "$ASK_ROW" -ne "$REDO_BASE" ]; then
  echo "MOUSE-SIDE PROBE: FAIL — the asking press must not undo"
  exit 1
fi

# Locate the modal's Yes button: flood-fill the dialog's dark body from
# the content center (the dialog floats centered), then find the button
# blobs inside it (their fill sits between the body's 27 and the text's
# 200+); Yes is the RIGHTMOST blob (the dialog_button_row convention).
YES="$(python3 - "$OUT/2_modal.png" <<'PY'
import sys
from PIL import Image

im = Image.open(sys.argv[1]).convert("L")
w, h = im.size
px = im.load()

def near_body(v):
    return abs(v - 27) <= 10

# Walk down from the content center until a dialog-body pixel (the modal
# floats centered on the content rect — ~(640, 413) at 1280x800).
sx, sy = 640, 413
while sy < h and not near_body(px[sx, sy]):
    sy += 1
if sy == h:
    raise SystemExit("")  # nothing dark down the center: no modal

stack = [(sx, sy)]
seen = set()
xs, ys = [], []
while stack:
    x, y = stack.pop()
    if (x, y) in seen or not (0 <= x < w and 0 <= y < h) or not near_body(px[x, y]):
        continue
    seen.add((x, y))
    xs.append(x); ys.append(y)
    stack += [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]
dx0, dx1, dy0, dy1 = min(xs), max(xs), min(ys), max(ys)
assert dx1 - dx0 > 200 and dy1 - dy0 > 80, f"dialog body too small: {(dx0, dy0, dx1, dy1)}"

blobs = []
seen = set()
for y0 in range(dy0, dy1 + 1):
    for x0 in range(dx0, dx1 + 1):
        if (x0, y0) in seen or not (40 <= px[x0, y0] <= 90):
            continue
        stack = [(x0, y0)]
        seen.add((x0, y0))
        bx, by = [], []
        while stack:
            x, y = stack.pop()
            bx.append(x); by.append(y)
            for nx, ny in ((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)):
                if (nx, ny) not in seen and dx0 <= nx <= dx1 and dy0 <= ny <= dy1 \
                        and 40 <= px[nx, ny] <= 90:
                    seen.add((nx, ny))
                    stack.append((nx, ny))
        bw = max(bx) - min(bx) + 1
        bh = max(by) - min(by) + 1
        if 25 <= bw <= 130 and 12 <= bh <= 40 and len(bx) >= 150:
            blobs.append((min(bx), min(by), bw, bh))
assert len(blobs) == 2, f"expected the Yes/No button pair, found {blobs}"
blobs.sort(key=lambda b: b[0])
yes = blobs[1]
print(f"{yes[0] + yes[2] // 2} {yes[1] + yes[3] // 2}")
PY
)"
if [ -z "$YES" ]; then
  echo "MOUSE-SIDE PROBE: FAIL — the modal never appeared"
  exit 1
fi
echo "Yes button at: $YES"
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove $YES sleep 0.2 click 1
sleep 0.5
shot 3_after_yes

# Mouse4 undoes, Mouse5 redoes.
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" click 8
sleep 0.5
shot 4_undo
AFTER_UNDO="$(red_on_row "$OUT/4_undo.png")"
echo "red pixels after Mouse4: $AFTER_UNDO"
if [ "$AFTER_UNDO" -ne 0 ]; then
  echo "MOUSE-SIDE PROBE: FAIL — Mouse4 did not undo the stroke"
  exit 1
fi

DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" click 9
sleep 0.5
shot 5_redo
AFTER_REDO="$(red_on_row "$OUT/5_redo.png")"
echo "red pixels after Mouse5: $AFTER_REDO"
if [ "$AFTER_REDO" -lt 180 ]; then
  echo "MOUSE-SIDE PROBE: FAIL — Mouse5 did not redo the stroke"
  exit 1
fi

pkill -x russet 2>/dev/null; pkill -x Xvfb 2>/dev/null
echo "MOUSE-SIDE PROBE: PASS"
