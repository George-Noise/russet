//! `app.rs` is the orchestrator: all shared state (`App`), the frame flow,
//! undo/redo, the canvas widget and the texture sync. Cohesive subsystems
//! live in child modules (they can access `App`'s private fields):
//!   [`camera`]    — §3.4: canvas rect, zoom (Paint.NET-style steps), pan
//!   [`dialogs`]   — §3.7/§3.8: New / Resize Canvas / unsaved-changes / Error
//!   [`document`]  — owner request 2026-11-13: document lifecycle — New/
//!                   Open/Save, the unsaved-changes gate, the window title
//!   [`input`]     — owner request 2026-11-13: hotkeys (§3.9), freehand
//!                   strokes, Eyedropper picking
//!   [`mcp`]       — owner request 2026-11-12, feature `mcp`: the stdio
//!                   MCP server (compiled out without the feature)
//!   [`menu`]      — owner request 2026-11-13: the top menu bar (§3.1)
//!   [`panels`]    — owner request 2026-11-10: the "History" log and the
//!                   "Layers" placeholder windows
//!   [`placement`] — §3.11: floating-window edge anchoring + resize repositioning
//!   [`preview`]   — owner request 2026-11-10: Line/Curve/Shape tools
//!                   (press–drag–release and polyline gestures, overlay
//!                   preview, commit)
//!   [`text_tool`] — owner request 2026-11-10: the Text tool input window
//!   [`tree_window`] — owner request 2026-11-12: the visual "History Tree"
//!                   windows (git-style graph per pseudo-layer, clones,
//!                   anchored scrolling); the model lives in `history_tree`
//!   [`windows`]   — §3.2/§3.3: the floating "Tools" and "Palette" windows
//!
//! Keep new subsystems as child modules instead of growing this file.

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use eframe::egui::{self, Color32, PointerButton, Pos2, Rect, Sense, Vec2};

use crate::canvas::Canvas;
#[cfg(target_arch = "wasm32")]
use crate::file_io::WebTask;
use crate::history::{ActionLog, History};
use crate::history_tree::HistoryTree;
use crate::tools::{Tool, ToolState};

mod camera;
mod dialogs;
mod document;
mod input;
#[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
mod mcp;
mod menu;
mod panels;
mod placement;
mod preview;
mod text_tool;
mod tree_window;
mod windows;

use self::document::{NEW_HEIGHT, NEW_WIDTH, PendingAction};
use self::placement::{TOOLS_FIRST_OPEN_RATIO, WindowPlacement};
use self::preview::PreviewState;
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
/// egui's raw wheel delta (40 pt per notch on native) pans slower than
/// Paint.NET; scaled up so panning feels ~1.5x slower than Paint.NET
/// (owner-tuned).
const SCROLL_PAN_SPEED: f32 = 1.5;
/// WASD pans the viewport continuously (§3.4): screen-space speed in
/// logical points per second, applied while a pan key is held (owner-tuned;
/// crossing a 1080p-height main window takes ~1.8 s).
const KEY_PAN_SPEED: f32 = 600.0;

pub struct App {
    ctx: egui::Context,
    canvas: Canvas,
    canvas_tex: egui::TextureHandle,
    checker_tex: egui::TextureHandle,
    tools: ToolState,
    /// The tool active on the previous frame: switching it cancels any
    /// pending preview gesture or text entry (see `draw_canvas`).
    last_tool: Tool,
    history: History,
    /// Visual mirror of pencil strokes per pseudo-layer (owner request
    /// 2026-11-12) — model in `history_tree`, windows in `app/tree_window`.
    history_tree: HistoryTree,
    /// MCP stdio server (owner request 2026-11-12, feature `mcp`): events
    /// from the reader thread are polled at the top of every frame and
    /// executed against this state. Compiled out without the feature.
    #[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
    mcp: Option<mcp::McpServer>,
    zoom: f32,
    pan: Vec2,
    is_stroking: bool,
    last_pos: Option<Pos2>,
    stroke_button: Option<PointerButton>,
    tools_window_open: bool,
    palette_window_open: bool,
    history_window_open: bool,
    layers_window_open: bool,
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
    /// §3.11 placement state of the four floating windows (owner request
    /// 2026-11-11 added the History/Layers pair); the Tools one also carries
    /// the one-shot §3.2 first-open vertical ratio (40%/60%).
    tools_placement: WindowPlacement,
    palette_placement: WindowPlacement,
    history_placement: WindowPlacement,
    layers_placement: WindowPlacement,
    /// Pending gesture of the Line/Shape preview tools (owner request
    /// 2026-11-10); interaction and commits live in [`preview`].
    preview: PreviewState,
    /// Pending text entry of the Text tool (owner request 2026-11-10);
    /// the input window and commit live in [`text_tool`].
    text_edit: Option<TextEditState>,
    /// Read-only action log behind the "History" window (owner request
    /// 2026-11-10).
    action_log: ActionLog,
    last_panel_rect: Rect,
    /// `ctx.content_rect()` of the previous frame; a change means the main
    /// window was resized and floating windows must be re-proportioned.
    last_content_rect: Option<Rect>,
    #[cfg(not(target_arch = "wasm32"))]
    current_path: Option<PathBuf>,
    #[cfg(target_arch = "wasm32")]
    web_task: Option<WebTask>,
    display_name: String,
    dirty: bool,
    show_new_dialog: bool,
    new_width: f32,
    new_height: f32,
    show_resize_dialog: bool,
    resize_width: f32,
    resize_height: f32,
    pending_action: Option<PendingAction>,
    error_popup: Option<String>,
    show_about: bool,
    window_title: String,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Owner request 2026-11-11: NO drop shadows on the app's windows —
        // a flat look. `window_shadow` covers every `egui::Window` (Tools,
        // Palette, History, Layers, the Text input and all dialogs) in BOTH
        // themes; menu/popup shadows (`visuals.popup_shadow`) stay as egui
        // draws them — popups are not "windows" in the owner's sense.
        cc.egui_ctx.all_styles_mut(|style| {
            style.visuals.window_shadow = egui::Shadow::NONE;
        });

