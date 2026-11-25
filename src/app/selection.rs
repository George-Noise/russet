//! The Select tool's interaction (owner request 2026-11-20): region
//! gestures (rectangle / lasso per [`SelectMode`]), the add (Shift) /
//! subtract (Alt) modifiers, the move drag, and the clipboard ops
//! (copy / cut / paste / crop). A child module of `app`, so it can touch
//! `App`'s private fields; the PURE domain (regions, the mask, the
//! boundary loops, the pixel helpers) lives in [`crate::selection`], and
//! the crop orchestration (one `crop` node per layer) in
//! [`crate::layers::LayerStack::crop_all`] — the same per-layer shape the
//! Resize dialog commits (owner request: "so there is no code duplication —
//! tie it somehow to the code that does the resize").
//!
//! Gesture model (mirrors [`super::preview`]):
//! - press OUTSIDE the selection (or with a modifier held) starts a region
//!   drag — replace (no modifier), add (Shift), subtract (Alt), frozen at
//!   press like the Shape tools freeze their colors; in the WAND mode
//!   (owner request 2026-11-23: the Paint.NET click-select) there is no
//!   drag — the press floods the clicked color band on the active layer
//!   and applies the reached pixels as a [`Region::Wand`] at once;
//! - press INSIDE the selection without modifiers starts a MOVE: the
//!   selected pixels lift into a floating buffer (erased from the layer —
//!   the layer's ONLY write until the release) and follow the pointer as
//!   a GPU overlay ABOVE the canvas, visible past the canvas bounds
//!   (owner redesign 2026-11-20); release commits the stamp at the final
//!   offset with ONE move node, Esc / a tool switch / a pan restores the
//!   backup (the wand mode re-wands instead — the move stays a Rect/Lasso
//!   affordance);
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
    /// cancel restore), `origin` the bounds the lift happened at, `offset`
    /// the live integer shift. The LAYER is
    /// written exactly TWICE total — the erase at the lift and the stamp
    /// at the release; during the drag the floating pixels ride as a GPU
    /// overlay ABOVE the canvas, visible past the canvas bounds (owner
    /// redesign 2026-11-20: the per-frame restore/erase/stamp painted
    /// unclear artifacts, and the dragged part must be a separate element
    /// from the canvas until release).
    MoveDrag {
        floating: tiny_skia::Pixmap,
        backup: tiny_skia::Pixmap,
        origin: IRect,
        grab: Pos2,
        offset: (i64, i64),
        /// The floating pixels as a GPU texture (premultiplied sRGBA),
        /// drawn by [`App::draw_move_overlay`]. The gesture keeps the
        /// handle alive for as long as the id is sampled (the
        /// texture-dies-with-handle rule) and drops it at the commit.
        overlay: egui::TextureHandle,
    },
}

impl SelectGesture {
    /// A region or move gesture is pending: the side-button undo/redo
    /// guard ([`super::mouse_side`]) swallows its presses so a mid-drag
    /// undo cannot corrupt the pending commit (a MoveDrag also holds the
    /// lifted pixels' invariant).
    pub(super) fn is_active(&self) -> bool {
        !matches!(self, SelectGesture::None)
    }
}

