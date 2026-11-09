# SPEC: "Russet" raster editor (Rust, Paint.NET-style MVP)

> **Agent instructions — read BEFORE starting work and follow literally.**
> 1. Read this file in full, then implement the milestones from §4 strictly in order, one at a time.
> 2. A milestone is considered done only when `cargo check` and `cargo clippy -- -D warnings` pass without errors or warnings; then run `cargo fmt`.
> 3. Do not add ANY dependencies beyond those listed in §1. Do not depend on `winit` or `wgpu` directly — they are already inside `eframe`.
> 4. Do not implement anything from §6 (out of scope). Do not improve what was not requested.
> 5. **All user-facing strings must be English only**, hardcoded (§0). No localization files, i18n crates, or translations.
> 6. If an internet example contradicts compiler errors, trust the compiler and the official template <https://github.com/emilk/eframe_template> for the `eframe` version pinned in Cargo.toml (newer eframe versions have a different signature for the `App` trait method than older articles).
> 7. Git is not a usable progress channel in this environment (the owner keeps git locally; the `.git` folder is not shared; sandbox commits are auto-generated). Instead, after each finished and verified milestone append one `M<N>: <short summary>`-style entry to `PROGRESS.md` (repo root) — see AGENTS.md "Git & progress log".
> 8. No `unwrap()`/`expect()` on execution paths involving user data (files, pixel indices) — show errors in a popup (§3.7); application panics are unacceptable. (`expect` in the `main.rs` entry point from the §2 template is allowed.)

---

## 0. Goal

A single-window raster application **"Russet"**: a canvas with drawing via three tools (pencil, brush, eraser), a top menu, floating "Tools" and "Palette" windows, undo/redo, zoom/pan, and PNG/JPEG open and save.

- **Platforms.** Development and running — **Windows**. A **web build (WASM, via trunk)** is required — milestone M8. Do not test the Linux build now (the code is portable anyway; `std::path` is allowed only in `file_io.rs`). **macOS is permanently out of scope** (the project owner has no Mac and never will).
- **UI language — English only.** All menu, button, dialog, and title labels are hardcoded English strings (lists are in §3). There will never be translations: do not add i18n.
- Development style: simple, straightforward code. No macro magic, no premature abstractions, no multithreading in application code.
- **Build:** daily work — dev builds only (`cargo run`), tuned for minimal rebuild time and low machine load (§1). The release profile is built manually by the owner when everything is ready — the agent does NOT need to build `--release` (except for the final M9 run).

## 1. Stack & build profile

| Crate | Version | Purpose |
|---|---|---|
| `eframe` (includes `egui`) | `0.37` | window (winit inside), event loop, all UI, rendering (glow — OpenGL, mipmap-capable) |
| `tiny-skia` | `0.11` | stroke rasterization into the `Pixmap` pixel buffer |
| `image` | `0.25`, `default-features = false`, `features = ["png", "jpeg"]` | PNG and JPEG read/write |
| `rfd` | `0.15` | open/save dialogs (native: synchronous; web: `AsyncFileDialog` only) |
| `log` + `env_logger` (native) | `0.4` / `0.11` | logging (on web — `eframe::WebLogger`) |
| `wasm-bindgen-futures`, `web-sys` (web) | `0.4` / `0.3` | web entry-point plumbing |

Web build tool: **trunk** (installed once, §8). It is not a Cargo dependency.

Full `Cargo.toml` — use as is:

```toml
[package]
name = "russet"
version = "0.1.0"
edition = "2024"
rust-version = "1.95"

[dependencies]
# Renderer: default eframe features MINUS wgpu PLUS glow (OpenGL).
# Rationale: no in-process Vulkan/D3D driver (~3-5x lower RAM), and glow is the
# only renderer in egui 0.37 that supports TextureOptions::mipmap_mode (§3.4).
eframe = { version = "0.37", default-features = false, features = [
  "glow",
  "accesskit",
  "clipboard",
  "default_fonts",
  "links",
  "system_fonts",
  "wayland",
  "x11",
] }
tiny-skia = "0.11"
image = { version = "0.25", default-features = false, features = ["png", "jpeg"] }
rfd = "0.15"
log = "0.4"

# native (Windows):
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
env_logger = "0.11"

# web:
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen-futures = "0.4"
web-sys = "0.3"

# FAST DEVELOPMENT:
# - own code rebuilds without optimizations (fast);
# - dependencies (eframe, glow, image...) are optimized ONCE and taken from cache —
#   the UI stays responsive in dev builds;
# - shorter debug info speeds up linking and barely loads the disk.
[profile.dev]
opt-level = 1
debug = "line-tables-only"

[profile.dev.package."*"]
opt-level = 2

# RELEASE: built by the owner MANUALLY when everything is ready (LTO is slow).
# Do NOT run `cargo build --release` during development.
[profile.release]
lto = "fat"
codegen-units = 1
strip = true
```

About build speed:
- cargo and trunk **already build in parallel on all cores** — do not pass `-j`, do not disable parallelism, do not touch `codegen-units` in the dev profile.
- The first `cargo run` is long (dependencies compile with optimizations) — this is normal; all subsequent rebuilds take seconds.
- Environment requirement: Rust newer than `1.95` (`rustup update`). For a Linux build, `libxcb-*-dev`, `libxkbcommon-dev`, and `libgtk-3-dev` will be needed eventually (for the `rfd` crate) — NOT required now.

## 2. Architecture

Project structure (the split is mandatory; a monolith is forbidden):

```
russet/
  Cargo.toml
  Trunk.toml      — trunk config: target web/index.html, output dist/ (§2.2)
  web/index.html  — page for trunk: <canvas id="russet_canvas">, title "Russet" (§2.2)
  PERF.md         — RAM / build-speed backlog; a design note, not a spec (§2)
  src/
    main.rs       — TWO entry points: native (run_native) and web (WebRunner); template below
    app.rs        — struct App (all state), impl eframe::App, the ORCHESTRATOR:
                    frame flow, hotkeys, strokes, file glue, menu, canvas widget
    app/             — child modules of app (they see App's private fields):
      camera.rs      — §3.4 viewport: canvas rect, zoom (Paint.NET-style steps), pan
      dialogs.rs     — §3.7/§3.8: New, Resize Canvas, unsaved-changes, Error dialogs
      windows.rs     — §3.2/§3.3: floating "Tools" and "Palette" windows
      placement.rs   — §3.11: floating-window edge anchoring + resize repositioning
    canvas.rs     — struct Canvas: pixel buffer, resize, pixel access
    tools.rs      — enum Tool, struct ToolState, stroke segment rasterization
    history.rs    — snapshot-based undo/redo
    file_io.rs    — open/save PNG/JPEG; the ONLY place with native/web cfg branches
    palette.rs    — fixed array of 34 preset colors (2 rows × 17)
```

