//! `app.rs` is the orchestrator: all shared state (`App`), the frame flow,
//! undo/redo, the canvas widget and the texture sync. Cohesive subsystems
//! live in child modules (they can access `App`'s private fields):
//!   [`camera`]    — §3.4: canvas rect, zoom (Paint.NET-style steps), pan
//!   [`dialogs`]   — §3.7/§3.8: New / Resize Canvas / unsaved-changes / Error
//!   [`document`]  — owner request 2026-11-13: document lifecycle — New/
//!                   Open/Save, the unsaved-changes gate, the window title
//!   [`input`]     — owner request 2026-11-13: hotkeys (§3.9), freehand
//!                   strokes, Eyedropper picking
//!   [`launch`]    — owner request 2026-11-17: the native launch arguments
//!                   (--primary/--secondary color wells, --tool,
//!                   --canvas-size, --help); parsed in `app/launch.rs`,
//!                   applied by `build`; the web build takes no arguments
//!   [`layers`]    — owner request 2026-11-13: REAL layers + per-layer PNG
//!                   snapshot history (the model, in `crate::layers`)
//!   [`layers_window`] — owner request 2026-11-13: the "Layers" window
//!                   (select/rename/visibility/reorder/delete-with-confirm)
//!   [`mcp`]       — owner request 2026-11-12, feature `mcp`: the stdio
//!                   MCP server (compiled out without the feature)
//!   [`menu`]      — owner request 2026-11-13: the top menu bar (§3.1)
//!   [`tool_icons`] — owner request 2026-11-18: the Tools window's ICON
//!                   buttons + the Shape panel's four button icons (owner
//!                   request 2026-11-20) — the tiny-skia atlas + the
//!                   Eyedropper's pipette cursor; the icon drawings live in
//!                   the child `tool_icons/draw` (the 2026-11-19 split)
//!   [`tool_log`]  — owner request 2026-11-10: the "Tool Log" window (the
//!                   read-only action log, named "History" until 2026-11-13)
//!                   — both were `app/panels.rs` until the 2026-11-17 split
//!   [`mouse_side`] — owner request 2026-11-23: the Mouse4/Mouse5
//!                   Undo/Redo setting (SPEC §3.9 + §3.14) — the persisted
//!                   Ask/Always/Never mode, the first-press ask modal, the
//!                   press routing; the app's ONE saved setting (via
//!                   `App::save`)
//!   [`placement`] — §3.11: floating-window edge anchoring + resize repositioning
//!   [`preview`]   — owner request 2026-11-10: Line/Curve/Shape tools
//!                   (press–drag–release and polyline gestures, overlay
//!                   preview, commit)
//!   [`save_dialog`] — owner request 2026-11-19: the "Save Image" dialog
//!                   BOTH Save and Save As open — the format radio (PNG/
//!                   JPEG), the file name, the JPEG quality, the native
//!                   folder row; the save glue lives in [`document`]
//!   [`scenarios`] — owner request 2026-11-17: the OWNER-LOCKED whole-app
//!                   scenario suite (tests only — the automated counterpart
//!                   of the owner's manual resize × history walkthrough;
//!                   edit it ONLY with the owner's permission)
//!   [`settings_window`] — owner request 2026-11-23: the "Settings" window
//!                   (SPEC §3.14, opened from the menu bar's Settings
//!                   menu) — the settings list with their descriptions;
//!                   currently ONE dropdown: the Mouse4/Mouse5 choice
//!   [`text_tool`] — owner request 2026-11-10: the Text tool input window
//!   [`tree_window`] — owner request 2026-11-12: the visual "History Tree"
//!                   windows (git-style graph per layer, node-dot click
//!                   switches the canvas, clones, anchored scrolling); the
//!                   model lives in `history_tree`
//!   [`windows`]   — §3.2/§3.3: the floating "Tools" and "Palette" windows
//!                   (the palette half — the color wells, egui's picker
//!                   popup and the hex field — lives in the child
//!                   `windows/palette_window.rs`, the owner-approved
//!                   2026-11-19 file-guard split, pure code motion)
//!
//! Keep new subsystems as child modules instead of growing this file.

// The save dialog's folder state (`save_dir`) exists on BOTH platforms —
// the folder row is hidden at runtime on web (`file_io::FOLDER_PICKING_
// SUPPORTED`), so the shared dialog code compiles unchanged.
use std::path::PathBuf;

use eframe::egui::{self, Color32, PointerButton, Pos2, Rect, Sense, Vec2};

use crate::canvas::Canvas;
use crate::file_io::{FileTask, SaveFormat};
use crate::history::ActionLog;
use crate::history_tree::{ActionKind, NodeId, ResizeAnchor, ResizeFilter, ResizeMode};
use crate::layers::LayerStack;
use crate::tools::{PencilTrail, SmoothStroke, StrokeWet, Tool, ToolState};

mod camera;
mod dialogs;
mod document;
mod input;
pub(crate) mod launch;
mod layers_window;
#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
mod mcp;
// TEMPORARY fill/wand flood benchmark (owner request 2026-11-23): the same
// feature pattern as `mcp` — the DEFAULT build compiles NONE of it, the
// gates never run it, and at runtime it needs the owner's explicit
// permission env (see the module doc for the rule).
#[cfg(feature = "fill-bench")]
pub mod fill_bench;
mod menu;
mod mouse_side;
mod placement;
mod preview;
mod save_dialog;
mod scenarios;
mod selection;
mod settings_window;
mod text_tool;
mod tool_icons;
mod tool_log;
mod tree_window;
mod windows;

