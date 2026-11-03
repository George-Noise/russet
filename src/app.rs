use eframe::egui::{self, Color32, Key, Modifiers, PointerButton, Pos2, Rect, Sense, Vec2};

use crate::canvas::Canvas;
use crate::history::History;
use crate::tools::{Tool, ToolState, draw_segment};

const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 8.0;
const ZOOM_STEP: f32 = 1.25;

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
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::P)) {
            self.tools.tool = Tool::Pencil;
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::B)) {
            self.tools.tool = Tool::Brush;
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::E)) {
            self.tools.tool = Tool::Eraser;
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

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.handle_hotkeys(&ctx);

        egui::CentralPanel::default().show(ui, |ui| {
            let panel_rect = ui.max_rect();
            self.draw_canvas(ui, panel_rect);
        });
    }
}
