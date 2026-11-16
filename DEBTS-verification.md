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
| V13 | 2026-11-13 | Real layers + per-node PNG history on Windows `cargo run`: (1) Layers window — add, rename (double-click), visibility, reorder, delete with confirmation; (2) switching a row switches EVERY History Tree window (the old switcher in the tree toolbar is gone); (3) LEFT-click a node DOT (not its label) switches the canvas, incl. the Start root; drawing on an old node forks; (4) undo/redo on the active layer; (5) Resize keeps nodes: after resize, an old node shows the cropped/padded image; (6) Save = the composite of all visible layers. | 2026-11-13 `Feat:` entry | open — narrowed 2026-11-13: the agent-side pass RAN headless after all (`SANDBOX.md` recipe; all 3 `visual_driver.py` scenarios green, 14 checks: strokes, undo/redo pixels, forking, per-layer tree independence), so items (3) fork-on-old-node, (4) undo/redo and the per-layer independence are machine-verified; still owner-only: the real-pointer feel of (1)+(2)+(3) — dot hit-areas, double-click rename, row switching (owner 2026-11-13: «не сейчас») |
| V14 | 2026-11-13 | Layers-window width fix + the white initial layer on Windows `cargo run`: the Layers window is compact and fits the main window (also after dropping it near an edge, §3.11); the canvas is white at startup AND after New (any dialog size); "+ Add layer" layers stay transparent (checkerboard shows through); long layer names clip instead of running under the row buttons. | 2026-11-13 `Fix:` entry | open — machine-verified headless (driver 15 checks incl. the fresh-white check; Xvfb screenshot shows the ~240 pt window); awaiting the owner's Windows pass (owner 2026-11-13: «не сейчас») |
| V15 | 2026-11-13 | Wheel pan is twice as fast: plain wheel (vertical) AND Shift+wheel (horizontal) move the view ~2× the former speed (`SCROLL_PAN_SPEED` 1.5 → 3.0). Owner-REQUESTED check («записать в долг проверить»). | 2026-11-13 `UX:` entry | open |
| V16 | 2026-11-13 | Shape tool (U): a right-button drag swaps the primary/secondary wells (like the Palette's Swap button — the swap persists) and the shape commits with the swapped wells: the former secondary in the outline, the former primary in the fill. LMB drawing and the Line/Text RMB behavior unchanged. | 2026-11-13 `UX:` entry | closed 2026-11-15 — superseded by the owner's own request: shapes no longer swap the wells at all; the replacement behavior is V19. |
| V17 | 2026-11-13 | Tool Log window (renamed from "History", View-menu checkbox too): the entry list's max height is halved (280 → 140 pt), entries render at 0.5rem = 8 pt with zero row spacing, first-open is bottom-center with the bottom edge FLUSH (gap 0) and it stays flush through main-window resizes until moved. | 2026-11-13 `UX:` entry | open |
| V18 | 2026-11-13 | Dialog button rows follow IBM/Apple/Google: right-aligned, the affirmative button rightmost, Cancel to its left — New, Resize Canvas, Unsaved Changes, Delete Layer, Error, About's Close, Text tool's Place/Cancel. | 2026-11-13 `UX:` entry | open |
| V19 | 2026-11-15 | Shape tool (U), new rule: a RIGHT-button drag USES the wells inverted (former secondary → outline, former primary → fill) and the Palette wells stay UNCHANGED afterwards (no swap, no well flips after drawing). LMB drawing and the Line/Text RMB behavior unchanged. Machine-verified: 3 unit tests in `app/preview/tests.rs` (pure well rule) + a headless real-input pass (XTEST right-button drag: the committed node carries the former SECONDARY color, the Palette wells read back black/white — unchanged). Awaiting the owner's Windows pass. | 2026-11-15 `Fix:` entry | open |
| V20 | 2026-11-15 | Every modal dialog (New, Resize Canvas, Unsaved Changes, Delete Layer, Error, About): arises CENTERED on the main window on EVERY open (not slightly right/below — the owner's complaint), also re-centered when reopened after the user dragged it aside; its height hugs the content (no more stretch down to the main-window bottom edge with the buttons detached from the fields — the owner's two screenshots); stays draggable while open. Machine-verified: 3 unit tests in `app/dialogs/tests.rs` (headless egui passes: centered, compact, recenter-after-move+close) + the scripted 9-check pass through the real binary (open → title-bar drag away → OK closes → reopen centered again); awaiting the owner's Windows pass. | 2026-11-15 `Fix:` entry | open |

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