use self::document::{MAX_CANVAS, MIN_CANVAS, NEW_HEIGHT, NEW_WIDTH, PendingAction};
use self::launch::LaunchConfig;
use self::placement::{
    AxisAnchor, TOOLS_FIRST_OPEN_RATIO, ToolSettingsPlacement, WindowAnchor, WindowPlacement,
};
use self::preview::PreviewState;
use self::selection::SelectGesture;
use self::text_tool::TextEditState;
use self::tree_window::TreeView;

/// Canvas texture at integer zoom (100%..=800%): the canvas rect is snapped
/// to the physical pixel grid, so each texel covers an exact whole number of
/// physical pixels and point magnification is perfectly crisp (like
/// Paint.NET at 100/200/300%).
const CANVAS_TEXTURE_CRISP: egui::TextureOptions = egui::TextureOptions {
    magnification: egui::TextureFilter::Nearest,
    minification: egui::TextureFilter::Linear,
    wrap_mode: egui::TextureWrapMode::ClampToEdge,
    mipmap_mode: Some(egui::TextureFilter::Linear),
};

/// Canvas texture otherwise (non-integer zoom, and zoomed out below 100%):
/// smooth bilinear/trilinear sampling, no dropped texel rows/columns.
const CANVAS_TEXTURE_SMOOTH: egui::TextureOptions = egui::TextureOptions {
    magnification: egui::TextureFilter::Linear,
    minification: egui::TextureFilter::Linear,
    wrap_mode: egui::TextureWrapMode::ClampToEdge,
    mipmap_mode: Some(egui::TextureFilter::Linear),
};

const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 8.0;
const ZOOM_STEP_FACTOR: f32 = 1.12;
const ZOOM_SNAP_RATIO: f32 = 0.06;
/// One physical wheel notch pans a FIXED 15 pt on EVERY platform (F12
/// variant B, owner pick 2026-11-16, INTERIM — not the final design; see
/// `app/camera.rs::wheel_notches` for the unit → notch conversion). The
/// value folds in F11 (owner: wheel pan ×5, calibrated on Windows — the
/// pre-F12 Windows notch was ±1 line × `SCROLL_PAN_SPEED` 3.0 = ±3 pt, and
/// 3.0 × 5 = 15.0). Plain-wheel vertical and Shift+wheel horizontal alike.
/// The feel check is V23 (DEBTS-verification.md); the interim calibration
/// may be retuned from `--features diagnostics` wheel logs.
const WHEEL_NOTCH_PAN: f32 = 15.0;
/// WASD pans the viewport continuously (§3.4): screen-space speed in
/// logical points per second, applied while a pan key is held (owner-tuned;
/// crossing a 1080p-height main window takes ~1.8 s).
const KEY_PAN_SPEED: f32 = 600.0;

