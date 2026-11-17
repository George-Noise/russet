#!/usr/bin/env bash
# Session-start environment probe (owner request 2026-11-17): what the sandbox
# HAS and MISSES before any build/test starts, so the agent never launches a
# heavy command into the void and installs tooling knowing exactly what exists.
# Report-only: always exit 0; the verdict lines at the bottom are the input to
# the session-start decision (AGENTS.md "Session start"). Fix hints quote the
# exact commands (SANDBOX.md is the source of truth for the GUI recipe).
#
# Usage: bash scripts/sandbox_check.sh [repo-dir]   (default: the repo root)
set -u
REPO="$(cd "$(dirname "$0")/.." && pwd)"
[ "${1:-}" != "" ] && REPO="$1"
PIN=$(sed -n 's/^channel *= *"\(.*\)"/\1/p' "$REPO/rust-toolchain.toml" 2>/dev/null)
EXPORT_PATH='export PATH="$HOME/.cargo/bin:$PATH"'

say()  { printf '%s\n' "$*"; }
ok()   { printf '  [yes] %s\n' "$*"; }
no()   { printf '  [NO ] %s\n' "$*"; }

say "=== sandbox_check: $REPO ($(date '+%Y-%m-%d %H:%M %Z')) ==="

say "Base tools:"
for t in git python3 zip unzip ffmpeg Xvfb curl; do
  if command -v "$t" > /dev/null 2>&1; then ok "$t"; else no "$t"; fi
done
python3 -c "import PIL" > /dev/null 2>&1 && ok "python3 Pillow (PIL)" || no "python3 Pillow (PIL) — pip install pillow"

say "Rust toolchain (pin: ${PIN:-unknown}):"
if ! command -v cargo > /dev/null 2>&1 && [ -x "$HOME/.cargo/bin/cargo" ]; then
  no "cargo not on PATH — $EXPORT_PATH"
fi
if command -v rustup > /dev/null 2>&1 || [ -x "$HOME/.cargo/bin/rustup" ]; then
  ok "rustup present"
  RUSTUP="rustup"; command -v rustup > /dev/null 2>&1 || RUSTUP="$HOME/.cargo/bin/rustup"
  if "$RUSTUP" toolchain list 2>/dev/null | grep -q "$PIN"; then
    ok "pinned toolchain $PIN installed"
  else
    no "pinned toolchain $PIN MISSING — first cargo command in the repo fetches it (network needed)"
  fi
  "$RUSTUP" target list --installed 2>/dev/null | grep -q wasm32-unknown-unknown \
    && ok "wasm32-unknown-unknown target" \
    || no "wasm32 target missing — $RUSTUP target add wasm32-unknown-unknown"
  if "$RUSTUP" component list --toolchain "$PIN" 2>/dev/null | grep -qE "^rustfmt-.* \(installed\)" \
     && "$RUSTUP" component list --toolchain "$PIN" 2>/dev/null | grep -qE "^clippy-.* \(installed\)"; then
    ok "rustfmt + clippy components"
  else
    no "rustfmt/clippy component missing — $RUSTUP component add --toolchain $PIN rustfmt clippy"
  fi
else
  no "rustup MISSING — sh <(curl -sSf https://sh.rustup.rs) -y --profile minimal --default-toolchain none"
fi

say "Network:"
curl -s -m 5 -o /dev/null https://index.crates.io/config.json \
  && ok "crates.io reachable" || no "crates.io NOT reachable — dep builds will fail"
curl -s -m 5 -o /dev/null https://github.com \
  && ok "github.com reachable" || no "github.com NOT reachable"

say "target/ warmth:"
if [ -d "$REPO/target" ]; then
  for lane in debug gates wasm32-unknown-unknown; do
    fp=$(find "$REPO/target/$lane" -maxdepth 2 -type d -name ".fingerprint" 2>/dev/null | head -1)
    n=0; [ "$fp" != "" ] && n=$(find "$fp" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | wc -l)
    if [ "$n" -gt 50 ]; then ok "target/$lane warm ($n fingerprints)"
    elif [ "$n" -gt 0 ]; then no "target/$lane only partially warm ($n fingerprints) — expect a partial rebuild"
    else no "target/$lane empty — cold build for this lane"
    fi
  done
  [ -x "$REPO/target/debug/russet" ] && ok "GUI binary target/debug/russet present" \
    || no "GUI binary not built — cargo build --features mcp"
else
  no "target/ ABSENT — cold build everywhere (or restore the hosted russet-target.zip, see AGENTS.md 'Session start')"
fi

say "GUI/Xvfb lane (SANDBOX.md recipe):"
XKBFIX=/tmp/xkbfix/ext/usr/lib/x86_64-linux-gnu
if [ -e "$XKBFIX/libxkbcommon-x11.so" ] && [ -e "$XKBFIX/libxcb-xkb1"* ] 2>/dev/null || [ -e "$XKBFIX/libxcb-xkb.so.1" ]; then
  ok "/tmp/xkbfix libs present"
else
  no "/tmp/xkbfix MISSING (wiped on sandbox recycle) — SANDBOX.md 'apt-get download libxkbcommon-x11-0 libxcb-xkb1' block"
fi
command -v xdotool > /dev/null 2>&1 || [ -x /tmp/xdotool/ext/usr/bin/xdotool ] \
  && ok "xdotool (drag probes)" || no "xdotool absent (optional; window drags need it)"

say "Verdict: the lines above are the session-start inventory. Do not start a"
say "build/test lane whose [NO] rows you are not prepared to fix first; log any"
say "heavy command's duration into TIMING.md."
exit 0
