//! The main-canvas viewport (SPEC §3.4): where the canvas sits on screen,
//! zooming (Paint.NET-style discrete steps, Ctrl+wheel toward the cursor,
//! physical-pixel snapping) and panning (plain/Shift wheel, middle button,
//! Space+LMB, W/A/S/D). A child module of `app`, so it can touch `App`'s
//! private fields; `app.rs` stays the orchestrator.

use eframe::egui::{self, Key, MouseWheelUnit, PointerButton, Pos2, Rect, Vec2};

use super::{
    App, CANVAS_TEXTURE_CRISP, CANVAS_TEXTURE_SMOOTH, KEY_PAN_SPEED, MAX_ZOOM, MIN_ZOOM,
    WHEEL_NOTCH_PAN, ZOOM_SNAP_RATIO, ZOOM_STEP_FACTOR,
};

/// One physical wheel notch of a POINT-unit wheel ≈ 100 pt (F12 variant B,
/// owner pick 2026-11-16, INTERIM — NOT the final design): eframe-web passes
/// the DOM wheel delta as-is and Chrome-based browsers emit ≈ 100 px per
/// notch (`DOM_DELTA_PIXEL` → `MouseWheelUnit::Point`); winit-Wayland sends
/// the same shape for smooth/high-resolution wheels (logical points). The
/// constant is calibrated from `--features diagnostics` wheel logs — retune
/// it when the owner's logs show another per-notch value. F12 stays open.
const POINT_UNIT_NOTCH_PT: f32 = 100.0;

/// On the WEB backend a Line-unit delta counts LINES: eframe passes the DOM
/// wheel delta as-is (`DOM_DELTA_LINE` → `MouseWheelUnit::Line`), and Firefox
/// — the major browser that emits line mode for a classic wheel click —
/// reports 3 lines per notch (Chrome/Safari use pixel mode there, i.e. the
/// `Point` branch). Native winit `LineDelta` is already ±1 per notch
/// (Windows/X11, Wayland discrete), so the divisor applies to the web
/// backend ONLY. INTERIM F12-B calibration (owner report 2026-11-16: one
/// Firefox click panned ~3× the Windows/Chrome notch — 3 lines ÷ 3 = 1
/// notch), retuned from `--features diagnostics` wheel logs. F12 stays open.
const WEB_LINE_UNIT_LINES_PER_NOTCH: f32 = 3.0;