pub struct App {
    ctx: egui::Context,
    /// The COMPOSITE display buffer: the visible layers blended into one
    /// premultiplied pixmap (kept in sync by `composite_dirty` /
    /// `refresh_composite`); feeds the texture, the Eyedropper and Save.
    canvas: Canvas,
    canvas_tex: egui::TextureHandle,
    checker_tex: egui::TextureHandle,
    tools: ToolState,
    /// The tool active on the previous frame: switching it cancels any
    /// pending preview gesture or text entry (see `draw_canvas`).
    last_tool: Tool,
    /// REAL layers with per-layer PNG-snapshot history (owner request
    /// 2026-11-13) — model in `crate::layers`, windows in `app/layers_window.rs`
    /// (Layers) and `app/tree_window.rs` (History Tree).
    layers: LayerStack,
    /// MCP stdio server (owner request 2026-11-12, feature `mcp`): events
    /// from the reader thread are polled at the top of every frame and
    /// executed against this state. Compiled out without the feature.
    #[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
    mcp: Option<mcp::McpServer>,
    zoom: f32,
    pan: Vec2,
    is_stroking: bool,
    /// Smoothed-stroke emission window (owner request 2026-11-19): the
    /// midpoint-quadratic window every coalesced pointer move of the
    /// frame feeds; the frame's pieces rasterize in one batch. Model in
    /// `tools::smooth`, gesture in [`input`].
    stroke_smooth: SmoothStroke,
    /// The pencil's drifting pen (owner request 2026-11-20): the exact
    /// float pen position + the last plotted pixel, carried across pieces
    /// and frames. The carried fractional phase keeps the 1 px staircase
    /// even and identical no matter how the pointer stream is batched per
    /// frame; the dedupe keeps translucent colors single-blended at the
    /// joints. State in `tools::smooth`, gesture in [`input`].
    stroke_trail: PencilTrail,
    stroke_button: Option<PointerButton>,
    /// The rasterized tool-icon atlas ([`tool_icons`]; owner request
    /// 2026-11-18): built once per scale/theme change, otherwise the Tools
    /// window only blits the cached textures and the Eyedropper reuses the
    /// pipette icon as its canvas cursor.
    tool_icons: Option<tool_icons::Atlas>,
    tools_window_open: bool,
    palette_window_open: bool,
    history_window_open: bool,
    layers_window_open: bool,
    /// Pending "delete layer?" confirmation ([`layers_window`]): the index to
    /// remove once the user confirms.
    layer_delete_confirm: Option<usize>,
    /// In-place layer rename ([`layers_window`]): (index, current buffer); the
    /// TextEdit lives in the Layers window rows.
    layer_rename: Option<(usize, String)>,
    /// The `--atlas-test` diagnostic (owner request 2026-11-21): paint a
    /// static 1 px black-and-white speckle ring on the outer edge of every
    /// atlas picture (`tool_icons::paint_debug_border`) — the owner's tool
    /// for measuring the icon's inset inside its picture.
    atlas_test: bool,
    /// Visual "History Tree" windows (owner request 2026-11-12): view 0 is
    /// the main window (shown while this flag is set), the rest are clones.
    tree_window_open: bool,
    tree_views: Vec<TreeView>,
    /// How many clone windows have been created so far (id + cascade base).
    tree_clones: usize,
    /// The ONE stroke color captured at stroke START, recorded as one
    /// visual node when the gesture completes
    /// (`tree_window`/`history_tree`; owner request 2026-11-13: a stroke
    /// has a single color, no primary/secondary pair).
    stroke_paint: Option<Color32>,
    /// The tool frozen at stroke start — one gesture is one node of one
    /// tool. A hotkey switching the tool mid-drag must not re-rasterize the
    /// remaining pieces with the new tool (and the wet layer bakes the
    /// gesture's blend and color in); the release records THIS tool.
    stroke_tool: Option<Tool>,
    /// The brush/eraser gesture's wet layer (owner bug fix 2026-11-23):
    /// the whole stroke's coverage accumulates at full alpha and the
    /// active layer is recomposited once per changed region
    /// ([`tools::StrokeWet`]), so a translucent color cannot double-blend
    /// at the frame joints. `None` for the pencil (the trail's dedupe is
    /// its translucency guard) and outside gestures.
    stroke_wet: Option<StrokeWet>,
    /// §3.11 placement state of the four floating windows (owner request
    /// 2026-11-11 added the Tool Log/Layers pair); the Tools one also carries
    /// the one-shot §3.2 first-open vertical ratio (40%/60%).
    tools_placement: WindowPlacement,
    /// Magnetized tool-settings panel (owner request 2026-11-17): the
    /// panel's last drawn size, feeding the below/above flip and the edge
    /// alignment of its next frame; state and math in [`placement`].
    tool_settings_placement: ToolSettingsPlacement,
    palette_placement: WindowPlacement,
    history_placement: WindowPlacement,
    layers_placement: WindowPlacement,
    /// Pending gesture of the Line/Shape preview tools (owner request
    /// 2026-11-10); interaction and commits live in [`preview`].
    preview: PreviewState,
    /// The selection (owner request 2026-11-20): regions + the derived
    /// pixel truth and boundary loops. The pure domain is
    /// [`crate::selection`], the gestures and the ops in [`selection`].
    /// DOCUMENT state, not history: cleared on New/Open/resize/crop.
    selection: crate::selection::Selection,
    /// The selection mask SNAPSHOT of the freehand gesture in flight (owner
    /// request 2026-11-21: a selection confines every painting tool). Built
    /// once at stroke start — the selection cannot change while the pointer
    /// drags a stroke (the OPS buttons need Select active) — and cleared
    /// with the gesture; `None` = no selection, no clipping.
    stroke_clip: Option<crate::selection::Mask>,
    /// The in-app clipboard (owner request 2026-11-20): the last copied or
    /// cut pixels of the ACTIVE layer, mask-shaped. Not the OS image
    /// clipboard; survives document switches.
    clipboard: Option<tiny_skia::Pixmap>,
    /// Pending gesture of the Select tool (owner request 2026-11-20); the
    /// region drags, the move lift/restore and the ops live in [`selection`].
    select_gesture: SelectGesture,
    /// Pending text entry of the Text tool (owner request 2026-11-10);
    /// the input window and commit live in [`text_tool`].
    text_edit: Option<TextEditState>,
    /// Read-only action log behind the "Tool Log" window (owner request
    /// 2026-11-10; the window was named "History" until 2026-11-13).
    action_log: ActionLog,
    last_panel_rect: Rect,
    /// `ctx.content_rect()` of the previous frame; a change means the main
    /// window was resized and floating windows must be re-proportioned.
    last_content_rect: Option<Rect>,
    #[cfg(not(target_arch = "wasm32"))]
    current_path: Option<PathBuf>,
    /// The ONE in-flight file task (F21, owner pick of variant A
    /// 2026-11-21): the Open picker (both platforms), the Save download
    /// (web) or the native folder pick of the Save dialog. The blocking
    /// rfd call runs on the task's worker thread — the UI loop never
    /// stops; [`document::poll_file_task`] consumes the result per frame.
    /// The slot doubles as the SINGLE-DIALOG GUARD: while `Some`, every
    /// new picker request is rejected (the first chooser keeps the
    /// screen; the Open menu item and the Save dialog's Browse… button
    /// gray while it lasts). `None` = nothing in flight: polling is one
    /// option check, no threads or channels exist.
    file_task: Option<FileTask>,
    /// The Save click that opened the native folder picker continues once
    /// the pick answers with a folder (the pre-F21 one-click save; F21).
    /// Cleared whenever ANY task resolves — the continuation belongs to
    /// its own pick only.
    save_after_folder_pick: bool,
    display_name: String,
    dirty: bool,
    show_new_dialog: bool,
    new_width: f32,
    new_height: f32,
    show_resize_dialog: bool,
    resize_width: f32,
    resize_height: f32,
    /// Resize-dialog choices that outlive the open dialog (owner request
    /// 2026-11-17): the mode (Canvas / Image), the 3×3 anchor of the
    /// Canvas mode, the resampling filter of the Image mode and the Image
    /// mode's aspect-ratio lock. Defaults are (re)set on every open.
    resize_mode: ResizeMode,
    resize_anchor: ResizeAnchor,
    resize_filter: ResizeFilter,
    resize_aspect: bool,
    /// "Save Image" dialog state (owner request 2026-11-19; dialog in
    /// [`save_dialog`], glue in [`document`]): the raw file name (the
    /// extension comes from the chosen format), the format radio — the
    /// PNG/JPEG choice lives IN the dialog — the per-format encoding knob
    /// (the PNG compression 0..=9, the JPEG quality 1..=100; both kept
    /// across opens) and the folder picked with the dialog's Browse…
    /// button (the document's own folder prefills it; the row is HIDDEN on
    /// web — the browser owns the download location — but the field stays
    /// on both platforms so the shared dialog code compiles unchanged).
    show_save_dialog: bool,
    save_file_name: String,
    save_format: SaveFormat,
    save_quality: u8,
    save_png_compression: u8,
    save_dir: Option<PathBuf>,
    /// Pending base choice after deleting the BOTTOM layer whose successor
    /// has a DIFFERENT size (owner request 2026-11-17): the user picks
    /// "keep the canvas size" (adapt the new bottom layer) or "use the
    /// layer as the base" (the canvas takes the layer's size).
    layer_base_choice: bool,
    pending_action: Option<PendingAction>,
    error_popup: Option<String>,
    show_about: bool,
    window_title: String,
    /// The menu-bar theme switcher's choice (owner request 2026-11-18):
    /// Dark, Light or System (the OS preference — the default). The
    /// PREFERENCE is stored (egui's `ThemePreference`); `ctx.theme()`
    /// resolves it to the active `Theme` for the icon atlas.
    /// Applied through [`App::apply_theme`], which is also the MCP
    /// `set_theme` tool's code path.
    theme: egui::ThemePreference,
    /// The Mouse4/Mouse5 Undo/Redo setting (owner request 2026-11-23,
    /// SPEC §3.9 + §3.14): the persisted Ask/Always/Never mode (the
    /// app's ONE saved setting — loaded in `new_with`, written in
    /// `save`) plus the ask modal's open flag. State machine, press
    /// routing and the dialog in [`mouse_side`].
    mouse_side: mouse_side::MouseSideButtons,
    /// The "Settings" window (owner request 2026-11-23, SPEC §3.14):
    /// opened from the menu bar's Settings menu; the window itself and
    /// its one dropdown live in [`settings_window`].
    settings_window_open: bool,
}

