#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use eframe::egui::{
    self, AreaState, Color32, Id, Key, LayerId, Modifiers, Order, PointerButton, Pos2, Rect, Sense,
    Vec2,
};

use crate::canvas::Canvas;
use crate::file_io;
#[cfg(target_arch = "wasm32")]
use crate::file_io::{WebOutcome, WebTask};
use crate::history::History;
use crate::palette::{COLORS, COLS, GRID_WIDTH, ROWS, TILE};
use crate::tools::{Tool, ToolState, draw_segment};

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
/// Floating-window magnetism (§3.11): when a window is DROPPED with an
/// edge closer than this distance to a main-window edge, that axis anchors
/// to the edge at the dropped gap — the window is never pulled flush. 2rem;
/// egui 0.37 has no rem API, so this is 2 × the 16 pt default body font
/// size. Anchors are decided only when a drag ends — never on main-window
/// resizes.
const WINDOW_SNAP_DISTANCE: f32 = 32.0;
const TOOL_COLUMNS: usize = 1;
const NEW_WIDTH: u32 = 800;
const NEW_HEIGHT: u32 = 600;
const MIN_CANVAS: u32 = 1;
const MAX_CANVAS: u32 = 4096;

#[derive(Clone, Copy)]
enum PendingAction {
    New,
    Open,
}

/// Placement bookkeeping for one floating window ("Tools"/"Palette").
#[derive(Default)]
struct WindowPlacement {
    /// Rect of the last frame the window was shown.
    rect: Option<Rect>,
    /// True while the window is dragged by its title bar; the falling edge
    /// (drag end) is where §3.11 anchors are decided.
    dragging: bool,
    /// Edge anchors fixed when the last drag ended. An anchored axis keeps
    /// its dropped gap to that main-window edge on resizes; a free axis
    /// keeps its relative placement (gap-ratio formula).
    anchor: WindowAnchor,
}

