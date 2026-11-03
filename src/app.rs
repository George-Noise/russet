#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use eframe::egui::{self, Color32, Key, Modifiers, PointerButton, Pos2, Rect, Sense, Vec2};

use crate::canvas::Canvas;
use crate::file_io;
#[cfg(target_arch = "wasm32")]
use crate::file_io::{WebOutcome, WebTask};
use crate::history::History;
use crate::palette::COLORS;
use crate::tools::{Tool, ToolState, draw_segment};

const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 8.0;
const ZOOM_STEP: f32 = 1.25;
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
    last_panel_rect: Rect,
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
            egui::TextureOptions::NEAREST,
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
            last_panel_rect: cc.egui_ctx.content_rect(),
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
        let size = egui::vec2(self.canvas.width as f32, self.canvas.height as f32) * self.zoom;
        Rect::from_center_size(panel_rect.center() + self.pan, size)
    }

    fn handle_hotkeys(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Num0)) {
            self.zoom = 1.0;
            self.pan = Vec2::ZERO;
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
        if self.history.can_undo() && ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Z)) {
            self.undo();
        }
        if self.history.can_redo()
            && (ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Y))
                || ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z)))
        {
            self.redo();
        }
    }

    fn undo(&mut self) {
        let current = self.canvas.pixmap().clone();
        if let Some(previous) = self.history.undo(current) {
            self.canvas.set_pixmap(previous);
            self.repaint_full();
        }
    }

    fn redo(&mut self) {
        let current = self.canvas.pixmap().clone();
        if let Some(next) = self.history.redo(current) {
            self.canvas.set_pixmap(next);
            self.repaint_full();
        }
    }

    fn handle_zoom(&mut self, ui: &egui::Ui, panel_rect: Rect, rect: Rect, hovered: bool) {
        if !hovered {
            return;
        }
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll.abs() < f32::EPSILON {
            return;
        }
        let Some(cursor) = ui.input(|i| i.pointer.hover_pos()) else {
            return;
        };

        let factor = if scroll > 0.0 {
            ZOOM_STEP
        } else {
            1.0 / ZOOM_STEP
        };
        let new_zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        if (new_zoom - self.zoom).abs() < f32::EPSILON {
            return;
        }

        let canvas_size = egui::vec2(self.canvas.width as f32, self.canvas.height as f32);
        let p = (cursor - rect.min) / self.zoom;
        let new_min = cursor - p * new_zoom;
        self.pan = new_min - panel_rect.center() + canvas_size * new_zoom * 0.5;
        self.zoom = new_zoom;
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
        let p = (screen - rect.min) / self.zoom;
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
        let sx = rect.width() / self.canvas.width as f32;
        let sy = rect.height() / self.canvas.height as f32;
        self.zoom = sx.min(sy).clamp(MIN_ZOOM, MAX_ZOOM);
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
        self.zoom = 1.0;
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
        self.zoom = 1.0;
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
                        self.zoom = 1.0;
                        self.pan = Vec2::ZERO;
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
        let mut open = true;
        egui::Window::new("Tools").open(&mut open).show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(self.tools.tool == Tool::Pencil, "Pencil (P)")
                    .clicked()
                {
                    self.tools.tool = Tool::Pencil;
                }
                if ui
                    .selectable_label(self.tools.tool == Tool::Brush, "Brush (B)")
                    .clicked()
                {
                    self.tools.tool = Tool::Brush;
                }
                if ui
                    .selectable_label(self.tools.tool == Tool::Eraser, "Eraser (E)")
                    .clicked()
                {
                    self.tools.tool = Tool::Eraser;
                }
            });

            let is_pencil = self.tools.tool == Tool::Pencil;
            ui.add_enabled(
                !is_pencil,
                egui::Slider::new(&mut self.tools.size, 1.0..=100.0).text("Size"),
            );

            ui.separator();
            ui.horizontal(|ui| {
                color_swatch(ui, self.tools.primary, 24.0);
                ui.label("Primary");
                color_swatch(ui, self.tools.secondary, 24.0);
                ui.label("Secondary");
            });
        });
        if !open {
            self.tools_window_open = false;
        }
    }

    fn palette_window(&mut self, ctx: &egui::Context) {
        if !self.palette_window_open {
            return;
        }
        let mut open = true;
        egui::Window::new("Palette")
            .open(&mut open)
            .show(ctx, |ui| {
                egui::Grid::new("palette_grid")
                    .spacing([2.0, 2.0])
                    .show(ui, |ui| {
                        for row in 0..2 {
                            for col in 0..10 {
                                let color = COLORS[row * 10 + col];
                                let (rect, response) =
                                    ui.allocate_exact_size(egui::vec2(20.0, 20.0), Sense::click());
                                ui.painter().rect_filled(rect, 0.0, color);
                                ui.painter().rect_stroke(
                                    rect,
                                    0.0,
                                    egui::Stroke::new(1.0, Color32::from_gray(40)),
                                    egui::StrokeKind::Inside,
                                );
                                if response.clicked() {
                                    self.tools.primary = color;
                                }
                                if response.secondary_clicked() {
                                    self.tools.secondary = color;
                                }
                            }
                            ui.end_row();
                        }
                    });

                ui.separator();
                ui.horizontal(|ui| {
                    color_swatch(ui, self.tools.primary, 32.0);
                    ui.label("Primary");
                    color_swatch(ui, self.tools.secondary, 32.0);
                    ui.label("Secondary");
                });

                ui.horizontal(|ui| {
                    if ui.button("Swap (X)").clicked() {
                        std::mem::swap(&mut self.tools.primary, &mut self.tools.secondary);
                    }
                    ui.label("Custom…");
                    ui.color_edit_button_srgba(&mut self.tools.primary);
                });
            });
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
            egui::TextureOptions::NEAREST,
        );
    }

    fn repaint_full(&mut self) {
        let image = egui::ColorImage::from_rgba_premultiplied(
            [self.canvas.width as usize, self.canvas.height as usize],
            self.canvas.data(),
        );
        self.canvas_tex.set(image, egui::TextureOptions::NEAREST);
    }

    fn draw_canvas(&mut self, ui: &mut egui::Ui, panel_rect: Rect) {
        self.last_panel_rect = panel_rect;
        let response = ui.interact(
            panel_rect,
            ui.scope_id().with("canvas_interaction"),
            Sense::click_and_drag(),
        );

        self.handle_zoom(
            ui,
            panel_rect,
            self.canvas_rect(panel_rect),
            response.hovered(),
        );
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
    }
}

fn color_swatch(ui: &mut egui::Ui, color: Color32, size: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), Sense::hover());
    ui.painter().rect_filled(rect, 2.0, color);
    ui.painter().rect_stroke(
        rect,
        2.0,
        egui::Stroke::new(1.0, Color32::from_gray(40)),
        egui::StrokeKind::Inside,
    );
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        #[cfg(target_arch = "wasm32")]
        self.poll_file_task();
        self.handle_hotkeys(&ctx);
        self.top_menu(ui);

        egui::CentralPanel::default().show(ui, |ui| {
            let panel_rect = ui.max_rect();
            self.draw_canvas(ui, panel_rect);
        });

        self.tools_window(&ctx);
        self.palette_window(&ctx);
        self.discard_dialog(&ctx);
        self.new_dialog(&ctx);
        self.resize_dialog(&ctx);
        self.error_dialog(&ctx);
        self.update_title(&ctx);
    }
}
