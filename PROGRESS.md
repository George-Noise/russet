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
| M1 Canvas + zoom/pan | done | pan: wheel / Shift+wheel / middle / Space+LMB / WASD (physical keys) |
| M2 Dirty-region texture updates | done | |
| M3 Tools + history | done | |
| M4 Menu + windows + §3.11 placement | done | §3.11 reworked once — see log 2026-11-09 "magnetism rework" |
| M5 Files (native, Windows) | done | |
| M6 Undo/Redo + hotkeys | done | |
| M7 Resize Canvas + New | done | |
| M8 Web (WASM) | code written, never built | `wasm32` branches exist in `main.rs` / `app.rs` / `file_io.rs`; trunk is unavailable in the sandbox — ALL wasm checks are deferred to the owner (Windows). The 2026-11-10 tools/panels add no new `#[cfg]` branches, so wasm compilation is unaffected by them |
| M9 Acceptance + release | pending (owner) | §7 checklist is manual, on the owner's Windows machine; `cargo build --release` is run by the owner |
| Extra: About window + Nix | code written | Help → "About Russet" with Platform/Renderer lines; `flake.nix` for Linux (Wayland/X11) — `nix build` itself must be verified by the owner on a Nix machine |
| Extra: README.md | done | English, build & run only (Windows / NixOS / Web); declared the human-facing doc in AGENTS.md + SPEC §8 |
| Extra: 6 tools/panels (owner 2026-11-10) | code written, tests green | Eyedropper (I), Line/Curve (L), Shape (U), Text (T), History log window, Layers placeholder window — see the log entry below; all are deliberately simple first iterations |

## Warnings — open issues deliberately NOT fixed yet

- 2026-11-10 — `src/tools.rs` is now **626 lines** (was 234): the owner-requested
  Line/Shape/Eyedropper work added `rasterize_polyline`, `rasterize_shape`,
  `pick_color`, `blend_pixel`, `dirty_rect_for` plus 8 tests. Over the 500-line
  soft guard — per AGENTS.md no split was made mid-task. Proposed split (needs
  owner approval): move the shape/polyline rasterizers + tests into a sibling
  `raster.rs` (or fold them into `canvas.rs`), leaving tool enum/state and the
  freehand `draw_segment` in `tools.rs`. `src/app.rs` grew 698 → 848 lines —
  still within the owner-approved orchestrator exception (roughly doubled would
  be ~1400).

- 2026-11-09 — RESOLVED: `src/app.rs` (1634 lines) was split into four child modules
  (`app/camera.rs`, `app/dialogs.rs`, `app/windows.rs`, `app/placement.rs`). `app.rs` itself
  remains ~690 lines BY OWNER'S DECISION — it stays the single orchestrator and may exceed
  the 500-line guard; do not propose splitting it further unless it roughly doubles
  (see the exception in AGENTS.md "Repo facts").

## Log — newest first. Insert a new entry directly under this heading.

