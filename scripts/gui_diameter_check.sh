#!/usr/bin/env bash
# The DIAMETER-CIRCLE PROBE (owner request 2026-11-21: the TESTING Shape
# feature — with the Ellipse kind selected and the panel's "Diameter circle
# (Shift)" checkbox ON, a Shift drag makes the press and release points the
# endpoints of the circle's diameter, so the center is the drag segment's
# midpoint). Replays the gestures under Xvfb per SANDBOX.md with xdotool (a
# panel click on the Ellipse kind button, a checkbox click, two Shift
# drags) and ffmpeg screenshots; the evidence is PIXEL GEOMETRY measured in
# Python on the SAME binary:
#   - the Ellipse kind click lands on the REAL laid-out KIND row (the
#     2/3/3 row sequence below the Tools window's 2/2/2/2 grid — the same
#     blob scan the select probe uses),
#   - the checkbox row's appearance is a pixel DIFF against the previous
#     screenshot at the icon slot (every other window's pixels cancel out),
#   - checkbox ON: the red ring's bbox is SQUARE, its center sits on the
#     drag segment's MIDPOINT, the bbox side is the drag length + the
#     stroke width, and both drag points lie ON the ring while the midpoint
#     (the CENTER) stays unpainted (the default outline-only paint),
#   - checkbox OFF (one click): the same drag draws the OLD corner-anchored
#     proportional circle — the center leaves the midpoint and drops to the
#     anchor diagonal, the press point (the bbox corner) leaves the ring,
#   - MID-DRAG screenshots: with the checkbox ON the live overlay shows the
#     measurement guides — dashed chord dashes along the drag segment and
#     the center dot's dark ring at the segment's midpoint; with the box
#     OFF no chord red and no dot leak at the same-kind samples.
# The panel-level rules (the checkbox exists ONLY for the Ellipse kind, the
# startup state is ON) are pinned by the headless unit tests
# (`app/windows/tests/settings_panel.rs`) — this probe is the end-to-end
# gesture + toggle evidence.
# Requires the SANDBOX.md fix (/tmp/xkbfix, /tmp/xdotool) and a built
# binary. No MCP needed. Exit 0 = all measured numbers in tolerance.
# Usage: bash scripts/gui_diameter_check.sh <path-to-russet-binary>
set -u
BIN="$1"
LIB=/tmp/xkbfix/ext/usr/lib/x86_64-linux-gnu:/tmp/xdotool/ext/usr/lib/x86_64-linux-gnu
XD=/tmp/xdotool/ext/usr/bin/xdotool
OUT=/tmp/diameter_shots
rm -rf "$OUT" && mkdir -p "$OUT"

pkill -x Xvfb 2>/dev/null; pkill -x russet 2>/dev/null; sleep 1

setsid Xvfb :99 -screen 0 1280x800x24 >/tmp/xvfb.log 2>&1 </dev/null &
sleep 2

DISPLAY=:99 LIBGL_ALWAYS_SOFTWARE=1 LD_LIBRARY_PATH="$LIB" \
  setsid "$BIN" --tool shape --primary FF0000 >/tmp/russet_run.log 2>&1 </dev/null &
sleep 8

# Raw Xvfb has NO window manager, so no window ever receives an ICCCM focus
# event — clicks land fine (pointer-follows focus), but KEY and MODIFIER
# events go nowhere and every Shift drag would draw a plain unconstrained
# shape. Focus the window explicitly (the same fix the drag-probe template
# needs for its drags).
WID="$(DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" search --name "Russet" | head -1)"
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" windowfocus "$WID"
sleep 0.5

shot() { # shot <name>
  ffmpeg -y -loglevel error -f x11grab -video_size 1280x800 -i :99 -frames:v 1 "$OUT/$1.png"
}

shot 1_shape_panel_rect_kind

