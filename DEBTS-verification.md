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
| V1 | 2026-11-10 | Six new tools, first iteration, on Windows `cargo run`: Eyedropper LMB/RMB + continuous sampling; Line straight/curve gestures (Enter/Esc/Backspace); Shape rect/ellipse + Fill, Esc mid-drag; Text commit == preview and sharp at 100% zoom; History log lists every action and survives reopen; Layers shows one live Background row; undo/redo for every new tool; hotkeys I/L/U/T muted while typing. | 2026-11-10 `Features:` entry | open |
| V2 | 2026-11-10 | Six UX-fix items: History opens below the menu bar (0.5rem gap); Layers does not cover the zoom %; Text/Curve hints stay narrow; ellipse preview radius == committed radius (Fill and Outline+Fill); Fill/Outline never both off, outline=primary / fill=secondary; line/curve previews have round caps like the commit. | 2026-11-10 `UX-fix:` entry | open |
| V3 | 2026-11-11 | History/Layers magnetize per §3.11 and follow resizes like Tools/Palette; Tools first open at the ~40/60 split; no shadows under floating windows/dialogs (the color-picker popup intentionally still has one — decision tracked as F7 in DEBTS-functional.md). | 2026-11-11 `UX-fix:` entry | open |
| V4 | 2026-11-11 | One `cargo run` on Windows after the toolchain pin + `gates` profile (the dev lane was not supposed to change). | 2026-11-11 `Tooling:` entry | open |
| V5 | 2026-11-12 | History Tree windows on Windows: draw, Ctrl+Z/Y, Clone, layer switch, direction toggle, debug "−"; confirm the anchored scrolling feels right. | 2026-11-12 `Extra:` entry | open |
| V6 | 2026-11-13 | History Tree look after the rendering rework + lanes rework (supersedes the V5 look checks): connector shape, lane order, per-row labels; many undos+strokes at one node and horizontal scrolling; "New at top" re-check (sign logic unchanged, not re-captured). | 2026-11-13 two `Fix:` entries | open |
| V7 | 2026-11-13 | File splits (document/menu/input, paint, layout): `cargo run` behaves exactly as before (menu, hotkeys, undo/redo, tree windows, MCP badge); skim the module map at the top of `app.rs`. | 2026-11-13 `Refactor:` entry | open |
| V8 | 2026-11-12 | MCP on Windows: `cargo build --release --features mcp`, run with `--mcp`, check the "MCP on (N)" badge. | 2026-11-12 `Feat:` MCP entry | open |
| V9 | 2026-11-09 | M8 web: a real-mouse pass in a browser — draw, zoom, pan, Open/Save (the 2026-11-09 Firefox check was a headless smoke only). | Status snapshot M8 | open |
| V10 | 2026-11-09 | Nix: `nix build` / `nix run` on an actual Nix machine. | Status snapshot Extra row | open |
| V11 | 2026-11-09 | About window on Windows: Help → "About Russet" shows the correct Platform/Renderer lines. | Status snapshot Extra row | open |
| V12 | 2026-11-12 | Doc sign-offs (read once): the dependency approval flow wording (AGENTS.md / SPEC §6 / CONTRIBUTING.md); CONTRIBUTING.md reads correctly for an outsider; README command blocks render correctly on Windows PowerShell vs bash. | 2026-11-12 `Docs:`, 2026-11-11 `CI+Docs:`, 2026-11-09 `Docs:` entries | open |

Resolved (kept until the next compaction, then dropped):

- (none yet)