**Tests are separate files** (owner decision, 2026-11-09): production sources contain no
`#[cfg(test)] mod tests { … }` blocks. A module with tests ends with `#[cfg(test)] mod tests;`
and the tests live in the sibling `<module>/tests.rs` (`src/canvas/tests.rs`,
`src/app/placement/tests.rs`, …). The tests file is a child module, so it reaches private items
with `use super::*;` and no visibility changes. There is no crate-root `tests/` directory (the
crate is a single binary, no lib target). Full rules: `AGENTS.md` → "Tests".

### 2.1 `main.rs` — skeleton (both platforms at once, following the eframe_template)

```rust
#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // no console in release on Windows

mod app;
mod canvas;
mod file_io;
mod history;
mod palette;
mod tools;

use app::App;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            // owner-set fixed minimum (not derived from the default)
            .with_min_inner_size([500.0, 400.0]),
        ..Default::default()
    };
    eframe::run_native("Russet", native_options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window().expect("No window").document().expect("No document");
        let canvas = document
            .get_element_by_id("russet_canvas")
            .expect("Failed to find russet_canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("russet_canvas was not a HtmlCanvasElement");
        let start_result = eframe::WebRunner::new()
            .start(canvas, web_options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
            .await;
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(()) => { loading_text.remove(); }
                Err(err) => {
                    loading_text.set_inner_html("The app has crashed. See the developer console for details.");
                    panic!("Failed to start eframe: {err:?}");
                }
            }
        }
    });
}
```

