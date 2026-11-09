//! The main-canvas viewport (SPEC §3.4): where the canvas sits on screen,
//! zooming (Paint.NET-style discrete steps, Ctrl+wheel toward the cursor,
//! physical-pixel snapping) and panning (plain/Shift wheel, middle button,
//! Space+LMB). A child module of `app`, so it can touch `App`'s private
//! fields; `app.rs` stays the orchestrator.

use eframe::egui::{self, Key, PointerButton, Pos2, Rect, Vec2};

use super::{
    App, CANVAS_TEXTURE_CRISP, CANVAS_TEXTURE_SMOOTH, MAX_ZOOM, MIN_ZOOM, SCROLL_PAN_SPEED,
    ZOOM_SNAP_RATIO, ZOOM_STEP_FACTOR,
};

impl App {
    pub(super) fn canvas_rect(&self, panel_rect: Rect) -> Rect {
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

    pub(super) fn canvas_texture_options(&self) -> egui::TextureOptions {
        if self.crisp_zoom() {
            CANVAS_TEXTURE_CRISP
        } else {
            CANVAS_TEXTURE_SMOOTH
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

    pub(super) fn set_zoom(&mut self, new_zoom: f32, anchor: Option<Pos2>, panel_rect: Rect) {
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

    pub(super) fn reset_zoom(&mut self) {
        self.set_zoom(1.0, None, self.last_panel_rect);
        self.pan = Vec2::ZERO;
    }

    pub(super) fn handle_scroll(&mut self, ui: &egui::Ui, panel_rect: Rect, hovered: bool) {
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

    pub(super) fn handle_pan(&mut self, ui: &egui::Ui, response: &egui::Response) -> bool {
        let space = ui.input(|i| i.key_down(Key::Space));
        let panning = response.dragged_by(PointerButton::Middle)
            || (space && !self.is_stroking && response.dragged_by(PointerButton::Primary));
        if panning {
            self.pan += response.drag_delta();
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        }
        panning
    }

    pub(super) fn canvas_pos(&self, rect: Rect, screen: Pos2) -> Pos2 {
        let ppp = self.ctx.pixels_per_point();
        let p = (screen - rect.min) * ppp / self.zoom;
        Pos2::new(p.x, p.y)
    }

    pub(super) fn fit_to_window(&mut self) {
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
}

#[cfg(test)]
mod tests {
    use super::App;

    fn percent(zoom: f32) -> i32 {
        (zoom * 100.0).round() as i32
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
}