# The Shape panel's KIND/MODE/PLACEMENT rows sit below the Tools window's
# 2/2/2/2 grid as a 2/3/3 row sequence. From those REAL laid-out rects both
# click targets are derived: the Ellipse kind button (the KIND row's second
# cell) and the checkbox row's icon (one 12-pt group gap below the
# PLACEMENT row's left cell — the same SHAPE_GROUP_GAP step the panel's
# rows use).
SCAN_OUT="$(python3 - "$OUT/1_shape_panel_rect_kind.png" <<'PYEOF'
import sys
from PIL import Image

path = sys.argv[1]

def squares(path):
    im = Image.open(path).convert("L")
    w, h = im.size
    px = im.load()
    mask = [[1 if px[x, y] > 100 else 0 for x in range(w)] for y in range(h)]
    seen = [[0] * w for _ in range(h)]
    blobs = []
    for y0 in range(h):
        for x0 in range(w):
            if mask[y0][x0] and not seen[y0][x0]:
                stack = [(x0, y0)]
                seen[y0][x0] = 1
                xs = [x0]; ys = [y0]
                while stack:
                    x, y = stack.pop()
                    for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                        nx, ny = x + dx, y + dy
                        if 0 <= nx < w and 0 <= ny < h and mask[ny][nx] and not seen[ny][nx]:
                            seen[ny][nx] = 1
                            stack.append((nx, ny))
                            xs.append(nx); ys.append(ny)
                bw = max(xs) - min(xs) + 1
                bh = max(ys) - min(ys) + 1
                if 8 <= bw <= 34 and 8 <= bh <= 34 and len(xs) >= 60:
                    blobs.append((min(xs), min(ys), bw, bh))

    def center(b):
        return (b[0] + b[2] / 2, b[1] + b[3] / 2)

    cells = []
    for b in sorted(blobs, key=center):
        cx, cy = center(b)
        for cell in cells:
            ex, ey = center(cell["bbox"])
            if abs(ex - cx) <= 8 and abs(ey - cy) <= 8:
                x0 = min(cell["bbox"][0], b[0]); y0 = min(cell["bbox"][1], b[1])
                x1 = max(cell["bbox"][0] + cell["bbox"][2], b[0] + b[2])
                y1 = max(cell["bbox"][1] + cell["bbox"][3], b[1] + b[3])
                cell["bbox"] = (x0, y0, x1 - x0, y1 - y0)
                break
        else:
            cells.append({"bbox": b})
    return [c["bbox"] for c in cells
            if 25 <= c["bbox"][2] <= 31 and 25 <= c["bbox"][3] <= 31]

def rows(blobs):
    out = []
    for b in sorted(blobs, key=lambda s: (s[1], s[0])):
        if out and abs(out[-1][0][1] - b[1]) <= 5:
            out[-1].append(b)
        else:
            out.append([b])
    return sorted(out, key=lambda r: r[0][1])

all_rows = rows(squares(path))
counts = [len(r) for r in all_rows]
print(f"rows: {counts}", file=sys.stderr)
idx = None
for i in range(len(counts) - 2):
    if counts[i:i + 3] == [2, 3, 3]:
        idx = i
        break
if idx is None:
    print("SCAN FAIL: no 2/3/3 Shape-panel row sequence", file=sys.stderr)
    sys.exit(1)
kind_row = sorted(all_rows[idx], key=lambda s: s[0])
placement_row = sorted(all_rows[idx + 2], key=lambda s: s[0])
ex, ey = kind_row[1][0] + kind_row[1][2] / 2, kind_row[1][1] + kind_row[1][3] / 2
px0 = placement_row[0][0]
py1 = placement_row[0][1] + placement_row[0][3]
print(f"Ellipse kind button center: ({ex:.0f},{ey:.0f}); "
      f"checkbox icon target: ({px0 + 7:.0f},{py1 + 24:.0f})", file=sys.stderr)