pub struct App {
    ctx: egui::Context,
    canvas: Canvas,
    canvas_tex: egui::TextureHandle,
    checker_tex: egui::TextureHandle,
    tools: ToolState,
    history: History,
    zoom: f32,
    pan: Vec2,
    is_stroking: bool,
    last_pos: Option<Pos2>,
    stroke_button: Option<PointerButton>,
    tools_window_open: bool,
    palette_window_open: bool,
    tools_placement: WindowPlacement,
    palette_placement: WindowPlacement,
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
    window_title: String,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
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
            history: History::new(),
            zoom: 1.0,
            pan: Vec2::ZERO,
            is_stroking: false,
            last_pos: None,
            stroke_button: None,
            tools_window_open: true,
            palette_window_open: true,
            tools_placement: WindowPlacement::default(),
            palette_placement: WindowPlacement::default(),
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
            window_title: String::new(),
        }
    }

    fn canvas_rect(&self, panel_rect: Rect) -> Rect {
        // Zoom is defined in PHYSICAL screen pixels (100% = 1 texel per
        // physical pixel): compute the rect on the physical pixel grid, snap
        // its top-left to whole physical pixels, feed egui the logical size
        // = physical / ppp.
        let ppp = self.ctx.pixels_per_point();
        let canvas = egui::vec2(self.canvas.width as f32, self.canvas.height as f32);
        let size_phys = canvas * self.zoom;
        let center_phys = panel_rect.center().to_vec2() * ppp;
        let min_phys = (center_phys - size_phys * 0.5 + self.pan * ppp).round();
        Rect::from_min_size((min_phys / ppp).to_pos2(), size_phys / ppp)
    }

    /// Zoom is honest at whole multiples of 100% (1 texel = N physical pixels).
    fn crisp_zoom(&self) -> bool {
        let percent = (self.zoom * 100.0).round();
        percent >= 100.0 && percent % 100.0 == 0.0
    }

    fn canvas_texture_options(&self) -> egui::TextureOptions {
        if self.crisp_zoom() {
            CANVAS_TEXTURE_CRISP
        } else {
            CANVAS_TEXTURE_SMOOTH
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
        }
    }

    fn redo(&mut self) {
        let current = self.canvas.pixmap().clone();
        if let Some(next) = self.history.redo(current) {
            self.canvas.set_pixmap(next);
            self.repaint_full();
            self.dirty = true;
        }
    }

    fn next_zoom(current: f32, zoom_in: bool) -> f32 {
        let percent = current * 100.0;
        let mut candidate = if zoom_in {
            (percent * ZOOM_STEP_FACTOR).round()
        } else {
            (percent / ZOOM_STEP_FACTOR).round()
        };
        let nearest_hundred = (candidate / 100.0).round() * 100.0;
        if nearest_hundred > 0.0
            && (candidate - nearest_hundred).abs() / candidate <= ZOOM_SNAP_RATIO
        {
            candidate = nearest_hundred;
        }
        candidate.clamp(MIN_ZOOM * 100.0, MAX_ZOOM * 100.0) / 100.0
    }

    fn set_zoom(&mut self, new_zoom: f32, anchor: Option<Pos2>, panel_rect: Rect) {
        let new_zoom = new_zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        if (new_zoom - self.zoom).abs() < f32::EPSILON {
            return;
        }
        if let Some(anchor) = anchor {
            let ppp = self.ctx.pixels_per_point();
            let rect = self.canvas_rect(panel_rect);
            let canvas = egui::vec2(self.canvas.width as f32, self.canvas.height as f32);
            let p = (anchor - rect.min) * ppp / self.zoom;
            let new_min = anchor - p * (new_zoom / ppp);
            self.pan = new_min - panel_rect.center() + canvas * (new_zoom / ppp) * 0.5;
        }
        self.zoom = new_zoom;
        // Re-upload once so the new point/bilinear sampling takes effect
        // immediately (glow applies the sampler params on every upload).
        self.repaint_full();
        log::info!("Zoom: {}%", (self.zoom * 100.0).round() as i32);
    }

    fn reset_zoom(&mut self) {
        self.set_zoom(1.0, None, self.last_panel_rect);
        self.pan = Vec2::ZERO;
    }

    fn handle_scroll(&mut self, ui: &egui::Ui, panel_rect: Rect, hovered: bool) {
        if !hovered {
            return;
        }
        let events = ui.input(|i| i.raw.events.clone());
        for event in events {
            let egui::Event::MouseWheel {
                delta, modifiers, ..
            } = event
            else {
                continue;
            };
            if modifiers.ctrl || modifiers.command {
                let scroll = delta.x + delta.y;
                if scroll.abs() < f32::EPSILON {
                    continue;
                }
                let Some(cursor) = ui.input(|i| i.pointer.hover_pos()) else {
                    continue;
                };
                let new_zoom = Self::next_zoom(self.zoom, scroll > 0.0);
                self.set_zoom(new_zoom, Some(cursor), panel_rect);
            } else if modifiers.shift {
                self.pan.x += (delta.x + delta.y) * SCROLL_PAN_SPEED;
            } else {
                self.pan.y += delta.y * SCROLL_PAN_SPEED;
            }
        }
    }

    fn handle_pan(&mut self, ui: &egui::Ui, response: &egui::Response) -> bool {
        let space = ui.input(|i| i.key_down(Key::Space));
        let panning = response.dragged_by(PointerButton::Middle)
            || (space && !self.is_stroking && response.dragged_by(PointerButton::Primary));
        if panning {
            self.pan += response.drag_delta();
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        }
        panning
    }

    fn canvas_pos(&self, rect: Rect, screen: Pos2) -> Pos2 {
        let ppp = self.ctx.pixels_per_point();
        let p = (screen - rect.min) * ppp / self.zoom;
        Pos2::new(p.x, p.y)
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
                self.stroke_button = Some(button);
                self.is_stroking = true;
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

    fn fit_to_window(&mut self) {
        let rect = self.last_panel_rect;
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            return;
        }
        let ppp = self.ctx.pixels_per_point();
        let sx = rect.width() * ppp / self.canvas.width as f32;
        let sy = rect.height() * ppp / self.canvas.height as f32;
        self.set_zoom(sx.min(sy), None, rect);
        self.pan = Vec2::ZERO;
    }

    fn reset_canvas(&mut self, width: u32, height: u32) {
        let width = width.clamp(MIN_CANVAS, MAX_CANVAS);
        let height = height.clamp(MIN_CANVAS, MAX_CANVAS);
        self.canvas = Canvas::new(width, height, Color32::WHITE);
        self.history.clear();
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.current_path = None;
        }
        self.display_name = "Untitled".to_string();
        self.dirty = false;
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
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.current_path = Some(loaded.path);
        }
        self.display_name = loaded.name;
        self.dirty = false;
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

    fn new_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_new_dialog {
            return;
        }
        let mut open = true;
        let mut ok = false;
        let mut cancel = false;
        egui::Window::new("New")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Width");
                    ui.add(
                        egui::DragValue::new(&mut self.new_width)
                            .range(MIN_CANVAS as f32..=MAX_CANVAS as f32)
                            .speed(1.0)
                            .max_decimals(0),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Height");
                    ui.add(
                        egui::DragValue::new(&mut self.new_height)
                            .range(MIN_CANVAS as f32..=MAX_CANVAS as f32)
                            .speed(1.0)
                            .max_decimals(0),
                    );
                });
                ui.horizontal(|ui| {
                    if ui.button("OK").clicked() {
                        ok = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                });
            });
        if ok {
            let width = self.new_width.round() as u32;
            let height = self.new_height.round() as u32;
            self.reset_canvas(width, height);
            self.show_new_dialog = false;
        } else if cancel || !open {
            self.show_new_dialog = false;
        }
    }

    fn resize_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_resize_dialog {
            return;
        }
        let mut open = true;
        let mut ok = false;
        let mut cancel = false;
        egui::Window::new("Resize Canvas")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Width");
                    ui.add(
                        egui::DragValue::new(&mut self.resize_width)
                            .range(MIN_CANVAS as f32..=MAX_CANVAS as f32)
                            .speed(1.0)
                            .max_decimals(0),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Height");
                    ui.add(
                        egui::DragValue::new(&mut self.resize_height)
                            .range(MIN_CANVAS as f32..=MAX_CANVAS as f32)
                            .speed(1.0)
                            .max_decimals(0),
                    );
                });
                ui.horizontal(|ui| {
                    if ui.button("OK").clicked() {
                        ok = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                });
            });
        if ok {
            let width = self.resize_width.round() as u32;
            let height = self.resize_height.round() as u32;
            self.history.push_snapshot(self.canvas.pixmap().clone());
            self.canvas.resize_canvas(width, height);
            self.repaint_full();
            self.dirty = true;
            self.show_resize_dialog = false;
        } else if cancel || !open {
            self.show_resize_dialog = false;
        }
    }

    fn discard_dialog(&mut self, ctx: &egui::Context) {
        let Some(action) = self.pending_action else {
            return;
        };
        let mut confirmed = false;
        let mut cancel = false;
        egui::Window::new("Unsaved Changes")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("You have unsaved changes. Discard them and continue?");
                ui.horizontal(|ui| {
                    if ui.button("OK").clicked() {
                        confirmed = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                });
            });
        if confirmed {
            self.pending_action = None;
            match action {
                PendingAction::New => self.open_new_dialog(),
                PendingAction::Open => self.open_now(ctx),
            }
        } else if cancel {
            self.pending_action = None;
        }
    }

    fn error_dialog(&mut self, ctx: &egui::Context) {
        let Some(message) = self.error_popup.clone() else {
            return;
        };
        let mut close = false;
        egui::Window::new("Error")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(message);
                if ui.button("OK").clicked() {
                    close = true;
                }
            });
        if close {
            self.error_popup = None;
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
                });
            });
        });
    }

    fn tools_window(&mut self, ctx: &egui::Context) {
        if !self.tools_window_open {
            return;
        }
        let tools = [
            (Tool::Pencil, "Pencil (P)"),
            (Tool::Brush, "Brush (B)"),
            (Tool::Eraser, "Eraser (E)"),
        ];
        let mut open = true;
        let shown = egui::Window::new("Tools")
            .id(tools_window_id())
            .open(&mut open)
            .auto_sized() // size follows the content; no resize handles
            .show(ctx, |ui| {
                egui::Grid::new("tools_grid")
                    .num_columns(TOOL_COLUMNS)
                    .spacing([4.0, 2.0])
                    .show(ui, |ui| {
                        for row in tools.chunks(TOOL_COLUMNS) {
                            for (tool, label) in row {
                                if ui
                                    .selectable_label(self.tools.tool == *tool, *label)
                                    .clicked()
                                {
                                    self.tools.tool = *tool;
                                }
                            }
                            ui.end_row();
                        }
                    });

                let is_pencil = self.tools.tool == Tool::Pencil;
                ui.add_enabled(
                    !is_pencil,
                    egui::Slider::new(&mut self.tools.size, 1.0..=100.0).text("Size"),
                );
            });
        if let Some(response) = &shown {
            place_floating_window(
                ctx,
                tools_window_id(),
                response.response.rect,
                ctx.content_rect(),
                self.last_content_rect,
                &mut self.tools_placement,
            );
        }
        if !open {
            self.tools_window_open = false;
        }
    }

    fn palette_window(&mut self, ctx: &egui::Context) {
        if !self.palette_window_open {
            return;
        }
        let mut open = true;
        let shown = egui::Window::new("Palette")
            .id(palette_window_id())
            .open(&mut open)
            .auto_sized() // size follows the content; no resize handles
            .show(ctx, |ui| {
                // Top: the primary/secondary color wells. A click on a well
                // opens egui's color picker for that color (the former
                // "Custom…" button, one per color); Swap sits to the right.
                ui.horizontal(|ui| {
                    color_swell_button(ui, &mut self.tools.primary, 32.0);
                    color_swell_button(ui, &mut self.tools.secondary, 32.0);
                    if ui.button("Swap (X)").clicked() {
                        std::mem::swap(&mut self.tools.primary, &mut self.tools.secondary);
                    }
                });

                // A plain ui.separator() fills all available width; inside an
                // auto-sized window that width feeds back into the window size
                // until the window spans the whole screen. Pin the divider to
                // the swatch-grid width instead (6.0 pt = default height).
                ui.add_sized([GRID_WIDTH, 6.0], egui::Separator::default());

                // Bottom: the seamless preset grid — tiles touch each other
                // (gap 0) and have no outline. LMB sets the primary color,
                // RMB the secondary. Not an egui::Grid: Grid enforces a
                // minimum column width (`interact_size.x`, 40 pt by default),
                // which leaves gaps between the 20 pt tiles. One exact-size
                // block is allocated instead; neighboring tiles share edges,
                // and clicks are resolved manually from the pointer position.
                let grid_size = egui::vec2(GRID_WIDTH, ROWS as f32 * TILE);
                let (grid_rect, grid) = ui.allocate_exact_size(grid_size, Sense::click());
                for row in 0..ROWS {
                    for col in 0..COLS {
                        let tile = Rect::from_min_size(
                            grid_rect.min + egui::vec2(col as f32 * TILE, row as f32 * TILE),
                            egui::vec2(TILE, TILE),
                        );
                        ui.painter()
                            .rect_filled(tile, 0.0, COLORS[row * COLS + col]);
                    }
                }
                if let Some(pos) = grid.interact_pointer_pos() {
                    let dx = pos.x - grid_rect.min.x;
                    let dy = pos.y - grid_rect.min.y;
                    if dx >= 0.0 && dy >= 0.0 {
                        let col = (dx / TILE) as usize;
                        let row = (dy / TILE) as usize;
                        if col < COLS && row < ROWS {
                            let color = COLORS[row * COLS + col];
                            if grid.clicked() {
                                self.tools.primary = color;
                            }
                            if grid.secondary_clicked() {
                                self.tools.secondary = color;
                            }
                        }
                    }
                }
            });
        if let Some(response) = &shown {
            place_floating_window(
                ctx,
                palette_window_id(),
                response.response.rect,
                ctx.content_rect(),
                self.last_content_rect,
                &mut self.palette_placement,
            );
        }
        if !open {
            self.palette_window_open = false;
        }
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
        let panning = self.handle_pan(ui, &response);
        let rect = self.canvas_rect(panel_rect);
        self.handle_stroke(ui, rect, &response, panning);

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

        let zoom_label = format!("{}%", (self.zoom * 100.0).round() as i32);
        painter.text(
            panel_rect.right_bottom() - egui::vec2(8.0, 6.0),
            egui::Align2::RIGHT_BOTTOM,
            zoom_label,
            egui::FontId::proportional(12.0),
            Color32::from_gray(140),
        );

        if response.hovered()
            && matches!(self.tools.tool, Tool::Brush | Tool::Eraser)
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

/// Stable ids for the two floating windows. They are needed to read the
/// windows' area state and title-bar drag responses from the placement
/// logic (§3.11), so they must not depend on the (changing) title text.
fn tools_window_id() -> Id {
    Id::unique("tools_window")
}

fn palette_window_id() -> Id {
    Id::unique("palette_window")
}

// -------------------------------------------------------------------------
// Floating window placement (§3.11): drop-time edge anchoring (the window
// stays where the user dropped it) and per-axis repositioning when the
// main window is resized.
// -------------------------------------------------------------------------

/// Per-axis anchor of a floating window to one main-window edge (§3.11).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum AxisAnchor {
    /// Free axis: keeps its relative placement (gap-ratio formula) on
    /// main-window resizes.
    #[default]
    None,
    /// Anchored to the lower-coordinate edge (left on x, top on y): keeps
    /// the gap the user dropped the window at.
    Min,
    /// Anchored to the higher-coordinate edge (right on x, bottom on y).
    Max,
}

