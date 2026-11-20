//! Raw input handling of the canvas: the global hotkeys (SPEC §3.9), the
//! freehand stroke gestures (pencil / brush / eraser — press, drag,
//! release, with the Space-pan guard), the Eyedropper picking, and the
//! smoothed-piece painting that feeds the freehand tools (owner request
//! 2026-11-19: every coalesced `Event::PointerMoved` of the frame feeds
//! the `tools::smooth` window — fast strokes are smooth, the pencil stays
//! a 1 px Bresenham line).
//!
//! Tool-bound gestures (Line/Curve/Shape/Text) live in [`super::preview`]
//! and [`super::text_tool`]; the per-frame dispatch happens in the
//! orchestrator's `draw_canvas` ([`super`]). Split off `app.rs` by the
//! owner request 2026-11-13.

use eframe::egui::{self, Color32, Key, Modifiers, PointerButton, Pos2, Rect};

use super::App;
use crate::history_tree::ActionKind;
use crate::tools::{
    MAX_STROKE_SIZE, PencilTrail, SmoothPiece, SmoothStroke, Tool, draw_segment, pencil_pixel,
    pick_color, rasterize_smooth_pieces,
};

/// Which platform clipboard combo egui-winit delivered as a dedicated
/// event this frame ([`App::handle_hotkeys`] consumes it and runs the
/// matching selection op).
#[derive(Clone, Copy, PartialEq)]
enum ClipboardOp {
    Copy,
    Cut,
    Paste,
}

