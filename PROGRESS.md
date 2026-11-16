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

**Verification stamp:** fmt clean, `cargo clippy --profile gates --all-targets -- -D warnings` clean,
`cargo test --profile gates` — 94 green; with `--features mcp` — clippy clean, 106 green
(2026-11-16, Rust 1.99.1 pinned via rust-toolchain.toml, Linux; Shift-proportional shapes
landed — the pure rule + 5 tests, every src file ≤ 500; rules.sh reports the
PROGRESS.md/Log budgets — green; the GUI runs headless per `SANDBOX.md` —
visual_driver.py 3/3 scenarios + the scripted 9-check XTEST Shift pass through the real
binary (unconstrained 88×38 vs Shift 88×88 vs mid-drag 78×78); V22 is the owner's
feel check). A docs-only debt-triage round followed the same day: rules.sh
HARD-green at both gates, no Rust code touched.
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

### 2026-11-16 — `Docs:` third debt triage — 8 checks verified, F11 born, zip-priority rule

- **What:** the owner's «проверил V22, V21, V20, V19, V18, V17, V14, V13» closed those
  EIGHT owner-checks in DEBTS-verification.md (chat-verified, the V4/V7 precedent).
  V15's verdict becomes a NEW functional debt F11 — wheel pan ANOTHER ×5 faster:
  `SCROLL_PAN_SPEED` 3.0 → 15.0 in `src/app.rs` + the two SPEC §3.4 mentions in the
  same edit — recorded with the how-to and POSTPONED by the owner («запиши как
  сделать потом»). AGENTS.md "Git & progress log" gained the owner's rule: a
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
  (owner: «при зажатом Shift фигуры получались пропорциональными, как в
  Paint Net»). The start point is the anchor corner; the free corner is
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
  row's chip shows both — outline\|fill halves (owner: «у тех инструментов
  у которых использовалось 2 цвета отображалось 2 цвета»): `TreeNode.color2`
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
  swapping them — the persistent swap is gone (owner: «цвета используются
  наоборот и без Swap»); pure `shape_gesture_wells` returns `(outline, fill)`
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

