#!/usr/bin/env bash
# The SELECT-PANEL PROBE (owner request 2026-11-20: the Select tool's
# magnetized panel — the MODE row of two icon buttons and the OPS row of
# four, the same style/spacing the Shape/Line panels use). The
# gui_magnet_check.sh template's drag/click probes are not needed here —
# the tool comes from the CLI (`--tool select`); the evidence is PIXEL
# GEOMETRY: under Xvfb, ffmpeg screenshots of the real binary, and a
# Python measurement of the 28×28 button squares. It measures, on the
# SAME binary:
#   - the Tools window's grid is 2/2/2/2 (EIGHT tools — Select sits after
#     the paint trio),
#   - the Select panel's rows are 2 (mode: Rect/Lasso) + 4 (ops:
#     Copy/Cut/Paste/Crop),
#   - the panel's in-row gaps equal the Tools grid's own 6 px and the rows
#     sit the Shape panels' 12-pt step apart.
# Requires the SANDBOX.md fix (/tmp/xkbfix) and a built binary. No clicks,
# no xdotool. Exit 0 = all measured numbers in tolerance.
# Usage: bash scripts/gui_select_check.sh <path-to-russet-binary>
set -u
BIN="$1"
LIB=/tmp/xkbfix/ext/usr/lib/x86_64-linux-gnu
OUT=/tmp/select_panel_shots
rm -rf "$OUT" && mkdir -p "$OUT"

pkill -x Xvfb 2>/dev/null; pkill -x russet 2>/dev/null; sleep 1

setsid Xvfb :99 -screen 0 1280x800x24 >/tmp/xvfb.log 2>&1 </dev/null &
sleep 2
DISPLAY=:99 LIBGL_ALWAYS_SOFTWARE=1 LD_LIBRARY_PATH="$LIB" \
  setsid "$BIN" --tool select >/tmp/russet_run.log 2>&1 </dev/null &
sleep 6
ffmpeg -y -loglevel error -f x11grab -video_size 1280x800 -i :99 -frames:v 1 "$OUT/1_select_panel.png"
pkill -x russet
pkill -x Xvfb

python3 - "$OUT" <<'PYEOF'
import sys
from PIL import Image

out = sys.argv[1]

def squares(path):
    """The 28×28 button CELLS (the line-panel probe's blob+cluster scan):
    bright blobs of a button-ish size clustered by center, accepted when
    the union bbox is a near-28 square."""
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

def gaps(row):
    row = sorted(row, key=lambda s: s[0])
    return [row[i + 1][0] - (row[i][0] + row[i][2]) for i in range(len(row) - 1)]

def vgap(a, b):
    return b[0][1] - max(s[1] + s[3] for s in a)

ok = True
path = f"{out}/1_select_panel.png"
all_rows = rows(squares(path))
total = sum(len(r) for r in all_rows)
print(f"== {path}: {total} cells in {len(all_rows)} rows")
for i, r in enumerate(all_rows):
    print(f"   row {i}: {len(r)} cells at y={r[0][1]} gaps={gaps(r)}")
# The Tools window's grid: 2/2/2/2 (eight tools — Select sits after the
# paint trio).
if len(all_rows) < 6 or [len(r) for r in all_rows[:4]] != [2, 2, 2, 2]:
    print("   FAIL: the first four rows must be the 2/2/2/2 Tools grid (8 tools)")
    ok = False
else:
    grid = all_rows[:4]
    bottom = max(s[1] + s[3] for s in grid[3])
    panel = [r for r in all_rows[4:] if r[0][1] - bottom <= 200]
    if [len(r) for r in panel] != [2, 4]:
        print(f"   FAIL: the Select panel must be 2 mode + 4 ops cells, got {[len(r) for r in panel]}")
        ok = False
    else:
        t_gaps = [g for r in grid for g in gaps(r)]
        print(f"   Tools grid in-row gaps: {t_gaps}")
        p_gaps = [g for r in panel for g in gaps(r)]
        print(f"   Select panel in-row gaps: {p_gaps}")
        if not p_gaps or any(abs(g - 6) > 1.5 for g in p_gaps):
            print("   FAIL: the panel's in-row gaps must be the 6px grid gap")
            ok = False
        vg = vgap(panel[0], panel[1])
        print(f"   Select panel row gap: {vg}")
        if abs(vg - 12) > 2.5:
            print("   FAIL: the mode and ops rows must sit 12px apart")
            ok = False
print("PROBE PASS" if ok else "PROBE FAIL")
sys.exit(0 if ok else 1)
PYEOF
