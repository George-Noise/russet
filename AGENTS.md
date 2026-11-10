# AGENTS.md

## Source of truth

- `SPEC.md` is the full product spec and the authoritative task list. Follow its milestones (M0–M9) in order.
- `PROGRESS.md` is the progress log and the substitute for git history (see "Git & progress log" below). Read it before starting any task and update it after every finished, verified one.
- Read `SPEC.md` §4–§7 before changing behavior. Do not add features from §6 (out of scope).

## Git & progress log

- The owner keeps git locally, but the `.git` folder is too heavy to transfer to the agent, and commits made inside the agent sandbox are auto-generated (UUID messages) and carry no usable information. **Git is not a progress channel here — `PROGRESS.md` (repo root) is.**
- Before starting work: read `PROGRESS.md` (status snapshot, warnings, latest log entries) to learn where the project actually stands.
- After finishing AND verifying a task or milestone: insert one entry at the top of the "Log" section of `PROGRESS.md` (newest first) — date, `M<N>:`-style title (or a short title for non-milestone work), what changed and in which files, how it was verified (commands + result), and what the owner still must check manually. Update the status snapshot and warnings in the same edit.
- **Commit titles (owner request 2026-11-10):** whenever a task leaves changes that will later go into a commit, ALWAYS propose a commit title for them — one short imperative line in the repo's existing `git log` style (multi-part changes joined with `;`), e.g. `Add Eyedropper, Line/Curve, Shape, Text tools and History/Layers windows`. State it both in the `PROGRESS.md` log entry (a `Commit title:` line) and in the reply to the owner, so the title can be copy-pasted when the owner commits locally. The agent never commits itself (see above).
- Never record work as done in `PROGRESS.md` before its verification commands actually pass.

## Commands

- Verify every milestone with: `cargo check`, then `cargo clippy -- -D warnings`, then `cargo fmt` / `cargo fmt --check`.
- Run tests with `cargo test` (unit tests are inline `#[cfg(test)]` modules, e.g. `canvas.rs`, `history.rs`).
- Daily run: `cargo run` (Windows). Do not run `cargo build --release` during development; release validation belongs to M9.
- Web (M8): `rustup target add wasm32-unknown-unknown`, `cargo install trunk --locked`, then `trunk serve` / `trunk build --release`. Trunk is not currently installed on this machine.

## Repo facts

- Single Rust binary crate `russet`: edition 2024, pinned deps from `SPEC.md` §1. Do not add dependencies.
- Required module split (monolith forbidden): `main`, `app`, `canvas`, `tools`, `history` (snapshot undo/redo + `ActionLog`), `file_io`, `palette`, `text` (text rasterization through egui's font atlas — owner request 2026-11-10). `app.rs` is the orchestrator (all `App` state, frame flow, hotkeys, strokes, file glue, menu, canvas widget); cohesive subsystems are its CHILD modules, which can access `App`'s private fields without any visibility churn: `app/camera.rs` (§3.4 viewport/zoom/pan), `app/dialogs.rs` (§3.7–3.8 dialogs), `app/windows.rs` (§3.2–3.3 floating windows), `app/placement.rs` (§3.11 placement), `app/preview.rs` (Line/Curve/Shape preview-then-commit gestures — owner request 2026-11-10), `app/text_tool.rs` (Text tool input window — owner request 2026-11-10), `app/panels.rs` (History log + Layers placeholder windows — owner request 2026-11-10; deliberately WITHOUT §3.11 placement). Keep new subsystems as such child modules instead of growing `app.rs`.
- File-size guard (soft): when a source file exceeds 500 lines, do not split it right away and never split mid-task. Instead, warn the owner that the file has outgrown the limit and propose a split — at the END of the current task, after the owner has verified that everything works. Split only with the owner's approval. Owner-approved exception (2026-11-09): `app.rs` stays the single orchestrator and may remain over 500 lines — agent readability beats the line count there; do not propose splitting it again unless it roughly doubles.
- Platform `cfg` branches may appear only in `main.rs` and `file_io.rs` (plus call sites in `app.rs`). The About window's platform/renderer strings are `crate::platform_label()` / `crate::renderer_label()` in `main.rs` — plain `cfg!()` macros (no `#[cfg]` attributes), mirroring winit's Linux backend choice: `WINIT_UNIX_BACKEND` override, then `WAYLAND_DISPLAY`/`WAYLAND_SOCKET` → Wayland, else `DISPLAY` → X11. Keep them in sync with the actual backend selection if winit's behavior changes.
- Canvas is a premultiplied `tiny-skia::Pixmap`; feed egui with `ColorImage::from_rgba_premultiplied`. Use `TextureHandle::set_partial(pos, image, options)` for dirty rectangles.
- File I/O: native uses synchronous `rfd::FileDialog`; WASM must use `rfd::AsyncFileDialog` only.
- Floating windows ("Tools"/"Palette"): placement (drop-time edge anchoring — a window keeps the gap the user dropped it at, never pulled flush; left/bottom tie rule — and per-axis repositioning on main-window resize: anchored axis keeps the dropped gap, free axis keeps the gap ratio) is specified in `SPEC.md` §3.11 and implemented in `app/placement.rs` with unit tests. Anchors are decided only when a drag ends — never during drags, never on resizes. First-open positions are §3.2/§3.3 facts: Tools 0.5rem (8 pt) from the left edge / vertically centered, Palette 0.5rem from the left and bottom edges — set via `Window::pivot` + `default_pos` (1 rem = 16 pt because egui 0.37 has no rem API; the `0.5 * REM` constant is `app/placement.rs::FIRST_OPEN_GAP`, next to the 2rem `WINDOW_SNAP_DISTANCE`); once the window has a stored area position, `default_pos` is dead and the user's placement wins.
- All user-facing strings are hardcoded English. No i18n/localization.
- `README.md` is written for HUMANS — people who want to build and run the app. It contains ONLY build & run instructions (Windows, NixOS/Linux via `flake.nix`, Web/WASM) and nothing else — no feature lists, no changelog, no agent guidance. Agents must read project rules here in AGENTS.md and in SPEC.md, never from README.md. When build/run commands change (SPEC §8, `flake.nix`, Cargo.toml toolchain requirements), update README.md in the same edit.
- `flake.nix` (owner decision 2026-11-09, an exception to SPEC §6's "Linux builds/CI"): builds/ runs the app on Linux (Wayland/X11) via `nix build` / `nix run` / `nix develop`. The binary links only fontconfig directly; ALL other GUI libs (wayland, libxkbcommon, X11 via `x11-dl`, GLX/EGL via glutin) are dlopen'd at runtime and are provided by `wrapProgram` (LD_LIBRARY_PATH). When `Cargo.toml` gains deps with runtime libs, update `runtimeLibs` in the flake. No vulkan-loader (glow never uses Vulkan).
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
- To move a `Window` programmatically (e.g. when a resize repositions a floating window), load `AreaState::load(ctx, area_id)`, call `set_left_top_pos`, and persist with `memory.areas_mut().set_state(LayerId::new(Order::Middle, area_id), state)` right after `Window::show` — `Window::current_pos` is discarded every frame in title-bar mode, and builder position calls would fight the drag.
- `Window::default_pos` applies only while the window's `AreaState` has no stored `pivot_pos` (fresh session; eframe `persistence` is off, so every launch is fresh). Pair it with `.pivot(Align2::…)` to pin a corner/edge instead of the left-top: `pivot(LEFT_BOTTOM)` + `default_pos(content_rect().left_bottom())` = flush bottom-left. `set_left_top_pos` converts through the stored pivot, so §3.11 repositioning stays correct with any pivot.
