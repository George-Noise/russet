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
| `winit` (native, Unix) | `0.30.13`, feature `wayland-csd-adwaita` | the same crate eframe resolves (cargo unifies them); the feature switches the Wayland CSD frame from sctk's bare `FallbackFrame` (its `set_title` is a no-op, no double-click-to-maximize) to `AdwaitaFrame` — title text + double-click maximize on GNOME (owner-approved 2026-11-16) |
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

# Linux/Wayland CSD frame (owner-approved 2026-11-16 — the full rationale
# lives in Cargo.toml): AdwaitaFrame instead of sctk's bare FallbackFrame —
# title text, Adwaita buttons, double-click maximize on GNOME.
[target.'cfg(all(unix, not(target_arch = "wasm32")))'.dependencies]
winit = { version = "0.30.13", features = ["wayland-csd-adwaita"] }

# web:
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen-futures = "0.4"
web-sys = "0.3"

# DEV — FULLY UNOPTIMIZED (owner decision 2026-11-18; the full rationale
# lives in Cargo.toml): the crate AND every dependency compile at opt-level 0,
# no debuginfo — dev builds must be fast, UI lag in `cargo run` is accepted
# (owner: "it's dev — it's fine if it lags").
[profile.dev]
opt-level = 0
debug = false

# RELEASE: built by the owner MANUALLY when everything is ready (LTO is slow).
# Do NOT run `cargo build --release` during development.
[profile.release]
lto = "fat"
codegen-units = 1
# NOT `strip = true` (= --strip-all): it deletes the wasm `target_features`
# custom section, and wasm-bindgen can no longer detect the enabled Wasm
# features — it warns on every release build and guesses. `debuginfo` keeps
# the custom sections; the dist/ wasm is cut to size by wasm-opt AFTER
# wasm-bindgen anyway (§2.2, PERF.md §1/§4).
strip = "debuginfo"
```

About build speed:
- cargo and trunk **already build in parallel on all cores** — do not pass `-j`, do not disable parallelism, do not touch `codegen-units` in the dev profile.
- The dev profile is FULLY unoptimized (owner decision 2026-11-18), so even the first build compiles without optimizations and stays in minutes; the app itself runs unoptimized — UI lag in `cargo run` is expected and accepted (dev is for visual checks, not feel).
- Environment requirement: Rust newer than `1.95` (`rustup update`). For a Linux build, `libxcb-*-dev`, `libxkbcommon-dev`, and `libgtk-3-dev` will be needed eventually (for the `rfd` crate) — NOT required now.

## 2. Architecture

Project structure (the split is mandatory; a monolith is forbidden). The normative
requirement is the module split listed below, NOT a file tree: no file tree is kept in the
docs (owner decision 2026-11-11) — agents list the actual files with their own tools when
they need them. Test files follow the `<module>/tests.rs` rule below and are implied for
every tested module; they are not listed anywhere.

Mandatory top-level modules: `main` (TWO entry points: native `run_native` and web
`WebRunner`; template in §2.1), `app` (`struct App` — all state; the ORCHESTRATOR: frame
flow, undo/redo, the canvas widget, texture sync), `canvas` (the COMPOSITE display
buffer: visible layers blended, resize, pixel access), `tools` (enum Tool, struct
ToolState, stroke rasterization), `history` (the read-only `ActionLog` convenience
view), `layers` (REAL layers: buffers, per-layer PNG-snapshot history trees,
composite — owner request 2026-11-13), `file_io` (open/save PNG/JPEG; native/web cfg branches live
only here, in `app/document.rs` and at `app.rs` call sites — §3.10), `palette` (fixed
array of 34 preset colors, 2 rows × 17), `text` (text
rasterization through egui's font atlas). Cohesive subsystems are CHILD modules of `app`
in `app/` — they see App's private fields; new subsystems are added there, never grown
into `app.rs` (current ones: viewport/camera, dialogs, document lifecycle, input
handling, top menu, floating windows, §3.11 placement, preview gestures, text tool,
the Tool Log and Layers windows (the former `panels` module, split 2026-11-17), History Tree windows, feature-gated MCP server; see the module map at the top
of `app.rs`). Root web config: `Trunk.toml` targets
`web/index.html` → `dist/` (§2.2). `PERF.md` is the RAM / build-speed reference backlog —
a design note, not a spec.

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
  <!-- Trunk resolves href relative to THIS file; the manifest is one level up.
       --all-features: trunk 0.22 runs wasm-opt on release builds, and its
       bundled wasm-opt 131 rejects the bulk-memory ops Rust 1.99 emits. -->
  <link data-trunk rel="rust" href="../Cargo.toml" data-wasm-opt-params="--all-features" />
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

`data-wasm-opt-params="--all-features"` is a **build requirement**, verified on trunk 0.22.1:
release builds run wasm-opt by default, and the bundled wasm-opt 131 rejects Rust 1.99's
bulk-memory instructions (`memory.copy operations require bulk memory operations`) without the
feature flags. This is not the wasm-size optimization from §6 — the measured effect is only
5.32 MB → 5.02 MB (`-O`; `-Oz` gives 4.94 MB) — it just makes trunk's default release pipeline
succeed.

Page requirement: nothing scrolls (`overflow: hidden`), the canvas fills the screen, otherwise Space and the mouse wheel will scroll the page instead of reaching the app.

## 3. Functional requirements

### 3.1 Top menu (all strings in English)
`egui::TopBottomPanel` + `egui::menu::bar`. Items:

- **File**: New… (Ctrl+N), Open… (Ctrl+O), Save (Ctrl+S), Save As… (Ctrl+Shift+S), separator, Exit (native build only; do not show on web).
- **Edit**: Undo (Ctrl+Z), Redo (Ctrl+Y; Ctrl+Shift+Z triggers the same redo action). Items are disabled when the corresponding stack is empty.
- **Canvas**: Resize Canvas…
- **View**: Zoom 100% (Ctrl+0), Fit to Window, separator, Tools, Palette, History Tree, Tool Log and Layers checkboxes (open/close the corresponding windows; the Tool Log checkbox was labeled "History" until 2026-11-13).
- **Help**: About (owner request 2026-11-13 renamed the ITEM from "About Russet"; the window it opens keeps the "About Russet" title; owner request, 2026-11-09) — opens the non-resizable, non-collapsible "About Russet" window: name, version (`CARGO_PKG_VERSION`), one-line description ("A tiny bitmap paint program written in Rust with egui/eframe." — owner request 2026-11-18: the window must say the project is written in Rust), a GitHub repository hyperlink ("GitHub: George-Noise/russet" → `https://github.com/George-Noise/russet`, owner request 2026-11-18; `ui.hyperlink_to` opens the system browser on native builds through eframe's `links` feature and `window.open` on web), `Platform:` — one of Wayland (Linux), X11 (Linux), Windows, macOS, Web (WASM) — and `Renderer:` (OpenGL on native, WebGL on web). The platform string comes from `crate::platform_label()` in `main.rs`, which mirrors winit's Linux backend choice (`WINIT_UNIX_BACKEND` override, then `WAYLAND_DISPLAY`/`WAYLAND_SOCKET` → Wayland, else `DISPLAY` → X11); the window has a Close button and no `open` flag (same pattern as the Error dialog).
- **Theme** (owner request 2026-11-18; v2 pass the same day): a menu-bar button BETWEEN View and Help, labeled just `Theme` — a menu with three radio options: Dark, Light and System (System follows the OS preference and is the STARTUP default). The plain labels are deliberate (owner v2 pass): no ": Dark" suffix on the button — the picked theme is visible by opening it — and no "(follow the OS)" explanation on the System option; Help ALWAYS stays the LAST menu item. The choice applies immediately through `App::apply_theme` → `Context::set_theme` (egui re-styles every widget the next pass; the icon atlas re-rasterizes through its `(pixels_per_point, theme)` key) and the Tool Log records it ("Theme: Dark" etc.). The MCP `set_theme` tool (dark/light/system) runs the SAME code path — the owner asked for the switcher in BOTH the top panel and MCP.