impl App {
    pub(super) fn handle_hotkeys(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Num0)) {
            self.reset_zoom();
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::S)) {
            self.save_as();
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::S)) {
            self.save();
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
        // The selection ops (owner request 2026-11-20): Ctrl+C/X/V mirror
        // the Edit menu and the Select panel's buttons. The egui-winit
        // `clipboard_shortcuts` conversion is DISABLED (owner decision
        // 2026-11-20 — `NativeOptions::clipboard_shortcuts = false` in
        // main.rs), so on NATIVE the combos arrive as plain Key events and
        // `consume_key` sees them; egui-winit fills the Key event's `key`
        // field as logical-or-physical (issue #3653), so the combos still
        // match on non-Latin keyboard layouts. The DEDICATED
        // `Event::Copy`/`Cut`/`Paste` events are consumed TOO — the web
        // backend synthesizes them from the browser's own copy/cut/paste
        // DOM events (eframe `web/events.rs`, not flag-controlled); a
        // frame that already fired an op from a Key event does not repeat
        // it for the dedicated twin (a web keydown AND a copy event can
        // carry the same gesture). At most ONE op runs per frame. A no-op
        // when the selection or the clipboard is empty — never a
        // fall-through (the redo-keys convention). The keys run BEFORE
        // the plain tool keys: consume_key matches modifiers logically
        // (extra Ctrl is ignored), so Ctrl+V must be consumed first or
        // the plain `V` (Select) would also fire — the same ordering the
        // redo/undo pair uses below.
        let key_op = if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::C)) {
            Some(ClipboardOp::Copy)
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::X)) {
            Some(ClipboardOp::Cut)
        } else if ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::V)) {
            Some(ClipboardOp::Paste)
        } else {
            None
        };
        let dedicated_op = ctx.input_mut(|i| {
            let mut op = None;
            i.events.retain(|event| match event {
                egui::Event::Copy => {
                    op = Some(ClipboardOp::Copy);
                    false
                }
                egui::Event::Cut => {
                    op = Some(ClipboardOp::Cut);
                    false
                }
                egui::Event::Paste(_) => {
                    op = Some(ClipboardOp::Paste);
                    false
                }
                _ => true,
            });
            op
        });
        match key_op.or(dedicated_op) {
            Some(ClipboardOp::Copy) => self.copy_selection(),
            Some(ClipboardOp::Cut) => self.cut_selection(),
            Some(ClipboardOp::Paste) => self.paste_clipboard(),
            None => {}
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::V)) {
            self.tools.tool = Tool::Select;
        }
        // The bracket keys step the size by one — the ceiling is the entry
        // box's sanity bound ([`MAX_STROKE_SIZE`]), NOT the slider's 100: a
        // manually entered 200 must survive a nudge (owner bug report
        // 2026-11-20 — the old slider-era clamp snapped it back to 100).
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::OpenBracket)) {
            self.tools.size = (self.tools.size - 1.0).clamp(1.0, MAX_STROKE_SIZE);
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::CloseBracket)) {
            self.tools.size = (self.tools.size + 1.0).clamp(1.0, MAX_STROKE_SIZE);
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
        if redo_pressed && self.layers.can_redo() {
            self.redo();
        }
        if self.layers.can_undo() && ctx.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::Z)) {
            self.undo();
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
            // text_tool.rs); the Eyedropper never paints; Select never
            // strokes.
            Tool::Eyedropper | Tool::Line | Tool::Shape | Tool::Text | Tool::Select => {
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
    pub(super) fn handle_pick(
        &mut self,
        ui: &egui::Ui,
        rect: Rect,
        response: &egui::Response,
        panning: bool,
    ) {
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
        let tool = self.tools.tool;
        let size = self.tools.size;
        // The gesture paints the ACTIVE layer's buffer; the display buffer
        // is re-blended for the same dirty region right after.
        let dirty = draw_segment(
            self.layers.active_mut().pixmap_mut(),
            tool,
            size,
            color,
            from,
            to,
        );
        if let Some(dirty) = dirty {
            self.composite_dirty(dirty.x, dirty.y, dirty.w, dirty.h);
            self.sync_texture(dirty.x, dirty.y, dirty.w, dirty.h);
            self.dirty = true;
        }
    }

    /// Rasterizes one frame's smoothed pieces into the ACTIVE layer's
    /// buffer in one batch (one dirty-rect composite + texture sync for
    /// the whole frame, however many pointer events it carried) and
    /// paints with the color captured at stroke START — the gesture's
    /// single history-node color. A mid-gesture tool switch abandons the
    /// paint (the pixels join the next node's snapshot, as before). `pin`
    /// is the release hook (see [`tools::smooth::rasterize_smooth_pieces`]).
    fn paint_smooth_pieces(&mut self, pieces: &[SmoothPiece], pin: Option<Pos2>) {
        let Some(color) = self.stroke_paint else {
            return;
        };
        let tool = self.tools.tool;
        let size = self.tools.size;
        let dirty = rasterize_smooth_pieces(
            self.layers.active_mut().pixmap_mut(),
            tool,
            size,
            color,
            pieces,
            &mut self.stroke_trail,
            pin,
        );
        if let Some(dirty) = dirty {
            self.composite_dirty(dirty.x, dirty.y, dirty.w, dirty.h);
            self.sync_texture(dirty.x, dirty.y, dirty.w, dirty.h);
            self.dirty = true;
        }
    }

    /// Rasterizes the smoothing window's closing piece (the last midpoint
    /// → the final raw point) so a released stroke carries its full path.
    /// The pencil's trail pins on the raw release point: the stroke ends
    /// exactly on the released pixel, like the classic Bresenham run.
    fn flush_stroke_close(&mut self) {
        if let Some(piece) = self.stroke_smooth.close() {
            let pin = match piece {
                SmoothPiece::Line { to, .. } | SmoothPiece::Quad { to, .. } => Some(to),
            };
            self.paint_smooth_pieces(std::slice::from_ref(&piece), pin);
        }
    }

    pub(super) fn handle_stroke(
        &mut self,
        ui: &egui::Ui,
        rect: Rect,
        response: &egui::Response,
        panning: bool,
    ) {
        if panning {
            self.is_stroking = false;
            self.stroke_smooth = SmoothStroke::new(Pos2::ZERO);
            self.stroke_trail = PencilTrail::default();
            self.stroke_button = None;
            self.stroke_paint = None;
            return;
        }

        let space = ui.input(|i| i.key_down(Key::Space));
        let (pointer_moves, primary_pressed, secondary_pressed, any_released) = ui.input(|i| {
            (
                pointer_move_positions(&i.events),
                i.pointer.button_pressed(PointerButton::Primary),
                i.pointer.button_pressed(PointerButton::Secondary),
                i.pointer.any_released(),
            )
        });

        if any_released {
            // One completed gesture = one node with the post-stroke canvas
            // as its PNG snapshot (the design concept's tldraw-style mark).
            // Owner request 2026-11-13: EVERY freehand tool records — pencil,
            // brush and eraser alike — on the active layer's tree. The color
            // was captured at stroke start — it is what the gesture painted
            // with.
            if self.is_stroking {
                // The smoothing window holds a closing piece; rasterize it
                // BEFORE the snapshot so the recorded canvas carries the
                // whole stroke.
                self.flush_stroke_close();
                if let Some(color) = self.stroke_paint.take() {
                    let kind = match self.tools.tool {
                        Tool::Pencil => Some(ActionKind::Pencil),
                        Tool::Brush => Some(ActionKind::Brush),
                        Tool::Eraser => Some(ActionKind::Eraser),
                        // A mid-gesture tool switch to a preview tool abandons
                        // the record; the pixels join the next node's snapshot.
                        // Select never freehand-strokes.
                        Tool::Eyedropper | Tool::Line | Tool::Shape | Tool::Text | Tool::Select => {
                            None
                        }
                    };
                    if let Some(kind) = kind {
                        self.record_active_action(kind, color, None);
                        let label = match self.tools.tool {
                            Tool::Pencil => "Pencil stroke",
                            Tool::Brush => "Brush stroke",
                            Tool::Eraser => "Eraser stroke",
                            Tool::Eyedropper
                            | Tool::Line
                            | Tool::Shape
                            | Tool::Text
                            | Tool::Select => "Stroke",
                        };
                        self.action_log.push(label);
                    }
                }
            }
            self.stroke_paint = None;
            self.is_stroking = false;
            self.stroke_smooth = SmoothStroke::new(Pos2::ZERO);
            self.stroke_trail = PencilTrail::default();
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
            if let (Some(button), Some(cursor)) = (button, ui.input(|i| i.pointer.latest_pos())) {
                self.stroke_button = Some(button);
                self.is_stroking = true;
                self.stroke_paint = Some(self.stroke_color());
                let pos = self.canvas_pos(rect, cursor);
                self.stroke_smooth = SmoothStroke::new(pos);
                // The press dot's pixel seeds the pencil trail's dedupe so
                // the first piece does not re-blend over the dot.
                self.stroke_trail = PencilTrail::starting_from(pencil_pixel(pos));
                self.paint_segment(pos, pos);
                ui.ctx().request_repaint();
                // The press frame's own coalesced moves are dropped: the
                // stroke starts AT the press point, like the touch dot says.
                return;
            }
        }

        // Drag: feed EVERY coalesced pointer move of the frame — one point
        // per frame (the old `latest_pos`) is what drew long straight runs
        // with sharp corners under a fast mouse.
        if self.is_stroking && !pointer_moves.is_empty() {
            let mut pieces = Vec::new();
            for screen in pointer_moves {
                let pos = self.canvas_pos(rect, screen);
                if let Some(piece) = self.stroke_smooth.push(pos) {
                    pieces.push(piece);
                }
            }
            if !pieces.is_empty() {
                self.paint_smooth_pieces(&pieces, None);
                ui.ctx().request_repaint();
            }
        }
    }
}

/// The frame's raw pointer stream: every `Event::PointerMoved` in event
/// order. egui coalesces the OS pointer stream per frame — `latest_pos`
/// alone would keep only the last one. Shared with the Select tool's
/// lasso drag ([`super::selection`]).
pub(super) fn pointer_move_positions(events: &[egui::Event]) -> Vec<Pos2> {
    events
        .iter()
        .filter_map(|event| match event {
            egui::Event::PointerMoved(pos) => Some(*pos),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests;
