//! The preview gestures' COMMITS (pure code motion, owner-approved
//! 2026-11-21 file-guard split): the drag commit ([`commit_drag`]) and the
//! polyline commit ([`finish_polyline`]) — the only places a preview
//! gesture writes the canvas (through the tools' rasterizers, so an active
//! selection clips these commits too) and records history. Methods on
//! [`App`]; the input FSM stayed in the parent.

use eframe::egui::{Color32, Pos2};

use super::{App, DragKind, PreviewState, diameter_rect, shape_record_colors};
use crate::history_tree::ActionKind;
use crate::tools::{ShapeKind, ShapeOutline, rasterize_polyline, rasterize_shape};

impl App {
    /// Commits a finished drag: straight line, rectangle or ellipse. The
    /// committed pixels land on the ACTIVE layer and the post-commit
    /// buffer becomes the node's PNG snapshot (owner request 2026-11-13).
    #[allow(clippy::too_many_arguments)] // the flat paint list the Drag state carries (same as layers_window.rs)
    pub(super) fn commit_drag(
        &mut self,
        kind: DragKind,
        start: Pos2,
        current: Pos2,
        outline: Option<Color32>,
        fill: Option<Color32>,
        width: f32,
        outline_pos: ShapeOutline,
        diameter: bool,
        clip: Option<&crate::selection::Mask>,
    ) {
        match kind {
            DragKind::Line => {
                let Some(color) = outline else {
                    return;
                };
                // A click without movement draws a dot (round cap).
                let dirty = rasterize_polyline(
                    self.layers.active_mut().pixmap_mut(),
                    &[start, current],
                    width,
                    color,
                    clip,
                );
                if let Some(dirty) = dirty {
                    self.composite_dirty(dirty.x, dirty.y, dirty.w, dirty.h);
                    self.sync_texture(dirty.x, dirty.y, dirty.w, dirty.h);
                    self.dirty = true;
                }
                self.record_active_action(ActionKind::Line, color, None);
                self.action_log.push("Line");
            }
            DragKind::Shape(shape_kind) => {
                let label = match shape_kind {
                    ShapeKind::Rectangle => "Rectangle",
                    ShapeKind::Ellipse => "Ellipse",
                };
                // TESTING (owner request 2026-11-21): a diameter-circle
                // gesture commits the circle's bounding SQUARE (the circle
                // through both raw drag points), not the corner-to-corner
                // rect — the same geometry the overlay previewed.
                let (from, to) = if diameter {
                    let rect = diameter_rect(start, current);
                    (rect.min, rect.max)
                } else {
                    (start, current)
                };
                let dirty = rasterize_shape(
                    self.layers.active_mut().pixmap_mut(),
                    shape_kind,
                    from,
                    to,
                    width,
                    outline,
                    fill,
                    outline_pos,
                    clip,
                );
                if let Some(dirty) = dirty {
                    self.composite_dirty(dirty.x, dirty.y, dirty.w, dirty.h);
                    self.sync_texture(dirty.x, dirty.y, dirty.w, dirty.h);
                    self.dirty = true;
                }
                let (color, color2) = shape_record_colors(outline, fill, self.tools.primary);
                self.record_active_action(ActionKind::Shape, color, color2);
                self.action_log.push(label);
            }
        }
    }

    /// Commits the pending polyline (curve) and logs it.
    pub(super) fn finish_polyline(&mut self) {
        let state = std::mem::take(&mut self.preview);
        let PreviewState::Polyline {
            mut points,
            color,
            width,
            clip,
            ..
        } = state
        else {
            return;
        };
        // Double-click commits after adding a duplicate vertex — drop it.
        points.dedup();
        if points.is_empty() {
            return;
        }
        let dirty = rasterize_polyline(
            self.layers.active_mut().pixmap_mut(),
            &points,
            width,
            color,
            clip.as_ref(),
        );
        if let Some(dirty) = dirty {
            self.composite_dirty(dirty.x, dirty.y, dirty.w, dirty.h);
            self.sync_texture(dirty.x, dirty.y, dirty.w, dirty.h);
            self.dirty = true;
        }
        self.record_active_action(ActionKind::Line, color, None);
        self.action_log.push("Curve");
    }
}
