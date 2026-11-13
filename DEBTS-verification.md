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
| V4 | 2026-11-11 | One `cargo run` on Windows after the toolchain pin + `gates` profile (the dev lane was not supposed to change). | 2026-11-11 `Tooling:` entry | open — explained in plain words 2026-11-13, awaiting the owner's word |
| V7 | 2026-11-13 | File splits (document/menu/input, paint, layout): `cargo run` behaves exactly as before (menu, hotkeys, undo/redo, tree windows, MCP badge); skim the module map at the top of `app.rs`. | 2026-11-13 `Refactor:` entry | open — explained in plain words 2026-11-13, awaiting the owner's word |
| V8 | 2026-11-12 | MCP on Windows: `cargo build --release --features mcp`, run with `--mcp`, check the "MCP on (N)" badge. | 2026-11-12 `Feat:` MCP entry | open — parked by the owner 2026-11-13: no Windows MCP pass for now; the agent-side MCP run is the working state |
| V9 | 2026-11-09 | M8 web: a real-mouse pass in a browser — draw, zoom, pan, Open/Save (the 2026-11-09 Firefox check was a headless smoke only). | Status snapshot M8 | open — parked by the owner 2026-11-13: no web checks planned in the near future |
| V13 | 2026-11-13 | Real layers + per-node PNG history on Windows `cargo run`: (1) Layers window — add, rename (double-click), visibility, reorder, delete with confirmation; (2) switching a row switches EVERY History Tree window (the old switcher in the tree toolbar is gone); (3) LEFT-click a node DOT (not its label) switches the canvas, incl. the Start root; drawing on an old node forks; (4) undo/redo on the active layer; (5) Resize keeps nodes: after resize, an old node shows the cropped/padded image; (6) Save = the composite of all visible layers. | 2026-11-13 `Feat:` entry | open — narrowed 2026-11-13: the agent-side pass RAN headless after all (`SANDBOX.md` recipe; all 3 `visual_driver.py` scenarios green, 14 checks: strokes, undo/redo pixels, forking, per-layer tree independence), so items (3) fork-on-old-node, (4) undo/redo and the per-layer independence are machine-verified; still owner-only: the real-pointer feel of (1)+(2)+(3) — dot hit-areas, double-click rename, row switching |
| V14 | 2026-11-13 | Layers-window width fix + the white initial layer on Windows `cargo run`: the Layers window is compact and fits the main window (also after dropping it near an edge, §3.11); the canvas is white at startup AND after New (any dialog size); "+ Add layer" layers stay transparent (checkerboard shows through); long layer names clip instead of running under the row buttons. | 2026-11-13 `Fix:` entry | open — machine-verified headless (driver 15 checks incl. the fresh-white check; Xvfb screenshot shows the ~240 pt window); awaiting the owner's Windows pass |

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