/// Anchors of one floating window, fixed when its last title-bar drag ended.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct WindowAnchor {
    x: AxisAnchor,
    y: AxisAnchor,
}

/// One axis of the drop-time magnetism: the axis anchors to an edge if the
/// window edge was dropped closer than `distance` to it. If BOTH edges
/// qualify at once (window nearly as wide/tall as the main window),
/// `conflict` wins — left on the horizontal axis, bottom on the vertical
/// one (§3.11).
fn anchor_axis(min_gap: f32, max_gap: f32, distance: f32, conflict: AxisAnchor) -> AxisAnchor {
    let min_close = min_gap < distance;
    let max_close = max_gap < distance;
    if min_close && max_close {
        conflict
    } else if min_close {
        AxisAnchor::Min
    } else if max_close {
        AxisAnchor::Max
    } else {
        AxisAnchor::None
    }
}

/// Anchors of a dropped window: per axis, the edge it was dropped closer
/// than `distance` to (left on horizontal ties, bottom on vertical ones).
/// The window itself stays exactly where the user dropped it — the anchor
/// only decides how it follows that edge on later main-window resizes.
fn window_anchor(rect: Rect, area: Rect, distance: f32) -> WindowAnchor {
    WindowAnchor {
        x: anchor_axis(
            rect.min.x - area.min.x,
            area.max.x - rect.max.x,
            distance,
            AxisAnchor::Min, // horizontal conflict → left
        ),
        y: anchor_axis(
            rect.min.y - area.min.y,
            area.max.y - rect.max.y,
            distance,
            AxisAnchor::Max, // vertical conflict → bottom
        ),
    }
}

