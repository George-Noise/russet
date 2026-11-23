#!/usr/bin/env bash
# The SETTINGS + PERSISTENCE probe (owner request 2026-11-23: only the
# Mouse4/Mouse5 choice persists — SPEC §3.14). End-to-end storage audit
# under Xvfb against the REAL binary, fully hermetic (the app's data dir
# is redirected to the probe's own XDG_DATA_HOME):
#   A. fresh session: the FIRST Mouse4 press opens the modal, YES settles
#      "Use for Undo / Redo"; eframe's 30-s autosave writes app.ron with
#      EXACTLY ONE key ("settings/mouse_side_buttons": "always") and the
#      graceful File ▸ Exit keeps it;
#   B. restart: the stroke UNDOES on Mouse4 and NO modal appears (the
#      choice loaded from storage, no re-asking);
#   C. fresh session again: the modal's NO settles "Don't use" — the file
#      holds "never";
#   D. restart: Mouse4 AND Mouse5 are INERT (no undo, no redo, no modal);
#   E. the default stays a ZERO-WRITE state: an untouched session (even
#      one that presses Mouse4 and answers nothing, killed before the
#      autosave) leaves NO app.ron at all;
#   F. visual: the Settings window opens from the menu and shows the
#      default "Ask every session" choice (the combo's loaded mode) —
#      kept as the owner-viewable screenshot.
# NOTE on the dropdown: the modal's Yes/No buttons and the palette's
# picker popup react to synthetic xdotool clicks, but the Settings
# window's ComboBox button does not (an Atom-based widget; plain Buttons
# around it click fine on the SAME binary) — the popup could not be
# driven headlessly, so the dropdown's own change path is pinned by the
# headless tests (`app/settings_window/tests.rs`,
# `app/mouse_side/tests.rs`) and stays a V75 owner check.
# Requires the SANDBOX.md fix (/tmp/xkbfix, /tmp/xdotool). Exit 0 = PASS.
# Usage: bash scripts/gui_settings_check.sh <path-to-russet-binary>
set -u
BIN="$1"
LIB=/tmp/xkbfix/ext/usr/lib/x86_64-linux-gnu:/tmp/xdotool/ext/usr/lib/x86_64-linux-gnu
XD=/tmp/xdotool/ext/usr/bin/xdotool
OUT=/tmp/settings_check
rm -rf "$OUT" && mkdir -p "$OUT"
DATA="$OUT/data"

pkill -x Xvfb 2>/dev/null; pkill -x russet 2>/dev/null; sleep 1

setsid Xvfb :99 -screen 0 1280x800x24 >/tmp/xvfb.log 2>&1 </dev/null &
sleep 2

launch() { # launch [args...] — a FRESH session on the probe's data dir
  DISPLAY=:99 LIBGL_ALWAYS_SOFTWARE=1 LD_LIBRARY_PATH="$LIB" \
    XDG_DATA_HOME="$DATA" setsid "$BIN" "$@" >/tmp/russet_run.log 2>&1 </dev/null &
  sleep 8
}

xd() { DISPLAY=:99 LD_LIBRARY_PATH="$LIB" "$XD" "$@"; }

shot() { ffmpeg -y -loglevel error -f x11grab -video_size 1280x800 -i :99 -frames:v 1 "$OUT/$1.png"; }

# Keeps egui repainting past the 30-s autosave mark (an idle app runs no
# frames, and eframe's autosave fires from a frame — AGENTS.md quirk).
keep_alive() { # keep_alive <seconds>
  local end=$((SECONDS + $1)) i=0
  while [ $SECONDS -lt $end ]; do
    i=$((i + 1))
    xd mousemove $((600 + i % 20)) 700 sleep 1
  done
}

# The modal's Yes/No button centers from the screenshot: flood-fill the
# dialog body from the content center, find the TWO button blobs (the
# dialog_button_row pair), No = left, Yes = right. Prints "x_no y_no
# x_yes y_yes" or nothing when no modal body (≥240 px wide AND ≥80 px
# tall — the Tool Log window under the center line is ~200×50 and must
# NOT read as the modal) is found.
modal_buttons() {
  python3 - "$OUT/$1" <<'PY'
import sys
from PIL import Image

im = Image.open(sys.argv[1]).convert("L")
w, h = im.size
px = im.load()

def near_body(v):
    return abs(v - 27) <= 10

sx, sy = 640, 413
while sy < h and not near_body(px[sx, sy]):
    sy += 1
if sy == h:
    raise SystemExit("")
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
if not (dx1 - dx0 > 240 and dy1 - dy0 > 80):
    raise SystemExit("")  # body too small: not the modal (no modal)

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
no, yes = blobs[0], blobs[1]
print(f"{no[0] + no[2] // 2} {no[1] + no[3] // 2} "
      f"{yes[0] + yes[2] // 2} {yes[1] + yes[3] // 2}")
PY
}

