# PROGRESS — Russet

> **Why this file exists.** `.git` reaches the agent only when the owner includes it in the
> delivery, and it is gone once the agent sandbox is recycled. Git is therefore NOT a
> progress channel in this setup. **This file is the substitute for the git history**: read
> it before starting work, append to it after every finished, verified task
> (see AGENTS.md "Git & progress log" for the exact routine). Entries older than the
> newest 3–5 live in `PROGRESS-archive.md` (grep it for history).

## Status snapshot — update in the same edit as a log entry

| Milestone | Status | Notes |
|---|---|---|
| M0 Scaffold | done | |
| M1 Canvas + zoom/pan | done | pan: wheel / Shift+wheel / middle / Space+LMB / WASD (physical keys) |
| M2 Dirty-region texture updates | done | |
| M3 Tools + history | done | |
| M4 Menu + windows + §3.11 placement | done | §3.11 reworked once — see PROGRESS-archive.md, 2026-11-09 "magnetism rework" |
| M5 Files (native, Windows) | done | |
| M6 Undo/Redo + hotkeys | done | |
| M7 Resize Canvas + New | done | |
| M8 Web (WASM) | built + headless Firefox check (2026-11-09) | wasm32 check/clippy green; `trunk build --release` OK (wasm 5.0 MB + 77 KB JS; page `web/index.html` + `Trunk.toml`); `trunk serve` + Firefox: UI renders, canvas draws, WebGL2 — a headless smoke test, not a human pass. Owner must click through drawing/zoom/pan/Open/Save with a real mouse |
| M9 Acceptance + release | pending (owner) | §7 checklist is manual, on the owner's Windows machine; `cargo build --release` is run by the owner |
| Extra: CLI launch args + owner-unconfirmed scenario markers (owner 2026-11-17) | done — gates green | native `--primary/--secondary/--tool/--canvas-size/--help` (`app/launch.rs`, parsed before the window opens; invalid values warn, defaults stay; wasm takes none); the 8 suite scenarios carry "NOT YET CONFIRMED" markers + a header ledger (the owner approves one by one); V29/V30 born |
| Extra: repo-wide English-only owner quotes (owner 2026-11-18) | done — docs only | every verbatim Russian owner quote across docs/scripts/comments translated to English (meaning kept); PROGRESS Log compacted — 2 entries moved to PROGRESS-archive.md |
| Extra: debt F14 — in-house windows later (owner 2026-11-18) | done — docs only | egui::Window → an in-house `egui::Area` frame recorded in DEBTS-functional.md from the owner's pros/cons chat review; owner-gated, no code touched |
| Extra: tool icon buttons + two-column grid (owner 2026-11-18) | done — gates green + visual driver | 1.5rem icon buttons via a cached tiny-skia atlas (`app/tool_icons.rs`), colored pencil/brush/eraser/pipette + mono line/shape/text through ONE pipeline, name+key hover tooltips, the pipette as the Eyedropper's canvas cursor; the two-column grid is debt F15; the driver screenshot caught the dead-texture trap (handles kept in the Atlas now — AGENTS.md quirks) |
| Extra: fully unoptimized dev profile (owner 2026-11-18) | done — cargo check green on the new lane | `[profile.dev]` opt-level 0 + no debuginfo, the `[profile.dev.package."*"] opt-level 2` dep override REMOVED (the "deps optimized once for responsive `cargo run`" policy retired — development is sandbox-only now, UI lag accepted); flake `russet-dev` already ran the unoptimized gates lane (untouched); SPEC §1 + AGENTS.md synced; owner visual pass = V34 |
| Extra: verify.sh hardening (owner 2026-11-18) | done — FULL gate green on a cold lane (191 s) | `exec </dev/null` closes stdin for the whole run (SANDBOX.md trap 10 fixed AT THE SOURCE — the mcp test stage exits in seconds instead of hanging); per-stage seconds + total printed (TIMING.md rows are copied now, not re-measured); cold-lane hint when `target/gates` is absent; the mcp pair stays UNCONDITIONAL (hot gate ≈ 15 s, zip sandboxes have no git, the gate never guesses) |
| Extra: brush + pipette icons redrawn + equal grid gaps (owner 2026-11-18) | done — hot gate 14 s + driver green | round volumetric bristle belly + crimped ferrule (the wedge read as a second pencil); tapering glass cone + round rubber bulb via a new `arc_pts` helper (the rectangle stick read as a pen); Tools-grid H-gap 18 pt → 6 pt = V-gap (egui Grid floors columns at `interact_size.x` 40 pt — `.min_col_width` fix, AGENTS.md quirk); tool_icons.rs 610 lines — the draw/atlas split proposed (owner-gated) |
| Extra: --warm warm-up + session rules; debts ask = 4 newest; V33 closed (owner 2026-11-18) | done — live cold-lane test + hot gate | `sandbox_check.sh --warm` = probe + detached gates-lane compile (clippy + test --no-run, nothing executed — the mcp stdin trap impossible) with a double-start pid guard; session start = `sandbox_check.sh --warm` FIRST; wasm ad-hoc checks only for web-facing changes; the batched debts ask lists only the 4 newest open items (AGENTS.md Debts rule 7) |
| Extra: permanent #[ignore] icon-dump + F16 rust-cache debt (owner 2026-11-18) | done — dump live-tested, gate skips it | `cargo test --profile gates -- --ignored icons_dump` = 14 PNGs (7 tools × 256/24 px) into /tmp/icon_dump, BLACK raster keeps the monochrome icons legible; plain gate = 156/168 passed + 1 ignored; rust-cache (`Swatinem/rust-cache@v2` after checkout) recorded as postponed F16 with the mechanism + limits written out, ci.yml untouched |
| Extra: icons v3 + Theme-button v2 (owner pass 2026-11-18) | done — contact sheet + FULL gate + driver | tool buttons keep the LIGHT-theme fills in both themes (`egui::Visuals::light()` in `windows.rs::tool_button`) → the outline is the ONE dark rim everywhere (v2 light rim reverted, tested); pipette glass = cylinder + cone only at the tip, wider collar ring; menu-bar button just "Theme" before Help (Help always last), System radio plain; MCP `set_theme` unchanged |
| Extra: icons v4 polish + title-bar-less Tools (owner pass 2026-11-18) | done — contact sheet + FULL gate | eraser rim dark plum (in the set's style); buttons' rest/hover fills dimmed −14/channel (selection stays bright); mono ink FIXED `TOOL_INK` gray 64 → icons fully theme-independent → atlas key = ppp only (the theme re-raster gone; the atlas test reworked: a theme switch uploads only egui's font atlas); Tools window `title_bar(false)` — id kept via `tools_window_id()`, drag falls back to Anywhere, close via View ▸ Tools; V37 verified, V36 → v4 re-check pending |
| Extra: Tools window v5 — the bar back, only the label hidden (owner pass 2026-11-18) | done — gate + driver + magnet check | `Window::new("")` + the explicit `tools_window_id()` renders the title bar WITHOUT the text (egui derives the Id from the title even when empty); `.open` plumbing restored (the ✕ is back); AGENTS quirk rewritten (the empty-title recipe); .gitignore keeps `__pycache__`/`*.pyc` out of the artifacts; V36 → v5 re-check pending |
| Extra: language rule → English-only whitelist (owner request 2026-11-18) | done — gate + deliver | the stamp's "Zero Cyrillic repo-wide" reworded: a language WHITELIST (English is the only written language), not a script blacklist — friendlier to readers of any writing system |
| Extra: Tools panel hides while collapsed (owner 2026-11-19) | done — gates + click probe green | the magnetized settings panel hides while the Tools window is collapsed to its title bar and returns on expand (`windows.rs::settings_panel_shown` on `Window::show`'s `inner.is_some()`); the header-width PROPOSAL delivered (measured options, no header code change — the owner picks); V38 born |

**Verification stamp:** FULL gate re-verified 2026-11-19 (hot lane): fmt clean, rules.sh
HARD-green, clippy `-D warnings` green (default + mcp), `cargo test --profile gates`
162 default / 175 mcp green (+1 ignored each — the permanent #[ignore] icon dump) with
+1 collapse-panel regression test; the `scripts/gui_collapse_check.sh` click probe
(expanded→collapsed→re-expanded, PIL-located buttons) ALL GREEN under Xvfb —
Rust 1.99.1 pinned via rust-toolchain.toml, Linux. The bare-`cargo test --features mcp`
STDIN TRAP fired once this session (an mcp stage hung 3 min — SANDBOX.md trap 10; only
verify.sh's `exec </dev/null` shield makes that lane safe to run by hand).
English-only repo-wide — a language WHITELIST (English is simply the only written language
here: comments, docs, commit titles), not a script blacklist. Web/trunk lanes last verified
2026-11-16 (see the archived log history);
every src file ≤ 500 EXCEPT the report-only owner-deferred `history_tree.rs` 569 AND
`app/tool_icons.rs` 673 (grew in the 2026-11-18 icon rounds; the natural split is the icon
draw fns vs the atlas/cursor pipeline — re-proposed to the owner at the 2026-11-19 delivery,
not done without his approval).

Update this stamp in the same edit as a log entry (AGENTS.md "Git & progress log").

## Warnings — open issues deliberately NOT fixed yet

- 2026-11-13 — No active warnings. Resolved items moved to PROGRESS-archive.md;
  open HUMAN checks live in DEBTS-verification.md, open functional debts in
  DEBTS-functional.md (the agent must ask about them per AGENTS.md "Debts").

## Log — newest first. Insert a new entry directly under this heading.

> Budget: ≤ ~20 lines per entry, fixed fields (What / Files / Verified / Owner-checks /
> Commit title); durable facts move to their canonical home in the same edit (AGENTS.md
> "Git & progress log"). Entries older than the newest 3–5 are moved verbatim to
> PROGRESS-archive.md. Open human checks live in DEBTS-verification.md — no entry is
> ever pinned in place by a pending check. Compacted 2026-11-18: the panels-split and
> docs rounds moved to PROGRESS-archive.md; later the same day the F14 + quotes rounds
> followed them (the verify.sh round needed the budget back), the tool-icons entry
> joined them in the evening (the icon-redraw round), the About entry followed
> it (V33's closure freed it — the tooling round's budget), the unoptimized-dev
> entry followed that (the dump round's budget), the verify.sh-hardening entry
> completed the day's compactions (the icons-v2 round's budget), the
> icon-redraw entry moved during the icons-v4 round's budget, and the
> Tooling entry moved during the v5 round's budget; the icon-dump entry
> moved during the wording round's budget, and the icons-v2 entry moved
> during the 2026-11-19 collapse round's budget.

### 2026-11-19 — `Fix:` the settings panel hides while the Tools window is collapsed — owner request

- **What:** two asks. (1) IMPLEMENTED: when the Tools window is collapsed to its title bar
  (the strip's ▼ button or a double-click), the magnetized settings panel now hides with it
  and returns the frame the window is expanded again — the guard is the pure
  `app/windows.rs::settings_panel_shown(tools_body_shown, tool)` fed with `Window::show`'s
  `response.inner.is_some()` (egui reports `inner: None` exactly while the collapsing body is
  closed; the collapse state is stored every frame under `area_id.with("collapsing")`), so the
  ✕ and View ▸ Tools are unaffected. (2) PROPOSED, not implemented: the header-width options
  (the strip paints 85.4 pt wide vs the 76-pt window — the ✕ lands half outside; root cause:
  `title_ui`'s zero-width advance in auto-sized windows — AGENTS quirks) — the owner picks.
- **Files:** `src/app/windows.rs` (the guard + the builder extracted as `tools_window_builder`),
  `windows/tests.rs` (+1 collapse regression test: the pure decision both ways + the egui
  semantic pinned on the real builder, expanded vs seeded-collapsed), `scripts/gui_collapse_check.sh`
  (NEW click probe: PIL-locates the strip icons, asserts panel present→absent→present),
  `SPEC.md` §3.2, `AGENTS.md` (+2 quirks: the title-strip overflow mechanics + collapse
  semantics incl. the pivot re-anchor on size change), `DEBTS-verification.md` (V38 born),
  PROGRESS (compaction: the icons-v2 entry → archive; snapshot + entry + stamp), TIMING
  (cold warm-up row; the table trimmed to 5).
- **Verified:** FULL hot gate ALL GATES GREEN 9 s (162 default / 175 mcp, +1 ignored each);
  `gui_collapse_check.sh` EXIT=0 under Xvfb (clicks at PIL-located positions); the bare
  `cargo test --features mcp` STDIN trap fired once (hang, no damage — verify.sh's
  `exec </dev/null` shield is mandatory for that lane).
- **Owner-checks:** V38 born (the collapse pass on Windows). Open: V38, V36 (v5 re-check
  pending), V30 (parked), V8 (parked).
- **Commit title:** `Tools window: hide the magnetized settings panel while collapsed`

### 2026-11-18 — `Docs:` the language rule reworded — an English whitelist, not a script blacklist — owner request

- **What:** the owner asked to reword the stamp's language rule: "Zero Cyrillic repo-wide" read as
  hostile toward people whose writing systems use Cyrillic. The rule is now a WHITELIST: English is
  the only language written in the repo (comments, docs, commit titles) — nothing is "banned",
  English is simply what is used. One line in the stamp; no other rule text referenced the old
  wording (grep: PROGRESS.md only; AGENTS.md's separate "user-facing strings are hardcoded English"
  product rule is untouched).
- **Files:** `PROGRESS.md` (stamp line; compaction: the icon-dump entry → archive; snapshot + this
  entry + quote).
- **Verified:** no src/ change — the hot gate below re-run for the ritual (161 default / 174 mcp);
  the delivery rebuilt (byte-compares MATCH), the zip carries the new stamp.
- **Owner-checks:** open: V36, V30 (parked), V8 (parked).
- **Commit title:** `Reword the language rule as an English-only whitelist (stamp)`

### 2026-11-18 — `Fix:` Tools window v5: the bar is back, only the label hidden — owner pass

- **What:** the owner corrected the v4 window change: the whole bar had vanished while only
  the title TEXT had to go. Fix: the title is an EMPTY STRING — `title_ui` builds the bar
  (button atoms + drag strip) regardless of the text (`window.rs`), so the bar renders
  unchanged minus the label; the Id stays EXPLICIT (`tools_window_id()` — egui derives it
  from the title, an empty one included; `Window::id` doc: "Required if the title changes,
  or is shared with another window"); the ✕ is back via the restored `.open` plumbing
  (plus View ▸ Tools). AGENTS quirk rewritten (the empty-title recipe; the
  `title_bar(false)` semantics kept as general knowledge); SPEC §3.2 updated. Also carried
  from the interrupted round: `__pycache__/` + `*.pyc` into .gitignore (the owner deletes
  the stray mcp_smoke.pyc by hand for now, but the next artifact must not re-add it; the
  cache leaks from visual_driver.py's `from mcp_smoke import Russet`).
- **Files:** `src/app/windows.rs` (empty title, open plumbing), `SPEC.md` (§3.2),
  `AGENTS.md` (quirk v5), `.gitignore`, `DEBTS-verification.md` (V36 → v5), `PROGRESS.md`
  (compaction: the Tooling entry → archive; snapshot + entry + stamp).
- **Verified:** FULL hot gate ALL GATES GREEN 15 s (161 default / 174 mcp); headless visual
  driver ALL SCENARIOS GREEN; gui_magnet_check EXIT=0 (the geometry re-measured with the
  bar back); the fresh zip carries ZERO `__pycache__` entries (grep -c = 0).
- **Owner-checks:** V36 → "v3 approved; v4 bar over-removed, v5 label-only delivered —
  re-check pending". Open: V36, V30 (parked), V8 (parked).
- **Commit title:** `Tools window: keep the title bar, hide only the label (empty title + explicit id)`

### 2026-11-18 — `Fix:` icons v4 (eraser rim, dimmed buttons, fixed ink) + title-bar-less Tools — owner pass

- **What:** the owner APPROVED the v3 pass ("much better now") and ordered four
  leftovers: (1) the ERASER's outline joined the set's style — dark plum (92,52,66)
  instead of the light pink that blended with the faces; (2) the tool buttons' REST/HOVER
  fills dimmed A STEP (−14/channel: gray 230 → 216 rest, 220 → 206 hover — the full
  light-theme button contrasted hard against BOTH themes' window backgrounds; the
  selection fill keeps the bright blue); (3) the MONO icons' ink FIXED — `TOOL_INK`
  gray 64 (the light theme's text gray 80, a bit darker), no white/black flip on theme
  switch → the icons are FULLY theme-independent → the atlas key dropped to just
  `pixels_per_point` (the theme re-raster died with its reason; the atlas test reworked:
  a theme switch uploads only egui's theme-dependent font atlas, never our 7 — AGENTS
  quirk); (4) the Tools window has NO TITLE BAR (`title_bar(false)` — the header was
  wider than the tiny window): egui derives the Id from the title even with the bar
  hidden (the explicit `tools_window_id()` keeps the identity — AGENTS quirk),
  `WindowDrag::TitleBar` falls back to Anywhere (still draggable), the ✕ is gone →
  View ▸ Tools closes.
- **Files:** `src/app/tool_icons.rs` (TOOL_INK, eraser rims, key/ensure), 
  `tool_icons/tests.rs` (atlas test reworked, mono-ink test, eraser-rim asserts),
  `src/app/windows.rs` (dim + title_bar(false)), `SPEC.md` (§3.2), `AGENTS.md` (+2
  quirks), `DEBTS-verification.md` (V37 verified; V36 → v4), `PROGRESS.md` (compaction:
  the icon-redraw entry → PROGRESS-archive.md; snapshot + entry + stamp).
- **Verified:** the eraser iterated on the dump contact sheet (reads as the set);
  FULL hot gate below (161 default / 174 mcp); headless visual driver ALL SCENARIOS
  GREEN (regression).
- **Owner-checks:** V37 verified (owner approved the look). V36 → "v3 approved; v4
  polish delivered — re-check pending". Open: V36, V30 (parked), V8 (parked).
- **Commit title:** `Icons v4: eraser rim in the set, dimmed buttons, fixed mono ink (theme-independent atlas), title-bar-less Tools`

### 2026-11-18 — `Fix:` icons v3 (light-theme buttons, dark rim, cylinder glass) + Theme-button v2 — owner pass

- **What:** the owner CHECKED V36 + V37 (chat) with refinements: (1) ICONS v3 — the v2
  theme-aware light rim REVERTED to the ONE dark rim in both themes, and the tool buttons
  carry the LIGHT-theme background in the dark theme instead (`windows.rs::tool_button`
  picks resting/hover/selected fills from `egui::Visuals::light()` — one drawing reads
  well everywhere, the owner's stated goal); the pipette's glass is a full CYLINDER with
  a cone ONLY at the tip and the collar ring got wider (still "a bit like a needle"
  before); (2) THEME-BUTTON v2 — the button reads just "Theme" (no ": Dark" suffix — the
  picked theme is visible by opening it), the System radio is plainly "System" (no
  "(follow the OS)"), and the button sits BETWEEN View and Help — Help ALWAYS last;
  the now-unused `theme_label()` is gone. SPEC §3.1/§3.2 updated; DEBTS V36/V37 carry
  the refined asks (first pass done, re-check pending).
- **Files:** `src/app/tool_icons.rs` (rim revert + pipette cylinder/wider collar),
  `tool_icons/tests.rs` (the rim test now asserts the DARK rim with BOTH raster colors
  + no light rim), `src/app/windows.rs` (light-theme button fills), `src/app/menu.rs`
  (Theme before Help, plain labels), `src/app.rs` (`theme_label` removed), `SPEC.md`
  (§3.1, §3.2), `DEBTS-verification.md` (V36/V37 updated), PROGRESS (snapshot + entry
  + stamp).
- **Verified:** the pipette was iterated on a dump contact sheet (cylinder + wider
  collar read clean at 24 px on BOTH the dark and light rows); the rim test covers the
  revert (dark rim with WHITE and BLACK raster); FULL hot gate below; visual driver
  ALL SCENARIOS GREEN (regression).
- **Owner-checks:** V36 + V37 updated to "refinements delivered; re-check pending".
  Open: V37, V36, V30 (parked), V8 (parked).
- **Commit title:** `Icons v3: light-theme button background + one dark rim, cylinder glass, wider collar; Theme button before Help`
