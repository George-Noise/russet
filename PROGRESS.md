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

**Verification stamp:** fmt clean, rules.sh HARD-green, `cargo clippy --profile gates --all-targets
-- -D warnings` clean default + mcp, `cargo test --profile gates` 140 default / 152 mcp green (+9
magnet-panel tests), `cargo check --target wasm32-unknown-unknown` green — 2026-11-17 (Rust 1.99.1
pinned via rust-toolchain.toml, Linux). Web/trunk lanes last verified 2026-11-16 (see the archived
log history); every src file ≤ 500 except the report-only `panels.rs` 504 / `history_tree.rs` 569.

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

### 2026-11-17 — `Tooling:` deliver.sh, drag-probe template, flake dev-build fix — owner request

- **What:** agent-workflow round, zero app-behavior change. (1) `git pull`: the owner's master
  carried the tooling round as `e364221` (commit title verbatim); the worktree was byte-identical —
  this round's base is `e364221`. (2) NEW `scripts/deliver.sh` (owner request): the delivery ritual
  in one command — stages the worktree exactly (tracked + untracked-not-ignored via git, plain-tar
  fallback without .git), builds `russet.zip` + `russet.patch` under the FIXED names, runs BOTH
  byte-compares (patched clean base ↔ worktree ↔ unpacked zip), prints sizes + sha256; exit 1 on any
  mismatch. AGENTS.md "Delivery ritual" points every delivery at it. (3) NEW `scripts/
  gui_magnet_check.sh` in the repo (owner request "keep it if useful"): the drag-probe TEMPLATE from
  the magnet round (Xvfb + ffmpeg x11grab + xdotool drags) — copy per GUI feature whose behavior is
  only visible through window drags; AGENTS.md GUI bullet references it. (4) V26 flake fix (owner
  hit `error: attribute 'override' missing` on `nix build .#russet-dev`): recent nixpkgs builds
  buildRustPackage through `lib.extendMkDerivation`, whose results no longer carry `.override` —
  verified against nixos-unstable sources (build-rust-package/default.nix, cargo-build/check-hook.sh);
  `russet-dev` is now its OWN `buildRustPackage (commonArgs // { buildType = "gates"; checkType =
  "gates"; })` call sharing `commonArgs` with `russet`; the knobs are unchanged upstream args
  (`--profile gates` in both hooks, doCheck defaults true). Release lane untouched.
  (5) Mini-compaction: the scenario-suite entry moved verbatim to PROGRESS-archive.md (the live log
  held 4 entries; V30's pointer updated).
- **Files:** NEW `scripts/deliver.sh`, NEW `scripts/gui_magnet_check.sh`; `flake.nix` (packages
  restructure), `AGENTS.md` (delivery ritual + GUI bullet), `DEBTS-verification.md` (V30 pointer),
  `PROGRESS-archive.md` (+1 entry), `PROGRESS.md` (this entry).
- **Verified:** `bash scripts/deliver.sh origin/master /tmp/deliv_test` — both byte-compares MATCH,
  artifacts + checksums printed; flake brace/paren balance 0/0 and no `.override` remains; the
  upstream knobs re-checked in the fetched nixpkgs sources (NOT from memory); rules.sh HARD-green,
  fmt clean, clippy `-D warnings` clean default + mcp, `cargo test --profile gates` 140/152 green.
- **Owner-checks:** none born. V26: apply this patch, then `nix build .#russet-dev` and `nix run
  .#russet-dev` on the Nix machine. The <100MB cold-start question: NO — the compiled dep tree is
  the cold cost (debug/deps 2.0 G + gates/deps 1.3 G + wasm deps 96 M raw); the smallest useful
  artifact is the whole store (1.2 G zip); wasm-only (~40 M zip) warms nothing native.
- **Commit title:** `Automate the delivery ritual (scripts/deliver.sh); drag-probe template in scripts; fix russet-dev override error`

### 2026-11-17 — `Tooling:` sandbox probe, timing log, warm-target store, V29 closed — owner request



- **What:** agent-workflow round, zero app-behavior change. (1) `git pull`: the owner's master
  carried the applied magnet-panel patch (`3dcad27` — the previous delivery, commit title verbatim);
  the sandbox worktree was already byte-identical to it, so this round's base is `3dcad27`.
  (2) NEW `scripts/sandbox_check.sh` — the session-start environment probe (base tools, pinned
  toolchain + wasm target + rustfmt/clippy components, network reachability, per-lane `target/`
  warmth via `.fingerprint` counts, the Xvfb xkbfix fix presence) with exact fix hints; report-only,
  always exit 0. (3) NEW `TIMING.md` — the cold/hot duration log of heavy commands (two tables, 5
  newest rows each, same-edit rule), seeded with this session's numbers (hot `verify.sh` 14 s; hot
  clippy after real changes 59 s; ≈ 8–12 min cold dep build; `touch` of an unchanged file does NOT
  invalidate cargo's fingerprints — 13 s). (4) `AGENTS.md` "Commands": a Session-start bullet
  (probe first; when `target/` is cold restore the owner-hosted `russet-target.zip` from the release
  `target-store`) + the heavy-command timing duty; the baseline rule reconfirmed by the owner
  2026-11-17 (NO pre-edit test/compile runs — tests run at task end and land in the repo; the only
  exception is a toolchain/target warm-up, which is environment work, not a test baseline); the
  Delivery ritual gained the note that `russet-target.zip` is a requested-only artifact, not a
  routine one. `SANDBOX.md`: the restore + probe pointer in the Rust bullet. (5) V29 → verified
  2026-11-17 via chat (owner: «проверил V29»). (6) The warm-up artifact BUILT and delivered this
  round on request: `russet-target.zip` 1.2 GiB (the 3.8 G `target/` minus the `*incremental*`
  caches; dev + gates + wasm lanes, the GUI binary included) — the owner uploads it to the release
  `target-store`.
- **Files:** NEW `TIMING.md`, NEW `scripts/sandbox_check.sh`; `AGENTS.md` (Commands: Session start
  + timing duty + baseline reconfirmation; Delivery ritual note), `SANDBOX.md` (Rust bullet: restore
  + probe), `DEBTS-verification.md` (V29 → verified), `PROGRESS.md` (this entry).
- **Verified:** `bash scripts/sandbox_check.sh` run live — every lane [yes] (705/634/99 fingerprints;
  two false negatives found and fixed on the way: the `rustfmt-.* (installed)` grep pattern and the
  triple-lane `.fingerprint` depth); `bash scripts/verify.sh` ALL GATES GREEN, time'd twice: 14 s
  (nothing changed) and 13 s (after `touch src/app.rs`); `cargo test --profile gates` 140 default /
  152 mcp green; the zip built and CRC-tested (`unzip -t`), 141 s zip time recorded in TIMING.md.
- **Owner-checks:** none born. The owner uploads `russet-target.zip` to the GitHub release
  `target-store`; when ready: V26 (Nix), V30/V31 approval in chat.
- **Commit title:** `Add sandbox_check.sh and TIMING.md; session-start probe and warm-target restore rules; V29 verified via chat`

### 2026-11-17 — `Feat:` magnetized tool-settings panel detached from the Tools window — owner request

- **What:** the per-tool settings (Size, Line Mode + curve hints, Shape Kind + Fill/Outline, Text hints)
  moved OUT of the Tools window into their own title-bar-less, immovable, content-hugging window glued to
  the Tools window with NO way to detach («без возможности отлипания пока что»): `Window::fixed_pos`
  re-applies the position EVERY frame (an egui 0.37 fact, pinned by a headless regression test) from the
  pure `app/placement.rs::magnet_settings_pos` + the `ToolSettingsPlacement` state (the panel's last drawn
  size). Vertically: below the Tools window at a `TOOL_SETTINGS_GAP` 0.25rem (4 pt); no room below → flips
  above. Horizontally: Tools window left of the main-window center → left edges align, right of center →
  right edges align (exact center = right); the result is clamped into the main window. The Size slider is
  SHOWN (enabled) only for Brush/Eraser/Line/Shape — the disabled-when-unapplicable slider is retired;
  Pencil/Eyedropper have no settings → the panel is not drawn at all. The Tools window is now only the
  tool list (§3.2 first-open ratio + §3.11 placement unchanged — they govern the smaller window).
- **Files:** `src/app/placement.rs` + `placement/tests.rs` (+5 magnet tests), `src/app/windows.rs` +
  NEW `src/app/windows/tests.rs` (+4: per-tool helpers + headless-egui magnet/hug regressions),
  `src/app.rs` (field), `SPEC.md` §3.2 (panel spec, stale Size bullet rewritten), `AGENTS.md` (module
  map + floating-windows fact), `DEBTS-verification.md` (V31 born), `PROGRESS.md` (+ archive: the
  resize-dialog and V27/V25 entries moved verbatim).
- **Verified:** baseline rules.sh HARD-green + fmt clean BEFORE the edit; after: rules.sh HARD-green,
  fmt clean, clippy `-D warnings` clean default + mcp, `cargo test --profile gates` 140 default / 152
  mcp green (+9), `cargo check --target wasm32-unknown-unknown` green; the headless MCP driver pass ALL
  GREEN under Xvfb (the SANDBOX.md recipe) with pixel-measured x11grab captures of the magnet behavior (gap 4 px exact, both edge alignments).
- **Owner-checks:** V31 born — the magnet pass on Windows: drag the Tools window (the panel follows
  same-frame), park it near the bottom edge (the panel flips above), move it across the screen center
  (left/right alignment switches), switch tools (panel content follows; Pencil/Eyedropper → no panel).
- **Commit title:** `Split tool settings into a magnetized panel under the Tools window (0.25rem gap, below/above flip, edge alignment)`

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