impl App {
    /// The eframe entry constructor: the REAL launch arguments of this
    /// process (`app/launch.rs`; empty on wasm — the web build takes no
    /// arguments) applied to a fresh `App`. Both entry points in `main.rs`
    /// go through here. The ONE persisted setting (the Mouse4/Mouse5 mode,
    /// SPEC §3.14) is read from eframe's storage HERE — the only place
    /// the creation context's storage is reachable; a missing or corrupt
    /// value leaves the fresh-session default untouched.
    pub fn new_with(cc: &eframe::CreationContext<'_>, launch: &LaunchConfig) -> Self {
        let mut app = Self::build(cc.egui_ctx.clone(), launch);
        if let Some(storage) = cc.storage {
            app.mouse_side.load(storage);
        }
        app
    }

    /// Test-only constructor of the whole-app scenario suite
    /// ([`scenarios`], owner request 2026-11-17): the REAL build path with
    /// a default egui `Context` — no renderer, no egui passes (a bare
    /// `CreationContext` is not constructible outside eframe: its window
    /// handles are `pub(crate)`). With the `mcp` feature this spawns the
    /// stdio server reader on the test process's piped stdin — harmless
    /// (immediate EOF, nothing joins the thread).
    #[cfg(test)]
    fn new_headless() -> Self {
        Self::build(egui::Context::default(), &LaunchConfig::default())
    }

    /// Same with explicit launch arguments: the launch-arguments tests
    /// ([`launch::tests`]) drive the REAL build path with a non-default
    /// config.
    #[cfg(test)]
    fn new_headless_with(launch: &LaunchConfig) -> Self {
        Self::build(egui::Context::default(), launch)
    }

