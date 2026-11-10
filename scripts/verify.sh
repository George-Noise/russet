#!/usr/bin/env bash
# Agent done-gate (AGENTS.md "Commands"): run BEFORE recording a task as done.
# Usage: bash scripts/verify.sh   (needs cargo; adds ~/.cargo/bin to PATH itself)
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"

echo "== cargo fmt --check =="
cargo fmt --check

echo "== cargo clippy --all-targets -- -D warnings =="
cargo clippy --all-targets -- -D warnings

echo "== cargo test =="
cargo test

# Soft file-size guard (AGENTS.md "Repo facts"): report src files over 500 lines.
# app.rs is the owner-approved exception — skipped. This REPORT never fails the
# gate; relay its lines to the owner instead (warn + propose a split at task end).
echo "== file-size guard (soft, 500 lines; app.rs owner-exempt) =="
shopt -s globstar nullglob
within=1
for f in src/**/*.rs; do
    [ "$f" = "src/app.rs" ] && continue
    lines=$(wc -l < "$f")
    if [ "$lines" -gt 500 ]; then
        echo "OVER GUARD: $f ($lines lines) — warn the owner, propose a split at task end"
        within=0
    fi
done
if [ "$within" -eq 1 ]; then
    echo "OK: all src files (except owner-exempt app.rs) are within 500 lines"
fi
echo "verify.sh: ALL GATES GREEN"
