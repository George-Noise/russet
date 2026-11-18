#!/usr/bin/env bash
# Agent done-gate (AGENTS.md "Commands"): run BEFORE recording a task as done.
# Usage: bash scripts/verify.sh   (needs cargo; adds ~/.cargo/bin to PATH itself)
#
# STDIN IS CLOSED for the whole run (`exec </dev/null` below): every mcp test
# binary spawns an MCP stdin reader, and in a recycled sandbox the session's
# stdin is an open pipe/pty whose EOF never comes — the cargo <-> test-binary
# teardown hung AFTER the tests had already passed (SANDBOX.md trap 10,
# observed 2026-11-17). With stdin at /dev/null the reader sees EOF at once.
# The old `verify.sh < /dev/null` habit is obsolete; a BARE
# `cargo test --features mcp` outside this script still needs the manual pipe.
#
# Every stage prints its own seconds and the total — copy any stage over
# ~60 s into TIMING.md (one command per row, measured exactly).
set -euo pipefail
cd "$(dirname "$0")/.."

exec </dev/null
T_TOTAL=$(date +%s)

# Runs one stage under the caller's `set -e`, printing its duration on success
# (a failing stage aborts the script right after cargo's own error output).
stage() {
    local label="$1"; shift
    local t0
    t0=$(date +%s)
    echo "== $label =="
    "$@"
    echo "-- $label: $(( $(date +%s) - t0 )) s --"
}

stage "repo rules (scripts/rules.sh; no cargo needed)" bash scripts/rules.sh

export PATH="$HOME/.cargo/bin:$PATH"

if [ ! -d target/gates ]; then
    echo "(cold gates lane: target/gates is absent — this first run compiles every"
    echo " dependency for minutes; the next run takes seconds. TIMING.md has the"
    echo " last measured numbers.)"
fi

stage "cargo fmt --check" cargo fmt --check
stage "cargo clippy --profile gates --all-targets -- -D warnings" \
    cargo clippy --profile gates --all-targets -- -D warnings
stage "cargo test --profile gates" cargo test --profile gates

# The MCP code is feature-gated: without this pair it never compiles and
# would rot unseen (owner request 2026-11-12/13: the mcp build must stay
# as green as the default one).
stage "cargo clippy --profile gates --features mcp --all-targets -- -D warnings" \
    cargo clippy --profile gates --features mcp --all-targets -- -D warnings
stage "cargo test --profile gates --features mcp" \
    cargo test --profile gates --features mcp

echo "verify.sh: ALL GATES GREEN ($(( $(date +%s) - T_TOTAL )) s total)"
