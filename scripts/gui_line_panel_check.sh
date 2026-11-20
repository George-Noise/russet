#!/usr/bin/env bash
# The LINE-PANEL PROBE (owner request 2026-11-20: the Line tool's word
# "Mode" radio became icon buttons "такие же по стилю и отступам" as the
# Shape panel's; the same day's SIZE-BLOCK pass is measured too). The
# gui_magnet_check.sh template's drag/click probes are not needed here —
# the tool comes from the CLI (`--tool line` / `--tool shape`); the
# evidence is PIXEL GEOMETRY: under Xvfb, ffmpeg screenshots of the real
# binary, and a Python measurement of the 28×28 button squares. It
# measures, on the SAME binary:
#   - the LINE panel's in-row horizontal gap (must be the Tools grid's
#     own 6 px),
#   - the SHAPE panel's in-row gaps (6 px) and its row gaps (12 px) on its
#     own screenshot — the two panels' numbers must AGREE (the "same
#     отступы" evidence) — plus the Tools window's own grid gap for
#     reference, and prints every number,
#   - the SIZE BLOCK above the button rows (owner request 2026-11-20): the
#     "Size" label glyphs sit ABOVE the slider's rail, the rail is ~96 px
#     wide (the button grid's row width), the click-to-type number box
#     paints right of the rail and the rail's center sits ~21 px above the
#     first button row (the 12-pt rows gap + the widget's lower half — the
#     exact 12 pt is pinned headless; the probe pins the real screen).
# Requires the SANDBOX.md fix (/tmp/xkbfix) and a built binary. No clicks,
# no xdotool. Exit 0 = all measured numbers in tolerance.
# Usage: bash scripts/gui_line_panel_check.sh <path-to-russet-binary>
set -u
BIN="$1"
LIB=/tmp/xkbfix/ext/usr/lib/x86_64-linux-gnu:/tmp/xdotool/ext/usr/lib/x86_64-linux-gnu
OUT=/tmp/line_panel_shots
rm -rf "$OUT" && mkdir -p "$OUT"

pkill -x Xvfb 2>/dev/null; pkill -x russet 2>/dev/null; sleep 1

start_xvfb() {
  setsid Xvfb :99 -screen 0 1280x800x24 >/tmp/xvfb.log 2>&1 </dev/null &
  sleep 2
}

shot() { # shot <name>
  ffmpeg -y -loglevel error -f x11grab -video_size 1280x800 -i :99 -frames:v 1 "$OUT/$1.png"
}

run_russet() { # run_russet <extra args...>
  DISPLAY=:99 LIBGL_ALWAYS_SOFTWARE=1 LD_LIBRARY_PATH="$LIB" \
    setsid "$BIN" "$@" >/tmp/russet_run.log 2>&1 </dev/null &
  sleep 6
}

start_xvfb
run_russet --tool line
shot 1_line_panel
pkill -x russet; sleep 1
run_russet --tool shape
shot 2_shape_panel
pkill -x russet
pkill -x Xvfb

python3 - "$OUT" <<'PYEOF'
import sys
from PIL import Image

out = sys.argv[1]

def squares(path):
    """The 28×28 button CELLS. A button is one bright rect over the dark
    window background, but the icon ink and the kind previews' inner fills
    SPLIT it into several connected components (the preview ring + its
    gray core, the emblem ink inside the selection blue). So: collect every
    bright blob of a button-ish size, CLUSTER the ones whose centers sit
    within a few pixels (they belong to one cell) and accept the clusters
    whose union bbox is a near-28 square. Text glyphs and the slider track
    stay smaller; the canvas checkerboard merges into one huge blob."""
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

    # Greedy clustering: merge blobs whose centers are within 8 px.
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
    """Group the cells into rows (a row shares its y)."""
    out = []
    for b in sorted(blobs, key=lambda s: (s[1], s[0])):
        if out and abs(out[-1][0][1] - b[1]) <= 5:
            out[-1].append(b)
        else:
            out.append([b])
    return sorted(out, key=lambda r: r[0][1])

