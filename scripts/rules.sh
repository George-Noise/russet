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
#   5. PROGRESS.md size and stale material (owner request 2026-11-13: "чтобы
#      агент и человек не забывали"): warn when the file or its Log section
#      outgrows its budget, when RESOLVED warnings are still parked in the
#      Warnings section, and list the live log entries that still carry
#      owner checks — the open items must be surfaced and asked about (see
#      DEBTS-verification.md / DEBTS-functional.md).
#   6. Commit-title reminder (owner request 2026-11-13: the suggested title
#      was repeatedly left out of the final chat reply): print the newest
#      log entry's `Commit title:` field so the done-gate hands the exact
#      string to state verbatim in the reply (AGENTS.md "Delivery ritual");
#      warn when the field is missing.
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

echo "== SOFT (report-only): PROGRESS.md size and stale material =="
if [ ! -f PROGRESS.md ]; then
    echo "NOTE: PROGRESS.md not found (skipping progress checks)"
else
    prog_lines=$(wc -l < PROGRESS.md)
    echo "PROGRESS.md: $prog_lines lines (budget ~200 total)"
    if [ "$prog_lines" -gt 200 ]; then
        echo "OVER BUDGET: compact the Log into PROGRESS-archive.md (newest 3-5 entries stay)"
    fi
    log_lines=$(awk '/^## Log/{flag=1; next} flag' PROGRESS.md | wc -l)
    echo "Log section: $log_lines lines (budget ~150 = newest 3-5 entries)"
    if [ "$log_lines" -gt 150 ]; then
        echo "OVER BUDGET: the Log section holds more than the newest 3-5 entries"
    fi
    resolved=$(awk '/^## Warnings/{flag=1; next} /^## Log/{flag=0} flag' PROGRESS.md | grep -c 'RESOLVED' || true)
    if [ "$resolved" -gt 0 ]; then
        echo "STALE: $resolved RESOLVED warning(s) still parked in Warnings — move them to PROGRESS-archive.md"
        awk '/^## Warnings/{flag=1; next} /^## Log/{flag=0} flag' PROGRESS.md | grep 'RESOLVED' | while IFS= read -r line; do
            echo "ARCHIVE ME: $line"
        done
    fi
    owner_checks=$(awk '/^## Log/{flag=1; next} flag' PROGRESS.md | grep -cE 'Owner-checks:|Owner must check:' || true)
    echo "Live log entries still carrying owner checks: $owner_checks"
    echo "REMINDER: ask the owner which are human-verified; the live list is DEBTS-verification.md"
    commit_title=$(awk '
        /^## Log/ { inlog = 1 }
        inlog && /^### / { entry = 1 }
        entry && /^- \*\*Commit title:\*\*/ {
            sub(/^- \*\*Commit title:\*\*[ \t]*/, "")
            gsub(/`/, "")
            print
            exit
        }
    ' PROGRESS.md)
    if [ -n "$commit_title" ]; then
        echo "Commit title of the newest log entry: $commit_title"
        echo "REMINDER: state this commit title VERBATIM in the final chat reply (AGENTS.md \"Delivery ritual\")"
    else
        echo "STALE: the newest log entry has no **Commit title:** field — fill it in before delivering"
    fi
fi

if [ "$hard_fail" -ne 0 ]; then
    echo "rules.sh: FAILED — fix the HARD violations above"
    exit 1
fi
echo "rules.sh: HARD RULES GREEN (soft notes above are report-only)"