print(f"{ex:.0f} {ey:.0f} {px0 + 7:.0f} {py1 + 24:.0f}")
PYEOF
)" || { echo "PROBE FAIL: the Shape panel scan"; exit 1; }
# "ex ey cbx cby" -> the two "x y" pairs (read's LAST variable swallows the
# rest of the line, so the split goes through awk).
ELLIPSE_XY="$(printf '%s\n' "$SCAN_OUT" | awk '{print $1, $2}')"
CB_XY="$(printf '%s\n' "$SCAN_OUT" | awk '{print $3, $4}')"
echo "Ellipse kind button at ($ELLIPSE_XY), checkbox icon target at ($CB_XY)"

DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove $ELLIPSE_XY click 1
sleep 1
shot 2_ellipse_kind_selected

drag() { # drag <x1> <y1> <x2> <y2> [midshot-name] — Shift held across the
  # whole gesture; with a 5th arg a screenshot is taken MID-DRAG (the live
  # preview + the measurement guides are on screen, the commit is not).
  DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove "$1" "$2"
  sleep 0.3
  DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" keydown shift
  sleep 0.3
  DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousedown 1
  sleep 0.3
  DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove "$3" "$4"
  sleep 0.5
  [ -n "${5:-}" ] && shot "$5"
  DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mouseup 1
  sleep 0.3
  DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" keyup shift
  # Park the pointer OFF the drawn shape: the crosshair sprite would
  # overpaint the ring pixels at the release point.
  DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove 1150 60
  sleep 1
}

# The feature starts ON (the testing default): the drag
# (500,300)->(700,340) must draw the diameter circle, and MID-DRAG the
# measurement guides — the dashed chord between the two points and the
# center dot at the segment's midpoint.
drag 500 300 700 340 5_guides_mid_drag_on
shot 3_diameter_circle_on

# One checkbox click turns the feature OFF: the same-kind drag must draw
# the OLD corner-anchored proportional circle — and NO guides MID-DRAG.
# The second drag runs in its OWN clean canvas region (disjoint from the
# first circle, so the pixel diff isolates it fully).
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" mousemove $CB_XY click 1
sleep 1
drag 400 450 600 490 6_guides_mid_drag_off
shot 4_diameter_circle_off

pkill -x russet
pkill -x Xvfb

python3 - "$OUT" $CB_XY <<'PYEOF'
import math
import sys
from PIL import Image

out = sys.argv[1]
cbx, cby = int(sys.argv[2]), int(sys.argv[3])
ok = True

def fail(msg):
    global ok
    ok = False
    print("FAIL -", msg)

def red_set(path):
    """Red pixels INSIDE the canvas interior (the Layers window's color
    chips sit right of the canvas at x >= 1040, the Palette's red
    well/swatches bottom-left at x < 250 — both excluded by the window)."""
    im = Image.open(path).convert("RGB")
    px = im.load()
    return {(x, y) for y in range(102, 698) for x in range(242, 1039)
            if px[x, y][0] > 150 and px[x, y][1] < 90 and px[x, y][2] < 90}

def ring(path_prev, path_now):
    """The ring the LAST drag drew: the largest cluster of red pixels NEW
    since the previous screenshot (the first shape stays on the canvas
    under the second one, and where the two rings cross they merge into
    one component — the diff isolates the new shape)."""
    old = red_set(path_prev)
    fresh = red_set(path_now) - old
    if not fresh:
        return None, set()
    seen = set()
    best = set()
    for p in fresh:
        if p in seen:
            continue
        stack = [p]
        comp = []
        while stack:
            c = stack.pop()
            if c in seen:
                continue
            seen.add(c)
            comp.append(c)
            x, y = c
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                n = (x + dx, y + dy)
                if n in fresh and n not in seen:
                    stack.append(n)
        if len(comp) > len(best):
            best = set(comp)
    xs = [p[0] for p in best]; ys = [p[1] for p in best]
    bbox = (min(xs), min(ys), max(xs) - min(xs) + 1, max(ys) - min(ys) + 1)
    return bbox, best

