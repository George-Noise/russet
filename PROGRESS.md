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
| Extra: About window + Nix | code written | Help → "About Russet" with Platform/Renderer lines; 2026-11-18: + "written in Rust" description + GitHub repo hyperlink; `flake.nix` for Linux (Wayland/X11) — `nix build` itself must be verified by the owner on a Nix machine |
| Extra: `tests.rs` layout + `PERF.md` (PR) | done (2026-11-10) | every module's tests live in `<module>/tests.rs`; `PERF.md` is a reference RAM/build-speed backlog, not an autonomous queue |
| Extra: README.md | done | English, build & run only (Windows / NixOS / Web); declared the human-facing doc in AGENTS.md + SPEC §8 |
| Extra: 6 tools/panels (owner 2026-11-10) | code written, tests green | Eyedropper (I), Line/Curve (L), Shape (U), Text (T), History log window, Layers placeholder window — see the log entries below; deliberately simple first iterations, followed by a UX-polish patch the same day (window positions, Shape outline+fill, preview fidelity) |
| Extra: CI + CONTRIBUTING.md + doc-tree removal (owner 2026-11-11) | done — first CI run GREEN (success for `0188c52`, GitHub Actions) | `.github/workflows/ci.yml` (native gate: fmt/clippy/test, Rust pinned 1.99.1; wasm job postponed by the owner); `CONTRIBUTING.md` + PR template for outsiders (one logical change per non-owner PR; the owner pushes directly); file trees removed from AGENTS.md and SPEC §2 — the filesystem is the source of truth; trunk pinned 0.22.1 everywhere, PERF P9 added |
| Extra: machine rules job (owner 2026-11-11) | code written, rules.sh tested in sandbox | new `scripts/rules.sh` — grep-level subset of AGENTS.md, single source of truth shared by verify.sh and CI: HARD (exit 1) = inline test modules + `unwrap()` anywhere in `src/`; SOFT report-only = `expect()` outside tests (5 known sites listed) + 500-line guard; CI gained a fast `rules` job (no Rust toolchain) |
| Extra: dependency approval flow (owner 2026-11-12) | done | AGENTS.md "Dependencies — approval flow" + SPEC §6 exception + CONTRIBUTING.md — the hard no-deps ban became propose → discuss → owner decides; no code touched |
| Extra: opt-in MCP server (owner 2026-11-12) | done — 10 tools behind `mcp` | default build byte-identical graph (no serde_json/threads); `get_tree_layout` + `add_layer` added 2026-11-13; verify.sh gates the feature |
| Extra: History Tree windows (owner 2026-11-12) | done — lanes reworked 2026-11-13 | divergence-only forks, recency lanes, shared connector exits, per-row labels; MCP-driven scenarios + Xvfb screenshots green |
| Extra: 500-guard splits (owner 2026-11-13) | done — pure code motion | `history_tree/layout.rs`, `tree_window/paint.rs`, `app/document.rs` + `menu.rs` + `input.rs`; app.rs 936 → ~440, every src file ≤ 500 |
| Extra: agent workflow v2 (owner 2026-11-13) | done | DEBTS-verification.md + DEBTS-functional.md debt lists; rules.sh PROGRESS budget check; baseline-before-edits rule; CI runs the mcp gates; visual_driver.py rewritten as the scenario runner |
| Extra: REAL layers + per-node PNG history (owner 2026-11-13) | done — gates green | pays F1+F4: interactive Layers window, per-layer git-style trees whose nodes carry full fast-PNG canvases, node-dot LEFT-click switching; SPEC §3.6 rewritten; GUI click-through = V13 (owner) |
| Extra: headless-GUI sandbox recipe (owner 2026-11-13) | done — driver all green headless | `SANDBOX.md` + `scripts/gui_recipe_test.sh`: raw Xvfb + two user-space .debs, no root; MCP `add_layer` added (10 tools); visual_driver.py 3/3 scenarios green in-sandbox; V13 narrowed to pointer feel |
| Extra: MCP tools split + commit-title ritual (owner 2026-11-13) | done — pure code motion | root `mcp/tools.rs` = the tool map (dispatch + declarations, 157); themed `tools/queries.rs` (200) + `tools/edits.rs` (177); rules.sh rule 6 prints the newest commit title (backticks kept) at every done-gate; AGENTS.md Delivery ritual requires it verbatim, backtick-wrapped, in the reply |
| Extra: UX round (owner 2026-11-13) | done — gates green | History window → Tool Log (list max height halved to 140 pt, 0.5rem entries, zero row spacing, bottom edge flush), Help item → About, dialog button rows per IBM/Apple/Google via shared `dialog_button_row`, Shape RMB-drag swaps the color wells, wheel pan ×2 (`SCROLL_PAN_SPEED` 3.0) |
| Extra: dialog epoch + Shape RMB round (owner 2026-11-15) | done — gates green | dialogs arise centered on EVERY open and hug their content (`dialog_window` + `forget_dialog_position`, NOT §3.11); Shape RMB uses the wells inverted WITHOUT swapping; Palette "Swap (X)" → "Swap"; 3 headless-egui regression tests + a scripted 9-check real-binary pass |
| Extra: layers split + two-color history chips (owner 2026-11-16) | done — gates green | pure code motion into `layers/snapshot.rs` + `layers/composite.rs` (tests split the same way); `TreeNode.color2` + the outline\|fill chip halves for two-color gestures; MCP `get_history_tree` dumps `color2`; preview's pure color rules → `app/preview/colors.rs` |
| Extra: Shift-proportional shapes (owner 2026-11-16) | done — gates green | Shape (U) drag with held Shift = square/circle, Paint.NET-style, LIVE (mid-drag snap/release); pure `proportional_current` in `app/preview.rs` + 5 unit tests; SPEC §3.2 bullet + §3.9 hold-gesture row; V22 born |
| Extra: third debt triage (owner 2026-11-16) | done — docs only | V13/V14/V17–V22 verified via chat; V15 closed into the NEW F11 (wheel pan ANOTHER ×5 — `SCROLL_PAN_SPEED` 3.0 → 15.0, how-to recorded in DEBTS-functional.md); AGENTS.md gained the zip-priority rule (a delivered .zip is used immediately — no git pull/clone); open after this edit: V8, V9, F3, F5, F8–F11 |
| Extra: V9 pass + platform-gap analysis (owner 2026-11-16) | done — docs only | V9 verified via chat (web build clicked through in a browser); the pass surfaced F12 (wheel-pan platform parity — `camera.rs` reads the RAW wheel delta in EVENT UNITS: Line ±1/notch on Windows/X11 vs DOM px as-is on the web) and F13 (window jitter on Wayland/Web resize — per-frame repositioning meets non-monotonic content rects); both analysis-only, fixes postponed by the owner |
| Extra: wheel-pan parity F12-B + resize-jitter fix F13-A + diag feature (owner 2026-11-16) | done — gates green | owner picked F12 variant B (INTERIM, not final; F11 folded into the math) and F13 variant A, approved the diagnostic patch behind a build flag; `wheel_notches` (unit → notches) + fixed `WHEEL_NOTCH_PAN` 15 pt/notch on every platform; `resize_step` pins the pre-resize rects and recomputes from the pin; temporary `diag` cargo feature ([diag] wheel/resize/reposition logs, compiled out of the default build); SPEC §3.4 + §7 rewritten; V23/V24 born |
| Extra: web-Line notch retune + `diag` → `diagnostics` rename (owner 2026-11-16) | done — gates green | Firefox scroll-pan ~3× overshoot fixed inside F12-B: Firefox emits `DOM_DELTA_LINE` ×3 lines/notch, so web `Line` deltas now divide by `WEB_LINE_UNIT_LINES_PER_NOTCH` 3.0 (native pass-through); the approved Windows/Chrome feel UNCHANGED (15 pt/notch); the temporary feature renamed `diag` → `diagnostics` (owner: the abbreviation was unclear) — feature, cfg sites, fn, `[diagnostics]` prefix; trunk-serve how-to: `trunk serve --features diagnostics` (logs → browser console); V23/V24 narrowed, not closed |
| Extra: Resize modes + per-layer resize nodes (owner 2026-11-17) | done — gates + headless driver green | the dialog's Canvas mode (3×3 anchor) / Image mode (nearest/bilinear/bicubic/Lanczos3 + aspect lock); ONE `resize` node per layer with the full `ResizeInfo`; the bottom layer owns the canvas size; the delete-bottom "Canvas Size Source" choice; native-size restores (F9 updated); V28 born |
| Extra: CLI launch args + owner-unconfirmed scenario markers (owner 2026-11-17) | done — gates green | native `--primary/--secondary/--tool/--canvas-size/--help` (`app/launch.rs`, parsed before the window opens; invalid values warn, defaults stay; wasm takes none); the 8 suite scenarios carry "NOT YET CONFIRMED" markers + a header ledger (the owner approves one by one); V29/V30 born |
| Extra: repo-wide English-only owner quotes (owner 2026-11-18) | done — docs only | every verbatim Russian owner quote across docs/scripts/comments translated to English (meaning kept); PROGRESS Log compacted — 2 entries moved to PROGRESS-archive.md |
| Extra: debt F14 — in-house windows later (owner 2026-11-18) | done — docs only | egui::Window → an in-house `egui::Area` frame recorded in DEBTS-functional.md from the owner's pros/cons chat review; owner-gated, no code touched |
| Extra: tool icon buttons + two-column grid (owner 2026-11-18) | done — gates green + visual driver | 1.5rem icon buttons via a cached tiny-skia atlas (`app/tool_icons.rs`), colored pencil/brush/eraser/pipette + mono line/shape/text through ONE pipeline, name+key hover tooltips, the pipette as the Eyedropper's canvas cursor; the two-column grid is debt F15; the driver screenshot caught the dead-texture trap (handles kept in the Atlas now — AGENTS.md quirks) |
| Extra: fully unoptimized dev profile (owner 2026-11-18) | done — cargo check green on the new lane | `[profile.dev]` opt-level 0 + no debuginfo, the `[profile.dev.package."*"] opt-level 2` dep override REMOVED (the "deps optimized once for responsive `cargo run`" policy retired — development is sandbox-only now, UI lag accepted); flake `russet-dev` already ran the unoptimized gates lane (untouched); SPEC §1 + AGENTS.md synced; owner visual pass = V34 |
| Extra: verify.sh hardening (owner 2026-11-18) | done — FULL gate green on a cold lane (191 s) | `exec </dev/null` closes stdin for the whole run (SANDBOX.md trap 10 fixed AT THE SOURCE — the mcp test stage exits in seconds instead of hanging); per-stage seconds + total printed (TIMING.md rows are copied now, not re-measured); cold-lane hint when `target/gates` is absent; the mcp pair stays UNCONDITIONAL (hot gate ≈ 15 s, zip sandboxes have no git, the gate never guesses) |
| Extra: brush + pipette icons redrawn + equal grid gaps (owner 2026-11-18) | done — hot gate 14 s + driver green | round volumetric bristle belly + crimped ferrule (the wedge read as a second pencil); tapering glass cone + round rubber bulb via a new `arc_pts` helper (the rectangle stick read as a pen); Tools-grid H-gap 18 pt → 6 pt = V-gap (egui Grid floors columns at `interact_size.x` 40 pt — `.min_col_width` fix, AGENTS.md quirk); tool_icons.rs 610 lines — the draw/atlas split proposed (owner-gated) |
| Extra: --warm warm-up + session rules; debts ask = 4 newest; V33 closed (owner 2026-11-18) | done — live cold-lane test + hot gate | `sandbox_check.sh --warm` = probe + detached gates-lane compile (clippy + test --no-run, nothing executed — the mcp stdin trap impossible) with a double-start pid guard; session start = `sandbox_check.sh --warm` FIRST; wasm ad-hoc checks only for web-facing changes; the batched debts ask lists only the 4 newest open items (AGENTS.md Debts rule 7) |
| Extra: permanent #[ignore] icon-dump + F16 rust-cache debt (owner 2026-11-18) | done — dump live-tested, gate skips it | `cargo test --profile gates -- --ignored icons_dump` = 14 PNGs (7 tools × 256/24 px) into /tmp/icon_dump, BLACK raster keeps the monochrome icons legible; plain gate = 156/168 passed + 1 ignored; rust-cache (`Swatinem/rust-cache@v2` after checkout) recorded as postponed F16 with the mechanism + limits written out, ci.yml untouched |
| Extra: icons v3 + Theme-button v2 (owner pass 2026-11-18) | done — contact sheet + FULL gate + driver | tool buttons keep the LIGHT-theme fills in both themes (`egui::Visuals::light()` in `windows.rs::tool_button`) → the outline is the ONE dark rim everywhere (v2 light rim reverted, tested); pipette glass = cylinder + cone only at the tip, wider collar ring; menu-bar button just "Theme" before Help (Help always last), System radio plain; MCP `set_theme` unchanged; V36/V37 → "refinements delivered; re-check pending" |

