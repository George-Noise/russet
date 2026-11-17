#!/usr/bin/env bash
# The DRAG-PROBE TEMPLATE (owner request 2026-11-17): a GUI feature whose
# behavior is only visible through window drags and pixel gaps gets a script
# like this one — copy it and adapt per feature. This concrete instance is the
# magnetized tool-settings panel evidence from the 2026-11-17 magnet round:
# it replays the drags under Xvfb per SANDBOX.md, with ffmpeg x11grab
# screenshots and xdotool drags for the below/above flip and the left/right
# edge alignment. Requires the SANDBOX.md fix (/tmp/xkbfix, /tmp/xdotool)
# and a built binary (`cargo build --features mcp`).
# Usage: bash scripts/gui_magnet_check.sh <path-to-russet-binary>
set -u
BIN="$1"
LIB=/tmp/xkbfix/ext/usr/lib/x86_64-linux-gnu:/tmp/xdotool/ext/usr/lib/x86_64-linux-gnu
OUT=/tmp/magnet_shots
rm -rf "$OUT" && mkdir -p "$OUT"

pkill -x Xvfb 2>/dev/null; pkill -x russet 2>/dev/null; sleep 1

start_xvfb() {
  setsid Xvfb :99 -screen 0 "${1:-1280x800}x24" >/tmp/xvfb.log 2>&1 </dev/null &
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

# 1. Default (Pencil): the panel must NOT be drawn at all.
start_xvfb 1280x800
run_russet
shot 1_pencil_no_panel
pkill -x russet; sleep 1

# 2. Line tool: Size + Mode panel below the Tools window at the gap.
run_russet --tool line
shot 2_line_below
pkill -x russet; sleep 1

# 3. Shape tool: Size + Kind + Fill/Outline.
run_russet --tool shape
shot 3_shape_below
pkill -x russet; sleep 1

# 4. Text tool: the two hint lines only.
run_russet --tool text
shot 4_text_hints
pkill -x russet; sleep 1

# 5. The flip: drag the Tools window title bar near the bottom edge. The
#    Tools window first opens LEFT_CENTER + 40/60 split; on the 800-tall
#    screen its title bar sits around y=230 (top edge ~215). Drag it to
#    (200, 760), then grab a frame.
run_russet --tool line
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" /tmp/xdotool/ext/usr/bin/xdotool \
  mousemove 60 258 mousedown 1 mousemove 80 400 mousemove 120 560 mousemove 200 760 mouseup 1
sleep 2
shot 5_flipped_above
pkill -x russet; sleep 1

# 6. The magnet follow: drag the Tools window toward the middle of the
#    RIGHT half (alignment must switch to right edges).
run_russet --tool line
DISPLAY=:99 LD_LIBRARY_PATH="$LIB" /tmp/xdotool/ext/usr/bin/xdotool \
  mousemove 60 258 sleep 0.3 mousedown 1 sleep 0.5 mousemove 200 300 sleep 0.3 mousemove 500 350 sleep 0.3 mousemove 900 400 sleep 0.5 mouseup 1
sleep 2
shot 6_right_half_align
pkill -x russet

pkill -x Xvfb
echo "SHOTS READY in $OUT"; ls -la "$OUT"