### 2026-11-10 — `Features:` Eyedropper, Line/Curve, Shape, Text, History log, Layers placeholder — owner request (6 items)
- **What:** six owner-requested additions, all deliberately simple first
  iterations ("простенько") that the owner will rework later. This work was
  explicitly ordered by the owner, overriding the §6 out-of-scope list
  (recorded as an owner exception in SPEC §6).
  1. **Eyedropper (I)** — pressing on the canvas samples the pixel under
     the pointer into the Primary (LMB) or Secondary (RMB) well and keeps
     sampling while the button is held (Paint.NET-style). Never touches the
     canvas or the history. Pixel demultiplication handled (premultiplied
     `Pixmap` → straight-alpha `Color32`).
  2. **Line/Curve (L)** — one tool, two modes (Tools window selector):
     *Straight*: press–drag–release with a translucent on-canvas preview,
     committed on release; *Curve*: click to add polyline vertices, rubber
     band to the cursor, Enter / double-click / RMB commits, Esc cancels,
     Backspace removes the last vertex. Preview never touches the `Pixmap`
     — a cancelled gesture leaves no history snapshot.
  3. **Shape (U)** — Rectangle / Ellipse selector + "Fill" checkbox in the
     Tools window; drag with preview, committed on release; Esc cancels.
     Outline width = the Size slider; LMB/RMB pick primary/secondary like
     every other tool.
  4. **Text (T)** — a click on the canvas places an anchor and opens a
     small "Text" window (font-size slider 8..=200 + one-line field) next
     to it; the text renders on-canvas live via egui's own layout engine;
     Enter / "Place" commits, Esc / "Cancel" / a new click re-anchors.
     Commit rasterizes the galley through egui's font atlas into the
     `Pixmap` (new `src/text.rs`) — no new dependencies, committed pixels
     match the preview, font size is in canvas pixels (zoom/DPI
     independent).
  5. **History window** — read-only, non-clickable log of document actions
     (strokes per tool, Line, Curve, Rectangle, Ellipse, Text, Undo, Redo,
     New/Opened/Resized), newest first, capped at 200 entries with
     monotonically increasing ids (`ActionLog` in `history.rs`). Reopen
     via View → History. Interactive undo-to-state will replace it later.
  6. **Layers window** — placeholder with exactly one row ("Background")
     showing a live 28×28 canvas thumbnail; nothing is interactive.
     Reopen via View → Layers.
  Behavior details: switching the tool cancels any pending gesture/text
  entry; New/Open/Resize also reset them and restart the log; Space-pan is
  blocked while a Line/Shape drag is in progress (same as freehand
  strokes); a degenerate polyline (single point) draws a round dot;
  perfectly horizontal/vertical polylines are nudged by a subpixel epsilon
  because tiny-skia rejects zero-height/width path bounds (found by the
  new tests). The brush cursor ring now also shows for Line/Shape; the
  four new tools show a crosshair cursor over the canvas.
- **Files:** new `src/text.rs` (galley+atlas text rasterization),
  `src/app/preview.rs` (Line/Curve/Shape gestures + overlay + commits),
  `src/app/text_tool.rs` (Text input window + preview + commit),
  `src/app/panels.rs` (History + Layers windows); modified `src/tools.rs`
  (Tool variants, LineMode/ShapeKind, `rasterize_polyline`,
  `rasterize_shape`, `pick_color`, shared `blend_pixel`), `src/history.rs`
  (`ActionLog`), `src/app.rs` (tool dispatch, hotkeys I/L/U/T, View menu,
  logs, `last_tool` switch handling), `src/app/windows.rs` (7 tool
  buttons, Size slider enablement, Line mode + Shape kind/Fill selectors),
  `src/app/dialogs.rs` (resize log + gesture reset),
  `src/app/placement.rs` (`set_window_pos` now `pub(super)` for the Text
  window reposition), `src/app/camera.rs` (inverse `screen_pos`), and
  `src/main.rs` (`mod text;`). Docs: SPEC §3.9 + §6 exception, AGENTS.md
  module list.
- **Verified:** `cargo fmt --check`, `cargo check`, `cargo clippy
  --all-targets -- -D warnings`, `cargo test` — all green, **41 tests**
  (27 → 41: polyline/shape/pick/text/ActionLog tests; Rust 1.99.1,
  Linux). The wasm32 target is not installed in this sandbox and wasm
  checks stay deferred to the owner (per the 2026-11-09 M8-notes entry);
  the new code adds no `#[cfg]` branches. No release build (owner runs it).
- **Owner must check:** on Windows `cargo run` — (1) Eyedropper: pick with
  LMB/RMB, drag to sample continuously; (2) Line: Straight drag preview,
  then Curve clicks + Enter/Esc/Backspace; (3) Shape: rect/ellipse outline
  and Fill, Esc mid-drag; (4) Text: click → type → size slider → Enter,
  verify committed text matches the preview and stays sharp at 100%
  zoom; (5) History window lists every action, newest on top, and survives
  window open/close via View; (6) Layers window shows one Background row
  with a live thumbnail; (7) undo/redo still work for every new tool;
  (8) hotkeys I/L/U/T do not fire while typing in the Text window.

