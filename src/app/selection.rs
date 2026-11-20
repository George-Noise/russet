//! The Select tool's interaction (owner request 2026-11-20): region
//! gestures (rectangle / lasso per [`SelectMode`]), the add (Shift) /
//! subtract (Alt) modifiers, the move drag, and the clipboard ops
//! (copy / cut / paste / crop). A child module of `app`, so it can touch
//! `App`'s private fields; the PURE domain (regions, the mask, the
//! boundary loops, the pixel helpers) lives in [`crate::selection`], and
//! the crop orchestration (one `crop` node per layer) in
//! [`crate::layers::LayerStack::crop_all`] — the same per-layer shape the
//! Resize dialog commits (owner request: "чтобы не было дублирования кода
//! можно как-то это связать с тем кодом что делает resize").
//!
//! Gesture model (mirrors [`super::preview`]):
//! - press OUTSIDE the selection (or with a modifier held) starts a region
//!   drag — replace (no modifier), add (Shift), subtract (Alt), frozen at
//!   press like the Shape tools freeze their colors;
//! - press INSIDE the selection without modifiers starts a MOVE: the
//!   selected pixels lift into a floating buffer (erased from the layer;
//!   a FULL-layer backup rides along for the restore), and every frame of
//!   the drag re-draws them at the integer offset; release commits ONE
//!   move node, Esc / a tool switch / a pan restores the backup;
//! - release of a region gesture applies the region to the selection (a
//!   plain lasso click — fewer than 3 points — deselects);
//! - Esc with no pending gesture deselects.
//!
//! The selection is DOCUMENT state, not history: it survives tool
//! switches, is cleared on New/Open/resize/crop, and the overlay (the
//! dashed boundary loops — white underlay + black dashes) draws under
//! EVERY tool via [`App::draw_selection_overlay`]. The clipboard is
//! in-app (not the OS image clipboard) and survives document switches;
//! paste stamps immediately — a floating drag-after-paste is a follow-up
//! (DEBTS-functional).

use eframe::egui::{self, Color32, Key, Modifiers, PointerButton, Pos2, Rect, Shape, Stroke};

use super::App;
use crate::history_tree::ActionKind;
use crate::selection::{self, IRect, Region};
use crate::tools::SelectMode;

/// The pending gesture of the Select tool. Borrow discipline: every frame
/// step TAKES the gesture out (`std::mem::replace` with
/// [`SelectGesture::None`]), works on owned values, and puts it back — the
/// pixel work touches `self.layers`/`self.canvas` while the gesture rides
/// in local bindings.
pub(super) enum SelectGesture {
    None,
    /// A rectangle dragged from `start` to `current`.
    RectDrag {
        start: Pos2,
        current: Pos2,
        op: RegionOp,
    },
    /// A freeform lasso: the pointer path so far (closed on release).
    LassoDrag {
        points: Vec<Pos2>,
        op: RegionOp,
    },
    /// The selected pixels are lifted: `floating` holds the mask-shaped
    /// copy, `backup` the FULL pre-drag bytes of the active layer (the
    /// cancel restore), `mask` the lift's pixel truth, `origin` the bounds
    /// the lift happened at, `offset` the live integer shift. Every changed
    /// frame re-draws restore(origin) + erase(mask) + stamp(origin+offset).
    MoveDrag {
        floating: tiny_skia::Pixmap,
        backup: tiny_skia::Pixmap,
        mask: selection::Mask,
        origin: IRect,
        grab: Pos2,
        offset: (i64, i64),
    },
}

/// What the region gesture does to the existing selection at release —
/// frozen at press from the modifiers (owner request: add MORE regions to
/// the selection or remove regions, "также прямоугольником или лассо").
#[derive(Clone, Copy, PartialEq)]
pub(super) enum RegionOp {
    /// No modifier — the new region replaces the selection.
    Replace,
    /// Shift held — union.
    Add,
    /// Alt held — cut out.
    Subtract,
}

