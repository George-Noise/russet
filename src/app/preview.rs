//! Interaction and overlay preview for the "preview-then-commit" tools
//! (owner request 2026-11-10): the Line tool (straight drag / curve
//! polyline) and the Shape tool (rectangle / ellipse drag). Nothing touches
//! the canvas `Pixmap` until the gesture is committed, so a cancelled
//! preview never leaves a history snapshot behind. A child module of `app`,
//! so it can touch `App`'s private fields; `app.rs` stays the orchestrator.
//! The PURE Shape color rules (which wells a gesture uses, what the commit
//! records) live in the child [`colors`]. Holding `Shift` during a Shape
//! drag constrains it to a proportional square/circle (Paint.NET-style —
//! owner request 2026-11-16), live, through the pure [`proportional_current`];
//! the TESTING diameter-circle rule (owner request 2026-11-21) replaces that
//! for the Ellipse kind through the pure [`diameter_rect`].

mod colors;

use colors::{shape_gesture_wells, shape_record_colors};

use eframe::egui::{
    self, Color32, Key, PointerButton, Pos2, Rect, Shape, Stroke, StrokeKind, pos2,
};
use std::f32::consts::TAU;

use crate::history_tree::ActionKind;
use crate::tools::{LineMode, ShapeKind, ShapeOutline, Tool, rasterize_polyline, rasterize_shape};

use super::App;

/// Pending gesture of the preview tools. Coordinates are canvas points.
#[derive(Default)]
pub(super) enum PreviewState {
    #[default]
    None,
    /// A straight line or a shape dragged from `start` to `current`.
    /// A shape stores the colors it will commit with: `outline` is the
    /// primary-well stroke color, `fill` the secondary-well fill color;
    /// `None` = the paint radio (owner request 2026-11-20) leaves that
    /// well out. A straight line always has `outline` set and `fill = None`.
    /// `outline_pos` carries the outline placement the shape commits with
    /// (owner request 2026-11-20; lines never outline, so they store the
    /// default) — stored like the colors, so a mid-drag panel switch
    /// cannot change what the release commits. `diameter` carries the
    /// TESTING diameter-circle rule (owner request 2026-11-21) the same
    /// way, refreshed live every frame like `current`: when set, the two
    /// raw drag points are the committed circle's diameter endpoints, so
    /// both the overlay and the commit draw the circle's bounding square
    /// ([`diameter_rect`]) instead of the corner-to-corner rect.
    Drag {
        kind: DragKind,
        start: Pos2,
        current: Pos2,
        outline: Option<Color32>,
        fill: Option<Color32>,
        width: f32,
        outline_pos: ShapeOutline,
        diameter: bool,
        /// The selection mask snapshot at drag start (owner request
        /// 2026-11-21): the commit transfers only masked pixels — the
        /// selection confines every painting tool. The selection cannot
        /// change mid-drag (the pointer is busy), so the snapshot stays
        /// exact.
        clip: Option<crate::selection::Mask>,
    },
    /// A curve-mode polyline: `points` are confirmed vertices, the rubber
    /// band follows the pointer; `mode` is remembered so that switching the
    /// Line mode mid-polyline cancels it.
    Polyline {
        points: Vec<Pos2>,
        rubber: Option<Pos2>,
        color: Color32,
        width: f32,
        mode: LineMode,
        /// The selection mask snapshot at gesture start (owner request
        /// 2026-11-21): the commit transfers only masked pixels.
        clip: Option<crate::selection::Mask>,
    },
}

/// What a [`PreviewState::Drag`] is drawing.
#[derive(Clone, Copy)]
pub(super) enum DragKind {
    Line,
    Shape(ShapeKind),
}

/// The free drag `current` point pulled onto the proportional square that
/// grows from `start` along the DOMINANT axis (Paint.NET's Shift-constrained
/// shapes — owner request 2026-11-16): the larger of the two deltas wins as
/// the side length, and each axis keeps its drag direction, so the square
/// always reaches toward the pointer from the anchor corner. A strictly
/// one-sided drag (one delta zero) still yields a square: the zero axis
/// extends by the full side in the down/right direction (Rust's
/// `f32::signum` maps `+0.0` to `1.0`), matching Paint.NET — a Shift drag
/// never produces a degenerate shape once the pointer has moved. Pure — no
/// egui, no `App` — so the rule stays unit-testable.
fn proportional_current(start: Pos2, current: Pos2) -> Pos2 {
    let dx = current.x - start.x;
    let dy = current.y - start.y;
    let side = dx.abs().max(dy.abs());
    pos2(start.x + side * dx.signum(), start.y + side * dy.signum())
}