### 2.2 `web/index.html` (trunk entry page) + `Trunk.toml`

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=no" />
  <title>Russet</title>
  <!-- Trunk resolves href relative to THIS file; the manifest is one level up. -->
  <link data-trunk rel="rust" href="../Cargo.toml" />
  <style>
    html, body { margin: 0; padding: 0; width: 100%; height: 100%;
                 overflow: hidden; background: #1e1e1e; }
    #russet_canvas { width: 100%; height: 100%; display: block; touch-action: none; }
    #loading_text { position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%);
                    color: #cccccc; font-family: sans-serif; }
  </style>
</head>
<body>
  <canvas id="russet_canvas"></canvas>
  <div id="loading_text">Loading Russet…</div>
</body>
</html>
```

`Trunk.toml` (repo root) points trunk at the page and keeps the output out of the root:

```toml
[build]
target = "web/index.html"
dist = "dist"
```

Why the explicit `data-trunk` link: trunk defaults to looking for `Cargo.toml` next to the HTML
file, so once the page lives in `web/` the manifest must be declared (`href="../Cargo.toml"`).
The link also makes the wasm entry explicit instead of relying on trunk's auto-discovery.

Page requirement: nothing scrolls (`overflow: hidden`), the canvas fills the screen, otherwise Space and the mouse wheel will scroll the page instead of reaching the app.

## 3. Functional requirements

### 3.1 Top menu (all strings in English)
`egui::TopBottomPanel` + `egui::menu::bar`. Items:

- **File**: New… (Ctrl+N), Open… (Ctrl+O), Save (Ctrl+S), Save As… (Ctrl+Shift+S), separator, Exit (native build only; do not show on web).
- **Edit**: Undo (Ctrl+Z), Redo (Ctrl+Y; Ctrl+Shift+Z triggers the same redo action). Items are disabled when the corresponding stack is empty.
- **Canvas**: Resize Canvas…
- **View**: Zoom 100% (Ctrl+0), Fit to Window, separator, Tools and Palette checkboxes (open/close the corresponding windows).
- **Help**: About Russet (owner request, 2026-11-09) — opens the non-resizable, non-collapsible "About Russet" window: name, version (`CARGO_PKG_VERSION`), one-line description, `Platform:` — one of Wayland (Linux), X11 (Linux), Windows, macOS, Web (WASM) — and `Renderer:` (OpenGL on native, WebGL on web). The platform string comes from `crate::platform_label()` in `main.rs`, which mirrors winit's Linux backend choice (`WINIT_UNIX_BACKEND` override, then `WAYLAND_DISPLAY`/`WAYLAND_SOCKET` → Wayland, else `DISPLAY` → X11); the window has a Close button and no `open` flag (same pattern as the Error dialog).

### 3.2 "Tools" window
Floating `egui::Window` titled "Tools", open by default, movable, auto-sized (`Window::auto_sized`: the window hugs its content and the user cannot resize it). On its FIRST open it sits 0.5rem (8 pt) from the left edge, vertically centered in the main window: `Window::pivot(Align2::LEFT_CENTER)` + `default_pos(ctx.content_rect().left_center() + vec2(0.5 * REM, 0.0))`, where `REM = 16 pt` (egui 0.37 has no rem API; the constant is `app/placement.rs::FIRST_OPEN_GAP`). `default_pos` is consumed only while the window has no stored area position — afterwards the user's placement (§3.11) always wins.

- Tool buttons are stacked in a single column: "Pencil (P)", "Brush (B)", "Eraser (E)"; the active tool is visually highlighted (`selected`). The layout must already be prepared for growth: when more tools are added, it must be possible to switch to two columns without restructuring the window (use a configurable column count, not hardcoded row assumptions).
- "Size" slider 1..=100 — affects brush and eraser; disabled for pencil.
- The Tools window must NOT show the currently selected Primary/Secondary colors. Color previews and color selection live only in the Palette window.

### 3.3 "Palette" window
Floating `egui::Window` titled "Palette", open by default, auto-sized (`Window::auto_sized`: the window hugs its content and the user cannot resize it). On its FIRST open its bottom-left corner sits 0.5rem (8 pt) from the main window's left and bottom edges: `Window::pivot(Align2::LEFT_BOTTOM)` + `default_pos(ctx.content_rect().left_bottom() + vec2(0.5 * REM, -0.5 * REM))`, `REM = 16 pt` (see §3.2). `default_pos` is consumed only while the window has no stored area position — afterwards the user's placement (§3.11) always wins. No width-filling widgets are allowed inside: a plain `ui.separator()` fills all available width, and in an auto-sized window that width feeds back into the window size until the window spans the whole screen. The divider is pinned to the swatch-grid width: `ui.add_sized([GRID_WIDTH, 6.0], egui::Separator::default())`, where `GRID_WIDTH = COLS as f32 * TILE` comes from `palette.rs`.

Layout, top to bottom:

1. **Primary/Secondary color wells** (32×32) with the "Swap (X)" button to their right. No text captions — the wells are visually self-explanatory. Clicking a well opens the built-in egui color picker for THAT well's color (the former "Custom…" button, now one per color). The well is drawn manually (filled rect, corner radius 2, 1 px `from_gray(40)` inside stroke) and wired like egui's own color button: `Popup::menu(&response)` + `color_picker::color_picker_hsva_2d(…, Alpha::Opaque)`, writing the value back only when the picker reports a change. Do not write a custom HSV picker.
2. **The divider** (pinned width, see above).
3. **The preset grid** — a seamless 17×2 grid of 34 colors: tile gap 0 (both axes), tiles 20×20, painted with `rect_filled` only, **no outline stroke**. The grid is NOT an `egui::Grid`: `Grid` enforces a minimum column width (`interact_size.x`, 40 pt by default) and therefore leaves gaps between the 20 pt tiles. Instead, one exact-size block is allocated (`ui.allocate_exact_size(vec2(GRID_WIDTH, ROWS as f32 * TILE), Sense::click())`); tiles are painted at exact `(col * TILE, row * TILE)` offsets so neighboring tiles share edges (no seams), and clicks are resolved manually from `interact_pointer_pos()` — LMB sets Primary, RMB sets Secondary. Only the set of colors and their order are taken from the original Russet web palette (russetpaint.github.io) — nothing else is borrowed from it. The grid is fixed in `palette.rs`:

  | | | | | | | | | | | | | | | | | |
  |---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
  | `#000000` | `#404040` | `#6B6B6B` | `#FF0000` | `#FF6A00` | `#FFD800` | `#B6FF00` | `#4CFF00` | `#00FF21` | `#00FF90` | `#00FFFF` | `#0094FF` | `#0026FF` | `#4800FF` | `#B200FF` | `#FF00DC` | `#FF006E` |
  | `#FFFFFF` | `#C2C2C2` | `#949494` | `#7F0000` | `#7F3300` | `#7F6A00` | `#5B7F00` | `#267F00` | `#007F0E` | `#007F46` | `#007F7F` | `#004A7F` | `#00137F` | `#21007F` | `#57007F` | `#7F006E` | `#7F0037` |

- LMB click on a swatch — set Primary; RMB click — set Secondary.

### 3.4 Canvas: display, zoom, pan, coordinates

- The canvas is drawn in `egui::CentralPanel::no_frame()` (zero panel margin), so the dark gray `#404040` background reaches the window edges with no border.
- Under the image — a "checkerboard" (transparency): generate one 2×2 pixel texture once (colors `#FFFFFF` and `#CCCCCC`) with `TextureOptions` and Repeat wrap mode, draw it with `painter().image(...)` using uv values proportional to the on-screen canvas size (tile ~16 screen pixels). If Repeat causes problems — the acceptable fallback is a solid `#999999` fill.
- On top of the checkerboard — the canvas texture: `painter().image(canvas_tex.id(), screen_rect, uv(0..1), Color32::WHITE)`, where the screen rect comes from the physical pixel grid: zoom is defined in PHYSICAL screen pixels (100% = 1 texel per physical pixel — "honest" Actual Size at any Windows display scaling); physical size = `canvas_size * zoom`, `min_physical = ((panel_center + pan) * pixels_per_point - physical_size / 2).round()`, and the logical rect fed to egui is `min_physical / pixels_per_point` with size `physical_size / pixels_per_point`.
- **Canvas texture sampling (mandatory).** Define TWO shared options consts and pick per current zoom for the creation and EVERY update of the canvas texture. At integer zoom ≥ 100% (100%, 200%, …, 800%) — `TextureOptions { magnification: TextureFilter::Nearest, minification: TextureFilter::Linear, wrap_mode: TextureWrapMode::ClampToEdge, mipmap_mode: Some(TextureFilter::Linear) }`: the rect is snapped to the physical pixel grid, so each texel covers an exact whole number of physical pixels and NEAREST is perfectly crisp (Paint.NET-style pixel grid for pencil work). At any other zoom (non-integer 112–281%, and everything < 100%) — `TextureOptions { magnification: TextureFilter::Linear, minification: TextureFilter::Linear, wrap_mode: TextureWrapMode::ClampToEdge, mipmap_mode: Some(TextureFilter::Linear) }`: smooth bilinear/trilinear sampling, no dropped rows/columns. When zoom crosses between a crisp and a smooth level, re-upload the texture once (a full `set` inside the zoom-change handler) so the new filter takes effect immediately. `mipmap_mode` is supported only by the `glow` renderer (§1); glow re-applies the sampler params on every upload, including `set_partial`, so switching options per zoom works. This does not apply to the checkerboard texture — keep its existing options.
- **Zoom percentage indicator.** In the **bottom-right corner** of the `CentralPanel`, draw a small, **low-contrast** label showing the current zoom percentage, e.g. `"100%"`. Use a muted gray (e.g. `Color32::from_gray(140)`) so it does not stand out against the `#404040` panel background. Use `painter().text(...)` (or a small overlay `Area` with a `Label`) — do **not** add a status-bar panel (out of scope, §6). Always visible.
- **Zoom is abrupt, not smooth.** Every zoom change (Ctrl+wheel, Ctrl+0, Fit to Window, menu) must jump instantly to the next discrete level, in the same frame. Do **not** use `ctx.animate_value_with_time`, `lerp`, or any other time-based interpolation for the zoom value. The canvas rect is recomputed with the new `zoom` in the same frame.
- **Zoom — `Ctrl` + mouse wheel**, always toward the cursor position, through discrete Paint.NET-style zoom levels. Plain wheel (no modifiers) does **not** zoom; `Shift` + wheel does **not** zoom. Zoom must be implemented as a dedicated step method (for example `next_zoom(current, direction)`) instead of an inline continuous multiplier:
  - zoom in: `candidate = round(current * 1.12)`;
  - zoom out: `candidate = round(current / 1.12)`;
  - if the nearest multiple of 100% is within 6% of the candidate, snap to that multiple;
  - clamp to 10%..=800%.
  Zooming in from 100% must produce exactly: 112, 125, 140, 157, 176, 200, 224, 251, 281, 300, ... (same behavior as Paint.NET). The anchor formula is mandatory:
  `p = ((cursor - rect.min) * pixels_per_point) / zoom;` (canvas texels under the cursor); after changing `zoom`: `rect.min' = cursor - p * zoom_new / pixels_per_point` (logical), then the drawn rect is snapped to whole physical pixels (anchor error ≤ 0.5 physical px).
- **Scroll without modifiers — vertical pan.** Plain mouse wheel adjusts `pan.y` by the raw scroll delta multiplied by `SCROLL_PAN_SPEED` (= 1.5): egui's raw delta (40 pt per wheel notch on native) pans noticeably slower than Paint.NET, and the multiplier is tuned so panning feels ~1.5x slower than Paint.NET. No horizontal change.
- **`Shift` + scroll — horizontal pan.** `Shift` + mouse wheel adjusts `pan.x` by the raw scroll delta multiplied by `SCROLL_PAN_SPEED` (= 1.5, see above). No vertical change.
- **Console log on zoom change.** Centralize zoom updates in a single function and, on every change, emit:
  `log::info!("Zoom: {}%", (zoom * 100.0).round() as i32);`
  Native dev builds: run with `RUST_LOG=info` to see it (the default `env_logger` filter is `error`). Web: `WebLogger` is initialized at `Debug`, so it goes to the browser console without further config. Do not duplicate the log call.
- **Pan — mouse**: middle-button drag OR LMB drag while Space is held. Space-pan has priority: while Space is held, a stroke must not start. Space state — via `ctx.input(...).key_down(Key::Space)`.
- **Pan — `W`/`A`/`S`/`D` (hold)**: a held key pans the viewport continuously at `KEY_PAN_SPEED` (= 600 pt/s, screen-space, owner-tuned), scaled by the frame time (`stable_dt.min(0.1)`, so a stalled frame cannot jump the canvas). The letter `egui::Key`s are PHYSICAL keys (egui-winit maps them from winit's `PhysicalKey`), so the bindings work on ANY keyboard layout — ЙЦУКЕН's Ц/Ф/Ы/В included — without per-layout handling. "Camera" convention, consistent with the wheel: W/S/A/D move the VIEWPORT up/down/left/right, i.e. the canvas content moves the opposite way on screen. W+A / S+D diagonals are normalized (no speed boost). Gated by `!ctx.egui_wants_keyboard_input()` like the other hotkeys; while a pan key is held, a stroke must not start (same priority rule as Space).
- Zoom reset to 100%: Ctrl+0 and the "Zoom 100%" menu item; "Fit to Window" — `zoom = min(panel_w/canvas_w, panel_h/canvas_h)`, clamped to the same range.
- **Coordinate conversion (critical):** screen point → canvas point: `p_canvas = ((screen_pos - rect.min) * pixels_per_point) / zoom`; pixel: `(floor(p_canvas.x), floor(p_canvas.y))`; cast numbers explicitly to `i32`/`u32`; before indexing check `0 <= x < width` and `0 <= y < height`. Drawing outside canvas bounds is clipped, never panics.

### 3.5 Tools: pencil, brush, eraser

Common stroke lifecycle (for all tools):

1. `pointer press` (primary or secondary button) on the canvas → `history.push_snapshot()`; for pencil/brush choose color: LMB → `primary`, RMB → `secondary`; eraser erases with both buttons. Draw a touch point and remember `last_pos`.
2. `pointer drag` → on every move draw a **segment** from `last_pos` to the current position (in canvas coordinates), update `last_pos`.
3. `pointer release` → end of stroke: `is_stroking = false`.
4. Every segment returns a dirty rectangle (bbox of the segment expanded by `ceil(size/2) + 2` px, clipped to the canvas) — the texture is updated using it (§3.4/§5).
5. During a stroke the texture is updated per dirty region via `set_partial`; afterwards — `ctx.request_repaint()`.

Segment rasterization — in `tools.rs`. Brush and eraser: via `tiny-skia` (`PathBuilder` line from→to, `Stroke` with width and caps); segments with round caps leave no gaps, no separate point interpolation is needed. Pencil: classic integer **Bresenham** between the rounded endpoints (`from.round()` → `to.round()`), plotted pixel-by-pixel directly into the `Pixmap` (premultiplied source-over; opaque color overwrites the pixel) — exactly one pixel per column (per row for steep segments), corner-connected diagonals, never 2 px thick at any angle, and a single click sets exactly one pixel. Do NOT rasterize the pencil through tiny-skia path stroking: its non-AA scanline filler crosses the 50% coverage threshold depending on the subpixel position, so some diagonal pixels come out 2 px thick (ragged 1px/2px lines).

| Tool | Width | LineCap | Antialias | Color | Note |
|---|---|---|---|---|---|
| Pencil | 1 | — | — (integer Bresenham, not a tiny-skia stroke) | primary/secondary | hard 1-pixel trail, like in Paint: exactly one pixel per step at every angle |
| Brush | `size` | Round | on | primary/secondary | hard round brush, like "Brush" in Paint.NET |
| Eraser | `size` | Round | on | color ignored | **`paint.blend_mode = BlendMode::DestinationOut`** — erases to transparency |

Coordinate limits: segment points — `f32` in canvas space; clip to `0..width`, `0..height` on the rasterization side (clip the segment or rely on `tiny-skia` bounds checks — at the agent's discretion, but without panics and without going outside the `Pixmap`).

### 3.6 Undo / Redo

- A snapshot (the whole `Pixmap`) is pushed to history **before the start of every stroke** and **before changing the canvas size**.
- History limits: 20 snapshots and a total of ≤ 256 MiB (`width*height*4` per snapshot); on overflow, remove the oldest.
- A new snapshot clears redo. Open and New **reset history completely**.
- Undo restores the snapshot; the canvas texture is then updated in full (`set`), not `set_partial`.
- Undo/redo with empty history — no-op, no panics.

### 3.7 Files: New / Open / Save / Save As

- **New…** — dialog with Width/Height fields (default 800×600; min 1, max 4096; non-numeric input is impossible — `DragValue` or filtered `TextEdit`), OK/Cancel buttons. If `dirty` — confirmation first: "You have unsaved changes. Discard them and continue?" (OK/Cancel). The new canvas is filled white, history is reset, `display_name = "Untitled"`, `current_path = None`.
- **Open…** — if `dirty`, confirmation as above. Native: `rfd::FileDialog::new().add_filter("Images", ["png", "jpg", "jpeg"])` + `image::open`. Web: `rfd::AsyncFileDialog` (§3.10, milestone M8) + `image::load_from_memory`. Then: `to_rgba8` → `Pixmap::from_vec`; canvas size = file size; zoom 100%, pan (0,0), history reset, `display_name` = file name, on native `current_path = Some(path)`, `dirty = false`.
- **Save (Ctrl+S)** — if `current_path == None` (native) — behave like Save As. Write according to the path extension: `.png` — PNG RGBA; `.jpg`/`.jpeg` — JPEG (alpha composited onto a white background, quality 90). On success `dirty = false`. On web, Save always downloads a file to the browser using the current `display_name` (§3.10).
- **Save As…** — save dialog with "PNG image" and "JPEG image" filters; if the user did not enter an extension — add the one matching the selected filter. Update `display_name` and (native) `current_path`.
- Any error — NOT a panic and NOT `unwrap`: popup titled "Error" with text (`error_popup`), work continues.
- **Window title**: format `{"*" if dirty}{display_name} - Russet` (example: `*Untitled - Russet`). Update when `dirty`/`display_name` change via `ctx.send_viewport_cmd(egui::ViewportCommand::Title(...))`; on web this changes the browser tab title.
- Native file I/O is synchronous, on the main thread, without threads or `tokio`.

### 3.8 "Resize Canvas" dialog

`egui::Window` "Resize Canvas": Width/Height fields (prefilled with current values, min 1, max 4096), OK/Cancel buttons.
On OK: `history.push_snapshot()`, then `canvas.resize_canvas(w, h)` — content is preserved, top-left anchor, added area transparent; the texture is recreated in full; zoom and pan do not change.

### 3.9 Hotkeys — ONLY this list

| Keys | Action |
|---|---|
| `P` / `B` / `E` | Pencil / Brush / Eraser |
| `I` / `L` / `U` / `T` | Eyedropper / Line / Shape / Text (owner request 2026-11-10, see §6 exception) |
| `Ctrl+Z` | Undo |
| `Ctrl+Y`, `Ctrl+Shift+Z` | Redo (both keys) |
| `Ctrl+N`, `Ctrl+O`, `Ctrl+S`, `Ctrl+Shift+S` | New / Open / Save / Save As |
| `Ctrl+0` | Zoom 100% |
| `[` / `]` | Brush/Eraser size −1 / +1 (clamped 1..=100) |
| `Space` (hold) + LMB drag | pan |
| `W`/`A`/`S`/`D` (hold) | continuous pan (PHYSICAL keys — layout-independent, §3.4) |
| `Ctrl` + mouse wheel | zoom toward cursor through the §3.4 zoom-step method (abrupt, no smoothing) |
| mouse wheel (no modifiers) | vertical pan |
| `Shift` + mouse wheel | horizontal pan |

Implementation: via `ctx.input_mut(|i| i.consume_key(...))` and modifiers; **must** be gated by `!ctx.wants_keyboard_input()` — hotkeys must not fire while focus is in dialog input fields. Do not add other hotkeys. egui matches modifiers logically (extra Shift/Alt are ignored), so test the most specific shortcut first: `Ctrl+Shift+Z` (redo) before `Ctrl+Z` (undo); the redo keys are consumed even when the redo stack is empty, so an exhausted stack is a no-op and never falls through to undo. On web the browser may keep some Ctrl combinations for itself — do not fight it; menu items must always work.

### 3.10 Web build (WASM) — specifics

- Built via **trunk** (§8), page — `web/index.html` (§2.2, config in `Trunk.toml`), canvas `#russet_canvas`.
- **File dialogs — `rfd::AsyncFileDialog` only**: synchronous `rfd::FileDialog` panics under wasm and is forbidden on web.
  - Open: `AsyncFileDialog::new().add_filter("Images", ["png","jpg","jpeg"]).pick_file().await` → `file.read().await` (bytes) → `image::load_from_memory`.
  - Save: `AsyncFileDialog::new().add_filter(...).set_file_name(&display_name).save_file().await` → `handle.write(bytes).await` — the browser downloads the file. `current_path` is not used on web.
- The async result is placed into `Arc<Mutex<Option<Result<LoadedImage, String>>>>`, then `ctx.request_repaint()` is called after completion; the App retrieves the result on the next frame. Use `spawn_local` from `wasm_bindgen_futures`.
- Platform cfg branches are allowed only in `file_io.rs` and at call sites in `app.rs`; the rest of the code knows nothing about the platform.
- Exit is not shown in the menu on web; do NOT add beforeunload protection against closing the tab (§6).

### 3.11 Floating-window placement: drop-time edge anchoring and resize repositioning

Applies to the two existing floating windows, "Tools" and "Palette" — no new windows, no changes to their content, no collapsing/shadows/resize handles (§6): plain `auto_sized` egui windows as before. All distances are in logical points; **2rem = 32 pt** (egui 0.37 has no rem API; 2 × the 16 pt default body text size).

**Edge magnetism — the window is fixed where the user drops it.** While a window is dragged by its title bar it follows the pointer freely: the magnet **never moves a window during the drag** and never pulls it flush to an edge — a gap of 0 happens only when the user drops the window flush. When the drag ends, each axis whose edge was dropped **closer than 2rem** to the corresponding edge of `ctx.content_rect()` becomes **anchored** to that edge at the dropped gap — the window stays exactly where the user placed it, and only this anchor state is remembered. **Tie rule:** if BOTH edges of one axis are simultaneously within 2rem of their respective edges (the window is nearly as wide/tall as the main window), the axis anchors to the **left** edge on x and the **bottom** edge on y. Dropping the window outside all 2rem zones clears every anchor.

**Repositioning on main-window resize — never snaps.** When the main window is resized (`ctx.content_rect()` changed between frames) — and never while a drag is in progress — each floating window is repositioned per axis: an **anchored** axis slides with its edge, **keeping the dropped gap** (clamped into the new range so the window never overflows it); a **free** axis keeps its relative placement via the **gap-ratio formula**. Per axis, with `old`/`new` = previous/new content-rect span and `win` = window span:

```text
min_gap  = max(0, win.min - old.min);  max_gap = max(0, old.max - win.max)
ratio    = min_gap / (min_gap + max_gap)        // 0 = flush left/top, 1 = flush right/bottom, 0.5 = centered
free_new = max(0, (new.max - new.min) - (win.max - win.min))
pos      = clamp(new.min + ratio * free_new, new.min, new.min + free_new)
```

so a top-left window stays top-left, a centered one stays centered, a bottom-right one stays bottom-right. Resizes never set, clear or change anchors and never pull a free window flush: shrinking the main window must not snap windows at 2rem — anchoring is a decision the user makes only by dropping the window.

**Implementation notes.** Per-window `WindowPlacement` (fields in `App`) stores the last shown rect, whether the window is being dragged, and the `WindowAnchor` fixed when the last drag ended (`window_anchor`); `place_floating_window` (in `app/placement.rs`, together with all §3.11 math and its unit tests) runs right after each `Window::show`; the title-bar drag response is read as `area_id.with("__title_click")` (the id `window.rs` uses internally); a reposition is persisted into the next frame by rewriting the window's `AreaState` (`set_left_top_pos` + `areas_mut().set_state`, layer `Order::Middle`) — `Window::current_pos` is discarded every frame in title-bar mode and cannot be used. Repositioning compares the previous frame's window rect and `content_rect`.

## 4. Milestones — do them in order

### M0. Scaffold
Package `russet` (edition 2024), Cargo.toml from §1, `main.rs` with both entry points per §2.1, `App::new` + a "Hello, Russet!" placeholder in CentralPanel.
**Done:** `cargo run` on Windows opens a 1200×800 window titled "Russet"; check/clippy/fmt are clean.

### M1. Canvas on screen + zoom/pan
`Canvas` 800×600, filled white; egui texture (`ColorImage::from_rgba_premultiplied`); checkerboard; drawing in CentralPanel using the §3.4 formulas; **`Ctrl`+wheel** zoom toward cursor through the §3.4 discrete zoom-step method (abrupt, no smoothing); **plain wheel = vertical pan, `Shift`+wheel = horizontal pan**; middle-button pan and Space+LMB; Ctrl+0 and "Fit to Window"; **low-contrast zoom-percentage label in the bottom-right of the CentralPanel**; **`log::info!` of the new zoom % on every zoom change**.
**Done:** `Ctrl`+wheel zoom steps toward the cursor through the Paint.NET-style levels across the whole 10–800% range (abrupt, no smoothing); plain wheel pans vertically, `Shift`+wheel horizontally; pan works via middle-button and Space+LMB; when the window is resized, the canvas stays in place relative to `pan`; the zoom % label is visible in the bottom-right corner and the console shows each zoom change (with `RUST_LOG=info` on native / automatically on web).

### M2. Dirty-region texture updates
Method `App::sync_texture(rect)`: the region is read from the `Pixmap` and the texture is updated via `set_partial` (the region position is in texture pixels!). `App::repaint_full()` stub — a full `set`.
Unit test: `region_bytes` returns `w*h*4` bytes and correct corner pixels.
**Done:** temporary filling of an arbitrary canvas rectangle via a test button updates only that rectangle on screen.

### M3. Tools + history
`Tool`/`ToolState`, segment rasterization per the table in §3.5 (pencil without AA, AA brush, eraser `DestinationOut`), press/drag/release stroke lifecycle, history snapshot at stroke start, `History` with the limits from §3.6. Drawing with LMB/RMB (secondary color). Coordinate bounds checking.
Unit tests: `History` — 20-snapshot limit and eviction; push clears redo.
**Done:** all three tools draw according to the table; the eraser makes pixels transparent (checkerboard visible); no gaps when the mouse moves fast; no panics when drawing outside the canvas.

### M4. UI: menu and windows
Menu from §3.1 (file and undo items are no-op stubs for now), "Tools" window from §3.2, "Palette" window from §3.3, window checkboxes in View, floating-window placement per §3.11 (drop-time edge anchoring — the window keeps the gap the user dropped it at, never pulled flush; left/bottom on ties — and per-axis repositioning on main-window resize: anchored axis keeps the dropped gap, free axis keeps the gap ratio). Ensure that clicks on windows do not paint on the canvas underneath them (egui does this itself — verify).
**Done:** the tool can be selected from the window too, size changes with the slider; palette LMB/RMB changes colors, "Swap (X)" works, clicking a color well opens the egui picker for that color; windows can be reopened from View; dropping a window within 2rem of a main-window edge anchors that axis at the dropped gap without pulling it flush, and re-dropping it farther away releases it; resizing the main window keeps an anchored window's dropped gap and a free window's relative placement.

### M5. Files (native, Windows)
`file_io.rs`: New (dialog + confirmation), Open, Save, Save As per §3.7 — synchronous rfd dialogs; `dirty`, `display_name`, `current_path`, window title via `ViewportCommand::Title`; "Error" popup. Do NOT implement web branches — they will appear in M8.
**Done:** a PNG with transparency saves and opens correctly; JPEG opens and saves on a white background; Ctrl+S on a new file opens the save dialog; the unsaved-changes confirmation works; the title shows `*`.

### M6. Undo/Redo + hotkeys
Bind undo/redo to the menu and the §3.9 keys; full texture update on undo/redo; disable menu items when history is empty; all hotkeys from the §3.9 table including `[`/`]`; gate with `wants_keyboard_input`. The scroll controls (`Ctrl`+wheel, plain wheel, `Shift`+wheel) must match the §3.9 table.
**Done:** 20 undo steps with a large number of strokes; redo works until a new stroke; hotkeys do not fire while focus is in dialog input fields.

### M7. Resize Canvas + New
Dialog from §3.8: history snapshot, resize with content preserved (top-left anchor), full texture recreation. The "New" dialog from M5 (if it was a stub — finish it: min 1, max 4096, white background).
**Done:** enlarging and shrinking the canvas does not lose the drawing, the added area is transparent; undo after resize restores the previous size and content.

### M8. Web build (WASM)
Set up web per §8 (`rustup target add wasm32-unknown-unknown`, `cargo install trunk --locked`), add `web/index.html` + `Trunk.toml` from §2.2, run `trunk serve` and get the app working in the browser. Then add web branches to `file_io.rs` per §3.10 (`AsyncFileDialog`: open via `read()`, save via `save_file().write()`, result via `Arc<Mutex<Option<...>>>` + `ctx.request_repaint()`); cfg forks at `app.rs` call sites. Verify in the browser: drawing, `Ctrl`+wheel zoom, wheel / `Shift`+wheel pan, undo, Open/Save.
**Done:** in the browser at `http://localhost:8080` all three tools draw, the eraser makes pixels transparent, PNG opens and downloads, the tab title updates, `Ctrl`+wheel zoom and wheel / `Shift`+wheel pan work, zoom % label and zoom-change console log are visible; `trunk build --release` succeeds.

### M9. Acceptance and polish
Run the §7 checklist in full (Windows + web), fix what is found. Optional (only if everything else is ready): round brush/eraser cursor preview (a circle outline with radius `size/2 * zoom` at the current cursor position).
**Done:** the entire §7 checklist passes manually without errors; `cargo build --release` succeeds.

## 5. Known pitfalls — read before coding

1. **Premultiplied alpha.** tiny-skia's `Pixmap` is premultiplied RGBA8. In egui use `ColorImage::from_rgba_premultiplied`, otherwise semi-transparent colors will look wrong. When writing files, account for premultiplication when compositing onto white (or demultiply before creating `image::RgbaImage`).
2. **`set_partial`**: the first argument is the region position/rectangle IN TEXTURE COORDINATES, and the data must be exactly `w*h*4` bytes for the region. A size mismatch panics egui.
3. **Zoom toward cursor** must use the anchor formula and the discrete zoom-step method from §3.4, otherwise the canvas "drifts" or the wheel produces the wrong percentage sequence. Zoom is bound to **`Ctrl`+wheel only**; plain wheel must pan vertically, `Shift`+wheel must pan horizontally. Zoom is applied **abruptly** — no smoothing or animation; do not use `ctx.animate_value_with_time`.
4. **Coordinates**: all screen→canvas→pixel conversions are explicit (`f32` → `floor` → `i32`/`u32` + clamp/bounds check). Unchecked `as` casts are a source of panics at boundaries.
5. **`eframe::App` signature** in 0.37 differs from old tutorials (the method may not be named `update`). Do not copy old code — look at the template for the same version and at compiler messages.
6. **Eraser** is `BlendMode::DestinationOut` on the `Pixmap`, not drawing with the background color.
7. **Right click**: CentralPanel has no context menu by default, but do not call `context_menu`: RMB with pencil/brush = secondary color, with eraser = erase.
8. **Space-pan**: while Space is held — do not start a stroke; if a stroke is already in progress, Space does not interrupt it (pan activates only on the next press).
9. **rfd: platforms differ.** Native (Windows) — synchronous `rfd::FileDialog`. Web — ONLY `rfd::AsyncFileDialog` (the synchronous one panics under wasm) + `spawn_local` + `ctx.request_repaint()`.
10. **Browser shortcuts**: on web, Ctrl+S/Ctrl+O/Ctrl+N (and `Ctrl`+wheel page zoom) may be captured by the browser — this is acceptable and not a "bug"; do not try to block browser behavior.
11. **UI strings** — hardcoded English only (§0). No `rust-i18n`/`fluent`/translation files.
12. **Build**: the dev profile from §1 is already tuned for speed and cargo parallelizes automatically — do not change or disable anything. Do not run `cargo build --release` (slow fat-LTO) during development, except for the final M9 check.
13. **Zoom is in PHYSICAL pixels.** Do not compute the canvas rect in logical points: physical size = `canvas_size * zoom`, top-left snapped to whole physical pixels, logical = physical / `pixels_per_point`; every screen→canvas conversion multiplies by `pixels_per_point`. Otherwise "100%" is uneven under Windows display scaling and strokes land offset from the cursor.
14. **Floating-window anchoring (§3.11)** is decided only when a title-bar drag ENDS: the magnet never moves a window during the drag and never pulls it flush; the anchor keeps the dropped gap on later resizes. Resize repositioning must never anchor, unanchor or snap anything. `Window::current_pos` is discarded every frame in title-bar mode — persist a new position via the window's `AreaState`, not via `Window` builder calls.

## 6. Out of scope — do NOT do in this version

Layers; selection and moving fragments; text; shapes (line/rectangle/ellipse); fill (flood fill); eyedropper; transforms (rotate/flip/image scale); action history list; saving application settings; recent files; themes/appearance customization; custom window decorations (`set_decorations(false)`); multithreading in application code (rayon/std::thread/tokio); any new dependencies; status bar; custom project format; internationalization and localization (in any form); macOS and Linux builds/CI; beforeunload tab protection on web; PWA/offline mode/service worker; wasm size optimization (wasm-opt, etc.).

Exception (owner decision, 2026-11-09): `flake.nix` for building and running the app on **Linux** (Wayland/X11, `nix build` / `nix run` / `nix develop`) is part of the repo and must be kept in sync with `Cargo.toml` (deps, binary name, runtime dlopen libs). This does not re-enable anything else from this list.

Exception (owner request, 2026-11-10): the owner explicitly ordered deliberately simple first iterations of the following §6 items, to be reworked later — **Eyedropper** (canvas picking only), **Line/Curve tool**, **Shapes** (rectangle + ellipse), **Text** (font size only), a **non-clickable action-history log** window, and a **Layers placeholder window** (single implicit layer). New hotkeys `I`/`L`/`U`/`T` (§3.9); no new dependencies; no new `#[cfg]` platform branches. Implemented behavior is described in `PROGRESS.md` (2026-11-10 entry), which is the source of truth for these provisional features until they are formally specified. This does not re-enable anything else from the list.

## 7. Acceptance checklist — run manually at the end of M9

**Windows (native):**
1. A 1200×800 "Russet" window opens; the File/Edit/Canvas/View/Help menus, Tools and Palette windows, and a white 800×600 canvas on a checkerboard are visible. The Tools window sits 0.5rem (8 pt) from the left edge, vertically centered, and the Palette window sits 0.5rem from the left and bottom edges; the main window cannot be resized below 500×400. Tools are stacked in one column and the Tools window does not show Primary/Secondary colors. The gray canvas background reaches the window edges (no margin); the Tools and Palette windows hug their content and have no resize handles; the Palette window is as wide as its swatch grid (not the full screen); in the Palette window the Primary/Secondary wells (no captions) and "Swap (X)" are on top, the seamless gap-0 swatch grid is below, and clicking a well opens the color picker for that well.
2. Pencil: hard 1px line (no antialiasing), LMB — Primary, RMB — Secondary; a diagonal at any angle is a 1 px staircase (classic Bresenham), never 2 px thick; a single click sets exactly one pixel.
3. Brush: smooth line; sizes 1 and 100 produce correct thickness; `]`/`[` change the size.
4. Eraser: erases to transparency (checkerboard visible), with both buttons.
5. Fast mouse zigzags leave no stroke gaps; drawing beyond the canvas edge does not panic and is clipped.
6. `Ctrl`+wheel zoom toward cursor 10–800% with Paint.NET-style levels (from 100%: 112, 125, 140, 157, 176, 200, 224, 251, 281, 300), applied abruptly (no smoothing); plain wheel pans vertically, `Shift`+wheel pans horizontally; pan with middle button and Space+LMB; Ctrl+0 returns to 100%; Fit to Window fits the canvas; zoom % label visible in the bottom-right of the panel; console logs each zoom change (with `RUST_LOG=info`); at 100%/200%/…/800% on any Windows display scaling the canvas is pixel-exact (each texel an exact whole number of physical pixels), non-integer zoom and < 100% are smoothly interpolated.
7. Undo/redo: a sequence of >20 strokes can be undone at most 20 steps and does not panic on empty history; redo is reset by a new stroke; Ctrl+Y and Ctrl+Shift+Z both redo, and Ctrl+Shift+Z never performs undo (including when there is nothing to redo).
8. New: 300×200 gives a white canvas; values 0 and 5000 are rejected; when dirty, a confirmation is shown.
9. Open: a PNG with alpha is displayed correctly on the checkerboard (colors are not distorted — premultiplied check).
10. Save PNG → the file opens in an external viewer, transparency is intact; Save JPEG → background is white.
11. Resize Canvas: content is not lost, the new area is transparent, undo restores the old size.
12. Hotkeys do not fire while focus is in a dialog input field.
13. Title looks like `*Untitled - Russet` when there are unsaved changes; Save removes the `*`.
14. Floating-window placement (§3.11): dropping Tools/Palette within 2rem of a main-window edge anchors that axis at the dropped gap — the window is NOT pulled flush to the edge (gap 0 only if it was dropped flush); when both edges of one axis are within 2rem, it anchors left (x) / bottom (y); re-dropping the window farther than 2rem from all edges clears the anchors; after resizing the main window an anchored window keeps its dropped gap to that edge (both when growing and shrinking), a free window keeps its relative placement (top-left stays top-left, centered stays centered, bottom-right stays bottom-right), and resizing never anchors, unanchors or pulls anything flush.

**Web (browser):**
15. `trunk serve`: the page opens, "Loading Russet…" disappears, the app fills the screen with no page scrolling.
16. Drawing, eraser, `Ctrl`+wheel zoom, plain-wheel vertical pan, `Shift`+wheel horizontal pan, middle/Space pan, undo/redo work in the browser; zoom % label is visible in the bottom-right; console logs each zoom change.
17. Open opens a PNG; Save downloads a file with the correct name; JPEG downloads with a white background.
18. The browser tab title shows `*Untitled - Russet` when dirty.
19. There is no synchronous `rfd::FileDialog` call on web (AsyncFileDialog only) — the app does not panic during file operations.
20. About: Help → "About Russet" opens a small non-resizable window with the name, version, and a `Platform:` line — `Windows` on Windows; `Wayland (Linux)` under a Wayland session (or `WINIT_UNIX_BACKEND=wayland`), `X11 (Linux)` under X11 (or `WINIT_UNIX_BACKEND=x11`); `Web (WASM)` in the browser — plus a `Renderer:` line (OpenGL native / WebGL web). Close hides it.

**Build:**
21. `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `cargo test` — all pass; `cargo build --release` succeeds (M9); `trunk build --release` succeeds; `nix build` on Linux succeeds and the wrapped binary runs in a Wayland and an X11 session.

## 8. Build & run

```bash
rustup update                                   # Rust >= 1.95 required

# --- Daily development (Windows) ---
cargo run                                       # dev build; first run is long (deps once), then seconds
cargo test                                      # unit tests, one file per module (src/<module>/tests.rs)
cargo clippy --all-targets -- -D warnings       # lints the tests too

# --- Release: MANUALLY, when everything is ready (fat LTO, slow) ---
cargo build --release

# --- Web (milestone M8), installed once ---
# Run from the repo root: Trunk.toml points at web/index.html and dist/.
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
trunk serve                                     # http://127.0.0.1:8080
trunk build --release                           # static files in dist/
```

cargo and trunk build in parallel on all logical cores by default — no `-j` flags are needed and none should be passed.
(For future reference: a Linux build will require the packages `libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libgtk-3-dev` — do NOT install or test this now. Owner decision 2026-11-09: on Linux the app is built and run via `flake.nix`, which needs none of these packages.)

A human-readable copy of the build & run instructions lives in `README.md` (English; build & run only — that file is written for people, not for agents). Keep it in sync with this section and `flake.nix` when commands change.
