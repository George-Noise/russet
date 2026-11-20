#!/usr/bin/env bash
# The ONLY writer of PROBLEMS.md (owner request 2026-11-20, the pure-log
# refinement): PREPENDS the entry file's bytes at the very top of the log —
# newest first — and never touches anything else. The agent never reads a
# line of PROBLEMS.md and never edits it by hand; this script is the single
# door in, so nothing in the log can be deleted or rewritten without the
# owner's explicit permission (the owner himself may edit the file freely).
#
# Integrity: the log is copied to a backup first; after the write, the
# previous content must still be present BYTE-FOR-BYTE below the new entry.
# On any mismatch the backup is restored and the script fails WITHOUT
# changing the log — a partial or destructive write is impossible.
#
# The log is a PURE log: it starts straight with its newest entry — no
# header, no template (those live in the AGENTS.md section "Agent-friction
# log — PROBLEMS.md", the only documentation of this file).
#
# Usage: bash scripts/log_problem.sh <entry-file> [target-log]
#   <entry-file>  a file holding ONE entry; prepended VERBATIM (no style
#                 enforcement — the AGENTS.md template is guidance, not a
#                 form). Keep it OUTSIDE the repo (e.g. /tmp/entry.md): a
#                 file inside the worktree would leak into the delivery
#                 artifacts (deliver.sh stages untracked files). The script
#                 prints a warning for such paths.
#   [target-log]  defaults to the repo's PROBLEMS.md; pass a path ONLY to
#                 test the script on a copy.
set -euo pipefail
cd "$(dirname "$0")/.."

ENTRY="${1:?usage: log_problem.sh <entry-file> [target-log]}"
TARGET="${2:-PROBLEMS.md}"

[ -f "$ENTRY" ] || { echo "log_problem.sh: entry file '$ENTRY' not found" >&2; exit 1; }
[ -s "$ENTRY" ] || { echo "log_problem.sh: entry file '$ENTRY' is empty — nothing to log" >&2; exit 1; }
if [ -e "$TARGET" ] && [ "$ENTRY" -ef "$TARGET" ]; then
  echo "log_problem.sh: the entry file IS the log — refusing" >&2
  exit 1
fi
ENTRY_ABS="$(realpath -m "$ENTRY")"
case "$ENTRY_ABS" in "$(pwd)"/*)
  echo "log_problem.sh: WARNING — the entry file lives INSIDE the repo ($ENTRY_ABS);" >&2
  echo "  it will leak into the delivery artifacts. Keep entries in /tmp." >&2
  ;;
esac

[ -f "$TARGET" ] || : > "$TARGET"   # a missing log starts empty (a pure log)

# Normalize the entry's tail: strip ALL trailing newlines (bash's $(cat)
# does it for free), refuse an entry that had no content at all, then write
# it back with ONE blank line before the old log — however the entry file
# happened to end.
ENTRY_NORM="$(cat "$ENTRY")"
[ -n "$ENTRY_NORM" ] || {
  echo "log_problem.sh: the entry has no content (only blank lines) — nothing to log" >&2
  exit 1
}

BACKUP="$(mktemp)"
NEW="$(mktemp)"
trap 'rm -f "$BACKUP" "$NEW"' EXIT
cp "$TARGET" "$BACKUP"

# The new log = the normalized entry + one blank line + the untouched old log.
{ printf '%s\n\n' "$ENTRY_NORM"; cat "$BACKUP"; } > "$NEW"

# Byte-exact prefix length = whatever NEW grew by over the backup.
SKIP=$(( $(wc -c < "$NEW") - $(wc -c < "$BACKUP") ))
if cmp -s <(tail -c +"$((SKIP + 1))" "$NEW") "$BACKUP"; then
  mv "$NEW" "$TARGET"
  echo "log_problem.sh: entry prepended — the log now starts with: $(head -1 "$TARGET")"
  echo "log_problem.sh: the previous content is intact below it (byte-checked)."
else
  cp "$BACKUP" "$TARGET"
  echo "log_problem.sh: INTEGRITY FAIL — the log restored untouched, nothing logged" >&2
  exit 1
fi
