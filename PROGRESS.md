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
| Extra: opt-in MCP server (owner 2026-11-12) | done — 9 tools behind `mcp` | default build byte-identical graph (no serde_json/threads); `get_tree_layout` added 2026-11-13; verify.sh gates the feature |
| Extra: History Tree windows (owner 2026-11-12) | done — lanes reworked 2026-11-13 | divergence-only forks, recency lanes, shared connector exits, per-row labels; MCP-driven scenarios + Xvfb screenshots green |
| Extra: 500-guard splits (owner 2026-11-13) | done — pure code motion | `history_tree/layout.rs`, `tree_window/paint.rs`, `app/document.rs` + `menu.rs` + `input.rs`; app.rs 936 → ~440, every src file ≤ 500 |
| Extra: agent workflow v2 (owner 2026-11-13) | done | DEBTS-verification.md + DEBTS-functional.md debt lists; rules.sh PROGRESS budget check; baseline-before-edits rule; CI runs the mcp gates; visual_driver.py rewritten as the scenario runner |

**Verification stamp:** fmt clean, `cargo clippy --profile gates --all-targets -- -D warnings` clean,
`cargo test --profile gates` — 69 green; with `--features mcp` — clippy clean, 81 green
(2026-11-13, Rust 1.99.1 pinned via rust-toolchain.toml, Linux; every src file ≤ 500 lines;
rules.sh additionally reports the PROGRESS.md/Log budgets — both green after this compaction).
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

### 2026-11-13 — `Refactor:` split the 500-guard files and slim app.rs into child modules — owner request

- **What:** ALL pure code motion, zero behavior change, consumer paths preserved.
  (1) `history_tree.rs` 600 → 350: `Row`/`BranchSpan`/`TreeLayout` + `layout_layer` moved to
  child `history_tree/layout.rs` (281) and re-exported, so `tree_window.rs`, the MCP tools and
  `tests.rs` keep the `history_tree::` paths untouched. (2) `app/tree_window.rs` 687 → 433:
  the painter + `GraphMetrics` + geometry constants + palette moved to child
  `app/tree_window/paint.rs` (269); the parent keeps windows/clones/scroll bookkeeping.
  (3) `app.rs` 936 → 439 (owner request "сделай app.rs меньше"): document lifecycle
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

> Budget: ≤ ~20 lines per entry, fixed fields (What / Files / Verified / Owner-checks /
> Commit title); durable facts move to their canonical home in the same edit (AGENTS.md
> "Git & progress log"). Entries older than the newest 3–5 are moved verbatim to
> `PROGRESS-archive.md`. The two 2026-11-10 entries below predate the budget and stay
> until the owner's Windows check of the six UX-fix items is done.

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
