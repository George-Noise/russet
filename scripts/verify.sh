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

echo "== cargo clippy --profile gates --all-targets -- -D warnings =="
cargo clippy --profile gates --all-targets -- -D warnings

echo "== cargo test --profile gates =="
cargo test --profile gates

# The MCP code is feature-gated: without this pair it never compiles and
# would rot unseen (owner request 2026-11-12/13: the mcp build must stay
# as green as the default one).
echo "== cargo clippy --profile gates --features mcp --all-targets -- -D warnings =="
cargo clippy --profile gates --features mcp --all-targets -- -D warnings

echo "== cargo test --profile gates --features mcp =="
cargo test --profile gates --features mcp

echo "verify.sh: ALL GATES GREEN"