    /// Every field of a fresh `App`, shared by both entry points and the
    /// test-only constructors above (ONE point of maintenance: a new field
    /// lands here and every constructor stays in sync). The launch
    /// arguments ([`launch`], owner request 2026-11-17) are applied here:
    /// the canvas size constructs the buffers, the color wells and the
    /// startup tool land via `apply_to` after the literal.
    fn build(ctx: egui::Context, launch: &LaunchConfig) -> Self {
        // Owner request 2026-11-11: NO drop shadows on the app's windows —
        // a flat look. `window_shadow` covers every `egui::Window` (Tools,
        // Palette, Tool Log, Layers, the Text input and all dialogs) in BOTH
        // themes; menu/popup shadows (`visuals.popup_shadow`) stay as egui
        // draws them — popups are not "windows" in the owner's sense.
        ctx.all_styles_mut(|style| {
            style.visuals.window_shadow = egui::Shadow::NONE;
        });
        // The theme switcher's default (owner request 2026-11-18): follow the
        // OS preference; the menu-bar button or the MCP tool changes it later.
        ctx.set_theme(egui::ThemePreference::System);

        // The STARTUP canvas (owner request 2026-11-17): the parsed launch
        // arguments (`--canvas-size`, already clamped 1..=4096 at parse
        // time) override the default; the New dialog keeps its
        // NEW_WIDTH/NEW_HEIGHT constants — see `document::open_new_dialog`.
        let (width, height) = launch.canvas.unwrap_or((NEW_WIDTH, NEW_HEIGHT));
        let canvas = Canvas::new(width, height, Color32::TRANSPARENT);
        let canvas_tex = ctx.load_texture(
            "canvas",
            egui::ColorImage::from_rgba_premultiplied(
                [canvas.width as usize, canvas.height as usize],
                canvas.data(),
            ),
            CANVAS_TEXTURE_CRISP, // initial zoom is 100%
        );

        let checker = egui::ColorImage::new(
            [2, 2],
            vec![
                Color32::WHITE,
                Color32::from_rgb(0xCC, 0xCC, 0xCC),
                Color32::from_rgb(0xCC, 0xCC, 0xCC),
                Color32::WHITE,
            ],
        );
        let checker_options = egui::TextureOptions {
            magnification: egui::TextureFilter::Nearest,
            minification: egui::TextureFilter::Nearest,
            wrap_mode: egui::TextureWrapMode::Repeat,
            mipmap_mode: None,
        };
        let checker_tex = ctx.load_texture("checker", checker, checker_options);

        let mut app = Self {
            ctx: ctx.clone(),
            canvas,
            canvas_tex,
            checker_tex,
            tools: ToolState::new(),
            last_tool: Tool::Pencil,
            layers: LayerStack::new(width, height),
            #[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
            mcp: mcp::McpServer::maybe_spawn(ctx.clone()),
            zoom: 1.0,
            pan: Vec2::ZERO,
            is_stroking: false,
            stroke_smooth: SmoothStroke::new(Pos2::ZERO),
            stroke_trail: PencilTrail::default(),
            stroke_button: None,
            tool_icons: None,
            atlas_test: false,
            tools_window_open: true,
            palette_window_open: true,
            history_window_open: true,
            layers_window_open: true,
            layer_delete_confirm: None,
            layer_rename: None,
            tree_window_open: true,
            tree_views: vec![TreeView::main()],
            tree_clones: 0,
            stroke_paint: None,
            stroke_tool: None,
            stroke_wet: None,
            // The first-open gaps are DROPPED gaps (owner bug fix 2026-11-17:
            // a never-dragged window had free axes, so its first-open gaps
            // scaled with the free space and drifted off the magnet
            // boundaries on the first main-window resize). Per-window:
            // Tools — 0.5rem from the LEFT edge (§3.2), vertical stays a
            // free 40/60 ratio; Palette — 0.5rem from LEFT and BOTTOM
            // (§3.3); Tool Log — bottom edge FLUSH (gap 0), horizontal
            // stays centered (free); Layers — 0.5rem from the RIGHT edge,
            // 2.25rem above the bottom edge (the zoom % label).
            tools_placement: WindowPlacement::first_open_at_ratio(TOOLS_FIRST_OPEN_RATIO)
                .with_first_open_anchors(WindowAnchor {
                    x: AxisAnchor::Min,
                    y: AxisAnchor::None,
                }),
            tool_settings_placement: ToolSettingsPlacement::default(),
            palette_placement: WindowPlacement::default().with_first_open_anchors(WindowAnchor {
                x: AxisAnchor::Min,
                y: AxisAnchor::Max,
            }),
            history_placement: WindowPlacement::default().with_first_open_anchors(WindowAnchor {
                x: AxisAnchor::None,
                y: AxisAnchor::Max,
            }),
            layers_placement: WindowPlacement::default().with_first_open_anchors(WindowAnchor {
                x: AxisAnchor::Max,
                y: AxisAnchor::Max,
            }),
            preview: PreviewState::None,
            selection: crate::selection::Selection::default(),
            stroke_clip: None,
            clipboard: None,
            select_gesture: SelectGesture::None,
            text_edit: None,
            action_log: ActionLog::new(),
            last_panel_rect: ctx.content_rect(),
            last_content_rect: None,
            #[cfg(not(target_arch = "wasm32"))]
            current_path: None,
            save_dir: None,
            file_task: None,
            save_after_folder_pick: false,
            display_name: "Untitled".to_string(),
            dirty: false,
            show_new_dialog: false,
            new_width: NEW_WIDTH as f32,
            new_height: NEW_HEIGHT as f32,
            show_resize_dialog: false,
            resize_width: NEW_WIDTH as f32,
            resize_height: NEW_HEIGHT as f32,
            resize_mode: ResizeMode::Canvas,
            resize_anchor: ResizeAnchor::Center,
            resize_filter: ResizeFilter::Bicubic,
            resize_aspect: true,
            show_save_dialog: false,
            save_file_name: "Untitled".to_string(),
            save_format: SaveFormat::Png,
            save_quality: crate::file_io::DEFAULT_JPEG_QUALITY,
            save_png_compression: crate::file_io::DEFAULT_PNG_COMPRESSION,
            layer_base_choice: false,
            pending_action: None,
            error_popup: None,
            show_about: false,
            window_title: String::new(),
            // The switcher's default: follow the OS preference (owner
            // request 2026-11-18 — "also the choice of the system color").
            theme: egui::ThemePreference::System,
            // The side-button setting starts at its default (Ask: the
            // first Mouse4/Mouse5 press opens the modal, `mouse_side`);
            // `new_with` overrides it from eframe's storage.
            mouse_side: mouse_side::MouseSideButtons::default(),
            settings_window_open: false,
        };
        // The parsed launch arguments land AFTER the literal (the config
        // tweaks existing fields; see `launch::LaunchConfig::apply_to`).
        launch.apply_to(&mut app);
        // The canvas pixmap is created TRANSPARENT above, but the fresh
        // stack's initial layer is WHITE (owner request 2026-11-13): blend
        // it into the display buffer once, or the first frame would show
        // the checkerboard through a white layer.
        app.refresh_composite();
        app
    }

    /// Shared action behind the Edit menu ([`menu`]) and the hotkeys
    /// ([`input`]): one node back on the ACTIVE layer's history tree — the
    /// pixels come from that node's PNG snapshot (owner request 2026-11-13:
    /// the per-layer trees ARE the history).
    pub(super) fn undo(&mut self) {
        if self.layers.undo() {
            self.restore_active_layer_state();
            self.dirty = true;
            self.action_log.push("Undo");
        }
    }

    /// Shared action behind the Edit menu ([`menu`]) and the hotkeys
    /// ([`input`]): one node forward on the ACTIVE layer's history tree
    /// (the "last left branch" rule lives in `LayerTree::redo`).
    pub(super) fn redo(&mut self) {
        if self.layers.redo() {
            self.restore_active_layer_state();
            self.dirty = true;
            self.action_log.push("Redo");
        }
    }