def near(pts, x, y, r):
    return any((x - px) ** 2 + (y - py) ** 2 <= r * r for px, py in pts)

def dist(ax, ay, bx, by):
    return math.hypot(ax - bx, ay - by)

A = (500, 300)
B = (700, 340)
M = ((A[0] + B[0]) / 2, (A[1] + B[1]) / 2)
drag_len = dist(*A, *B)

# --- screenshot 2: the checkbox row appeared under the Ellipse kind ---
# The evidence is a pixel DIFF against screenshot 1 at the icon slot (the
# layout is identical except the Ellipse selection and the new checkbox
# row, so every other window's pixels cancel out): bright pixels present in
# shot 2 and absent in shot 1 inside the icon box = the checked checkbox.
im1 = Image.open(f"{out}/1_shape_panel_rect_kind.png").convert("L")
im2 = Image.open(f"{out}/2_ellipse_kind_selected.png").convert("L")
p1, p2 = im1.load(), im2.load()
appeared = sum(
    1
    for x in range(cbx - 8, cbx + 9)
    for y in range(cby - 10, cby + 11)
    if 0 <= x < im2.size[0] and 0 <= y < im2.size[1]
    and p2[x, y] > 100 and p1[x, y] <= 100
)
print(f"   checkbox icon: {appeared} new bright pixels at ({cbx},{cby})")
if appeared < 25:
    fail(f"the checkbox icon did not appear at ({cbx},{cby}) after the Ellipse click")

def red_near(path, x, y, r):
    return near(red_set(path), x, y, r)

def dark_near(path, x, y, r, thresh=140):
    """Dark pixels in a window (the center dot's dark ring; red converts to
    ~76 gray in L, so chord dashes count as dark too — only used where the
    chord's presence/absence is checked separately)."""
    im = Image.open(path).convert("L")
    px = im.load()
    return sum(
        1
        for xx in range(x - r, x + r + 1)
        for yy in range(y - r, y + r + 1)
        if 0 <= xx < im.size[0] and 0 <= yy < im.size[1]
        and px[xx, yy] < thresh
    )

# --- screenshot 5: MID-DRAG with the checkbox ON — the measurement guides ---
# The live overlay shows the dashed chord (red dashes along the drag
# segment; the two samples sit at t=0.25 / t=0.75 of A->B, deep inside the
# live circle, ~50 px off its ring) and the center dot's dark ring at the
# segment's midpoint.
for sx, sy in [(550, 310), (650, 330)]:
    if not red_near(f"{out}/5_guides_mid_drag_on.png", sx, sy, 4):
        fail(f"no chord dash near ({sx},{sy}) mid-drag with the feature ON")
dark5 = dark_near(f"{out}/5_guides_mid_drag_on.png", int(M[0]), int(M[1]), 4)
if dark5 < 6:
    fail(f"no center-dot ring at the midpoint {M}: {dark5} dark pixels")
print(f"   ON guides: chord dashes at both samples, dot ring {dark5} dark px at {M}")

# --- screenshot 6: MID-DRAG with the checkbox OFF — no guides ---
# The second drag ran A2=(400,450)->B2=(600,490); its chord samples sit at
# t=0.5 / t=0.7 of A2->B2, ~20 px inside the live proportional ring's band —
# with the feature OFF there must be NO chord red and NO center dot there.
for sx, sy in [(500, 470), (540, 478)]:
    if red_near(f"{out}/6_guides_mid_drag_off.png", sx, sy, 4):
        fail(f"red near ({sx},{sy}) mid-drag with the feature OFF — a chord leaked")
dark6 = dark_near(f"{out}/6_guides_mid_drag_off.png", 500, 470, 4)
if dark6 != 0:
    fail(f"dark pixels near (500,470) mid-drag with the feature OFF — a center dot leaked")