### 2026-11-10 — `UX+Build:` WASD panning, 0.5rem first-open gaps, flake de-warned — owner request (4 items)
- **What:** three code changes + the tests question answered in chat. (1) **`flake.nix`:** the deprecated `xorg.*` aliases in `runtimeLibs` were replaced with the new-style top-level attrs (`libx11`, `libxcursor`, `libxrandr`, `libxi`, `libxrender`, `libxext`, `libxcb`) — this removes the nixpkgs evaluation warnings; same libraries, same dlopen names, runtime behavior identical. (2) **First-open gaps:** Tools and Palette no longer sit flush — each keeps a 0.5rem (8 pt) gap from the edges: Tools 8 pt from the left edge (vertically centered), Palette 8 pt from the left AND bottom edges. `0.5 * REM` (REM = 16 pt; egui 0.37 has no rem API) lives in `app/placement.rs::FIRST_OPEN_GAP` next to the 2rem `WINDOW_SNAP_DISTANCE` (now `2.0 * REM`), applied to the `Window::default_pos` anchors in `app/windows.rs`; §3.11 treats the offset as an ordinary dropped gap (the free-axis gap-ratio preserves it even before the first drag). (3) **W/A/S/D panning:** a held key pans the viewport continuously — pure `key_pan_dir(w,a,s,d)` in `app/camera.rs` (diagonals normalized), speed `KEY_PAN_SPEED` = 600 pt/s (screen-space, owner-tuned) scaled by `stable_dt.min(0.1)` so a stalled frame cannot jump the canvas. `egui::Key` letters are PHYSICAL keys (egui-winit maps them from winit `PhysicalKey`), so it is layout-independent — ЙЦУКЕН's Ц/Ф/Ы/В included. Gated by `!egui_wants_keyboard_input()` like the hotkeys; while a pan key is held no stroke starts; explicit `request_repaint()` while keys are held so panning is smooth even without OS key repeat. (4) Docs synced: SPEC §3.2, §3.3, §3.4 (new "Pan — W/A/S/D" bullet), §3.9 table (new row), §7.1; AGENTS.md placement bullet.
- **Files:** `flake.nix`; `src/app/windows.rs`, `src/app/placement.rs` (REM / FIRST_OPEN_GAP consts), `src/app/camera.rs` (key pan, 2 new tests), `src/app.rs` (`KEY_PAN_SPEED`); docs: `SPEC.md`, `AGENTS.md`.
- **Verified:** `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` — all green, 27 tests (25 + 2 new WASD direction/diagonal tests; Rust 1.99.1, Linux). Nix is still unavailable in the sandbox, so the flake itself is only desk-checked (attrs cross-checked against the nixpkgs rename: top-level `libx*`/`libxcb` are the non-deprecated forms).
- **Owner must check:** on Windows `cargo run` — Tools/Palette appear with a small 8 pt margin instead of flush; holding W/A/S/D pans the canvas (switch the layout to Russian and confirm Ц/Ф/Ы/В do the same; typing in a dialog field must NOT pan); wheel/middle/Space panning unchanged. On a Nix machine — `nix build` (or `nix flake check`) must now run WITHOUT evaluation warnings.