    /// The shared action behind the menu-bar theme switcher ([`menu`]) and
    /// the MCP `set_theme` tool (owner request 2026-11-18): applies
    /// immediately — egui re-styles every widget next pass, and the icon
    /// atlas re-rasterizes through its `(pixels_per_point, theme)` key.
    pub(super) fn apply_theme(&mut self, theme: egui::ThemePreference) {
        self.theme = theme;
        self.ctx.set_theme(theme);
        self.action_log.push(match theme {
            egui::ThemePreference::Dark => "Theme: Dark",
            egui::ThemePreference::Light => "Theme: Light",
            egui::ThemePreference::System => "Theme: System",
        });
    }

    /// Node-dot click in a History Tree window: the head moves to the
    /// clicked node and the canvas shows its PNG snapshot. A stroke right
    /// after that forks the tree git-style.
    pub(super) fn jump_to_history_node(&mut self, id: NodeId) {
        if self.layers.jump_to(id) {
            self.restore_active_layer_state();
            self.dirty = true;
            self.action_log.push("History jump");
        }
    }

    /// Decodes the active layer's head snapshot and restores it at its
    /// NATIVE size (owner request 2026-11-17: nodes keep the size at commit
    /// time). A bottom-layer restore also resizes the CANVAS — the bottom
    /// layer owns it — while any other layer just becomes smaller/larger
    /// than the canvas and the composite clips it at (0,0). Called after
    /// every successful undo / redo / jump / debug removal.
    pub(super) fn restore_active_layer_state(&mut self) {
        let Some(head) = self.layers.head() else {
            return;
        };
        let Some(bytes) = self.layers.node(head).map(|node| node.snapshot.clone()) else {
            return;
        };
        match crate::layers::decode_snapshot(&bytes) {
            Ok(pixmap) => {
                self.layers.active_mut().set_pixmap(pixmap);
                // The bottom layer owns the canvas size — a bottom restore
                // may grow or shrink the display buffer (and the texture).
                self.sync_canvas_to_bottom();
                self.refresh_composite();
            }
            Err(error) => self.error_popup = Some(error),
        }
    }

    /// Enforces the size model's invariant: the display buffer matches the
    /// BOTTOM layer's pixmap size. No-op when they already match. The
    /// caller refreshes the composite (this only rebuilds the buffer).
    pub(super) fn sync_canvas_to_bottom(&mut self) {
        let (width, height) = self.layers.bottom_size();
        if (width, height) != (self.canvas.width, self.canvas.height) {
            // The content is overwritten by the caller's composite anyway;
            // `resize_canvas` keeps the Canvas invariants (clamps + size
            // fields) in one place.
            self.canvas.resize_canvas(width, height);
        }
    }

    /// One Resize-dialog commit (owner request 2026-11-17): adapts every
    /// layer (Canvas mode: the same anchor crop/pad; Image mode: the
    /// proportional rescale with the chosen filter), records ONE `resize`
    /// node PER LAYER carrying the full dialog choices, rebuilds the
    /// display buffer at the new size and drops pending gestures whose
    /// canvas coordinates may no longer be meaningful. Zoom and pan do not
    /// change (SPEC §3.8). A request that would change nothing (the size
    /// already matches everywhere) is a silent no-op — no nodes, no dirty.
    pub(super) fn apply_resize(
        &mut self,
        mode: ResizeMode,
        width: u32,
        height: u32,
        anchor: ResizeAnchor,
        filter: ResizeFilter,
    ) {
        let width = width.clamp(MIN_CANVAS, MAX_CANVAS);
        let height = height.clamp(MIN_CANVAS, MAX_CANVAS);
        if width == self.canvas.width
            && height == self.canvas.height
            && self.layers.all_layers_match(width, height)
        {
            return;
        }
        self.layers.resize_all(mode, width, height, anchor, filter);
        self.cancel_preview();
        self.cancel_text_edit();
        // The selection's bounds shift off-canvas (owner request
        // 2026-11-20): the pending Select gesture restores a lifted move,
        // the selection clears — the same reset the crop applies.
        self.cancel_select_gesture();
        self.selection.clear();
        self.canvas.resize_canvas(width, height);
        self.refresh_composite();
        self.dirty = true;
        let detail = match mode {
            ResizeMode::Canvas => anchor.label().to_string(),
            ResizeMode::Image => filter.label().to_string(),
        };
        self.action_log.push(format!(
            "{} resized to {width}×{height} ({detail})",
            mode.label(),
        ));
    }

    /// One completed gesture on the ACTIVE layer: the post-action buffer
    /// becomes the node's PNG snapshot. `color` is the gesture's main
    /// color, `color2` the second one of a two-color gesture (the Shape
    /// fill next to the outline — owner request 2026-11-16; single-color
    /// tools pass `None`). Encoding failures (practically impossible)
    /// surface through the error popup instead of panicking.
    pub(super) fn record_active_action(
        &mut self,
        kind: ActionKind,
        color: Color32,
        color2: Option<Color32>,
    ) {
        if let Err(error) = self.layers.record_action(kind, color, color2) {
            self.error_popup = Some(error);
        }
    }

    /// Re-blends the whole composite and uploads the full texture — after
    /// undo/redo/jumps, layer ops and document resets.
    pub(super) fn refresh_composite(&mut self) {
        let (width, height) = (self.canvas.width, self.canvas.height);
        crate::layers::composite(
            self.layers.stack(),
            self.canvas.pixmap_mut(),
            0,
            0,
            width,
            height,
        );
        self.repaint_full();
    }

    /// Re-blends ONLY the stroke's dirty region into the display buffer —
    /// the per-segment hot path of the freehand tools.
    pub(super) fn composite_dirty(&mut self, x: u32, y: u32, w: u32, h: u32) {
        crate::layers::composite(self.layers.stack(), self.canvas.pixmap_mut(), x, y, w, h);
    }