/// What the region gesture does to the existing selection at release —
/// frozen at press from the modifiers (owner request: add MORE regions to
/// the selection or remove regions, "also by rectangle or lasso").
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
    /// The selection mask snapshot a painting gesture takes at start (owner
    /// request 2026-11-21: an active selection confines every canvas-
    /// changing tool). `None` = no selection — the tool paints unclipped.
    pub(super) fn selection_clip(&self) -> Option<crate::selection::Mask> {
        self.selection
            .build_mask(self.canvas.width, self.canvas.height)
    }

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

        // The move drag consumes the frame: track the offset (the canvas
        // is not written during the drag — the overlay carries the pixels),
        // commit the stamp on release.
        if let SelectGesture::MoveDrag { grab, offset, .. } = &self.select_gesture {
            let (grab, last) = (*grab, *offset);
            let offset = match pos {
                Some(p) => ((p.x - grab.x).round() as i64, (p.y - grab.y).round() as i64),
                None => last,
            };
            if any_released {
                self.commit_move(offset);
            } else if offset != last {
                self.update_move(offset);
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
            SelectMode::Wand => {
                // The wand is instant (owner request 2026-11-23: the
                // Paint.NET click-select): the flood runs AT PRESS, no drag
                // state. Pressing inside the selection still re-wands (the
                // move gesture stays a Rect/Lasso-mode affordance).
                let region = self.wand_region(p);
                self.apply_region(region, op);
            }
        }
    }

    /// Floods the clicked color band on the ACTIVE LAYER (the layer the
    /// fill paints — not the composite) and packs the reached pixels as a
    /// [`Region::Wand`]. A click off the canvas or an empty flood yields an
    /// empty pixel set — the same degenerate rule the lasso click follows
    /// (a replace with it deselects).
    fn wand_region(&self, p: Pos2) -> Region {
        let seed = (p.x.floor() as i64, p.y.floor() as i64);
        let tolerance = crate::selection::tolerance_from_percent(self.tools.flood_tolerance_pct);
        let pixels = {
            let pm = self.layers.active().pixmap();
            let (width, height) = (pm.width(), pm.height());
            selection::flood_pixels(width, height, seed, tolerance, |px, py| {
                crate::tools::pick_color(pm, px as u32, py as u32)
            })
        };
        Region::Wand { pixels }
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
        // over the bounds) — the ONLY layer write until the release. The
        // floating buffer rides as a GPU overlay texture (kept alive by the
        // gesture) instead of being re-stamped into the layer per frame.
        selection::erase_masked(self.layers.active_mut().pixmap_mut(), &mask);
        let overlay = self.ctx.load_texture(
            "select_move_overlay",
            egui::ColorImage::from_rgba_premultiplied(
                [floating.width() as usize, floating.height() as usize],
                floating.data(),
            ),
            self.canvas_texture_options(),
        );
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
            origin,
            grab,
            offset: (0, 0),
            overlay,
        };
        self.ctx.request_repaint();
    }

    /// One frame of a moved selection: ONLY the integer offset updates —
    /// the layer is not written (owner redesign 2026-11-20: the drag must
    /// not paint onto the canvas at all; the pixels follow the pointer as
    /// the overlay element).
    fn update_move(&mut self, offset: (i64, i64)) {
        if let SelectGesture::MoveDrag { offset: last, .. } = &mut self.select_gesture
            && *last != offset
        {
            *last = offset;
            self.ctx.request_repaint();
        }
    }

    /// Commits a released move: stamp the floating at the final offset
    /// (the layer's second and last write — the drag itself painted
    /// nothing), refresh that region, translate the selection, and record
    /// ONE move node with the post-move layer (the pixels already sit at
    /// the final position).
    fn commit_move(&mut self, offset: (i64, i64)) {
        let state = std::mem::replace(&mut self.select_gesture, SelectGesture::None);
        let SelectGesture::MoveDrag {
            floating, origin, ..
        } = state
        else {
            return;
        };
        let moved = IRect {
            x: origin.x + offset.0,
            y: origin.y + offset.1,
            w: origin.w,
            h: origin.h,
        };
        selection::stamp(
            self.layers.active_mut().pixmap_mut(),
            &floating,
            moved.x,
            moved.y,
        );
        if let Some(dirty) = moved.clamped(self.canvas.width, self.canvas.height) {
            self.composite_dirty(dirty.x as u32, dirty.y as u32, dirty.w, dirty.h);
            self.sync_texture(dirty.x as u32, dirty.y as u32, dirty.w, dirty.h);
        }
        self.dirty = true;
        let (width, height) = (self.canvas.width, self.canvas.height);
        self.selection.translate(offset.0, offset.1, width, height);
        // Owner bug report 2026-11-20: a move that lands PARTIALLY
        // off-canvas must shrink the selection to the pixels that actually
        // survived (the stamp clips them) — the un-clipped region list
        // would restore the full shape on the next move, because
        // `contains` (the move hit test) and `translate` work on the ops,
        // not the derived views.
        self.selection.clip_to_canvas(width, height);
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

    /// Select All (owner request 2026-11-24, the Ctrl+A hotkey in
    /// [`super::input`] and the Edit menu item): one Rect region covering
    /// the WHOLE canvas replaces the selection. Every layer in russet is
    /// canvas-sized (a resize re-fits all of them), so the full-canvas
    /// rect IS "everything on the current layer" — the copy/cut/move ops
    /// read the ACTIVE layer through the mask. A replace (not an add):
    /// pressing Ctrl+A twice lands the same rect, never a doubled op
    /// list.
    pub(super) fn select_all(&mut self) {
        let (width, height) = (self.canvas.width, self.canvas.height);
        self.selection.replace(
            Some(Region::Rect {
                a: Pos2::new(0.0, 0.0),
                b: Pos2::new(width as f32, height as f32),
            }),
            width,
            height,
        );
        self.action_log.push("Select all");
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
    /// pending gesture's outline — all clipped to the canvas panel. While
    /// a move drag is live the committed loops are drawn by
    /// [`App::draw_move_overlay`][Self::draw_move_overlay] instead, on the
    /// foreground layer, so they follow the floating pixels past the
    /// canvas bounds. Never touches the Pixmap.
    pub(super) fn draw_selection_overlay(&self, painter: &egui::Painter, canvas_rect: Rect) {
        let ppp = self.ctx.pixels_per_point();
        let to_screen = |p: Pos2| self.screen_pos(canvas_rect, p);
        if !matches!(self.select_gesture, SelectGesture::MoveDrag { .. }) {
            // The committed selection's loops — unless a move drag carries
            // them on the foreground overlay.
            for loop_points in self.selection.ants() {
                let screen: Vec<Pos2> = loop_points.iter().map(|p| to_screen(*p)).collect();
                draw_ant_loop(painter, &screen, ppp);
            }
        }
        // The pending region gesture's outline.
        match &self.select_gesture {
            SelectGesture::RectDrag { start, current, .. } => {
                let a = to_screen(*start);
                let b = to_screen(*current);
                let rect = Rect::from_two_pos(a, b);
                draw_ant_loop(
                    painter,
                    &[
                        rect.left_top(),
                        rect.right_top(),
                        rect.right_bottom(),
                        rect.left_bottom(),
                    ],
                    ppp,
                );
            }
            SelectGesture::LassoDrag { points, .. } => {
                let screen: Vec<Pos2> = points.iter().map(|p| to_screen(*p)).collect();
                draw_ant_loop(painter, &screen, ppp);
            }
            _ => {}
        }
    }

    /// The move drag's floating pixels as a SEPARATE element ABOVE
    /// everything (owner redesign 2026-11-20): the dragged part is an
    /// overlay, visible past the canvas bounds until release, and the
    /// canvas texture is not touched while it moves. Painted on
    /// `Order::Foreground` with the whole-screen clip: the floating
    /// texture at the dragged offset plus the selection's dashed loops
    /// riding with it.
    pub(super) fn draw_move_overlay(&self, canvas_rect: Rect) {
        let SelectGesture::MoveDrag {
            overlay, offset, ..
        } = &self.select_gesture
        else {
            return;
        };
        let painter = egui::Painter::new(
            self.ctx.clone(),
            egui::LayerId::new(
                egui::Order::Foreground,
                // Id::unique hashes deterministically (fixed seeds) — a
                // stable layer id across frames; `Id::new` is deprecated
                // in egui 0.37.
                egui::Id::unique("select_move_overlay"),
            ),
            self.ctx.viewport_rect(),
        );
        let to_screen = |p: Pos2| {
            self.screen_pos(
                canvas_rect,
                Pos2::new(p.x + offset.0 as f32, p.y + offset.1 as f32),
            )
        };
        for loop_points in self.selection.ants() {
            let screen: Vec<Pos2> = loop_points.iter().map(|p| to_screen(*p)).collect();
            draw_ant_loop(&painter, &screen, self.ctx.pixels_per_point());
        }
        let rect = self.move_overlay_rect(canvas_rect);
        painter.image(
            overlay.id(),
            rect,
            Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    }

    /// The screen rect the floating pixels are drawn at for the CURRENT
    /// drag offset (a helper so the overlay geometry is unit-testable —
    /// the owner's "visible past the canvas bounds" promise). `Rect::NOTHING`
    /// when no move drag is live.
    fn move_overlay_rect(&self, canvas_rect: Rect) -> Rect {
        let SelectGesture::MoveDrag { origin, offset, .. } = &self.select_gesture else {
            return Rect::NOTHING;
        };
        let min = self.screen_pos(
            canvas_rect,
            Pos2::new((origin.x + offset.0) as f32, (origin.y + offset.1) as f32),
        );
        let scale = self.zoom / self.ctx.pixels_per_point();
        Rect::from_min_size(
            min,
            egui::vec2(origin.w as f32 * scale, origin.h as f32 * scale),
        )
    }
}

/// One dashed ant loop (white underlay + black dashes) over SCREEN points.
/// The dash rhythm lives in SCREEN points — the ants read the same at
/// every zoom. Shared by the canvas-painter overlay and the foreground
/// move overlay.
fn draw_ant_loop(painter: &egui::Painter, points: &[Pos2], ppp: f32) {
    if points.len() < 2 {
        return;
    }
    let white = Stroke::new(1.0, Color32::from_white_alpha(230));
    let black = Stroke::new(1.0, Color32::from_black_alpha(230));
    let (dash, gap) = (4.0 / ppp.max(0.5), 3.0 / ppp.max(0.5));
    painter.add(Shape::closed_line(points.to_vec(), white));
    let mut dashed_path = points.to_vec();
    dashed_path.push(points[0]); // dashed_line is open — close it
    for shape in Shape::dashed_line(&dashed_path, black, dash, gap) {
        painter.add(shape);
    }
}

#[cfg(test)]
mod tests;