**Verification stamp:** FULL gate re-verified 2026-11-18 (fresh sandbox): fmt clean, rules.sh
HARD-green, clippy `-D warnings` green (default + mcp), `cargo test --profile gates` 156 default /
168 mcp green (+11 new tool-icon tests), `cargo check --target wasm32-unknown-unknown` green,
visual driver ALL SCENARIOS GREEN + `--shots` — Rust 1.99.1 pinned via rust-toolchain.toml, Linux.
The hardened gate (stdin shield + stage timers, same day) then re-ran EVERYTHING on a cold lane
after the dev-profile de-optimization: 191 s total, 156/168 green again — the profile change is
fully gate-verified. The icon-redraw round (same day) exercised the hardened gate on a REAL
change: hot run 14 s (156/168 green), the glass test caught a real drawing bug pre-delivery,
driver ALL SCENARIOS GREEN, the grid gaps pixel-measured 6 pt = 6 pt. The tooling round (same
day) proved `sandbox_check.sh --warm` on a truly cold lane (0.6 s return, the double-start
guard fired, a hot gate after the log's "finished") and closed V33 (NixOS + Windows). The
test round added the permanent #[ignore] icon dump (explicit-only: 14 PNGs live-checked,
the plain gate counts it as 1 ignored) and recorded CI rust-cache as F16 (owner: not now).
The icons-v2 round (same day) iterated the redraw on contact sheets, added the theme
switcher (menu bar + MCP `set_theme`, System default) and gate-verified it all. The
icons-v3 round (the owner's pass) reverted the rim to ONE dark rim over the now
light-theme tool-button background, made the pipette glass a cylinder and moved the
Theme button before Help: the gate run below is green with 160 default / 173 mcp
passed (+1 ignored each).
Zero Cyrillic repo-wide. Web/trunk lanes last verified 2026-11-16 (see the archived log history);
every src file ≤ 500 EXCEPT the report-only owner-deferred `history_tree.rs` 569 AND
`app/tool_icons.rs` 610 (grew in the 2026-11-18 icon redraw — the natural split is the icon
draw fns vs the atlas/cursor pipeline; proposed to the owner at delivery, not done without
his approval).

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
> entry followed that (the dump round's budget), and the verify.sh-hardening entry
> completed the day's compactions (the icons-v2 round's budget).

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

### 2026-11-18 — `Feat:` icons v2 + theme switcher (UI + MCP) + dump dir — owner request

- **What:** the owner VERIFIED V35 + V34 (chat) and ordered on top: (1) icon refinements —
  outlines 0.9 → 1.25 pt and THEME-AWARE (`outline_color` picks the rim from the text color's
  luminance: a LIGHT rim in the dark theme — the graphite + the brush's loaded tip no longer
  blend into the dark background); the pipette's collar is the classic WIDE band between glass
  and bulb with the bulb dominant (the slim collar read as a NEEDLE at 1.5rem); Line = a smooth
  S-curve of two quad arcs (the tool has curve mode); Shape's diamond → circle; Text = a
  serif-curved T plus a small lowercase t; (2) the THEME SWITCHER — a menu-bar button right of
  Help (Dark/Light/System radios; System = follow the OS, the startup default) through
  `App::apply_theme` → `ctx.set_theme` (egui's `ThemePreference` — `Theme` itself has no
  System/FollowSystem variant: an API quirk caught by the first compile, AGENTS.md); the Tool
  Log records it; the MCP `set_theme` tool (dark/light/system) runs the SAME path; SPEC §6
  drops "themes/appearance customization" from the out-of-scope list; (3) the ignored dump
  takes `RUSSET_ICON_DUMP_DIR` (a leading ~ expands to HOME/USERPROFILE) — README documents
  the ~/Pictures run (POSIX + PowerShell).
- **Files:** `src/app/tool_icons.rs` (outline_color/quad_pts + 5 redraws), `tool_icons/tests.rs`
  (+4 tests, dump dir), `src/app.rs` (theme field + apply_theme/theme_label), `src/app/menu.rs`
  (switcher), `mcp/tools.rs` + `edits.rs` + `tools/tests.rs` (set_theme), `README.md` (dump
  section), `SPEC.md` (§3.1 Theme, §3.2 icon v2, §6, §7 item 21), `AGENTS.md` (theme-API quirk),
  DEBTS (V35/V34 verified; V36/V37 born), PROGRESS (compaction: verify.sh entry → archive;
  snapshot + entry + stamp).
- **Verified:** the icons were iterated on dump contact sheets (4 rounds, dark + light rows +
  the real 24 px row); the new tests: the S's bends off the chord on BOTH sides, the 45° arc
  vs the empty old-diamond point, the serif hooks + small t, the theme-following rim; the dump
  live-ran with RUSSET_ICON_DUMP_DIR; `set_theme` MCP test (light / purple-error / system);
  FULL hot gate below (160 default / 173 mcp + 1 ignored); visual driver ALL SCENARIOS GREEN.
- **Owner-checks:** V36 (icons-v2 look, both themes) and V37 (theme switcher, three states)
  born. Open: V37, V36 (new), V30 (parked), V8 (parked).
- **Commit title:** `Icons v2: theme-aware rims, pipette collar, S-curve, circle, T+t; theme switcher in menu + MCP; dump dir param`

### 2026-11-18 — `Test:` permanent #[ignore] icon-dump; F16 rust-cache debt — owner request

- **What:** two owner orders (2026-11-18): (1) the icon contact sheet is now a PERMANENT ignored
  test `icons_dump_when_requested` in `app/tool_icons/tests.rs` — a plain `cargo test` (the
  done-gate) SKIPS it, run explicitly `cargo test --profile gates -- --ignored icons_dump`
  when redraw/adjusting an icon; it renders every tool at 256 px (design view) + the real
  24 px button size into /tmp/icon_dump/ (cleared first), BLACK raster so the monochrome
  icons (Line/Shape/Text take the raster color) stay legible; the icon-redraw loop's
  temporary dump test is no longer needed each time; (2) rust-cache recorded as F16 in
  DEBTS-functional.md (owner: not now — a possible improvement for when CI speed matters):
  one `Swatinem/rust-cache@v2` step after checkout, the mechanism, limits and the owner-gate
  written into the row; ci.yml untouched.
- **Files:** `src/app/tool_icons/tests.rs` (+ the ignored dump test), `DEBTS-functional.md`
  (F16 born), `PROGRESS.md` (compaction: the unoptimized-dev entry moved; snapshot row +
  this entry + stamp).
- **Verified:** the dump ran explicitly on a hot lane: 1 passed in 0.09 s, 14 PNGs (7 tools ×
  256/24 px) in /tmp/icon_dump/, PIL-checked 256×256 and 24×24; the plain gate SKIPS it —
  the test stages below count it as 1 ignored (156 default / 168 mcp passed, unchanged);
  rules.sh HARD-green.
- **Owner-checks:** none born (F16 is owner-gated work, not a human check). Open: V35 (the
  icon look), V34 (the unoptimized-dev pass), V30 (parked), V8 (parked).
- **Commit title:** `Add the permanent #[ignore] icon-dump test; record CI rust-cache as F16`

### 2026-11-18 — `Tooling:` warm-up flag + session rules; debts-ask trimmed; V33 closed — owner request

- **What:** four owner orders (2026-11-18), one docs/tooling round, NO src/ change:
  (1) `sandbox_check.sh --warm` — after the usual probe it starts a DETACHED gates-lane warm-up
  (clippy + `cargo test --no-run --features mcp`: exactly the compile work verify.sh does, nothing
  EXECUTED — no test binary spawns, so the mcp stdin trap is impossible here even without the
  shield), a pid-file guard blocks double starts; the session-start rule now runs
  `sandbox_check.sh --warm` FIRST — the done-gate is hot when the task ends (in-sandbox
  compilation; the retired warm-target STORE stays retired); (2) AGENTS.md wasm-check economy:
  ad-hoc `cargo check --target wasm32-unknown-unknown` ONLY for web-facing changes — neither
  verify.sh nor CI runs wasm, so an unneeded run is pure lost minutes; (3) debts rule 7
  (AGENTS.md + DEBTS-verification.md): the batched ask lists ONLY the 4 newest open items,
  newest first — the files keep every open item; (4) V33 verified (owner: About checked on
  NixOS AND Windows).
- **Files:** `scripts/sandbox_check.sh` (--warm), `AGENTS.md` (session-start bullet, wasm
  bullet, Debts rule 7), `DEBTS-verification.md` (presenting bullet, V33 → verified, V33's
  source ref → archive), `TIMING.md` (cold warm-up row), `PROGRESS.md` (compaction: the About
  entry moved; snapshot row + this entry + stamp).
- **Verified:** live test on a truly cold lane (`rm -rf target/gates`): `--warm` returned in
  0.6 s with the background build compiling (log /tmp/russet_warm_build.log), a second `--warm`
  hit the guard ("already running, pid N"), and after the log's "finished" the same-session
  verify.sh ran HOT (this round's gate run below); rules.sh HARD-green; no src/ change — no
  wasm check (the new economy rule's first application).
- **Owner-checks:** V33 closed. Open after this edit: V35 (the icon look), V34 (the
  unoptimized-dev pass), V30 (parked), V8 (parked).
- **Commit title:** `Add sandbox_check --warm warm-up; AGENTS wasm-economy + debts-ask rules; close V33`

### 2026-11-18 — `Feat:` brush + pipette icons redrawn; equal Tools-grid gaps — owner request

- **What:** the owner tested the hardened gate on a real task (2026-11-18): (1) the BRUSH
  icon's chisel wedge read as a second pencil — redrawn as a classic round brush: a ROUND
  volumetric bristle belly (peak half-height 2.7 pt past the ferrule) split into light/dark
  facets for volume, a metal ferrule with two crimp ridges, the loaded paint tip kept;
  (2) the PIPETTE's all-rectangle stick read as a pen — redrawn as a classic eyedropper:
  a tapering glass cone with a blue liquid stripe (the tip full of the sample), a WIDER metal
  collar band, and a ROUND rubber bulb (a new `arc_pts` helper tessellates the round parts);
  `PIPETTE_TIP` and the cursor hotspot unchanged; (3) the Tools grid's horizontal gap was
  18 pt vs 6 pt vertical — ROOT CAUSE inside egui's own `grid.rs`: every column is floored
  at `interact_size.x` (40 pt), padding the 28 pt buttons with dead width; fixed with
  `.min_col_width(TOOL_BUTTON_PT)` — pixel-measured 6 pt = 6 pt after the fix; quirk
  recorded in AGENTS.md.
- **Files:** `src/app/tool_icons.rs` (draw_brush + draw_pipette redrawn, `arc_pts`, the glass
  widened so the outline doesn't eat it), `src/app/windows.rs` (`min_col_width`),
  `src/app/tool_icons/tests.rs` (the glass assert follows the new palette), `AGENTS.md`
  (Grid quirk), `PROGRESS.md` (compaction: the tool-icons entry moved to PROGRESS-archive.md;
  snapshot row + this entry + stamp).
- **Verified:** the design was iterated on rendered PNGs (256 px + real 24 px contact sheets),
  then the FULL hot gate: ALL GATES GREEN 14 s (156 default / 168 mcp, fmt + clippy
  `-D warnings` clean) — the FIRST gate run caught a real bug: the glass band was thinner
  than two outline half-widths, so zero opaque glass pixels survived and the glass test
  failed; fixed by widening the cone; headless visual driver ALL SCENARIOS GREEN + `--shots`,
  the Tools window re-measured pixel-exact after the fix (H 6 / V 6).
- **Owner-checks:** V35 born (the redrawn-icons look pass on Windows). Open: V35 (new), V34
  (the unoptimized-dev pass), V33 (About), V30 (parked), V8 (parked).
- **Commit title:** `Redraw brush + pipette icons; equal Tools-grid gaps (egui Grid interact_size quirk)`
