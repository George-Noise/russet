# AGENTS.md

## Source of truth

- `SPEC.md` is the full product spec and the authoritative task list. Follow its milestones (M0–M9) in order.
- `PROGRESS.md` is the progress log and the substitute for git history (see "Git & progress log" below). Read it before starting any task and update it after every finished, verified one.
- Read `SPEC.md` §4–§7 before changing behavior. Do not add features from §6 (out of scope).

## Git & progress log

- The owner keeps git locally, but the `.git` folder is too heavy to transfer to the agent, and commits made inside the agent sandbox are auto-generated (UUID messages) and carry no usable information. **Git is not a progress channel here — `PROGRESS.md` (repo root) is.**
- Before starting work: read `PROGRESS.md` (status snapshot, warnings, latest log entries) to learn where the project actually stands.
- After finishing AND verifying a task or milestone: insert one entry at the top of the "Log" section of `PROGRESS.md` (newest first) — date, `M<N>:`-style title (or a short title for non-milestone work), what changed and in which files, how it was verified (commands + result), and what the owner still must check manually. Update the status snapshot and warnings in the same edit.
- Never record work as done in `PROGRESS.md` before its verification commands actually pass.

## Commands

- Verify every milestone with: `cargo check`, then `cargo clippy -- -D warnings`, then `cargo fmt` / `cargo fmt --check`.
- Run tests with `cargo test` (unit tests are inline `#[cfg(test)]` modules, e.g. `canvas.rs`, `history.rs`).
- Daily run: `cargo run` (Windows). Do not run `cargo build --release` during development; release validation belongs to M9.
- Web (M8): `rustup target add wasm32-unknown-unknown`, `cargo install trunk --locked`, then `trunk serve` / `trunk build --release`. Trunk is not currently installed on this machine.

## Repo facts

- Single Rust binary crate `russet`: edition 2024, pinned deps from `SPEC.md` §1. Do not add dependencies.
- Required module split (monolith forbidden): `main`, `app`, `canvas`, `tools`, `history`, `file_io`, `palette`.
- File-size guard (soft): when a source file exceeds 500 lines, do not split it right away and never split mid-task. Instead, warn the owner that the file has outgrown the limit and propose a split — at the END of the current task, after the owner has verified that everything works. Split only with the owner's approval.
- Platform `cfg` branches may appear only in `main.rs` and `file_io.rs` (plus call sites in `app.rs`).
- Canvas is a premultiplied `tiny-skia::Pixmap`; feed egui with `ColorImage::from_rgba_premultiplied`. Use `TextureHandle::set_partial(pos, image, options)` for dirty rectangles.
- File I/O: native uses synchronous `rfd::FileDialog`; WASM must use `rfd::AsyncFileDialog` only.
- Floating windows ("Tools"/"Palette"): placement (drop-time edge anchoring — a window keeps the gap the user dropped it at, never pulled flush; left/bottom tie rule — and per-axis repositioning on main-window resize: anchored axis keeps the dropped gap, free axis keeps the gap ratio) is specified in `SPEC.md` §3.11 and implemented at the bottom of `app.rs` with unit tests. Anchors are decided only when a drag ends — never during drags, never on resizes.
- All user-facing strings are hardcoded English. No i18n/localization.
- Never `unwrap()`/`expect()` on user-data paths; report errors via `error_popup`. Panics are forbidden.

## eframe/egui 0.37 API quirks (verified against installed crate)

- `eframe::App` implements `fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame)`; there is no `update` method.
- `egui::CentralPanel::default().show(ui, ...)` takes `&mut egui::Ui`, not `&egui::Context`.
- `egui` is not a direct dependency: `use eframe::egui;` (or fully qualify `eframe::egui`).
- Keyboard-focus guard is `ctx.egui_wants_keyboard_input()`, not `ctx.wants_keyboard_input()`.
- `Ui::id()` is deprecated; use `Ui::scope_id()`.
- Panels are unified: `egui::Panel::top(id)` / `bottom` / `left` / `right`; there is no `TopBottomPanel`/`SidePanel`.
- Menu bar is `egui::MenuBar::new().ui(ui, |ui| { ui.menu_button(...) })`; there is no `egui::menu::bar`.
- Tool windows/panels are drawn with `egui::Window::new(...).show(&ctx, ...)`; input over them must not paint on the canvas (use response hover/occlusion, not raw pointer position).
- Title-bar drag of a `Window` is detected via `ctx.read_response(area_id.with("__title_click"))` (the same id `window.rs` uses internally).
- To move a `Window` programmatically (e.g. after a drag-time snap), load `AreaState::load(ctx, area_id)`, call `set_left_top_pos`, and persist with `memory.areas_mut().set_state(LayerId::new(Order::Middle, area_id), state)` right after `Window::show` — `Window::current_pos` is discarded every frame in title-bar mode, and builder position calls would fight the drag.