impl App {
    /// Press/drag/release handling of the Select tool (the dispatch lives
    /// in `draw_canvas`, the same pattern as the freehand and preview
    /// handlers).
    pub(super) fn handle_select_input(
        &mut self,
        ui: &egui::Ui,
        rect: Rect,
        response: &egui::Response,
        panning: bool,
    ) {
        let (cursor, primary_pressed, any_released, pointer_moves) = ui.input(|i| {
            (
                i.pointer.latest_pos(),
                i.pointer.button_pressed(PointerButton::Primary),
                i.pointer.any_released(),
                super::input::pointer_move_positions(&i.events),
            )
        });
        let (escape, shift, alt) = ui.input(|i| {
            (
                i.key_pressed(Key::Escape),
                i.modifiers.contains(Modifiers::SHIFT),
                i.modifiers.contains(Modifiers::ALT),
            )
        });
        let pos = cursor.map(|c| self.canvas_pos(rect, c));

        // Esc first: cancel a pending gesture, or deselect.
        if escape {
            if matches!(self.select_gesture, SelectGesture::MoveDrag { .. }) {
                self.cancel_select_gesture();
            } else {
                self.select_gesture = SelectGesture::None;
            }
            if !self.selection.is_empty() {
                self.selection.clear();
                self.action_log.push("Deselected");
            }
            return;
        }
        // A Space/middle-button pan takes the pointer: a region drag is
        // dropped (nothing was touched), a move restores the lift.
        if panning {
            self.cancel_select_gesture();
            return;
        }

        // The move drag consumes the frame: read the grab/offset, redraw at
        // the new integer offset, commit on release.
        if let SelectGesture::MoveDrag { grab, offset, .. } = &self.select_gesture {
            let (grab, last) = (*grab, *offset);
            let offset = match pos {
                Some(p) => ((p.x - grab.x).round() as i64, (p.y - grab.y).round() as i64),
                None => last,
            };
            if offset != last || any_released {
                self.redraw_moved(offset);
            }
            if any_released {
                self.commit_move(offset);
            }
            return;
        }
        // The rectangle drag: the latest pointer wins (the rect is the two
        // corners).
        if let SelectGesture::RectDrag { current, .. } = &mut self.select_gesture {
            if let Some(p) = pos {
                *current = p;
            }
            ui.ctx().request_repaint();
            if any_released {
                let state = std::mem::replace(&mut self.select_gesture, SelectGesture::None);
                if let SelectGesture::RectDrag { start, current, op } = state {
                    self.apply_region(
                        Region::Rect {
                            a: start,
                            b: current,
                        },
                        op,
                    );
                }
            }
            return;
        }
        // The lasso drag: EVERY coalesced move of the frame feeds the path,
        // one canvas point per at least a canvas pixel of travel (the same
        // frame batching the freehand stroke rides). The canvas conversion
        // runs BEFORE the gesture borrow (canvas_pos reads `self`).
        let canvas_moves: Vec<Pos2> = pointer_moves
            .iter()
            .map(|screen| self.canvas_pos(rect, *screen))
            .collect();
        if let SelectGesture::LassoDrag { points, .. } = &mut self.select_gesture {
            for p in canvas_moves {
                if points.last().is_none_or(|last| last.distance(p) >= 1.0) {
                    points.push(p);
                }
            }
            ui.ctx().request_repaint();
            if any_released {
                let state = std::mem::replace(&mut self.select_gesture, SelectGesture::None);
                if let SelectGesture::LassoDrag { points, op } = state {
                    self.apply_region(Region::Lasso { points }, op);
                }
            }
            return;
        }

        // No pending gesture: a press routes to a move or a region drag.
        if !primary_pressed || !response.hovered() {
            return;
        }
        let Some(p) = pos else {
            return;
        };
        // The modifiers win over the move: pressing inside the selection
        // WITH Shift/Alt still edits the region set.
        let op = if shift {
            RegionOp::Add
        } else if alt {
            RegionOp::Subtract
        } else {
            RegionOp::Replace
        };
        if op == RegionOp::Replace && self.selection.contains(p) {
            self.start_move(p);
            return;
        }
        match self.tools.select_mode {
            SelectMode::Rect => {
                self.select_gesture = SelectGesture::RectDrag {
                    start: p,
                    current: p,
                    op,
                };
            }
            SelectMode::Lasso => {
                self.select_gesture = SelectGesture::LassoDrag {
                    points: vec![p],
                    op,
                };
            }
        }
    }