    /// Dirty-rect texture update, called from [`input`] after each
    /// painted segment and from the document paths in [`document`].
    pub(super) fn sync_texture(&mut self, x: u32, y: u32, w: u32, h: u32) {
        let x0 = x.min(self.canvas.width);
        let y0 = y.min(self.canvas.height);
        let x1 = x.saturating_add(w).min(self.canvas.width);
        let y1 = y.saturating_add(h).min(self.canvas.height);
        if x1 <= x0 || y1 <= y0 {
            return;
        }
        if x0 == 0 && y0 == 0 && x1 == self.canvas.width && y1 == self.canvas.height {
            self.repaint_full();
            return;
        }
        let region_w = x1 - x0;
        let region_h = y1 - y0;
        let bytes = self.canvas.region_bytes(x0, y0, region_w, region_h);
        let image = egui::ColorImage::from_rgba_premultiplied(
            [region_w as usize, region_h as usize],
            &bytes,
        );
        self.canvas_tex.set_partial(
            [x0 as usize, y0 as usize],
            image,
            self.canvas_texture_options(),
        );
    }

    /// Full texture upload after undo/redo and document resets — callers
    /// in this file and in [`document`].
    pub(super) fn repaint_full(&mut self) {
        let image = egui::ColorImage::from_rgba_premultiplied(
            [self.canvas.width as usize, self.canvas.height as usize],
            self.canvas.data(),
        );
        self.canvas_tex.set(image, self.canvas_texture_options());
    }

    fn draw_canvas(&mut self, ui: &mut egui::Ui, panel_rect: Rect) {
        self.last_panel_rect = panel_rect;
        let response = ui.interact(
            panel_rect,
            ui.scope_id().with("canvas_interaction"),
            Sense::click_and_drag(),
        );

        self.handle_scroll(ui, panel_rect, response.hovered());
        // A tool switch (hotkey, Tools window, menu) cancels any pending
        // preview gesture or text entry — they are tool-bound states. The
        // Select gesture restores a lifted move before dropping.
        if self.last_tool != self.tools.tool {
            self.last_tool = self.tools.tool;
            self.cancel_preview();
            self.cancel_text_edit();
            self.cancel_select_gesture();
            // A mid-gesture tool switch abandons the pending visual node.
            self.stroke_paint = None;
        }
        let panning = self.handle_pan(ui, &response);
        let rect = self.canvas_rect(panel_rect);
        match self.tools.tool {
            Tool::Pencil | Tool::Brush | Tool::Eraser => {
                self.handle_stroke(ui, rect, &response, panning);
            }
            Tool::Eyedropper => self.handle_pick(ui, rect, &response, panning),
            Tool::Fill => self.handle_fill_input(ui, rect, &response, panning),
            Tool::Line | Tool::Shape => self.handle_preview_input(ui, rect, &response, panning),
            Tool::Text => self.handle_text_input(ui, rect, &response, panning),
            Tool::Select => self.handle_select_input(ui, rect, &response, panning),
        }

        let painter = ui.painter_at(panel_rect);
        painter.rect_filled(panel_rect, 0.0, Color32::from_rgb(0x40, 0x40, 0x40));

        let screen_size = rect.size();

        let checker_uv = Rect::from_min_max(
            egui::pos2(0.0, 0.0),
            egui::pos2(screen_size.x / 16.0, screen_size.y / 16.0),
        );
        painter.image(self.checker_tex.id(), rect, checker_uv, Color32::WHITE);

        let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        painter.image(self.canvas_tex.id(), rect, uv, Color32::WHITE);

        // Pending-gesture previews (Line/Curve/Shape/Text), on top of the
        // canvas texture; they never touch the Pixmap.
        self.draw_preview_overlay(&painter, rect);
        self.draw_text_preview(&painter, rect);
        // The selection's boundary loops (owner request 2026-11-20) draw
        // under EVERY tool — the selection is document state — plus the
        // pending Select gesture's outline. Overlay shapes only. During a
        // move drag the floating pixels ride a FOREGROUND overlay above
        // everything (owner redesign 2026-11-20: a separate element,
        // visible past the canvas bounds; the canvas is written only at
        // the lift and the release).
        self.draw_selection_overlay(&painter, rect);
        self.draw_move_overlay(rect);

        let zoom_label = format!("{}%", (self.zoom * 100.0).round() as i32);
        painter.text(
            panel_rect.right_bottom() - egui::vec2(8.0, 6.0),
            egui::Align2::RIGHT_BOTTOM,
            zoom_label,
            egui::FontId::proportional(12.0),
            Color32::from_gray(140),
        );

        if response.hovered() {
            let tool = self.tools.tool;
            if matches!(
                tool,
                Tool::Eyedropper | Tool::Line | Tool::Shape | Tool::Text
            ) {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
            }
            // The Select tool's cursor (owner request 2026-11-20): a
            // crosshair over empty canvas, the MOVE arrow over the selection
            // (the press there starts the move drag).
            if tool == Tool::Select
                && let Some(hover) = ui.input(|i| i.pointer.hover_pos())
                && rect.contains(hover)
            {
                let inside = self.selection.contains(self.canvas_pos(rect, hover));
                ui.ctx().set_cursor_icon(if inside {
                    egui::CursorIcon::Move
                } else {
                    egui::CursorIcon::Crosshair
                });
            }
            // The Eyedropper's cursor is the pipette icon itself (owner
            // request 2026-11-18): the bitmap cursor sits where the icon
            // points — its hotspot is the pipette tip. `set_cursor_image`
            // is native-only (web silently falls back to the Crosshair
            // above); egui-winit dedupes by Arc identity, so pushing the
            // same image every frame is cheap.
            if tool == Tool::Eyedropper
                && let Some(image) = self
                    .tool_icons
                    .as_ref()
                    .and_then(|a| a.pipette_cursor.clone())
            {
                ui.ctx().set_cursor_image(Some(image));
            }
            if matches!(tool, Tool::Brush | Tool::Eraser | Tool::Line | Tool::Shape)
                && let Some(cursor) = ui.input(|i| i.pointer.hover_pos())
            {
                let ppp = self.ctx.pixels_per_point();
                let radius = (self.tools.size * self.zoom * 0.5 / ppp).max(0.5);
                painter.circle_stroke(
                    cursor,
                    radius + 1.0,
                    egui::Stroke::new(1.0, Color32::from_black_alpha(180)),
                );
                painter.circle_stroke(
                    cursor,
                    radius,
                    egui::Stroke::new(1.0, Color32::from_white_alpha(220)),
                );
            }
        }
    }