### 2026-11-09 — `Docs:` README.md added (build & run for humans) — owner request
- **What:** new `README.md` in English, deliberately MINIMAL per the owner's instruction: it contains ONLY how to build and run the app for the three supported targets — Windows (`cargo run`, `cargo build --release`), NixOS/Linux (`nix build` / `nix run` / `nix develop`, flake enablement one-liner, `WINIT_UNIX_BACKEND` override, non-NixOS distros welcome) and Web/WASM (wasm32 target, `trunk serve`, `trunk build --release` → `dist/`, one example of serving it). No feature descriptions, no changelog. The fact that README.md is the HUMAN-facing document (agents read AGENTS.md/SPEC.md instead) is recorded in AGENTS.md ("Repo facts") and SPEC §8, with a keep-in-sync rule when commands change.
- **Files:** new `README.md`; `AGENTS.md` (README audience bullet), `SPEC.md` (§8 pointer + note that Linux builds go through `flake.nix` since 2026-11-09).
- **Verified:** documentation-only change — no code touched; commands cross-checked against SPEC §8, `flake.nix` and `index.html` (trunk entry).
- **Owner must check:** the three command blocks read correctly on Windows PowerShell vs bash; nothing else.

### 2026-11-09 — `Extras:` About window with platform label + flake.nix (Linux) — owner request
- **What:** two owner-requested additions. (1) A **Help → "About Russet"** menu item opens a small non-resizable window (name, version, one-line description, Close button) that states the platform the app runs on: `Wayland (Linux)`, `X11 (Linux)`, `Windows`, `macOS` or `Web (WASM)`, plus the renderer (OpenGL/WebGL via eframe glow). The Linux Wayland-vs-X11 check mirrors winit's own backend selection: `WINIT_UNIX_BACKEND` override first, then `WAYLAND_DISPLAY`/`WAYLAND_SOCKET` → Wayland, else `DISPLAY` → X11. Implemented with plain `cfg!()` macros (one code path compiles for all targets; no new `#[cfg]` attributes outside `main.rs`). (2) **`flake.nix`** builds the app on Linux (`nix build` / `nix run` / `nix develop`, x86_64+aarch64): `rustPlatform.buildRustPackage` against the existing `Cargo.lock`; only `fontconfig` is linked directly (egui system fonts), all other GUI libs (wayland, libxkbcommon, X11, GLX/EGL) are dlopen'd at runtime and provided via `wrapProgram` LD_LIBRARY_PATH, so one binary runs in Wayland and X11 sessions (force with `WINIT_UNIX_BACKEND=x11|wayland`). No new crate dependencies.
- **Files:** `src/main.rs` (`platform_label` / `renderer_label`), `src/app.rs` (`show_about` state, Help menu, dialog call), `src/app/dialogs.rs` (`about_dialog`), new `flake.nix`; docs synced: SPEC §3.1 (Help menu), §6 (owner exception for the flake), §7 (item 1 mentions Help; new item 20 About; Build renumbered to 21), AGENTS.md (platform-label + flake bullets).
- **Verified:** `cargo check` (baseline AND after the change), `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` — all green, 25 tests passed (Rust 1.99.1, Linux). The flake itself was NOT built here — the sandbox has no Nix; see below.
- **Owner must check:** on a Nix machine: `nix build && ./result/bin/russet` in a Wayland session (About must say `Wayland (Linux)`) and in an X11 session / `WINIT_UNIX_BACKEND=x11` (About must say `X11 (Linux)`); on Windows: Help → About Russet shows `Platform: Windows`; web build: `Platform: Web (WASM)`.

### 2026-11-09 — `UX-fix:` main-window min size is a fixed 500×400 (owner decision)
- **What:** the owner clarified the intent — the minimum window size is simply FIXED at
  500×400, not derived from the default (the previous 800×540 "~1.5x" interpretation is
  reverted). Process change recorded per owner's instruction: to save time, heavy checks
  (`cargo test` / `clippy` / `fmt`) are no longer run by default — see AGENTS.md "Commands";
  default verification is `cargo check` only.
- **Files:** `src/main.rs` (min inner size + comment), `SPEC.md` (§2.1 code quote, §7 item 1),
  `AGENTS.md` ("Commands" — new verification policy).
- **Verified:** `cargo check` — green. Tests/clippy/fmt skipped per the owner's
  time-saving instruction (policy above); the change is one constant.
- **Owner must check:** the window resizes down to 500×400 and no further (next `cargo run`).

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
