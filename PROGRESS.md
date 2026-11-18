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
| Extra: About window + Nix | code written | Help → "About Russet" with Platform/Renderer lines; 2026-11-18: + "written in Rust" description + GitHub repo hyperlink; `flake.nix` for Linux (Wayland/X11) — `nix build` itself must be verified by the owner on a Nix machine |
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
| Extra: repo-wide English-only owner quotes (owner 2026-11-18) | done — docs only | every verbatim Russian owner quote across docs/scripts/comments translated to English (meaning kept); PROGRESS Log compacted — 2 entries moved to PROGRESS-archive.md |
| Extra: debt F14 — in-house windows later (owner 2026-11-18) | done — docs only | egui::Window → an in-house `egui::Area` frame recorded in DEBTS-functional.md from the owner's pros/cons chat review; owner-gated, no code touched |
| Extra: tool icon buttons + two-column grid (owner 2026-11-18) | done — gates green + visual driver | 1.5rem icon buttons via a cached tiny-skia atlas (`app/tool_icons.rs`), colored pencil/brush/eraser/pipette + mono line/shape/text through ONE pipeline, name+key hover tooltips, the pipette as the Eyedropper's canvas cursor; the two-column grid is debt F15; the driver screenshot caught the dead-texture trap (handles kept in the Atlas now — AGENTS.md quirks) |

**Verification stamp:** FULL gate re-verified 2026-11-18 (fresh sandbox): fmt clean, rules.sh
HARD-green, clippy `-D warnings` green (default + mcp), `cargo test --profile gates` 156 default /
168 mcp green (+11 new tool-icon tests), `cargo check --target wasm32-unknown-unknown` green,
visual driver ALL SCENARIOS GREEN + `--shots` — Rust 1.99.1 pinned via rust-toolchain.toml, Linux.
Zero Cyrillic repo-wide. Web/trunk lanes last verified 2026-11-16 (see the archived log history);
every src file ≤ 500 EXCEPT the report-only owner-deferred `history_tree.rs` 569 AND the fresh
`app/tool_icons.rs` 512 (over the 500 guard — the split is proposed to the owner at delivery,
not done without his approval).

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
> ever pinned in place by a pending check. Compacted 2026-11-18: the panels-split and
> docs rounds moved to PROGRESS-archive.md.

### 2026-11-18 — `Feat:` tool icon buttons (cached tiny-skia atlas) + two-column Tools grid — owner request

- **What:** the owner (2026-11-18) ordered the Tools window onto ICON buttons: 1.5rem (24 pt)
  icons, some COLORED, colored/monochrome through the SAME draw path ("no duplicated code"),
  a name+key hover tooltip per button, and a two-column INTERIM grid ("a better idea later" —
  debt F15). New child module `app/tool_icons.rs` (+ `tests.rs`): every icon is a tiny-skia
  vector drawing rasterized ONCE per (pixels_per_point, theme) change into an egui texture
  (`Context::load_texture_cached`) — a button's per-frame cost is one textured quad. Colored:
  Pencil (classic yellow body, wood cone, black graphite, metal ferrule, eraser end), Brush
  (varnished-wood handle + metal ferrule + bristles), Eraser (classic pink slab), Eyedropper
  (a pipette); monochrome (the theme text color): Line, Shape, Text. The pipette doubles as
  the Eyedropper's canvas cursor via `Context::set_cursor_image` (`CustomCursorImage`, straight
  RGBA, hotspot at the pipette tip; web falls back to `Crosshair`) — egui 0.37's custom-cursor
  path verified against the crate sources. `App` owns `tool_icons: Option<Atlas>`;
  `App::ui` calls `tool_icons::ensure` before the canvas draws. The uncommitted F14 docs round
  (recorded earlier today) rides in the SAME commit per the owner's one-commit request.
- **Files:** `src/app/tool_icons.rs` + `src/app/tool_icons/tests.rs` (new), `src/app/windows.rs`
  (icon buttons + `TOOL_COLUMNS` 2), `src/app.rs` (module + atlas field + ensure + cursor),
  `SPEC.md` §3.2, `AGENTS.md` (module map), `DEBTS-functional.md` (F15), `PROGRESS.md`
  (snapshot row + this entry + stamp).
- **Verified:** FULL gate re-run after the final fix: fmt clean, rules.sh HARD-green, clippy
  `-D warnings` green (default + mcp), `cargo test --profile gates` 156 default / 168 mcp green
  (+11 new icon tests), `cargo check --target wasm32-unknown-unknown` green; headless visual
  driver (Xvfb + MCP, the SANDBOX.md recipe): ALL SCENARIOS GREEN + `--shots` screenshots —
  the FIRST driver run caught a real bug: dropping the `load_texture_cached` handle freed the
  texture the SAME frame (retain-count rule) and the buttons rendered EMPTY; fixed by keeping
  the handles in `Atlas` — both new quirks (dead texture, custom OS cursors) recorded in
  AGENTS.md "eframe/egui 0.37 API quirks".
- **Owner-checks:** none born. Open: V33 (the About pass), V30 (parked), V8 (parked).
- **Commit title:** `Record debts F14/F15; tool icon buttons via a cached tiny-skia atlas + two-column Tools grid`

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

### 2026-11-18 — `Feat:` About window names Rust + links the GitHub repo — owner request

- **What:** the "About Russet" window gained (owner request 2026-11-18): (1) the language — the
  description line now reads "A tiny bitmap paint program written in Rust with egui/eframe."
  (was "built with egui/eframe."); (2) a GitHub repository hyperlink "GitHub: George-Noise/russet"
  → `https://github.com/George-Noise/russet`, via `ui.hyperlink_to` — opens the system browser on
  native (eframe's `links` feature, already enabled) and `window.open` on web; no new dependency.
  The window's CONTENT moved into the free `about_window_content` (no `App` state) so the new
  headless-egui content tests render the REAL body; placement, Close button and the dialog
  plumbing untouched. New egui quirk recorded in AGENTS.md: a window's first `run_pass` paints
  only `Shape::Noop` — content shapes exist from pass 2 on.
- **Files:** `src/app/dialogs.rs` (ABOUT_* constants + `about_window_content` + `about_dialog`),
  `src/app/dialogs/tests.rs` (+2 content tests), `SPEC.md` (§3.1 Help bullet + §7 checklist item
  20), `AGENTS.md` (1 quirk), `DEBTS-verification.md` (V33 born), `TIMING.md` (2 rows),
  `PROGRESS.md` (snapshot row + this entry).
- **Verified:** `bash scripts/verify.sh` ALL GATES GREEN — 145 default / 157 mcp (fmt, clippy
  `-D warnings`, rules.sh HARD-green; the history_tree.rs OVER GUARD is the owner-deferred item);
  `cargo check --target wasm32-unknown-unknown` green; headless `visual_driver.py` 3/3 scenarios
  green (15 checks) after the change; scripted real-binary About pass 4/4 under Xvfb (Help menu →
  About opens → the hyperlink renders in link color → Close hides it; evidence frame for the
  owner in the chat download dir).
- **Owner-checks:** V33 born (the About-window pass on Windows / browser).
- **Commit title:** `About window: "written in Rust" line + GitHub repo hyperlink`
