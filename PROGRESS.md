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

**Verification stamp:** fmt clean, `cargo clippy --profile gates --all-targets -- -D warnings` clean,
`cargo test --profile gates` — 78 green; with `--features mcp` — clippy clean, 90 green
(2026-11-13, Rust 1.99.1 pinned via rust-toolchain.toml, Linux; every src file ≤ 500 lines;
rules.sh reports the PROGRESS.md/Log budgets — green; the GUI runs headless in this sandbox per
`SANDBOX.md` — visual_driver.py 3/3 scenarios, 15 checks green, re-proven after the Layers-width /
white-initial-layer fix; V13/V14 remain the owner's pointer-feel checks).
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

### 2026-11-13 — `Docs:` second debt triage — owner answered the batched V/F list

- **What:** dispositions applied, zero code change. Verified via chat: V4, V7 (owner:
  «проверил V4, V7» — one `cargo run` each after the toolchain pin / the file splits).
  F2 DROPPED from the debts by owner decision: the History window stays AS IS, the
  owner will say himself when it is no longer needed — not tracked anymore. F5 parked
  for the long term («долгий ящик»): memory limits not needed during active
  development. The rest reconfirmed open: V8, V9, V13, V14, F3, F8, F9, F10 — owner:
  «остальное не сейчас» (statuses carry the date). Open tables: V 6 → 4, F 6 → 5.
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