/// The TESTING diameter-circle rule (owner request 2026-11-21): with the
/// Ellipse kind and `Shift` held (the panel's feature checkbox on), the
/// drag's two points do NOT corner the shape — they both lie ON the circle,
/// and the segment between them is its DIAMETER: the center is the segment's
/// midpoint and the radius is half its length. The returned rect is the
/// circle's bounding square — the exact opposite corners
/// [`crate::tools::rasterize_shape`] inscribes the ellipse into, so the
/// committed boundary passes through both drag points. Pure — no egui
/// state, no `App` — so the rule stays unit-testable.
fn diameter_rect(start: Pos2, current: Pos2) -> Rect {
    let center = Pos2::new((start.x + current.x) * 0.5, (start.y + current.y) * 0.5);
    let radius = (current - start).length() * 0.5;
    Rect::from_center_size(center, egui::Vec2::splat(radius * 2.0))
}

/// The TESTING diameter-circle guides (owner request 2026-11-21): while a
/// diameter gesture is live, the overlay draws WHAT IS MEASURED on top of
/// the shape — a thin DASHED chord between the two drag points and a small
/// dot at the segment's midpoint (the circle's center; the white-core +
/// dark-ring marker style of the polyline's vertex handles, visible on any
/// canvas color) — so the rule reads while dragging. Overlay-only: the
/// commit rasterizes just the ellipse, the guides die with the gesture.
/// Screen-space (the overlay's coordinate system); pure — no egui state, no
/// `App` — so the geometry stays unit-testable.
fn diameter_guides(a: Pos2, b: Pos2, color: Color32) -> Vec<Shape> {
    let mut shapes = Shape::dashed_line(&[a, b], Stroke::new(1.0, color), 4.0, 3.0);
    let center = Pos2::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
    shapes.push(Shape::circle_filled(center, 2.5, Color32::WHITE));
    shapes.push(Shape::circle_stroke(
        center,
        2.5,
        Stroke::new(1.0, Color32::from_black_alpha(180)),
    ));
    shapes
}

