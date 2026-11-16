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
| V16 | 2026-11-13 | Shape tool (U): a right-button drag swaps the primary/secondary wells (like the Palette's Swap button — the swap persists) and the shape commits with the swapped wells: the former secondary in the outline, the former primary in the fill. LMB drawing and the Line/Text RMB behavior unchanged. | 2026-11-13 `UX:` entry | closed 2026-11-15 — superseded by the owner's own request: shapes no longer swap the wells at all; the replacement behavior is V19. |
| V23 | 2026-11-16 | F12-B wheel-pan parity (variant B is INTERIM, not final): ONE physical wheel notch pans the SAME fixed 15 pt on Windows, Linux Wayland and Web — plain wheel vertical, `Shift`+wheel horizontal — and the folded-in F11 ×5 feels right on Windows (the notch = the old ±3 pt × 5); touchpad/trackpad scrolling stays proportional. Owner pass 2026-11-16: Windows + Chrome approved («скорость pan через scroll устраивает»); Firefox overshot ~3× — FIXED the same day (web `Line` ÷ `WEB_LINE_UNIT_LINES_PER_NOTCH` 3 lines/notch) — re-check Firefox still owed; the Linux/Wayland leg is also open (the owner cannot test Linux yet). OPTIONAL but useful: run with `--features diagnostics` (web: `trunk serve --features diagnostics`, logs → browser console) and paste the `[diagnostics] wheel …` lines from all three platforms — the raw unit/delta values retune the interim constants (web Line 3, Point 100 pt). | 2026-11-16 `Feat:` parity entry | open — Firefox re-check + Linux pending |
| V24 | 2026-11-16 | F13-A resize: NO floating-window jitter («колбасит») while the main window is resized; Windows behavior unchanged; after the resize ends an anchored window keeps its dropped gap and a free window its relative placement. Owner pass 2026-11-16: Windows, Firefox, Chrome — clean (no jitter reported); ONLY the Linux/Wayland leg remains open (the owner cannot test Linux yet). OPTIONAL: with `--features diagnostics` the `[diagnostics] main resize …` / `[diagnostics] place …` lines pin any residual mechanics. | 2026-11-16 `Feat:` parity entry | open — Linux/Wayland only |
| V25 | 2026-11-16 | GNOME Wayland pass of the new winit Adwaita CSD frame (the owner-approved `winit` dependency, SPEC §1): the title bar shows "Untitled - Russet" / "*name - Russet" and follows New/Open/Save/dirty; a double-click on the bar maximizes and a second double-click restores; the close/minimize buttons work; the bar looks right in both light and dark shell themes. The same pass can close the Linux legs of V23 (wheel-pan feel) and V24 (resize jitter) — until this dep they were blocked: the owner had no usable Linux window bar at all. | 2026-11-16 `Fix:` Wayland CSD entry | open — needs the owner's Linux machine |

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
- V9 — verified 2026-11-16 via chat (owner: «V9 проверил»): the owner clicked through
  the web build in a browser with a real mouse — draw, zoom, pan, Open/Save; the pass
  surfaced two platform gaps, born as F12 (wheel-pan parity) and F13 (floating-window
  jitter on main-window resize) in DEBTS-functional.md — the M8 human check itself is
  done.
