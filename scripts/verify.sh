#!/usr/bin/env bash
# Agent done-gate (AGENTS.md "Commands"): run BEFORE recording a task as done.
# Usage: bash scripts/verify.sh   (needs cargo; adds ~/.cargo/bin to PATH itself)
set -euo pipefail
cd "$(dirname "$0")/.."

echo "== repo rules (scripts/rules.sh; no cargo needed) =="
bash "$(dirname "$0")/rules.sh"

export PATH="$HOME/.cargo/bin:$PATH"

echo "== cargo fmt --check =="
cargo fmt --check

echo "== cargo clippy --all-targets -- -D warnings =="
cargo clippy --all-targets -- -D warnings

echo "== cargo test =="
cargo test

echo "verify.sh: ALL GATES GREEN"