    /// F13 step-0 diagnostics (owner OK 2026-11-16, temporary
    /// `diagnostics` feature): one `[diagnostics]` line per frame whose
    /// content rect changed — the raw material for the Wayland/Web
    /// resize-jitter analysis (non-monotonic configure series). Compiled out
    /// without `--features diagnostics`; removed together with the feature
    /// when F12/F13 close.
    #[cfg(feature = "diagnostics")]
    fn diagnostics_main_resize(&self) {
        let Some(prev) = self.last_content_rect else {
            return;
        };
        let now = self.ctx.content_rect();
        if prev == now {
            return;
        }
        eprintln!(
            "[diagnostics] main resize: content min=({:.0},{:.0}) size=({:.0},{:.0}) -> min=({:.0},{:.0}) size=({:.0},{:.0})",
            prev.min.x,
            prev.min.y,
            prev.width(),
            prev.height(),
            now.min.x,
            now.min.y,
            now.width(),
            now.height(),
        );
    }
}

/// The eframe entry point. Runs after `App::new`; owns the whole frame.
impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // MCP events first: tool calls land before anything else draws.
        #[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
        self.mcp_poll();
        self.poll_file_task();
        self.handle_hotkeys(&ctx);
        self.handle_mouse_side_buttons(&ctx);
        // The icon atlas must exist before the canvas (the Eyedropper's
        // pipette cursor) and the Tools window (the icon buttons) draw. It
        // is scale-keyed only — theme-independent since the v4 pass and
        // kind-independent since the owner request 2026-11-20 (the Shape
        // panel's MODE radios draw the Shape tool's emblem); the
        // `--atlas-test` flag turns on the 1 px speckle edge ring (owner
        // request 2026-11-21).
        tool_icons::ensure(&mut self.tool_icons, &ctx, self.atlas_test);
        self.top_menu(ui);

        // no_frame: zero margin, the gray canvas background reaches the
        // window edges (default panel frame leaves an 8 pt dark border).
        egui::CentralPanel::no_frame().show(ui, |ui| {
            let panel_rect = ui.max_rect();
            self.draw_canvas(ui, panel_rect);
        });

        self.tools_window(&ctx);
        self.palette_window(&ctx);
        self.history_window(&ctx);
        self.layers_window(&ctx);
        self.tree_windows(&ctx);
        self.text_tool_window(&ctx);
        self.settings_window(&ctx);
        // Both windows must observe the resize before the tracker flips.
        #[cfg(feature = "diagnostics")]
        self.diagnostics_main_resize();
        self.last_content_rect = Some(ctx.content_rect());
        self.discard_dialog(&ctx);
        self.new_dialog(&ctx);
        self.resize_dialog(&ctx);
        self.layer_delete_dialog(&ctx);
        self.layer_base_dialog(&ctx);
        self.save_dialog(&ctx);
        self.error_dialog(&ctx);
        self.about_dialog(&ctx);
        self.mouse_side_dialog(&ctx);
        self.update_title(&ctx);
    }

    // --- eframe `persistence` (owner request 2026-11-23) ---
    //
    // The `persistence` feature is compiled in (Cargo.toml). The two
    // AUTOMATIC save channels stay off: `persist_window: false` in main.rs
    // (the OS window's geometry) and `persist_egui_memory() -> false` below
    // (the whole egui Memory — window positions, z-orders, collapse states,
    // the theme preference). The app's OWN hook saves exactly ONE thing —
    // the Mouse4/Mouse5 mode (`mouse_side`, SPEC §3.14): nothing else may
    // dirty the storage (pinned by `app/tests.rs`), and the default (`Ask`)
    // keeps the storage byte-empty — no `app.ron` at all. eframe calls
    // `save` on its 30-s autosave and at exit; there is no request-save
    // API (AGENTS.md quirk).

    /// egui memory (ALL floating-window/dialog positions, z-orders, collapse
    /// states, scroll offsets, the theme preference and the launch-time
    /// input/interaction options) must NOT persist: windows would reopen
    /// where they were closed and every launch would no longer be fresh —
    /// the documented `Window::default_pos` / §3.11 placement behavior
    /// (fresh session) depends on this. `false` keeps eframe from writing
    /// (or loading) the "egui" key of app.ron — on web, the
    /// `egui_memory_ron` localStorage entry.
    fn persist_egui_memory(&self) -> bool {
        false
    }

    /// The app's OWN storage hook (SPEC §3.14): the Mouse4/Mouse5 mode —
    /// the app's ONE persisted setting — written by [`mouse_side`] as a
    /// plain string under its key (`Ask` removes the key, so an undecided
    /// user keeps an empty storage). Everything else stays out: the
    /// automatic channels are off (above) and no other state may land
    /// here without a deliberate settings decision (pinned by
    /// `app/tests.rs::a_fresh_app_persists_nothing`).
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.mouse_side.persist(storage);
    }
}

#[cfg(test)]
mod tests;
