#!/usr/bin/env bash
# Machine-readable repo rules — the grep-level subset of AGENTS.md.
# Single source of truth: run locally (scripts/verify.sh calls it first) and
# in CI (.github/workflows/ci.yml, job "rules"). Needs NO Rust toolchain.
#
# HARD rules fail the run (exit 1):
#   1. No inline test modules ("mod tests {") in src/ — tests live in sibling
#      tests.rs files (AGENTS.md "Tests" rule 1; the required form there is
#      `#[cfg(test)] mod tests;` + `<module>/tests.rs`).
#   2. No unwrap() anywhere in src/, tests included — AGENTS.md "Tests" rule 6
#      ("unwrap() forbidden everywhere") and "Repo facts" (panics are
#      forbidden; report errors via error_popup instead).
# SOFT rules are REPORT-ONLY (never fail the run):
#   3. expect() outside test files — allowed only off user-data paths
#      (AGENTS.md "Repo facts"): boot-time lookups (no window/document yet)
#      and internal invariants ("checked above") pass; file open/parse/decode
#      of user data must use error_popup, not a panic. Each hit is listed so
#      new ones are visible in every review and CI run.
#   4. src files over 500 lines (app.rs is the owner-approved exception) —
#      AGENTS.md "Repo facts" file-size guard.
set -u
cd "$(dirname "$0")/.."

hard_fail=0

echo "== HARD: no inline test modules in src/ (tests live in <module>/tests.rs) =="
while IFS= read -r hit; do
    echo "VIOLATION: $hit"
    hard_fail=1
done < <(grep -rnE --include='*.rs' 'mod[[:space:]]+tests[[:space:]]*\{' src || true)
if [ "$hard_fail" -eq 0 ]; then
    echo "OK"
fi

echo "== HARD: no unwrap() anywhere in src/ (incl. tests) =="
unwrap_fail=0
while IFS= read -r hit; do
    echo "VIOLATION: $hit"
    unwrap_fail=1
done < <(grep -rnE --include='*.rs' '[.]unwrap\(' src || true)
if [ "$unwrap_fail" -eq 0 ]; then
    echo "OK"
fi
if [ "$unwrap_fail" -eq 1 ]; then
    hard_fail=1
fi

echo "== SOFT (report-only): expect() outside test files =="
expect_hits=0
while IFS= read -r hit; do
    echo "NOTE: $hit"
    expect_hits=$((expect_hits + 1))
done < <(grep -rnE --include='*.rs' --exclude='tests.rs' '[.]expect\(' src || true)
echo "expect() outside tests: $expect_hits site(s) — each must stay off user-data paths"

echo "== SOFT (report-only): file-size guard, 500 lines (app.rs owner-exempt) =="
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

if [ "$hard_fail" -ne 0 ]; then
    echo "rules.sh: FAILED — fix the HARD violations above"
    exit 1
fi
echo "rules.sh: HARD RULES GREEN (soft notes above are report-only)"
