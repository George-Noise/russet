//! `app.rs` is the orchestrator: all shared state (`App`), the frame flow
//! and everything that wires subsystems together. Cohesive subsystems live
//! in child modules (they can access `App`'s private fields):
//!   [`camera`]    — §3.4: canvas rect, zoom (Paint.NET-style steps), pan
//!   [`dialogs`]   — §3.7/§3.8: New / Resize Canvas / unsaved-changes / Error
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

use eframe::egui::{self, Color32, Key, Modifiers, PointerButton, Pos2, Rect, Sense, Vec2};

use crate::canvas::Canvas;
use crate::file_io;
#[cfg(target_arch = "wasm32")]
use crate::file_io::{WebOutcome, WebTask};
use crate::history::{ActionLog, History};
use crate::history_tree::HistoryTree;
use crate::tools::{Tool, ToolState, draw_segment, pick_color};

mod camera;
mod dialogs;
mod panels;
mod placement;
mod preview;
mod text_tool;
mod tree_window;
mod windows;

use self::tree_window::TreeView;

use self::placement::{TOOLS_FIRST_OPEN_RATIO, WindowPlacement};
use self::preview::PreviewState;
use self::text_tool::TextEditState;

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
const NEW_WIDTH: u32 = 800;
const NEW_HEIGHT: u32 = 600;
const MIN_CANVAS: u32 = 1;
const MAX_CANVAS: u32 = 4096;

