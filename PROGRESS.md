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
| Extra: MCP tools split + commit-title ritual (owner 2026-11-13) | done — pure code motion | root `mcp/tools.rs` = the tool map (dispatch + declarations, 157); themed `tools/queries.rs` (200) + `tools/edits.rs` (177); rules.sh rule 6 prints the newest commit title at every done-gate; AGENTS.md Delivery ritual requires it verbatim in the reply |

**Verification stamp:** fmt clean, `cargo clippy --profile gates --all-targets -- -D warnings` clean,
`cargo test --profile gates` — 78 green; with `--features mcp` — clippy clean, 90 green
(2026-11-13, Rust 1.99.1 pinned via rust-toolchain.toml, Linux; every src file ≤ 500 lines;
rules.sh reports the PROGRESS.md/Log budgets — green; the GUI runs headless in this sandbox per
`SANDBOX.md` — visual_driver.py 3/3 scenarios, 14 checks green, re-proven after the mcp/tools
split; V13 remains the owner's pointer-feel check).
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

### 2026-11-13 — `Tooling:` headless-GUI recipe — the sandbox CAN run the GUI after all — owner request ("запиши опыт в .md")

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

### 2026-11-13 — `Feat:` real layers + per-node PNG history + node-dot switching — owner request ("кусок большой фичи")

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
  "cargo run отлично работает" closes V4/V7.
- **Commit title:** `Add real layers, per-node PNG history trees and node-dot canvas switching`

### 2026-11-13 — `Docs:` first debt-triage round — owner answered the batched V/F list

- **What:** owner dispositions applied to the two debt files; zero code change.
  Verified via chat: V1, V2, V3, V11 (Windows tool/UX/About checks), V5+V6 (History
  Tree — "руками потрогал, устраивает"; later improvements = new tasks), V12 (docs
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
