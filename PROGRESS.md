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
| Extra: Tools strip fits the window — variant A2 (owner 2026-11-19) | done — gates + probes green | the title strip's HORIZONTAL padding zeroed via `Window::title_frame` (`windows.rs::tools_title_frame` = egui's default window frame with `inner_margin.left/right = 0`): the strip paints EXACTLY the window's width instead of ~11 pt past it, the ✕ no longer hangs half outside, BOTH buttons kept, the strip's height/fill/corners unchanged (variant A `Margin::ZERO` shrank the strip vertically — not picked); +1 regression test on the `__title_click` geometry; `gui_collapse_check.sh` locate fixed (the idle crosshair cursor fooled it); V38 + V36 verified in chat, V39 born |
| Extra: Tools panel hides while collapsed (owner 2026-11-19) | done — gates + click probe green | the magnetized settings panel hides while the Tools window is collapsed to its title bar and returns on expand (`windows.rs::settings_panel_shown` on `Window::show`'s `inner.is_some()`); the header-width PROPOSAL delivered (measured options, no header code change — the owner picks); V38 born |
| Extra: tool_icons.rs file-guard split — the drawings to the child `draw` (owner 2026-11-19) | done — gates green, suite unchanged | 673 → 308 + 400, both under the 500 guard: the SEVEN icon drawings (`draw_*` fns, `TOOL_ANGLE`, `PIPETTE_TIP`, the `rasterize_tool_icon` dispatcher) moved VERBATIM to the child `app/tool_icons/draw.rs`; the parent keeps the atlas/cursor pipeline + the shared primitives and imports the dispatcher — zero external path changes (app.rs/windows.rs untouched, the tests' `super::` imports byte-identical); rules.sh's second OVER-GUARD note retired (only the owner-deferred history_tree.rs 569 remains) |

**Verification stamp:** FULL gate re-verified 2026-11-19 (hot lane, 14 s): fmt clean,
rules.sh HARD-green, clippy `-D warnings` green (default + mcp), `cargo test --profile gates`
163 default / 176 mcp green (+1 ignored each — the permanent #[ignore] icon dump) with the
title-strip-fit regression test (verified to FAIL pre-fix); the visual driver ALL SCENARIOS
GREEN and the `scripts/gui_collapse_check.sh` click probe ALL GREEN under Xvfb
(expanded→collapsed→re-expanded, PIL-located buttons, the cursor-run locate fix) —
Rust 1.99.1 pinned via rust-toolchain.toml, Linux.
English-only repo-wide — a language WHITELIST (English is simply the only written language
here: comments, docs, commit titles), not a script blacklist. Web/trunk lanes last verified
2026-11-16 (see the archived log history);
every src file ≤ 500 EXCEPT the report-only owner-deferred `history_tree.rs` 569
(the 2026-11-19 owner-approved split retired the last agent-side exception:
`app/tool_icons.rs` 673 → 308 + the child `app/tool_icons/draw.rs` 400).

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
> during the 2026-11-19 collapse round's budget; the icons-v3 and
> language-rule entries moved during the 2026-11-19 header round's budget;
> the icons-v4 entry moved during the 2026-11-19 split round's budget.

### 2026-11-19 — `Refactor:` tool_icons.rs split — the icon drawings to the child draw module — owner-approved

- **What:** the file-guard debt paid for `app/tool_icons.rs` (673 lines after the 2026-11-18
  icon rounds; proposed twice, and the owner approved it at the V39 check: "do the split of
  app/tool_icons.rs"). PURE code motion, zero behavior change: the SEVEN ICON DRAWINGS — the
  per-tool `draw_*` fns with their fixed color specs, `TOOL_ANGLE`, `PIPETTE_TIP` and the
  `rasterize_tool_icon` dispatcher — moved VERBATIM to the child module `app/tool_icons/draw.rs`
  (400 lines), which imports the parent's shared primitives (`fill`/`stroke`/`arc_pts`/
  `quad_pts`/`icon_transform`/`OUTLINE_W`); the parent (308 lines) keeps the atlas/cursor
  pipeline and the shared primitives and imports the dispatcher (`use self::draw::{…}`) — every
  `tool_icons::` path stays (app.rs/windows.rs untouched) and the tests' `super::` imports are
  unchanged byte-for-byte (the parent's private bindings resolve them). Same shape as the
  `tree_window`/`paint` child. No owner check born (the panels-split precedent: pure code
  motion is proven by the unchanged suite + the gates).
- **Files:** `src/app/tool_icons.rs` (673 → 308), NEW `src/app/tool_icons/draw.rs` (400),
  `src/app.rs` (the module map's `tool_icons` entry added, with the child mention),
  `AGENTS.md` (the module-list entry updated), `DEBTS-verification.md` (V39 verified + moved
  to Resolved; two resolved-entry owner quotes translated to English per the language
  whitelist), PROGRESS (snapshot + entry + stamp + the icons-v4 entry → archive).
- **Verified:** FULL hot gate ALL GATES GREEN 14 s — rules HARD green (the tool_icons
  OVER-GUARD note retired; only the owner-deferred history_tree.rs 569 remains), fmt clean,
  clippy `-D warnings` green (default + mcp), `cargo test --profile gates` 163 default /
  176 mcp green (+1 ignored each) — the counts UNCHANGED from the pre-split tree, the
  pure-code-motion proof. No wasm lane: not a web-facing change (the session rule).
- **Owner-checks:** none born. Open: V30 (parked), V8 (parked).
- **Commit title:** `Split app/tool_icons.rs: the icon drawings move to the child tool_icons/draw.rs (pure code motion)`

### 2026-11-19 — `Fix:` the Tools header strip fits the window — variant A2 — owner request

- **What:** the owner picked "variant A2" among the measured header-width options: the title
  strip's HORIZONTAL padding is zeroed — `tools_window_builder` feeds `Window::title_frame`
  with egui's default window frame whose `inner_margin.left/right` are 0
  (`app/windows.rs::tools_title_frame`) — so the both-buttons strip paints EXACTLY the
  window's width instead of ~11 pt past it and the ✕ no longer lands half outside. BOTH
  buttons kept (the ask); the VERTICAL padding stays, so the strip's height, fill and
  rounded corners are unchanged (variant A, `Margin::ZERO`, additionally shrank the strip
  vertically and shifted the first-open position — NOT picked). Also closed V38 + V36 in
  chat and carried the probe fix below.
- **Files:** `src/app/windows.rs` (tools_title_frame + the builder + the `TOOL_GRID_GAP`
  const extracted), `windows/tests.rs` (+1 regression test: the strip's `__title_click`
  drag strip spans collapse-icon-right .. close-icon-left, so the ▼/✕ atoms' outer edges
  `(click ± (heading_h + icon_width)/2)` must stay within the window — verified to FAIL
  pre-fix), `scripts/gui_collapse_check.sh` (locate_strip_icon now takes the first ≥10-row
  non-canvas run at x=8 — the idle X11 crosshair cursor's ~6-row crossing fooled it into
  clicking the cursor's own position; probe bug, not an app bug), `SPEC.md` §3.2,
  `AGENTS.md` (quirk extended: the chosen fix + the test's atom math),
  `DEBTS-verification.md` (V38 + V36 verified, V39 born), PROGRESS (compaction: the
  icons-v3 + language-rule entries → archive; snapshot + entry + stamp), TIMING (2
  build rows).
- **Verified:** FULL hot gate ALL GATES GREEN 14 s (163 default / 176 mcp, +1 ignored
  each); the pre-fix regression check (red with the title_frame line removed, green with
  it); Xvfb strip probe: window 9..82 / strip 9..82 flush, ▼ 14..23 and ✕ 67..77 fully
  inside (`download/tools_header_A2_fixed.png`); visual driver ALL SCENARIOS GREEN;
  gui_collapse_check EXIT=0 (expanded→collapsed→re-expanded).
- **Owner-checks:** V39 born (the header pass on Windows). Open: V39, V30 (parked),
  V8 (parked).
- **Commit title:** `Tools window: the title strip fits the window (zeroed strip side padding, variant A2)`

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