impl App {
    /// Press/drag/release and click handling for the Line and Shape tools.
    pub(super) fn handle_preview_input(
        &mut self,
        ui: &egui::Ui,
        rect: Rect,
        response: &egui::Response,
        panning: bool,
    ) {
        self.cancel_stale_preview();

        let (cursor, primary_pressed, secondary_pressed, primary_double, any_released) =
            ui.input(|i| {
                (
                    i.pointer.latest_pos(),
                    i.pointer.button_pressed(PointerButton::Primary),
                    i.pointer.button_pressed(PointerButton::Secondary),
                    i.pointer.button_double_clicked(PointerButton::Primary),
                    i.pointer.any_released(),
                )
            });
        let (escape, enter, backspace, shift) = ui.input(|i| {
            (
                i.key_pressed(Key::Escape),
                i.key_pressed(Key::Enter),
                i.key_pressed(Key::Backspace),
                i.modifiers.shift,
            )
        });
        let pos = cursor.map(|c| self.canvas_pos(rect, c));

        // Global finish/cancel keys work while a polyline is pending, even
        // when the pointer left the canvas.
        if let PreviewState::Polyline { .. } = self.preview {
            if escape {
                self.preview = PreviewState::None;
                return;
            }
            if enter || primary_double || secondary_pressed {
                self.finish_polyline();
                return;
            }
        }

        match self.preview {
            PreviewState::None => {
                if panning || !response.hovered() {
                    return;
                }
                let button = if primary_pressed {
                    Some(PointerButton::Primary)
                } else if secondary_pressed {
                    Some(PointerButton::Secondary)
                } else {
                    None
                };
                let Some((button, _)) = button.zip(cursor) else {
                    return;
                };
                let Some(pos) = pos else {
                    return;
                };
                let width = self.tools.size;
                match self.tools.tool {
                    Tool::Line => {
                        let color = if button == PointerButton::Primary {
                            self.tools.primary
                        } else {
                            self.tools.secondary
                        };
                        match self.tools.line_mode {
                            LineMode::Straight => {
                                self.preview = PreviewState::Drag {
                                    kind: DragKind::Line,
                                    start: pos,
                                    current: pos,
                                    outline: Some(color),
                                    fill: None,
                                    width,
                                    outline_pos: ShapeOutline::Center,
                                    diameter: false,
                                    clip: self.selection_clip(),
                                };
                                // Blocks Space-pan while the drag is in
                                // progress, same as a freehand stroke.
                                self.is_stroking = true;
                            }
                            LineMode::Curve => {
                                self.preview = PreviewState::Polyline {
                                    points: vec![pos],
                                    rubber: Some(pos),
                                    color,
                                    width,
                                    mode: LineMode::Curve,
                                    clip: self.selection_clip(),
                                };
                            }
                        }
                    }
                    Tool::Shape => {
                        // Primary paints the outline, secondary the fill
                        // (owner request 2026-11-10); a RIGHT-button gesture
                        // uses the two wells INVERTED without swapping them
                        // (owner request 2026-11-15). Exactly one paint is
                        // ever active (the panel's radio), so a color always
                        // reaches the gesture — the pure
                        // [`shape_gesture_wells`].
                        let paint = self.tools.shape_paint;
                        let (outline, fill) = shape_gesture_wells(
                            button,
                            paint.outlines(),
                            paint.fills(),
                            self.tools.primary,
                            self.tools.secondary,
                        );
                        self.preview = PreviewState::Drag {
                            kind: DragKind::Shape(self.tools.shape_kind),
                            start: pos,
                            current: pos,
                            outline,
                            fill,
                            width,
                            outline_pos: self.tools.shape_outline,
                            diameter: false,
                            clip: self.selection_clip(),
                        };
                        self.is_stroking = true;
                    }
                    _ => {}
                }
            }
            PreviewState::Drag { .. } => {
                if escape {
                    // Esc cancels an in-progress drag (like Paint.NET's
                    // shape tools); nothing was committed, so no snapshot.
                    self.preview = PreviewState::None;
                    self.is_stroking = false;
                    return;
                }
                if let Some(pos) = pos
                    && let PreviewState::Drag {
                        kind,
                        start,
                        current,
                        diameter,
                        ..
                    } = &mut self.preview
                {
                    // Shift held mid-drag: a Shape grows proportionally
                    // (square/circle), live — the free pointer point is
                    // pulled onto the dominant-axis square each frame. The
                    // constrained point is STORED, so both the overlay
                    // preview and the commit rasterize the same geometry.
                    // TESTING (owner request 2026-11-21): with the Ellipse
                    // kind and the panel's diameter-circle checkbox on, the
                    // Shift rule is the DIAMETER circle instead — `current`
                    // stays the raw pointer position (both drag points are
                    // the circle's diameter endpoints) and the flag marks
                    // the geometry the overlay and the commit must convert.
                    // The flag is refreshed live like the point, so the
                    // release commits exactly what the preview showed.
                    let diameter_now = shift
                        && matches!(kind, DragKind::Shape(ShapeKind::Ellipse))
                        && self.tools.shape_diameter_circle;
                    *current = if shift && matches!(kind, DragKind::Shape(_)) && !diameter_now {
                        proportional_current(*start, pos)
                    } else {
                        pos
                    };
                    *diameter = diameter_now;
                }
                if any_released {
                    let state = std::mem::take(&mut self.preview);
                    if let PreviewState::Drag {
                        kind,
                        start,
                        current,
                        outline,
                        fill,
                        width,
                        outline_pos,
                        diameter,
                        clip,
                    } = state
                    {
                        self.commit_drag(
                            kind,
                            start,
                            current,
                            outline,
                            fill,
                            width,
                            outline_pos,
                            diameter,
                            clip.as_ref(),
                        );
                    }
                    self.is_stroking = false;
                }
            }
            PreviewState::Polyline { .. } => {
                if panning {
                    return; // a Space/middle drag pans; clicks are ignored
                }
                if primary_pressed
                    && let Some(pos) = pos
                    && let PreviewState::Polyline { points, rubber, .. } = &mut self.preview
                {
                    points.push(pos);
                    *rubber = Some(pos);
                }
                if let Some(pos) = pos
                    && let PreviewState::Polyline { rubber, .. } = &mut self.preview
                {
                    *rubber = Some(pos);
                }
                if backspace && let PreviewState::Polyline { points, .. } = &mut self.preview {
                    points.pop();
                    if points.is_empty() {
                        self.preview = PreviewState::None;
                    }
                }
            }
        }
    }

