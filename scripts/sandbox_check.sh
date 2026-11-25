#!/usr/bin/env bash
# Session-start environment probe (owner request 2026-11-17): what the sandbox
# HAS and MISSES before any build/test starts, so the agent never launches a
# heavy command into the void and installs tooling knowing exactly what exists.
# Report-only: always exit 0; the verdict lines at the bottom are the input to
# the session-start decision (AGENTS.md "Session start"). Fix hints quote the
# exact commands (SANDBOX.md is the source of truth for the GUI recipe).
#
# Usage: bash scripts/sandbox_check.sh [--warm] [repo-dir]   (default: the repo root)
set -u
WARM=0
[ "${1:-}" = "--warm" ] && { WARM=1; shift; }
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
  no "target/ ABSENT — cold build everywhere (accepted cost: the hosted store is retired, owner decision 2026-11-17 — dev profile builds fast enough for tests)"
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
# The pitfall-11 row (owner request 2026-11-21, PROBLEMS.md): a raw Xvfb has
# no WM, so KEY/MODIFIER events need one explicit `xdotool windowfocus <id>`
# per launch — probe that the subcommand is really available, alongside the
# xdotool row above. `xdotool help` inits its xdo instance BEFORE dispatching,
# so the probe needs a reachable X display; without one it degrades to a
# reminder line instead of a false [NO].
XBIN=""
command -v xdotool > /dev/null 2>&1 && XBIN="$(command -v xdotool)"
[ -z "$XBIN" ] && [ -x /tmp/xdotool/ext/usr/bin/xdotool ] && XBIN=/tmp/xdotool/ext/usr/bin/xdotool
if [ -n "$XBIN" ]; then
  # The /tmp bundle binary needs its own lib dir on LD_LIBRARY_PATH (SANDBOX.md
  # trap 13) — without it every invocation dies in the loader and the probe
  # below would misreport a healthy xdotool as broken.
  XLDP="${LD_LIBRARY_PATH:-}"
  [ -x /tmp/xdotool/ext/usr/bin/xdotool ] \
    && XLDP="/tmp/xdotool/ext/usr/lib/x86_64-linux-gnu${XLDP:+:$XLDP}"
  XHELP_DISPLAY="${DISPLAY:-:99}"
  XHELP_OUT="$(DISPLAY="$XHELP_DISPLAY" LD_LIBRARY_PATH="$XLDP" "$XBIN" help 2>&1)"
  if printf '%s\n' "$XHELP_OUT" | grep -qE '^[[:space:]]*windowfocus[[:space:]]*$'; then
    ok "windowfocus subcommand (display $XHELP_DISPLAY) — the raw-Xvfb key-event fix (SANDBOX.md pitfall 11)"
  elif printf '%s' "$XHELP_OUT" | grep -q 'loading shared libraries'; then
    no "xdotool is BROKEN (its lib debs are missing — SANDBOX.md trap 13: apt-get download xdotool libxdo3 libxtst6 libxi6)"
  elif printf '%s' "$XHELP_OUT" | grep -q "Can't open display"; then
    say "  [--] windowfocus NOT probed (no X display $XHELP_DISPLAY — start Xvfb first; pitfall 11: one windowfocus per launch fixes key events)"
  else
    no "xdotool lacks the windowfocus subcommand — the pitfall-11 key-event fix cannot run with this xdotool"
  fi
fi

say "Gates-lane warm-up:"
if [ "$WARM" = "1" ]; then
  # Detached compilation of EXACTLY the gate's compile work: clippy (check
  # units) + `cargo test --no-run` (test binaries compiled, NOTHING executed —
  # no test binary spawns, so the mcp stdin reader cannot hang; SANDBOX.md
  # trap 10 stays impossible here without the stdin shield). The lane is hot
  # while the agent reads code and plans. In-sandbox compilation, NOT the
  # retired warm-target store (owner decision 2026-11-17).
  if [ -f /tmp/russet_warm_build.pid ] && kill -0 "$(cat /tmp/russet_warm_build.pid 2>/dev/null)" 2>/dev/null; then
    ok "warm-up build already running (pid $(cat /tmp/russet_warm_build.pid), log /tmp/russet_warm_build.log)"
  else
    rm -f /tmp/russet_warm_build.pid
    # Gate on the toolchain FIRST (PROBLEMS.md 2026-11-20/23/24 — the same lie
    # three times): a warm-up without cargo died instantly while its log still
    # claimed "finished (exit 0) — verify.sh is now hot". The verdict line
    # below also captures each rc BEFORE any $(date ...) substitution resets
    # $? (that substitution is what made the old line print exit 0).
    if ! command -v cargo > /dev/null 2>&1 && [ ! -x "$HOME/.cargo/bin/cargo" ]; then
      no "warm-up SKIPPED (no cargo on PATH and no ~/.cargo/bin/cargo — install rustup + the pinned toolchain first, then re-run --warm)"
    else
      setsid bash -c '
        export PATH="$HOME/.cargo/bin:$PATH"
        echo $$ > /tmp/russet_warm_build.pid
        echo "russet warm-up (pid $$) started $(date +%T): clippy + test --no-run on the gates lane"
        cargo clippy --profile gates --all-targets --features mcp -- -D warnings
        rc_clippy=$?
        rc_test=skipped
        if [ "$rc_clippy" -eq 0 ]; then
          cargo test --profile gates --features mcp --no-run
          rc_test=$?
        fi
        if [ "$rc_clippy" -eq 0 ] && [ "$rc_test" -eq 0 ]; then
          echo "warm-up finished $(date +%T) (clippy 0, test 0) — verify.sh is now hot"
        else
          echo "warm-up FAILED $(date +%T) (clippy rc=$rc_clippy, test rc=$rc_test) — verify.sh is NOT hot"
        fi
      ' > /tmp/russet_warm_build.log 2>&1 < /dev/null &
      ok "background warm-up started (log /tmp/russet_warm_build.log) — the log ends 'finished' ONLY when both cargo stages exit 0; a 'warm-up FAILED' line means verify.sh is NOT hot"
    fi
  fi
else
  say "  [--] pass --warm to start a detached gates-lane warm-up (clippy + test --no-run:"
  say "       the same compile work verify.sh does, nothing executed)"
fi

say "Verdict: the lines above are the session-start inventory. Do not start a"
say "build/test lane whose [NO] rows you are not prepared to fix first; log any"
say "heavy command's duration into TIMING.md."
exit 0