click_at() { # click_at <x> <y>
  xd mousemove "$1" "$2" sleep 0.2 mousedown 1 sleep 0.15 mouseup 1
}

graceful_exit() { # File ▸ Exit — the menu item sits at (27, 141)
  xd mousemove 21 12 sleep 0.4 mousedown 1 sleep 0.15 mouseup 1
  sleep 0.8
  xd mousemove 27 141 sleep 0.4 mousedown 1 sleep 0.15 mouseup 1
  sleep 2
}

storage_value() {
  python3 - "$1" <<'PY'
import sys
try:
    text = open(sys.argv[1]).read()
except FileNotFoundError:
    print("MISSING")
    raise SystemExit
# The ron file is one flat map: { "settings/mouse_side_buttons": "…", }
import re
keys = re.findall(r'"([^"]+)":\s*"([^"]*)"', text)
for k, v in keys:
    if k == "settings/mouse_side_buttons":
        print(f"{k}={v} (all keys: {len(keys)})")
        raise SystemExit
print(f"NO_MOUSE_SIDE_KEY (all keys: {len(keys)})")
PY
}

red_on_row() { # red stroke pixels on the probe row
  python3 - "$OUT/$1" <<'PY'
import sys
from PIL import Image
im = Image.open(sys.argv[1]).convert("RGB")
row = list(im.crop((300, 300, 600, 301)).getdata())
print(sum(1 for p in row if p[0] - p[2] > 30))
PY
}

drag() { # drag <x0> <y0> <x1> <y1> — the paint gesture
  xd mousemove "$1" "$2" sleep 0.1 mousedown 1 sleep 0.1
  local n=12 i
  for i in $(seq 1 "$n"); do
    local x y
    x="$(python3 -c "print(int($1 + ($3 - $1) * $i / $n))")"
    y="$(python3 -c "print(int($2 + ($4 - $2) * $i / $n))")"
    xd mousemove "$x" "$y" sleep 0.03
  done
  xd mouseup 1 sleep 0.3
}

FILE="$DATA/russet/app.ron"
fail() { echo "SETTINGS PROBE: FAIL — $1"; pkill -x russet 2>/dev/null; pkill -x Xvfb 2>/dev/null; exit 1; }

# --- A. ask → Yes → "always" persisted (autosave + exit save) -------------
rm -rf "$DATA"
launch
xd click 8
sleep 1
shot a1_modal
BTN="$(modal_buttons a1_modal.png)"
[ -n "$BTN" ] || fail "the first Mouse4 press did not open the modal"
read -r NX NY YX YY <<<"$BTN"
click_at "$YX" "$YY"
sleep 1
shot a2_after_yes
keep_alive 24
[ -f "$FILE" ] || fail "the 30-s autosave wrote no app.ron after the Yes answer"
A_VALUE="$(storage_value "$FILE")"
echo "A: storage after autosave: $A_VALUE"
[ "$A_VALUE" = "settings/mouse_side_buttons=always (all keys: 1)" ] \
  || fail "the autosave file is not exactly the ONE 'always' key: $A_VALUE"
graceful_exit
pgrep -x russet >/dev/null && fail "File ▸ Exit did not close the app"
A_VALUE="$(storage_value "$FILE")"
echo "A: storage after the exit save: $A_VALUE"
[ "$A_VALUE" = "settings/mouse_side_buttons=always (all keys: 1)" ] \
  || fail "the exit save changed or dropped the key: $A_VALUE"

# --- B. restart: no modal, Mouse4 undoes ----------------------------------
launch --tool brush --primary FF0000FF
drag 300 300 600 300
sleep 0.5
shot b1_stroke
BASE="$(red_on_row b1_stroke.png)"
[ "$BASE" -ge 180 ] || fail "the stroke never painted ($BASE red pixels)"
xd click 8
sleep 1
shot b2_after_undo
UNDO="$(red_on_row b2_after_undo.png)"
[ "$UNDO" -eq 0 ] || fail "Mouse4 did not undo on the loaded Always mode ($UNDO red pixels left)"
[ -z "$(modal_buttons b2_after_undo.png)" ] || fail "the restart RE-ASKED (a modal appeared)"
echo "B: undo on restart without re-asking — ok"

