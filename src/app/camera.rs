//! The main-canvas viewport (SPEC §3.4): where the canvas sits on screen,
//! zooming (Paint.NET-style discrete steps, Ctrl+wheel toward the cursor,
//! physical-pixel snapping) and panning (plain/Shift wheel, middle button,
//! Space+LMB, W/A/S/D — with the combo latch, [`PanKeyLatch`], that keeps a
//! letter pressed under an accelerator from panning — and the MULTI-TOUCH
//! gesture, [`TouchGesture`], owner requests 2026-11-25/26: two or more
//! fingers PAN by the average movement of all touch points and PINCH-zoom
//! by the touches' spreading distance ratio, egui's
//! `InputState::multi_touch()` deltas). The SECOND finger's landing frame
//! (the raw Touch stream's 1→2 id transition — owner bug report
//! 2026-11-26, the second pass: a real phone's fingers never land in one
//! frame, and egui's gesture may engage LATE or never at all, so the
//! takeover and the suppression ride the RAW stream, never the gesture)
//! triggers the TAKEOVER revert in `input.rs::touch_takeover`. A child
//! module of `app`, so it can touch `App`'s private fields; `app.rs` stays
//! the orchestrator.

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
        // The full re-upload below exists so a crisp/smooth SAMPLING change
        // takes effect immediately (glow applies the sampler params on
        // every upload), and it only matters when the zoom actually CROSSES
        // between the crisp and the smooth family — so it is paid exactly
        // then (owner request 2026-11-26, with the continuous pinch: one
        // re-upload per crossing instead of one per pinch frame).
        let crisp_before = self.crisp_zoom();
        if let Some(anchor) = anchor {
            let ppp = self.ctx.pixels_per_point();
            let rect = self.canvas_rect(panel_rect);
            let canvas = egui::vec2(self.canvas.width as f32, self.canvas.height as f32);
            let p = (anchor - rect.min) * ppp / self.zoom;
            let new_min = anchor - p * (new_zoom / ppp);
            self.pan = new_min - panel_rect.center() + canvas * (new_zoom / ppp) * 0.5;
        }
        self.zoom = new_zoom;
        if self.crisp_zoom() != crisp_before {
            self.repaint_full();
        }
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
            // The content follows the pointer (SPEC §3.4's mouse pan). This
            // line was LOST in the 2026-11-13 app.rs split (e8eb5d1) — the
            // detection, the Grabbing cursor and the stroke suppression
            // survived, but the view never moved; restored 2026-11-25 with
            // a regression test (camera/tests/touch_gesture.rs). Same sign
            // convention as the touch pan below: the canvas moves WITH the
            // drag.
            self.pan += response.drag_delta();
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        }
        drag_panning || key_panning
    }

    /// The MULTI-TOUCH gesture step (owner requests 2026-11-25/2026-11-26,
    /// the F20 web slices): while TWO OR MORE fingers are down egui
    /// reports a multi-touch gesture — its `translation_delta` (the
    /// average movement of all touch points) pans the view, the content
    /// following the fingers (the same convention as the mouse drag pan
    /// above), and its `zoom_delta` (the ratio of the touches' average
    /// pairwise distance to the previous frame's) is the PINCH zoom: a
    /// phone user has neither keyboard nor wheel, so the same two fingers
    /// carry BOTH jobs — dragging them pans, spreading/pinching them
    /// zooms. The takeover and the suppression, though, ride the RAW
    /// Touch stream (owner bug report 2026-11-26, second pass: a real
    /// phone's fingers land in DIFFERENT frames and the gesture may
    /// engage LATE or never): the SECOND finger's landing frame
    /// ([`TouchGestureFrame::second_finger`]) REVERTS everything the
    /// touch session has painted so far (`input::touch_takeover`), and
    /// the session stays "panning"
    /// ([`TouchGestureFrame::session`]) — no stroke starts or resumes
    /// — until the last finger lifts, gesture or no gesture.
    /// egui itself zeroes BOTH deltas on the frames the finger COUNT changes
    /// (finger add/remove re-baselines the average), so no debounce is
    /// needed on this side either.
    pub(super) fn handle_touch_gesture(
        &mut self,
        ui: &egui::Ui,
        panel_rect: Rect,
    ) -> TouchGestureFrame {
        let (multi, touches_down, fold) = ui.input(|i| {
            (
                i.multi_touch(),
                i.any_touches(),
                fold_touch_ids(&i.raw.events),
            )
        });
        let frame = self
            .touch_gesture
            .step(&fold.0, &fold.1, touches_down, multi);
        // The takeover baseline (owner bug report 2026-11-26): latched on
        // the touch session's BIRTH pass — before the same pass's tool
        // dispatch can paint anything (the first finger's emulated pointer
        // press paints AFTER this) — and cleared when the last finger
        // lifts. The SECOND finger's landing frame (`frame.second_finger`,
        // handled by the caller in `draw_canvas`) reverts the active
        // layer's history to this baseline — see [`input::touch_takeover`].
        if frame.first_touch {
            self.touch_baseline = Some((self.layers.active_layer(), self.layers.head()));
        }
        if !touches_down {
            self.touch_baseline = None;
        }
        // Zoom FIRST, pan SECOND — the exact fingers-follow math: scaling
        // around the OLD midpoint keeps the canvas point that was under it
        // fixed there, and the pan delta (the midpoint's own movement)
        // then carries it to the NEW midpoint. (Applied in the other order
        // the anchor would read the already-shifted canvas and the content
        // would drift off the fingers by (f-1)·delta each frame.)
        if frame.zoom_factor != 1.0 && frame.zoom_factor.is_finite() {
            // The anchor is the PREVIOUS frame's gesture center (see
            // [`TouchGestureFrame::anchor`]); `None` (no prior center —
            // practically only a re-baselined first frame, whose factor is
            // 1.0 anyway) takes `set_zoom`'s plain panel-center path.
            // `is_finite` guards a degenerate device report: egui computes
            // the ratio of two average distances, and two fingers resting
            // on the SAME point make the previous average 0 — the ratio
            // turns inf (or 0/0 NaN) and would poison the zoom state;
            // the frame is dropped instead.
            self.set_zoom(self.zoom * frame.zoom_factor, frame.anchor, panel_rect);
            ui.ctx().request_repaint();
        }
        if frame.delta != Vec2::ZERO {
            self.pan += frame.delta;
            ui.ctx().request_repaint();
        }
        frame
    }

    /// W/A/S/D pan the viewport continuously (§3.4). `egui::Key` letter
    /// variants are PHYSICAL keys — egui-winit maps them from winit's
    /// `PhysicalKey` — so the bindings are keyboard-layout independent: on
    /// the Russian typewriter layout the same physical keys work, and so on for every
    /// other layout. While a pan key is held, no stroke starts (the caller
    /// treats this as panning, same as Space+LMB).
    fn handle_key_pan(&mut self, ui: &egui::Ui) -> bool {
        // The combo latch rides the RAW events (`i.raw.events`), which
        // nothing consumes: the hotkeys' `consume_key` clears only
        // `i.events` (egui 0.37 `count_and_consume_key`), so the Ctrl+A /
        // Ctrl+S press that already fired a combo still reaches this scan
        // with its modifiers intact. The update runs BEFORE the keyboard
        // gate so release events are never missed while a text field holds
        // the focus.
        ui.input(|i| self.pan_key_latch.update(&i.raw.events));
        // Same gate as the hotkeys: keys must not pan while focus is in a
        // dialog input field.
        if ui.ctx().egui_wants_keyboard_input() {
            return false;
        }
        let latch = self.pan_key_latch;
        let (dir, suppressed) = ui.input(|i| {
            (
                key_pan_dir(
                    i.key_down(Key::W) && !latch.blocks(Key::W),
                    i.key_down(Key::A) && !latch.blocks(Key::A),
                    i.key_down(Key::S) && !latch.blocks(Key::S),
                    i.key_down(Key::D) && !latch.blocks(Key::D),
                ),
                pan_suppressed_by_modifiers(i.modifiers),
            )
        });
        if suppressed || dir == Vec2::ZERO {
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

/// An ACCELERATOR modifier (Ctrl/⌘/Alt) suppresses the W/A/S/D pan (owner
/// bug report 2026-11-20: Ctrl+S and Ctrl+Shift+S also panned, because the
/// pan reads the PRESSED state — `i.key_down` — and the save's
/// `consume_key` does not lift the key back up; with Ctrl held the keys
/// belong to the hotkey layer). Shift stays allowed: it is no
/// accelerator, and Shift+pan is harmless (the region-edit modifiers
/// apply to pointer drags only).
fn pan_suppressed_by_modifiers(m: egui::Modifiers) -> bool {
    m.ctrl || m.command || m.mac_cmd || m.alt
}

/// The multi-touch gesture session (owner requests 2026-11-25/2026-11-26,
/// the F20 web slices; the pure machine behind
/// [`App::handle_touch_gesture`]). egui's `multi_touch()` reports a gesture
/// while two or more fingers are down; this state keeps the SESSION alive
/// after the gesture proper ends (one finger still down): the whole rest of
/// the touch session counts as "panning", so the leftover finger cannot
/// resume painting until every finger has lifted (the standard paint-app
/// convention). `began` flags the gesture's FIRST frame — the takeover
/// moment, on which the caller REVERTS everything the touch session has
/// painted so far (`input::touch_takeover`; the previous convention —
/// committing the in-flight stroke, the painted pixels staying — was the
/// owner's bug report 2026-11-26). `first_touch` flags the session's BIRTH
/// pass (the 0→any-touches transition), on which the caller latches the
/// history baseline to revert to ([`App::touch_baseline`]). The state also
/// remembers the PREVIOUS frame's gesture center — the pinch zoom's
/// anchor (see [`TouchGestureFrame::anchor`]). Pure so the convention is
/// unit-testable without an egui context; App carries it next to `pan`
/// (like [`PanKeyLatch`]).
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct TouchGesture {
    /// The touch ids the raw stream says are on the glass RIGHT NOW —
    /// [`fold_touch_ids`] of each frame's `Event::Touch` stream adds ids
    /// on `Start` and removes them on `End`/`Cancel` (a `Move` carries no
    /// id change). The set is the machine's own truth, reconciled against
    /// egui's `any_touches()` each step.
    active: Vec<egui::TouchId>,
    /// Has this touch session already seen a SECOND finger? Set on the
    /// 1→2 id transition (or a `multi_touch()` report — whichever
    /// comes first) and held until the last finger lifts; while it holds,
    /// the caller suppresses all tool input for the session.
    poisoned: bool,
    /// The PREVIOUS gesture frame's center — the pinch zoom's anchor.
    prev_center: Option<Pos2>,
}

/// What one frame of [`TouchGesture::step`] decided.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct TouchGestureFrame {
    /// The pan delta to apply THIS frame — zero on every frame without an
    /// active multi-touch gesture, including the finger-count-change
    /// frames where egui itself zeroes the delta.
    pub(super) delta: Vec2,
    /// The pinch zoom factor to apply THIS frame — a MULTIPLICATIVE
    /// ratio (1.0 = none; > 1 spread, < 1 pinch), zeroed to 1.0 by egui
    /// on the finger-count-change frames exactly like `delta`.
    pub(super) zoom_factor: f32,
    /// The PREVIOUS frame's gesture center — the zoom's anchor: keeping
    /// THAT screen point fixed while scaling by `zoom_factor`, THEN
    /// applying `delta`, lands the canvas point that was under the old
    /// midpoint exactly under the new one (the content follows the
    /// fingers). `None` when no gesture frame preceded this one
    /// (practically only a re-baselined first frame — its `zoom_factor`
    /// is 1.0 anyway, so the anchor is never read there).
    pub(super) anchor: Option<Pos2>,
    /// True on the frame the SECOND finger lands — the takeover
    /// moment: the raw Touch stream's 1→2 id transition (a simultaneous
    /// landing births AND takes over in one pass), or the first
    /// `multi_touch()` report if the raw fold somehow missed the starts.
    /// The caller reverts the session's tool marks
    /// ([`App::touch_takeover`]) and the session turns "panning" from
    /// this frame.
    pub(super) second_finger: bool,
    /// True while the touch session is POISONED — a second finger has
    /// landed and not every finger has lifted yet — the caller treats it
    /// as "panning" (no stroke starts or resumes). A lone-finger drawing
    /// session never poisons; the leftover finger of a pinch keeps the
    /// poison until it too lifts.
    pub(super) session: bool,
    /// True on the touch session's BIRTH pass — the pass where touches
    /// went from none to any (one finger landing, or a simultaneous
    /// landing that also begins the gesture). The caller latches the
    /// history baseline ON this pass ([`App::touch_baseline`]), before
    /// the same pass's tool dispatch can paint — so a gesture that
    /// grows out of the session can revert every mark the session
    /// made ([`App::touch_takeover`]).
    pub(super) first_touch: bool,
}

impl TouchGesture {
    /// The derived session state: poisoned AND some finger still on the
    /// glass — what the caller's "panning" gate reads (the frame carries
    /// it as `session`; this accessor is for between-pass assertions).
    pub(super) fn session_live(&self) -> bool {
        self.poisoned && !self.active.is_empty()
    }

    /// One frame's decision. `started`/`ended` are the frame's raw
    /// Touch-stream id changes ([`fold_touch_ids`] of `i.raw.events`),
    /// `any_touches` is egui's authoritative "is any finger down"
    /// (reconciles the id set), and `multi` is the frame's multi-touch
    /// info — `Some` exactly while a two-plus-finger gesture is
    /// underway — egui computes `translation_delta` and `zoom_delta`
    /// against the previous gesture state, zeroing BOTH on
    /// finger-count-change frames. The takeover and the suppression are
    /// decided from the RAW ids alone: a second finger that lands while
    /// the gesture has not (yet) engaged still fires `second_finger` and
    /// poisons the session (owner report 2026-11-26: a phone's fingers
    /// land in DIFFERENT frames, and the gesture may never engage).
    fn step(
        &mut self,
        started: &[egui::TouchId],
        ended: &[egui::TouchId],
        any_touches: bool,
        multi: Option<egui::MultiTouchInfo>,
    ) -> TouchGestureFrame {
        // The raw fold: remove the ended ids, add the started ones (the
        // set is order-proof — an End and a Start of the same id in one
        // frame leaves it down, a re-Start of a live id is a no-op).
        let was_empty = self.active.is_empty();
        for id in ended {
            self.active.retain(|a| a != id);
        }
        for id in started {
            if !self.active.contains(id) {
                self.active.push(*id);
            }
        }
        // Reconcile with egui's touch state: if no finger is down there,
        // the whole set is stale (a missed End/Cancel) — clear it, so a
        // lost end cannot wedge the session poisoned forever.
        if !any_touches {
            self.active.clear();
        }
        // The session's birth: no live ids before this fold, and any now.
        // A simultaneous landing (both fingers in ONE pass) births AND
        // takes over in the same pass — the birth flag still fires so
        // the baseline latch precedes the takeover.
        let first_touch = was_empty && !self.active.is_empty();
        // The second finger: the 1→2 raw transition, or the gesture report
        // as a belt-and-braces fallback (a seeded gesture implies two
        // live touches, so the fold must have seen the starts).
        let second_finger = !self.poisoned && (self.active.len() >= 2 || multi.is_some());
        if second_finger {
            self.poisoned = true;
        }
        // The session ends with the last finger: the poison lifts only
        // when the glass is empty again.
        if self.active.is_empty() {
            self.poisoned = false;
            self.prev_center = None;
        }
        let (delta, zoom_factor, anchor) = match multi {
            Some(m) => {
                // The zoom anchors at the PREVIOUS center (the exact
                // content-follows-the-fingers math — see
                // `TouchGestureFrame::anchor`); this frame's center becomes
                // the next frame's previous.
                let anchor = self.prev_center;
                self.prev_center = Some(m.center_pos);
                (m.translation_delta, m.zoom_delta, anchor)
            }
            None => (Vec2::ZERO, 1.0, None),
        };
        TouchGestureFrame {
            delta,
            zoom_factor,
            anchor,
            second_finger,
            session: self.session_live(),
            first_touch,
        }
    }
}

/// The raw Touch stream's per-frame fold: which touch ids STARTED this
/// frame, which ENDED (`End` or `Cancel` — both take the finger off the
/// glass). `Move` events carry no id change, and multiple coalesced
/// events of one id in a frame fold to the same result. Feeding
/// [`TouchGesture::step`] from THIS fold — not from egui's gesture —
/// is what makes the takeover and the suppression independent of the
/// gesture's backend-specific seeding (owner report 2026-11-26).
fn fold_touch_ids(events: &[egui::Event]) -> (Vec<egui::TouchId>, Vec<egui::TouchId>) {
    let mut started = Vec::new();
    let mut ended = Vec::new();
    for event in events {
        if let egui::Event::Touch { id, phase, .. } = event {
            match phase {
                egui::TouchPhase::Start => {
                    if !started.contains(id) {
                        started.push(*id);
                    }
                }
                egui::TouchPhase::End | egui::TouchPhase::Cancel => {
                    if !ended.contains(id) {
                        ended.push(*id);
                    }
                }
                egui::TouchPhase::Move => {}
            }
        }
    }
    (started, ended)
}

/// The W/A/S/D combo latch (owner bug report 2026-11-24: Ctrl+S with Ctrl
/// RELEASED first — S still held — panned down, and Ctrl+A ditto for A;
/// the owner asked for a ≥100 ms debounce so the combo's tail cannot move
/// the view). A letter whose PRESS event arrives while an accelerator is
/// held belongs to the hotkey layer: it pans NOTHING for that physical
/// press, even after the modifier lifts — the suppression lives until the
/// letter's own release event, which is strictly longer than any debounce
/// window the hold can outlive. A release (or a press with no accelerator,
/// i.e. a fresh deliberate pan) never latches. State is four flags, one
/// per pan letter; `App` carries it next to `pan`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct PanKeyLatch {
    w: bool,
    a: bool,
    s: bool,
    d: bool,
}