/// A PAGE-unit wheel notch (practically only old Firefox with a pref) is one
/// viewport tall — egui itself converts it to `viewport_rect.height()`. For
/// pan it is capped at 3 notches per page: an interim product choice (F12-B,
/// not final), never observed in the wild so far.
const PAGE_UNIT_NOTCHES: f32 = 3.0;

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
                unit,
                delta,
                source: _source,
                modifiers,
                ..
            } = event
            else {
                continue;
            };
            // F12 variant B (owner pick 2026-11-16, INTERIM): pan is defined
            // in PRODUCT NOTCHES, not in the event's raw unit — one physical
            // notch pans the same distance on every platform. The raw delta
            // is first converted to notches per its unit (see
            // [`wheel_notches`]), then each notch pans [`WHEEL_NOTCH_PAN`].
            // Zoom below is untouched: it reads the SIGN of the raw delta
            // only, which is why it was always platform-consistent.
            let notches = wheel_notches(unit, delta, cfg!(target_arch = "wasm32"));
            let pan_step = if modifiers.ctrl || modifiers.command {
                Vec2::ZERO // the ctrl branch below zooms on the sign only
            } else if modifiers.shift {
                egui::vec2((notches.x + notches.y) * WHEEL_NOTCH_PAN, 0.0)
            } else {
                egui::vec2(0.0, notches.y * WHEEL_NOTCH_PAN)
            };
            #[cfg(feature = "diagnostics")]
            eprintln!(
                "[diagnostics] wheel unit={unit:?} src={_source:?} delta=({:.2},{:.2}) notches=({:.3},{:.3}) ctrl={} shift={} pan_step=({:.2},{:.2})",
                delta.x,
                delta.y,
                notches.x,
                notches.y,
                modifiers.ctrl,
                modifiers.shift,
                pan_step.x,
                pan_step.y
            );
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
                self.pan.x += pan_step.x;
            } else {
                self.pan.y += pan_step.y;
            }
        }
    }

    pub(super) fn handle_pan(&mut self, ui: &egui::Ui, response: &egui::Response) -> bool {
        let space = ui.input(|i| i.key_down(Key::Space));
        let drag_panning = response.dragged_by(PointerButton::Middle)
            || (space && !self.is_stroking && response.dragged_by(PointerButton::Primary));
        let key_panning = self.handle_key_pan(ui);
        if drag_panning {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        }
        drag_panning || key_panning
    }

    /// W/A/S/D pan the viewport continuously (§3.4). `egui::Key` letter
    /// variants are PHYSICAL keys — egui-winit maps them from winit's
    /// `PhysicalKey` — so the bindings are keyboard-layout independent: on
    /// the Russian typewriter layout the same physical keys work, and so on for every
    /// other layout. While a pan key is held, no stroke starts (the caller
    /// treats this as panning, same as Space+LMB).
    fn handle_key_pan(&mut self, ui: &egui::Ui) -> bool {
        // Same gate as the hotkeys: keys must not pan while focus is in a
        // dialog input field.
        if ui.ctx().egui_wants_keyboard_input() {
            return false;
        }
        let dir = ui.input(|i| {
            key_pan_dir(
                i.key_down(Key::W),
                i.key_down(Key::A),
                i.key_down(Key::S),
                i.key_down(Key::D),
            )
        });
        if dir == Vec2::ZERO {
            return false;
        }
        self.pan += dir * (KEY_PAN_SPEED * ui.input(|i| i.stable_dt.min(0.1)));
        // Repaint even without OS key-repeat events, so a held key pans
        // smoothly every frame.
        ui.ctx().request_repaint();
        true
    }

    pub(super) fn canvas_pos(&self, rect: Rect, screen: Pos2) -> Pos2 {
        let ppp = self.ctx.pixels_per_point();
        let p = (screen - rect.min) * ppp / self.zoom;
        Pos2::new(p.x, p.y)
    }

    /// Inverse of [`Self::canvas_pos`]: canvas point → logical screen point
    /// (used by the preview overlays of the Line/Shape/Text tools).
    pub(super) fn screen_pos(&self, rect: Rect, canvas: Pos2) -> Pos2 {
        let ppp = self.ctx.pixels_per_point();
        rect.min + canvas.to_vec2() * (self.zoom / ppp)
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

/// Wheel delta in its EVENT UNIT → NOTCH count (F12 variant B): native
/// `Line` deltas are already notches (winit sends `LineDelta(±1)` per notch
/// on Windows/X11 and for Wayland discrete events; fractional lines =
/// high-resolution wheels/touchpads, so panning stays proportional there);
/// on the WEB backend `Line` counts lines, so the delta is divided by
/// [`WEB_LINE_UNIT_LINES_PER_NOTCH`] (Firefox's 3 lines per notch — the
/// owner's overshoot report 2026-11-16). `Point` deltas (eframe-web DOM
/// pixels, Wayland pixel deltas) are divided by [`POINT_UNIT_NOTCH_PT`]; a
/// `Page` notch is capped at [`PAGE_UNIT_NOTCHES`]. `web_backend` comes from
/// `cfg!(target_arch = "wasm32")` at the call site. A free function so the
/// convention is unit-testable without an egui context (both backend
/// branches are tested on native).
fn wheel_notches(unit: MouseWheelUnit, delta: Vec2, web_backend: bool) -> Vec2 {
    match unit {
        MouseWheelUnit::Point => delta / POINT_UNIT_NOTCH_PT,
        MouseWheelUnit::Line => {
            if web_backend {
                delta / WEB_LINE_UNIT_LINES_PER_NOTCH
            } else {
                delta
            }
        }
        MouseWheelUnit::Page => delta * PAGE_UNIT_NOTCHES,
    }
}

/// W/A/S/D pan direction, normalized (a free function so the convention is
/// unit-testable without an egui context). "Camera" convention, matching
/// the wheel: W/S/A/D move the VIEWPORT up/down/left/right — the canvas
/// content moves the opposite way on screen. Signs follow `pan`'s
/// definition in `canvas_rect`: growing `pan.y` draws the canvas lower,
/// which is the view moving up (same as a wheel-up scroll).
fn key_pan_dir(w: bool, a: bool, s: bool, d: bool) -> Vec2 {
    let mut dir = Vec2::ZERO;
    if w {
        dir.y += 1.0;
    }
    if s {
        dir.y -= 1.0;
    }
    if a {
        dir.x += 1.0;
    }
    if d {
        dir.x -= 1.0;
    }
    // `normalized` is the safe variant in emath 0.37 (zero in → zero out).
    dir.normalized()
}

#[cfg(test)]
mod tests;
