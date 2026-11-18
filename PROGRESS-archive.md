# PROGRESS — archive (Russet)

### 2026-11-18 — `Docs:` debt F14 born — in-house window solution, owner-gated (later)

- **What:** the owner (owner: "record it in the debts — replace egui::Window with our own
  solution, we'll do it later") ordered the window-solution replacement recorded as a debt.
  F14 added to DEBTS-functional.md (newest-first row): an in-house frame over `egui::Area`
  replacing all floating windows and dialogs; the entry carries the 2026-11-18 chat pros/cons
  review (the recorded `egui::Window` quirks freed vs drag/occlusion/AccessKit/placement-glue/
  test rework across ~9 call sites), the REJECTED frameless-main-window alternative and the
  owner gate. No code touched — SPEC §3.7–§3.8/§3.11 behavior unchanged. The same chat answered
  the icon-rendering question (Painter primitives + one-time rasterization into a cached
  `TextureHandle`, re-rasterized on ppp/theme change) — analysis only, no repo change.
- **Files:** `DEBTS-functional.md` (F14), `PROGRESS.md` (this entry + snapshot row + stamp).
- **Verified:** docs/comment-only round — `cargo fmt --check` clean, `cargo check` green,
  rules.sh HARD-green; no clippy/test rerun per AGENTS.md "Docs/comment-only edits:
  cargo check is enough".
- **Owner-checks:** none born. Open: V33 (the About pass), V30 (parked), V8 (parked).
- **Commit title:** `Record debts F14/F15; tool icon buttons via a cached tiny-skia atlas + two-column Tools grid`
  (superseded 2026-11-18: the owner merges the F14 docs round into the ONE icon-buttons commit —
  the newest entry's title above is the commit's canonical title)

### 2026-11-18 — `Docs:` Russian owner quotes translated to English repo-wide — owner request

- **What:** the owner asked (2026-11-18) to translate every Cyrillic text in the project to English
  without losing meaning. All verbatim owner quotes across AGENTS.md, SPEC.md, PROGRESS.md,
  PROGRESS-archive.md, TIMING.md, DEBTS-verification.md, DEBTS-functional.md, scripts/rules.sh and
  the doc comments of 9 src files are English now (the `owner: "…"` attribution style kept; the
  keyboard-layout examples dropped the Cyrillic key glyphs — "the Russian typewriter layout's
  W/A/S/D keys"). The owner-locked `app/scenarios/tests.rs`: the request IS the explicit
  permission — only its 2 header quotes changed, no scenario steps touched. Zero Cyrillic left
  repo-wide (script-checked). Also: the `roxmltree` entry in Cargo.lock was verified to come from
  `fontique` (the `system_fonts` feature's fontconfig enumeration) — NOT an SVG dependency.
- **Files:** `SPEC.md`, `AGENTS.md`, `TIMING.md`, `scripts/rules.sh`, `DEBTS-verification.md`,
  `DEBTS-functional.md`, `src/{app/{scenarios/tests,tool_log,dialogs,windows,placement,camera,
  preview/colors}.rs, app/placement/tests/resize.rs, history_tree.rs}`; PROGRESS Log compacted.
- **Verified:** docs/comment-only round — `cargo fmt --check` clean, `cargo check` green
  (3m51s cold), rules.sh HARD-green; no clippy/test rerun per AGENTS.md "Docs/comment-only
  edits: cargo check is enough".
- **Owner-checks:** none born. Open: V33 (the About pass), V30 (parked), V8 (parked).
- **Commit title:** `Translate the recorded owner quotes to English across docs and comments`

### 2026-11-17 — `Refactor:` placement/tests.rs split by topic — owner-approved — owner request

- **What:** the 500-line file guard paid for `app/placement/tests.rs` (518 lines, owner: "do the
  placement/tests.rs split"). PURE code motion, zero behavior change — the 25 test bodies moved
  byte-identical. Per "Tests" rule 7 the file becomes a DECLARATION-ONLY module (10 lines: six
  `mod` lines + a header) and the suites move to the repo's FIRST `tests/` subdirectory:
  `app/placement/tests/common.rs` (the rect/assert_close helpers), `anchors.rs` (5 tests: the
  drop-time magnet detection), `reposition.rs` (8: the anchored/free resize math), `resize.rs`
  (3: the F13-A pin lifecycle), `first_open.rs` (4: the §3.2 ratio + the first-open anchor
  regression trio), `panel.rs` (5: the tool-settings magnet). New precedent: topic files reach
  the placement items with `crate::app::placement::…` — descendant modules see the private items,
  and the flat `use super::` style would have meant an unreadable `super::super::`.
- **Also:** V32 CLOSED — the owner verified the first-open fix on his machine ("checked the windows");
  the row stays in DEBTS-verification.md as a verified anchor row. The first `tests/` split
  convention is this entry; future over-guard test files follow it.
- **Files:** `src/app/placement/tests.rs` (10), `src/app/placement/tests/{common,anchors,
  reposition,resize,first_open,panel}.rs` (15/114/168/80/104/78), `DEBTS-verification.md`,
  `PROGRESS.md`.
- **Verified:** rules.sh HARD-green — the placement/tests.rs OVER GUARD is retired (only the
  owner-deferred `history_tree.rs` remains). fmt clean, clippy `-D warnings` clean, tests 143/155
  green with the SAME counts (pure motion), wasm green. verify.sh 11 s.
- **Owner-checks:** none born (code motion; the suite itself is the check). Open: V30 (parked),
  V8 (parked).
- **Commit title:** `Split placement/tests.rs by topic per rule 7; close V32 (first-open fix verified)`

### 2026-11-17 — `Fix:` first-open window gaps drift on the first main-window resize — owner bug report — owner request

- **What:** the owner's report (fresh launch → resize the main window → the windows slid off their
  magnet boundaries; a manual re-drag re-anchored them). ROOT CAUSE: §3.11 anchors are decided only
  on drag-end, so a never-dragged window had FREE axes — and the free-axis gap-ratio formula scales
  the gaps with the free space (Layers right gap 8 pt → 8.83 on a 1200→1300 grow), while the SPEC
  promises «§3.11 keeps the dropped gap afterwards» for the first-open positions. FIX: explicit
  FIRST-OPEN anchors per window — `WindowPlacement::with_first_open_anchors` (placement.rs;
  `AxisAnchor`/`WindowAnchor` become pub(super) for the app-level init) applied in `App::new`:
  Tools x=Min/y=free (the 40/60 ratio is a free-axis placement BY DESIGN), Palette Min/Max, Tool
  Log free-x (stays centered)/Max, Layers Max/Max (the 36 pt zoom-label lift is KEPT, not scaled);
  tree_window.rs: main tree + clones Max/Min (right edge + below the menu bar). A drag-end still
  overwrites the anchor with the user's dropped gaps — the user's placement always wins.
- **Tests:** three regression tests in `placement/tests.rs` (143/155 total now): the anchored
  first-open gaps survive grow+shrink; WITHOUT anchors they drift (the bug, pinned); the Tools
  window keeps the left gap AND the vertical ratio. ALSO: the owner's commit `3be38d4` carried the
  old `panels.rs` as a DEAD ORPHAN next to its split successors (the rename-form patch was applied
  by a tool that skipped the rename deletion; the file had no `mod` declaration, so everything
  still compiled) — the orphan is deleted in this round, and `deliver.sh` now emits
  `--no-renames` patches so plain delete+add diffs apply unambiguously in any patcher. V30 parked
  by the owner (light-tasks mode); V32 born (the fix's manual pass).
- **Files:** `src/app/placement.rs` (+builder, visibility), `src/app.rs` (anchor init + import),
  `src/app/tree_window.rs` (2 sites + import), `src/app/placement/tests.rs` (+3 tests; 518 lines —
  over the 500 guard: the topic split per "Tests" rule 7 is PROPOSED, awaiting the owner's OK),
  `scripts/deliver.sh` (--no-renames), `DEBTS-verification.md`, `PROGRESS.md`.
- **Verified:** rules.sh HARD-green (OVER GUARD: the orphan panels.rs is GONE, history_tree.rs is
  the owner-deferred one, placement/tests.rs 518 — proposed split), fmt clean, clippy `-D warnings`
  clean default+mcp, tests 143/155 green, wasm check green. verify.sh 13 s.
- **Owner-checks:** V32 born (the fresh-launch resize pass, see DEBTS-verification.md). Open: V32,
  V30 (parked), V8 (parked).
- **Commit title:** `Fix first-open window gaps drifting on resize (explicit first-open anchors); drop the panels.rs orphan`


Verbatim log entries moved out of `PROGRESS.md` by the log-compaction rule
(AGENTS.md "Git & progress log": the live Log keeps the newest 3–5 entries,
older ones go here unchanged). Read this file only via grep when investigating
history — e.g. `rg -n "magnetism|min size|flake" PROGRESS-archive.md`. Test
counts inside are HISTORICAL (they grow over time); the current count is in
the `PROGRESS.md` verification stamp. Newest first, same format as the live log.

---

### 2026-11-17 — `Refactor:` panels.rs split into tool_log.rs + layers_window.rs — owner-approved — owner request

- **What:** the 500-line file guard debt paid for `app/panels.rs` (504 lines, owner: "do the
  panels.rs split, but not at the cost of agent development"). PURE code motion, zero behavior change: the file
  becomes TWO DIRECT children of `app` — `app/tool_log.rs` (102: the Tool Log window, its id fn and
  the TOOL_LOG_* consts) and `app/layers_window.rs` (430: the Layers window, LayerRequest, the row/
  arrow helpers, the layer ops and the delete dialog). The two-direct-children shape (no panels.rs
  shell, no grandchild `impl App`) was chosen over a child-module split DELIBERATELY so nothing
  changes for future agents: every `impl App` stays in `app.rs` or its direct children (the repo
  convention), every visibility stays `pub(super)`, every `use super::...` line moves over verbatim,
  the app.rs call sites (`self.history_window` / `self.layers_window` / `self.layer_delete_dialog`)
  are untouched. The `panels` name survives only in split-history notes (both new module docs,
  app.rs module map, AGENTS.md/SPEC.md pointers updated). history_tree.rs (569) is NOT split —
  owner deferral (his unfinished task lives there); AGENTS.md "File-size guard" now records the
  deferral so the split is not re-proposed. `git pull`: the owner's master carried the store-
  retirement round as `cc2734d` (commit title verbatim); the worktree was byte-identical — this
  round's base is `cc2734d`.
- **Files:** NEW `src/app/tool_log.rs`, NEW `src/app/layers_window.rs`; DELETED `src/app/panels.rs`;
  `src/app.rs` (mod decls + module map + 3 doc refs), `AGENTS.md` (pattern table, module list,
  file-size guard bullet, egui quirk pointer), `SPEC.md` (§1 module list), `TIMING.md` (new hot row +
  the done-gate note corrected to the measured ~20 s small-change cost), `PROGRESS.md`.
- **Verified:** the baseline was the byte-identical-to-origin `cc2734d` (CI-green commit; the
  pre-edit rules.sh/fmt baseline was skipped this round — nothing had touched the tree). After:
  rules.sh HARD-green
  (the only OVER GUARD left is the owner-deferred history_tree.rs), fmt clean, clippy `-D warnings`
  clean default + mcp, `cargo test --profile gates` 140 default / 152 mcp green (counts unchanged —
  the suite is byte-identical), `cargo check --target wasm32-unknown-unknown` green. verify.sh 18 s
  for a small change. Delivery byte-compared via deliver.sh.
- **Owner-checks:** none born. Open: V30 (8 scenario markers), V8 (parked).
- **Commit title:** `Split app/panels.rs into tool_log.rs + layers_window.rs (pure code motion); defer history_tree.rs split`

### 2026-11-17 — `Docs:` V26/V28/V31 verified in chat; warm-target store retired — owner request

- **What:** bookkeeping round, zero repo-behavior change. (1) `git pull`: the owner's master carried
  the deliver.sh round as `778b214` (commit title verbatim); the worktree was byte-identical — this
  round's base is `778b214`. (2) Three human checks verified via chat and closed in
  DEBTS-verification.md (verbatim owner quotes kept in the rows): V26 — `nix build .#russet-dev`
  (the flake dev build compiles in gates AND runs the tests), V28 — the Resize-dialog pass
  ("I tried the Resize dialog, it suits me for now"), V31 — the magnetized Tools+panel pass on his
  machine. Remaining open: V30 (the 8 scenario markers) + V8 (parked). (3) The warm-target STORE is
  RETIRED (owner decision: "drop the warm-target idea for now — dev builds fast enough for tests"): AGENTS.md "Session start" + "Delivery ritual" and TIMING.md now state the cold
  build is the accepted cost; `sandbox_check.sh` still reports per-lane warmth but no longer points
  at a hosted zip; the 1.2 G `russet-target.zip` deleted from the chat download dir.
- **Files:** `DEBTS-verification.md`, `AGENTS.md` (2 bullets), `TIMING.md` (header + numbers),
  `scripts/sandbox_check.sh` (verdict line), `PROGRESS.md` (this entry).
- **Verified:** docs-only round — verify.sh ALL GATES GREEN, no src/ change so no rebuild;
  rules.sh HARD-green; the three DEBTS quotes checked verbatim against the chat message.
- **Owner-checks:** none born. Open: V30, V8 (parked).
- **Commit title:** `Close V26/V28/V31 via chat; retire the warm-target store rule`

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

---

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
  2026-11-17 via chat (owner: "checked V29"). (6) The warm-up artifact BUILT and delivered this
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

---

### 2026-11-17 — `Feat:` magnetized tool-settings panel detached from the Tools window — owner request

- **What:** the per-tool settings (Size, Line Mode + curve hints, Shape Kind + Fill/Outline, Text hints)
  moved OUT of the Tools window into their own title-bar-less, immovable, content-hugging window glued to
  the Tools window with NO way to detach ("with no way to detach it, for now"): `Window::fixed_pos`
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

---

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

---

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

---

### 2026-11-17 — `Fix:` release `strip = true` deleted the wasm `target_features` section (the wasm-bindgen warning) — owner request

- **What:** every `trunk build --release` warned "the `target_features` custom section is
  missing, likely due to `strip = true`": symbols-stripping is `--strip-all`, which deletes
  ALL wasm custom sections — including the `target_features` list rustc embeds so wasm-bindgen
  can detect the enabled Wasm features (here: bulk-memory, reference-types, multivalue,
  sign-ext). Harmless today (nothing non-default is enabled), but bindgen silently mis-glues
  the day simd128/atomics/threads land, and it already served a guessed empty-default section.
  Fix per wasm-bindgen's own advice: `strip = true` → `strip = "debuginfo"` — keeps the custom
  sections (release `debug` is 0, nothing extra to strip; Windows/MSVC unaffected), the size
  cut stays with wasm-opt AFTER wasm-bindgen (trunk 0.22 runs it on every release build —
  PERF P5). No Rust code touched — manifest + docs only.
- **Files:** `Cargo.toml`, `SPEC.md` §1, `PERF.md` §1 + §4, `DEBTS-verification.md` (V27 born),
  `PROGRESS.md` (+ `PROGRESS-archive.md`: the 2026-11-16 wheel-pan-parity entry compacted
  verbatim — the file hit the ~200-line budget).
- **Verified:** baseline rules.sh HARD-green + fmt clean before the edit; three in-sandbox
  `trunk build --release` runs (trunk 0.22.1, bindgen 0.2.130, wasm-opt 131, toolchain 1.99.1):
  the warning reproduces with `strip = true` and is gone with `"debuginfo"`; dist/ wasm
  5,155,501 → 5,156,370 B (+869 B — the real feature section), JS 76,952 → 78,750 B raw
  (+112 B gzipped) — measured pairs in PERF §1. Manifest-only change → clippy/test lanes not
  re-run (no Rust touched, the `gates` profile untouched); rules.sh + fmt re-run after.
- **Owner-checks:** V27 born — rebuild `trunk build --release` + a browser smoke (load, draw,
  zoom/pan) on the owner's machine.
- **Commit title:** `Fix the wasm-bindgen target_features warning: release strip=true → debuginfo`

---

### 2026-11-16 — `Feat:` fast dev Nix build (`russet-dev`); V23/V24 verified → F13 paid; the debt-ordering rule — owner request

- **What:** (1) the owner verified V23 + V24 via chat ("checked V23, V24") → both moved to the
  Resolved list; that satisfied F13's documented LAST pending condition (the owner's Wayland
  feel pass), so F13 is PAID (the `diagnostics` feature lives until F12 closes); F12 stays open
  as the final-design decision only. (2) The flake gained `russet-dev` (owner request: a fast
  non-release build to check that everything builds AND to poke the functionality; plain
  `nix build` / `nix run` stay release): the SAME source/lock/tests via buildRustPackage's
  supported profile knobs `buildType = checkType = "gates"` — the repo's gate lane (opt-level 0,
  no LTO; the nixos-unstable hook sources confirm `--profile $cargoBuildType` for the build,
  `cargo test --profile $cargoCheckType` for the check and `target/<triple>/$cargoBuildType`
  for the install) + a `.#russet-dev` app entry; README + SPEC §8 updated in the same edit.
  No nix in the sandbox → NOT machine-verified → V26 born. (3) AGENTS.md "Debts" gained rule 6
  (owner request): open items sorted V → F → rest, newest birth date first within a group,
  closed items leave the open tables the same edit; both DEBTS headers echo it and both files
  were reordered (V16's closed row → Resolved — bookkeeping, announced in chat).
- **Files:** `flake.nix`, `README.md`, `SPEC.md` §8, `AGENTS.md`, `DEBTS-verification.md`,
  `DEBTS-functional.md`, `SANDBOX.md` (pitfall 9: bare-dir `git apply` is a silent no-op),
  `PROGRESS.md`. NO Rust touched → cargo gates not re-run this round.
- **Verified:** rules.sh HARD-green before and after; the `--profile` mechanism quoted from the
  nixos-unstable sources (sparse clone of `pkgs/build-support/rust/`); the delivery
  byte-compared as usual.
- **Owner-checks:** V26 born — `nix build .#russet-dev` / `nix run .#russet-dev` on the Nix
  machine; plain `nix build` unchanged.
- **Commit title:** `Add the fast russet-dev Nix build (gates profile); docs: verify V23 V24, pay F13, add the debt-ordering rule`

---

### 2026-11-16 — `Fix:` GNOME Wayland title bar — winit Adwaita CSD frame (approved dep) — owner request

- **What:** on the owner's GNOME (Wayland) the window had NO title text and a double-click on
  the title bar did not maximize/restore. Root cause, verified against the pinned sources:
  eframe 0.37 enables only `winit/wayland` → winit falls back to sctk 0.19.2's `FallbackFrame`,
  whose `set_title` is a NO-OP and whose title-bar click emits only `Move` (no double-click
  logic at all); Mutter draws no server-side decorations, so nothing masked it. Fix per the
  dependency approval flow (the owner picked the option in chat): a Linux-only direct `winit`
  dependency with `wayland-csd-adwaita` — same 0.30.13 that eframe resolves, cargo unifies
  them, and winit switches to `AdwaitaFrame` (title text, Adwaita buttons, GTK-like 400 ms
  double-click maximize). Manifest + lock only, no Rust code touched (a unit test cannot reach
  winit's internal frame choice — the fix is verified by the gates + `cargo tree` + V25);
  pure-Rust deps → flake.nix `runtimeLibs` untouched; no commands changed → README untouched;
  the durable quirk moved to AGENTS.md quirks in the same edit.
- **Files:** `Cargo.toml`, `Cargo.lock` (+5 packages, 0 version bumps), `SPEC.md` §1,
  `AGENTS.md` (quirks), `DEBTS-verification.md` (V25 born), `PROGRESS.md`.
- **Verified:** verify.sh ALL GATES GREEN in-sandbox (toolchain 1.99.1 via rust-toolchain.toml):
  rules.sh HARD-green before and after, fmt clean, clippy `-D warnings` clean default + mcp +
  diagnostics, 99 default / 111 mcp tests green; `cargo tree -i sctk-adwaita` shows the feature
  unified into eframe's winit; the delivery byte-compared as usual.
- **Owner-checks:** V25 born — the GNOME Wayland pass on the owner's Linux machine (it also
  unblocks the Linux legs of V23/V24, which had no usable window bar to test against).
- **Commit title:** `Fix GNOME Wayland title bar: enable the winit Adwaita CSD frame (approved dep)`

### 2026-11-16 — `Fix:` Firefox scroll-pan notch retune + the diag→diagnostics rename — owner request

- **What:** the owner's platform pass (Windows/Firefox/Chrome; Linux not yet possible)
  narrowed V23/V24 and surfaced ONE gap: in Firefox one wheel click panned ~3× the
  Windows/Chrome notch — Firefox emits `DOM_DELTA_LINE` with 3 lines per click and eframe
  passes the DOM delta as-is (verified against eframe 0.37.0 sources), so a `Line`-unit
  delta hit `wheel_notches` as 3 notches. Fix inside the INTERIM F12-B convention: on the
  WEB backend only (`web_backend` = `cfg!(target_arch = "wasm32")`), `Line` deltas divide
  by the new `WEB_LINE_UNIT_LINES_PER_NOTCH` 3.0; native stays pass-through; the approved
  Windows/Chrome feel (15 pt/notch, F11 folded) is UNCHANGED; fractional lines stay
  proportional. (2) The rename (owner: the abbreviation was unclear): cargo feature `diag`
  → `diagnostics`, fn `diag_main_resize` → `diagnostics_main_resize`, log prefix `[diag]` →
  `[diagnostics]`; SPEC §3.4 + Cargo.toml + both DEBTS files updated. How-to: native
  `cargo run --features diagnostics`; web `trunk serve --features diagnostics` → logs in
  the browser console. F12 STAYS OPEN (the interim constants retune from the logs).
- **Files:** `src/app/camera.rs` (+ tests — args updated, +1 test), `src/app.rs`,
  `src/app/placement.rs`, `Cargo.toml`, `SPEC.md`, `DEBTS-functional.md`,
  `DEBTS-verification.md`, `PROGRESS.md`, `PROGRESS-archive.md` (the Shift-proportional
  entry moved verbatim).
- **Verified:** verify.sh ALL GATES GREEN — 99 default / 111 mcp; `--features diagnostics`
  clippy+test green (99); wasm check green with and without the feature;
  `trunk serve --features diagnostics` smoke-tested in-sandbox (builds + serves on
  localhost:8080; trunk 0.22.1 re-installed after the sandbox recycle); rules.sh
  HARD-green before/after; delivery byte-compared as usual.
- **Owner-checks:** none born — V23 narrowed (re-check Firefox after the divisor; the
  Linux leg), V24 narrowed (only the Linux/Wayland leg) — updated in DEBTS-verification.md.
- **Commit title:** `Fix Firefox wheel-pan overshoot (web Line ÷ 3 lines/notch) and rename the diag feature to diagnostics`

### 2026-11-16 — `Feat:` wheel-pan parity (F12-B interim), pin-based resize repositioning (F13-A), the temporary diag feature — owner request

- **What:** the owner picked the designs: F12 → variant B ("with a note that it is not
  the final design"; F11 is folded into the math — it was written after the Windows check),
  F13 → variant A, and approved the diagnostic patch ONLY behind a build flag. (1) F12-B:
  the new pure `app/camera.rs::wheel_notches` converts the raw delta to PRODUCT NOTCHES per
  its `unit` — `Line` passes through (winit ±1/notch on Windows/X11/Wayland-discrete),
  `Point` ÷ 100 pt (eframe-web passes the DOM pixels as-is, Chrome ≈ 100/notch — INTERIM
  calibration), `Page` × 3 — and every notch pans the FIXED `WHEEL_NOTCH_PAN` 15 pt on EVERY
  platform (`SCROLL_PAN_SPEED` 3.0 superseded; F11 folded: 3.0 × 5 = 15.0). Zoom untouched
  (sign-only — that is why it was always consistent). F12 STAYS OPEN (not final): the
  constants are retuned from diag logs. (2) F13-A: the pure `resize_step` (3 tests) pins the
  PRE-resize (window, content) rects on the FIRST changed frame and recomputes EVERY resize
  frame from the pin — Wayland/Web non-monotonic content series can no longer accumulate the
  chain drift ("wobbles"); the pin clears on the first stable frame and on title-bar drags.
  (3) The TEMPORARY `diag` cargo feature (default build compiles NONE of it): one `[diag]`
  stderr line per wheel event (unit/source/delta → notches → pan step — the calibration
  data) + per changed content rect + per repositioned floating window; on the web the lines
  reach the browser console. Removed when F12/F13 close. SPEC §3.4 + the §7 checklist
  rewritten; F11 paid (folded), F12/F13 statuses updated, V23/V24 born.
- **Files:** `src/app/camera.rs` (+ tests), `src/app.rs`, `src/app/placement.rs` (+ tests),
  `Cargo.toml`, `SPEC.md`, `DEBTS-functional.md`, `DEBTS-verification.md`, `PROGRESS.md`,
  `PROGRESS-archive.md` (the 2026-11-16 Refactor entry moved verbatim).
- **Verified:** verify.sh ALL GATES GREEN in-sandbox (the pinned 1.99.1 toolchain
  bootstrapped this session) — fmt clean; clippy -D warnings clean default/mcp/diag;
  98 default / 110 mcp tests green (+4 new: the unit→notch table, 3 × the pin semantics);
  rules.sh HARD-green before and after; delivery byte-compared as usual.
- **Owner-checks:** V23 (one notch = the same 15 pt on Windows/Wayland/Web + the ×5 feel;
  optional `[diag] wheel` logs) and V24 (no Wayland/Web resize jitter; optional
  `[diag] main resize`/`place` logs) born in DEBTS-verification.md.
- **Commit title:** `Fix wheel-pan platform parity (fixed 15 pt notch, F12-B interim + F11) and the resize window jitter (F13-A reference-based); add the temporary diag feature`

### 2026-11-16 — `Docs:` V9 verified, the wheel-pan platform gap and the resize jitter analyzed (F12/F13) — owner request

- **What:** (1) the owner's "checked V9" closes the M8 web real-mouse check in
  DEBTS-verification.md. (2) The same pass surfaced TWO platform gaps, born as
  F12 + F13 (DEBTS-functional.md), analysis-only by request ("none of this needs
  fixing yet"). F12 — wheel-pan parity: `app/camera.rs::handle_scroll`
  reads the RAW `Event::MouseWheel.delta` in its EVENT UNIT and ignores `unit`,
  verified against the pinned sources (egui 0.37.0: delta unconverted, unit
  Line/Point/Page, its own web=8 vs native=40 line speed; winit 0.30.13:
  Windows/X11 `LineDelta(±1)` per notch, Wayland discrete-or-pixel; eframe web:
  DOM pixels as-is) — one notch pans ±3 pt on Windows vs ~±300 pt on the Web.
  Zoom is immune (it uses only the SIGN). Options A/B/C in F12; the owner's
  "pan via the zoom method" maps to B/C; F11's ×5 must be RE-DECIDED after the unit
  fix (measured on the buggy speed). F13 — window jitter on Linux/Web resize:
  per-frame repositioning (`place_floating_window` → `set_window_pos`) meets
  non-monotonic content-rect series + auto-sized windows resizing in the same
  frames; options a/b/c in F13. SPEC §3.4's «40 pt per notch» describes egui's
  INTERNAL conversion, not the raw delta russet reads — rewrite on the fix.
- **Files:** `DEBTS-verification.md`, `DEBTS-functional.md`, `PROGRESS.md`,
  `PROGRESS-archive.md` — docs only, NO code touched (cargo gates not re-run).
- **Verified:** rules.sh HARD-green before and after the edit; source facts
  quoted from clones of the pinned versions (egui 0.37.0, winit 0.30.13,
  sctk 0.19.2); the delivery byte-compared as usual.
- **Owner-checks:** none born — F12/F13 are open DESIGN decisions for the owner.
- **Commit title:** `Docs: verify V9, analyze the wheel-pan platform gap and the resize window jitter (F12/F13)`

### 2026-11-16 — `Docs:` third debt triage — 8 checks verified, F11 born, zip-priority rule

- **What:** the owner's "checked V22, V21, V20, V19, V18, V17, V14, V13" closed those
  EIGHT owner-checks in DEBTS-verification.md (chat-verified, the V4/V7 precedent).
  V15's verdict becomes a NEW functional debt F11 — wheel pan ANOTHER ×5 faster:
  `SCROLL_PAN_SPEED` 3.0 → 15.0 in `src/app.rs` + the two SPEC §3.4 mentions in the
  same edit — recorded with the how-to and POSTPONED by the owner ("record it as
  something to do later"). AGENTS.md "Git & progress log" gained the owner's rule: a
  delivered `russet.zip` is used IMMEDIATELY — no `git pull`/`git clone` first
  (the .zip signals priority and saves setup time); the patch base is snapshotted
  from the unpacked .zip before the first edit. Compaction: the 2026-11-13 `UX:`
  entry moved verbatim to PROGRESS-archive.md. Open after this edit: V8, V9
  (both parked by the owner) + F3, F5, F8, F9, F10, F11.
- **Files:** `DEBTS-verification.md`, `DEBTS-functional.md`, `AGENTS.md`,
  `PROGRESS.md`, `PROGRESS-archive.md` — docs only, NO Rust code touched (the
  cargo gates are not re-run: the tree arrived stamped 2026-11-16, 94/106 green).
- **Verified:** `bash scripts/rules.sh` HARD-green before and after the edit (it
  needs no toolchain); the delivery patch applied onto the snapshotted base tree
  and byte-compared with the working tree AND the unpacked zip — all three match.
- **Owner-checks:** none born — F11 is owner-gated postponed work, not a check.
- **Commit title:** `Docs: third debt triage — verify V13 V14 V17-V22, record the wheel-pan x5 debt as F11, add the zip-priority rule`

### 2026-11-16 — `Feat:` Shift makes Shape drags proportional (Paint.NET-style) — owner request

- **What:** holding `Shift` during a Shape (U) drag constrains it to a
  proportional shape — a square for the Rectangle, a circle for the Ellipse
  (owner: "with Shift held, the shapes came out proportional, as in
  Paint Net"). The start point is the anchor corner; the free corner is
  pulled, LIVE (every frame — pressing/releasing Shift MID-drag snaps and
  releases instantly), onto the dominant-axis square: side = max(|dx|,|dy|),
  each axis keeps its drag direction, so the shape always grows toward the
  pointer; a strictly one-sided drag still yields a square (the zero axis
  extends down/right — Rust's `signum(+0.0) = 1.0`; the tests caught the
  wrong first assumption). The constrained point is STORED in the preview
  state, so the overlay preview and the commit rasterize the same geometry.
  The pure rule lives in `app/preview.rs::proportional_current`; the Line
  tool is NOT affected. SPEC §3.2 (Shape bullet) + a §3.9 row (a
  hold-gesture like Space-pan, not a consumed hotkey); V22 born.
- **Files:** `src/app/preview.rs` (modifier read, Drag constraint, the pure
  helper + module doc), `src/app/preview/tests.rs` (+5 tests); docs:
  `SPEC.md`, `DEBTS-verification.md`, `PROGRESS.md`.
- **Verified:** verify.sh ALL GATES GREEN — 94 default / 106 mcp; headless
  real-input pass through the real binary (XTEST via xdotool, a sandbox-only
  script): unconstrained dx=80/dy=30 commits 88×38, the same drag with Shift
  commits 88×88, Shift pressed MID-drag commits 78×78 (side 70 = the final
  dominant delta — the live constraint), undo restores the blank canvas
  between the drags; visual_driver.py 3/3 scenarios after the change.
- **Owner-checks:** V22 (the dominant-axis square feels like Paint.NET on
  Windows) born in DEBTS-verification.md; no other new checks.
- **Commit title:** `Add Shift-proportional shape drawing (square/circle, Paint.NET-style)`

### 2026-11-16 — `Refactor:` layers split into snapshot/composite children + two-color history chips — owner request (2 chat items)

- **What:** (1) `layers.rs` (504/500, the owner-approved split) lost its pixel
  plumbing to two child modules — PURE code motion: `layers/snapshot.rs`
  (the PNG codec `encode_snapshot`/`decode_snapshot` + the fit policy
  `fit_to_canvas`/`fit_copy`/`pixmap_of`) and `layers/composite.rs` (the
  region src-over blending); both re-exported from `layers.rs`, so every
  `layers::` path (app.rs call sites, the history_tree doc links) is
  unchanged — layers.rs 504 → 354. The tests split by topic the same way:
  `layers/tests/snapshot.rs` + `layers/tests/composite.rs`. (2) A two-color
  gesture now carries BOTH colors on its history node and the History Tree
  row's chip shows both — outline\|fill halves (owner: "the tools
  that used two colors showed two colors"): `TreeNode.color2`
  (Option), threaded `LayerStack::record_action` → `record_active_action`;
  the pure `shape_record_colors` mapping lives in the NEW child
  `app/preview/colors.rs` (placed there from the start — the mapping plus
  the wells rule would have pushed preview.rs over the 500 guard; pure code
  placement, not a split of grown content); single-color gestures (freehand,
  Line, Text, one-checkbox shape) keep one color. MCP `get_history_tree`
  dumps `color2` (null for single-color nodes).
- **Files:** NEW `src/layers/snapshot.rs`, `src/layers/composite.rs`,
  `src/layers/tests/{snapshot,composite}.rs`, `src/app/preview/colors.rs`;
  moved/edited: `layers.rs`, `layers/tests.rs`, `history_tree.rs` + tests,
  `app.rs`, `app/input.rs`, `app/preview.rs` + tests, `app/text_tool.rs`,
  `app/tree_window/paint.rs`, `app/mcp/tools.rs` + `tools/queries.rs` +
  `tools/edits.rs`; docs: SPEC §3.6, AGENTS.md (module map),
  DEBTS-verification.md (V21 born), PROGRESS.md.
- **Verified:** verify.sh ALL GATES GREEN — 89 default / 101 mcp; headless
  visual_driver.py 3/3 scenarios after the record-path change; the scripted
  9-check real-binary pass (xdotool/XTEST): LMB shape commits black\|white,
  RMB shape commits them INVERTED with the wells untouched (no swap), an
  outline-only shape stays single-color; the screenshot shows the
  outline\|fill chip halves in the History Tree window.
- **Owner-checks:** V21 (the chip halves feel on Windows) born in
  DEBTS-verification.md; no other new checks.
- **Commit title:** `Split layers into snapshot and composite children, show both colors of a two-color shape in the history tree`
### 2026-11-15 — `Fix:` dialogs centered + content-sized, Shape RMB wells inverted without Swap, Swap button renamed — owner request (4 chat items)

- **What:** (1) The modal dialogs (New, Resize Canvas, Unsaved Changes, Delete
  Layer, Error, About) opened at egui's default outer size 340×420 (AGENTS.md
  quirk: a plain Window never shrinks below the first frame) and at egui's
  cascade position — the owner's screenshots showed the fields at the top and
  the button row detached ~3/4 down, stretching toward the main-window bottom
  edge. Fix: the shared `dialog_window` builder (pivot CENTER_CENTER +
  `default_pos(content_rect().center())`) + `forget_dialog_position` on every
  close path — the "dialog epoch": the next open re-centers even after the
  user dragged the dialog aside; while open it stays draggable; NOT §3.11
  windows. (2) The stretch's other half: `dialog_button_row`'s bare
  `with_layout` inherited the whole remaining available rect — replaced with
  a bounded `allocate_ui_with_layout` row. (3) Shape (U) RMB: the wells are
  USED inverted (former secondary → outline, former primary → fill) WITHOUT
  swapping them — the persistent swap is gone (owner: "the colors are used
  the other way around and without Swap"); pure `shape_gesture_wells` returns `(outline, fill)`
  and has no `&mut` access to the wells. (4) Palette button "Swap (X)" →
  "Swap" (no hotkey behind it and none wanted — §3.9 stays exhaustive).
  NEW file: `src/app/dialogs/tests.rs` — 3 headless-egui regression tests
  that FAIL on the old code (measured height 420, cascade center).
- **Files:** `src/app/dialogs.rs` + NEW `dialogs/tests.rs`, `app/panels.rs`
  (Delete Layer via the builder), `app/preview.rs` + `preview/tests.rs`,
  `app/windows.rs`; docs: SPEC.md (§3.2–3.3, §3.7–3.8, §7), AGENTS.md (new
  egui quirk), DEBTS-verification.md (V19/V20 born, V16 closed), PROGRESS.md.
- **Verified:** verify.sh ALL GATES GREEN — 84 default / 96 mcp; the
  scripted headless pass (scripts/dlg_verify.py, sandbox-only) through the
  real binary 9/9: open → centered+compact → title-bar drag away → OK
  closes → reopen centered again; XTEST RMB shape: the node commits with the
  former SECONDARY color and the Palette wells read back unchanged;
  visual_driver.py 3/3 scenarios after the changes.
- **Owner-checks:** V19 (Shape RMB inverted, wells untouched), V20 (dialogs
  centered on every open, content-hugging height) born in
  DEBTS-verification.md; V16 closed as superseded by the owner's request.
- **Commit title:** `Fix dialogs to arise centered and content-sized, use Shape RMB wells inverted without swap, rename the Swap button`

### 2026-11-13 — `UX:` Tool Log restyle, About item, dialog rows, Shape RMB swap, 2x wheel pan — owner request (5 chat items)

- **What:** (1) the "History" window (NOT the History Tree) → "Tool Log" (View
  checkbox too; egui id + internal `history_*` names unchanged), entry list max
  height HALVED 280→140 pt, entries at 0.5rem = 8 pt with zero row spacing,
  first-open bottom edge FLUSH (gap 0; §3.11 keeps the dropped gap). (2) Help
  item "About Russet" → "About" (the window keeps its title). (3) Every dialog
  button row → the IBM Carbon / Apple HIG / Google Material order via the new
  shared `dialog_button_row` (right-aligned, affirmative RIGHTMOST): New, Resize
  Canvas, Unsaved Changes, Delete Layer, Error, About's Close, Text Place/Cancel.
  (4) Shape (U) right-button drag swaps the wells first (persistent, like the
  Palette Swap button) and commits the shape with the swapped wells — pure
  `shape_gesture_wells` + 3 tests. (5) Wheel pan ×2: `SCROLL_PAN_SPEED` 1.5→3.0.
  (6) F10 hand-checked by the owner — stays open (DEBTS-functional.md note).
  Found + reported, NOT fixed: the Palette "Swap (X)" button has no X hotkey
  behind it (§3.9 has none).
- **Files:** `src/app.rs`, `src/app/preview.rs` + NEW `src/app/preview/tests.rs`,
  `dialogs.rs`, `panels.rs`, `text_tool.rs`, `menu.rs`; docs: `SPEC.md`
  (§3.1–3.2, §3.4, §3.6–3.8, §3.11, §7), `AGENTS.md`, both DEBTS files.
- **Verified:** verify.sh ALL GATES GREEN — 81 tests default / 93 with
  `--features mcp` (+3 preview tests); headless `visual_driver.py` 3/3
  scenarios, 15 checks green after the preview change; rules.sh budgets green.
- **Owner-checks:** V15 (pan ×2 — the owner-requested check), V16 (Shape RMB
  swap), V17 (Tool Log restyle), V18 (dialog rows) born in
  DEBTS-verification.md.
- **Commit title:** `UX: Tool Log, About, dialog order, Shape RMB swap, 2x wheel pan`

### 2026-11-13 — `Docs:` second debt triage — owner answered the batched V/F list

- **What:** dispositions applied, zero code change. Verified via chat: V4, V7 (owner:
  "checked V4, V7" — one `cargo run` each after the toolchain pin / the file splits).
  F2 DROPPED from the debts by owner decision: the History window stays AS IS, the
  owner will say himself when it is no longer needed — not tracked anymore. F5 parked
  for the long term ("long-term parking"): memory limits not needed during active
  development. The rest reconfirmed open: V8, V9, V13, V14, F3, F8, F9, F10 — owner:
  "the rest — not now" (statuses carry the date). Open tables: V 6 → 4, F 6 → 5.
- **Files:** `DEBTS-verification.md` (V4/V7 → Resolved; V8/V9/V13/V14 notes),
  `DEBTS-functional.md` (F2 → closed; F5 parked), PROGRESS.md (entry + compaction:
  the `Feat:` real-layers and first-triage entries moved VERBATIM to
  PROGRESS-archive.md — the Log keeps the newest 5).
- **Verified:** docs-only round (AGENTS.md "Docs/comment-only edits" — no cargo run):
  rules.sh green (budgets back under the limits after the compaction, no NOT WRAPPED
  hits), `cargo fmt --check` clean; delivery patch/zip byte-verified.
- **Owner-checks:** none new; V13/V14 remain the standing pointer-feel asks for the
  next Windows pass; F3/F8/F9/F10 stay parked.
- **Commit title:** `Docs: second debt triage — verify V4 V7, drop F2, park F5`

### 2026-11-13 — `Docs:` commit titles are always backtick-wrapped strings — owner request

- **What:** the ritual fixes the title's canonical form as ONE backtick-wrapped
  string in BOTH places it is stated (the PROGRESS.md `Commit title:` field and the
  final chat reply's last line): in the chat UI a backticked string is click-to-copy,
  one click puts the exact title on the owner's clipboard. rules.sh rule 6 stopped
  stripping the backticks at the done-gate (the hand-over string is now verbatim
  end-to-end) and warns NOT WRAPPED when the newest entry's title lacks them. The
  Delivery ritual's patch bullet also documents the new base: with the owner
  committing deliveries locally, the base is the PRE-EDIT tree object
  (`git write-tree`, no commit/ref created), not an old sandbox SHA.
- **Files:** `AGENTS.md` ("Commit titles" bullet + "Delivery ritual"),
  `scripts/rules.sh` (rule 6: backticks kept + NOT WRAPPED warning),
  PROGRESS.md (entry + snapshot row).
- **Verified:** docs-only round (AGENTS.md "Docs/comment-only edits" — no cargo
  needed): rules.sh green, its reminder now prints the title WITH backticks, no
  NOT WRAPPED hits; `cargo fmt --check` clean; delivery patch/zip byte-verified.
- **Owner-checks:** none new; the open V/F lists are unchanged (batched ask in the
  chat reply).
- **Commit title:** `Docs: wrap commit titles in backticks for click-to-copy in chat`

### 2026-11-13 — `Fix:` Layers window width + the white initial layer — owner report (3 chat items)

- **What:** (1) the Layers window smeared screen-wide and did not fit the main window:
  `.auto_sized()`'s FIRST layout sees `available_width` = the whole screen and
  `desired_size` never shrinks, while the row names derived their width from it — a
  self-feeding per-frame growth loop. Now `.default_width(240)` + `.resizable(false)`
  (auto-sized was non-resizable too) with a FIXED 120 pt name column, text clipped so
  long names cannot run under the buttons; the egui quirk is recorded in AGENTS.md.
  (2) A NEW document's initial layer is WHITE again (§3.8): `LayerStack::new` fills
  the Start canvas, layers added later stay transparent; `App::new`/`reset_canvas`
  now composite the fresh stack into the display buffer once (it is created
  transparent). (3) The third item was owner praise for the node-dot history switching
  (no change needed).
- **Files:** `src/app/panels.rs`, `src/layers.rs` + tests, `src/app.rs`,
  `src/app/document.rs` (composite on New), `scripts/visual_driver.py` (+1 check),
  `SPEC.md` §3.6, `AGENTS.md` (quirk), PROGRESS.md.
- **Verified:** verify.sh ALL GATES GREEN — 78 default / 90 mcp; headless driver 3/3
  scenarios, 15 checks incl. the new fresh-white check; Xvfb screenshot: Layers ~240 pt
  wide, fits the main window, canvas white with strokes.
- **Owner-checks:** V14 in DEBTS-verification.md (Windows click-through: window fits,
  New canvas white, added layers transparent).
- **Commit title:** `Fix the Layers window width and restore the white initial layer`

### 2026-11-13 — `Refactor:` mcp/tools split into themed modules + commit-title delivery ritual — owner request (3 chat items)

- **What:** (1) `mcp/tools.rs` (499, guard at 500) split PURE CODE MOTION, style of the earlier
  splits: the root keeps the MAP of the surface — dispatch `tool_call` + `tool_declarations` +
  reply glue (`tool_text`/`tool_error`/`hex_color`) — 157 lines; handlers themed into
  `tools/queries.rs` (read-only: layers/trees/layout/snapshot/export, 200) and `tools/edits.rs`
  (mutating: stroke/undo/redo/add_layer/set_active_layer + frame-free pure helpers, 177).
  Adding a tool = one dispatch arm + one declaration in the root, one themed handler. LLM
  navigation guarded: both children's docs state what they own; `pub(super)` with caller notes;
  `tests.rs` imports the moved helpers explicitly. (2) Commit-title ritual: rules.sh rule 6
  prints the newest entry's Commit title as a REMINDER at every done-gate — it fires exactly
  before the final reply is composed; AGENTS.md "Delivery ritual" requires the reply to carry
  it verbatim (owner: missed repeatedly).
- **Files:** split `src/app/mcp/tools.rs` → NEW `src/app/mcp/tools/queries.rs`,
  `src/app/mcp/tools/edits.rs`; `src/app/mcp/tools/tests.rs` (imports); `scripts/rules.sh`
  (rule 6); `AGENTS.md` (Delivery ritual); PROGRESS.md (entry + archive move + snapshot).
- **Verified:** verify.sh ALL GATES GREEN — 78 default / 90 mcp; visual_driver.py headless
  3/3 scenarios, 14 checks green AFTER the split (pure motion proven); rules.sh self-test
  prints the title.
- **Owner-checks:** skim `tools.rs` — does the root-as-map layout read well (same ask as V7);
  the ritual wording in AGENTS.md.
- **Commit title:** `Split MCP tools into a root map plus queries/edits modules and enforce the commit-title ritual`

### Warnings — resolved (moved 2026-11-13 from PROGRESS.md)

- 2026-11-13 — RESOLVED: the soft 500 guard kept flagging `app/tree_window.rs` (687) and
  `history_tree.rs` (600). With the owner's approval both are split — painter →
  `app/tree_window/paint.rs`, layout → `history_tree/layout.rs` — and, also on the owner's
  request, the owner-exempt `app.rs` (936) was slimmed into `app/document.rs` + `app/menu.rs` +
  `app/input.rs`. Pure code motion, re-exports keep the old paths, no behavior change; every
  src file is ≤ ~440 lines now.

- 2026-11-10 — RESOLVED: `src/tools.rs` was **679 lines**; moving its inline tests to
  `src/tools/tests.rs` (2026-11-10, PR rebase) brought it to ~430 lines, so the proposed
  `raster.rs` split is retired. `src/app.rs` is 848 lines — still within the owner-approved
  orchestrator exception (roughly doubled would be ~1400).

- 2026-11-09 — RESOLVED: `src/app.rs` (1634 lines) was split into four child modules
  (`app/camera.rs`, `app/dialogs.rs`, `app/windows.rs`, `app/placement.rs`). `app.rs` itself
  remains ~690 lines BY OWNER'S DECISION — it stays the single orchestrator and may exceed
  the 500-line guard; do not propose splitting it further unless it roughly doubles
  (see the exception in AGENTS.md "Repo facts").

---

### 2026-11-13 — `Tooling:` headless-GUI recipe — the sandbox CAN run the GUI after all — owner request ("record the experience in a .md")

- **What:** the previous round's "GUI impossible here" was wrong. NEW `SANDBOX.md` records the
  verified recipe: raw Xvfb (not xvfb-run — no xauth) + two user-space .debs
  (`libxkbcommon-x11-0`, `libxcb-xkb1`) + unversioned symlink + LD_LIBRARY_PATH /
  LIBGL_ALWAYS_SOFTWARE, plus pitfalls 1-8 (silent transitive deps, pipe-hanging
  backgrounds, `timeout -k`, no Wayland fallback). NEW `scripts/gui_recipe_test.sh` is the
  executable proof (MCP `initialize` answers from a live window). Follow-up fixes it unlocked:
  MCP `add_layer` (parity with the Layers-window button; 9→10 tools) — without it no scenario
  could address a second layer, which is why two driver scenarios sat stale; their expectations
  corrected (tree rows include the Start root).
- **Files:** NEW `SANDBOX.md`, `scripts/gui_recipe_test.sh`; `src/app/mcp/tools.rs` (+tests:
  declaration count 9→10), `scripts/visual_driver.py`; AGENTS.md Commands bullet;
  DEBTS-verification.md V13 narrowed; PROGRESS.md snapshot+stamp.
- **Verified:** `bash scripts/verify.sh` ALL GATES GREEN — 78 tests default, 90 with
  `--features mcp`; `visual_driver.py` headless — ALL 3 scenarios green, 14 checks (strokes,
  undo/redo pixel restore, git-style fork, per-layer tree independence) through the real binary.
- **Owner-checks:** V13 narrows to real-pointer feel (node-dot hit areas, double-click rename,
  row switching); F8/F9/F10 stay open as agreed.
- **Commit title:** `Add SANDBOX headless-GUI recipe, MCP add_layer tool and green driver scenarios`


---

### 2026-11-13 — `Feat:` real layers + per-node PNG history + node-dot switching — owner request ("a chunk of the big feature")

- **What:** owner's staged rollout of the history design, decisions fixed in chat:
  ALL tools record nodes; fork-after-jump git-style; Start root; no node limit;
  ALL layers transparent; Resize KEEPS nodes (old PNGs crop/pad-fitted on restore,
  per his wording). (1) NEW top-level `layers.rs` (+tests): `Layer`/`LayerStack`,
  lossless fast PNG snapshots (`image` crate `CompressionType::Fast` — roundtrip
  bit-exact, no new deps), `fit_to_canvas`, premultiplied region composite. (2)
  `history_tree.rs` became the REAL per-layer DAG (`LayerTree` owned by each layer;
  ids from one document counter); the 20-snapshot stack in `history.rs` is GONE
  (only `ActionLog` remains). (3) Canvas = composite display buffer; gestures paint
  the ACTIVE layer; undo/redo/jump restore from the head's PNG. (4) Layers window:
  select/rename/visibility/reorder/delete-with-confirm; the tree-window switcher is
  now a passive label. (5) History Tree windows: LEFT-click a node DOT (not the
  label) switches the canvas; debug "-" also restores pixels. (6) MCP: tools work on
  real layers, `add_stroke` records a node. NEW debts F8 (no eviction), F9 (resize
  interim policy), F10 (undo on the active layer only); F1+F4 paid; V13 born.
- **Files:** NEW `src/layers.rs`, `src/layers/tests.rs`, `src/history_tree/tests/layout.rs`;
  rewritten `src/history_tree.rs` (+tests), `src/history.rs` (+tests), `src/app.rs`,
  `src/app/{input,preview,text_tool,dialogs,document,panels,menu}.rs`,
  `src/app/tree_window.rs` + `paint.rs`, `src/app/mcp/tools.rs` (+tests), `src/main.rs`
  (+`mod layers`); docs: SPEC §1/§3.6/§6, AGENTS.md module map, DEBTS-*.md, PROGRESS.md.
- **Verified:** `bash scripts/verify.sh` ALL GATES GREEN — 78 tests default, 90 with
  `--features mcp`; baseline was green before edits (rules.sh + fmt). The GUI could
  NOT be run in this sandbox (no libxkbcommon-x11, no root — even headless start
  panics), so no Xvfb/MCP-smoke pass this round; the mcp-gated compile+tests cover
  the wiring frame-free.
- **Owner-checks:** V13 click-through (layers ops, node-dot switching, fork after
  old-node draw, resize-fitted restore, Save composite); say the word if your remark
  "cargo run works great" closes V4/V7.
- **Commit title:** `Add real layers, per-node PNG history trees and node-dot canvas switching`

---

### 2026-11-13 — `Docs:` first debt-triage round — owner answered the batched V/F list

- **What:** owner dispositions applied to the two debt files; zero code change.
  Verified via chat: V1, V2, V3, V11 (Windows tool/UX/About checks), V5+V6 (History
  Tree — "tried it by hand, it suits me"; later improvements = new tasks), V12 (docs
  skimmed, dev stable). Closed as not-a-debt: V10 (owner builds on Nix periodically
  himself), F6 (line-cap circles fine for now), F7 (popup shadow stays). Parked by
  the owner, stay open: V8 (no Windows MCP pass for now), V9 (no web checks soon).
  Awaiting a one-word answer after a plain-words explanation: V4 (one `cargo run`
  after the toolchain pin + `gates` profile) and V7 (`cargo run` after the file
  splits — nothing was supposed to change). Open tables: V 12 → 4, F 7 → 5.
- **Files:** `DEBTS-verification.md`, `DEBTS-functional.md`, `PROGRESS.md` (stale
  "two 2026-11-10 entries" note removed — V1/V2 verified; the 2026-11-12 MCP entry
  → PROGRESS-archive.md, the Log keeps the newest 5).
- **Verified:** docs-only edit (AGENTS.md "Docs/comment-only edits" — no cargo run):
  rules.sh green — PROGRESS.md and Log under budget, no parked RESOLVED warnings,
  5 live owner-check lines in the Log; `cargo fmt --check` clean.
- **Owner-checks:** say the word on V4 and V7 (explained in chat) — one word each
  closes them; V8/V9 stay parked until you plan those checks; F1–F5 unchanged.
- **Commit title:** `Docs: first debt triage — verify V1-V3 V5 V6 V11 V12, close V10 F6 F7, park V8 V9`

---

### 2026-11-13 — `Refactor:` split the 500-guard files and slim app.rs into child modules — owner request

- **What:** ALL pure code motion, zero behavior change, consumer paths preserved.
  (1) `history_tree.rs` 600 → 350: `Row`/`BranchSpan`/`TreeLayout` + `layout_layer` moved to
  child `history_tree/layout.rs` (281) and re-exported, so `tree_window.rs`, the MCP tools and
  `tests.rs` keep the `history_tree::` paths untouched. (2) `app/tree_window.rs` 687 → 433:
  the painter + `GraphMetrics` + geometry constants + palette moved to child
  `app/tree_window/paint.rs` (269); the parent keeps windows/clones/scroll bookkeeping.
  (3) `app.rs` 936 → 439 (owner request "make app.rs smaller"): document lifecycle
  (New/Open/Save, unsaved-changes gate, window title) → `app/document.rs`; top menu bar →
  `app/menu.rs`; hotkeys + freehand strokes + Eyedropper → `app/input.rs`. `app.rs` keeps App
  state, frame flow, undo/redo, canvas widget, texture sync. LLM navigation guarded: every new
  module's doc states what it owns and what its neighbours own; the module map at the top of
  `app.rs` lists all 12 child modules; cross-module methods are `pub(super)` with caller notes.
- **Files:** NEW `src/history_tree/layout.rs`, `src/app/tree_window/paint.rs`,
  `src/app/document.rs`, `src/app/menu.rs`, `src/app/input.rs`; trimmed `src/history_tree.rs`,
  `src/app/tree_window.rs`, `src/app.rs`; `src/app/dialogs.rs` (one import now points at
  `document`); docs synced: AGENTS.md (module split, cfg rule, app.rs exception note), SPEC §2
  + §3.10, PERF.md path mentions, PROGRESS.md.
- **Verified:** `bash scripts/verify.sh` ALL GATES GREEN — 69 tests default, 81 with
  `--features mcp`; rules.sh reports the size guard fully green for the first time since the
  tree landed (no file over 500).
- **Owner-checks:** `cargo run` must behave exactly as before (menu, hotkeys, undo/redo, tree
  windows, MCP badge) — nothing was supposed to change; skim the module map at the top of
  `app.rs` and say if the grouping reads well.
- **Commit title:** `Split the guarded files and slim app.rs into layout/painter/document/menu/input child modules`

---

### 2026-11-12 — `Feat:` opt-in MCP stdio server behind the `mcp` feature — owner request

- **What:** a JSON-RPC 2.0 stdio server for agent-driven development, compiled ONLY
  with `--features mcp`. A reader thread pushes parsed events into an mpsc queue; the
  UI frame polls it with `try_recv` (main thread never blocks) and executes 8 tools
  against the live App: get_layers, get_history_tree, add_stroke (the exact
  hand-gesture path: snapshot → pixels → tree node → forks after undo), undo/redo,
  set_active_layer, get_canvas_snapshot (PNG image content), export_png. Starts on
  `--mcp` or piped stdin; menu-bar badge shows "MCP on (N)". serde_json (optional
  dep) + the thread pump are owner-approved 2026-11-12 via the new approval flow;
  the default build has NO mcp code/threads/serde_json (`cargo tree` proof) —
  release: 14.52 MB default vs 14.71 MB mcp (+189 KB).
- **Files:** Cargo.toml (+`[features] mcp`, optional serde_json, lock updated);
  NEW src/app/mcp.rs + mcp/tools.rs + mcp/tests.rs + mcp/tools/tests.rs; app.rs (5
  cfg hooks); history_tree.rs (read-only layer_nodes/layer_head); file_io.rs
  (pub(crate) encode_png); README.md (MCP build section). tree_window.rs untouched.
- **Verified:** verify.sh ALL GATES GREEN (64 tests); `--features mcp`: clippy
  clean, 76 tests. Headless demo: 11/11 JSON-RPC requests OK — house drawn via
  add_stroke, snapshot decoded to PNG, undo, fork 2:[3,4] confirmed by
  get_history_tree; 2 screenshots + transcript in download/mcp-demo/.
- **Owner-checks:** Windows run of the flag build (`cargo build --release --features
  mcp`, then `--mcp`); badge look; the tree_window.rs review is still pending.
- **Commit title:** `Add opt-in MCP stdio server behind the mcp feature`

---

### 2026-11-12 — `Docs:` dependency convention reworked: hard ban → owner approval flow — owner request

- **What:** the "Do not add dependencies" rule is replaced by an approval flow:
  the agent stops and proposes (why it is needed / candidate crates with
  trade-offs / the no-dependency alternative / a recommendation), the owner
  decides. First live case already queued: the MCP server's JSON layer
  (serde_json vs the official rmcp SDK vs an in-tree codec) — the question goes
  to the owner before any MCP code lands.
- **Files:** AGENTS.md (new "Dependencies — approval flow" section + the Repo
  facts bullet), SPEC.md §6 (approval-flow exception paragraph), CONTRIBUTING.md
  (outsider rules now point at the flow), PROGRESS.md (this entry).
- **Verified:** docs-only edit — no cargo run (no source or manifest change;
  scripts/rules.sh greps src/ only).
- **Owner-checks:** the wording of the flow; the pending MCP JSON decision.
- **Commit title:** `Replace the no-dependencies ban with an owner approval flow`

### 2026-11-13 — `Fix:` History Tree rendering rework (owner review of 04_fork screenshot) — owner request

- **What:** seven owner fixes in one pass. (1) Fork connectors: the old per-row rail
  segments drew a stray stub through the parent's row and the bezier bulged wrong —
  replaced with ONE rail segment per branch plus an explicit connector: straight
  horizontal run → quarter ellipse (cubic, r = LANE_STEP/2) → straight rail to the
  node; successive forks from one parent stagger their runs one step per crossed
  lane (the drop hides inside the parent's rail). (2) ONE color per action: nodes
  store/display `TreeNode::color` — no primary/secondary pair (capture via the
  existing `App::stroke_color()` at stroke start). (3) Horizontal scrolling:
  `ScrollArea::both`; content width = gutter + label column. (4) No-merge invariant
  documented (paint branches never converge → rails are plain segments, no join
  handling). (5) VS Code-style gutter: all dots/rails stay inside
  `lanes × LANE_STEP`; labels share one aligned x after it — no line/text
  collisions by construction. (6+7) Lane assignment: the CURRENT timeline
  (root..head) owns lane 0 as the straight left edge; abandoned tails hang off to
  the right in creation order (spans now carry `parent_row` for the connector).
- **Files:** `src/history_tree.rs` (model: one color, head-path lanes, spans),
  `src/history_tree/tests.rs` (rewrote fork expectations, +3 tests),
  `src/app/tree_window.rs` (painter rewrite: rails/connectors/gutter/h-scroll,
  single swatch), `src/app.rs` (stroke_colors → stroke_paint), `PROGRESS.md`.
- **Verified:** `bash scripts/verify.sh` — ALL GATES GREEN (rules: 0 hard violations;
  fmt clean; clippy `-D warnings` clean; `cargo test --profile gates` 64 green).
  Headless Xvfb + xdotool screenshots (reviewed): fork keeps lane 0 straight with
  the tail peeling off via the quarter ellipse; 11-branch fan renders staggered
  arcs without overlaps; labels stay aligned and collision-free; shift+wheel
  scrolls horizontally (labels/swatches revealed); "New at top" mirrors the
  connectors upward; single swatch per row.
- **Owner-checks:** `cargo run` on Windows — review the new connector shape and
  lane order against the taste from the 04_fork screenshot; try many undos+strokes
  at one node and scroll the tree right; check the single color square reads well.
  `src/app/tree_window.rs` is 668 lines — still over the soft 500 guard; the
  proposed `tree_window/paint.rs` split is still pending the owner's OK.
- **Commit title:** `Rework History Tree rendering: lane-0 head path, quarter-ellipse connectors, single color, guttered labels, horizontal scroll`

### 2026-11-12 — `Extra:` visual "History Tree" windows (git-style graph prototype) — owner request

- **What:** purely VISUAL history tree per the owner's design concept + live answers:
  model `src/history_tree.rs` — 3 pseudo-layers with one GLOBAL active layer; nodes carry
  kind + both tool colors only; undo/redo mirror the real `History` after it succeeded
  (pencil-only sessions stay exact; interleaved brush/eraser strokes are a known
  prototype caveat, commented at `App::undo`); a stroke after undo forks the graph,
  old tail stays; debug `remove_last` returns forward/backward retarget hints; New/Open
  reset. Window `src/app/tree_window.rs` — floating main window at the OLD History spot;
  toolbar per window: layer switcher, direction toggle ("New at bottom" top-down default /
  "New at top" bottom-up), Clone (cascade; inherits direction + scroll anchor), debug "−";
  rows = commit circle + "pencil" + two stacked color squares exactly one text line tall;
  branch rails + fork curves (VS Code-graph look), head ring; scrolling: at the graph's
  end the view auto-follows, scrolled away it pins to the top visible node, removal
  retargets pins FORWARD (backward as last resort; at-the-end windows keep following).
  Old "History" window first-opens at the BOTTOM CENTER. View menu: "History Tree" toggle.
  Not wired to any drawing/undo behavior — a read-only mirror, pencil strokes only.
- **Files:** new `src/history_tree.rs`, `src/history_tree/tests.rs`, `src/app/tree_window.rs`;
  `src/app.rs` (fields + stroke/undo/redo/reset hooks + View menu), `src/app/panels.rs`
  (History first-open → bottom center), `src/app/placement.rs` (`WindowPlacement` gained
  Clone+Copy), `src/main.rs` (`mod history_tree`), `PROGRESS.md`.
- **Verified:** `bash scripts/verify.sh` — ALL GATES GREEN (rules: 0 hard violations; fmt
  clean; clippy `-D warnings` clean; `cargo test --profile gates` 61 green = 49 + 12 new).
  Headless smoke run under Xvfb + xdotool (screenshots reviewed, app log panic-free):
  strokes append nodes; Ctrl+Z then a stroke forks with the old tail kept; redo returns
  via the left-branch hint; Clone inherits direction + anchor; global layer switch resets
  both windows; at-the-end auto-follow; scrolled-away pin holds through removals; removing
  nodes keeps pinned windows consistent without jumping to the end; clone X removes it.
- **Owner-checks:** `cargo run` on Windows — draw pencil strokes, Ctrl+Z/Y, Clone, layer
  switch, direction toggle, debug "−"; confirm the scroll behavior feels right.
  `src/app/tree_window.rs` is 607 lines — OVER the soft 500 guard; propose splitting the
  graph painter into `src/app/tree_window/paint.rs` in a later task (needs owner's OK).
- **Commit title:** `Add visual History Tree windows with pseudo-layers, clones and anchored scrolling; move History window to bottom center`

### 2026-11-11 — `Tooling:` Rust pinned via rust-toolchain.toml; unoptimized `gates` profile for gate runs — owner request

- **What:** new `rust-toolchain.toml` (channel 1.99.1, minimal + clippy/rustfmt) — rustup installs the exact compiler on the first cargo command in the repo, so every machine and agent session builds with the same toolchain; new `[profile.gates]` in `Cargo.toml` (inherits dev; opt-level 0, no debuginfo, no incremental, dependencies included) — gate runs skip the egui-template dep optimization that pure-logic tests never use; `verify.sh` clippy/test now run `--profile gates`; AGENTS.md (Commands + Repo facts), SPEC §8 and README updated in the same edit. CI untouched (needs the owner's OK) — it keeps the full optimized dev lane as the independent full-profile check. GUI dev lane unchanged.
- **Files:** new `rust-toolchain.toml`; `Cargo.toml`, `scripts/verify.sh`, `AGENTS.md`, `SPEC.md`, `README.md`, `PROGRESS.md`.
- **Verified:** cold start in the agent sandbox (empty `target/`, 2 cores): `cargo fmt --check` 0.1s; `cargo clippy --profile gates --all-targets -- -D warnings` clean — 1m24s; `cargo test --profile gates` 49 green — 1m42s; gate total ~3m (was ~12m45s on the same box with the default profile). Then `bash scripts/verify.sh` — ALL GATES GREEN (rules 0 hard violations; warm rerun ~5s).
- **Owner-checks:** one `cargo run` on Windows (dev lane untouched); the next CI run after push — the workflow installs 1.99.1 explicitly, which matches the new pin; `rust-toolchain.toml` is new to the repo.
- **Commit title:** `Pin Rust 1.99.1 via rust-toolchain.toml; add unoptimized gates profile for cold-start test runs`

### 2026-11-11 — `UX-fix:` History/Layers under §3.11; Tools 40/60 first open; shadows off — owner request (3 items)
- **Commit title:** `Place History/Layers windows per §3.11; lift Tools first open; drop window shadows`
- **What:** (1) History and Layers now follow the SAME §3.11 placement as
  Tools/Palette — drop-time 2rem edge anchoring (at drag end only, never pulled
  flush) and per-axis resize repositioning; their first-open anchors are
  unchanged (`history_placement`/`layers_placement` added to `App`). (2) The
  Tools window's first open is a bit ABOVE center: 40% of the FREE vertical
  space above / 60% below (owner: "40% of the gap to the top edge,
  60% to the bottom"). A pivot + `default_pos` cannot express this (the auto-sized height is
  unknown before the first layout), so the exact split is a ONE-SHOT fix on the
  first frame — `TOOLS_FIRST_OPEN_RATIO` + `first_open_ratio_pos` in
  placement.rs, persisted via the same `set_window_pos` path as §3.11; §3.11
  takes over right after, reopening the window never re-fires it. (3) Window
  drop shadows are OFF app-wide: `visuals.window_shadow = Shadow::NONE` in
  `App::new`, both themes, every `egui::Window` (dialogs included); popup/menu
  shadows deliberately untouched. Overrides §6's shadow item — recorded in SPEC
  §3.11; SPEC §3.2/§7.1/§7.14 + AGENTS.md repo facts updated in the same edit.
- **Files:** `src/app/placement.rs` (+ pure `first_open_ratio_pos`, 1 test),
  `src/app/panels.rs` (placement glue for both windows), `src/app.rs` (two
  placement fields, Tools ratio, shadow style), `src/app/windows.rs` (comment);
  docs: `SPEC.md`, `AGENTS.md`, `PROGRESS.md`.
- **Verified:** `bash scripts/verify.sh` — ALL GATES GREEN: rules.sh (0 hard
  violations, the same 5 known expect() NOTEs, size guard OK), fmt clean,
  clippy `-D warnings` clean, `cargo test` 49 green (48 → 49: the 40/60 split
  test; Rust 1.99.1, Linux — toolchain installed fresh in this sandbox). No
  wasm check (no new `#[cfg]` branches, glue-only change).
- **Owner must check:** on Windows `cargo run` — (1) History/Layers magnetize
  at dropped edges and follow main-window resizes like Tools/Palette; (2) the
  Tools window opens at the ~40/60 split and keeps it through resizes; (3) no
  shadows under the floating windows/dialogs — the color-picker popup still
  has one (say the word to drop that too).
### 2026-11-11 — `CI:` machine rules job — scripts/rules.sh wired into CI and verify.sh — owner request
- **Commit title:** `Add scripts/rules.sh machine rules; run them in CI and verify.sh`
- **What:** the owner-approved "machine-readable rules" round. New `scripts/rules.sh` — the
  grep-level subset of AGENTS.md, no Rust toolchain needed: HARD (exit 1) = inline test
  modules (`mod tests {` in `src/`) and `unwrap()` anywhere in `src/` (tests included, per
  "Tests" rule 6); SOFT report-only = `expect()` outside test files (each hit must stay off
  user-data paths — the 5 existing ones are wasm boot + one internal invariant) and the
  500-line file-size guard (moved here from verify.sh, single source of truth).
  `.github/workflows/ci.yml` gained a fast `rules` job (checkout + the script, no toolchain
  install) next to `native`. `scripts/verify.sh` now runs rules.sh first, so the local gate
  fails before cargo even starts. Docs updated: AGENTS.md (Commands, Repo facts),
  CONTRIBUTING.md (gates), PR template (checkbox).
- **Files:** new `scripts/rules.sh`; `scripts/verify.sh`, `.github/workflows/ci.yml`,
  `AGENTS.md`, `CONTRIBUTING.md`, `.github/pull_request_template.md`, `PROGRESS.md`.
- **Verified:** `bash scripts/rules.sh` green in this sandbox on the pushed master (0 hard
  violations; 5 expect() NOTEs; size guard OK); negative tests — an injected `.unwrap()` in
  a tests.rs and an injected `mod tests {` in canvas.rs both fail with exit 1, `unwrap_or`
  does not match, an injected 501-line file reports OVER GUARD but exits 0; ci.yml parsed
  with python yaml.safe_load. No Rust code touched — cargo gates unchanged; the stamp below
  stays at its last real run (and the FIRST CI run for `0188c52` came back success).
- **Owner must check:** the two green jobs (rules + native) in the next Actions run after
  pushing this; skim the 5 expect() NOTE lines — they are the tolerated exceptions.
### 2026-11-11 — `CI+Docs:` CI workflow, CONTRIBUTING.md + PR template, doc trees removed, trunk pinned — owner request
- **Commit title:** `Add CI (fmt/clippy/test), CONTRIBUTING.md and PR rules; drop doc file trees; pin trunk 0.22.1`
- **What:** per the owner's request: (1) `.github/workflows/ci.yml` — the done-gate
  (fmt/clippy/test) on every push to `master` and every PR, Rust pinned 1.99.1; wasm job
  deliberately postponed by the owner. (2) `CONTRIBUTING.md` for outsiders +
  `.github/pull_request_template.md` enforcing ONE logical change per non-owner PR (review
  target: an LLM reviewer; the owner pushes directly). (3) File trees removed from AGENTS.md
  and SPEC §2 (owner decision: the filesystem is the source of truth; rules kept as prose).
  (4) Trunk pinned to 0.22.1 in README/SPEC §8/AGENTS, removal condition for
  `data-wasm-opt-params` written into `web/index.html`, PERF P9 added. (5) README: fixed the
  truncated last sentence and the stale `index.html` path; added the Contributing pointer.
- **Files:** new `.github/workflows/ci.yml`, `.github/pull_request_template.md`,
  `CONTRIBUTING.md`; `AGENTS.md`, `SPEC.md`, `README.md`, `PERF.md`, `PROGRESS.md`,
  `web/index.html`.
- **Verified:** docs/config-only batch (AGENTS.md "Commands": cargo check is enough), but cargo
  is not installed in this sandbox, so NO Rust gate was re-run here — the stamp below stays at
  its last real run; `ci.yml` parsed with python yaml.safe_load; delivery byte-compared per
  "Delivery ritual". The FIRST PUSH runs the new CI — that is the independent verification.
- **Owner must check:** the first Actions run after pushing (repo → Actions tab, ~3–5 min);
  CONTRIBUTING.md reads correctly for a human contributor; P3/P6 decisions still pending.
### 2026-11-10 — `PR:` rebased on master; new inline tests moved to `tests.rs`; PERF.md demoted to a reference
- **Commit title:** `Rebase PR onto master; move new inline tests to tests.rs; make PERF.md a reference backlog`
- **What:** rebased the open PR onto current `master` (WASD/0.5 rem first-open gaps, six
  tools/panels, reading routes/delivery ritual, 2026-11-10 UX fixes) and resolved the conflicts:
  `AGENTS.md` keeps master's reading routes / delivery ritual / commit-title rules **plus** the
  "Tests — separate files" section and the project-layout tree; `PROGRESS.md` keeps master's
  entries. Converted the new upstream inline tests: `tools.rs` 16 tests → `tools/tests.rs` (+2 of
  ours = 18), `text.rs` 3 → new `text/tests.rs`, `history.rs` 2 `ActionLog` tests →
  `history/tests.rs`. `tools.rs` dropped 679 → 429 lines, so the proposed `raster.rs` split is
  retired. `PERF.md` no longer tells the agent to start on its own — it is a reference backlog the
  owner picks from. Review item ".gitignore lacks dist/" was a false positive: `/dist` was already
  ignored (verified with `git check-ignore`).
- **Files:** `AGENTS.md`, `PROGRESS.md`, `PERF.md`, `web/index.html`, `SPEC.md`; `src/tools.rs`,
  `src/history.rs`, `src/text.rs` + `src/{tools,history,text}/tests.rs`.
- **Verified:** `bash scripts/verify.sh` — ALL GATES GREEN (fmt/clippy clean, `cargo test` 48 green,
  file-size guard OK); `cargo check --target wasm32-unknown-unknown`; `trunk build --release` → `dist/`.
- **Owner must check:** interactive web pass (real-mouse draw/zoom/pan, Open/Save); decide on
  `PERF.md` P3 (region undo) and P6 (web budgets).
### 2026-11-09 — `Docs:` README.md added (build & run for humans) — owner request
- **What:** new `README.md` in English, deliberately MINIMAL per the owner's instruction: it contains ONLY how to build and run the app for the three supported targets — Windows (`cargo run`, `cargo build --release`), NixOS/Linux (`nix build` / `nix run` / `nix develop`, flake enablement one-liner, `WINIT_UNIX_BACKEND` override, non-NixOS distros welcome) and Web/WASM (wasm32 target, `trunk serve`, `trunk build --release` → `dist/`, one example of serving it). No feature descriptions, no changelog. The fact that README.md is the HUMAN-facing document (agents read AGENTS.md/SPEC.md instead) is recorded in AGENTS.md ("Repo facts") and SPEC §8, with a keep-in-sync rule when commands change.
- **Files:** new `README.md`; `AGENTS.md` (README audience bullet), `SPEC.md` (§8 pointer + note that Linux builds go through `flake.nix` since 2026-11-09).
- **Verified:** documentation-only change — no code touched; commands cross-checked against SPEC §8, `flake.nix` and `index.html` (trunk entry).
- **Owner must check:** the three command blocks read correctly on Windows PowerShell vs bash; nothing else.
### 2026-11-09 — `Docs:` `PERF.md` — RAM / build-speed backlog + first release-build measurement
- **What:** new `PERF.md` (design note, not part of the spec) collects everything about memory and
  build cost: (1) a table of where RAM goes for 800×600 / 2000×1500 / 4096×4096 (pixmap + texture +
  mips + full-snapshot history) and the wasm "heap never shrinks" caveat with the transient spikes
  (resize, open, undo); (2) a ranked backlog — A. region-based undo instead of full snapshots
  (10–100× less history, no full-canvas memcpy per stroke; needs a SPEC §3.6 decision), B. mipmaps
  only at zoom < 100 % (`egui_glow` regenerates the whole mip chain on every `set_partial` —
  `painter.rs:632`), C. `into_rgba8()` + no intermediate copy on open, D. wasm-only eframe features
  (drop `accesskit`/`system_fonts`), release log level, wasm-opt (§6 exception), `panic=abort`,
  E. canvas/history budgets on web, F. micro-optimizations; (3) build-time options (thin LTO, codegen
  units, opt-level) with the measured baseline; (4) measurement recipes for browser + native.
  **First release measurement** on this Linux box (24 threads; `jobs = 8` and `-C target-cpu=native`
  come from the owner's global `~/.cargo/config.toml`): cold `cargo build --release` **243 s**,
  one-file relink after `touch src/main.rs` **160 s**, no-op **6 s**, stripped binary **14.1 MiB**.
  No application code changed.
- **Files:** new `PERF.md`; the layout trees in `AGENTS.md` and `SPEC.md` §2 now list it.
- **Verified:** the three build timings above (Linux, Rust 1.99.1); `cargo check` green;
  `cargo fmt --check` green (no Rust changes).
- **Owner must check:** read `PERF.md`, pick what to implement and confirm the two decisions it
  needs — region undo (SPEC §3.6 rewrite) and the wasm-opt exception to SPEC §6. Nothing in the
  app behaves differently.

### 2026-11-10 — `Docs:` fixed artifact names `russet.zip`/`russet.patch`; commit title last in reply — owner request
- **Commit title:** `Fix delivery artifact names; require commit title as the reply's last line`
- **What:** delivery ritual hardened per owner request: the zip artifact is ALWAYS named
  `russet.zip` and the patch ALWAYS `russet.patch` (stable names across sessions, the new
  round overwrites the old files); the commit title is now stated BOTH in the log entry AND
  as the very last line of the chat reply, so it is always findable and copyable. No
  app-behavior change.
- **Files:** `AGENTS.md` (delivery ritual fixed names, commit-title rule), `PROGRESS.md`
  (this entry); the delivered artifact renamed to `russet.zip`.
- **Verified:** documentation-only change — no code touched; zip rebuilt and byte-compared
  with the working tree (unpack == tree), sha256 in the chat reply.
- **Owner must check:** nothing to run — docs and delivery naming only.

### 2026-11-10 — `Tooling:` delivery ritual, reading routes, verify.sh, log discipline — owner request
- **Commit title:** `Add delivery ritual, reading routes, verify.sh and PROGRESS log discipline`
- **What:** agent-workflow improvements, no app-behavior change. AGENTS.md: task-type →
  reading-route table replaces the blanket "read §4–§7" rule; delivery ritual (patch+zip
  always prepared together, patch only when `.git` is present, zip never contains `.git`,
  byte-compare + sha256 before delivery); log-entry budget ≤ ~20 lines with a durable-fact
  funnel (quirks → AGENTS.md, laws → SPEC, facts → docstrings); Log compaction to
  `PROGRESS-archive.md`; `scripts/verify.sh` as the done gate. This round itself ran
  WITHOUT `.git` (sandbox recycle; tree restored from the previous zip), so per the new
  ritual there is NO patch — the zip is the only artifact.
- **Files:** `AGENTS.md` (routes, ritual, log discipline, Commands), `PROGRESS.md` (intro,
  verification stamp, Log trimmed to the two 2026-11-10 entries), new `PROGRESS-archive.md`
  (nine entries of 2026-11-06/09 moved verbatim), new `scripts/verify.sh`.
- **Verified:** `bash scripts/verify.sh` — all gates green, 43 tests (see the verification
  stamp); delivery: unpacked zip byte-compared with the working tree, checksums in the chat
  reply.
- **Owner must check:** nothing to run — docs and agent tooling only.

### 2026-11-10 — `UX-fix:` preview/commit fidelity, Shape outline+fill, window nudges — owner request (6 items)
- **Commit title:** `Fix line/ellipse preview fidelity; Shape outline+fill; window nudges`
- **What:** six owner-requested corrections to the previous patch, found on
  the first Windows run.
  1. **History window** first-open position: it sat over the menu strip
     (`content_rect()` is the FULL viewport incl. the menu bar — verified in
     the egui 0.37 source). It is now anchored exactly 0.5rem BELOW the
     "File Edit Canvas View Help" bar: the top edge = central-panel top
     (`last_panel_rect`, whose top IS the menu bar's bottom because the
     no_frame central panel has zero margin) + 0.5rem, still 0.5rem from the
     right edge.
  2. **Layers window** first-open position lifted by 1.75rem: its bottom now
     sits 0.5rem + 1.75rem = 2.25rem above the window bottom (new constant
     `placement::LAYERS_FIRST_OPEN_LIFT`), so the zoom % label in the
     canvas's bottom-right corner stays readable.
  3. **Tools-window hints** wrapped into short lines: the Text hint is two
     weak lines, and the same was done for the Curve hint (identical
     problem — one long line stretched the auto-sized window).
  4. **Ellipse outline preview fixed.** Root cause: egui's `EllipseShape`
     strokes OUTSIDE the path (`PathStroke::outside()`, verified in epaint
     0.37), while the commit strokes the tiny-skia oval CENTERED — the
     preview ring was bigger than the committed ring. The preview now uses
     a closed `PathShape` (smooth polygon, ~3 pt chords) whose plain stroke
     kind is Middle = centered, so preview and commit agree in both Fill
     and Outline modes. Degenerate (click-without-drag) previews draw
     nothing, matching the commit.
  5. **Shape tool got a second checkbox "Outline"** next to "Fill": the
     primary color paints the outline, the secondary the fill (button
     choice no longer changes shape colors), both may be on at once (fill
     under the stroke). At least one always stays on: unchecking Fill
     re-checks Outline, unchecking Outline re-checks Fill (owner rule), so
     the tool can never draw nothing. `rasterize_shape` now takes
     `outline: Option<Color32>, fill: Option<Color32>` and applies fill
     first, stroke on top; `ToolState` gained `shape_outline` (default:
     outline on, fill off — same feel as before). Rectangle gets the same
     two options (the checkboxes are per-tool state).
  6. **Round caps in line previews:** egui line ends are flat (verified in
     epaint — "no option to add a circular end"), while the commit uses
     `LineCap::Round`. The Straight-line and Curve-polyline previews now
     draw filled circles (radius = stroke width / 2) at both endpoints —
     always, at least until the owner gets a cap choice; a click-without-
     drag now previews the same round dot the commit draws.
- **Files:** `src/tools.rs` (shape_outline, rasterize_shape signature, 2 new
  tests), `src/app/preview.rs` (Drag outline/fill, PathShape ellipse,
  round-cap circles), `src/app/windows.rs` (Outline checkbox + exclusivity,
  hint wrapping), `src/app/panels.rs` (History/Layers first-open anchors),
  `src/app/placement.rs` (LAYERS_FIRST_OPEN_LIFT); docs: AGENTS.md (commit-
  title rule), PROGRESS.md (this entry).
- **Verified:** `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets
  -- -D warnings`, `cargo test` — all green, **43 tests** (41 → 43:
  fill+outline combined, outline+fill-both-None; Rust 1.99.1, Linux). No
  app build, no release build (owner runs those); wasm untouched (no new
  `#[cfg]` branches).
- **Owner must check:** on Windows `cargo run` — (1) History opens below the
  menu bar with a 0.5rem gap; (2) Layers no longer covers the zoom %;
  (3) Text/Curve hints are narrow, the Tools window stays compact; (4)
  Ellipse outline: preview radius == committed radius now, with Fill and
  with Outline+Fill combined; (5) unchecking Fill/Outline never leaves both
  off; outline takes the primary color, fill the secondary; (6) line and
  curve previews have round caps identical to the committed line.

### 2026-11-10 — `Features:` Eyedropper, Line/Curve, Shape, Text, History log, Layers placeholder — owner request (6 items)
- **What:** six owner-requested additions, all deliberately simple first
  iterations ("keep it simple") that the owner will rework later. This work was
  explicitly ordered by the owner, overriding the §6 out-of-scope list
  (recorded as an owner exception in SPEC §6).
  1. **Eyedropper (I)** — pressing on the canvas samples the pixel under
     the pointer into the Primary (LMB) or Secondary (RMB) well and keeps
     sampling while the button is held (Paint.NET-style). Never touches the
     canvas or the history. Pixel demultiplication handled (premultiplied
     `Pixmap` → straight-alpha `Color32`).
  2. **Line/Curve (L)** — one tool, two modes (Tools window selector):
     *Straight*: press–drag–release with a translucent on-canvas preview,
     committed on release; *Curve*: click to add polyline vertices, rubber
     band to the cursor, Enter / double-click / RMB commits, Esc cancels,
     Backspace removes the last vertex. Preview never touches the `Pixmap`
     — a cancelled gesture leaves no history snapshot.
  3. **Shape (U)** — Rectangle / Ellipse selector + "Fill" checkbox in the
     Tools window; drag with preview, committed on release; Esc cancels.
     Outline width = the Size slider; LMB/RMB pick primary/secondary like
     every other tool.
  4. **Text (T)** — a click on the canvas places an anchor and opens a
     small "Text" window (font-size slider 8..=200 + one-line field) next
     to it; the text renders on-canvas live via egui's own layout engine;
     Enter / "Place" commits, Esc / "Cancel" / a new click re-anchors.
     Commit rasterizes the galley through egui's font atlas into the
     `Pixmap` (new `src/text.rs`) — no new dependencies, committed pixels
     match the preview, font size is in canvas pixels (zoom/DPI
     independent).
  5. **History window** — read-only, non-clickable log of document actions
     (strokes per tool, Line, Curve, Rectangle, Ellipse, Text, Undo, Redo,
     New/Opened/Resized), newest first, capped at 200 entries with
     monotonically increasing ids (`ActionLog` in `history.rs`). Reopen
     via View → History. Interactive undo-to-state will replace it later.
  6. **Layers window** — placeholder with exactly one row ("Background")
     showing a live 28×28 canvas thumbnail; nothing is interactive.
     Reopen via View → Layers.
  Behavior details: switching the tool cancels any pending gesture/text
  entry; New/Open/Resize also reset them and restart the log; Space-pan is
  blocked while a Line/Shape drag is in progress (same as freehand
  strokes); a degenerate polyline (single point) draws a round dot;
  perfectly horizontal/vertical polylines are nudged by a subpixel epsilon
  because tiny-skia rejects zero-height/width path bounds (found by the
  new tests). The brush cursor ring now also shows for Line/Shape; the
  four new tools show a crosshair cursor over the canvas.
- **Commit title:** `Add Eyedropper, Line/Curve, Shape, Text tools and History/Layers windows`
- **Files:** new `src/text.rs` (galley+atlas text rasterization),
  `src/app/preview.rs` (Line/Curve/Shape gestures + overlay + commits),
  `src/app/text_tool.rs` (Text input window + preview + commit),
  `src/app/panels.rs` (History + Layers windows); modified `src/tools.rs`
  (Tool variants, LineMode/ShapeKind, `rasterize_polyline`,
  `rasterize_shape`, `pick_color`, shared `blend_pixel`), `src/history.rs`
  (`ActionLog`), `src/app.rs` (tool dispatch, hotkeys I/L/U/T, View menu,
  logs, `last_tool` switch handling), `src/app/windows.rs` (7 tool
  buttons, Size slider enablement, Line mode + Shape kind/Fill selectors),
  `src/app/dialogs.rs` (resize log + gesture reset),
  `src/app/placement.rs` (`set_window_pos` now `pub(super)` for the Text
  window reposition), `src/app/camera.rs` (inverse `screen_pos`), and
  `src/main.rs` (`mod text;`). Docs: SPEC §3.9 + §6 exception, AGENTS.md
  module list.
- **Verified:** `cargo fmt --check`, `cargo check`, `cargo clippy
  --all-targets -- -D warnings`, `cargo test` — all green, **41 tests**
  (27 → 41: polyline/shape/pick/text/ActionLog tests; Rust 1.99.1,
  Linux). The wasm32 target is not installed in this sandbox and wasm
  checks stay deferred to the owner (per the 2026-11-09 M8-notes entry);
  the new code adds no `#[cfg]` branches. No release build (owner runs it).
- **Owner must check:** on Windows `cargo run` — (1) Eyedropper: pick with
  LMB/RMB, drag to sample continuously; (2) Line: Straight drag preview,
  then Curve clicks + Enter/Esc/Backspace; (3) Shape: rect/ellipse outline
  and Fill, Esc mid-drag; (4) Text: click → type → size slider → Enter,
  verify committed text matches the preview and stays sharp at 100%
  zoom; (5) History window lists every action, newest on top, and survives
  window open/close via View; (6) Layers window shows one Background row
  with a live thumbnail; (7) undo/redo still work for every new tool;
  (8) hotkeys I/L/U/T do not fire while typing in the Text window.

### 2026-11-10 — `UX+Build:` WASD panning, 0.5rem first-open gaps, flake de-warned — owner request (4 items)
- **What:** three code changes + the tests question answered in chat. (1) **`flake.nix`:** the deprecated `xorg.*` aliases in `runtimeLibs` were replaced with the new-style top-level attrs (`libx11`, `libxcursor`, `libxrandr`, `libxi`, `libxrender`, `libxext`, `libxcb`) — this removes the nixpkgs evaluation warnings; same libraries, same dlopen names, runtime behavior identical. (2) **First-open gaps:** Tools and Palette no longer sit flush — each keeps a 0.5rem (8 pt) gap from the edges: Tools 8 pt from the left edge (vertically centered), Palette 8 pt from the left AND bottom edges. `0.5 * REM` (REM = 16 pt; egui 0.37 has no rem API) lives in `app/placement.rs::FIRST_OPEN_GAP` next to the 2rem `WINDOW_SNAP_DISTANCE` (now `2.0 * REM`), applied to the `Window::default_pos` anchors in `app/windows.rs`; §3.11 treats the offset as an ordinary dropped gap (the free-axis gap-ratio preserves it even before the first drag). (3) **W/A/S/D panning:** a held key pans the viewport continuously — pure `key_pan_dir(w,a,s,d)` in `app/camera.rs` (diagonals normalized), speed `KEY_PAN_SPEED` = 600 pt/s (screen-space, owner-tuned) scaled by `stable_dt.min(0.1)` so a stalled frame cannot jump the canvas. `egui::Key` letters are PHYSICAL keys (egui-winit maps them from winit `PhysicalKey`), so it is layout-independent — the Russian typewriter layout's W/A/S/D keys included. Gated by `!egui_wants_keyboard_input()` like the hotkeys; while a pan key is held no stroke starts; explicit `request_repaint()` while keys are held so panning is smooth even without OS key repeat. (4) Docs synced: SPEC §3.2, §3.3, §3.4 (new "Pan — W/A/S/D" bullet), §3.9 table (new row), §7.1; AGENTS.md placement bullet.
- **Files:** `flake.nix`; `src/app/windows.rs`, `src/app/placement.rs` (REM / FIRST_OPEN_GAP consts), `src/app/camera.rs` (key pan, 2 new tests), `src/app.rs` (`KEY_PAN_SPEED`); docs: `SPEC.md`, `AGENTS.md`.
- **Verified:** `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` — all green, 27 tests (25 + 2 new WASD direction/diagonal tests; Rust 1.99.1, Linux). Nix is still unavailable in the sandbox, so the flake itself is only desk-checked (attrs cross-checked against the nixpkgs rename: top-level `libx*`/`libxcb` are the non-deprecated forms).
- **Owner must check:** on Windows `cargo run` — Tools/Palette appear with a small 8 pt margin instead of flush; holding W/A/S/D pans the canvas (switch to the Russian layout and confirm its W/A/S/D keys do the same; typing in a dialog field must NOT pan); wheel/middle/Space panning unchanged. On a Nix machine — `nix build` (or `nix flake check`) must now run WITHOUT evaluation warnings.

### 2026-11-09 — `Docs:` README.md added (build & run for humans) — owner request
- **What:** new `README.md` in English, deliberately MINIMAL per the owner's instruction: it contains ONLY how to build and run the app for the three supported targets — Windows (`cargo run`, `cargo build --release`), NixOS/Linux (`nix build` / `nix run` / `nix develop`, flake enablement one-liner, `WINIT_UNIX_BACKEND` override, non-NixOS distros welcome) and Web/WASM (wasm32 target, `trunk serve`, `trunk build --release` → `dist/`, one example of serving it). No feature descriptions, no changelog. The fact that README.md is the HUMAN-facing document (agents read AGENTS.md/SPEC.md instead) is recorded in AGENTS.md ("Repo facts") and SPEC §8, with a keep-in-sync rule when commands change.
- **Files:** new `README.md`; `AGENTS.md` (README audience bullet), `SPEC.md` (§8 pointer + note that Linux builds go through `flake.nix` since 2026-11-09).
- **Verified:** documentation-only change — no code touched; commands cross-checked against SPEC §8, `flake.nix` and `index.html` (trunk entry).
- **Owner must check:** the three command blocks read correctly on Windows PowerShell vs bash; nothing else.

### 2026-11-09 — `Extras:` About window with platform label + flake.nix (Linux) — owner request
- **What:** two owner-requested additions. (1) A **Help → "About Russet"** menu item opens a small non-resizable window (name, version, one-line description, Close button) that states the platform the app runs on: `Wayland (Linux)`, `X11 (Linux)`, `Windows`, `macOS` or `Web (WASM)`, plus the renderer (OpenGL/WebGL via eframe glow). The Linux Wayland-vs-X11 check mirrors winit's own backend selection: `WINIT_UNIX_BACKEND` override first, then `WAYLAND_DISPLAY`/`WAYLAND_SOCKET` → Wayland, else `DISPLAY` → X11. Implemented with plain `cfg!()` macros (one code path compiles for all targets; no new `#[cfg]` attributes outside `main.rs`). (2) **`flake.nix`** builds the app on Linux (`nix build` / `nix run` / `nix develop`, x86_64+aarch64): `rustPlatform.buildRustPackage` against the existing `Cargo.lock`; only `fontconfig` is linked directly (egui system fonts), all other GUI libs (wayland, libxkbcommon, X11, GLX/EGL) are dlopen'd at runtime and provided via `wrapProgram` LD_LIBRARY_PATH, so one binary runs in Wayland and X11 sessions (force with `WINIT_UNIX_BACKEND=x11|wayland`). No new crate dependencies.
- **Files:** `src/main.rs` (`platform_label` / `renderer_label`), `src/app.rs` (`show_about` state, Help menu, dialog call), `src/app/dialogs.rs` (`about_dialog`), new `flake.nix`; docs synced: SPEC §3.1 (Help menu), §6 (owner exception for the flake), §7 (item 1 mentions Help; new item 20 About; Build renumbered to 21), AGENTS.md (platform-label + flake bullets).
- **Verified:** `cargo check` (baseline AND after the change), `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` — all green, 25 tests passed (Rust 1.99.1, Linux). The flake itself was NOT built here — the sandbox has no Nix; see below.
- **Owner must check:** on a Nix machine: `nix build && ./result/bin/russet` in a Wayland session (About must say `Wayland (Linux)`) and in an X11 session / `WINIT_UNIX_BACKEND=x11` (About must say `X11 (Linux)`); on Windows: Help → About Russet shows `Platform: Windows`; web build: `Platform: Web (WASM)`.

### 2026-11-09 — `UX-fix:` main-window min size is a fixed 500×400 (owner decision)
- **What:** the owner clarified the intent — the minimum window size is simply FIXED at
  500×400, not derived from the default (the previous 800×540 "~1.5x" interpretation is
  reverted). Process change recorded per owner's instruction: to save time, heavy checks
  (`cargo test` / `clippy` / `fmt`) are no longer run by default — see AGENTS.md "Commands";
  default verification is `cargo check` only.
- **Files:** `src/main.rs` (min inner size + comment), `SPEC.md` (§2.1 code quote, §7 item 1),
  `AGENTS.md` ("Commands" — new verification policy).
- **Verified:** `cargo check` — green. Tests/clippy/fmt skipped per the owner's
  time-saving instruction (policy above); the change is one constant.
- **Owner must check:** the window resizes down to 500×400 and no further (next `cargo run`).

### 2026-11-09 — `UX:` first-open positions for Tools/Palette + main-window min size
- **What:** on its first open the Tools window sits flush left, vertically centered in the
  main window (`Window::pivot(Align2::LEFT_CENTER)` + `default_pos(content_rect().left_center())`),
  and the Palette window sits flush in the bottom-left corner (`Align2::LEFT_BOTTOM` +
  `content_rect().left_bottom()`). `default_pos` is consumed only until the window has a
  stored area position, so the user's placement (§3.11) always wins afterwards; the free-axis
  gap-ratio formula preserves both layouts on resizes (ratios 0.5 and 1.0). Main-window min
  size lowered 600 → 540 so the window can shrink ~1.5x in BOTH axes (1200/800 = 1.5 exactly,
  800/540 ≈ 1.48 — whole "nice" numbers per owner's request).
- **Files:** `src/main.rs` (min inner size + comment), `src/app/windows.rs` (pivot +
  default_pos on both windows, module doc); docs synced: SPEC §2.1 code quote, §3.2, §3.3,
  §7 item 1, AGENTS.md placement bullet + a new egui `pivot`/`default_pos` quirk bullet.
- **Verified:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test` — all green, 25 tests passed (2026-11-09).
- **Owner must check:** on Windows `cargo run` — Tools appears left-center, Palette
  bottom-left; the window shrinks down to 800×540 but no further; dragging the windows and
  resizing the main window still behaves per §3.11.

### 2026-11-09 — `refactor:` app.rs split into child modules (owner-approved)
- **What:** the 1634-line `app.rs` was split for agent readability, with `app.rs` kept as
  the orchestrator (struct `App`, frame flow, hotkeys, strokes, file glue, menu, canvas
  widget — ~690 lines, over the 500 guard by owner's decision). New child modules (they can
  access `App`'s private fields, so no `pub(crate)` churn; `main.rs` and its mod list are
  untouched): `app/camera.rs` — §3.4 viewport/zoom/pan + zoom tests; `app/dialogs.rs` —
  §3.7/§3.8 New/Resize/Discard/Error; `app/windows.rs` — §3.2/§3.3 Tools/Palette windows,
  color wells, stable window ids; `app/placement.rs` — §3.11 entirely, incl. all 12
  placement tests. Behavior unchanged.
- **Files:** `src/app.rs` rewritten (1634 → 684 lines), new `src/app/{camera,dialogs,windows,placement}.rs`;
  docs synced: SPEC §2 structure + §3.11 implementation notes, AGENTS.md module facts +
  placement bullet + owner-approved app.rs exception.
- **Verified:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test` — all green, 25 tests passed (2026-11-09).
- **Owner must check:** nothing new — behavior must be identical; a `cargo run` smoke test
  (draw, zoom, dialogs, drag the floating windows) is enough.

### 2026-11-09 — `M4-rework:` floating-window magnetism switched to drop-point anchoring
- **What:** per owner feedback, the magnet no longer pulls a window flush to an edge and
  never moves a window during a drag. When a title-bar drag ends within 2rem of a
  main-window edge, that axis is anchored exactly where the user dropped the window (a gap
  of 0 happens only if the window was dropped flush); the dropped gap is preserved on later
  main-window resizes; the free axis keeps the gap-ratio placement; resizes never anchor,
  unanchor or pull anything flush.
- **Files:** `src/app.rs` (`anchor_axis` / `window_anchor` / `reposition_axis` /
  `reposition_window_pos`, tests rewritten), `SPEC.md` (§3.11, M4, pitfall 14,
  checklist item 14), `AGENTS.md` (placement bullet).
- **Verified:** `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test` — all green, 25 tests passed (re-run 2026-11-09).
- **Owner must check:** the new magnet feel via `cargo run` on Windows (drop a window near
  an edge → it stays where dropped; resize the main window → the dropped gap is kept).

### 2026-11-09 — `M8-notes:` wasm checks deferred by the owner
- **What:** the owner postponed ALL wasm/trunk verification until the native app is checked
  by hand on Windows; the agent must not run long wasm builds.
- **Owner must check (M8, on their machine):** `cargo check --target wasm32-unknown-unknown`,
  `trunk serve`, `trunk build --release`.

### 2026-11-06 — `M0–M7:` native application implemented from SPEC
- **What:** scaffold; 800×600 canvas with zoom/pan (Ctrl+wheel, Paint.NET-style steps,
  wheel pans); dirty-region texture updates; pencil/brush/eraser with snapshot history;
  top menu + floating "Tools"/"Palette" windows; native file I/O (New/Open/Save/Save As,
  rfd dialogs, unsaved-changes guard); undo/redo + the full hotkey table; Resize Canvas and
  New dialogs.
- **Files:** the whole `src/` tree per SPEC §2.
- **Verified:** `cargo check`, `cargo clippy -- -D warnings` (incl. `--all-targets`),
  `cargo test` — green (23 tests at that point).
- **Owner must check:** §7 checklist items 1–14 (Windows native).

---

## Log entries moved 2026-11-13 (compaction — the live Log keeps the newest 4)
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

- **What:** the owner verified both open checks in one pass ("checked V27, V25"):
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

### 2026-11-13 — `Fix:` History Tree lanes: divergence-only forks, recency placement, shared connector exits, per-row labels — owner request

- **What:** five owner points from the live review. (1) A PLAIN UNDO no longer diverts
  the tail sideways: the main line is now root..head PLUS the redo chain beyond the head
  (the redo hint first, then always the first-created child — exactly what repeated redo
  visits), so the undone tail stays on lane 0 until the user really draws past the head.
  (2) Labels hug EACH row's dot — kind + node id ("pencil 7") on an opaque window-fill
  backdrop; the swatch follows the text width. (3) ALL connectors leaving one node exit
  at the same height — the parent dot's center line (the FORK_DROP_STEP stagger is gone;
  overlapping runs are fine by owner rule). (4) A branch takes the LOWEST lane free at
  its vertical range (fork parent's row..= tip), so a late fork below early branches
  reuses lane 1 instead of creeping right. (5) Branches are placed NEWEST first — newer
  forks hug the trunk, older stack right; a branch forked off a side branch is pinned
  right of its parent's lane (connectors never run backwards). Spans are per BRANCH now
  (lane reuse means two spans can share one lane number).
- **Files:** `src/history_tree.rs` (layout_layer rewrite + layer_layout for MCP),
  `src/history_tree/tests.rs` (recency expectations reworked, +5 tests),
  `src/app/tree_window.rs` (connector geometry, per-row labels + backdrop, content
  width from the widest row), PROGRESS.md.
- **Verified:** `bash scripts/verify.sh` ALL GATES GREEN — 69 tests default, 81 with
  `--features mcp` (soft notes: tree_window.rs 687 / history_tree.rs 600 lines over the
  500 guard — report-only). Headless MCP scenarios (scripts/mcp_smoke.py): plain undo →
  all lane 0; late fork reuses lane 1 ([0,1,0,0,1,0]); newest fork on lane 1
  ([0,2,2,1,0]). Xvfb screenshots download/tree_stage*.png reviewed.
- **Owner-checks:** `cargo run` — lane order + label look; "New at top" re-check (sign
  logic unchanged, not re-captured); tree_window.rs split still pending your OK.
- **Commit title:** `Rework History Tree lanes: divergence-only forks, recency placement, shared connector exits, per-row labels`

### 2026-11-13 — `Feat:` MCP `get_tree_layout` tool + feature-gated verify gates — owner request ("use MCP, extend it if it helps")

- **What:** (1) the 9th MCP tool `get_tree_layout` — dumps the LAYOUT the History Tree
  window renders (rows with lane/fork/head flags + one rail span per branch), so an
  agent asserts forks and lane placement over the wire; layer-arg parsing shared with
  get_history_tree. (2) verify.sh now also runs clippy+tests with `--features mcp` —
  the feature code compiled only on manual builds and could rot unseen. (3) NEW
  scripts/mcp_smoke.py — a ~100-line stdio client (initialize / tools/list /
  tools/call) with scenario assertions; NEW scripts/visual_driver.py — the Xvfb +
  ffmpeg screenshot scenario runner used for the tree checks above.
- **Files:** `src/app/mcp/tools.rs` (+tool, +declaration, tests 8→9), `scripts/verify.sh`
  (+2 gated steps), NEW `scripts/mcp_smoke.py`, NEW `scripts/visual_driver.py`.
- **Verified:** verify.sh ALL GATES GREEN; smoke client green against the real binary
  under Xvfb (9 tools, 9 scenario checks); default-tree `cargo tree` still has no
  serde_json (781 vs 786 tree lines with the feature).
- **Owner-checks:** keep the two python helpers in scripts/ or drop them; CI (ci.yml)
  does not run the mcp gates yet — say the word and they go there too.
- **Commit title:** `Add get_tree_layout MCP tool and gate the mcp feature in verify.sh`

### 2026-11-13 — `Tooling:` debt lists, PROGRESS budget check, baseline rule, CI mcp gates, scenario driver — owner request

- **What:** agentic-workflow round, zero app-behavior change. (1) NEW
  `DEBTS-verification.md` (12 open human checks, V1–V12) + `DEBTS-functional.md`
  (7 open debts F1–F7, 2 paid); AGENTS.md "Debts": the agent surfaces open
  items in ONE batched list at session start and task end and ASKS which are
  human-verified. (2) rules.sh reports PROGRESS.md size (~200 budget), Log
  size (~120), parked RESOLVED warnings and the live owner-check count —
  report-only reminder. (3) THIS compaction: 15 entries + 3 RESOLVED warnings
  → archive; Warnings is active-only. (4) Commands: BASELINE before edits =
  rules.sh + `cargo fmt --check` only — no compiling pre-checks (the tree
  always lands green). (5) CI `native` runs clippy+test with `--features mcp`
  (owner OK; verify.sh has gated the pair since 2026-11-13). (6)
  `visual_driver.py` rewritten: scenarios as data, assertions as text lines —
  layout via get_tree_layout, committed pixels via get_canvas_snapshot + PIL
  in canvas space; screenshots only with `--shots`. 3 scenarios, 14 checks,
  green under Xvfb.
- **Files:** NEW `DEBTS-verification.md`, `DEBTS-functional.md`;
  `scripts/rules.sh`, `scripts/visual_driver.py` (rewritten),
  `.github/workflows/ci.yml`, `AGENTS.md` (Debts section, baseline rule, CI),
  `PROGRESS.md` (compaction + entry), `PROGRESS-archive.md` (moves in).
- **Verified:** `bash scripts/verify.sh` ALL GATES GREEN — 69 tests default,
  81 with `--features mcp`; driver 14/14 checks green under Xvfb; ci.yml
  parsed with yaml.safe_load.
- **Owner-checks:** answer the batched debt questions (V1–V12, F-decisions) —
  first live run of the ask-don't-accumulate rule; skim the DEBTS wording.
- **Commit title:** `Add DEBTS files and PROGRESS budget check; baseline rule; CI mcp gates; scenario driver`
