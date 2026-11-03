use eframe::egui::{self, Color32, Key, Modifiers, PointerButton, Rect, Sense, Vec2};

use crate::canvas::Canvas;

const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 8.0;
const ZOOM_STEP: f32 = 1.25;

pub struct App {
    canvas: Canvas,
    canvas_tex: egui::TextureHandle,
    checker_tex: egui::TextureHandle,
    zoom: f32,
    pan: Vec2,
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
            zoom: 1.0,
            pan: Vec2::ZERO,
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

    fn handle_pan(&mut self, ui: &egui::Ui, response: &egui::Response) {
        let space = ui.input(|i| i.key_down(Key::Space));
        let panning = response.dragged_by(PointerButton::Middle)
            || (space && response.dragged_by(PointerButton::Primary));
        if panning {
            self.pan += response.drag_delta();
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
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
        self.handle_pan(ui, &response);

        let painter = ui.painter_at(panel_rect);
        painter.rect_filled(panel_rect, 0.0, Color32::from_rgb(0x40, 0x40, 0x40));

        let rect = self.canvas_rect(panel_rect);
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

        egui::Window::new("M2 Test").show(&ctx, |ui| {
            if ui.button("Fill region").clicked() {
                let (x, y, w, h) = (10u32, 10u32, 50u32, 50u32);
                let stride = self.canvas.width as usize * 4;
                let data = self.canvas.pixmap_mut().data_mut();
                for row in y..y + h {
                    for col in x..x + w {
                        let i = row as usize * stride + col as usize * 4;
                        data[i..i + 4].copy_from_slice(&[0, 0, 255, 255]);
                    }
                }
                self.sync_texture(x, y, w, h);
            }
            if ui.button("Repaint full").clicked() {
                self.repaint_full();
            }
        });
    }
}
