# AGENTS.md

## Source of truth

- `SPEC.md` is the full product spec and the authoritative task list. Follow its milestones (M0–M9) in order, one commit per milestone (`M<N>: <summary>`).
- Read `SPEC.md` §4–§7 before changing behavior. Do not add features from §6 (out of scope).

## Commands

- Verify every milestone with: `cargo check`, then `cargo clippy -- -D warnings`, then `cargo fmt` / `cargo fmt --check`.
- Run tests with `cargo test` (unit tests are inline `#[cfg(test)]` modules, e.g. `canvas.rs`, `history.rs`).
- Daily run: `cargo run` (Windows). Do not run `cargo build --release` during development; release validation belongs to M9.
- Web (M8): `rustup target add wasm32-unknown-unknown`, `cargo install trunk --locked`, then `trunk serve` / `trunk build --release`. Trunk is not currently installed on this machine.

## Repo facts

- Single Rust binary crate `russet`: edition 2024, pinned deps from `SPEC.md` §1. Do not add dependencies.
- Required module split (monolith forbidden): `main`, `app`, `canvas`, `tools`, `history`, `file_io`, `palette`.
- Platform `cfg` branches may appear only in `main.rs` and `file_io.rs` (plus call sites in `app.rs`).
- Canvas is a premultiplied `tiny-skia::Pixmap`; feed egui with `ColorImage::from_rgba_premultiplied`. Use `TextureHandle::set_partial(pos, image, options)` for dirty rectangles.
- File I/O: native uses synchronous `rfd::FileDialog`; WASM must use `rfd::AsyncFileDialog` only.
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