/// New window LEFT/TOP coordinate along one axis after the available range
/// changed from `old` to `new` (§3.11). An anchored axis slides with its
/// edge, keeping the gap the user dropped the window at (clamped into the
/// new range); a free axis keeps its relative placement: the ratio between
/// the gap to the lower edge (`win.0 - old.0`) and the gap to the higher
/// edge (`old.1 - win.1`) is preserved (0 = flush left/top, 1 = flush
/// right/bottom, 0.5 = centered). Pure repositioning — it never moves a
/// window flush and never changes anchors.
fn reposition_axis(old: (f32, f32), new: (f32, f32), win: (f32, f32), anchor: AxisAnchor) -> f32 {
    let free_new = ((new.1 - new.0) - (win.1 - win.0)).max(0.0);
    let lo = new.0;
    let hi = new.0 + free_new;
    match anchor {
        AxisAnchor::Min => (lo + (win.0 - old.0).max(0.0)).min(hi),
        AxisAnchor::Max => (hi - (old.1 - win.1).max(0.0)).max(lo),
        AxisAnchor::None => {
            let min_gap = (win.0 - old.0).max(0.0);
            let max_gap = (old.1 - win.1).max(0.0);
            let free_old = min_gap + max_gap;
            let ratio = if free_old > 0.0 {
                min_gap / free_old
            } else {
                0.0
            };
            (lo + ratio * (hi - lo)).clamp(lo, hi)
        }
    }
}