    /// Drops pending previews that no longer match the active tool or the
    /// Line mode (e.g. the tool was switched mid-gesture).
    fn cancel_stale_preview(&mut self) {
        match self.preview {
            PreviewState::Drag { kind, .. } => {
                let stale = match kind {
                    DragKind::Line => self.tools.tool != Tool::Line,
                    DragKind::Shape(_) => self.tools.tool != Tool::Shape,
                };
                if stale {
                    self.preview = PreviewState::None;
                    self.is_stroking = false;
                }
            }
            PreviewState::Polyline { mode, .. } => {
                if self.tools.tool != Tool::Line || self.tools.line_mode != mode {
                    self.preview = PreviewState::None;
                }
            }
            PreviewState::None => {}
        }
    }

    /// Commits a finished drag: straight line, rectangle or ellipse. The
    /// committed pixels land on the ACTIVE layer and the post-commit
    /// buffer becomes the node's PNG snapshot (owner request 2026-11-13).
    #[allow(clippy::too_many_arguments)] // the flat paint list the Drag state carries (same as layers_window.rs)
    fn commit_drag(
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
    fn finish_polyline(&mut self) {
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

    /// The translucent on-canvas preview drawn on top of the canvas texture
    /// while a gesture is pending (never touches the `Pixmap`).
    pub(super) fn draw_preview_overlay(&self, painter: &egui::Painter, canvas_rect: Rect) {
        let ppp = self.ctx.pixels_per_point();
        // The live preview follows the selection too (owner request
        // 2026-11-21): the commit will never paint outside the selected
        // region, so the overlay must not show paint there either. The
        // CACHED tight bounds clip pixel-exact for rect selections; for a
        // lasso they clip to the bounding rect (a superset — the commit
        // still clips exactly to the mask).
        let painter = match self.selection.bounds() {
            Some(b) => {
                let min = self.screen_pos(canvas_rect, Pos2::new(b.x as f32, b.y as f32));
                let max = self.screen_pos(
                    canvas_rect,
                    Pos2::new((b.x + b.w as i64) as f32, (b.y + b.h as i64) as f32),
                );
                painter
                    .clone()
                    .with_clip_rect(painter.clip_rect().intersect(Rect::from_two_pos(min, max)))
            }
            None => painter.clone(),
        };
        let screen_stroke =
            |width: f32| Stroke::new((width * self.zoom / ppp).max(1.0), Color32::TRANSPARENT);
        let to_screen = |p: Pos2| self.screen_pos(canvas_rect, p);

        match &self.preview {
            PreviewState::None => {}
            PreviewState::Drag {
                kind,
                start,
                current,
                outline,
                fill,
                width,
                outline_pos,
                diameter,
                ..
            } => {
                // TESTING (owner request 2026-11-21): the diameter-circle
                // gesture previews the COMMITTED bounding square — the
                // circle through both raw drag points — not the raw
                // corner-to-corner rect.
                let (a, b) = if *diameter {
                    let rect = diameter_rect(*start, *current);
                    (to_screen(rect.min), to_screen(rect.max))
                } else {
                    (to_screen(*start), to_screen(*current))
                };
                let stroke = outline
                    .map(|c| {
                        let mut s = screen_stroke(*width);
                        s.color = c;
                        s
                    })
                    .unwrap_or(Stroke::NONE);
                match kind {
                    DragKind::Line => {
                        // Round caps match the committed raster (LineCap::Round);
                        // egui's own line ends are flat (owner request 2026-11-10).
                        painter.add(Shape::line_segment([a, b], stroke));
                        painter.circle_filled(a, stroke.width * 0.5, stroke.color);
                        painter.circle_filled(b, stroke.width * 0.5, stroke.color);
                    }
                    DragKind::Shape(shape_kind) => {
                        let rect = Rect::from_two_pos(a, b);
                        match shape_kind {
                            ShapeKind::Rectangle => {
                                if let Some(fill) = fill {
                                    painter.rect_filled(rect, 0.0, *fill);
                                }
                                if stroke.width > 0.0 {
                                    // The stroke KIND carries the outline
                                    // placement (owner request 2026-11-20):
                                    // outside = fully beyond the two points,
                                    // inside = fully within, centered = the
                                    // classic straddle — the same bands the
                                    // commit's shifted stroke paths draw.
                                    let kind = match outline_pos {
                                        ShapeOutline::Outside => StrokeKind::Outside,
                                        ShapeOutline::Center => StrokeKind::Middle,
                                        ShapeOutline::Inside => StrokeKind::Inside,
                                    };
                                    painter.rect_stroke(rect, 0.0, stroke, kind);
                                }
                            }
                            ShapeKind::Ellipse => {
                                // egui's EllipseShape strokes OUTSIDE the path,
                                // while the commit (tiny-skia) strokes the oval
                                // CENTERED — a closed PathShape with a plain
                                // (Middle-kind) stroke is centered, matching the
                                // commit (owner request 2026-11-10). For the
                                // outline PLACEMENT modes (owner request
                                // 2026-11-20) the commit shifts the stroked oval
                                // by half a stroke width (`tools.rs
                                // ::outline_stroke_bounds`); egui strokes a path
                                // CENTERED, so the preview shifts the SAMPLED
                                // radii by the same half width and keeps the
                                // stroke centered — the band lands where the
                                // commit's does.
                                if rect.width() < 0.5 || rect.height() < 0.5 {
                                    return; // degenerate click; commit draws nothing
                                }
                                let center = rect.center();
                                let (rx, ry) = (rect.width() * 0.5, rect.height() * 0.5);
                                let (rx, ry) = match outline_pos {
                                    ShapeOutline::Center => (rx, ry),
                                    ShapeOutline::Outside => {
                                        (rx + stroke.width * 0.5, ry + stroke.width * 0.5)
                                    }
                                    ShapeOutline::Inside => (
                                        (rx - stroke.width * 0.5).max(0.25),
                                        (ry - stroke.width * 0.5).max(0.25),
                                    ),
                                };
                                // ~3 pt chords keep the polygon smooth at any zoom.
                                let n = ((rx.max(ry) * TAU / 3.0) as usize).clamp(24, 256);
                                let points: Vec<Pos2> = (0..n)
                                    .map(|i| {
                                        let t = TAU * i as f32 / n as f32;
                                        pos2(center.x + rx * t.cos(), center.y + ry * t.sin())
                                    })
                                    .collect();
                                painter.add(Shape::Path(egui::epaint::PathShape {
                                    points,
                                    closed: true,
                                    fill: fill.unwrap_or(Color32::TRANSPARENT),
                                    stroke: stroke.into(),
                                }));
                            }
                        }
                    }
                }
                // TESTING (owner request 2026-11-21): while the diameter
                // rule shapes the gesture, the overlay also draws WHAT IS
                // MEASURED — the dashed chord between the two raw drag
                // points and the center dot at the segment's midpoint —
                // on top of the shape (only diameter gestures ever carry
                // the flag, so Line never draws guides).
                if *diameter {
                    let guide_color = outline.unwrap_or(fill.unwrap_or(Color32::GRAY));
                    painter.extend(diameter_guides(
                        to_screen(*start),
                        to_screen(*current),
                        guide_color,
                    ));
                }
            }
            PreviewState::Polyline {
                points,
                rubber,
                color,
                width,
                ..
            } => {
                let mut stroke = screen_stroke(*width);
                stroke.color = *color;
                let mut pts: Vec<Pos2> = points.iter().copied().map(to_screen).collect();
                if let Some(rubber) = rubber {
                    pts.push(to_screen(*rubber));
                }
                if pts.len() >= 2 {
                    painter.add(Shape::line(pts.clone(), stroke));
                    // Round caps at both ends, matching the committed
                    // raster (LineCap::Round).
                    painter.circle_filled(pts[0], stroke.width * 0.5, stroke.color);
                    let last = pts[pts.len() - 1];
                    painter.circle_filled(last, stroke.width * 0.5, stroke.color);
                }
                // Vertex handles: small white dots, visible on any color.
                for p in points {
                    let s = to_screen(*p);
                    painter.circle_filled(s, 3.0, Color32::WHITE);
                    painter.circle_stroke(s, 3.0, Stroke::new(1.0, Color32::from_black_alpha(180)));
                }
            }
        }
    }

    /// Cancels any pending preview gesture (tool switch, canvas reset…).
    pub(super) fn cancel_preview(&mut self) {
        self.preview = PreviewState::None;
        self.is_stroking = false;
    }
}

#[cfg(test)]
mod tests;