def gaps(row):
    row = sorted(row, key=lambda s: s[0])
    return [row[i + 1][0] - (row[i][0] + row[i][2]) for i in range(len(row) - 1)]

def vgap(a, b):
    return b[0][1] - max(s[1] + s[3] for s in a)

def row_run(px, x0, x1, y, thr=60):
    """The widest horizontal bright run on row y inside [x0, x1): its width
    and [start, end). The threshold sits BELOW the button scan's 100: the
    slider's rail paints a medium gray."""
    best = 0; bs = be = None
    run = 0; s = None
    for x in range(x0, x1):
        if px[x, y] > thr:
            if s is None:
                s = x
            run += 1
        else:
            if run > best:
                best, bs, be = run, s, s + run
            run = 0; s = None
    if run > best:
        best, bs, be = run, s, s + run
    return best, bs, be

def size_block(path, panel_x0, button_top):
    """The Size block above the first button row (owner request 2026-11-20):
    the "Size" label glyphs ABOVE the slider's rail band, the rail ~96 px
    wide, the click-to-type number box right of it, and the rail CENTER
    ~21-24 px above the buttons (the 12-pt rows gap + the widget's lower
    half — the exact 12 pt is pinned by the headless test; the probe pins
    the real screen look). Brightness map of the real panel: window bg 27,
    the number-box fill AND the unfilled rail ~50, the rail's filled part
    ~60, the handle ring 100+, the canvas ~64 — so the scan threshold is 45
    and the number box separates from the rail by COLUMN COUNT (its box is
    ~19 rows tall, the rail band only ~7) and by the dark 6-pt gap between
    them."""
    im = Image.open(path).convert("L")
    px = im.load()
    X0, X1 = max(0, panel_x0 - 2), panel_x0 + 170
    Y0 = max(0, button_top - 60)
    # 1. The rail rows: rows whose widest 45-thr run is 60..130 px. The rail
    #    is ~96 (fill + unfilled merge into one run); the number box is only
    #    ~37 and sits on its own rows, the label glyphs are narrower still.
    rail = []  # (y, start, end)
    for y in range(Y0, button_top):
        w, s, e = row_run(px, X0, X1, y, 45)
        if 60 <= w <= 130:
            rail.append((y, s, e))
    if not rail:
        return None
    rail_top = rail[0][0]
    rail_bot = rail[-1][0]
    rail_s = min(r[1] for r in rail)
    rail_e = max(r[2] for r in rail)
    rail_w = rail_e - rail_s
    rail_c = (rail_top + rail_bot) / 2
    # 2. The number box: the contiguous bright column run right of the rail
    #    (the panel's dark right margin separates it from the canvas).
    mid = int(rail_c)
    dv_s = dv_e = None
    for x in range(rail_e + 3, X1):
        if px[x, mid] > 45:
            if dv_s is None:
                dv_s = x
            dv_e = x
        elif dv_s is not None:
            break
    dv_w = (dv_e - dv_s + 1) if dv_s is not None else 0
    dv_gap = (dv_s - rail_e) if dv_s is not None else None
    # 3. The "Size" label: rows above the rail band with enough bright
    #    coverage in the block's left zone (the glyphs are thin — per-row
    #    runs would split at the letter gaps, so count pixels, runs).
    label_above = False
    for y in range(max(0, rail_top - 30), rail_top - 2):
        n = sum(1 for x in range(X0, min(X1, rail_s + 45)) if px[x, y] > 45)
        if n >= 6:
            label_above = True
    return dict(rail_top=rail_top, rail_bot=rail_bot, rail_s=rail_s,
                rail_w=rail_w, rail_c=rail_c, dv_w=dv_w, dv_gap=dv_gap,
                label_above=label_above)

