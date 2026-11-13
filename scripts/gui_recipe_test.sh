#!/usr/bin/env bash
# Verified sandbox recipe for running russet (GUI + MCP) headlessly.
# See doc/sandbox-limitations.md for the full background.
# Usage: bash scripts/gui_recipe_test.sh   (no root, no args)
set -uo pipefail
cd "$(dirname "$0")/.."

export DISPLAY="${DISPLAY:-:99}"
export LIBGL_ALWAYS_SOFTWARE=1
export LD_LIBRARY_PATH="${LD_LIBRARY_PATH:+$LD_LIBRARY_PATH:}/tmp/xkbfix/ext/usr/lib/x86_64-linux-gnu"

# --- 0. One-time: fetch the two X11 libs the sandbox image lacks. ---
if [ ! -e /tmp/xkbfix/ext/usr/lib/x86_64-linux-gnu/libxkbcommon-x11.so.0 ]; then
    echo "== fetching missing X11 libs into /tmp/xkbfix (no root needed) =="
    mkdir -p /tmp/xkbfix && cd /tmp/xkbfix
    apt-get download libxkbcommon-x11-0 libxcb-xkb1 >/dev/null
    for d in *.deb; do dpkg -x "$d" ext; done
    ln -sf libxkbcommon-x11.so.0 ext/usr/lib/x86_64-linux-gnu/libxkbcommon-x11.so
    cd - >/dev/null
fi

# --- 1. Xvfb in its own session (setsid), fully detached from our pipes. ---
if ! pgrep -x Xvfb >/dev/null; then
    setsid Xvfb :99 -screen 0 1280x800x24 >/tmp/xvfb.log 2>&1 </dev/null &
    sleep 2
fi

# --- 2. One real MCP request: prove the event loop + MCP reader both run. ---
REQ='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"probe","version":"0"}}}'
printf '%s\n' "$REQ" | timeout -k 5 20 ./target/debug/russet >/tmp/russet_run.log 2>&1
echo "russet exit=$? (124/137 = had to be killed; that is fine, it is a GUI app)"

echo "--- russet MCP reply (first 3 lines) ---"
head -n 3 /tmp/russet_run.log

# --- 3. Always clean up the X server. ---
pkill -x Xvfb 2>/dev/null
echo "recipe test done"