impl PanKeyLatch {
    /// Does this latched letter hold its key's pan contribution back?
    /// Keys outside the pan set are never blocked.
    fn blocks(self, key: Key) -> bool {
        match key {
            Key::W => self.w,
            Key::A => self.a,
            Key::S => self.s,
            Key::D => self.d,
            _ => false,
        }
    }

    /// Consumes ONE frame's raw events in delivery order: a pan-letter
    /// press under an accelerator latches the letter, the letter's release
    /// clears it. Every `Event::Key` carries the modifier state of its own
    /// moment (egui-winit stamps it), so no cross-frame modifier tracking
    /// is needed. Repeats change nothing: the latch rides the physical
    /// press, a repeat under the still-held accelerator re-sets an already
    /// set flag, and a repeat after the modifier lifted carries no
    /// accelerator, so it cannot set or clear anything.
    fn update(&mut self, events: &[egui::Event]) {
        for event in events {
            let egui::Event::Key {
                key,
                pressed,
                modifiers,
                ..
            } = event
            else {
                continue;
            };
            match key {
                Key::W | Key::A | Key::S | Key::D => {
                    if *pressed {
                        if pan_suppressed_by_modifiers(*modifiers) {
                            self.set(*key);
                        }
                    } else {
                        self.clear(*key);
                    }
                }
                _ => {}
            }
        }
    }

    fn set(&mut self, key: Key) {
        match key {
            Key::W => self.w = true,
            Key::A => self.a = true,
            Key::S => self.s = true,
            Key::D => self.d = true,
            _ => {}
        }
    }

    fn clear(&mut self, key: Key) {
        match key {
            Key::W => self.w = false,
            Key::A => self.a = false,
            Key::S => self.s = false,
            Key::D => self.d = false,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