print("   OFF guides: no chord dashes, no center dot at the samples")

# --- screenshot 3: checkbox ON — the diameter circle ---
bbox, pts = ring(f"{out}/1_shape_panel_rect_kind.png", f"{out}/3_diameter_circle_on.png")
if bbox is None:
    fail("no red ring on the ON screenshot")
else:
    x0, y0, bw, bh = bbox
    cx, cy = x0 + bw / 2, y0 + bh / 2
    print(f"ON: ring bbox {bbox}, center ({cx:.0f},{cy:.0f})")
    if abs(bw - bh) > 6:
        fail(f"the ring bbox must be SQUARE, got {bw}x{bh}")
    if abs(cx - M[0]) > 5 or abs(cy - M[1]) > 5:
        fail(f"the ring center ({cx:.0f},{cy:.0f}) must be the drag midpoint {M}")
    r_half = drag_len / 2
    da, db = dist(*A, cx, cy), dist(*B, cx, cy)
    if abs(da - r_half) > 8:
        fail(f"the press point is {da:.1f} from the center, expected the radius {r_half:.1f}")
    if abs(db - r_half) > 8:
        fail(f"the release point is {db:.1f} from the center, expected the radius {r_half:.1f}")
    if abs(bw - (drag_len + 8)) > 8:
        fail(f"the ring side {bw} must be the drag length {drag_len:.1f} + the stroke width 8")
    if not near(pts, *A, 3):
        fail("the press point must lie ON the ring (red at the drag start)")
    if not near(pts, *B, 3):
        fail("the release point must lie ON the ring (red at the drag end)")
    if near(pts, cx, cy, 4):
        fail("the segment midpoint is the CENTER — it must stay unpainted (outline-only paint)")

# --- screenshot 4: checkbox OFF — the old corner-anchored square ---
# The OFF drag ran A2=(400,450)->B2=(600,490): the dominant delta is
# dx=200, so the old rule grows the square A2..A2+(200,200) — its ring
# bbox (396,446) 208x208 and its center on the anchor diagonal at
# (500,550), NOT the drag midpoint (500,470).
A2 = (400, 450)
B2 = (600, 490)
M2 = ((A2[0] + B2[0]) / 2, (A2[1] + B2[1]) / 2)
bbox, pts = ring(f"{out}/3_diameter_circle_on.png", f"{out}/4_diameter_circle_off.png")
if bbox is None:
    fail("no red ring on the OFF screenshot")
else:
    x0, y0, bw, bh = bbox
    cx, cy = x0 + bw / 2, y0 + bh / 2
    print(f"OFF: ring bbox {bbox}, center ({cx:.0f},{cy:.0f})")
    if abs(bw - bh) > 6:
        fail(f"the proportional shape is still a SQUARE bbox, got {bw}x{bh}")
    if abs(cx - 500) > 5 or abs(cy - 550) > 5:
        fail(f"OFF: the ring center ({cx:.0f},{cy:.0f}) must drop to the anchor diagonal (500,550)")
    if abs(cx - M2[0]) <= 5 and abs(cy - M2[1]) <= 5:
        fail("OFF: the center must NOT sit on the drag midpoint anymore")
    if abs(bw - 208) > 8:
        fail(f"OFF: the square side {bw} must be the dominant delta 200 + the stroke width 8")
    r_off = 100.0
    if abs(dist(*A2, cx, cy) - r_off) <= 10:
        fail("OFF: the press point (the bbox corner) must lie OFF the drawn CIRCLE "
             "(on the old rule it corners the bounding square, r√2 from the center)")
    if abs(dist(*B2, cx, cy) - r_off) <= 10:
        fail("OFF: the release point must NOT be pulled onto the circle "
             "(it stays wherever the pointer ended, inside the square)")

print("PROBE PASS" if ok else "PROBE FAIL")
sys.exit(0 if ok else 1)
PYEOF
