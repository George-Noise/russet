//! `app.rs` is the orchestrator: all shared state (`App`), the frame flow
//! and everything that wires subsystems together. Cohesive subsystems live
//! in child modules (they can access `App`'s private fields):
//!   [`camera`]    — §3.4: canvas rect, zoom (Paint.NET-style steps), pan
//!   [`dialogs`]   — §3.7/§3.8: New / Resize Canvas / unsaved-changes / Error
//!   [`placement`] — §3.11: floating-window edge anchoring + resize repositioning
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
use crate::history::History;
use crate::tools::{Tool, ToolState, draw_segment};

mod camera;
mod dialogs;
mod placement;
mod windows;

use self::placement::WindowPlacement;

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
    show_about: bool,
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
        self.about_dialog(&ctx);
        self.update_title(&ctx);
    }
}
