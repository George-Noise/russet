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
| Extra: About window + Nix | code written | Help → "About Russet" with Platform/Renderer lines; `flake.nix` for Linux (Wayland/X11) — `nix build` itself must be verified by the owner on a Nix machine |
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

**Verification stamp:** fmt clean, `cargo clippy --profile gates --all-targets -- -D warnings` clean,
`cargo test --profile gates` — 99 green; with `--features mcp` — clippy clean, 111 green;
with `--features diagnostics` — clippy clean, 99 green; `trunk serve --features diagnostics`
smoke-tested earlier 2026-11-16 (build + serve OK, wasm checks green with and without the
feature); the winit-CSD dependency landed (manifest-only, no new tests — re-gated 2026-11-16)
(2026-11-16, Rust 1.99.1 pinned via rust-toolchain.toml, Linux; every src file ≤ 500;
rules.sh reports the PROGRESS.md/Log budgets — green; the later flake/docs round the same
day touched no Rust — cargo gates not re-run).
2026-11-17: the release-strip fix round (manifest-only, no Rust touched) — rules.sh
HARD-green + fmt clean before and after; `trunk build --release` run twice in-sandbox
(`strip = true` → the warning reproduces; `strip = "debuginfo"` → clean; sizes in PERF §1);
clippy/test lanes not re-run — no Rust change and the `gates` profile is untouched.
Later 2026-11-17: V27 + V25 verified via chat → Resolved (docs-only round, no Rust
touched — rules.sh HARD-green before/after, gates not re-run).
2026-11-17 (resize round): fmt clean, clippy `-D warnings` clean default+mcp+diagnostics, `cargo test --profile gates` 111 default / 123 mcp / 111 diagnostics green (+12), `cargo check --target wasm32-unknown-unknown` green, the headless MCP driver pass ALL GREEN (15 checks, real binary, Xvfb recipe of SANDBOX.md).
2026-11-17 (scenario-suite round): rules.sh HARD-green before and after; fmt clean, clippy `-D warnings` clean default+mcp, `cargo test --profile gates` 119 default / 131 mcp (+8 whole-app scenario tests) green, `cargo check --target wasm32-unknown-unknown` green.
2026-11-17 (launch-args round): fmt clean, clippy `-D warnings` clean default + mcp + diagnostics, `cargo test --profile gates` 131 green (+12 launch-args tests), `cargo check --target wasm32-unknown-unknown` green (no warnings); live smoke: `--help` prints, parse warnings hit stderr. The mcp TEST lane cold-compiled past 2×10 min in the recycled sandbox — interrupted (mcp clippy --all-targets green; no MCP code touched; CI runs the mcp tests).
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
> ever pinned in place by a pending check.

### 2026-11-17 — `Feat:` CLI launch arguments (color wells, tool, canvas size) + owner-unconfirmed scenario markers — owner request

