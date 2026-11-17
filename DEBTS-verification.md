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
> - Open items are sorted: V checks first, then F debts (functional), then
>   anything else — NEWEST birth date first within each group (owner request
>   2026-11-16, AGENTS.md "Debts" rule 6). A closed row leaves the table for
>   the Resolved list in the same edit.
> - `scripts/rules.sh` reports PROGRESS.md size and live owner-checks as a
>   report-only reminder, so nothing hides.
> - Milestones (M8/M9) keep their Status-snapshot rows; only their still-open
>   human checks are mirrored here.

| # | Opened | What the human must check | Source (PROGRESS.md log) | Status |
|---|---|---|---|---|
| V28 | 2026-11-17 | The Resize-dialog pass on the owner's machine (Windows `cargo run`; optionally the browser build): (1) Canvas mode — the 3×3 anchor moves the crop/pad correctly (grow AND shrink from every corner/edge/center); (2) Image mode — Nearest/Bilinear/Bicubic/Lanczos3 differ as expected (nearest = hard pixels), the aspect lock links W/H; (3) every layer's History Tree gains ONE `resize` row printing the choices (e.g. "resize 5 canvas 800×600→1000×800 (TL)"), and jumping a layer back beyond it restores that node's NATIVE size — the bottom layer's jump resizes the canvas with it, a non-bottom one just gets clipped; (4) deleting the bottom layer over a DIFFERENT-size successor opens "Canvas Size Source" and BOTH branches behave (Keep = top-left adaptation + its own resize node; Use = the canvas takes the layer's size); a same-size successor switches silently. | 2026-11-17 `Feat:` resize modes entry | open — needs the owner's machine |
| V26 | 2026-11-16 | The flake DEV build: `nix build .#russet-dev` on the Nix machine compiles in the fast `gates` profile (the repo's gate lane: opt-level 0, no LTO — minutes faster than the release lane) AND still runs the pure unit tests (doCheck), and the produced binary installs and runs: `nix run .#russet-dev` starts the app (UI UNOPTIMIZED — functional checks, not feel); plain `nix build` / `nix run` must stay release, unchanged. | 2026-11-16 `Feat:` flake dev build entry | open — needs the owner's Nix machine |
| V8 | 2026-11-12 | MCP on Windows: `cargo build --release --features mcp`, run with `--mcp`, check the "MCP on (N)" badge. | 2026-11-12 `Feat:` MCP entry | open — parked by the owner 2026-11-13: no Windows MCP pass for now; the agent-side MCP run is the working state; reconfirmed 2026-11-13 |

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
- V16 — closed 2026-11-15 (superseded by the owner's own request: Shape RMB no longer
  swaps the wells at all; the replacement behavior was born as V19 and verified
  2026-11-16). The closed row sat in the open table; it moved here on 2026-11-16 when
  the debt-ordering rule (AGENTS.md "Debts" rule 6) landed — bookkeeping only.
- V23, V24 — verified 2026-11-16 via chat (owner: «проверил V23, V24»): the F12-B
  wheel-pan feel (one notch = the same fixed 15 pt on Windows, Firefox after the
  Line÷3 retune, Chrome, and the Linux/Wayland leg — testable only once the winit
  Adwaita CSD frame made the Linux window usable) and the F13-A resize pass (no
  floating-window jitter on Wayland, Windows unchanged). V24's pass satisfied F13's
  documented last pending condition → F13 paid in DEBTS-functional.md the same edit;
  F12 stays open as the final-design decision.
- V27, V25 — verified 2026-11-17 via chat (owner: «проверил V27, V25»): V27 — the release
  web build after the `strip` fix: the owner's local `trunk build --release` logs NO
  wasm-bindgen `target_features` warning and the release page passed the browser smoke
  (page loads, a stroke draws, zoom/pan); V25 — the GNOME Wayland pass of the winit
  Adwaita CSD frame (title text follows New/Open/Save/dirty, double-click
  maximize/restore, close/minimize buttons, light and dark shell themes). Nothing else
  rode on these two — V23/V24 and F13 were already closed 2026-11-16.