### 3.2 "Tools" window
Floating `egui::Window` titled "Tools", open by default, movable, auto-sized (`Window::auto_sized`: the window hugs its content and the user cannot resize it). On its FIRST open it sits 0.5rem (8 pt) from the left edge and a bit ABOVE the vertical center: 40% of the FREE vertical space (the space not occupied by the window) is above the window and 60% below it (owner request 2026-11-11). `Window::pivot(Align2::LEFT_CENTER)` + `default_pos(ctx.content_rect().left_center() + vec2(0.5 * REM, 0.0))` set the left gap and a centered start (`REM = 16 pt` — egui 0.37 has no rem API; the constant is `app/placement.rs::FIRST_OPEN_GAP`); the exact 40/60 split is applied ONCE on the first frame, when the auto-sized height is first known, through the placement glue (`app/placement.rs::TOOLS_FIRST_OPEN_RATIO` + `first_open_ratio_pos`). `default_pos` is consumed only while the window has no stored area position — afterwards the user's placement (§3.11) always wins.

- Tool buttons are ICON buttons in a TWO-COLUMN grid (owner request 2026-11-18: "two columns for now — it will be reworked later, I have a better idea"; the rework is debt F15, the column count stays the configurable `app/windows.rs::TOOL_COLUMNS`). Each button is a 1.5rem (24 pt) icon centered in a small square with the selectable-label visual states — selected = the selection fill + stroke, hover = the hovered fill; the active tool is visually highlighted. Since the v4 pass (owner 2026-11-18) the REST/HOVER fills are the light-theme widget fills DIMMED A STEP (−14 per channel, `tool_button`'s `dim`) — the full gray(230) light-theme button contrasted hard against BOTH themes' window background; the selection fill keeps the bright selection blue so the active tool pops. Hovering a button shows a TOOLTIP with the tool's name and its keyboard key ("Pencil (P)", "Brush (B)", … — the single source `app/tool_icons.rs::label`). The icons are RASTERIZED ONCE per scale change by the pure `app/tool_icons.rs` (tiny-skia — a dependency the canvas raster ops already pay) into egui textures via `Context::load_texture_cached`: a button's per-frame cost is one textured quad, never a re-tessellation (owner request 2026-11-18). The icons are FULLY THEME-INDEPENDENT since the v4 pass — fixed rims (below) + the FIXED monochrome ink `TOOL_INK` (gray 64 — the light-theme text color a bit darker, owner v4: the Line/Shape/Text icons must not flip between near-white and near-black) — so the atlas key is just `pixels_per_point`; the old theme re-raster died with its reason (the v2 theme-aware rims/ink). COLORED and MONOCHROME icons share the SAME draw pipeline (closed polygons + round-cap strokes through the shared primitives, plus the arc/quadratic sampling helpers): colored are Pencil (classic yellow body with light/dark facets, wood cone, black graphite tip, metal ferrule, eraser on the end), Brush (varnished-wood handle, small metal ferrule, bristles with a loaded tip), Eraser (the classic pink two-face slab, the same dark-plum rim as the set — owner v4: its light-pink outline blended with the faces and broke the style) and Eyedropper (a pipette — a CYLINDRICAL glass tube with a blue liquid stripe, a cone ONLY at the tip (owner v3 pass: "still a bit like a needle"), a WIDE metal collar ring between tube and bulb — owner: "the wide piece between the glass and the bulb made the image stick" — and a dominant round rubber bulb); monochrome (the fixed `TOOL_INK`) are Line (an S-CURVE, not a straight segment — the tool itself has curve mode), Shape (a rectangle and a CIRCLE — the owner replaced the diamond) and Text (a serif-curved big T with a small lowercase t beside it). The colored icons' outline is 1.25 pt, the SAME dark rim in BOTH themes and one rim hue per tool (owner v3 pass 2026-11-18: the v2 theme-aware light rim was REVERTED — the tool buttons carry the LIGHT-theme look in the dark theme instead, so ONE drawing reads well everywhere; `windows.rs::tool_button` picks its resting/hover/selected fills from `egui::Visuals::light()` regardless of the active theme). The hand-held tools point down-left. The Eyedropper's icon doubles as its CANVAS CURSOR (owner request 2026-11-18): while the Eyedropper is active and the pointer is over the canvas, the pipette is pushed as the OS cursor via `Context::set_cursor_image` (egui's `CustomCursorImage` — straight alpha, hotspot at the pipette tip, `Arc` identity dedupe in egui-winit) with `CursorIcon::Crosshair` as the fallback for backends without custom cursors (web). The Tools window itself contains ONLY the tool list — the per-tool settings live in the magnetized panel below (owner request 2026-11-17). The window's TITLE BAR is kept but its LABEL is hidden (owner v5 2026-11-18 — the v4 `title_bar(false)` removed the whole bar while only the title TEXT had to go): the title is an EMPTY STRING, which renders the bar (drag strip + buttons) minus the text (`window.rs::title_ui` builds the bar from the button atoms regardless of the text), and the explicit `tools_window_id()` keeps the identity — egui derives a window's Id from its title, an empty one included (`Window::id` doc: "Required if the title changes, or is shared with another window"); the ✕ (`.open`) and View ▸ Tools both close the window. Since the v6 pass (owner 2026-11-19, "variant A2" — the owner's pick among the measured header-width options) the strip FITS the window: egui's default title frame carried the window frame's 6-pt side margin, and an auto-sized window's strip is free to PAINT wider than the window, so the both-buttons strip (2 Heading-height button atoms + 4 item-spacing gaps = 73.4 pt of content) painted ~85.4 pt against the ~74-pt window and the ✕ landed half outside it (AGENTS.md quirk). The fix is `Window::title_frame` fed with egui's default window frame whose `inner_margin.left/right` are ZEROED (`app/windows.rs::tools_title_frame`): the strip now paints exactly the window's width, both buttons sit inside it, and the VERTICAL padding stays — the strip's height, fill and rounded top corners are unchanged (variant A, `Margin::ZERO`, additionally shrank the strip vertically and shifted the window's first-open position; not picked).
- The per-tool settings that used to sit below the tool list inside the Tools window live in a SEPARATE panel magnetically attached to it (owner request 2026-11-17): a title-bar-less, immovable, content-hugging `egui::Window` (`app/windows.rs::tool_settings_window_builder`), whose position `Window::fixed_pos` re-applies EVERY frame from the Tools window's drawn rect — no stored area position (a drag, a stale state) can ever win over it, so the panel cannot be dragged or otherwise detached from the Tools window ("with no way to detach it, for now"). The position math is the pure `app/placement.rs::magnet_settings_pos` with the state `ToolSettingsPlacement` (the panel's LAST DRAWN size — egui knows the true size of an auto-sized window only after its first, invisible layout pass):
  - Vertical: the panel prefers the space BELOW the Tools window — its top edge sits a `TOOL_SETTINGS_GAP` = 0.25rem (4 pt) gap below the Tools window's bottom edge. When it would not fit between that edge and the main window's bottom edge, it flips ABOVE the Tools window — its bottom edge sits the same gap above the Tools top edge. A panel that fits nowhere (pathologically small main window) is clamped into the main window (egui's own area constrain is the backstop).
  - Horizontal: the panel aligns the edge on the side of the main-window center the Tools window sits on — Tools window left of center → the panel's LEFT edge aligns with the Tools window's left edge; right of center → the RIGHT edges align (exactly centered counts as right).
  - The panel is drawn only while the Tools window shows its BODY AND the current tool has settings: Pencil and Eyedropper have none, so for them the panel is not drawn at all. A COLLAPSED Tools window (the title-bar collapse button, or double-clicking the bar) shows no tool list, so the panel hides with it (owner request 2026-11-19) and comes back the frame the window is expanded again; the guard is `Window::show`'s `inner.is_some()`, which egui reports as `None` exactly while the collapsing body is closed (`app/windows.rs::settings_panel_shown`, headless regression test + the `scripts/gui_collapse_check.sh` probe).
- Per tool the panel shows: the "Size" slider 1..=100 for Brush, Eraser, Line and Shape — SHOWN and ENABLED for these; a tool without a size setting gets NO slider, not a disabled one (owner request 2026-11-17 retired the former disabled-when-unapplicable slider that lived in the Tools window). The Line tool adds (owner request 2026-11-20 — the pass that reshaped the Shape panel, "такие же по стилю и отступам") ONE row of TWO icon buttons in the Tools window's button style — the same 28-pt squares, the light-theme states, the tooltips — replacing the former word "Mode" radio: Straight and Curve (`app/windows.rs::line_settings_ui`), the buttons drawing the LINE TOOL's emblem (`tool_icons::LineIcon` — the mark the Tools window's Line button strokes: the Curve choice pixel-for-pixel the tool's icon, the Straight choice the same stroke unbent), sitting EXACTLY the Tools grid gap `app/windows.rs::TOOL_GRID_GAP` (6 pt) apart, the tooltips carrying the old words (`tool_icons::line_icon_label`); in Curve mode the two hint lines stay UNDER the row. The Shape tool adds the two KIND icon buttons, the Fill / Outline / Fill+Outline MODE icon radio and the outline-PLACEMENT icon radio (below); the Text tool shows the two placement hint lines.
- Shape (U) — since owner request 2026-11-20 the word radio and the checkboxes are ICON BUTTONS in the Tools-window button style (the same 1.5rem icon squares, light-theme states, tooltips): the KIND row is two buttons — the square and the circle drawn as neutral previews (a mid-gray fill `app/tool_icons/draw.rs::SHAPE_PREVIEW_FILL` + the fixed ink rim) selecting Rectangle / Ellipse — and, after a group gap (`app/windows.rs::SHAPE_GROUP_GAP`, the distance-separates-meaning principle; the owner's 2026-11-20 spacing pass: WITHIN a row the buttons sit EXACTLY the Tools window's element gap `app/windows.rs::TOOL_GRID_GAP` (6 pt) apart horizontally, and the rows separate VERTICALLY by 12 pt — egui's 3-pt item spacing plus the gap — a step FURTHER apart than that horizontal gap, so the three groups read as groups), the MODE row is a THREE-WAY RADIO over `app/tools.rs::ShapePaint` (the follow-up the same day: "не 2 выбора, а 3 и это уже будет radio"; the former two TOGGLES' "both on" state was reachable only by a hidden second click — the explicit third choice replaces it, and EXACTLY one paint is ever active): Fill (the Shape TOOL's emblem — the rectangle + circle of the Tools window's Shape button — drawn SOLID in the fixed ink), Outline (the SAME emblem HOLLOW, rim only — pixel-for-pixel the tool's icon) and Fill + Outline (the emblem with the gray fill UNDER the ink rim — both ingredients visible, the vocabulary the kind previews speak). The MODE icons do NOT repeat the selected kind (the V51 toggles previewed it; the follow-up: "не повторение выбранной фигуры, а иконка инструмента Shape"), so they are kind-INDEPENDENT and the atlas key is pixels-per-point ALONE again (a kind switch re-rasterizes nothing). Hovering a button shows the old words — "Fill" / "Outline" — and the third radio names both: "Fill + Outline" (`tool_icons::shape_icon_label`). The PRIMARY well paints the outline, the SECONDARY the fill — in Fill + Outline BOTH wells paint (the fill goes first, the stroke covers it, `tools.rs::rasterize_shape`); drawing with the RIGHT mouse button uses the two wells INVERTED (owner request 2026-11-15: the colors are used the other way around and NO swap happens) — the former secondary paints the outline and the former primary the fill for that one gesture, while the Palette wells stay exactly as they are. Left-button drawing and the Line/Text right-button behavior (which merely USES the other well) are unchanged. Holding `Shift` during a Shape drag constrains it to a PROPORTIONAL shape — a square for the Rectangle, a circle for the Ellipse (Paint.NET-style, owner request 2026-11-16): the drag's start point is the anchor corner and the free corner is pulled, LIVE (evaluated every frame, so pressing/releasing Shift mid-drag snaps/releases instantly), onto the square whose side is the dominant axis delta — the larger of |dx|, |dy| — with each axis keeping its drag direction, so the shape always grows toward the pointer; a strictly one-sided drag (one delta zero) still yields a square — the zero axis extends by the full side in the fixed down/right direction (the constraint lives in the pure `app/preview.rs::proportional_current`). The Line tool is NOT affected: Shift during a Line drag does nothing (its only Shift use is the §3.9 horizontal pan). The Shape panel's THIRD row (owner request 2026-11-20, after another group gap) is a second THREE-WAY RADIO — the outline PLACEMENT over `app/tools.rs::ShapeOutline`: OUTSIDE (the outline lies fully beyond the two mouse points — its inner edge is the shape's boundary, the footprint grows by the stroke width), CENTERED (the outline straddles the boundary half in half out — exactly how every earlier version drew, the default) and INSIDE (the outline lies fully within the two points — its outer edge is the boundary, the footprint stays exactly the drag; a shape thinner than the stroke keeps a clamped non-degenerate path and the stroke simply covers it). The placement rides the gesture like the colors do (a mid-drag switch cannot change what the release commits), the on-canvas preview matches the commit for every mode (the rectangle strokes with egui's Outside/Middle/Inside stroke kind; the ellipse samples radii shifted by the same half stroke width), the Fill-only paint keeps the radio clickable (the choice is remembered for the next outlined shape), and the radio's three icons always draw a SQUARE's corner — whatever kind is selected — as the gray fill quadrant (where the shape is), the click dot (a white core with the ink ring at the corner) and two ink segments — horizontal and vertical — along the outline's lines in the mode's position (`app/tool_icons/draw.rs::draw_outline_mode`).
- The Tools window must NOT show the currently selected Primary/Secondary colors. Color previews and color selection live only in the Palette window.

### 3.3 "Palette" window
Floating `egui::Window` titled "Palette", open by default, auto-sized (`Window::auto_sized`: the window hugs its content and the user cannot resize it). On its FIRST open its bottom-left corner sits 0.5rem (8 pt) from the main window's left and bottom edges: `Window::pivot(Align2::LEFT_BOTTOM)` + `default_pos(ctx.content_rect().left_bottom() + vec2(0.5 * REM, -0.5 * REM))`, `REM = 16 pt` (see §3.2). `default_pos` is consumed only while the window has no stored area position — afterwards the user's placement (§3.11) always wins. No width-filling widgets are allowed inside: a plain `ui.separator()` fills all available width, and in an auto-sized window that width feeds back into the window size until the window spans the whole screen. The divider is pinned to the swatch-grid width: `ui.add_sized([GRID_WIDTH, 6.0], egui::Separator::default())`, where `GRID_WIDTH = COLS as f32 * TILE` comes from `palette.rs`.

Layout, top to bottom:

1. **Primary/Secondary color wells** (32×32) with the "Swap" button to their right. No text captions — the wells are visually self-explanatory. The button has NO hotkey and its label names none (owner request 2026-11-15: the former "Swap (X)" implied a key §3.9 deliberately does not list — §3.9 stays exhaustive). Clicking a well opens the built-in egui color picker for THAT well's color (the former "Custom…" button, now one per color). The well is drawn manually (filled rect, corner radius 2, 1 px `from_gray(40)` inside stroke) and wired like egui's own color button: `Popup::menu(&response)` + `color_picker::color_picker_hsva_2d(…, Alpha::OnlyBlend)`, writing the value back only when a change is reported. The picker edits the ALPHA too (owner request 2026-11-19: `Alpha::OnlyBlend` instead of the former `Alpha::Opaque` — egui's alpha slider and the A DragValue appear, and egui stops forcing the stored alpha to 1.0, which used to silently erase an Eyedropper-picked translucent pixel on the first popup open); the drawing pipeline composites alpha end-to-end, so a translucent well paints translucently. Under the picker sits a hex field (owner request 2026-11-19: "hex is needed too often") parsing the SAME junk-tolerant syntax as the `--primary`/`--secondary` launch flags (§3.12): every non-hex-digit character is STRIPPED before parsing (an optional `0x` prefix dropped first), so `##FF7F00`, `ff 7f 00` and `0xFF7F00` all parse (owner request 2026-11-19: keep only what means color, delete the rest brazenly), and the remaining DIGIT COUNT picks the form: 3 = `RGB` (each digit doubled), 4 = `RGBA` (each doubled), 5 = `RGBAA` (the shorthand RGB + a full alpha byte), 6 = `RRGGBB`, 7 = `RRGGBBA` (the full RGB + a doubled alpha digit), 8 = `RRGGBBAA` — the 5- and 7-digit forms carry different RGB digit counts (3 vs 6) BY DESIGN, the owner calls the asymmetry deliberate. An alpha-carrying entry (4/5/7/8 digits) SETS the alpha; a short entry (3/6) KEEPS the alpha the well had when this edit session started (a per-well seed alpha re-snapshotted together with the text on popup open / outside changes — a live-typed longer entry passes through alpha-carrying prefixes, and the seed keeps such a prefix from leaking its alpha into the final color); the field always displays `#RRGGBBAA` and re-formats to it when the edit ends (Enter/click-elsewhere) or when the color changed outside the field (fresh popup, sliders). Do not write a custom HSV picker — the hex field is an input row under egui's picker, not a picker.
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
- **Scroll without modifiers — vertical pan.** Plain mouse wheel adjusts `pan.y` by a FIXED 15 pt per wheel notch on EVERY platform (`WHEEL_NOTCH_PAN`; F12 variant B, owner pick 2026-11-16, INTERIM — not the final design). The raw `Event::MouseWheel` delta is first converted to NOTCHES per its `unit` (`app/camera.rs::wheel_notches`): native `Line` (Windows/X11 and Wayland discrete events — winit sends ±1 per notch) passes through; on the WEB backend `Line` counts LINES and is divided by the interim 3 lines/notch (Firefox emits `DOM_DELTA_LINE` with 3 lines per click and eframe passes the DOM delta as-is — the owner's overshoot report 2026-11-16: one Firefox click panned ~3× the Windows/Chrome notch); `Point` (eframe-web passes the DOM wheel pixels as-is — Chrome ≈ 100 px per notch — and winit-Wayland sends pixel deltas the same way) is divided by the interim 100 pt, `Page` is capped at 3 notches; fractional line/pixel deltas (touchpads, high-resolution wheels) stay proportional. The 15 pt folds in F11 (owner: "another 5× faster", calibrated on Windows where the pre-fix notch was ±3 pt = ±1 line × the old `SCROLL_PAN_SPEED` 3.0). The interim Line(web)/Point/Page constants are retuned from `--features diagnostics` wheel logs (one `[diagnostics]` line per event: unit, source, raw delta → notches → the applied pan step). History: the raw delta used to pan per EVENT UNIT (Windows ±3 pt vs Web ≈ ±300 pt per notch) — F12 born 2026-11-16, fixed the same day; the web-Line divisor added the same day after the owner's Firefox report. No horizontal change.
- **`Shift` + scroll — horizontal pan.** `Shift` + mouse wheel adjusts `pan.x` the same way: the same unit → notch conversion (both delta axes summed), the same fixed 15 pt per notch. No vertical change.
- **Console log on zoom change.** Centralize zoom updates in a single function and, on every change, emit:
  `log::info!("Zoom: {}%", (zoom * 100.0).round() as i32);`
  Native dev builds: run with `RUST_LOG=info` to see it (the default `env_logger` filter is `error`). Web: `WebLogger` is initialized at `Debug`, so it goes to the browser console without further config. Do not duplicate the log call.
- **Pan — mouse**: middle-button drag OR LMB drag while Space is held. Space-pan has priority: while Space is held, a stroke must not start. Space state — via `ctx.input(...).key_down(Key::Space)`.
- **Pan — `W`/`A`/`S`/`D` (hold)**: a held key pans the viewport continuously at `KEY_PAN_SPEED` (= 600 pt/s, screen-space, owner-tuned), scaled by the frame time (`stable_dt.min(0.1)`, so a stalled frame cannot jump the canvas). The letter `egui::Key`s are PHYSICAL keys (egui-winit maps them from winit's `PhysicalKey`), so the bindings work on ANY keyboard layout — the Russian typewriter layout's W/A/S/D keys included — without per-layout handling. "Camera" convention, consistent with the wheel: W/S/A/D move the VIEWPORT up/down/left/right, i.e. the canvas content moves the opposite way on screen. W+A / S+D diagonals are normalized (no speed boost). Gated by `!ctx.egui_wants_keyboard_input()` like the other hotkeys; while a pan key is held, a stroke must not start (same priority rule as Space).
- Zoom reset to 100%: Ctrl+0 and the "Zoom 100%" menu item; "Fit to Window" — `zoom = min(panel_w/canvas_w, panel_h/canvas_h)`, clamped to the same range.
- **Coordinate conversion (critical):** screen point → canvas point: `p_canvas = ((screen_pos - rect.min) * pixels_per_point) / zoom`; pixel: `(floor(p_canvas.x), floor(p_canvas.y))`; cast numbers explicitly to `i32`/`u32`; before indexing check `0 <= x < width` and `0 <= y < height`. Drawing outside canvas bounds is clipped, never panics.

### 3.5 Tools: pencil, brush, eraser

Common stroke lifecycle (for all tools):

1. `pointer press` (primary or secondary button) on the canvas → `history.push_snapshot()`; for pencil/brush choose color: LMB → `primary`, RMB → `secondary`; eraser erases with both buttons. Draw a touch point and start the stroke's smoothing window at that point.
2. `pointer drag` → collect EVERY coalesced `Event::PointerMoved` of the frame (egui carries the whole OS pointer stream of the frame; `latest_pos` alone keeps only the last one and turns fast strokes into long straight runs), convert each to canvas coordinates and feed the smoothing window (`tools::smooth::SmoothStroke`): the first fed point emits the straight filler `start → mid(start, p)`, every following point emits the quadratic piece `mid(prev2, prev1) → (control prev1) → mid(prev1, p)` — tangent-continuous (C1) at every joint, never overshooting (a quadratic stays inside its control triangle). A duplicate of the previous point emits nothing.
3. `pointer release` → the window's closing piece (`mid(prev1, p) → p`) rasterizes FIRST, then `is_stroking = false` and the gesture records its node (§3.6).
4. The frame's pieces rasterize in ONE batch and return their UNION dirty rectangle (per piece the bbox of its `from`/`control`/`to` — the curve stays inside its control hull — expanded by `ceil(size/2) + 2` px, clipped to the canvas) — the texture is updated once per frame using it (§3.4/§5).
5. During a stroke the texture is updated per dirty region via `set_partial`; afterwards — `ctx.request_repaint()`.

Smoothed-piece rasterization — in `tools/smooth.rs` (`rasterize_smooth_pieces`). Brush and eraser: the frame's pieces form ONE continuous tiny-skia path (`PathBuilder::quad_to`, `Stroke` with width and round caps, AA) — the pieces join at their shared midpoints, so there are no per-segment cap seams; a perfectly horizontal or vertical piece chain has zero-height/width path bounds, which tiny-skia rejects — the final point is nudged by a subpixel epsilon (the same trick `rasterize_polyline` uses). Pencil: the same smoothed centerline rasterized as the classic hard 1-pixel trail by a **drifting pen** (`PencilTrail`, owner request 2026-11-20) — a gesture-level state carried across pieces AND frames. Quads are flattened into ~1 px steps (the step count is the ceil of the control-polygon length, a 4096 sanity cap); the pen walks toward each flattened sample in UNIT steps of the chord-dominant axis, its minor coordinate advancing proportionally and carried as an EXACT float — never rounded into the state — which is the same error accumulation one long Bresenham run performs, with NO per-run restarts. Each step plots the pen's rounded position (one pixel per integer grid crossing, 8-connected by construction). The carried fractional phase makes the drawn staircase IDENTICAL no matter how the pointer stream is batched into frames — the old code re-ran Bresenham between independently rounded ~1 px endpoints, which stacked double-covered columns and punched holes into every diagonal, with the pattern varying from computer to computer (the coalesced frame batching differs); the pixel trail now depends only on the smoothed geometry. The smoothed path of integer-rounded pointer events still wobbles locally past 45 degrees, so an isolated stacked pixel can appear — part of the path's honest staircase, kept rare (roughly ≤ 1 per 10 px of span; regression-tested). The release pins the trail on the raw release point (`pin` argument, wired by `app/input.rs::flush_stroke_close`), so the stroke ends exactly on its pixel like the classic run. Consecutive duplicate pixels are skipped (the dedupe lives in the trail) because premultiplied source-over is not idempotent — a translucent color must not double-blend at the subdivision/frame joints. A single click sets exactly one pixel. Do NOT rasterize the pencil through tiny-skia path stroking: its non-AA scanline filler crosses the 50% coverage threshold depending on the subpixel position, so some diagonal pixels come out 2 px thick (ragged 1px/2px lines).

| Tool | Width | LineCap | Antialias | Color | Note |
|---|---|---|---|---|---|
| Pencil | 1 | — | — (the drifting pen's 1 px trail along the flattened smoothed centerline, not a tiny-skia stroke) | primary/secondary | hard 1-pixel trail, like in Paint, following the SMOOTHED path: one pixel per dominant step at every angle, frame-batch independent, 8-connected |
| Brush | `size` | Round | on | primary/secondary | hard round brush, like "Brush" in Paint.NET |
| Eraser | `size` | Round | on | color ignored | **`paint.blend_mode = BlendMode::DestinationOut`** — erases to transparency |

Coordinate limits: smoothed-piece points — `f32` in canvas space; clip to `0..width`, `0..height` on the rasterization side (clip the piece chain or rely on `tiny-skia` bounds checks — at the agent's discretion, but without panics and without going outside the `Pixmap`).

### 3.6 Undo / Redo — per-layer history trees (owner request 2026-11-13)

The history lives PER LAYER as a git-style tree of nodes (`layers.rs` owns the stack,
`history_tree.rs` the DAG); the prototype's 20-snapshot stack is gone.

- Every completed drawing gesture is ONE node on the ACTIVE layer's tree: pencil/brush/
  eraser strokes and the Line/Curve/Shape/Text commits. A node carries the action kind,
  the stroke colors and a full PNG snapshot of the layer's canvas AFTER the action
  (lossless fast-compressed PNG in RAM, `layers::encode_snapshot`, image crate
  `CompressionType::Fast`; the roundtrip is bit-exact, so no rebuild exists yet —
  actions arrive later). Single-color gestures (the freehand tools, Line, Text, and a
  Shape with only one of the two checkboxes on) carry ONE color; a Shape commit that
  used BOTH the outline and the fill carries BOTH — outline in `color`, fill in
  `color2` (owner request 2026-11-16: the History Tree row's color chip shows both —
  split into outline|fill halves — for the tools that used two colors; the
  2026-11-13 one-color-per-node rule narrowed to the single-color tools).
- Each tree has a permanent **Start root** whose snapshot is the layer's initial canvas:
  undo past the first stroke lands on it, and clicking it restores the initial canvas.
- Undo (Ctrl+Z) / redo (Ctrl+Shift+Z, Ctrl+Y) / the Edit menu / the MCP `undo`/`redo`
  tools move the head one node on the ACTIVE layer's tree and restore the pixels from
  the target node's PNG (full texture update, `set`, not `set_partial`). Redo follows
  the "last left branch" rule: the child the last undo left, else the first-created
  child. No-ops (Start root, no children) never panic.
- LEFT-clicking a node's DOT in a History Tree window (the label is never a click
  target) jumps the head there and shows that node's canvas; drawing right after forks
  the tree git-style — the abandoned tail stays and stays reachable.
- No node-count limit at this stage (owner decision 2026-11-13): the PNG compression
  keeps memory small; eviction-by-memory arrives together with the action model.
- **Sizes (owner request 2026-11-17): the BOTTOM layer owns the canvas size** — its buffer
  IS the canvas; every other layer keeps its own size and the composite draws layers at
  (0,0), clipped to the canvas (mixed-size layers are legal). New layers are canvas-sized;
  New/Open rebuild the stack with the one layer that IS the canvas.
- **Resize commits ONE `resize` node PER LAYER** (owner request 2026-11-17: "record into
  each layer's history-tree node all the choices made in the resize window"):
  the new `ActionKind::Resize` carries a full `ResizeInfo` — the mode (canvas/image), the
  layer's before/after sizes and the mode parameter (the 3×3 anchor or the resampling
  filter). The History Tree row prints the summary ("resize 12 canvas 800×600→1000×800
  (TL)"), the dot hover explains it, the MCP `get_history_tree` dumps the whole struct —
  jumping a layer back beyond the resize tells the user what that resize did to THIS
  layer. Canvas mode adapts every layer by the SAME anchor geometry (pixels keep their
  positions relative to the anchor corner); Image mode rescales every layer
  proportionally by the canvas factor with the chosen filter (premultiplied-correct,
  the pinned `image` crate's `imageops`). Restoring an old node restores its NATIVE size
  (the 2026-11-13 crop/pad-fit restore is retired) — a bottom-layer restore resizes the
  canvas with it (the bottom layer owns the size), a non-bottom one just becomes
  smaller/larger and the composite clips it.
- The owner's manual walkthrough scenarios of the resize × history coupling (chat,
  2026-11-17) have an automated counterpart: the OWNER-LOCKED whole-app scenario suite
  `app/scenarios` (headless real-`App` tests, one scenario = one continuous run). New
  scenarios land there on the owner's request only; the file is exempt from the size
  guards and must not be edited without the owner's explicit permission (AGENTS.md
  "Tests" rule 7). Every scenario carries an OWNER-CONFIRMATION marker (owner request
  2026-11-17, the ledger in the suite's header): the automated expectations stay the
  agent's coding of the chat walkthrough until the owner approves each scenario one by
  one — then the marker flips to CONFIRMED <date>; the live approvals queue is V30
  (DEBTS-verification.md).
- Open and New rebuild the layer stack (history restarts). A NEW document's initial
  layer is filled WHITE — the §3.8 white New canvas, restored by the owner request
  2026-11-13 after the layers rewrite had made everything transparent — and every
  layer added later starts fully transparent; Open's initial layer IS the loaded
  picture. Save writes the COMPOSITE of all visible layers as one PNG/JPEG.
- The debug "-" button of the History Tree windows removes the head node (refused on
  the Start root); its children reattach to the grandparent and keep their snapshots.
- The read-only log window is a convenience view, unchanged in spirit; owner request 2026-11-13 renamed it from "History" to "Tool Log" (the View-menu checkbox too) and restyled it: the entry list is capped at 140 pt (half of the former 280 pt), entries render at 0.5rem = 8 pt with zero row spacing (each row is the 8 pt font's own line height — the closest egui gets to a 0.5rem line height), and the first-open position is bottom-center with the bottom edge FLUSH to the main window's bottom edge (gap 0; §3.11 keeps the dropped gap afterwards).

### 3.7 Files: New / Open / Save / Save As

- **New…** — dialog with Width/Height fields (default 800×600; min 1, max 4096; non-numeric input is impossible — `DragValue` or filtered `TextEdit`), OK/Cancel buttons. If `dirty` — confirmation first: "You have unsaved changes. Discard them and continue?" (OK/Cancel). The new canvas is filled white, history is reset, `display_name = "Untitled"`, `current_path = None`. Every dialog button row — New, the unsaved-changes confirmation, Resize Canvas (§3.8), the Delete-layer confirmation, the Error popup, About's Close and the Text tool's Place/Cancel — follows the IBM Carbon / Apple HIG / Google Material convention (owner request 2026-11-13: "like every other app"; the Resize Canvas dialog was the owner's example): the row is right-aligned, the AFFIRMATIVE button is the RIGHTMOST one, the dismissive button sits to its left; Microsoft Windows's opposite order (primary leftmost) is deliberately not followed. Implemented as the shared `app/dialogs.rs::dialog_button_row`. Every MODAL dialog (New, unsaved-changes, Resize Canvas, Delete Layer, the Delete-bottom
"Canvas Size Source" choice, Error, About) is built by the same module's `dialog_window` builder and arises CENTERED on the main window on EVERY open (owner request 2026-11-15: dialogs "appear centered on the main window", not at egui's cascade position slightly right/below of the floating windows): `pivot(CENTER_CENTER)` + `default_pos(content_rect().center())` place the dialog's center exactly, and every close path calls `forget_dialog_position`, which wipes the stored area state so the next open consumes `default_pos` again — a dialog the user dragged aside re-centers when reopened (the "dialog epoch" rule). While open the dialog stays draggable like any other window. Dialogs are NOT §3.11 windows: no edge anchoring, no resize repositioning. The dialog's height hugs its content — never egui's default 340×420 outer size (the bounded `dialog_button_row` keeps the measured content from inheriting the whole available rect).
- **Open…** — if `dirty`, confirmation as above. Native: `rfd::FileDialog::new().add_filter("Images", ["png", "jpg", "jpeg"])` + `image::open`. Web: `rfd::AsyncFileDialog` (§3.10, milestone M8) + `image::load_from_memory`. Then: `to_rgba8` → `Pixmap::from_vec`; canvas size = file size; zoom 100%, pan (0,0), history reset, `display_name` = file name, on native `current_path = Some(path)`, `dirty = false`.
- **Save (Ctrl+S) / Save As (Ctrl+Shift+S)** — BOTH open the in-app "Save Image" dialog (owner request 2026-11-19 — the OS save dialog is gone; the format choice lives IN the dialog, not in a filter list). The rows, top to bottom (the order and the grouping are owner request 2026-11-20): the composed-name preview "Saving as: name.ext" sits ABOVE the file name field (the result reads first); the file name field (prefilled from `display_name` without its image extension); — native only — the "Folder" row DIRECTLY under the file name (prefilled from the current path's folder; "Browse…" opens the native folder picker; the row is hidden on web — the browser owns the download location), so "what is it called" and "where does it go" read as one cluster; the format dropdown "PNG image" / "JPEG image" (a two-radio row until 2026-11-20; prefilled from the current path's extension, else the display name's, else PNG — and kept across opens); and after a visible vertical gap the per-format encoding GROUP — one semantic unit, separated from the rows above (and closed before the buttons) by distance: the "Lossless" checkbox row (owner request 2026-11-20 — THE guarantee, and deliberately a per-format DISPLAY, not a toggle: neither format can do otherwise, so a toggle would be a placebo. PNG locks it ON — "(always — PNG is lossless)"; JPEG locks it OFF — "(never — JPEG is always lossy)": the JPEG standard has a rarely-implemented lossless mode, but the pinned `image` encoder cannot write it and mainstream browsers cannot decode it, so JPEG carries no lossless promise; a format with a real lossless/lossy choice would make the row interactive) sits ABOVE the format's encoding knob — for PNG a "Compression" slider 0..=9 (owner request 2026-11-20; default 6, the encoder's balanced deflate level. The stops are the encoder's REAL, DISTINCT settings — `CompressionType::Level`: 0 stores the rows uncompressed, 1..=9 are the exact deflate effort levels — higher = a smaller file, a slower encode. The decoded pixels NEVER change: PNG is lossless at every level; the pinned encoder's pre-slider default, the fdeflate "Fast" effort, has no stop on this scale and lives on only in the MCP/path export), for JPEG the "Quality" slider 1..=100 (default 90 — the value the old hardcoded encoder used) — and under EACH knob a live meaning caption plus a fixed trade-off line (owner request 2026-11-20: the numbers must explain themselves — they were not obvious even to the owner): the PNG caption reads "<level> — stored raw / light packing / balanced packing / tight packing: …" (what THIS level does to the file — the size-vs-save-speed bucket) with the tail "The picture is identical at every level — the slider only trades file size against save speed."; the JPEG caption reads "<quality> — near-perfect picture / high quality / medium quality / low quality: …" (the size-vs-quality bucket of the current number) with the tail "Lower = a smaller file, but the picture loses more detail.". The composed name = the cleaned typed name + the CHOSEN format's extension (`save_dialog.rs::compose_file_name`): path separators and the Windows-forbidden characters are stripped, a typed extension matching the format survives (canonicalized — `.jpeg` → `.jpg`), a conflicting image extension is replaced, a non-image one stays part of the base; an empty name falls back to "Untitled". Save (the rightmost button per this §'s convention) writes `.png` — PNG RGBA, lossless at every Compression level — or `.jpg` — JPEG flattened onto white (§5.1). Native: the folder is the dialog's pick, else the document's own folder, else the folder picker opens right from the Save click (cancelling keeps the dialog open); on success `dirty = false`, `display_name` = the saved file name, `current_path` = the full path; a write error is the "Error" popup and the dialog STAYS OPEN. Web: Save encodes the bytes and hands them STRAIGHT to the browser (owner request 2026-11-20 — rfd's save-fallback window, the "click here to download your file" Ok/Cancel dialog, is gone from the flow): a Blob of the format's MIME type (`SaveFormat::mime_type`) over an object URL, one synthetic click on a detached `<a download>` anchor carrying the composed name, the URL revoked on a grace timer; there is nothing to cancel — the task resolves to the saved image or the "Error" popup, and the browser owns where the file lands (§3.10). The MCP `export_png` tool keeps the extension-decides path export (`file_io::save_image_to`, feature `mcp`).
- Any error — NOT a panic and NOT `unwrap`: popup titled "Error" with text (`error_popup`), work continues.
- **Window title**: format `{"*" if dirty}{display_name} - Russet` (example: `*Untitled - Russet`). Update when `dirty`/`display_name` change via `ctx.send_viewport_cmd(egui::ViewportCommand::Title(...))`; on web this changes the browser tab title.
- Native file I/O is synchronous, on the main thread, without threads or `tokio`.

### 3.8 "Resize Canvas" dialog

`egui::Window` "Resize Canvas" (centered, content-hugging — §3.7): a MODE radio —
"Canvas size" / "Image size" (owner request 2026-11-17) — Width/Height fields
(prefilled with the current values, min 1, max 4096), and the mode's parameter:

- **Canvas size** — the Paint.NET-style 3×3 ANCHOR grid of painted dots: which part of
  the picture stays in place while the rest crops (shrink) or pads (grow, transparent).
  Center uses floor semantics (an odd crop takes the extra pixel from the left/top, an
  odd pad adds it on the right/bottom). Every layer follows the same anchor geometry, so
  pixels keep their positions relative to the anchor corner.
- **Image size** — a "Maintain aspect ratio" checkbox (on by default; the ratio is the
  canvas's at dialog open and the field the user changed drives the other) and a
  resampling radio: "Nearest neighbor" (hard pixels), "Bilinear", "Bicubic" (the default,
  the Paint.NET default quality), "Lanczos3". Backed by the already-pinned `image`
  crate's `imageops` filters applied to the PREMULTIPLIED bytes (linear filtering
  commutes with premultiplication — the alpha stays correct).

OK/Cancel per §3.7 (OK rightmost). OK with the size already matching everywhere is a
silent no-op (no nodes, not dirty). On OK (`App::apply_resize`): every layer adapts
(Canvas: the same anchor crop/pad; Image: the proportional rescale), ONE `resize` node
per layer records the choices (§3.6), the display buffer rebuilds at the new size with a
full texture upload; zoom and pan do not change; pending gestures are dropped. The
bottom layer defines the new canvas size (the §3.6 size model).

Deleting the BOTTOM layer whose successor has a DIFFERENT size opens the "Canvas Size
Source" dialog (§3.7's builder and button row): "Keep canvas size" (the new base
crop/pads to the canvas, top-left — the composite never shifts; the adaptation records
its own resize node) or "Use layer as base" (the canvas takes the layer's size). A
SAME-size successor becomes the base silently. There is no dismiss: the deletion already
happened and the size model needs a decision.

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
| `Shift` (hold) during a Shape (U) drag | proportional shape — square/circle, Paint.NET-style (§3.2; owner request 2026-11-16). Not a consumed hotkey: read live as `i.modifiers.shift` in the preview gesture, so it also works mid-drag |

Implementation: via `ctx.input_mut(|i| i.consume_key(...))` and modifiers; **must** be gated by `!ctx.wants_keyboard_input()` — hotkeys must not fire while focus is in dialog input fields. Do not add other hotkeys. egui matches modifiers logically (extra Shift/Alt are ignored), so test the most specific shortcut first: `Ctrl+Shift+Z` (redo) before `Ctrl+Z` (undo); the redo keys are consumed even when the redo stack is empty, so an exhausted stack is a no-op and never falls through to undo. On web the browser may keep some Ctrl combinations for itself — do not fight it; menu items must always work.

### 3.10 Web build (WASM) — specifics

- Built via **trunk** (§8), page — `web/index.html` (§2.2, config in `Trunk.toml`), canvas `#russet_canvas`.
- **File dialogs — `rfd::AsyncFileDialog` only**: synchronous `rfd::FileDialog` panics under wasm and is forbidden on web.
  - Open: `AsyncFileDialog::new().add_filter("Images", ["png","jpg","jpeg"]).pick_file().await` → `file.read().await` (bytes) → `image::load_from_memory`.
  - Save: the "Save Image" dialog (§3.7 — the folder row is hidden) collects name/format/quality; `AsyncFileDialog::new().set_file_name(&composed_name).save_file().await` → encode by the chosen format → `handle.write(bytes).await` — the browser downloads the file. `current_path` is not used on web.
- The async result is placed into `Arc<Mutex<Option<Result<LoadedImage, String>>>>`, then `ctx.request_repaint()` is called after completion; the App retrieves the result on the next frame. Use `spawn_local` from `wasm_bindgen_futures`.
- Platform cfg branches are allowed only in `file_io.rs`, at call sites in `app.rs`, and in
  `app/document.rs` (the New/Open/Save glue, split off `app.rs` 2026-11-13); the rest of the code
  knows nothing about the platform.
- Exit is not shown in the menu on web; do NOT add beforeunload protection against closing the tab (§6).

### 3.11 Floating-window placement: drop-time edge anchoring and resize repositioning

Applies to all FOUR floating windows — "Tools", "Palette", "Tool Log" and "Layers" (owner request 2026-11-11 extended it from Tools/Palette to the Tool Log/Layers pair; their content is unchanged). The Tool Log window is user-resizable — §3.11 governs only its position, never its size. No collapsing and no resize-handle changes: plain egui windows as before. Window drop shadows are DISABLED app-wide (every `egui::Window`, both themes; `visuals.window_shadow = Shadow::NONE` in `App::new`) by the owner request 2026-11-11 — this was a §6 item before; menu/popup shadows are deliberately untouched. All distances are in logical points; **2rem = 32 pt** (egui 0.37 has no rem API; 2 × the 16 pt default body text size).

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

### 3.12 Launch arguments (native — owner request 2026-11-17)

The native binary parses argv BEFORE the window opens (`app/launch.rs`; both `--flag value` and `--flag=value` forms). `main.rs` owns the `--help` TEXT (printing is entry-point logic; the platform cfg lives there per AGENTS.md "Repo facts"): `--help` / `-h` prints the list to stdout and exits WITHOUT opening a window. Flags:

- `--primary <hex>` / `--secondary <hex>` — the color wells of the Palette window. The SAME junk-tolerant hex syntax as the Palette's hex field (§3.3): every non-hex-digit character is stripped (an optional `0x` prefix dropped first) and the remaining digit count picks the form — `RGB` (3, each digit doubled), `RGBA` (4, doubled), `RGBAA` (5: the shorthand RGB + a full alpha byte), `RRGGBB` (6), `RRGGBBA` (7: the full RGB + a doubled alpha digit), `RRGGBBAA` (8); a missing alpha defaults to 255. Fewer than 3 or more than 8 digits after the strip warns and keeps the default.
- `--tool <name>` — the tool active on the first frame: pencil, brush, eraser, eyedropper, line, shape, text (case-insensitive; `last_tool` is set to match, so the first frame never sees a phantom tool switch).
- `--canvas-size <WxH>` — the STARTUP canvas (e.g. `1024x768`; `X` and `×` separators work): the composite buffer, the initial white layer AND the Resize-dialog prefill all follow it. Clamped to the same 1..=4096 per-side limits the dialogs enforce, clamps reported. The New dialog keeps its NEW_WIDTH/NEW_HEIGHT constants — the arguments set the STARTUP document, not the New-dialog defaults.
- `--mcp` is skipped silently here (the `mcp` feature scans argv itself in `app/mcp.rs`).

NOTHING panics and nothing aborts: an invalid value or an unrecognized argument prints a `russet: ...` warning to stderr and the built-in default stays (a release build on Windows has no console by `windows_subsystem` — the warnings live in the `cargo run` dev lane). The web build takes no arguments: `std::env::args_os` yields nothing on wasm — ONE code path for both targets. Tests: `app/launch/tests.rs` (pure parsing + the real headless `App` build path through `new_headless_with`).

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
**Done:** the tool can be selected from the window too, size changes with the slider; palette LMB/RMB changes colors, "Swap" works, clicking a color well opens the egui picker for that color; windows can be reopened from View; dropping a window within 2rem of a main-window edge anchors that axis at the dropped gap without pulling it flush, and re-dropping it farther away releases it; resizing the main window keeps an anchored window's dropped gap and a free window's relative placement.

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

Layers; selection and moving fragments; text; shapes (line/rectangle/ellipse); fill (flood fill); eyedropper; transforms (rotate/flip/image scale); action history list; saving application settings; recent files; custom window decorations (`set_decorations(false)`); multithreading in application code (rayon/std::thread/tokio); any new dependencies; status bar; custom project format; internationalization and localization (in any form); macOS and Linux builds/CI; beforeunload tab protection on web; PWA/offline mode/service worker; wasm size optimization (wasm-opt, etc.). (THEME/Appearance customization was REMOVED from this list 2026-11-18: the owner ordered the minimal theme switcher — §3.1; deeper appearance customization remains owner-gated work.)

Exception (owner decision, 2026-11-09): `flake.nix` for building and running the app on **Linux** (Wayland/X11, `nix build` / `nix run` / `nix develop`) is part of the repo and must be kept in sync with `Cargo.toml` (deps, binary name, runtime dlopen libs). This does not re-enable anything else from this list.

Exception (owner request, 2026-11-10): the owner explicitly ordered deliberately simple first iterations of the following §6 items, to be reworked later — **Eyedropper** (canvas picking only), **Line/Curve tool**, **Shapes** (rectangle + ellipse), **Text** (font size only), and a **non-clickable action-history log** window. The **Layers placeholder** left this list on 2026-11-13: REAL layers + the per-layer PNG-snapshot history trees are specified in §3.6 and implemented (`layers.rs`); the History Tree windows switch canvases by node-dot click. New hotkeys `I`/`L`/`U`/`T` (§3.9); no new dependencies; no new `#[cfg]` platform branches. Implemented behavior of the remaining provisional features is described in `PROGRESS.md` (the 2026-11-10 entry), which is the source of truth for them until they are formally specified. This does not re-enable anything else from the list.

Exception (owner decision, 2026-11-12): the flat "any new dependencies" item moves from a hard ban to the approval flow in AGENTS.md ("Dependencies — approval flow"): the agent stops and proposes the need, the candidate crates with trade-offs, the no-dependency alternative and a recommendation; the owner decides. Nothing is pre-approved by this exception, and every other §6 item stays out of scope.

Exception (owner request, 2026-11-12): the MCP development feature (owner chat order) ships behind the opt-in `mcp` cargo feature: a JSON-RPC 2.0 stdio server (`src/app/mcp.rs` + children) lets an AI agent drive the running app during development. Inside that feature build ONLY, `serde_json` is an approved optional dependency (the first case of the approval flow above) and one `std::thread` pair pumps stdin/stdout — the main thread stays event-driven and never blocks. The default build compiles none of it: no server code, no threads, no new dependencies.

Exception (owner request, 2026-11-17): the Resize dialog's IMAGE mode — image rescale
with a resampling filter — is ordered by the owner in chat (the message that requested
the whole resize feature); it narrows the "transforms (rotate/flip/image scale)" item to
exactly this dialog and reuses the pinned `image` crate (`imageops` — no new
dependencies). Nothing else on this list is re-enabled.

## 7. Acceptance checklist — run manually at the end of M9

**Windows (native):**
1. A 1200×800 "Russet" window opens; the File/Edit/Canvas/View/Help menus, Tools and Palette windows, and a white 800×600 canvas on a checkerboard are visible. The Tools window sits 0.5rem (8 pt) from the left edge with 40% of the free vertical space above it and 60% below (a bit above center), and the Palette window sits 0.5rem from the left and bottom edges; none of the floating windows has a drop shadow; the main window cannot be resized below 500×400. Tools are stacked in one column and the Tools window does not show Primary/Secondary colors. The gray canvas background reaches the window edges (no margin); the Tools and Palette windows hug their content and have no resize handles; the Palette window is as wide as its swatch grid (not the full screen); in the Palette window the Primary/Secondary wells (no captions) and "Swap" are on top, the seamless gap-0 swatch grid is below, and clicking a well opens the color picker for that well.
2. Pencil: hard 1px line (no antialiasing), LMB — Primary, RMB — Secondary; a diagonal at any angle is a 1 px staircase (the drifting pen's trail — an even staircase, never broken or hole-ridden; a locally wobbled stretch may stack an isolated pixel), never 2 px thick in runs; a single click sets exactly one pixel.
3. Brush: smooth line; sizes 1 and 100 produce correct thickness; `]`/`[` change the size.
4. Eraser: erases to transparency (checkerboard visible), with both buttons.
5. Fast mouse zigzags leave no stroke gaps; drawing beyond the canvas edge does not panic and is clipped.
6. `Ctrl`+wheel zoom toward cursor 10–800% with Paint.NET-style levels (from 100%: 112, 125, 140, 157, 176, 200, 224, 251, 281, 300), applied abruptly (no smoothing); plain wheel pans vertically and `Shift`+wheel pans horizontally a FIXED 15 pt per wheel notch on every platform (F12 variant B interim + F11's ×5, owner 2026-11-16 — check V23); pan with middle button and Space+LMB; Ctrl+0 returns to 100%; Fit to Window fits the canvas; zoom % label visible in the bottom-right of the panel; console logs each zoom change (with `RUST_LOG=info`); at 100%/200%/…/800% on any Windows display scaling the canvas is pixel-exact (each texel an exact whole number of physical pixels), non-integer zoom and < 100% are smoothly interpolated.
7. Undo/redo: a sequence of >20 strokes can be undone at most 20 steps and does not panic on empty history; redo is reset by a new stroke; Ctrl+Y and Ctrl+Shift+Z both redo, and Ctrl+Shift+Z never performs undo (including when there is nothing to redo).
8. New: 300×200 gives a white canvas; values 0 and 5000 are rejected; when dirty, a confirmation is shown.
9. Open: a PNG with alpha is displayed correctly on the checkerboard (colors are not distorted — premultiplied check).
10. Save PNG → the file opens in an external viewer, transparency is intact; Save JPEG → background is white.
11. Resize Canvas: Canvas mode — content is not lost, the new area is transparent, the
3×3 anchor moves the crop/pad to the chosen corner/edge/center, undo restores the old
size (a bottom-layer undo also restores the old canvas size); Image mode —
Nearest/Bilinear/Bicubic/Lanczos3 differ visibly (nearest stays pixel-hard), the aspect
lock links W/H; every layer's History Tree gains ONE `resize` row printing the choices;
deleting a bottom layer followed by a DIFFERENT-size layer asks "Keep canvas size / Use
layer as base" and both branches are correct (a same-size successor switches silently).
12. Hotkeys do not fire while focus is in a dialog input field.
13. Title looks like `*Untitled - Russet` when there are unsaved changes; Save removes the `*`.
14. Floating-window placement (§3.11, all four floating windows Tools/Palette/Tool Log/Layers): dropping any of them within 2rem of a main-window edge anchors that axis at the dropped gap — the window is NOT pulled flush to the edge (gap 0 only if it was dropped flush); when both edges of one axis are within 2rem, it anchors left (x) / bottom (y); re-dropping the window farther than 2rem from all edges clears the anchors; after resizing the main window an anchored window keeps its dropped gap to that edge (both when growing and shrinking), a free window keeps its relative placement (top-left stays top-left, centered stays centered, bottom-right stays bottom-right), and resizing never anchors, unanchors or pulls anything flush.

**Web (browser):**
15. `trunk serve`: the page opens, "Loading Russet…" disappears, the app fills the screen with no page scrolling.
16. Drawing, eraser, `Ctrl`+wheel zoom, plain-wheel vertical pan, `Shift`+wheel horizontal pan, middle/Space pan, undo/redo work in the browser; zoom % label is visible in the bottom-right; console logs each zoom change.
17. Open opens a PNG; Save downloads a file with the correct name; JPEG downloads with a white background.
18. The browser tab title shows `*Untitled - Russet` when dirty.
19. There is no synchronous `rfd::FileDialog` call on web (AsyncFileDialog only) — the app does not panic during file operations.
20. About: Help → "About" opens a small non-resizable window with the name, version, the one-line description ("…written in Rust with egui/eframe." — owner request 2026-11-18), a GitHub repository hyperlink ("GitHub: George-Noise/russet" → `https://github.com/George-Noise/russet`; clicking opens the system browser on native, a new tab on web), and a `Platform:` line — `Windows` on Windows; `Wayland (Linux)` under a Wayland session (or `WINIT_UNIX_BACKEND=wayland`), `X11 (Linux)` under X11 (or `WINIT_UNIX_BACKEND=x11`); `Web (WASM)` in the browser — plus a `Renderer:` line (OpenGL native / WebGL web). Close hides it.
21. Theme: the menu-bar switcher (§3.1) offers Dark / Light / System with System (follow the OS) as the startup default; picking one re-styles the WHOLE UI immediately — the icon atlas re-rasterizes for the new theme — the Tool Log records "Theme: …", and the MCP `set_theme` tool (dark/light/system) drives the same `App::apply_theme` path.

**Build:**
21. `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `cargo test` — all pass; `cargo build --release` succeeds (M9); `trunk build --release` succeeds; `nix build` on Linux succeeds and the wrapped binary runs in a Wayland and an X11 session.

## 8. Build & run

```bash
rustup update                                   # keep rustup current; the repo pins Rust 1.99.1 (rust-toolchain.toml), installed on first use

# --- Daily development (Windows) ---
cargo run                                       # dev build; first run is long (deps once), then seconds
cargo test --profile gates                      # unit tests, one file per module (src/<module>/tests.rs)
cargo clippy --profile gates --all-targets -- -D warnings   # same lints, unoptimized fast lane — AGENTS.md "Repo facts"

# --- Release: MANUALLY, when everything is ready (fat LTO, slow) ---
cargo build --release

# --- Linux via Nix (flake.nix) ---
nix build                # release (fat LTO) -> ./result/bin/russet
nix run                  # build and start the release app (owner decision
                         # 2026-11-19: the default keeps its old meaning —
                         # the web apps are always explicit)
nix build .#russet-dev   # FAST dev build (owner request 2026-11-16): the repo's
                         # `gates` profile (opt-level 0, no LTO) + the unit
                         # tests; the UI is unoptimized — functional checks,
                         # not feel. Plain `nix build` stays release.
nix run .#russet-dev     # run the dev binary
nix develop              # dev shell: cargo/clippy/fmt + the pinned trunk 0.22.1,
                         # GUI libs on LD_LIBRARY_PATH (`cargo run` works here)

# --- Web via Nix (owner request 2026-11-19): the trunk lanes as apps ---
# All three run the repo-pinned trunk 0.22.1 that the flake builds from the
# crates.io source (nixpkgs' trunk is older). They execute OUTSIDE the nix
# sandbox, so trunk self-manages its tools on first use (the wasm-bindgen
# 0.2.130 matching Cargo.lock + its pinned wasm-opt, cached in ~/.cache/trunk).
nix run .#web-serve           # trunk serve — http://127.0.0.1:8080
nix run .#web-build           # trunk build -> ./dist (dev)
nix run .#web-build-release   # trunk build --release -> ./dist

# --- Web (milestone M8), installed once (Windows / Linux without Nix) ---
# Run from the repo root: Trunk.toml points at web/index.html and dist/.
rustup target add wasm32-unknown-unknown
cargo install trunk --locked --version 0.22.1      # trunk is PINNED — see AGENTS.md "Repo facts"
trunk serve                                     # http://127.0.0.1:8080
trunk build --release                           # static files in dist/
```

cargo and trunk build in parallel on all logical cores by default — no `-j` flags are needed and none should be passed.
(For future reference: a Linux build will require the packages `libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libgtk-3-dev` — do NOT install or test this now. Owner decision 2026-11-09: on Linux the app is built and run via `flake.nix`, which needs none of these packages.)

A human-readable copy of the build & run instructions lives in `README.md` (English; build & run only — that file is written for people, not for agents). Keep it in sync with this section and `flake.nix` when commands change.