/// New left-top of a floating window after the available area changed, so
/// that an axis anchored to an edge keeps the dropped gap, a top-left
/// window stays top-left, a centered window stays centered and a
/// bottom-right window stays bottom-right (§3.11).
fn reposition_window_pos(
    prev_rect: Rect,
    prev_area: Rect,
    new_area: Rect,
    anchor: WindowAnchor,
) -> Pos2 {
    egui::pos2(
        reposition_axis(
            (prev_area.min.x, prev_area.max.x),
            (new_area.min.x, new_area.max.x),
            (prev_rect.min.x, prev_rect.max.x),
            anchor.x,
        ),
        reposition_axis(
            (prev_area.min.y, prev_area.max.y),
            (new_area.min.y, new_area.max.y),
            (prev_rect.min.y, prev_rect.max.y),
            anchor.y,
        ),
    )
}

/// Moves a floating window's underlying area to `left_top`.
///
/// `Window::current_pos` cannot be used here: in title-bar mode
/// `Window::show_dyn` discards it every frame (it restores the pre-begin
/// position, which is how title-bar drags are applied). Rewriting the area
/// state right after `Window::show` persists into the next frame instead.
fn set_window_pos(ctx: &egui::Context, area_id: Id, left_top: Pos2) {
    if let Some(mut state) = AreaState::load(ctx, area_id) {
        state.set_left_top_pos(left_top);
        // Windows live in `Order::Middle` (the `Window`/`Area` default);
        // `Areas::get_mut` is `pub(crate)`, `set_state` is public.
        ctx.memory_mut(|mem| {
            mem.areas_mut()
                .set_state(LayerId::new(Order::Middle, area_id), state)
        });
    }
}