        let canvas = Canvas::new(800, 600, Color32::WHITE);
        let canvas_tex = cc.egui_ctx.load_texture(
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
        let checker_tex = cc
            .egui_ctx
            .load_texture("checker", checker, checker_options);

        Self {
            ctx: cc.egui_ctx.clone(),
            canvas,
            canvas_tex,
            checker_tex,
            tools: ToolState::new(),
            last_tool: Tool::Pencil,
            history: History::new(),
            history_tree: HistoryTree::new(),
            #[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
            mcp: mcp::McpServer::maybe_spawn(cc.egui_ctx.clone()),
            zoom: 1.0,
            pan: Vec2::ZERO,
            is_stroking: false,
            last_pos: None,
            stroke_button: None,
            tools_window_open: true,
            palette_window_open: true,
            history_window_open: true,
            layers_window_open: true,
            tree_window_open: true,
            tree_views: vec![TreeView::main()],
            tree_clones: 0,
            stroke_paint: None,
            tools_placement: WindowPlacement::first_open_at_ratio(TOOLS_FIRST_OPEN_RATIO),
            palette_placement: WindowPlacement::default(),
            history_placement: WindowPlacement::default(),
            layers_placement: WindowPlacement::default(),
            preview: PreviewState::None,
            text_edit: None,
            action_log: ActionLog::new(),
            last_panel_rect: cc.egui_ctx.content_rect(),
            last_content_rect: None,
            #[cfg(not(target_arch = "wasm32"))]
            current_path: None,
            #[cfg(target_arch = "wasm32")]
            web_task: None,
            display_name: "Untitled".to_string(),
            dirty: false,
            show_new_dialog: false,
            new_width: NEW_WIDTH as f32,
            new_height: NEW_HEIGHT as f32,
            show_resize_dialog: false,
            resize_width: NEW_WIDTH as f32,
            resize_height: NEW_HEIGHT as f32,
            pending_action: None,
            error_popup: None,
            show_about: false,
            window_title: String::new(),
        }
    }

    /// Shared action behind the Edit menu ([`menu`]) and the hotkeys
    /// ([`input`]): one snapshot back; mirrors into the visual tree.
    pub(super) fn undo(&mut self) {
        let current = self.canvas.pixmap().clone();
        if let Some(previous) = self.history.undo(current) {
            self.canvas.set_pixmap(previous);
            self.repaint_full();
            self.dirty = true;
            self.action_log.push("Undo");
            // Visual mirror (owner request 2026-11-12): the tree head steps
            // back with every successful real undo. Caveat accepted for the
            // prototype: the real history is one mixed snapshot stack, so a
            // brush/eraser stroke interleaved with pencil strokes shifts it
            // without the tree knowing — pencil-only sessions stay exact.
            self.history_tree.on_undo();
        }
    }

    /// Shared action behind the Edit menu ([`menu`]) and the hotkeys
    /// ([`input`]): one snapshot forward; mirrors into the visual tree.
    pub(super) fn redo(&mut self) {
        let current = self.canvas.pixmap().clone();
        if let Some(next) = self.history.redo(current) {
            self.canvas.set_pixmap(next);
            self.repaint_full();
            self.dirty = true;
            self.action_log.push("Redo");
            self.history_tree.on_redo();
        }
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
        // preview gesture or text entry — they are tool-bound states.
        if self.last_tool != self.tools.tool {
            self.last_tool = self.tools.tool;
            self.cancel_preview();
            self.cancel_text_edit();
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
            Tool::Line | Tool::Shape => self.handle_preview_input(ui, rect, &response, panning),
            Tool::Text => self.handle_text_input(ui, rect, &response, panning),
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
}

/// The eframe entry point. Runs after `App::new`; owns the whole frame.
impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // MCP events first: tool calls land before anything else draws.
        #[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
        self.mcp_poll();
        #[cfg(target_arch = "wasm32")]
        self.poll_file_task();
        self.handle_hotkeys(&ctx);
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
        // Both windows must observe the resize before the tracker flips.
        self.last_content_rect = Some(ctx.content_rect());
        self.discard_dialog(&ctx);
        self.new_dialog(&ctx);
        self.resize_dialog(&ctx);
        self.error_dialog(&ctx);
        self.about_dialog(&ctx);
        self.update_title(&ctx);
    }
}
