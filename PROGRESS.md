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

**Verification stamp:** fmt clean, `cargo clippy --all-targets -- -D warnings` clean,
`cargo test` — 48 green (2026-11-10, Rust 1.99.1, Linux). Update this stamp in the same
edit as a log entry (AGENTS.md "Git & progress log").

## Warnings — open issues deliberately NOT fixed yet

- 2026-11-10 — RESOLVED: `src/tools.rs` was **679 lines**; moving its inline tests to
  `src/tools/tests.rs` (2026-11-10, PR rebase) brought it to ~430 lines, so the proposed
  `raster.rs` split is retired. `src/app.rs` is 848 lines — still within the owner-approved
  orchestrator exception (roughly doubled would be ~1400).

- 2026-11-09 — RESOLVED: `src/app.rs` (1634 lines) was split into four child modules
  (`app/camera.rs`, `app/dialogs.rs`, `app/windows.rs`, `app/placement.rs`). `app.rs` itself
  remains ~690 lines BY OWNER'S DECISION — it stays the single orchestrator and may exceed
  the 500-line guard; do not propose splitting it further unless it roughly doubles
  (see the exception in AGENTS.md "Repo facts").

## Log — newest first. Insert a new entry directly under this heading.

> Budget: ≤ ~20 lines per entry, fixed fields (What / Files / Verified / Owner-checks /
> Commit title); durable facts move to their canonical home in the same edit (AGENTS.md
> "Git & progress log"). Entries older than the newest 3–5 are moved verbatim to
> `PROGRESS-archive.md`. The two 2026-11-10 entries below predate the budget and stay
> until the owner's Windows check of the six UX-fix items is done.

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