ok = True
for name, panel_rows in (("1_line_panel.png", [2]), ("2_shape_panel.png", [2, 3, 3])):
    path = f"{out}/{name}"
    all_rows = rows(squares(path))
    total = sum(len(r) for r in all_rows)
    print(f"== {name}: {total} cells in {len(all_rows)} rows")
    for i, r in enumerate(all_rows):
        print(f"   row {i}: {len(r)} cells at y={r[0][1]} gaps={gaps(r)}")
    # The Tools window's grid is the FIRST FOUR rows (2/2/2/1 — two
    # columns, seven tools). Everything within 140 px below its bottom row
    # is the magnetized panel (the window-to-panel gap ~40 px, the three
    # panel rows add 80; farther bright UI — the bottom-left log window —
    # is excluded).
    if len(all_rows) < 5 or [len(r) for r in all_rows[:4]] != [2, 2, 2, 1]:
        print("   FAIL: the first four rows must be the 2/2/2/1 Tools grid")
        ok = False
        continue
    grid = all_rows[:4]
    bottom = max(s[1] + s[3] for s in grid[3])
    panel = [r for r in all_rows[4:] if r[0][1] - bottom <= 200]
    if [len(r) for r in panel] != panel_rows:
        print(f"   FAIL: the panel must be {panel_rows} cells per row, got {[len(r) for r in panel]}")
        ok = False
        continue
    # The Tools grid's own in-row gaps — the reference number.
    t_gaps = [g for r in grid for g in gaps(r)]
    print(f"   Tools grid in-row gaps: {t_gaps}")
    # The panel: in-row gaps must equal the grid gap (6 px) — "ровно такой
    # же" as the Tools window's, AND the same number the Shape panel shows
    # on ITS screenshot (both measured here).
    p_gaps = [g for r in panel for g in gaps(r)]
    print(f"   panel in-row gaps: {p_gaps}")
    if not p_gaps or any(abs(g - 6) > 1.5 for g in p_gaps):
        print("   FAIL: the panel's in-row gaps must be the 6px grid gap")
        ok = False
    # The Shape panel's rows must sit a step further apart (12 px).
    if len(panel_rows) > 1:
        vg = [vgap(panel[i], panel[i + 1]) for i in range(len(panel) - 1)]
        print(f"   panel row gaps: {vg}")
        if any(abs(g - 12) > 2.5 for g in vg):
            print("   FAIL: the Shape panel's rows must sit 12px apart")
            ok = False
    # The SIZE BLOCK above the button rows (owner request 2026-11-20): the
    # label above the rail, the rail ~96 px (the button grid's row width),
    # the number box right of the rail, the rail center ~24 px above the
    # first button row (12-pt gap + the widget's lower half).
    sz = size_block(path, min(s[0] for s in panel[0]), panel[0][0][1])
    if sz is None:
        print("   FAIL: the slider's rail band not found above the button row")
        ok = False
    else:
        dist = panel[0][0][1] - sz["rail_c"]
        print(f"   size block: rail {sz['rail_w']}px wide at x={sz['rail_s']} "
              f"(rows {sz['rail_top']}..{sz['rail_bot']}), label above={sz['label_above']}, "
              f"number box w={sz['dv_w']} gap={sz['dv_gap']}, rail-center->buttons {dist:.1f}px")
        if not (93 <= sz["rail_w"] <= 99):
            print("   FAIL: the rail must be ~96px (the button grid's row width)")
            ok = False
        if not sz["label_above"]:
            print("   FAIL: the Size label must sit ABOVE the slider")
            ok = False
        if not (20 <= (sz["dv_w"] or 0) <= 50 and 3 <= (sz["dv_gap"] or -1) <= 15):
            print("   FAIL: the click-to-type number box must sit right of the slider")
            ok = False
        if not (17 <= dist <= 26):
            print("   FAIL: the slider row must sit ~24px above the buttons (12pt gap + widget half)")
            ok = False
print("PROBE PASS" if ok else "PROBE FAIL")
sys.exit(0 if ok else 1)
PYEOF