    /// Applies a finished region gesture to the selection. A replace with
    /// an EMPTY region (a plain lasso click — fewer than 3 points)
    /// deselects; a rect click selects its one pixel.
    fn apply_region(&mut self, region: Region, op: RegionOp) {
        let (width, height) = (self.canvas.width, self.canvas.height);
        let empty = region.pixel_rect().is_none();
        match op {
            RegionOp::Replace => {
                if empty {
                    self.selection.replace(None, width, height);
                    self.action_log.push("Deselected");
                } else {
                    self.selection.replace(Some(region), width, height);
                    self.action_log.push("Selection replaced");
                }
            }
            RegionOp::Add => {
                if !empty {
                    self.selection.add(region, width, height);
                    self.action_log.push("Selection added");
                }
            }
            RegionOp::Subtract => {
                if !empty {
                    self.selection.subtract(region, width, height);
                    self.action_log.push("Selection subtracted");
                }
            }
        }
    }

    /// Starts the move: lift the selected pixels out of the active layer
    /// (erased from it — the full-layer backup rides along for the
    /// restore), and draw them back at the zero offset. The drag redraw
    /// and the commit live in [`App::redraw_moved`] / [`App::commit_move`].
    fn start_move(&mut self, grab: Pos2) {
        let Some(mask) = self
            .selection
            .build_mask(self.canvas.width, self.canvas.height)
        else {
            return;
        };
        let Some(origin) = self.selection.bounds() else {
            return;
        };
        let Some(floating) = selection::extract_masked(self.layers.active().pixmap(), &mask) else {
            return;
        };
        let backup = self.layers.active().pixmap().clone();
        // The lift: erase the selected pixels (one composite + texture sync
        // over the bounds) — the floating buffer rides in the gesture.
        selection::erase_masked(self.layers.active_mut().pixmap_mut(), &mask);
        self.composite_dirty(
            origin.x.max(0) as u32,
            origin.y.max(0) as u32,
            origin.w,
            origin.h,
        );
        self.sync_texture(
            origin.x.max(0) as u32,
            origin.y.max(0) as u32,
            origin.w,
            origin.h,
        );
        self.dirty = true;
        self.select_gesture = SelectGesture::MoveDrag {
            floating,
            backup,
            mask,
            origin,
            grab,
            offset: (0, 0),
        };
        self.ctx.request_repaint();
    }

    /// One frame of a moved selection: restore the pre-drag bytes, erase
    /// the lifted pixels, stamp the floating at the new offset — all over
    /// the union rect of the old and new positions. The gesture is taken
    /// out and put back (the pixel work borrows the layer).
    fn redraw_moved(&mut self, offset: (i64, i64)) {
        let state = std::mem::replace(&mut self.select_gesture, SelectGesture::None);
        let SelectGesture::MoveDrag {
            floating,
            backup,
            mask,
            origin,
            grab,
            offset: _,
        } = state
        else {
            self.select_gesture = state;
            return;
        };
        let moved = IRect {
            x: origin.x + offset.0,
            y: origin.y + offset.1,
            w: origin.w,
            h: origin.h,
        };
        selection::restore_region(self.layers.active_mut().pixmap_mut(), &backup, origin);
        selection::erase_masked(self.layers.active_mut().pixmap_mut(), &mask);
        selection::stamp(
            self.layers.active_mut().pixmap_mut(),
            &floating,
            moved.x,
            moved.y,
        );
        if let Some(dirty) = origin
            .union(moved)
            .clamped(self.canvas.width, self.canvas.height)
        {
            self.composite_dirty(dirty.x as u32, dirty.y as u32, dirty.w, dirty.h);
            self.sync_texture(dirty.x as u32, dirty.y as u32, dirty.w, dirty.h);
            self.dirty = true;
        }
        self.select_gesture = SelectGesture::MoveDrag {
            floating,
            backup,
            mask,
            origin,
            grab,
            offset,
        };
        self.ctx.request_repaint();
    }

