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

- 2026-11-09 — `src/app.rs` is **1634 lines**, over the 500-line guard (AGENTS.md "Repo facts").
  Per the owner's decision the file is NOT split for now. The split must be proposed again at
  the end of some future task, after the owner has verified that everything works; do not
  split unprompted.

## Log — newest first. Insert a new entry directly under this heading.

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