#[derive(Clone, Copy)]
enum PendingAction {
    New,
    Open,
}

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

    fn handle_hotkeys(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Num0)) {
            self.reset_zoom();
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::S)) {
            self.save_as(ctx);
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::S)) {
            self.save(ctx);
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::N)) {
            self.request_new();
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::O)) {
            self.request_open(ctx);
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::P)) {
            self.tools.tool = Tool::Pencil;
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::B)) {
            self.tools.tool = Tool::Brush;
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::E)) {
            self.tools.tool = Tool::Eraser;
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::I)) {
            self.tools.tool = Tool::Eyedropper;
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::L)) {
            self.tools.tool = Tool::Line;
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::U)) {
            self.tools.tool = Tool::Shape;
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::T)) {
            self.tools.tool = Tool::Text;
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::OpenBracket)) {
            self.tools.size = (self.tools.size - 1.0).clamp(1.0, 100.0);
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::CloseBracket)) {
            self.tools.size = (self.tools.size + 1.0).clamp(1.0, 100.0);
        }
        // egui's consume_key matches modifiers logically (extra Shift is
        // ignored), so Ctrl+Shift+Z also matches the plain Ctrl+Z binding:
        // redo must be tested and consumed BEFORE undo. The redo keys are
        // consumed unconditionally, so an exhausted redo stack is a no-op
        // instead of falling through to undo.
        let redo_pressed = ctx.input_mut(|i| {
            let shift_z = i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z);
            let y = i.consume_key(Modifiers::COMMAND, Key::Y);
            shift_z || y
        });
        if self.history.can_redo() && redo_pressed {
            self.redo();
        }
        if self.history.can_undo() && ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Z)) {
            self.undo();
        }
    }

    fn undo(&mut self) {
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

    fn redo(&mut self) {
        let current = self.canvas.pixmap().clone();
        if let Some(next) = self.history.redo(current) {
            self.canvas.set_pixmap(next);
            self.repaint_full();
            self.dirty = true;
            self.action_log.push("Redo");
            self.history_tree.on_redo();
        }
    }

    fn stroke_color(&self) -> Color32 {
        match self.tools.tool {
            Tool::Eraser => Color32::WHITE,
            Tool::Pencil | Tool::Brush => {
                if self.stroke_button == Some(PointerButton::Secondary) {
                    self.tools.secondary
                } else {
                    self.tools.primary
                }
            }
            // Unreachable in practice: the Line/Shape/Text tools freeze
            // their color when the gesture starts (see preview.rs /
            // text_tool.rs); the Eyedropper never paints.
            Tool::Eyedropper | Tool::Line | Tool::Shape | Tool::Text => {
                if self.stroke_button == Some(PointerButton::Secondary) {
                    self.tools.secondary
                } else {
                    self.tools.primary
                }
            }
        }
    }

    /// Eyedropper: pressing on the canvas samples the pixel under the
    /// pointer into the primary (LMB) or secondary (RMB) well, and holding
    /// the button down keeps sampling while dragging (Paint.NET-style).
    /// Never touches the canvas or the history.
    fn handle_pick(&mut self, ui: &egui::Ui, rect: Rect, response: &egui::Response, panning: bool) {
        if panning {
            return;
        }
        let (cursor, primary_down, secondary_down, primary_pressed, secondary_pressed) =
            ui.input(|i| {
                (
                    i.pointer.latest_pos(),
                    i.pointer.button_down(PointerButton::Primary),
                    i.pointer.button_down(PointerButton::Secondary),
                    i.pointer.button_pressed(PointerButton::Primary),
                    i.pointer.button_pressed(PointerButton::Secondary),
                )
            });
        let start = (primary_pressed || secondary_pressed) && response.hovered();
        let drag_continue = response.dragged();
        if !start && !drag_continue {
            return;
        }
        let Some(cursor) = cursor else {
            return;
        };
        if !rect.contains(cursor) {
            return;
        }
        let pos = self.canvas_pos(rect, cursor);
        let x = pos.x.floor() as i32;
        let y = pos.y.floor() as i32;
        if x < 0 || y < 0 || x >= self.canvas.width as i32 || y >= self.canvas.height as i32 {
            return;
        }
        let Some(color) = pick_color(self.canvas.pixmap(), x as u32, y as u32) else {
            return;
        };
        if primary_down {
            self.tools.primary = color;
        } else if secondary_down {
            self.tools.secondary = color;
        }
    }

    fn paint_segment(&mut self, from: Pos2, to: Pos2) {
        let color = self.stroke_color();
        let dirty = draw_segment(
            self.canvas.pixmap_mut(),
            self.tools.tool,
            self.tools.size,
            color,
            from,
            to,
        );
        if let Some(dirty) = dirty {
            self.sync_texture(dirty.x, dirty.y, dirty.w, dirty.h);
            self.dirty = true;
        }
    }

    fn handle_stroke(
        &mut self,
        ui: &egui::Ui,
        rect: Rect,
        response: &egui::Response,
        panning: bool,
    ) {
        if panning {
            self.is_stroking = false;
            self.last_pos = None;
            self.stroke_button = None;
            self.stroke_paint = None;
            return;
        }

        let space = ui.input(|i| i.key_down(Key::Space));
        let (cursor, primary_pressed, secondary_pressed, any_released) = ui.input(|i| {
            (
                i.pointer.latest_pos(),
                i.pointer.button_pressed(PointerButton::Primary),
                i.pointer.button_pressed(PointerButton::Secondary),
                i.pointer.any_released(),
            )
        });

        if any_released {
            // One completed gesture = one visual node (the design concept's
            // tldraw-style mark); the prototype records PENCIL strokes only
            // (owner request 2026-11-12). The color was captured at stroke
            // start — it is what the gesture actually painted with.
            if self.is_stroking
                && self.tools.tool == Tool::Pencil
                && let Some(color) = self.stroke_paint.take()
            {
                self.history_tree.record_stroke(color);
            }
            self.stroke_paint = None;
            self.is_stroking = false;
            self.last_pos = None;
            self.stroke_button = None;
        }

        if !self.is_stroking && response.hovered() && !space {
            let button = if primary_pressed {
                Some(PointerButton::Primary)
            } else if secondary_pressed {
                Some(PointerButton::Secondary)
            } else {
                None
            };
            if let (Some(button), Some(cursor)) = (button, cursor) {
                self.history.push_snapshot(self.canvas.pixmap().clone());
                self.action_log.push(match self.tools.tool {
                    Tool::Pencil => "Pencil stroke",
                    Tool::Brush => "Brush stroke",
                    Tool::Eraser => "Eraser stroke",
                    // Guard: the preview tools never reach this path.
                    Tool::Eyedropper | Tool::Line | Tool::Shape | Tool::Text => "Stroke",
                });
                self.stroke_button = Some(button);
                self.is_stroking = true;
                self.stroke_paint = Some(self.stroke_color());
                let pos = self.canvas_pos(rect, cursor);
                self.last_pos = Some(pos);
                self.paint_segment(pos, pos);
                ui.ctx().request_repaint();
            }
        }

        if self.is_stroking
            && let Some(cursor) = cursor
        {
            let pos = self.canvas_pos(rect, cursor);
            if let Some(last) = self.last_pos
                && pos != last
            {
                self.paint_segment(last, pos);
                self.last_pos = Some(pos);
                ui.ctx().request_repaint();
            }
        }
    }

    fn reset_canvas(&mut self, width: u32, height: u32) {
        let width = width.clamp(MIN_CANVAS, MAX_CANVAS);
        let height = height.clamp(MIN_CANVAS, MAX_CANVAS);
        self.canvas = Canvas::new(width, height, Color32::WHITE);
        self.history.clear();
        // New document: the visual trees reset the same way.
        self.history_tree.reset();
        self.reset_tree_views();
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.current_path = None;
        }
        self.display_name = "Untitled".to_string();
        self.dirty = false;
        // A new document: drop pending gestures and restart the log.
        self.cancel_preview();
        self.cancel_text_edit();
        self.action_log.clear();
        self.action_log.push(format!("New canvas {width}×{height}"));
        self.set_zoom(1.0, None, self.last_panel_rect);
        self.pan = Vec2::ZERO;
        self.repaint_full();
    }

    fn open_new_dialog(&mut self) {
        self.new_width = NEW_WIDTH as f32;
        self.new_height = NEW_HEIGHT as f32;
        self.show_new_dialog = true;
    }

    fn request_new(&mut self) {
        if self.dirty {
            self.pending_action = Some(PendingAction::New);
        } else {
            self.open_new_dialog();
        }
    }

    fn request_open(&mut self, ctx: &egui::Context) {
        if self.dirty {
            self.pending_action = Some(PendingAction::Open);
        } else {
            self.open_now(ctx);
        }
    }

    fn apply_loaded(&mut self, loaded: file_io::LoadedImage) {
        self.canvas.set_pixmap(loaded.pixmap);
        self.history.clear();
        // New document: the visual trees reset the same way.
        self.history_tree.reset();
        self.reset_tree_views();
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.current_path = Some(loaded.path);
        }
        self.display_name = loaded.name;
        self.dirty = false;
        // A new document: drop pending gestures and restart the log.
        self.cancel_preview();
        self.cancel_text_edit();
        self.action_log.clear();
        let name = self.display_name.clone();
        self.action_log.push(format!("Opened {name}"));
        self.set_zoom(1.0, None, self.last_panel_rect);
        self.pan = Vec2::ZERO;
        self.repaint_full();
    }

    fn apply_saved(&mut self, saved: file_io::SavedImage) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.current_path = Some(saved.path);
        }
        self.display_name = saved.name;
        self.dirty = false;
    }

    fn open_now(&mut self, ctx: &egui::Context) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = ctx;
            match file_io::open_image() {
                Ok(Some(loaded)) => self.apply_loaded(loaded),
                Ok(None) => {}
                Err(error) => self.error_popup = Some(error),
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.web_task = Some(file_io::open_image(ctx));
        }
    }

    fn save(&mut self, ctx: &egui::Context) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = ctx;
            if let Some(path) = self.current_path.clone() {
                self.save_to(&path);
            } else {
                self.save_as(ctx);
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.save_as(ctx);
        }
    }

    fn save_as(&mut self, ctx: &egui::Context) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = ctx;
            match file_io::save_image_as(self.canvas.pixmap(), &self.display_name) {
                Ok(Some(saved)) => self.apply_saved(saved),
                Ok(None) => {}
                Err(error) => self.error_popup = Some(error),
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.web_task = Some(file_io::save_image_as(
                ctx,
                self.canvas.pixmap().clone(),
                self.display_name.clone(),
            ));
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn save_to(&mut self, path: &std::path::Path) {
        match file_io::save_image_to(self.canvas.pixmap(), path) {
            Ok(()) => self.dirty = false,
            Err(error) => self.error_popup = Some(error),
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn poll_file_task(&mut self) {
        let Some(task) = self.web_task.take() else {
            return;
        };
        match task.poll() {
            Some(Ok(WebOutcome::Opened(Some(loaded)))) => self.apply_loaded(loaded),
            Some(Ok(WebOutcome::Opened(None))) | Some(Ok(WebOutcome::Saved(None))) => {}
            Some(Ok(WebOutcome::Saved(Some(saved)))) => self.apply_saved(saved),
            Some(Err(error)) => self.error_popup = Some(error),
            None => self.web_task = Some(task),
        }
    }

    fn update_title(&mut self, ctx: &egui::Context) {
        let title = format!(
            "{}{} - Russet",
            if self.dirty { "*" } else { "" },
            self.display_name
        );
        if title != self.window_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.window_title = title;
        }
    }

    fn top_menu(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("top_menu").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui
                        .add(egui::Button::new("New…").shortcut_text("Ctrl+N"))
                        .clicked()
                    {
                        self.request_new();
                        ui.close();
                    }
                    if ui
                        .add(egui::Button::new("Open…").shortcut_text("Ctrl+O"))
                        .clicked()
                    {
                        self.request_open(ui.ctx());
                        ui.close();
                    }
                    if ui
                        .add(egui::Button::new("Save").shortcut_text("Ctrl+S"))
                        .clicked()
                    {
                        self.save(ui.ctx());
                        ui.close();
                    }
                    if ui
                        .add(egui::Button::new("Save As…").shortcut_text("Ctrl+Shift+S"))
                        .clicked()
                    {
                        self.save_as(ui.ctx());
                        ui.close();
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        ui.separator();
                        if ui.button("Exit").clicked() {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                });

                ui.menu_button("Edit", |ui| {
                    let can_undo = self.history.can_undo();
                    if ui
                        .add_enabled(can_undo, egui::Button::new("Undo (Ctrl+Z)"))
                        .clicked()
                    {
                        self.undo();
                        ui.close();
                    }
                    let can_redo = self.history.can_redo();
                    if ui
                        .add_enabled(can_redo, egui::Button::new("Redo (Ctrl+Y)"))
                        .clicked()
                    {
                        self.redo();
                        ui.close();
                    }
                });

                ui.menu_button("Canvas", |ui| {
                    if ui.button("Resize Canvas…").clicked() {
                        self.resize_width = self.canvas.width as f32;
                        self.resize_height = self.canvas.height as f32;
                        self.show_resize_dialog = true;
                        ui.close();
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Zoom 100% (Ctrl+0)").clicked() {
                        self.reset_zoom();
                        ui.close();
                    }
                    if ui.button("Fit to Window").clicked() {
                        self.fit_to_window();
                        ui.close();
                    }
                    ui.separator();
                    ui.checkbox(&mut self.tools_window_open, "Tools");
                    ui.checkbox(&mut self.palette_window_open, "Palette");
                    ui.checkbox(&mut self.tree_window_open, "History Tree");
                    ui.checkbox(&mut self.history_window_open, "History");
                    ui.checkbox(&mut self.layers_window_open, "Layers");
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About Russet").clicked() {
                        self.show_about = true;
                        ui.close();
                    }
                });
            });
        });
    }

    fn sync_texture(&mut self, x: u32, y: u32, w: u32, h: u32) {
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

    fn repaint_full(&mut self) {
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