    /// Commits a released move: the selection translates by the drag's
    /// integer offset and ONE move node records the post-move layer (the
    /// pixels already sit at the final position).
    fn commit_move(&mut self, offset: (i64, i64)) {
        self.select_gesture = SelectGesture::None;
        let (width, height) = (self.canvas.width, self.canvas.height);
        self.selection.translate(offset.0, offset.1, width, height);
        self.record_active_action(ActionKind::Move, Color32::WHITE, None);
        self.action_log.push("Selection moved");
    }

    /// Drops the pending gesture, restoring a lifted move (the ONLY
    /// gesture state that touched pixels): the full-layer backup goes back
    /// and one composite pass refreshes the buffer. Tool switches, pans,
    /// resizes and document resets go through here.
    pub(super) fn cancel_select_gesture(&mut self) {
        let state = std::mem::replace(&mut self.select_gesture, SelectGesture::None);
        let SelectGesture::MoveDrag { backup, .. } = state else {
            return;
        };
        let (width, height) = (self.canvas.width, self.canvas.height);
        selection::restore_region(
            self.layers.active_mut().pixmap_mut(),
            &backup,
            IRect {
                x: 0,
                y: 0,
                w: width,
                h: height,
            },
        );
        self.refresh_composite();
        self.dirty = true;
    }

    /// Copy: the selected pixels of the ACTIVE layer into the in-app
    /// clipboard (no history node — the pixels do not change).
    pub(super) fn copy_selection(&mut self) {
        let Some(mask) = self
            .selection
            .build_mask(self.canvas.width, self.canvas.height)
        else {
            return;
        };
        let Some(clip) = selection::extract_masked(self.layers.active().pixmap(), &mask) else {
            return;
        };
        self.clipboard = Some(clip);
        self.action_log.push("Copy");
    }

    /// Cut: copy + erase the selected pixels (ONE cut node — the post-cut
    /// buffer becomes the snapshot). The selection stays (Paint.NET-style).
    pub(super) fn cut_selection(&mut self) {
        let Some(mask) = self
            .selection
            .build_mask(self.canvas.width, self.canvas.height)
        else {
            return;
        };
        let Some(clip) = selection::extract_masked(self.layers.active().pixmap(), &mask) else {
            return;
        };
        self.clipboard = Some(clip);
        let bounds = self.selection.bounds();
        selection::erase_masked(self.layers.active_mut().pixmap_mut(), &mask);
        if let Some(rect) = bounds {
            self.composite_dirty(rect.x.max(0) as u32, rect.y.max(0) as u32, rect.w, rect.h);
            self.sync_texture(rect.x.max(0) as u32, rect.y.max(0) as u32, rect.w, rect.h);
        }
        self.dirty = true;
        self.record_active_action(ActionKind::Cut, Color32::WHITE, None);
        self.action_log.push("Cut");
    }

    /// Paste: stamp the clipboard into the ACTIVE layer, centered in the
    /// viewport (clamped inside the canvas), and select the pasted area —
    /// the user can move it right away with the same tool. The clipboard
    /// SURVIVES the paste (repeated Ctrl+V works).
    pub(super) fn paste_clipboard(&mut self) {
        let Some(clip) = self.clipboard.clone() else {
            return;
        };
        let content = self.ctx.content_rect();
        let center = self.canvas_pos(self.canvas_rect(content), content.center());
        let x = (center.x - clip.width() as f32 / 2.0).round() as i64;
        let y = (center.y - clip.height() as f32 / 2.0).round() as i64;
        let x = x.clamp(0, (self.canvas.width as i64 - clip.width() as i64).max(0));
        let y = y.clamp(0, (self.canvas.height as i64 - clip.height() as i64).max(0));
        selection::stamp(self.layers.active_mut().pixmap_mut(), &clip, x, y);
        let pasted = IRect {
            x,
            y,
            w: clip.width(),
            h: clip.height(),
        };
        if let Some(clamped) = pasted.clamped(self.canvas.width, self.canvas.height) {
            self.composite_dirty(clamped.x as u32, clamped.y as u32, clamped.w, clamped.h);
            self.sync_texture(clamped.x as u32, clamped.y as u32, clamped.w, clamped.h);
        }
        self.dirty = true;
        self.selection.replace(
            Some(Region::Rect {
                a: Pos2::new(x as f32, y as f32),
                b: Pos2::new(
                    (x + clip.width() as i64) as f32,
                    (y + clip.height() as i64) as f32,
                ),
            }),
            self.canvas.width,
            self.canvas.height,
        );
        self.record_active_action(ActionKind::Paste, Color32::WHITE, None);
        self.action_log.push("Paste");
    }