/// Placement logic for one floating window, run right after `Window::show`:
///
/// - while the window is dragged by its title bar it follows the pointer
///   freely — §3.11 magnetism never moves a window during the drag and
///   never pulls it flush to an edge;
/// - when the drag ends, each axis whose edge the window was dropped
///   closer than `WINDOW_SNAP_DISTANCE` to a main-window edge is anchored
///   to that edge AT THE DROPPED GAP — the window stays exactly where the
///   user placed it (left wins over right, bottom over top); dropping
///   outside the 2rem zones clears all anchors;
/// - after a main-window resize — and never while dragging — an anchored
///   axis keeps its dropped gap and a free axis keeps its relative
///   placement via the gap-ratio formula. Resizes never anchor, unanchor
///   or move anything flush.
fn place_floating_window(
    ctx: &egui::Context,
    area_id: Id,
    rect: Rect,
    content: Rect,
    prev_content: Option<Rect>,
    place: &mut WindowPlacement,
) {
    // The title-bar drag response; the same id `window.rs` uses internally.
    let dragging = ctx
        .read_response(area_id.with("__title_click"))
        .is_some_and(|response| response.dragged());

    if dragging {
        place.dragging = true;
    } else if place.dragging {
        // Drag just ended: fix the window where the USER dropped it.
        place.anchor = window_anchor(rect, content, WINDOW_SNAP_DISTANCE);
        place.dragging = false;
    } else if let (Some(prev_rect), Some(prev_content)) = (place.rect, prev_content)
        && prev_content != content
    {
        let left_top = reposition_window_pos(prev_rect, prev_content, content, place.anchor);
        set_window_pos(ctx, area_id, left_top);
    }
    place.rect = Some(rect);
}

