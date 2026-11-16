# DEBTS-verification — checks awaiting a HUMAN

> **Why this file exists.** Verification debts are checks only the OWNER can do:
> Windows `cargo run` click-throughs, browser passes, `nix build` on a Nix machine,
> doc wording sign-offs. They used to live only as `Owner-checks` / `Owner must check`
> lines inside `PROGRESS.md` log entries, where they silently accumulated — nobody
> was reminded, and the owner forgot what is still unverified (owner request
> 2026-11-13: "спрашивать, проверено ли руками, чтобы оно не копилось").
>
> **Rules (AGENTS.md "Debts"):**
> - The agent reads this file at session start and at task end and MUST ask the
>   owner, in ONE batched list, which open items are now human-verified.
> - A verified item gets `Status: verified <date>` (and how: chat / commit check)
>   and is dropped at the next compaction; the historical record stays in the
>   `PROGRESS.md` log entry — this file only tracks what is STILL open.
> - Every new verification debt lands here in the SAME edit as the task that
>   creates it, with the date it was born.
> - `scripts/rules.sh` reports PROGRESS.md size and live owner-checks as a
>   report-only reminder, so nothing hides.
> - Milestones (M8/M9) keep their Status-snapshot rows; only their still-open
>   human checks are mirrored here.

| # | Opened | What the human must check | Source (PROGRESS.md log) | Status |
|---|---|---|---|---|
| V8 | 2026-11-12 | MCP on Windows: `cargo build --release --features mcp`, run with `--mcp`, check the "MCP on (N)" badge. | 2026-11-12 `Feat:` MCP entry | open — parked by the owner 2026-11-13: no Windows MCP pass for now; the agent-side MCP run is the working state; reconfirmed 2026-11-13 |
| V9 | 2026-11-09 | M8 web: a real-mouse pass in a browser — draw, zoom, pan, Open/Save (the 2026-11-09 Firefox check was a headless smoke only). | Status snapshot M8 | open — parked by the owner 2026-11-13: no web checks planned in the near future; reconfirmed 2026-11-13 |
| V16 | 2026-11-13 | Shape tool (U): a right-button drag swaps the primary/secondary wells (like the Palette's Swap button — the swap persists) and the shape commits with the swapped wells: the former secondary in the outline, the former primary in the fill. LMB drawing and the Line/Text RMB behavior unchanged. | 2026-11-13 `UX:` entry | closed 2026-11-15 — superseded by the owner's own request: shapes no longer swap the wells at all; the replacement behavior is V19. |

Resolved (kept until the next compaction, then dropped):

- V1, V2, V3 — verified 2026-11-13 via chat (owner: "проверено V1 V2 V3"): the six
  tools, the six UX-fix items, the §3.11 magnetize/shadow pass — all clicked through
  on Windows.
- V5, V6 — verified 2026-11-13 via chat: the owner tried the History Tree by hand and
  is satisfied for now ("руками потрогал, на текущий момент устраивает"). Improvement
  ideas may come later — those will be NEW tasks, not reopened checks.
- V10 — closed 2026-11-13, owner decision: he builds/runs on Nix periodically himself
  ("почти всё ок") — not a debt, stop reminding.
- V11 — verified 2026-11-13 via chat (About window checked on Windows).
- V12 — verified 2026-11-13 via chat, partial read: the owner skimmed the docs;
  development is stable and good, no wording fixes requested.
- V4, V7 — verified 2026-11-13 via chat (owner: «проверил V4, V7»): one `cargo run`
  each — after the toolchain pin + `gates` profile and after the file splits; behaves
  exactly as before.
- V13, V14, V17, V18, V19, V20, V21, V22 — verified 2026-11-16 via chat (owner:
  «проверил V22, V21, V20, V19, V18, V17, V14, V13»): the real-layers / History-tree
  click-through, the Layers-window width + white initial layer, the Tool Log restyle,
  the dialog button rows, the centered content-hugging dialogs, Shape RMB's inverted
  wells without a swap, the two-color history chips, the Shift-proportional shapes.
- V15 — checked 2026-11-16 via chat: the ×2 bump works, but the owner wants wheel pan
  ANOTHER ×5 faster — the follow-up is born as F11 in DEBTS-functional.md (owner:
  «запиши как сделать потом»); the check itself is closed.
