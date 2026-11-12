# PROGRESS — archive (Russet)

Verbatim log entries moved out of `PROGRESS.md` by the log-compaction rule
(AGENTS.md "Git & progress log": the live Log keeps the newest 3–5 entries,
older ones go here unchanged). Read this file only via grep when investigating
history — e.g. `rg -n "magnetism|min size|flake" PROGRESS-archive.md`. Test
counts inside are HISTORICAL (they grow over time); the current count is in
the `PROGRESS.md` verification stamp. Newest first, same format as the live log.

---

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
  space above / 60% below (owner: "расстояние до верхнего края 40%, до нижнего
  60%"). A pivot + `default_pos` cannot express this (the auto-sized height is
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
  iterations ("простенько") that the owner will rework later. This work was
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
- **What:** three code changes + the tests question answered in chat. (1) **`flake.nix`:** the deprecated `xorg.*` aliases in `runtimeLibs` were replaced with the new-style top-level attrs (`libx11`, `libxcursor`, `libxrandr`, `libxi`, `libxrender`, `libxext`, `libxcb`) — this removes the nixpkgs evaluation warnings; same libraries, same dlopen names, runtime behavior identical. (2) **First-open gaps:** Tools and Palette no longer sit flush — each keeps a 0.5rem (8 pt) gap from the edges: Tools 8 pt from the left edge (vertically centered), Palette 8 pt from the left AND bottom edges. `0.5 * REM` (REM = 16 pt; egui 0.37 has no rem API) lives in `app/placement.rs::FIRST_OPEN_GAP` next to the 2rem `WINDOW_SNAP_DISTANCE` (now `2.0 * REM`), applied to the `Window::default_pos` anchors in `app/windows.rs`; §3.11 treats the offset as an ordinary dropped gap (the free-axis gap-ratio preserves it even before the first drag). (3) **W/A/S/D panning:** a held key pans the viewport continuously — pure `key_pan_dir(w,a,s,d)` in `app/camera.rs` (diagonals normalized), speed `KEY_PAN_SPEED` = 600 pt/s (screen-space, owner-tuned) scaled by `stable_dt.min(0.1)` so a stalled frame cannot jump the canvas. `egui::Key` letters are PHYSICAL keys (egui-winit maps them from winit `PhysicalKey`), so it is layout-independent — ЙЦУКЕН's Ц/Ф/Ы/В included. Gated by `!egui_wants_keyboard_input()` like the hotkeys; while a pan key is held no stroke starts; explicit `request_repaint()` while keys are held so panning is smooth even without OS key repeat. (4) Docs synced: SPEC §3.2, §3.3, §3.4 (new "Pan — W/A/S/D" bullet), §3.9 table (new row), §7.1; AGENTS.md placement bullet.
- **Files:** `flake.nix`; `src/app/windows.rs`, `src/app/placement.rs` (REM / FIRST_OPEN_GAP consts), `src/app/camera.rs` (key pan, 2 new tests), `src/app.rs` (`KEY_PAN_SPEED`); docs: `SPEC.md`, `AGENTS.md`.
- **Verified:** `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` — all green, 27 tests (25 + 2 new WASD direction/diagonal tests; Rust 1.99.1, Linux). Nix is still unavailable in the sandbox, so the flake itself is only desk-checked (attrs cross-checked against the nixpkgs rename: top-level `libx*`/`libxcb` are the non-deprecated forms).
- **Owner must check:** on Windows `cargo run` — Tools/Palette appear with a small 8 pt margin instead of flush; holding W/A/S/D pans the canvas (switch the layout to Russian and confirm Ц/Ф/Ы/В do the same; typing in a dialog field must NOT pan); wheel/middle/Space panning unchanged. On a Nix machine — `nix build` (or `nix flake check`) must now run WITHOUT evaluation warnings.

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