/// A primary/secondary color well: keeps the classic swatch look, but a
/// click opens egui's built-in color picker for THIS color (each well edits
/// its own color; there are no captions).
fn color_swell_button(ui: &mut egui::Ui, color: &mut Color32, size: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), Sense::click());
    ui.painter().rect_filled(rect, 2.0, *color);
    ui.painter().rect_stroke(
        rect,
        2.0,
        egui::Stroke::new(1.0, Color32::from_gray(40)),
        egui::StrokeKind::Inside,
    );

    // Same wiring as egui's own color button (color_edit_button_hsva): a
    // click toggles the popup; the HSV state is re-derived from the current
    // color every frame and written back only when the picker changed it.
    let mut hsva = egui::ecolor::Hsva::from(*color);
    let mut button_response = response;
    egui::Popup::menu(&button_response)
        .id(button_response.id.with("color_popup"))
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.spacing_mut().slider_width = 275.0;
            if egui::color_picker::color_picker_hsva_2d(
                ui,
                &mut hsva,
                egui::color_picker::Alpha::Opaque,
            ) {
                *color = Color32::from(hsva);
                button_response.mark_changed();
            }
        });
    button_response
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
        // Both windows must observe the resize before the tracker flips.
        self.last_content_rect = Some(ctx.content_rect());
        self.discard_dialog(&ctx);
        self.new_dialog(&ctx);
        self.resize_dialog(&ctx);
        self.error_dialog(&ctx);
        self.update_title(&ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        App, AxisAnchor, WINDOW_SNAP_DISTANCE, WindowAnchor, anchor_axis, reposition_axis,
        reposition_window_pos, window_anchor,
    };
    use eframe::egui::{Rect, pos2, vec2};

    fn percent(zoom: f32) -> i32 {
        (zoom * 100.0).round() as i32
    }

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(pos2(x, y), vec2(w, h))
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 1e-3,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn zoom_in_sequence_matches_paint_net() {
        let mut zoom = 1.0;
        for expected in [112, 125, 140, 157, 176, 200, 224, 251, 281, 300] {
            zoom = App::next_zoom(zoom, true);
            assert_eq!(percent(zoom), expected);
        }
    }

    #[test]
    fn zoom_out_returns_to_100_and_clamps() {
        assert_eq!(percent(App::next_zoom(1.12, false)), 100);
        assert_eq!(percent(App::next_zoom(8.0, true)), 800);
        assert_eq!(percent(App::next_zoom(0.1, false)), 10);
    }

    #[test]
    fn anchor_axis_covers_both_sides_and_dead_zone() {
        assert_eq!(
            anchor_axis(10.0, 40.0, 32.0, AxisAnchor::Min),
            AxisAnchor::Min
        );
        assert_eq!(
            anchor_axis(40.0, 4.0, 32.0, AxisAnchor::Min),
            AxisAnchor::Max
        );
        assert_eq!(
            anchor_axis(40.0, 40.0, 32.0, AxisAnchor::Min),
            AxisAnchor::None
        );
        // Exactly at the distance: "closer than" is strict, so no anchor.
        assert_eq!(
            anchor_axis(32.0, 40.0, 32.0, AxisAnchor::Min),
            AxisAnchor::None
        );
    }

    #[test]
    fn anchor_axis_conflict_takes_the_requested_side() {
        // Both edges within reach: the horizontal axis must pick LEFT,
        // the vertical one BOTTOM (§3.11).
        assert_eq!(
            anchor_axis(10.0, 20.0, 32.0, AxisAnchor::Min),
            AxisAnchor::Min
        );
        assert_eq!(
            anchor_axis(10.0, 20.0, 32.0, AxisAnchor::Max),
            AxisAnchor::Max
        );
    }

    #[test]
    fn dropped_window_anchors_to_each_close_edge() {
        let area = rect(0.0, 0.0, 800.0, 600.0);
        // The magnet only decides the ANCHOR; the window stays exactly
        // where the user dropped it — never pulled flush (§3.11).
        assert_eq!(
            window_anchor(rect(10.0, 300.0, 100.0, 50.0), area, WINDOW_SNAP_DISTANCE),
            WindowAnchor {
                x: AxisAnchor::Min,
                y: AxisAnchor::None
            }
        );
        assert_eq!(
            window_anchor(rect(690.0, 300.0, 100.0, 50.0), area, WINDOW_SNAP_DISTANCE),
            WindowAnchor {
                x: AxisAnchor::Max,
                y: AxisAnchor::None
            }
        );
        assert_eq!(
            window_anchor(rect(300.0, 8.0, 100.0, 50.0), area, WINDOW_SNAP_DISTANCE),
            WindowAnchor {
                x: AxisAnchor::None,
                y: AxisAnchor::Min
            }
        );
        assert_eq!(
            window_anchor(rect(300.0, 542.0, 100.0, 50.0), area, WINDOW_SNAP_DISTANCE),
            WindowAnchor {
                x: AxisAnchor::None,
                y: AxisAnchor::Max
            }
        );
    }

    #[test]
    fn dropped_window_middle_exact_distance_and_flush() {
        let area = rect(0.0, 0.0, 800.0, 600.0);
        // Mid-screen: no edge within reach.
        assert_eq!(
            window_anchor(rect(300.0, 300.0, 100.0, 50.0), area, WINDOW_SNAP_DISTANCE),
            WindowAnchor::default()
        );
        // Exactly 2rem from the left edge: not "closer than".
        assert_eq!(
            window_anchor(rect(32.0, 300.0, 100.0, 50.0), area, WINDOW_SNAP_DISTANCE).x,
            AxisAnchor::None
        );
        // Dropped flush: gap 0 is a valid dropped gap — anchored left, so
        // the window stays flush through resizes.
        assert_eq!(
            window_anchor(rect(0.0, 300.0, 100.0, 50.0), area, WINDOW_SNAP_DISTANCE).x,
            AxisAnchor::Min
        );
    }

    #[test]
    fn anchor_conflict_prefers_left_and_bottom() {
        // 150 pt window in a 200 pt main window: both horizontal gaps (20/30)
        // and both vertical gaps (20/30) are within reach → left and bottom.
        let area = rect(0.0, 0.0, 200.0, 200.0);
        assert_eq!(
            window_anchor(rect(20.0, 20.0, 150.0, 150.0), area, WINDOW_SNAP_DISTANCE),
            WindowAnchor {
                x: AxisAnchor::Min,
                y: AxisAnchor::Max
            }
        );
    }

    #[test]
    fn reposition_free_axis_keeps_anchors_and_center() {
        let old = (0.0, 1000.0);
        let new = (0.0, 500.0);
        assert_close(
            reposition_axis(old, new, (0.0, 200.0), AxisAnchor::None),
            0.0,
        ); // left-anchored
        assert_close(
            reposition_axis(old, new, (800.0, 1000.0), AxisAnchor::None),
            300.0,
        ); // right-anchored
        assert_close(
            reposition_axis(old, new, (400.0, 600.0), AxisAnchor::None),
            150.0,
        ); // centered
    }

    #[test]
    fn reposition_free_axis_keeps_gap_ratio_when_growing_and_shrinking() {
        let old = (0.0, 1000.0);
        // gaps 240/560 → ratio 0.3
        assert_close(
            reposition_axis(old, (0.0, 2000.0), (240.0, 440.0), AxisAnchor::None),
            540.0,
        );
        assert_close(
            reposition_axis(old, (0.0, 500.0), (240.0, 440.0), AxisAnchor::None),
            90.0,
        );
    }

    #[test]
    fn reposition_free_axis_clamps_window_into_new_range() {
        // Window wider than the new range: clamped flush to the lower edge.
        assert_close(
            reposition_axis(
                (0.0, 1000.0),
                (0.0, 100.0),
                (100.0, 300.0),
                AxisAnchor::None,
            ),
            0.0,
        );
        // Window fills the old range exactly (no gaps): keep the lower edge.
        assert_close(
            reposition_axis((0.0, 200.0), (0.0, 600.0), (0.0, 200.0), AxisAnchor::None),
            0.0,
        );
    }

    #[test]
    fn anchored_axis_keeps_the_dropped_gap_on_resize() {
        let old = (0.0, 1000.0);
        // Dropped with a 30 pt left gap: the gap survives grow and shrink.
        assert_close(
            reposition_axis(old, (0.0, 2000.0), (30.0, 230.0), AxisAnchor::Min),
            30.0,
        );
        assert_close(
            reposition_axis(old, (0.0, 500.0), (30.0, 230.0), AxisAnchor::Min),
            30.0,
        );
        // Dropped with a 30 pt right gap: slides with the right edge.
        assert_close(
            reposition_axis(old, (0.0, 2000.0), (770.0, 970.0), AxisAnchor::Max),
            1770.0,
        );
        assert_close(
            reposition_axis(old, (0.0, 500.0), (770.0, 970.0), AxisAnchor::Max),
            270.0,
        );
    }

    #[test]
    fn anchored_axis_clamps_into_the_new_range() {
        let old = (0.0, 1000.0);
        // Left gap 300 no longer fits (free space 200): the window is kept
        // inside the range instead of overflowing the right edge.
        assert_close(
            reposition_axis(old, (0.0, 400.0), (300.0, 500.0), AxisAnchor::Min),
            200.0,
        );
        // Right gap 300 no longer fits: clamped against the left edge.
        assert_close(
            reposition_axis(old, (0.0, 400.0), (500.0, 700.0), AxisAnchor::Max),
            0.0,
        );
    }

    #[test]
    fn reposition_bottom_right_free_window_stays_bottom_right() {
        let prev_area = rect(0.0, 0.0, 1000.0, 800.0);
        let new_area = rect(0.0, 0.0, 500.0, 600.0);
        let prev = rect(800.0, 650.0, 200.0, 150.0); // flush right-bottom
        assert_eq!(
            reposition_window_pos(prev, prev_area, new_area, WindowAnchor::default()),
            pos2(300.0, 450.0)
        );
    }

    #[test]
    fn reposition_does_not_magnetize_free_windows() {
        // 100 pt off both edges: after a resize the window is only
        // re-proportioned, never pulled flush to an edge.
        let prev_area = rect(0.0, 0.0, 1000.0, 800.0);
        let new_area = rect(0.0, 0.0, 400.0, 300.0);
        let result = reposition_window_pos(
            rect(100.0, 100.0, 200.0, 200.0),
            prev_area,
            new_area,
            WindowAnchor::default(),
        );
        assert_close(result.x, 25.0); // x ratio 100/800 = 0.125, free_new 200
        assert_close(result.y, 16.667); // y ratio 100/600 = 1/6, free_new 100
    }

    #[test]
    fn anchored_window_keeps_dropped_gaps_through_resize() {
        let prev_area = rect(0.0, 0.0, 1000.0, 800.0);
        let new_area = rect(0.0, 0.0, 500.0, 600.0);
        // Dropped 25 pt from the left and 20 pt from the top: BOTH gaps are
        // kept exactly, on any resize.
        let anchor = window_anchor(
            rect(25.0, 20.0, 200.0, 150.0),
            prev_area,
            WINDOW_SNAP_DISTANCE,
        );
        assert_eq!(
            anchor,
            WindowAnchor {
                x: AxisAnchor::Min,
                y: AxisAnchor::Min
            }
        );
        assert_eq!(
            reposition_window_pos(rect(25.0, 20.0, 200.0, 150.0), prev_area, new_area, anchor),
            pos2(25.0, 20.0)
        );
        // Dropped flush right-bottom: stays flush right-bottom.
        let flush = window_anchor(
            rect(800.0, 650.0, 200.0, 150.0),
            prev_area,
            WINDOW_SNAP_DISTANCE,
        );
        assert_eq!(
            flush,
            WindowAnchor {
                x: AxisAnchor::Max,
                y: AxisAnchor::Max
            }
        );
        assert_eq!(
            reposition_window_pos(rect(800.0, 650.0, 200.0, 150.0), prev_area, new_area, flush),
            pos2(300.0, 450.0)
        );
    }
}