    /// Crop: the canvas (and EVERY layer) becomes the selection's pixel
    /// bounds — the Resize machinery's per-layer crop (`crop_all`, one
    /// node per layer like a resize commit), then the display buffer
    /// rebuilds, the pending gestures drop (their coordinates shift) and
    /// the selection clears (its bounds shift off-canvas), exactly like
    /// `apply_resize`. Zoom and pan do not change.
    pub(super) fn crop_to_selection(&mut self) {
        let Some(rect) = self.selection.bounds() else {
            return;
        };
        self.cancel_select_gesture();
        self.layers.crop_all(rect.x, rect.y, rect.w, rect.h);
        self.cancel_preview();
        self.cancel_text_edit();
        self.selection.clear();
        self.canvas.resize_canvas(rect.w, rect.h);
        self.refresh_composite();
        self.dirty = true;
        self.action_log
            .push(format!("Canvas cropped to {}×{}", rect.w, rect.h));
    }

    /// The dashed selection overlay (white underlay + black dashes): the
    /// committed selection's boundary loops under EVERY tool, plus the
    /// pending gesture's outline. The move drag shifts the loops by its
    /// live offset. Never touches the Pixmap.
    pub(super) fn draw_selection_overlay(&self, painter: &egui::Painter, canvas_rect: Rect) {
        let ppp = self.ctx.pixels_per_point();
        // The dash rhythm lives in SCREEN points — the ants read the same
        // at every zoom.
        let white = Stroke::new(1.0, Color32::from_white_alpha(230));
        let black = Stroke::new(1.0, Color32::from_black_alpha(230));
        let (dash, gap) = (4.0 / ppp.max(0.5), 3.0 / ppp.max(0.5));
        let move_offset = match &self.select_gesture {
            SelectGesture::MoveDrag { offset, .. } => egui::vec2(offset.0 as f32, offset.1 as f32),
            _ => egui::Vec2::ZERO,
        };
        let draw_points = |painter: &egui::Painter, points: Vec<Pos2>| {
            if points.len() < 2 {
                return;
            }
            painter.add(Shape::closed_line(points.clone(), white));
            let mut dashed_path = points.clone();
            dashed_path.push(points[0]); // dashed_line is open — close it
            for shape in Shape::dashed_line(&dashed_path, black, dash, gap) {
                painter.add(shape);
            }
        };
        let to_screen = |p: Pos2| self.screen_pos(canvas_rect, p);
        // The committed selection's loops, shifted live during a move drag.
        for loop_points in self.selection.ants() {
            let screen: Vec<Pos2> = loop_points
                .iter()
                .map(|p| to_screen(Pos2::new(p.x + move_offset.x, p.y + move_offset.y)))
                .collect();
            draw_points(painter, screen);
        }
        // The pending region gesture's outline.
        match &self.select_gesture {
            SelectGesture::RectDrag { start, current, .. } => {
                let a = to_screen(*start);
                let b = to_screen(*current);
                let rect = Rect::from_two_pos(a, b);
                draw_points(
                    painter,
                    vec![
                        rect.left_top(),
                        rect.right_top(),
                        rect.right_bottom(),
                        rect.left_bottom(),
                    ],
                );
            }
            SelectGesture::LassoDrag { points, .. } => {
                let screen: Vec<Pos2> = points.iter().map(|p| to_screen(*p)).collect();
                draw_points(painter, screen);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
