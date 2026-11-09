# PROGRESS — Russet

> **Why this file exists.** The owner keeps git locally, but the `.git` folder is too heavy
> to share with the agent, and commits made inside the agent sandbox are auto-generated
> (UUID messages) and carry no usable information. Git is therefore NOT a progress channel
> in this setup. **This file is the substitute for the git history**: read it before
> starting work, append to it after every finished, verified task
> (see AGENTS.md "Git & progress log" for the exact routine).

## Status snapshot — update in the same edit as a log entry

| Milestone | Status | Notes |
|---|---|---|
| M0 Scaffold | done | |
| M1 Canvas + zoom/pan | done | |
| M2 Dirty-region texture updates | done | |
| M3 Tools + history | done | |
| M4 Menu + windows + §3.11 placement | done | §3.11 reworked once — see log 2026-11-09 "magnetism rework" |
| M5 Files (native, Windows) | done | |
| M6 Undo/Redo + hotkeys | done | |
| M7 Resize Canvas + New | done | |
| M8 Web (WASM) | code written, never built | `wasm32` branches exist in `main.rs` / `app.rs` / `file_io.rs`; trunk is unavailable in the sandbox — ALL wasm checks are deferred to the owner (Windows) |
| M9 Acceptance + release | pending (owner) | §7 checklist is manual, on the owner's Windows machine; `cargo build --release` is run by the owner |

## Warnings — open issues deliberately NOT fixed yet

- 2026-11-09 — RESOLVED: `src/app.rs` (1634 lines) was split into four child modules
  (`app/camera.rs`, `app/dialogs.rs`, `app/windows.rs`, `app/placement.rs`). `app.rs` itself
  remains ~690 lines BY OWNER'S DECISION — it stays the single orchestrator and may exceed
  the 500-line guard; do not propose splitting it further unless it roughly doubles
  (see the exception in AGENTS.md "Repo facts").

## Log — newest first. Insert a new entry directly under this heading.

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
