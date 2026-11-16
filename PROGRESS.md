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

**Verification stamp:** fmt clean, `cargo clippy --profile gates --all-targets -- -D warnings` clean,
`cargo test --profile gates` — 84 green; with `--features mcp` — clippy clean, 96 green
(2026-11-15, Rust 1.99.1 pinned via rust-toolchain.toml, Linux; layers.rs sits at 504/500 lines —
over the soft guard, a split is proposed at task end; rules.sh reports the PROGRESS.md/Log budgets —
green; the GUI runs headless in this sandbox per `SANDBOX.md` — visual_driver.py 3/3 scenarios
+ the scripted dlg_verify pass 9/9 through the real binary; V19–V20 are the owner's feel checks).
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