# --- C. ask → No → "never" persisted --------------------------------------
pkill -x russet 2>/dev/null; sleep 1
rm -rf "$DATA"
launch
xd click 8
sleep 1
shot c1_modal
BTN="$(modal_buttons c1_modal.png)"
[ -n "$BTN" ] || fail "the fresh session did not ask again after a data reset"
read -r NX NY YX YY <<<"$BTN"
click_at "$NX" "$NY"
sleep 1
shot c2_after_no
keep_alive 24
[ -f "$FILE" ] || fail "the No answer wrote no app.ron"
C_VALUE="$(storage_value "$FILE")"
echo "C: storage after the No answer: $C_VALUE"
[ "$C_VALUE" = "settings/mouse_side_buttons=never (all keys: 1)" ] \
  || fail "the No answer did not settle 'never': $C_VALUE"
graceful_exit

# --- D. restart with "never": both side buttons inert ----------------------
launch --tool brush --primary FF0000FF
drag 300 300 600 300
sleep 0.5
shot d1_stroke
BASE="$(red_on_row d1_stroke.png)"
[ "$BASE" -ge 180 ] || fail "the stroke never painted ($BASE red pixels)"
xd click 8
sleep 1
shot d2_no_undo
[ "$(red_on_row d2_no_undo.png)" -eq "$BASE" ] || fail "Mouse4 UNDOED while the mode is Don't use"
[ -z "$(modal_buttons d2_no_undo.png)" ] || fail "a modal appeared while the mode is Don't use"
xd click 9
sleep 1
shot d3_no_redo
[ "$(red_on_row d3_no_redo.png)" -eq "$BASE" ] || fail "Mouse5 REDOED while the mode is Don't use"
[ -z "$(modal_buttons d3_no_redo.png)" ] || fail "a modal appeared after Mouse5"
echo "D: both side buttons inert on the loaded Never mode — ok"
pkill -x russet 2>/dev/null; sleep 1

# --- E. the default is a ZERO-WRITE state ---------------------------------
rm -rf "$DATA"
launch
keep_alive 26
[ ! -f "$FILE" ] || fail "an UNTOUCHED session wrote $(storage_value "$FILE") — the Ask default must save nothing"
echo "E1: an untouched 30-s session writes no app.ron — ok"
launch
xd click 8
sleep 1
shot e1_modal
BTN="$(modal_buttons e1_modal.png)"
[ -n "$BTN" ] || fail "the fresh session (clean data dir) did not ask"
pkill -x russet 2>/dev/null
sleep 1
[ ! -f "$FILE" ] || fail "a session killed before the autosave left $(storage_value "$FILE") behind"
echo "E2: an UNANSWERED session (killed before the autosave) writes nothing — ok"

# --- F. visual: the Settings window shows the default choice --------------
launch
xd mousemove 245 12 sleep 0.3 mousedown 1 sleep 0.15 mouseup 1
sleep 0.8
xd mousemove 259 39 sleep 0.3 mousedown 1 sleep 0.15 mouseup 1
sleep 1
shot f1_settings_window
SETTINGS_SHOWN="$(python3 - "$OUT/f1_settings_window.png" <<'PY'
import sys
from PIL import Image
im = Image.open(sys.argv[1]).convert("L")
px = im.load()
# The window is open iff the combo BUTTON (one wide 40..90-gray blob,
# 120..200 px wide — the combo's ~157 px button fill; the Close button is
# ~48 px, the background strip ~350 px, both excluded by the bound) sits
# in the centered window's area.
found = False
seen = set()
for y0 in range(340, 500):
    for x0 in range(430, 780):
        if (x0, y0) in seen or not (40 <= px[x0, y0] <= 90):
            continue
        stack = [(x0, y0)]
        seen.add((x0, y0))
        bx, by = [], []
        while stack:
            x, y = stack.pop()
            bx.append(x); by.append(y)
            for nx, ny in ((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)):
                if (nx, ny) not in seen and 430 <= nx < 780 and 340 <= ny < 500 \
                        and 40 <= px[nx, ny] <= 90:
                    seen.add((nx, ny))
                    stack.append((nx, ny))
        bw = max(bx) - min(bx) + 1
        bh = max(by) - min(by) + 1
        if 120 <= bw <= 200 and 12 <= bh <= 30 and len(bx) >= 800:
            found = True
print("yes" if found else "no")
PY
)"
[ "$SETTINGS_SHOWN" = "yes" ] || fail "the Settings window did not open from the menu"
echo "F: the Settings window opened (screenshot f1_settings_window.png)"
pkill -x russet 2>/dev/null; pkill -x Xvfb 2>/dev/null
echo "SETTINGS PROBE: PASS"