- **What:** two requests, one round. (1) Launch arguments (native; parsed BEFORE the
  window opens — `app/launch.rs`): `--primary`/`--secondary` hex color wells (RRGGBB;
  #RGB, RRGGBBAA too), `--tool` (7 names, case-insensitive), `--canvas-size WxH` — the
  STARTUP canvas (buffers + white initial layer + the Resize-dialog prefill; the dialog
  1..=4096 clamp applies and reports; the New dialog keeps its 800×600 constants),
  `--help`/`-h` prints (text owned by main.rs) and exits windowless, `--mcp` skipped
  silently (the feature scans argv itself). NOTHING panics: invalid values and unknown
  args warn on stderr (`russet: ...`) and the default stays; ONE code path native + wasm
  (args_os is empty there). `App::new` → `new_with(cc, config)`: both entry points;
  `build(ctx, config)` takes the size, `apply_to` the rest. (2) The 8 whole-app scenarios
  are marked NOT YET CONFIRMED (owner request): a confirmation ledger in the suite's
  header + a per-scenario OWNER STATUS marker each — a comment-only edit of the locked
  file on the owner's explicit word; approvals flip the markers one by one (V30).
- **Files:** `src/app/launch.rs` + `src/app/launch/tests.rs` (new, 12 tests),
  `src/app.rs` (mod + `new_with`/`new_headless_with` + `build(ctx, config)`),
  `src/main.rs` (parse/help/warnings + the native-only USAGE text), `src/tools.rs`
  (Debug derive on Tool — for assert_eq!), `src/app/document.rs` (const doc),
  `src/app/scenarios/tests.rs` (markers, comment-only), `AGENTS.md` (module map),
  `SPEC.md` (§3.12 + the §3.6 marker sentence), `README.md` (Launch arguments),
  `DEBTS-verification.md` (V29/V30 born), `PROGRESS.md` (+ archive: the 2026-11-17
  strip-fix entry moved verbatim).
- **Verified:** baseline rules.sh HARD-green + fmt clean BEFORE the edit; after: rules.sh
  HARD-green, fmt clean, clippy `-D warnings` clean default + mcp + diagnostics,
  `cargo test --profile gates` 131 green (+12), `cargo check --target
  wasm32-unknown-unknown` green (no warnings); live smoke: `--help` prints, warnings hit
  stderr. The mcp TEST lane cold-compiled past 2×10 min in the recycled sandbox —
  interrupted (mcp clippy --all-targets green; no MCP code touched; CI runs it); with
  `< /dev/null` (SANDBOX pitfall 10) verify.sh ALL GATES GREEN (143 mcp green incl.).
  Delivery byte-compared: worktree ↔ unpacked zip ↔ patch applied on a clean clone of
  the base commit — all three match.
- **Owner-checks:** V29 born (the launch-args pass on Windows) + V30 born (approve the
  8 scenarios one by one). SOFT to relay: `panels.rs` 504 and `history_tree.rs` 569
  lines over the 500 guard (pre-existing; splits only on the owner's approval).
- **Commit title:** `Add CLI launch arguments (color wells, tool, canvas size); mark the suite scenarios owner-unconfirmed`

### 2026-11-17 — `Test:` the whole-app scenario suite — owner-locked, size-guard exempt — owner request

- **What:** while the owner walks the 8 manual chat scenarios by hand, ONE automated
  whole-app suite lands: `app/scenarios.rs` + its `tests.rs` (8 tests) driving a REAL
  headless `App` (no renderer, no egui passes) through the resize × history coupling:
  mid-chain undo/redo, bottom-layer canvas ownership, a fork of two resizes, node jumps
  + branch pixels, chained resizes step-by-step, bottom-layer deletion (same/different
  size), "Use layer as base" (no node) vs "Keep canvas size" (own node + the undo
  re-expand). OWNER-LOCKED (owner request 2026-11-17): edits only on the owner's
  explicit permission, the agent only APPENDS new scenarios on request; exempt from
  BOTH size guards, NO split debt (AGENTS.md "Tests" rule 7 + "Repo facts"; rules.sh
  skips the file). Dialog CLICKS stay owner-manual: the suite asserts the dialog
  trigger (the successor-size condition) and runs the exact branch bodies the buttons
  call. `App::new` refactored (pure code motion) into `build(ctx)` + a cfg(test)
  `new_headless()` — eframe's `CreationContext` is not constructible outside the crate.
  The chat walkthrough had three expectations the CODE corrected: the BR crop lands the
  dab at (100,100) (not (300,300)); reaching 800×600 in scenario 5 takes THREE undos
  (the head steps onto the dab first); scenario 3's fork hangs off the mark's node, not
  the Start root — the owner's manual pass should use the corrected steps.
- **Files:** `src/app.rs` (build refactor + mod), `src/app/scenarios.rs` (new),
  `src/app/scenarios/tests.rs` (new, owner-locked), `scripts/rules.sh`, `AGENTS.md`,
  `SPEC.md` §3.6, `PROGRESS.md`.
- **Verified:** baseline rules.sh HARD-green + fmt clean BEFORE the edit; after:
  rules.sh HARD-green, fmt clean, clippy `-D warnings` clean default + mcp,
  `cargo test --profile gates` 119 default / 131 mcp (+8 scenarios) green,
  `cargo check --target wasm32-unknown-unknown` green. Delivery byte-compared.
- **Owner-checks:** none born (V28 already tracks the manual resize walkthrough; this
  suite is its automated counterpart). SOFT notes to relay: `panels.rs` 504 and
  `history_tree.rs` 569 lines over the 500 guard (pre-existing from the resize round;
  splits only on the owner's approval).
- **Commit title:** `Add the owner-locked whole-app scenario suite (resize x history walkthrough); App::new build() refactor`

### 2026-11-17 — `Feat:` Resize dialog modes — canvas anchor grid, image resample filters, per-layer resize nodes, bottom layer owns the canvas — owner request

- **What:** the owner's 4-point request (chat, 2026-11-17). (1) The dialog gained a MODE
  radio: "Canvas size" — the Paint.NET-style 3×3 painted-dot ANCHOR grid (center = floor:
  an odd crop takes the extra pixel left/top, an odd pad adds it right/bottom) — vs
  "Image size" — a resampling radio Nearest neighbor / Bilinear / Bicubic (default) /
  Lanczos3 via the PINNED `image` crate's `imageops` on PREMULTIPLIED bytes (no new deps,
  approval flow not needed) + a "Maintain aspect ratio" lock (the changed field drives the
  other along the canvas ratio). (2) EVERY layer's tree records ONE `resize` node — new
  `ActionKind::Resize` + `TreeNode::resize: Option<ResizeInfo>` (mode, the layer's
  from→to, anchor/filter); the tree row prints the summary, the dot hover explains it, the
  MCP `get_history_tree` dumps the struct — jumping back beyond the resize shows what it
  did to THAT layer. (3) SIZE MODEL: the BOTTOM layer owns the canvas size; other layers
  composite at (0,0) clipped (`composite` learned to clip — mixed sizes are legal);
  restores keep the node's NATIVE size (the 2026-11-13 crop/pad fit retired — F9 updated;
  a bottom-layer restore resizes the canvas with it). (4) Deleting the bottom layer:
  same-size successor = silent new base; different size = the "Canvas Size Source" dialog
  ("Keep canvas size" = top-left adaptation + its own resize node / "Use layer as base" =
  the canvas takes the size; no dismiss). New `layers/resize.rs` (anchor_offset,
  fit_copy_at, rescale, scaled_layer_size); OK on an unchanged size is a silent no-op;
  `fit_to_canvas` removed. SPEC §3.6/§3.7/§3.8/§6 exception/§7 item 11 rewritten.
- **Files:** `src/history_tree.rs`, `src/layers.rs`, `src/layers/resize.rs` (new),
  `src/layers/snapshot.rs`, `src/layers/composite.rs`, `src/app.rs`, `src/app/dialogs.rs`,
  `src/app/menu.rs`, `src/app/panels.rs`, `src/app/tree_window/paint.rs`,
  `src/app/mcp/tools/queries.rs`, `src/app/document.rs`, `src/canvas.rs` (unchanged),
  `SPEC.md`, `DEBTS-verification.md` (V28 born), `DEBTS-functional.md` (F9 update),
  `PROGRESS.md` (+ archive: the Wayland-CSD and Firefox-retune entries compacted verbatim).
- **Verified:** rules.sh HARD-green; fmt clean; clippy `-D warnings` clean default + mcp +
  diagnostics; `cargo test --profile gates` 111 default / 123 mcp / 111 diagnostics green
  (+12 new: the anchor table, crop/pad offsets, nearest ×2, ×1 identity, proportional
  rounding, per-layer resize nodes, undo/redo over a resize node, fit_bottom_to, the
  summaries, the aspect lock, record_resize); `cargo check --target wasm32-unknown-unknown`
  green; the headless MCP driver pass — ALL 3 scenarios green (15 checks) through the real
  gates-profile binary under Xvfb (the SANDBOX.md recipe). Delivery byte-compared as usual.
- **Owner-checks:** V28 born — the resize pass (Windows, optionally browser): anchors,
  filters, per-layer nodes + native-size jumps, the base-choice dialog branches.
- **Commit title:** `Add Resize modes (canvas anchor grid, image resample filters), per-layer resize nodes, bottom-layer canvas ownership`

### 2026-11-17 — Docs: V27 + V25 verified via chat → Resolved — owner request

- **What:** the owner verified both open checks in one pass («проверил V27, V25»):
  V27 — his local `trunk build --release` after the strip fix logs NO wasm-bindgen
  `target_features` warning and the release page passed the browser smoke (loads, draws,
  zoom/pan); V25 — the GNOME Wayland pass of the winit Adwaita CSD frame (title bar text
  follows New/Open/Save/dirty, double-click maximizes/restores, close/minimize work,
  light and dark shell themes look right). Both rows left the open table for the Resolved
  list in the same edit; no new checks born; no F-debt rode on them (V23/V24 closed and
  F13 paid 2026-11-16). Open after this edit: V26, V8; F12, F8, F9, F10, F3, F5.
- **Files:** `DEBTS-verification.md`, `PROGRESS.md`. Docs-only → cargo gates not re-run.
- **Verified:** rules.sh HARD-green before and after; delivery byte-compared as usual.
- **Owner-checks:** none born.
- **Commit title:** `docs: verify V27 V25 (release-strip rebuild, Wayland CSD pass)`
